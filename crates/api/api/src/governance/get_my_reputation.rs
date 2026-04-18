//! `GET /api/v4/governance/reputation/me` — return the calling user's own
//! reputation summary in the optional community context.
//!
//! Read-only handler — emits NO governance_log entry. Mirror of the
//! existing read-only handlers `get_case` and `list_my_jury_queue` in
//! shape; differs only in that the snapshot is computed on demand via
//! `load_or_compute_snapshot` (so a first-time caller still receives a
//! populated row rather than `404`).
//!
//! Per ADR-005 the four raw dimension scores
//! (`reporting_accuracy`, `jury_reliability`,
//! `participation_consistency`, `endorsement_strength`) are
//! `#[serde(skip)]` on `ReputationSummaryView` itself, so the wire shape
//! exposes capabilities (`jury_eligible`, `trusted_reporter`) and counts
//! (`active_sanctions`) only — no leak even via accidental
//! pass-through.

use crate::governance::{config::ConfigCache, reputation_snapshot::load_or_compute_snapshot};
use actix_web::web::{Data, Json, Query};
use lemmy_api_common::governance::{GetMyReputation, GetMyReputationResponse};
use lemmy_api_utils::{context::LemmyContext, utils::check_local_user_valid};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_db_views_reputation::{ReputationSummaryView, impls::count_active_sanctions};
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;

pub async fn get_my_reputation(
  Query(data): Query<GetMyReputation>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<GetMyReputationResponse>> {
  check_local_user_valid(&local_user_view)?;

  let person_id = local_user_view.person.id;
  let mut cache = ConfigCache::new();

  let mut pool = context.pool();
  let conn = &mut get_conn(&mut pool).await?;

  let snapshot = load_or_compute_snapshot(conn, person_id, data.community_id, &mut cache).await?;
  let active_sanctions = count_active_sanctions(conn, person_id).await?;

  let view = ReputationSummaryView {
    active_sanctions,
    ..ReputationSummaryView::from(&snapshot)
  };

  Ok(Json(GetMyReputationResponse { view }))
}
