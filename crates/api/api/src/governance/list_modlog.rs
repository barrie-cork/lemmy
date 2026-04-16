//! `GET /api/v4/governance/modlog` — public redacted case log.
//!
//! Transparency endpoint per [04 §7]. No auth required; the caller's
//! `Option<LocalUserView>` is accepted so the extractor matches the
//! scope wrapper but its value is irrelevant to the response.
//!
//! v0 pagination is Rust-side via `.skip()` + `.take()` on the full
//! result set per plan §Design Decision: Pagination in Modlog Handler.
//! The Phase 2 query `list_public_case_log` does not yet accept
//! pagination params; Phase 5 or v1 should push the page + limit into
//! the Diesel query for efficiency. In the interim, the governance
//! modlog is expected to contain <100 entries so the cost is trivial.
//!
//! Defaults: `page = 1`, `limit = 20`, max limit clamped to 50.

use actix_web::web::{Data, Json, Query};
use lemmy_api_common::governance::ListGovernanceModlog;
use lemmy_api_utils::context::LemmyContext;
use lemmy_db_views_governance_modlog::{
  GovernanceModlogView,
  impls::{list_public_case_log, list_public_case_log_for_community},
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::LemmyResult;

const DEFAULT_PAGE: i64 = 1;
const DEFAULT_LIMIT: i64 = 20;
const MAX_LIMIT: i64 = 50;

pub async fn list_modlog(
  Query(data): Query<ListGovernanceModlog>,
  context: Data<LemmyContext>,
  _local_user_view: Option<LocalUserView>,
) -> LemmyResult<Json<Vec<GovernanceModlogView>>> {
  let rows = match data.community_id {
    Some(community_id) => {
      list_public_case_log_for_community(&mut context.pool(), community_id).await?
    }
    None => list_public_case_log(&mut context.pool()).await?,
  };

  let page = data.page.unwrap_or(DEFAULT_PAGE).max(1);
  let limit = data.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
  let skip = usize::try_from((page - 1).saturating_mul(limit)).unwrap_or(0);
  let take = usize::try_from(limit).unwrap_or(0);

  let paged: Vec<GovernanceModlogView> = rows.into_iter().skip(skip).take(take).collect();
  Ok(Json(paged))
}
