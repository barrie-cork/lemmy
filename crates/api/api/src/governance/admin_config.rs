//! Brehon admin-config HTTP endpoints — dry-run impact + handlers.
//!
//! This module backs the three `/api/v4/governance/admin/config*` routes
//! (registration in task 7). Task 3 lands the dry-run `compute_downstream_impact`
//! dispatcher and its seven per-category implementations; the `admin_set_config`
//! / `admin_get_config` / `admin_get_config_audit` handlers land in tasks 4-5.
//!
//! ## Dry-run impact (OQ-V1-AD-03 resolution, 2026-04-20 `3f0dd6572`)
//!
//! `diesel_utils::run_transaction` exposes no SAVEPOINT primitive. The
//! impact preview therefore runs as a pure read-only query against the
//! current persisted state + the proposed value, executed BEFORE the
//! write transaction opens. When `dry_run = true` the handler returns the
//! preview and no write happens. When `dry_run = false` the already-
//! computed preview is returned alongside the applied row. Post-commit
//! drift (another admin write between preview-query and write tx) is not
//! the handler's responsibility to describe — the pre-tx snapshot is the
//! contract.
//!
//! ## No writes in this module (task 3)
//!
//! Every function here is read-only. `fn(pool: &mut DbPool<'_>)` or
//! `fn(conn: &mut AsyncPgConnection)` only — no `run_transaction`, no
//! `insert_into`, no `update`. Task 4's handler performs the write.
//!
//! ## Impact shape by key category (PRD §4.3)
//!
//! | Key family | Impact function | Return shape |
//! |---|---|---|
//! | `thresholds.*` | `impact_for_threshold_key` | `{current_eligible, proposed_eligible, delta}` |
//! | `jury.panel_size` | `impact_for_jury_panel_size` | `{open_cases_needing_reassembly}` |
//! | `jury.max_concurrent_assignments` | `impact_for_jury_max_concurrent` | `{over_cap_jurors}` |
//! | `liability.sponsor_liability_floor` | `impact_for_liability_sponsor_floor_marker` | `{message: "requires materialised view"}` |
//! | `report.case_threshold_micros` | `impact_for_report_case_threshold_micros` | `{cases_that_would_open, cases_that_would_stay_closed}` |
//! | `decay.*` | `impact_for_decay_key` | `{message: "gradual"}` |
//! | other | `impact_for_other` | `{estimated_first_effect_at}` |

use crate::governance::config::Scope;
use diesel::{
  QueryableByName,
  sql_query,
  sql_types::{BigInt, Integer, Nullable},
};
use diesel_async::RunQueryDsl;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::{Value, json};

/// Row shape for the two-count FILTER aggregate used by threshold + liability +
/// report-threshold impact queries. Module-scope per workspace lint
/// `items-after-statements`.
#[derive(QueryableByName)]
struct TwoCountRow {
  #[diesel(sql_type = BigInt)]
  c_current: i64,
  #[diesel(sql_type = BigInt)]
  c_proposed: i64,
}

/// Row shape for single-count queries (open-cases, over-cap-jurors).
#[derive(QueryableByName)]
struct SingleCountRow {
  #[diesel(sql_type = BigInt)]
  c: i64,
}

/// Central dispatcher. Task 4's `admin_set_config` calls this PRE-transaction
/// after policy + type + range validation pass, and before the write closure
/// opens.
///
/// Category is selected by exact-match on `jury.panel_size` /
/// `jury.max_concurrent_assignments` / `liability.sponsor_liability_floor` /
/// `report.case_threshold_micros`, by prefix on `thresholds.*` and `decay.*`,
/// and falls back to `impact_for_other` for every other key. Keys that don't
/// have a meaningful count-based preview fall into `impact_for_other` — that
/// is not a bug; it's the "we don't preview this shape" signal per PRD §4.3.
///
/// The `current_value` / `proposed_value` JSON values are already in rehydrated
/// typed form (int → JSON number, bool → JSON bool, etc.); extraction uses
/// `as_i64()` / `as_f64()` accordingly.
pub async fn compute_downstream_impact(
  pool: &mut DbPool<'_>,
  key: &str,
  scope: Scope,
  current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let community_bind: Option<i32> = match scope {
    Scope::Instance => None,
    Scope::Community(id) => Some(id.0),
  };

  if key.starts_with("thresholds.") {
    return impact_for_threshold_key(pool, key, community_bind, current_value, proposed_value)
      .await;
  }

  if key.starts_with("decay.") {
    return Ok(impact_for_decay_key());
  }

  match key {
    "jury.panel_size" => impact_for_jury_panel_size(pool, community_bind).await,
    "jury.max_concurrent_assignments" => {
      impact_for_jury_max_concurrent(pool, current_value, proposed_value).await
    }
    "liability.sponsor_liability_floor" => Ok(impact_for_liability_sponsor_floor_marker()),
    "report.case_threshold_micros" => {
      impact_for_report_case_threshold_micros(pool, community_bind, current_value, proposed_value)
        .await
    }
    _ => Ok(impact_for_other()),
  }
}

/// `thresholds.jury_reliability | reporting_accuracy | endorsement_strength`.
///
/// **Hard GOTCHA**: `reputation_snapshot.jury_eligible` is a DERIVED boolean
/// computed at snapshot time against the CURRENT threshold. Previewing the
/// impact of a PROPOSED threshold MUST query the RAW columns
/// (`jury_reliability`, `reporting_accuracy`, `endorsement_strength`), not
/// `jury_eligible`. The raw columns are `Int4` per `schema.rs:1186-1189`.
///
/// Returns `{current_eligible, proposed_eligible, delta}` where `delta =
/// proposed - current` (signed; negative = more users excluded).
async fn impact_for_threshold_key(
  pool: &mut DbPool<'_>,
  key: &str,
  community_bind: Option<i32>,
  current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let column = match key {
    "thresholds.jury_reliability" => "jury_reliability",
    "thresholds.reporting_accuracy" => "reporting_accuracy",
    "thresholds.endorsement_strength" => "endorsement_strength",
    other => {
      return Err(LemmyErrorType::Unknown(format!("not a threshold key: {other}")).into());
    }
  };

  // Clamp i64 → i32 to match the `Int4` column type. Config values seeded
  // for thresholds have `NumericRange::max = 100` in `CONFIG_KEY_METADATA`,
  // so the clamp is a correctness guarantee, not a hack.
  let current_threshold = i32_from_value(current_value, "current_value", key)?;
  let proposed_threshold = i32_from_value(proposed_value, "proposed_value", key)?;

  let conn = &mut get_conn(pool).await?;

  // One round-trip — two filtered counts on the raw column. `community_id IS
  // NOT DISTINCT FROM $1` mirrors the convention used by `capability_query` +
  // `bucket_query` in admin_reputation_stats.rs (treats NULL as equal to NULL
  // so an instance-wide impact query binds None).
  let sql = format!(
    "SELECT \
       COUNT(*) FILTER (WHERE {column} >= $2)::bigint AS c_current, \
       COUNT(*) FILTER (WHERE {column} >= $3)::bigint AS c_proposed \
     FROM reputation_snapshot \
     WHERE community_id IS NOT DISTINCT FROM $1"
  );

  let row: TwoCountRow = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .bind::<Integer, _>(current_threshold)
    .bind::<Integer, _>(proposed_threshold)
    .get_result(conn)
    .await?;

  Ok(json!({
    "current_eligible": row.c_current,
    "proposed_eligible": row.c_proposed,
    "delta": row.c_proposed - row.c_current,
  }))
}

/// Helper: rehydrate a JSON value as `i32` with clear error messaging. Used by
/// threshold queries where the column type is `Int4`.
fn i32_from_value(v: &Value, label: &str, key: &str) -> LemmyResult<i32> {
  let n = v.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(format!("threshold impact: {label} for `{key}` is not an integer"))
  })?;
  i32::try_from(n).map_err(|_| {
    LemmyErrorType::Unknown(format!(
      "threshold impact: {label} for `{key}` (= {n}) does not fit in i32"
    ))
    .into()
  })
}

/// `jury.panel_size`. Counts open cases in the two actively-assembling states
/// — any increase in panel size forces these cases to pick up additional
/// jurors on their next `panel_assembled` event.
///
/// Plan §13 task 3: `status IN ('JurySelection', 'InReview')`.
async fn impact_for_jury_panel_size(
  pool: &mut DbPool<'_>,
  community_bind: Option<i32>,
) -> LemmyResult<Value> {
  let conn = &mut get_conn(pool).await?;

  let sql = "\
     SELECT COUNT(*)::bigint AS c \
     FROM moderation_case \
     WHERE status IN ('JurySelection', 'InReview') \
       AND community_id IS NOT DISTINCT FROM $1";

  let row: SingleCountRow = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .get_result(conn)
    .await?;

  Ok(json!({ "open_cases_needing_reassembly": row.c }))
}

/// `jury.max_concurrent_assignments`. Counts jurors whose current count of
/// Selected/Accepted assignments exceeds the proposed cap.
///
/// Mirrors the sub-query shape from `admin_assign_jury.rs:321-326`'s strict
/// eligibility filter but returns the over-cap count instead of excluding
/// them from a selection. Scope is instance-wide here: concurrent-cap is
/// per-person across all communities (a juror over-cap on the instance is
/// over-cap everywhere).
async fn impact_for_jury_max_concurrent(
  pool: &mut DbPool<'_>,
  _current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let proposed_cap = proposed_value.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(
      "jury.max_concurrent_assignments impact: proposed_value is not an integer".to_string(),
    )
  })?;

  let conn = &mut get_conn(pool).await?;

  let sql = "\
     SELECT COUNT(*)::bigint AS c FROM ( \
       SELECT ja.person_id FROM jury_assignment ja \
       WHERE ja.status IN ('Selected', 'Accepted') \
       GROUP BY ja.person_id \
       HAVING count(*) > $1 \
     ) sub";

  let row: SingleCountRow = sql_query(sql)
    .bind::<BigInt, _>(proposed_cap)
    .get_result(conn)
    .await?;

  Ok(json!({ "over_cap_jurors": row.c }))
}

/// `liability.sponsor_liability_floor`. The floor is a config-only tuning
/// knob — no column on `reputation_snapshot` materialises a person's running
/// liability total, and aggregating from `reputation_event` rows in a
/// pre-transaction read is expensive. v1-AD-b returns a descriptive marker;
/// if operators need a count, it lands as a later refinement against a
/// materialised view (out of scope per plan §12).
///
/// `_community_bind` and `_proposed_value` are intentionally unused — they
/// are the dispatcher's contract shape, kept for forward compat when the
/// materialised liability total exists.
fn impact_for_liability_sponsor_floor_marker() -> Value {
  json!({
    "message": "Sponsor liability floor is a tuning knob; count-based preview requires the liability-aggregate view (deferred to v1-AD-b.1).",
  })
}

/// `report.case_threshold_micros`. The per-case `moderation_case.threshold_score`
/// column (Int8) stores the threshold scalar at which the case transitioned out
/// of Open. Count cases sitting in the band between old and new thresholds so
/// operators can reason about flip direction without loading the full case list.
///
/// Returns two counts:
/// - `cases_that_would_open` — cases whose `threshold_score` sits in
///   `[proposed, current)`: the proposed (lower) threshold would have opened them.
/// - `cases_that_would_stay_closed` — cases whose `threshold_score` sits in
///   `[current, proposed)`: the proposed (higher) threshold would no longer
///   trip them.
///
/// Exactly one of the two bands is non-empty per request (the other is empty
/// due to the ordering constraint inside `FILTER`). The UI interprets the
/// populated side.
async fn impact_for_report_case_threshold_micros(
  pool: &mut DbPool<'_>,
  community_bind: Option<i32>,
  current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let current_threshold = current_value.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(
      "report.case_threshold_micros impact: current_value is not an integer".to_string(),
    )
  })?;
  let proposed_threshold = proposed_value.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(
      "report.case_threshold_micros impact: proposed_value is not an integer".to_string(),
    )
  })?;

  let conn = &mut get_conn(pool).await?;

  let sql = "\
     SELECT \
       COUNT(*) FILTER (WHERE threshold_score >= $2 AND threshold_score <  $3)::bigint \
         AS c_current, \
       COUNT(*) FILTER (WHERE threshold_score >= $3 AND threshold_score <  $2)::bigint \
         AS c_proposed \
     FROM moderation_case \
     WHERE community_id IS NOT DISTINCT FROM $1";

  let row: TwoCountRow = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .bind::<BigInt, _>(proposed_threshold)
    .bind::<BigInt, _>(current_threshold)
    .get_result(conn)
    .await?;

  Ok(json!({
    "cases_that_would_open": row.c_current,
    "cases_that_would_stay_closed": row.c_proposed,
  }))
}

/// `decay.*` family. Half-life constants affect every future reputation
/// snapshot tick, not the current snapshot — so a count-based preview against
/// `reputation_snapshot` tells the operator nothing useful. Return the
/// "gradual" marker per PRD §4.3.
fn impact_for_decay_key() -> Value {
  json!({
    "message": "Decay changes take effect at next reputation snapshot; no immediate count diff available.",
  })
}

/// Catch-all for keys that have no meaningful count-based impact (e.g.
/// `governance.dashboard.*`, `onboarding.*`, `job.*`). Returns the "first
/// effect at next handler invocation" marker per PRD §4.3.
fn impact_for_other() -> Value {
  json!({ "estimated_first_effect_at": "next handler invocation" })
}
