//! `POST /api/v4/governance/jury/decline` — a selected or accepted juror
//! declines their assignment; the handler picks a replacement juror if the
//! eligible pool is non-empty.
//!
//! Allowed source statuses: `Selected` or `Accepted`. The row flips to
//! `Declined` and a single replacement is picked via
//! [`admin_assign_jury::select_eligible_jurors`] with the current active
//! assignees in the exclude list. If the pool is exhausted the case
//! continues with one fewer juror — quorum=3 of 5 tolerates up to two
//! unreplaced declines per plan §11.5 GOTCHA.

use crate::governance::{
  actor_pseudonym_helper,
  admin_assign_jury::select_eligible_jurors,
  config::{self, ConfigCache, DEFAULT_JURY_PANEL_SIZE, Scope},
  governance_log::{self, ENTRY_KIND_JURY_DECLINED, ENTRY_KIND_JURY_REPLACEMENT_SELECTED},
};
use actix_web::web::{Data, Json};
use chrono::Utc;
use diesel::{
  BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper, insert_into, update,
};
use diesel_async::RunQueryDsl;
use lemmy_api_common::governance::{DeclineJuryAssignment, DeclineJuryAssignmentResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::source::governance::{
  jury_assignment::{JuryAssignment, JuryAssignmentInsertForm},
  moderation_case::ModerationCase,
};
use lemmy_db_schema_file::{
  PersonId,
  enums::JuryAssignmentStatus,
  schema::{jury_assignment, moderation_case},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

pub async fn decline_jury_assignment(
  Json(data): Json<DeclineJuryAssignment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<DeclineJuryAssignmentResponse>> {
  check_local_user_valid(&local_user_view)?;

  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data.clone();
  let pseudonym_for_tx = caller_pseudonym.clone();

  let replacement = conn
    .run_transaction(async |conn| {
      process_decline(conn, caller_id, pseudonym_for_tx, data_for_tx).await
    })
    .await?;

  Ok(Json(DeclineJuryAssignmentResponse {
    case_id: data.case_id,
    declined: true,
    replacement_person_id: replacement,
  }))
}

async fn process_decline(
  conn: &mut diesel_async::AsyncPgConnection,
  caller_id: PersonId,
  caller_pseudonym: String,
  data: DeclineJuryAssignment,
) -> LemmyResult<Option<PersonId>> {
  let mut cache = ConfigCache::new();

  // 1. Load the assignment row; must be Selected or Accepted.
  let assignment: JuryAssignment = jury_assignment::table
    .filter(jury_assignment::case_id.eq(data.case_id))
    .filter(jury_assignment::person_id.eq(caller_id))
    .filter(
      jury_assignment::status
        .eq(JuryAssignmentStatus::Selected)
        .or(jury_assignment::status.eq(JuryAssignmentStatus::Accepted)),
    )
    .select(JuryAssignment::as_select())
    .first(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;

  // 2. Load the case for the replacement-pick query.
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // 3. Flip status → Declined; stamp responded_at.
  let now = Utc::now();
  update(jury_assignment::table.filter(jury_assignment::id.eq(assignment.id)))
    .set((
      jury_assignment::status.eq(JuryAssignmentStatus::Declined),
      jury_assignment::responded_at.eq(Some(now)),
    ))
    .execute(conn)
    .await?;

  // 4. Log the decline. `reason` rides in the payload and is auto-scrubbed
  //    by `append` (via scrub_json inside `governance_log::append`).
  governance_log::append(
    &mut (&mut *conn).into(),
    ENTRY_KIND_JURY_DECLINED,
    json!({
      "case_id": data.case_id.0,
      "juror_pseudonym": caller_pseudonym,
      "reason_present": data.reason.is_some(),
      "reason": data.reason,
    }),
    Some(caller_pseudonym.clone()),
  )
  .await?;

  // 5. Build the exclude list: all active assignees on this case PLUS
  //    the declining juror. Filtering `jury_assignment` by
  //    `ne(Declined)` excludes the declining juror's row from the
  //    active-assignee query, so without the explicit push below they
  //    would remain eligible and `select_eligible_jurors` could pick
  //    them as their own replacement (GH #33).
  let mut exclude_person_ids: Vec<PersonId> = jury_assignment::table
    .filter(jury_assignment::case_id.eq(data.case_id))
    .filter(jury_assignment::status.ne(JuryAssignmentStatus::Declined))
    .filter(jury_assignment::status.ne(JuryAssignmentStatus::Expired))
    .select(jury_assignment::person_id)
    .load(conn)
    .await?;
  exclude_person_ids.push(caller_id);

  // 6. Try to pick ONE replacement. `select_eligible_jurors` returns up
  //    to panel_size; we take the head only. The case's panel_size
  //    snapshot (v1-JM-a addition) tells us how wide the original panel
  //    was — use that so replacement-selection picks from the same
  //    constraint-profile the original assemble used. Falls back to
  //    bare `jury.panel_size` (v0-compatible) when no snapshot is set
  //    (e.g. pre-JM-a case that got a replacement request).
  let panel_size = match case.panel_size_snapshot {
    Some(n) => i64::from(n),
    None => config::get_int(
      &mut cache,
      &mut (&mut *conn).into(),
      Scope::Instance,
      "jury.panel_size",
    )
    .await
    .unwrap_or(DEFAULT_JURY_PANEL_SIZE),
  };
  let (replacements, record) = select_eligible_jurors(
    conn,
    &case,
    panel_size,
    Some(&exclude_person_ids),
    &mut cache,
  )
  .await?;
  let replacement_id = replacements.into_iter().next();

  // 7. If a replacement exists, insert a Selected row + emit a
  //    replacement_selected log entry attributed to the declining juror.
  if let Some(new_id) = replacement_id {
    let form = JuryAssignmentInsertForm {
      case_id: data.case_id,
      person_id: new_id,
      status: JuryAssignmentStatus::Selected,
      selected_under_constraints: Some(record.to_json()),
      ..Default::default()
    };
    insert_into(jury_assignment::table)
      .values(&form)
      .execute(conn)
      .await?;

    let new_pseudonym =
      actor_pseudonym_helper::get_or_create(&mut (&mut *conn).into(), new_id).await?;
    governance_log::append(
      &mut (&mut *conn).into(),
      ENTRY_KIND_JURY_REPLACEMENT_SELECTED,
      json!({
        "case_id": data.case_id.0,
        "new_juror_pseudonym": new_pseudonym,
      }),
      Some(caller_pseudonym.clone()),
    )
    .await?;
  }

  Ok(replacement_id)
}
