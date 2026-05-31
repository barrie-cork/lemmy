//! `GET /api/v4/governance/admin/reputation/rollup` — admin-only
//! instance-wide reputation rollup query.
//!
//! Returns the materialised rollup snapshot (`community_id IS NULL`) for a
//! given person, plus all per-community contributing snapshots. The rollup
//! row is `None` if the weekly cron has not yet run for this person, or if
//! every community they have a snapshot in is banned from.
//!
//! Read-only; emits NO governance_log entry (read path, per the same
//! convention as `admin_reputation_stats.rs`).

use actix_web::web::{Data, Json, Query};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_api_common::governance::{AdminReputationRollup, AdminReputationRollupResponse};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot;
use lemmy_db_schema_file::schema::reputation_snapshot;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;

pub async fn admin_reputation_rollup(
  Query(data): Query<AdminReputationRollup>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminReputationRollupResponse>> {
  is_admin(&local_user_view)?;

  let mut pool = context.pool();
  let conn = &mut get_conn(&mut pool).await?;
  let person_id = data.person_id;

  // Load the instance-wide rollup row (community_id IS NULL).
  let rollup: Option<ReputationSnapshot> = reputation_snapshot::table
    .filter(reputation_snapshot::person_id.eq(person_id))
    .filter(reputation_snapshot::community_id.is_null())
    .first::<ReputationSnapshot>(conn)
    .await
    .optional()?;

  // Load per-community contributing snapshots (community_id IS NOT NULL).
  let contributing: Vec<ReputationSnapshot> = reputation_snapshot::table
    .filter(reputation_snapshot::person_id.eq(person_id))
    .filter(reputation_snapshot::community_id.is_not_null())
    .load::<ReputationSnapshot>(conn)
    .await?;

  Ok(Json(AdminReputationRollupResponse {
    rollup,
    contributing,
  }))
}
