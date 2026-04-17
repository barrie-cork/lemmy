//! Brehon governance config reader.
//!
//! Single source of tuneable values for the whole governance platform. Every
//! hardcoded constant from Phase 4 (jury panel size, reputation deltas,
//! thresholds, founder multipliers, etc.) migrates to a row in
//! `governance_config` over Phase 5a/5b. Handlers never read constants
//! directly; they call `get_int`/`get_float`/`get_bool`/`get_text` with a
//! `Scope` and a dotted key.
//!
//! ## Fallback cascade
//!
//! For each read, the reader tries in order:
//!
//! 1. `community:<id>` row (most specific — v1 feature; no v0 handler writes
//!    community-scoped rows but the reader supports them for forward compat)
//! 2. `instance` row
//! 3. Rust `const DEFAULT_*` fallback defined below — Watch 1 parity contract
//!
//! The const fallback matters: if an admin accidentally deletes a seeded
//! row, reads degrade gracefully to the default instead of surfacing an
//! error.
//!
//! ## Parity contract (Watch 1)
//!
//! Every seeded row in the task-50 migration MUST have a matching Rust
//! `const` declared below, with a matching type. Two tests enforce this:
//!
//! - `parity::seeded_keys_count_matches_const_count` — structural, in-crate,
//!   no DB required.
//! - `config_parity_round_trip` — DB-backed; lives in
//!   `crates/server/tests/e2e.rs` per GOTCHA-50h landing-spot A. Calls the
//!   typed accessor for every seeded key with its declared `value_type`.
//!
//! ## ConfigCache
//!
//! Per-request memoisation. Constructed at handler entry
//! (`ConfigCache::new()`), threaded into the tx closure if the handler uses
//! `run_transaction`. First read per `(scope, key)` hits the DB; subsequent
//! reads return the cached value. The cache does NOT escape the request
//! — a new request starts fresh.

use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::CommunityId;
use lemmy_db_schema_file::schema::governance_config_current;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use std::collections::HashMap;

// -- Public API -------------------------------------------------------------

/// Scope discriminator for config reads. `Community(id)` first, then
/// `Instance`, then the Rust const. v0 handlers pass `Instance` — no v0
/// code writes community-scoped rows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Scope {
  Instance,
  Community(CommunityId),
}

impl Scope {
  fn as_str(self) -> String {
    match self {
      Scope::Instance => "instance".to_string(),
      Scope::Community(CommunityId(id)) => format!("community:{id}"),
    }
  }
}

/// One cached value. Matches the row shape — only one variant populated.
#[derive(Debug, Clone)]
enum CachedValue {
  Int(i64),
  Float(f64),
  Bool(bool),
  Text(String),
}

/// Per-request memo cache. Keys are `(scope_repr, key)` tuples.
#[derive(Debug, Default)]
pub struct ConfigCache {
  entries: HashMap<(String, String), CachedValue>,
}

impl ConfigCache {
  pub fn new() -> Self {
    Self::default()
  }
}

// -- Typed accessors --------------------------------------------------------

/// Read an `int` config value. Cascades `community:<id>` → `instance` → const.
///
/// Returns the `DEFAULT_*` const for this key if no row matches — i.e.
/// admin row deletion degrades gracefully, not into an error.
pub async fn get_int(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<i64> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Int(v)) = cache.entries.get(&(scope_repr.clone(), key.to_string())) {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Int(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as int but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_int(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr, key.to_string()), CachedValue::Int(v));
  Ok(v)
}

pub async fn get_float(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<f64> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Float(v)) = cache.entries.get(&(scope_repr.clone(), key.to_string())) {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Float(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as float but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_float(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr, key.to_string()), CachedValue::Float(v));
  Ok(v)
}

pub async fn get_bool(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<bool> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Bool(v)) = cache.entries.get(&(scope_repr.clone(), key.to_string())) {
    return Ok(*v);
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Bool(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as bool but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_bool(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr, key.to_string()), CachedValue::Bool(v));
  Ok(v)
}

pub async fn get_text(
  cache: &mut ConfigCache,
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<String> {
  let scope_repr = scope.as_str();
  if let Some(CachedValue::Text(v)) = cache.entries.get(&(scope_repr.clone(), key.to_string())) {
    return Ok(v.clone());
  }
  let v = match fetch_value(pool, scope, key).await? {
    Some(CachedValue::Text(v)) => v,
    Some(other) => {
      return Err(LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` requested as text but stored as {other:?}"
      ))
      .into());
    }
    None => const_default_text(key).ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "governance_config key `{key}` missing from DB and has no Rust const default"
      ))
    })?,
  };
  cache
    .entries
    .insert((scope_repr, key.to_string()), CachedValue::Text(v.clone()));
  Ok(v)
}

// -- Private helpers --------------------------------------------------------

/// Tuple shape for the `governance_config_current` row load. Module-scope
/// because `items-after-statements` is denied at workspace level.
type ConfigRow = (
  String,
  Option<i64>,
  Option<f64>,
  Option<bool>,
  Option<String>,
);

/// Fetch a single value from `governance_config_current` — the view already
/// returns the most-recent row per `(scope, key)`. Returns `None` if no row
/// matches the requested scope.
///
/// Fallback cascade: if `scope = Community(id)` returns None, retry with
/// `Instance` before returning None to the caller.
async fn fetch_value(
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<CachedValue>> {
  if let Scope::Community(_) = scope {
    if let Some(v) = fetch_value_at_scope(pool, scope.as_str(), key).await? {
      return Ok(Some(v));
    }
    return fetch_value_at_scope(pool, Scope::Instance.as_str(), key).await;
  }
  fetch_value_at_scope(pool, scope.as_str(), key).await
}

async fn fetch_value_at_scope(
  pool: &mut DbPool<'_>,
  scope_str: String,
  key: &str,
) -> LemmyResult<Option<CachedValue>> {
  let conn = &mut get_conn(pool).await?;

  let row: Option<ConfigRow> = governance_config_current::table
    .filter(governance_config_current::scope.eq(scope_str))
    .filter(governance_config_current::key.eq(key))
    .select((
      governance_config_current::value_type,
      governance_config_current::value_int,
      governance_config_current::value_float,
      governance_config_current::value_bool,
      governance_config_current::value_text,
    ))
    .first::<ConfigRow>(conn)
    .await
    .optional()?;

  Ok(row.and_then(|(vtype, vi, vf, vb, vt)| match vtype.as_str() {
    "int" => vi.map(CachedValue::Int),
    "float" => vf.map(CachedValue::Float),
    "bool" => vb.map(CachedValue::Bool),
    "text" => vt.map(CachedValue::Text),
    _ => None,
  }))
}

// -- Const defaults ---------------------------------------------------------
//
// Every `pub const DEFAULT_*` below mirrors a seeded row in the task 50
// migration. The `parity::SEEDED_KEYS_WITH_CONSTS` list + structural test
// enforce the one-to-one mapping.

pub const DEFAULT_THRESHOLDS_JURY_RELIABILITY: i64 = 50;
pub const DEFAULT_THRESHOLDS_REPORTING_ACCURACY: i64 = 50;
pub const DEFAULT_THRESHOLDS_ENDORSEMENT_STRENGTH: i64 = 25;
pub const DEFAULT_JURY_PANEL_SIZE: i64 = 5;
pub const DEFAULT_JURY_QUORUM: i64 = 3;
pub const DEFAULT_JURY_AGE_REQUIREMENT_DAYS: i64 = 60;
pub const DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS: i64 = 3;
pub const DEFAULT_JURY_FALLBACK_ON_SMALL_POOL: bool = true;
pub const DEFAULT_DELTAS_JUROR_ALIGNED: i64 = 10;
pub const DEFAULT_DELTAS_JUROR_OUTLIER: i64 = -5;
pub const DEFAULT_DELTAS_REPORTER_UPHELD: i64 = 10;
pub const DEFAULT_DELTAS_REPORTER_DISMISSED: i64 = -5;
pub const DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSOR: i64 = 5;
pub const DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSEE: i64 = 5;
pub const DEFAULT_DELTAS_SPONSOR_LIABILITY_MINOR: i64 = -10;
pub const DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE: i64 = -50;
pub const DEFAULT_DELTAS_SPONSOR_LIABILITY_SEVERE: i64 = -200;
pub const DEFAULT_LIABILITY_FOUNDER_MULTIPLIER: f64 = 2.0;
pub const DEFAULT_LIABILITY_REGULAR_MULTIPLIER: f64 = 1.0;
pub const DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR: i64 = 0;
pub const DEFAULT_REPORT_BASE_WEIGHT: f64 = 1.0;
pub const DEFAULT_REPORT_CLAMP_MIN: f64 = 0.1;
pub const DEFAULT_REPORT_CLAMP_MAX: f64 = 2.0;
pub const DEFAULT_REPORT_RECENCY_HALF_LIFE_HOURS: f64 = 168.0;
pub const DEFAULT_REPORT_CASE_THRESHOLD_MICROS: i64 = 3_000_000;
pub const DEFAULT_DECAY_POSITIVE_HALF_LIFE_DAYS: i64 = 90;
pub const DEFAULT_ONBOARDING_DEFAULT_MEMBERSHIP_STATE: &str = "member";
pub const DEFAULT_ONBOARDING_SPONSOR_GATE_STRATEGY: &str = "age";
pub const DEFAULT_ONBOARDING_SPONSOR_MIN_ACCOUNT_AGE_DAYS: i64 = 30;
pub const DEFAULT_FOUNDER_MAX_FOUNDERS_ACTIVE: i64 = 20;
pub const DEFAULT_FOUNDER_MAX_EXPIRES_DAYS: i64 = 365;
pub const DEFAULT_FOUNDER_MAX_SEED_DELTA: i64 = 200;
pub const DEFAULT_JOB_SNAPSHOT_INTERVAL_SECONDS: i64 = 900;
pub const DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE: i64 = 500;

fn const_default_int(key: &str) -> Option<i64> {
  match key {
    "thresholds.jury_reliability" => Some(DEFAULT_THRESHOLDS_JURY_RELIABILITY),
    "thresholds.reporting_accuracy" => Some(DEFAULT_THRESHOLDS_REPORTING_ACCURACY),
    "thresholds.endorsement_strength" => Some(DEFAULT_THRESHOLDS_ENDORSEMENT_STRENGTH),
    "jury.panel_size" => Some(DEFAULT_JURY_PANEL_SIZE),
    "jury.quorum" => Some(DEFAULT_JURY_QUORUM),
    "jury.age_requirement_days" => Some(DEFAULT_JURY_AGE_REQUIREMENT_DAYS),
    "jury.max_concurrent_assignments" => Some(DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS),
    "deltas.juror_aligned" => Some(DEFAULT_DELTAS_JUROR_ALIGNED),
    "deltas.juror_outlier" => Some(DEFAULT_DELTAS_JUROR_OUTLIER),
    "deltas.reporter_upheld" => Some(DEFAULT_DELTAS_REPORTER_UPHELD),
    "deltas.reporter_dismissed" => Some(DEFAULT_DELTAS_REPORTER_DISMISSED),
    "deltas.endorsement_created_sponsor" => Some(DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSOR),
    "deltas.endorsement_created_sponsee" => Some(DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSEE),
    "deltas.sponsor_liability_minor" => Some(DEFAULT_DELTAS_SPONSOR_LIABILITY_MINOR),
    "deltas.sponsor_liability_moderate" => Some(DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE),
    "deltas.sponsor_liability_severe" => Some(DEFAULT_DELTAS_SPONSOR_LIABILITY_SEVERE),
    "liability.sponsor_liability_floor" => Some(DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR),
    "report.case_threshold_micros" => Some(DEFAULT_REPORT_CASE_THRESHOLD_MICROS),
    "decay.positive_half_life_days" => Some(DEFAULT_DECAY_POSITIVE_HALF_LIFE_DAYS),
    "onboarding.sponsor_min_account_age_days" => Some(DEFAULT_ONBOARDING_SPONSOR_MIN_ACCOUNT_AGE_DAYS),
    "founder.max_founders_active" => Some(DEFAULT_FOUNDER_MAX_FOUNDERS_ACTIVE),
    "founder.max_expires_days" => Some(DEFAULT_FOUNDER_MAX_EXPIRES_DAYS),
    "founder.max_seed_delta" => Some(DEFAULT_FOUNDER_MAX_SEED_DELTA),
    "job.snapshot_interval_seconds" => Some(DEFAULT_JOB_SNAPSHOT_INTERVAL_SECONDS),
    "job.snapshot_batch_chunk_size" => Some(DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE),
    _ => None,
  }
}

fn const_default_float(key: &str) -> Option<f64> {
  match key {
    "liability.founder_multiplier" => Some(DEFAULT_LIABILITY_FOUNDER_MULTIPLIER),
    "liability.regular_multiplier" => Some(DEFAULT_LIABILITY_REGULAR_MULTIPLIER),
    "report.base_weight" => Some(DEFAULT_REPORT_BASE_WEIGHT),
    "report.clamp_min" => Some(DEFAULT_REPORT_CLAMP_MIN),
    "report.clamp_max" => Some(DEFAULT_REPORT_CLAMP_MAX),
    "report.recency_half_life_hours" => Some(DEFAULT_REPORT_RECENCY_HALF_LIFE_HOURS),
    _ => None,
  }
}

fn const_default_bool(key: &str) -> Option<bool> {
  match key {
    "jury.fallback_on_small_pool" => Some(DEFAULT_JURY_FALLBACK_ON_SMALL_POOL),
    _ => None,
  }
}

fn const_default_text(key: &str) -> Option<String> {
  match key {
    "onboarding.default_membership_state" => {
      Some(DEFAULT_ONBOARDING_DEFAULT_MEMBERSHIP_STATE.to_string())
    }
    "onboarding.sponsor_gate_strategy" => {
      Some(DEFAULT_ONBOARDING_SPONSOR_GATE_STRATEGY.to_string())
    }
    _ => None,
  }
}

// -- Parity list (Watch 1) --------------------------------------------------

/// The canonical seeded-keys list. Each tuple is `(key, const_name, value_type)`.
/// The `const_name` is not consumed at runtime — it exists for the structural
/// parity test and for grep-ability (decision-queue #14 carry-over rationale
/// in GOTCHA-50i). `pub` so `crates/server/tests/e2e.rs::config_parity_round_trip`
/// can walk it without re-declaring.
pub const SEEDED_KEYS_WITH_CONSTS: &[(&str, &str, &str)] = &[
  ("thresholds.jury_reliability", "DEFAULT_THRESHOLDS_JURY_RELIABILITY", "int"),
  ("thresholds.reporting_accuracy", "DEFAULT_THRESHOLDS_REPORTING_ACCURACY", "int"),
  ("thresholds.endorsement_strength", "DEFAULT_THRESHOLDS_ENDORSEMENT_STRENGTH", "int"),
  ("jury.panel_size", "DEFAULT_JURY_PANEL_SIZE", "int"),
  ("jury.quorum", "DEFAULT_JURY_QUORUM", "int"),
  ("jury.age_requirement_days", "DEFAULT_JURY_AGE_REQUIREMENT_DAYS", "int"),
  ("jury.max_concurrent_assignments", "DEFAULT_JURY_MAX_CONCURRENT_ASSIGNMENTS", "int"),
  ("jury.fallback_on_small_pool", "DEFAULT_JURY_FALLBACK_ON_SMALL_POOL", "bool"),
  ("deltas.juror_aligned", "DEFAULT_DELTAS_JUROR_ALIGNED", "int"),
  ("deltas.juror_outlier", "DEFAULT_DELTAS_JUROR_OUTLIER", "int"),
  ("deltas.reporter_upheld", "DEFAULT_DELTAS_REPORTER_UPHELD", "int"),
  ("deltas.reporter_dismissed", "DEFAULT_DELTAS_REPORTER_DISMISSED", "int"),
  ("deltas.endorsement_created_sponsor", "DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSOR", "int"),
  ("deltas.endorsement_created_sponsee", "DEFAULT_DELTAS_ENDORSEMENT_CREATED_SPONSEE", "int"),
  ("deltas.sponsor_liability_minor", "DEFAULT_DELTAS_SPONSOR_LIABILITY_MINOR", "int"),
  ("deltas.sponsor_liability_moderate", "DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE", "int"),
  ("deltas.sponsor_liability_severe", "DEFAULT_DELTAS_SPONSOR_LIABILITY_SEVERE", "int"),
  ("liability.founder_multiplier", "DEFAULT_LIABILITY_FOUNDER_MULTIPLIER", "float"),
  ("liability.regular_multiplier", "DEFAULT_LIABILITY_REGULAR_MULTIPLIER", "float"),
  ("liability.sponsor_liability_floor", "DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR", "int"),
  ("report.base_weight", "DEFAULT_REPORT_BASE_WEIGHT", "float"),
  ("report.clamp_min", "DEFAULT_REPORT_CLAMP_MIN", "float"),
  ("report.clamp_max", "DEFAULT_REPORT_CLAMP_MAX", "float"),
  ("report.recency_half_life_hours", "DEFAULT_REPORT_RECENCY_HALF_LIFE_HOURS", "float"),
  ("report.case_threshold_micros", "DEFAULT_REPORT_CASE_THRESHOLD_MICROS", "int"),
  ("decay.positive_half_life_days", "DEFAULT_DECAY_POSITIVE_HALF_LIFE_DAYS", "int"),
  ("onboarding.default_membership_state", "DEFAULT_ONBOARDING_DEFAULT_MEMBERSHIP_STATE", "text"),
  ("onboarding.sponsor_gate_strategy", "DEFAULT_ONBOARDING_SPONSOR_GATE_STRATEGY", "text"),
  ("onboarding.sponsor_min_account_age_days", "DEFAULT_ONBOARDING_SPONSOR_MIN_ACCOUNT_AGE_DAYS", "int"),
  ("founder.max_founders_active", "DEFAULT_FOUNDER_MAX_FOUNDERS_ACTIVE", "int"),
  ("founder.max_expires_days", "DEFAULT_FOUNDER_MAX_EXPIRES_DAYS", "int"),
  ("founder.max_seed_delta", "DEFAULT_FOUNDER_MAX_SEED_DELTA", "int"),
  ("job.snapshot_interval_seconds", "DEFAULT_JOB_SNAPSHOT_INTERVAL_SECONDS", "int"),
  ("job.snapshot_batch_chunk_size", "DEFAULT_JOB_SNAPSHOT_BATCH_CHUNK_SIZE", "int"),
];

/// 34 after Perplexity-review 2026-04-17 added `job.snapshot_batch_chunk_size`
/// (GOTCHA-50f). The parity test asserts the `SEEDED_KEYS_WITH_CONSTS` length
/// matches this count.
pub const EXPECTED_SEED_COUNT: usize = 34;

#[cfg(test)]
mod parity {
  use super::*;

  #[test]
  fn seeded_keys_count_matches_const_count() {
    assert_eq!(
      SEEDED_KEYS_WITH_CONSTS.len(),
      EXPECTED_SEED_COUNT,
      "SEEDED_KEYS_WITH_CONSTS length ({}) must equal EXPECTED_SEED_COUNT ({}) — add/remove keys \
       in both places when changing the seed list",
      SEEDED_KEYS_WITH_CONSTS.len(),
      EXPECTED_SEED_COUNT,
    );
  }

  /// Ensure every seeded key has a matching `const_default_*` fallback that
  /// returns `Some`. Catches the class of bug where the seed list and the
  /// const-default lookup tables drift.
  #[test]
  fn every_seeded_key_has_const_fallback() {
    for (key, _const_name, vtype) in SEEDED_KEYS_WITH_CONSTS {
      let resolved = match *vtype {
        "int" => const_default_int(key).is_some(),
        "float" => const_default_float(key).is_some(),
        "bool" => const_default_bool(key).is_some(),
        "text" => const_default_text(key).is_some(),
        other => panic!("unknown value_type '{other}' for key '{key}'"),
      };
      assert!(
        resolved,
        "seeded key `{key}` (type {vtype}) has no matching Rust const fallback — add it to \
         const_default_{vtype}"
      );
    }
  }
}
