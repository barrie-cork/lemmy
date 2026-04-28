//! `POST /api/v4/governance/appeal` — sanction target or original reporter
//! requests an appeal of a decided case.
//!
//! v1-JM-d introduces two caller-eligibility paths:
//! - **Defendant** (`target_person_id`): always eligible while the appeal window is open.
//! - **OriginalReporter** (`creator_id`, light-touch-outcome cases only): eligible when the winning
//!   decision is `NoAction` or `AdvisoryLabel`.
//!
//! Appeal window check uses `appeal_window_expires_at` (computed at
//! `submit_jury_vote.rs` step 9 per JM-c §10.5). The v0 `closed_at` check
//! was a regression corrected in this sub-phase.
//!
//! When `appeal.auto_select_on_appeal_acceptance = true`, the appeal panel is
//! seated immediately (original jurors excluded, larger panel, bumped threshold
//! tier per PRD §6.1-§6.3). Otherwise the case sits in `Appealed` awaiting
//! `admin_trigger_appeal_rejury`.
//!
//! `CaseStatus` is matched exhaustively per ADR-013 (no `_ =>`).

use actix_web::web::{Data, Json};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into, update};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api::governance::{
  actor_pseudonym_helper,
  admin_assign_jury::{seat_appeal_panel, select_appeal_panel},
  config::{self, ConfigCache, Scope},
  governance_log::{self, ENTRY_KIND_APPEAL_REQUESTED},
};
use lemmy_api_common::governance::{RequestAppeal, RequestAppealResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::{
  newtypes::AppealId,
  source::governance::{
    appeal::{Appeal, AppealInsertForm},
    moderation_case::ModerationCase,
  },
};
use lemmy_db_schema_file::{
  PersonId,
  enums::{AppealRequesterRole, AppealStatus, CaseStatus, JuryDecision},
  schema::{appeal, moderation_case},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

pub async fn request_appeal(
  Json(data): Json<RequestAppeal>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<RequestAppealResponse>> {
  check_local_user_valid(&local_user_view)?;

  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data.clone();
  let pseudonym_for_tx = caller_pseudonym.clone();

  let appeal_id = conn
    .run_transaction(|conn| {
      async move { process_appeal(conn, caller_id, pseudonym_for_tx, data_for_tx).await }
        .scope_boxed()
    })
    .await?;

  Ok(Json(RequestAppealResponse {
    appeal_id,
    case_id: data.case_id,
  }))
}

async fn process_appeal(
  conn: &mut diesel_async::AsyncPgConnection,
  caller_id: PersonId,
  caller_pseudonym: String,
  data: RequestAppeal,
) -> LemmyResult<AppealId> {
  // 1. Load the case.
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;

  // 2. Exhaustive match on case.status per ADR-013.
  match case.status {
    CaseStatus::Decided => {}
    CaseStatus::Open
    | CaseStatus::ThresholdMet
    | CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::Appealed
    | CaseStatus::Closed
    | CaseStatus::EmergencyRemove
    | CaseStatus::AdminReview => {
      return Err(LemmyErrorType::NotFound.into());
    }
  }

  // 3. Appeal window: appeal_window_expires_at must be in the future.
  // submit_jury_vote step 9 stamps this at decided-flip time (JM-c §10.5).
  // NULL on a Decided case is treated as an expired window (data error).
  let within_window = case
    .appeal_window_expires_at
    .map(|t| t > Utc::now())
    .unwrap_or(false);
  if !within_window {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 4. Caller eligibility — defendant or original-reporter.
  let requester_role = if case.target_person_id == Some(caller_id) {
    AppealRequesterRole::Defendant
  } else if case.creator_id == Some(caller_id)
    && matches!(
      case.winning_decision,
      Some(JuryDecision::NoAction | JuryDecision::AdvisoryLabel)
    )
  {
    AppealRequesterRole::OriginalReporter
  } else {
    return Err(LemmyErrorType::NotFound.into());
  };

  // 5. Insert the Appeal row. Snapshot fields are NULL until the auto-rejury
  // branch (step 8) stamps them via UPDATE.
  let form = AppealInsertForm {
    case_id: data.case_id,
    requester_id: caller_id,
    reason: data.reason.clone(),
    status: AppealStatus::Requested,
    requester_role: Some(requester_role),
    ..Default::default()
  };
  let new_appeal: Appeal = insert_into(appeal::table)
    .values(&form)
    .returning(Appeal::as_returning())
    .get_result(conn)
    .await?;

  // 6. Flip case Decided → Appealed.
  update(moderation_case::table.filter(moderation_case::id.eq(data.case_id)))
    .set(moderation_case::status.eq(CaseStatus::Appealed))
    .execute(conn)
    .await?;

  // 7. Audit log. `reason` is auto-scrubbed by `append`'s scrub_json layer.
  governance_log::append(
    &mut (&mut *conn).into(),
    ENTRY_KIND_APPEAL_REQUESTED,
    json!({
      "case_id": data.case_id.0,
      "appeal_id": new_appeal.id.0,
      "reason": data.reason,
    }),
    Some(caller_pseudonym.clone()),
  )
  .await?;

  // 8. Auto-rejury branch: seat the appeal panel inside this same transaction
  // so a panel-seating failure rolls back the appeal-row insert as well.
  let mut cache = ConfigCache::new();
  let auto_select = config::get_bool(
    &mut cache,
    &mut (&mut *conn).into(),
    Scope::Instance,
    "appeal.auto_select_on_appeal_acceptance",
  )
  .await?;

  if auto_select {
    let selection = select_appeal_panel(conn, &case, &mut cache).await?;
    seat_appeal_panel(conn, data.case_id, new_appeal.id, &selection, caller_pseudonym.clone())
      .await?;
  }

  Ok(new_appeal.id)
}
