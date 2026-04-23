//! `GET /api/v4/governance/admin/dashboard` — single-round-trip
//! instance-wide aggregate for the admin governance dashboard.
//!
//! Read-only; emits NO `governance_log` entry (ADR-008 signed-log
//! compliance: dashboard viewing is not a governance write).

use crate::governance::{
  admin_reputation_stats::{bucket_query, capability_query, founder_query},
  audit_projection::project_to_audit_entry,
  config::{ConfigCache, Scope, get_int, get_int_opt},
  governance_log::{ENTRY_KIND_ADMIN_CONFIG_CHANGED, ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED},
};
use actix_web::web::{Data, Json};
use chrono::Utc;
use diesel::{
  ExpressionMethods, QueryDsl, QueryableByName, SelectableHelper, sql_query,
  sql_types::{BigInt, Integer, Text},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_common::governance::{
  ActiveCasesSummary, AdminConfigAuditEntry, AdminDashboardResponse,
  AdminReputationStatsResponse, FederationSummary, JuryQueueSummary, PerCommunityActiveRuleSet,
  ReputationBuckets, RuleSetSummary, ThresholdsSnapshot,
};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{newtypes::CommunityId, source::governance::governance_log::GovernanceLog};
use lemmy_db_schema_file::schema::governance_log as governance_log_schema;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
use std::collections::BTreeMap;

pub async fn admin_dashboard(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminDashboardResponse>> {
  is_admin(&local_user_view)?;

  let mut cache = ConfigCache::new();
  let mut pool = context.pool();
  let conn = &mut get_conn(&mut pool).await?;

  let active_cases = count_active_cases(conn).await?;
  let jury_queue = count_jury_queue(conn).await?;
  let recent_config_changes = list_recent_config_changes(conn).await?;
  let federation = federation_summary(conn).await?;
  let reputation = reputation_instance_scope(conn, &mut cache, &mut context.pool()).await?;
  let rule_sets = rule_sets_summary(conn, &mut cache, &mut context.pool()).await?;

  Ok(Json(AdminDashboardResponse {
    active_cases,
    jury_queue,
    recent_config_changes,
    federation,
    reputation,
    rule_sets,
    calculated_at: Utc::now(),
  }))
}

#[derive(QueryableByName)]
struct StatusCountRow {
  #[diesel(sql_type = Text)]
  status: String,
  #[diesel(sql_type = BigInt)]
  c: i64,
}

async fn count_active_cases(conn: &mut AsyncPgConnection) -> LemmyResult<ActiveCasesSummary> {
  let sql = "\
     SELECT status::text AS status, COUNT(*)::bigint AS c \
     FROM moderation_case \
     GROUP BY status";
  let rows: Vec<StatusCountRow> = sql_query(sql).load(conn).await?;
  let by_status: BTreeMap<String, i64> = rows.into_iter().map(|r| (r.status, r.c)).collect();

  let total_active: i64 = by_status
    .iter()
    .filter(|(k, _)| !matches!(k.as_str(), "Decided" | "Closed" | "EmergencyRemove"))
    .map(|(_, v)| *v)
    .sum();

  Ok(ActiveCasesSummary {
    by_status,
    total_active,
  })
}

#[derive(QueryableByName)]
struct JuryCountRow {
  #[diesel(sql_type = BigInt)]
  pending_accept: i64,
  #[diesel(sql_type = BigInt)]
  accepted: i64,
  #[diesel(sql_type = BigInt)]
  submitted: i64,
}

async fn count_jury_queue(conn: &mut AsyncPgConnection) -> LemmyResult<JuryQueueSummary> {
  let sql = "\
     SELECT \
       COUNT(*) FILTER (WHERE status = 'Selected')::bigint  AS pending_accept, \
       COUNT(*) FILTER (WHERE status = 'Accepted')::bigint  AS accepted, \
       COUNT(*) FILTER (WHERE status = 'Submitted')::bigint AS submitted \
     FROM jury_assignment";
  let row: JuryCountRow = sql_query(sql).get_result(conn).await?;
  Ok(JuryQueueSummary {
    pending_accept: row.pending_accept,
    accepted: row.accepted,
    submitted: row.submitted,
  })
}

async fn list_recent_config_changes(
  conn: &mut AsyncPgConnection,
) -> LemmyResult<Vec<AdminConfigAuditEntry>> {
  let rows: Vec<GovernanceLog> = governance_log_schema::table
    .filter(governance_log_schema::entry_kind.eq_any(vec![
      ENTRY_KIND_ADMIN_CONFIG_CHANGED,
      ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
    ]))
    .order_by((
      governance_log_schema::created_at.desc(),
      governance_log_schema::id.desc(),
    ))
    .limit(20)
    .select(GovernanceLog::as_select())
    .load(conn)
    .await?;
  Ok(rows.into_iter().map(project_to_audit_entry).collect())
}

#[derive(QueryableByName)]
struct FederationCountRow {
  #[diesel(sql_type = BigInt)]
  active: i64,
  #[diesel(sql_type = BigInt)]
  expired: i64,
  #[diesel(sql_type = BigInt)]
  total: i64,
}

async fn federation_summary(conn: &mut AsyncPgConnection) -> LemmyResult<FederationSummary> {
  let sql = "\
     SELECT \
       COUNT(*) FILTER (WHERE valid_until IS NULL OR valid_until > now())::bigint AS active, \
       COUNT(*) FILTER (WHERE valid_until IS NOT NULL AND valid_until <= now())::bigint AS expired, \
       COUNT(*)::bigint AS total \
     FROM federation_attestation";
  let row: FederationCountRow = sql_query(sql).get_result(conn).await?;
  Ok(FederationSummary {
    active: row.active,
    expired: row.expired,
    total: row.total,
  })
}

#[derive(QueryableByName)]
struct RuleSetAggregateRow {
  #[diesel(sql_type = BigInt)]
  communities_with_rule_sets: i64,
  #[diesel(sql_type = BigInt)]
  total_versions: i64,
}

#[derive(QueryableByName)]
struct CommunityIdRow {
  #[diesel(sql_type = Integer)]
  community_id: i32,
}

async fn rule_sets_summary(
  conn: &mut AsyncPgConnection,
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
) -> LemmyResult<RuleSetSummary> {
  let agg: RuleSetAggregateRow = sql_query(
    "SELECT \
       COUNT(DISTINCT community_id)::bigint AS communities_with_rule_sets, \
       COUNT(*)::bigint AS total_versions \
     FROM rule_set_version",
  )
  .get_result(conn)
  .await?;

  let community_ids: Vec<CommunityIdRow> = sql_query(
    "SELECT DISTINCT community_id \
     FROM rule_set_version \
     ORDER BY community_id \
     LIMIT 100",
  )
  .load(conn)
  .await?;

  let mut per_community = Vec::with_capacity(community_ids.len());
  for r in community_ids {
    let cid = CommunityId(r.community_id);
    let active_version_id = get_int_opt(
      cache,
      pool,
      Scope::Community(cid),
      "rule_set.active_version_id",
    )
    .await?
    .and_then(|i| i32::try_from(i).ok());
    per_community.push(PerCommunityActiveRuleSet {
      community_id: cid,
      active_version_id,
    });
  }

  Ok(RuleSetSummary {
    communities_with_rule_sets: agg.communities_with_rule_sets,
    total_versions: agg.total_versions,
    per_community,
  })
}

async fn reputation_instance_scope(
  conn: &mut AsyncPgConnection,
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
) -> LemmyResult<AdminReputationStatsResponse> {
  let thresholds_current = ThresholdsSnapshot {
    jury_reliability: get_int(cache, pool, Scope::Instance, "thresholds.jury_reliability").await?,
    reporting_accuracy: get_int(cache, pool, Scope::Instance, "thresholds.reporting_accuracy")
      .await?,
    endorsement_strength: get_int(cache, pool, Scope::Instance, "thresholds.endorsement_strength")
      .await?,
  };

  let buckets = ReputationBuckets {
    reporting_accuracy: bucket_query(conn, "reporting_accuracy", None).await?,
    jury_reliability: bucket_query(conn, "jury_reliability", None).await?,
    participation_consistency: bucket_query(conn, "participation_consistency", None).await?,
    endorsement_strength: bucket_query(conn, "endorsement_strength", None).await?,
  };
  let capability_counts = capability_query(conn, None).await?;
  let founder_event_stats = founder_query(conn, None).await?;

  Ok(AdminReputationStatsResponse {
    buckets,
    thresholds_current,
    capability_counts,
    founder_event_stats,
    calculated_at: Utc::now(),
  })
}
