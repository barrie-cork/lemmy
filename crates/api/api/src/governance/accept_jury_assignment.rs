//! `POST /api/v4/governance/jury/accept` — a selected juror accepts
//! their assignment, transitioning `jury_assignment.status`
//! Selected → Accepted.
//!
//! `CaseStatus` is matched exhaustively per ADR-013 (no `_ =>`).
//!
//! v0 conflict checks (per `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`
//! Phase 5c task 64):
//!
//! 1. Caller must have a `jury_assignment` row with `status = Selected`
//!    for the case.
//! 2. Caller must NOT be the case creator (v0 "first reporter" proxy —
//!    see Phase 5c task 64 GOTCHA; there is no separate `case_report`
//!    table in v0 per ADR-013).
//! 3. Caller must NOT share an active sponsor with the case's
//!    `target_person_id` (one-hop sponsor-cluster rule; v1 may extend
//!    to two-hop / endorsement clusters per OQ-008).
//!
//! v1-JM-e role-dispatch: Original-role jurors accept while the case is
//! in JurySelection or InReview (the v0 lifecycle); Appeal-role jurors
//! (seated by `seat_appeal_panel` per JM-d) accept while the case is in
//! Appealed (the appeal-panel lifecycle). This mirrors the role-dispatch
//! JM-c added to `submit_jury_vote`.
//!
//! All checks run inside one `run_transaction` with the status flip and
//! governance-log append, so a failed conflict check leaves no
//! half-written state.

use crate::governance::{
  actor_pseudonym_helper,
  governance_log::{self, ENTRY_KIND_JURY_ACCEPTED},
  jury_common::shares_active_sponsor,
};
use actix_web::web::{Data, Json};
use chrono::Utc;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, update};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_common::governance::{AcceptJuryAssignment, AcceptJuryAssignmentResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, JuryAssignmentRole, JuryAssignmentStatus},
  schema::{jury_assignment, moderation_case},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;

pub async fn accept_jury_assignment(
  Json(data): Json<AcceptJuryAssignment>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AcceptJuryAssignmentResponse>> {
  check_local_user_valid(&local_user_view)?;

  let caller_id = local_user_view.person.id;
  let caller_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let outcome = conn
    .run_transaction(|conn| {
      async move { process_accept(conn, caller_id, caller_pseudonym, data).await }.scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}

async fn process_accept(
  conn: &mut diesel_async::AsyncPgConnection,
  caller_id: PersonId,
  caller_pseudonym: String,
  data: AcceptJuryAssignment,
) -> LemmyResult<AcceptJuryAssignmentResponse> {
  // 1. Verify assignment exists with status = Selected; capture role for the
  //    role-aware case-status guard in step 3. JM-d shipped Appeal-role
  //    assignments seated post-`request_appeal` (case status = Appealed);
  //    the role-dispatch here mirrors the dispatch in `submit_jury_vote`
  //    (Appeal-role jurors take a different lifecycle than Original).
  let role: JuryAssignmentRole = jury_assignment::table
    .filter(jury_assignment::case_id.eq(data.case_id))
    .filter(jury_assignment::person_id.eq(caller_id))
    .filter(jury_assignment::status.eq(JuryAssignmentStatus::Selected))
    .select(jury_assignment::role)
    .first(conn)
    .await
    .map_err(|_e| LemmyErrorType::NotFound)?;

  // 2. Load case for conflict checks.
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(data.case_id))
    .select(ModerationCase::as_select())
    .first(conn)
    .await?;

  // 3. Guard: accept is only valid while the case is in a status the role
  //    expects. Original-role jurors accept while the case is in
  //    JurySelection or InReview (the original-jury lifecycle, per Phase 5c
  //    task 64). Appeal-role jurors accept while the case is in Appealed
  //    (the appeal-jury lifecycle introduced by JM-d's `seat_appeal_panel`,
  //    mirroring the role-dispatch JM-c added to `submit_jury_vote`).
  //    All other (role, status) combinations — including EmergencyRemove
  //    per ADR-013 — return NotFound so callers cannot infer internal state.
  //    Both arms enumerate CaseStatus exhaustively per ADR-013 (no `_ =>`).
  match role {
    JuryAssignmentRole::Original => match case.status {
      CaseStatus::JurySelection | CaseStatus::InReview => {}
      CaseStatus::Open
      | CaseStatus::ThresholdMet
      | CaseStatus::Decided
      | CaseStatus::Appealed
      | CaseStatus::Closed
      | CaseStatus::EmergencyRemove
      | CaseStatus::AdminReview
      // PRD §3.3 + ADR-013: original-jury cases are pre-Decided;
      // sponsor-liability lifecycle is post-Decided.
      | CaseStatus::SponsorLiabilityPending
      | CaseStatus::SponsorLiabilityFired
      | CaseStatus::SponsorLiabilityEscaped => {
        return Err(LemmyErrorType::NotFound.into());
      }
    },
    JuryAssignmentRole::Appeal => match case.status {
      CaseStatus::Appealed => {}
      CaseStatus::Open
      | CaseStatus::JurySelection
      | CaseStatus::InReview
      | CaseStatus::ThresholdMet
      | CaseStatus::Decided
      | CaseStatus::Closed
      | CaseStatus::EmergencyRemove
      | CaseStatus::AdminReview
      // PRD §3.3 + ADR-013: appeal-jury seated only on Appealed cases;
      // sponsor-liability lifecycle is post-Decided.
      | CaseStatus::SponsorLiabilityPending
      | CaseStatus::SponsorLiabilityFired
      | CaseStatus::SponsorLiabilityEscaped => {
        return Err(LemmyErrorType::NotFound.into());
      }
    },
  }

  // 4. Conflict 1: caller is not the case creator (v0 "first reporter"
  //    proxy; mask as 404 per the pseudo-403 convention used across
  //    governance handlers).
  if case.creator_id == Some(caller_id) {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 5. Conflict 2: caller NOT in target's active-sponsor cluster.
  //    Skipped when the case has no person target (Post/Comment-targeted
  //    cases have target_person_id = None).
  if let Some(target_id) = case.target_person_id
    && shares_active_sponsor(conn, caller_id, target_id).await?
  {
    return Err(LemmyErrorType::NotFound.into());
  }

  // 6. Flip status → Accepted; stamp responded_at.
  //    The UPDATE predicate includes `status = Selected` to guard against a
  //    concurrent `decline_jury_assignment` that could flip the same row to
  //    `Declined` between our SELECT (step 1) and this UPDATE. Under READ
  //    COMMITTED, the other tx's commit is visible here even inside our own
  //    transaction. If rows_affected == 0, the row was concurrently flipped;
  //    return NotFound so the caller sees a clean "assignment gone" response.
  let now = Utc::now();
  let rows_affected = update(
    jury_assignment::table
      .filter(jury_assignment::case_id.eq(data.case_id))
      .filter(jury_assignment::person_id.eq(caller_id))
      .filter(jury_assignment::status.eq(JuryAssignmentStatus::Selected)),
  )
  .set((
    jury_assignment::status.eq(JuryAssignmentStatus::Accepted),
    jury_assignment::responded_at.eq(Some(now)),
  ))
  .execute(conn)
  .await?;

  if rows_affected == 0 {
    tracing::warn!(
      case_id = %data.case_id.0,
      "accept_jury_assignment: UPDATE matched 0 rows — assignment was concurrently modified (likely declined); returning NotFound"
    );
    return Err(LemmyErrorType::NotFound.into());
  }

  // 7. Audit log — only reached when the UPDATE succeeded (rows_affected == 1).
  governance_log::append(
    &mut (&mut *conn).into(),
    ENTRY_KIND_JURY_ACCEPTED,
    json!({
      "case_id": data.case_id.0,
      "juror_pseudonym": &caller_pseudonym,
    }),
    Some(caller_pseudonym),
  )
  .await?;

  Ok(AcceptJuryAssignmentResponse {
    case_id: data.case_id,
    accepted: true,
  })
}
