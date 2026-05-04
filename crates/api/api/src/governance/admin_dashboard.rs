//! `GET /api/v4/governance/admin/dashboard` — single-round-trip
//! instance-wide aggregate for the admin governance dashboard.
//!
//! Read-only; emits NO `governance_log` entry (ADR-008 signed-log
//! compliance: dashboard viewing is not a governance write).

use crate::governance::{
  admin_reputation_stats::{bucket_query, capability_query, founder_query},
  audit_projection::project_to_audit_entry,
  config::{ConfigCache, Scope, get_int},
  governance_log::{ENTRY_KIND_ADMIN_CONFIG_CHANGED, ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED},
};
use actix_web::web::{Data, Json};
use chrono::Utc;
use diesel::{
  ExpressionMethods, QueryDsl, QueryableByName, SelectableHelper,
  dsl::count_star,
  sql_query,
  sql_types::{BigInt, Integer},
};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_common::governance::{
  ActiveCasesSummary, AdminConfigAuditEntry, AdminDashboardResponse,
  AdminReputationStatsResponse, FederationSummary, JuryQueueSummary, PerCommunityActiveRuleSet,
  ReputationBuckets, RuleSetSummary, ThresholdsSnapshot,
};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{newtypes::CommunityId, source::governance::governance_log::GovernanceLog};
use lemmy_db_schema_file::{
  enums::CaseStatus,
  schema::{governance_log as governance_log_schema, moderation_case},
};
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
  let rule_sets = rule_sets_summary(conn).await?;

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

/// Classifies a `CaseStatus` variant as active (counts toward
/// `total_active`) or not. The exhaustive match makes any new enum
/// variant a compile error forcing this decision — no silent drift
/// (cr-23; ADR-013 coding guideline).
///
/// Per `ActiveCasesSummary` DTO contract, `Decided`, `Closed`, and
/// `EmergencyRemove` are terminal states that do NOT count.
/// `AdminReview` is paused by a direct mod action (OQ-008 resolution)
/// but remains pending admin resume — kept in the active set so the
/// dashboard surfaces it as work-in-progress.
fn is_active_status(status: CaseStatus) -> bool {
  match status {
    CaseStatus::Open
    | CaseStatus::ThresholdMet
    | CaseStatus::JurySelection
    | CaseStatus::InReview
    | CaseStatus::Appealed
    | CaseStatus::AdminReview
    // PRD §3.3 + ADR-013: grace window is actively pending — show in dashboard.
    | CaseStatus::SponsorLiabilityPending => true,
    CaseStatus::Decided
    | CaseStatus::Closed
    | CaseStatus::EmergencyRemove
    // PRD §3.3 + ADR-013: fired and escaped are terminal — not active dashboard cases.
    | CaseStatus::SponsorLiabilityFired
    | CaseStatus::SponsorLiabilityEscaped => false,
  }
}

/// Renders a `CaseStatus` as the PascalCase key used in
/// `ActiveCasesSummary.by_status` — byte-identical to the verbatim
/// `DbValueStyle` string that the prior `status::text` SQL cast
/// produced, so consumers keyed to "Open", "EmergencyRemove", ... keep
/// working.
fn status_key(status: CaseStatus) -> &'static str {
  match status {
    CaseStatus::Open => "Open",
    CaseStatus::ThresholdMet => "ThresholdMet",
    CaseStatus::JurySelection => "JurySelection",
    CaseStatus::InReview => "InReview",
    CaseStatus::Decided => "Decided",
    CaseStatus::Appealed => "Appealed",
    CaseStatus::Closed => "Closed",
    CaseStatus::EmergencyRemove => "EmergencyRemove",
    CaseStatus::AdminReview => "AdminReview",
    // PRD §3.3 + ADR-013: sponsor-liability variants map to their PascalCase names.
    CaseStatus::SponsorLiabilityPending => "SponsorLiabilityPending",
    CaseStatus::SponsorLiabilityFired => "SponsorLiabilityFired",
    CaseStatus::SponsorLiabilityEscaped => "SponsorLiabilityEscaped",
  }
}

async fn count_active_cases(conn: &mut AsyncPgConnection) -> LemmyResult<ActiveCasesSummary> {
  // Use Diesel's typed DSL so the rows come back as `CaseStatus` enum
  // values — the downstream `is_active_status` match is then
  // compile-time exhaustive (cr-23).
  let rows: Vec<(CaseStatus, i64)> = moderation_case::table
    .group_by(moderation_case::status)
    .select((moderation_case::status, count_star()))
    .load(conn)
    .await?;

  let mut by_status: BTreeMap<String, i64> = BTreeMap::new();
  let mut total_active: i64 = 0;
  for (status, count) in rows {
    if is_active_status(status) {
      total_active += count;
    }
    by_status.insert(status_key(status).to_string(), count);
  }

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
  // Filter `signature IS NOT NULL` so rows left half-written by a failed
  // signature UPDATE (see `governance_log::append` in
  // crates/api/api/src/governance/governance_log.rs) cannot leak into the
  // dashboard. This matches the SSE audit-stream invariant: the
  // `governance_log_notify_trigger` fires on `signature NULL → NOT NULL`,
  // so signed rows are the only subscribable artifacts. Unsigned rows
  // would also not carry a verifiable hash chain position, so rendering
  // them in the dashboard would mislead the admin (cr-22).
  let rows: Vec<GovernanceLog> = governance_log_schema::table
    .filter(governance_log_schema::entry_kind.eq_any(vec![
      ENTRY_KIND_ADMIN_CONFIG_CHANGED,
      ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
    ]))
    .filter(governance_log_schema::signature.is_not_null())
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

#[derive(diesel::Queryable)]
struct ScopeValueIntRow {
  scope: String,
  value_int: Option<i64>,
}

async fn rule_sets_summary(conn: &mut AsyncPgConnection) -> LemmyResult<RuleSetSummary> {
  use lemmy_db_schema_file::schema::governance_config;

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

  // cr-24: batched replacement for the per-community
  // `get_int_opt(..., Scope::Community(cid), "rule_set.active_version_id")`
  // loop. The prior shape issued 2×N DB round-trips worst case (one per
  // community, each cascading Community→Instance) while `conn` was still
  // held and `pool` was borrowed. One set-based SELECT closes both: the
  // IN-list covers every community scope plus the shared instance
  // fallback, and a simple map-lookup per community replays the
  // cascade semantics of `get_int_opt`.
  //
  // `ConfigCache` is deliberately NOT populated here: it is a
  // per-request memo, the per-request handler reads
  // `rule_set.active_version_id` exactly once per community (above), and
  // each community maps to a distinct `(scope, key)` cache entry — so
  // populating the cache would cost a HashMap insert with no downstream
  // reader. If a future call site reads these keys later in the same
  // request, it can re-fetch through `get_int_opt` unchanged.
  let scope_strings: Vec<String> = community_ids
    .iter()
    .map(|r| format!("community:{}", r.community_id))
    .chain(std::iter::once("instance".to_string()))
    .collect();

  // governance_config is append-only; ORDER BY valid_from DESC ensures the
  // most recent row per (scope, key) comes first. or_insert below then
  // keeps only that first (latest) value for each scope.
  let scope_value_rows: Vec<ScopeValueIntRow> = governance_config::table
    .filter(governance_config::key.eq("rule_set.active_version_id"))
    .filter(governance_config::value_type.eq("int"))
    .filter(governance_config::scope.eq_any(&scope_strings))
    .select((
      governance_config::scope,
      governance_config::value_int,
    ))
    .order_by(governance_config::valid_from.desc())
    .load::<ScopeValueIntRow>(conn)
    .await?;

  let mut scope_to_int: std::collections::HashMap<String, i64> = std::collections::HashMap::new();
  for row in scope_value_rows {
    if let Some(v) = row.value_int {
      scope_to_int.entry(row.scope).or_insert(v);
    }
  }
  let instance_fallback = scope_to_int.get("instance").copied();

  let mut per_community = Vec::with_capacity(community_ids.len());
  for r in community_ids {
    let cid = CommunityId(r.community_id);
    // Cascade: community-scoped row wins; fall back to instance-scoped.
    // Same semantics as `fetch_value` / `get_int_opt` for
    // `Scope::Community(_)`.
    let active_version_id = scope_to_int
      .get(&format!("community:{}", r.community_id))
      .copied()
      .or(instance_fallback)
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
