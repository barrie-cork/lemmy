//! `POST /api/v4/governance/admin/assign-jury` — admin backstop to seat a
//! jury panel without reputation gating.
//!
//! v0 simplification per [99 ADR-007] and [IMPLEMENTATION-PLAN-v0.md §3
//! Phase 4 task 44]: the admin selects five eligible jurors (not target,
//! not reporter) via `ORDER BY random() LIMIT 5`. Assignments are inserted
//! with `status = Accepted` so the juror can immediately vote; Phase 5
//! introduces the proper `Selected → Accepted` flow with opt-in.
//!
//! The case is flipped `Open | ThresholdMet | EmergencyRemove` →
//! `JurySelection`; the plan's shorthand `InPanel` maps to the enum's
//! `JurySelection` variant (decision-queue #7). Per [99 ADR-013] the
//! status match is exhaustive; no `_ =>` catchall.
//!
//! All DB writes run inside one `run_transaction` so the 5 juror inserts +
//! the case update + the 6 log entries land atomically. Partial execution
//! would leave the case in a half-assembled panel state.

use crate::governance::{actor_pseudonym_helper, governance_log};
use actix_web::web::{Data, Json};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into, update};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_common::governance::{AdminAssignJury, AdminAssignJuryResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::{
  jury_assignment::JuryAssignmentInsertForm,
  moderation_case::ModerationCase,
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, JuryAssignmentStatus},
  schema::{jury_assignment, local_user, moderation_case, person},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::{connection::get_conn, utils::functions::random};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

/// Panel size per [99 ADR-007] / [05 §3].
const PANEL_SIZE: i64 = 5;

pub async fn admin_assign_jury(
  Json(data): Json<AdminAssignJury>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminAssignJuryResponse>> {
  is_admin(&local_user_view)?;

  let admin_id = local_user_view.person.id;
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data;
  let pseudonym_for_tx = admin_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move { process_assignment(conn, pseudonym_for_tx, data_for_tx).await }.scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}

/// Body of the `run_transaction` closure. Kept under the workspace
/// `large_futures` lint threshold by matching the submit_jury_vote split.
async fn process_assignment(
  conn: &mut diesel_async::AsyncPgConnection,
  admin_pseudonym: String,
  data: AdminAssignJury,
) -> LemmyResult<AdminAssignJuryResponse> {
  // 1. Read the case for the status guard + exclusion IDs.
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // 2. Exhaustive status match per [99 ADR-013].
  match case.status {
    CaseStatus::Open | CaseStatus::ThresholdMet | CaseStatus::EmergencyRemove => {}
    CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::Decided
    | CaseStatus::Appealed
    | CaseStatus::Closed
    | CaseStatus::AdminReview => return Err(LemmyErrorType::NotFound.into()),
  }

  // 3. Select 5 eligible jurors (not target, not reporter).
  let eligible = select_eligible_jurors(conn, &case).await?;
  if (eligible.len() as i64) < PANEL_SIZE {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 4. Insert 5 JuryAssignment rows with status=Accepted (v0 testability).
  let forms: Vec<JuryAssignmentInsertForm> = eligible
    .iter()
    .map(|person_id| JuryAssignmentInsertForm {
      case_id: data.case_id,
      person_id: *person_id,
      status: JuryAssignmentStatus::Accepted,
    })
    .collect();
  insert_into(jury_assignment::table)
    .values(&forms)
    .execute(conn)
    .await?;

  // 5. Flip case → JurySelection (plan shorthand "InPanel").
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set(moderation_case::status.eq(CaseStatus::JurySelection))
    .execute(conn)
    .await?;

  // 6. One governance_log "jury_assigned" row per juror. Each payload
  //    carries the per-juror pseudonym so the modlog stays pseudonymous
  //    per [99 ADR-015].
  for person_id in &eligible {
    let juror_pseudonym =
      actor_pseudonym_helper::get_or_create(&mut conn.into(), *person_id).await?;
    governance_log::append(
      &mut conn.into(),
      "jury_assigned",
      json!({
        "case_id": data.case_id.0,
        "juror_pseudonym": juror_pseudonym,
      }),
      Some(admin_pseudonym.clone()),
    )
    .await?;
  }

  // 7. Single panel_assembled marker — canonical record of Open→panel.
  governance_log::append(
    &mut conn.into(),
    "panel_assembled",
    json!({
      "case_id": data.case_id.0,
      "juror_count": PANEL_SIZE,
    }),
    Some(admin_pseudonym.clone()),
  )
  .await?;

  Ok(AdminAssignJuryResponse {
    case_id: data.case_id,
    assigned_person_ids: eligible,
  })
}

/// Select up to `PANEL_SIZE` eligible jurors for the case. v0 eligibility
/// is "not target AND not reporter AND not deleted AND accepted_application";
/// reputation gating is Phase 5 per [IMPLEMENTATION-PLAN-v0.md §3 Phase 5].
async fn select_eligible_jurors(
  conn: &mut diesel_async::AsyncPgConnection,
  case: &ModerationCase,
) -> LemmyResult<Vec<PersonId>> {
  let target = case.target_person_id;
  let reporter = case.creator_id;

  let mut query = person::table
    .inner_join(local_user::table)
    .filter(person::deleted.eq(false))
    .filter(local_user::accepted_application.eq(true))
    .into_boxed();
  if let Some(target_id) = target {
    query = query.filter(person::id.ne(target_id));
  }
  if let Some(reporter_id) = reporter {
    query = query.filter(person::id.ne(reporter_id));
  }

  let eligible: Vec<PersonId> = query
    .order(random())
    .limit(PANEL_SIZE)
    .select(person::id)
    .load::<PersonId>(conn)
    .await?;
  Ok(eligible)
}
