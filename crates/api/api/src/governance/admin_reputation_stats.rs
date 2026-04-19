//! `GET /api/v4/governance/admin/reputation-stats` — admin-only
//! cross-population reputation observability.
//!
//! Per plan §11.2 + IMPLEMENTATION-PLAN-v0.md line 373. Six round-trips
//! total (4 dimension bucket queries + 1 capability-count query +
//! 1 founder-event query) — bucketing via SQL `CASE WHEN` per dimension
//! is cheaper than loading all snapshots into Rust.
//!
//! Read-only; emits NO governance_log entry per advisor decision-queue
//! #25 (Watch 11 scope is governance-weight WRITES, not READS;
//! admin_assign_jury / admin_close_case only log writes, not query
//! invocations). TODO(brehon-fork): audit-log admin queries in v1 per
//! OQ-026-candidate.
//!
//! `community_id` filter on the input narrows bucket queries to
//! `reputation_snapshot.community_id = $1`; when `None` queries run
//! instance-wide on `community_id IS NULL` rows. Thresholds are read
//! from `Scope::Instance` per plan §11.2 (per-community thresholds are
//! post-MVP).
//!
//! Bucket boundaries per probe-62 verification on pg18: `[0, 1-30,
//! 31-80, 81-200, 200+]`.

use crate::governance::config::{ConfigCache, Scope, get_int};
use actix_web::web::{Data, Json, Query};
use chrono::Utc;
use diesel::{
  QueryableByName,
  sql_query,
  sql_types::{BigInt, Nullable, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_common::governance::{
  AdminReputationStats,
  AdminReputationStatsResponse,
  CapabilityCounts,
  FounderEventStats,
  ReputationBuckets,
  ThresholdsSnapshot,
};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::get_conn;
use lemmy_utils::error::LemmyResult;

/// Row shape for the bucket queries. Module-scope per workspace lint
/// `items-after-statements`.
#[derive(QueryableByName)]
struct BucketRow {
  #[diesel(sql_type = Text)]
  bucket: String,
  #[diesel(sql_type = BigInt)]
  count: i64,
}

/// Row shape for the single capability-count query.
#[derive(QueryableByName)]
struct CapabilityRow {
  #[diesel(sql_type = BigInt)]
  jury_eligible_count: i64,
  #[diesel(sql_type = BigInt)]
  trusted_reporter_count: i64,
  #[diesel(sql_type = BigInt)]
  can_sponsor_count: i64,
}

/// Row shape for the single founder-event-stats query.
#[derive(QueryableByName)]
struct FounderRow {
  #[diesel(sql_type = BigInt)]
  active_count: i64,
  #[diesel(sql_type = BigInt)]
  expired_count: i64,
}

pub async fn admin_reputation_stats(
  Query(data): Query<AdminReputationStats>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminReputationStatsResponse>> {
  is_admin(&local_user_view)?;

  let mut cache = ConfigCache::new();
  let mut pool = context.pool();
  let conn = &mut get_conn(&mut pool).await?;

  let community_bind: Option<i32> = data.community_id.map(|c| c.0);

  // Step 1 — read three thresholds from instance-scope config.
  let thresholds_current = ThresholdsSnapshot {
    jury_reliability: get_int(
      &mut cache,
      &mut (&mut *conn).into(),
      Scope::Instance,
      "thresholds.jury_reliability",
    )
    .await?,
    reporting_accuracy: get_int(
      &mut cache,
      &mut (&mut *conn).into(),
      Scope::Instance,
      "thresholds.reporting_accuracy",
    )
    .await?,
    endorsement_strength: get_int(
      &mut cache,
      &mut (&mut *conn).into(),
      Scope::Instance,
      "thresholds.endorsement_strength",
    )
    .await?,
  };

  // Step 2 — four bucket queries, one per dimension.
  let buckets = ReputationBuckets {
    reporting_accuracy: bucket_query(conn, "reporting_accuracy", community_bind).await?,
    jury_reliability: bucket_query(conn, "jury_reliability", community_bind).await?,
    participation_consistency: bucket_query(conn, "participation_consistency", community_bind)
      .await?,
    endorsement_strength: bucket_query(conn, "endorsement_strength", community_bind).await?,
  };

  // Step 3 — capability counts (one round-trip with three FILTER aggregates).
  let capability_counts = capability_query(conn, community_bind).await?;

  // Step 4 — founder-event stats (one round-trip).
  let founder_event_stats = founder_query(conn, community_bind).await?;

  Ok(Json(AdminReputationStatsResponse {
    buckets,
    thresholds_current,
    capability_counts,
    founder_event_stats,
    calculated_at: Utc::now(),
  }))
}

/// Run the `CASE WHEN` bucket query for one dimension column. The column
/// name is interpolated literally into the SQL — the only legal callers
/// are the four hardcoded names above, none of them user-controlled.
async fn bucket_query(
  conn: &mut AsyncPgConnection,
  column: &str,
  community_bind: Option<i32>,
) -> LemmyResult<[i64; 5]> {
  // Construct SQL with the literal column name. Bucket boundaries match
  // probe-62 verification: 0 / 1-30 / 31-80 / 81-200 / 200+.
  let sql = format!(
    "\
       SELECT bucket, COUNT(*)::bigint AS count \
       FROM ( \
         SELECT \
           CASE \
             WHEN {column} = 0 THEN '0' \
             WHEN {column} BETWEEN 1 AND 30 THEN '1-30' \
             WHEN {column} BETWEEN 31 AND 80 THEN '31-80' \
             WHEN {column} BETWEEN 81 AND 200 THEN '81-200' \
             ELSE '200+' \
           END AS bucket \
         FROM reputation_snapshot \
         WHERE community_id IS NOT DISTINCT FROM $1 \
       ) s \
       GROUP BY bucket"
  );

  let rows: Vec<BucketRow> = sql_query(sql)
    .bind::<Nullable<diesel::sql_types::Integer>, _>(community_bind)
    .load(conn)
    .await?;

  // Map labelled rows into the fixed 5-bucket array. Missing buckets stay 0.
  // Workspace clippy denies `indexing_slicing` — use `.get_mut()` so the
  // indexing path is explicitly bounds-checked even though the indices
  // are statically derived from the match below.
  let mut out = [0_i64; 5];
  for row in rows {
    let idx = match row.bucket.as_str() {
      "0" => 0_usize,
      "1-30" => 1,
      "31-80" => 2,
      "81-200" => 3,
      "200+" => 4,
      // Unreachable in practice — the CASE expression covers every i32 — but
      // workspace clippy denies wildcards in production code if we ignore
      // the branch silently. Treat as 200+ bucket fallback.
      _ => 4,
    };
    if let Some(slot) = out.get_mut(idx) {
      *slot = row.count;
    }
  }
  Ok(out)
}

/// Three capability counts in one round-trip via `FILTER` aggregates.
async fn capability_query(
  conn: &mut AsyncPgConnection,
  community_bind: Option<i32>,
) -> LemmyResult<CapabilityCounts> {
  let sql = "\
     SELECT \
       COUNT(*) FILTER (WHERE jury_eligible)::bigint AS jury_eligible_count, \
       COUNT(*) FILTER (WHERE trusted_reporter)::bigint AS trusted_reporter_count, \
       COUNT(*) FILTER (WHERE can_sponsor)::bigint AS can_sponsor_count \
     FROM reputation_snapshot \
     WHERE community_id IS NOT DISTINCT FROM $1";

  let row: CapabilityRow = sql_query(sql)
    .bind::<Nullable<diesel::sql_types::Integer>, _>(community_bind)
    .get_result(conn)
    .await?;

  Ok(CapabilityCounts {
    jury_eligible_count: row.jury_eligible_count,
    trusted_reporter_count: row.trusted_reporter_count,
    can_sponsor_count: row.can_sponsor_count,
  })
}

/// Founder-event stats: count of `reputation_event` rows with non-null
/// `expires_at`, split on whether the expiry is future or past relative
/// to `now()`. Scoped by `community_id` using `IS NOT DISTINCT FROM` to
/// match the convention used by `bucket_query` and `capability_query`.
async fn founder_query(
  conn: &mut AsyncPgConnection,
  community_bind: Option<i32>,
) -> LemmyResult<FounderEventStats> {
  let sql = "\
     SELECT \
       COUNT(*) FILTER (WHERE expires_at > now())::bigint AS active_count, \
       COUNT(*) FILTER (WHERE expires_at <= now())::bigint AS expired_count \
     FROM reputation_event \
     WHERE expires_at IS NOT NULL \
       AND community_id IS NOT DISTINCT FROM $1";

  let row: FounderRow = sql_query(sql)
    .bind::<Nullable<diesel::sql_types::Integer>, _>(community_bind)
    .get_result(conn)
    .await?;

  Ok(FounderEventStats {
    active_count: row.active_count,
    expired_count: row.expired_count,
  })
}
