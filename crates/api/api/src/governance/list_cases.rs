//! `GET /api/v4/governance/cases` — list governance cases filtered by
//! community and/or status, paginated.
//!
//! v0 scope per [04 §6.2]:
//!
//! - Authenticated — the extractor is `LocalUserView` (not Option). Unauth
//!   callers get a 401 via actix's extractor error bridge.
//! - Filter surface: `community_id`, `status`, `page`, `limit`. The
//!   `target_person_id` / `assignee` filter is v1 scope (the DTO does not
//!   carry it) per plan §11.7 GOTCHA.
//! - No per-community permission filter in v0 — any authenticated user
//!   sees all cases. v1 may restrict to cases from communities the user
//!   belongs to (noted in plan §11.7 GOTCHA).
//! - The `reporter_count` / `jury_needed` drift stubs in
//!   `GovernanceCaseSummaryView` stay stubbed (Phase 2a contract); v1
//!   unstubs them.

use actix_web::web::{Data, Json, Query};
use lemmy_api_common::governance::{ListGovernanceCases, ListGovernanceCasesResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_views_governance_case::impls::{CasesFilter, list_cases_filtered};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

pub async fn list_cases(
  Query(data): Query<ListGovernanceCases>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<ListGovernanceCasesResponse>> {
  check_local_user_valid(&local_user_view)?;

  let filter = CasesFilter {
    community_id: data.community_id,
    status: data.status,
    target_person_id: None,
    page: data.page,
    limit: data.limit,
  };

  let cases = list_cases_filtered(&mut context.pool(), filter).await?;

  Ok(Json(ListGovernanceCasesResponse { cases }))
}
