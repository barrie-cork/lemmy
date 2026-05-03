//! `GET /api/v4/governance/case?case_id=N` — read a single case.
//!
//! v0 permission model per [04 §6.2] and the plan §Task 5 decision:
//!
//! - Unauthenticated callers see `Decided` / `Closed` / `Appealed` cases
//!   only. Any other status returns `NotFound` (not a silent empty) so
//!   anonymous probing can't distinguish "case doesn't exist" from "case
//!   exists but is private".
//! - Authenticated callers see every status — but if the underlying row
//!   is `EmergencyRemove`, the response is preserved as-is (the view
//!   intentionally surfaces the removal so admins and jurors can audit
//!   post-facto, per [ADR-013]).
//! - `AdminReview` cases are always visible to authenticated users;
//!   unauthenticated callers get `NotFound`.
//!
//! Exhaustive match on `CaseStatus` is mandatory per [ADR-013] — no
//! `_ => ...` arm. When a new variant is added upstream, every call site
//! must update.

use actix_web::web::{Data, Json, Query};
use lemmy_api_common::governance::GetGovernanceCase;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_schema_file::enums::CaseStatus;
use lemmy_db_views_governance_case::{GovernanceCaseDetailView, impls::read_case_detail};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

pub async fn get_case(
  Query(data): Query<GetGovernanceCase>,
  context: Data<LemmyContext>,
  local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<GovernanceCaseDetailView>> {
  let view = read_case_detail(&mut context.pool(), data.case_id).await?;

  if local_user_view.is_none() && !is_public_status(view.row.case_row.status) {
    return Err(LemmyErrorType::NotFound.into());
  }

  Ok(Json(view))
}

/// True iff the given status may be read by an unauthenticated caller.
///
/// Exhaustive match — adding a new `CaseStatus` variant upstream forces
/// this function to be updated, per [ADR-013].
fn is_public_status(status: CaseStatus) -> bool {
  match status {
    CaseStatus::Decided
    | CaseStatus::Closed
    | CaseStatus::Appealed
    // PRD §3.3 + ADR-013: SL states are post-Decided outcomes — publicly visible
    // the same as Decided/Closed (unauthenticated callers may read case details).
    | CaseStatus::SponsorLiabilityPending
    | CaseStatus::SponsorLiabilityFired
    | CaseStatus::SponsorLiabilityEscaped => true,
    CaseStatus::Open
    | CaseStatus::ThresholdMet
    | CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::EmergencyRemove
    | CaseStatus::AdminReview => false,
  }
}
