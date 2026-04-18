//! `POST /api/v4/governance/appeal` — sanction target requests an appeal
//! of a decided case.
//!
//! v0 scope per IMPLEMENTATION-PLAN-v0.md line 381 and ADR-010:
//!
//! - Only the case `target_person_id` can appeal. Original-reporter appeals
//!   are deferred to v1.
//! - No automatic re-jury. The case flips `Decided → Appealed` and sits
//!   until `admin_close_case` runs. The larger-jury appeal flow is a v1
//!   item per ADR-010.
//! - Appeal window is implicit: while `case.closed_at IS NULL`. Once
//!   `admin_close_case` stamps `closed_at`, further appeals on the case
//!   are rejected.
//!
//! `CaseStatus` is matched exhaustively per ADR-013 (no `_ =>`).

use actix_web::web::{Data, Json};
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, insert_into, update};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api::governance::{
  actor_pseudonym_helper,
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
  enums::{AppealStatus, CaseStatus},
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
      async move {
        process_appeal(conn, caller_id, pseudonym_for_tx, data_for_tx).await
      }
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

  // 3. Appeal window: closed_at must still be null.
  if case.closed_at.is_some() {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 4. Caller must be the sanction target.
  if case.target_person_id != Some(caller_id) {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 5. Insert the Appeal row.
  let form = AppealInsertForm {
    case_id: data.case_id,
    requester_id: caller_id,
    reason: data.reason.clone(),
    status: AppealStatus::Requested,
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

  Ok(new_appeal.id)
}
