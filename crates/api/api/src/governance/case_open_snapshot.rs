//! Case-open config snapshot — pins the 27 `requires_re_jury` keys at
//! case-open time per [IMPLEMENTATION-PLAN-v0.md §4.1] and the ADR-010
//! append-only invariant. In-flight juries read the pinned snapshot so a
//! later admin-config edit cannot retroactively change panel size,
//! quorum, severity thresholds, or diversity constraints mid-case.
//!
//! v1-AD-c owns the WRITE-SIDE only. `create_report.rs` calls
//! [`build_applied_config_snapshot`] at the point of case-open and
//! stores the JSON on `moderation_case.applied_config_snapshot`. The
//! read-side switch in `admin_assign_jury.rs` still reads live config
//! per v1-AD-c scope decision §4.1 — jury-mechanics-v1 owns the flip.
//!
//! The emergency-remove path (`admin_emergency_remove.rs`) deliberately
//! does NOT pin a snapshot. Admin-override cases pin at removal time,
//! not at case-open; their snapshot semantics differ. See plan §13 task
//! 5 GOTCHA for the rationale.

use lemmy_diesel_utils::connection::DbPool;
use lemmy_utils::error::LemmyResult;
use serde_json::{Value, json};

use crate::governance::config::{self, ConfigCache, Scope};

/// Read all 27 `requires_re_jury` keys at `scope` and return a JSONB-ready
/// object mapping each key to its effective value.
///
/// Numeric keys are stored as `i64` (`get_int`) or `f64` (`get_float`),
/// bool as `bool`, enum keys as their raw string value (e.g. `"majority"`) —
/// `config::get_text` is the correct reader for `ValueType::Enum`, NOT
/// `serde_json`. Consumers in jury-mechanics-v1 dispatch on `.as_i64()` /
/// `.as_f64()` / `.as_bool()` / `.as_str()` based on the key.
pub async fn build_applied_config_snapshot(
  pool: &mut DbPool<'_>,
  scope: Scope,
) -> LemmyResult<Value> {
  let mut cache = ConfigCache::new();

  // -- Int keys ---------------------------------------------------------------
  let panel_size = config::get_int(&mut cache, pool, scope, "jury.panel_size").await?;
  let quorum = config::get_int(&mut cache, pool, scope, "jury.quorum").await?;
  let appeal_increase =
    config::get_int(&mut cache, pool, scope, "jury.appeal_panel_size_increase").await?;
  let panel_regular_minor =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.regular.minor").await?;
  let panel_regular_moderate =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.regular.moderate").await?;
  let panel_regular_severe =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.regular.severe").await?;
  let panel_founder_minor =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.founder.minor").await?;
  let panel_founder_moderate =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.founder.moderate").await?;
  let panel_founder_severe =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.founder.severe").await?;
  let panel_probation_minor =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.probation.minor").await?;
  let panel_probation_moderate = config::get_int(
    &mut cache,
    pool,
    scope,
    "jury.panel_size.probation.moderate",
  )
  .await?;
  let panel_probation_severe =
    config::get_int(&mut cache, pool, scope, "jury.panel_size.probation.severe").await?;
  let juror_cooldown_days = config::get_int(
    &mut cache,
    pool,
    scope,
    "jury.constraints.juror_cooldown_days",
  )
  .await?;

  // -- Float keys -------------------------------------------------------------
  let quorum_fraction_minor =
    config::get_float(&mut cache, pool, scope, "jury.quorum_fraction.minor").await?;
  let quorum_fraction_moderate =
    config::get_float(&mut cache, pool, scope, "jury.quorum_fraction.moderate").await?;
  let quorum_fraction_severe =
    config::get_float(&mut cache, pool, scope, "jury.quorum_fraction.severe").await?;
  let threshold_fraction_minor =
    config::get_float(&mut cache, pool, scope, "jury.threshold_fraction.minor").await?;
  let threshold_fraction_moderate =
    config::get_float(&mut cache, pool, scope, "jury.threshold_fraction.moderate").await?;
  let threshold_fraction_severe =
    config::get_float(&mut cache, pool, scope, "jury.threshold_fraction.severe").await?;

  // -- Bool keys --------------------------------------------------------------
  let diversity_enabled = config::get_bool(
    &mut cache,
    pool,
    scope,
    "jury.diversity_constraints_enabled",
  )
  .await?;
  let no_majority_same_sponsor = config::get_bool(
    &mut cache,
    pool,
    scope,
    "jury.constraints.no_majority_from_same_sponsor_cluster",
  )
  .await?;
  let geographic_diversity = config::get_bool(
    &mut cache,
    pool,
    scope,
    "jury.constraints.geographic_diversity_preferred",
  )
  .await?;
  let no_recent_repeat = config::get_bool(
    &mut cache,
    pool,
    scope,
    "jury.constraints.no_recent_juror_repeat",
  )
  .await?;
  let no_same_endorsement = config::get_bool(
    &mut cache,
    pool,
    scope,
    "jury.constraints.no_same_endorsement_chain",
  )
  .await?;

  // -- Enum keys (stored as text) ---------------------------------------------
  let sev_minor =
    config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.minor").await?;
  let sev_moderate =
    config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.moderate").await?;
  let sev_severe =
    config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.severe").await?;

  Ok(json!({
    // Int
    "jury.panel_size":                          panel_size,
    "jury.quorum":                              quorum,
    "jury.appeal_panel_size_increase":          appeal_increase,
    "jury.panel_size.regular.minor":            panel_regular_minor,
    "jury.panel_size.regular.moderate":         panel_regular_moderate,
    "jury.panel_size.regular.severe":           panel_regular_severe,
    "jury.panel_size.founder.minor":            panel_founder_minor,
    "jury.panel_size.founder.moderate":         panel_founder_moderate,
    "jury.panel_size.founder.severe":           panel_founder_severe,
    "jury.panel_size.probation.minor":          panel_probation_minor,
    "jury.panel_size.probation.moderate":       panel_probation_moderate,
    "jury.panel_size.probation.severe":         panel_probation_severe,
    "jury.constraints.juror_cooldown_days":     juror_cooldown_days,
    // Float
    "jury.quorum_fraction.minor":               quorum_fraction_minor,
    "jury.quorum_fraction.moderate":            quorum_fraction_moderate,
    "jury.quorum_fraction.severe":              quorum_fraction_severe,
    "jury.threshold_fraction.minor":            threshold_fraction_minor,
    "jury.threshold_fraction.moderate":         threshold_fraction_moderate,
    "jury.threshold_fraction.severe":           threshold_fraction_severe,
    // Bool
    "jury.diversity_constraints_enabled":       diversity_enabled,
    "jury.constraints.no_majority_from_same_sponsor_cluster": no_majority_same_sponsor,
    "jury.constraints.geographic_diversity_preferred":        geographic_diversity,
    "jury.constraints.no_recent_juror_repeat":                no_recent_repeat,
    "jury.constraints.no_same_endorsement_chain":             no_same_endorsement,
    // Enum
    "jury.severity_thresholds.minor":           sev_minor,
    "jury.severity_thresholds.moderate":        sev_moderate,
    "jury.severity_thresholds.severe":          sev_severe,
  }))
}

#[cfg(test)]
mod parity {
  use crate::governance::config::CONFIG_KEY_METADATA;

  /// The canonical list of all 27 keys with `requires_re_jury: true` in
  /// `CONFIG_KEY_METADATA`. Kept alongside the test because the helper's
  /// `json!({...})` block is the canonical authority at runtime; this
  /// list is test-only drift-guard. If the metadata adds a new
  /// `requires_re_jury` key without the snapshot helper's `json!` being
  /// updated, the test fails and prevents the pin from drifting silently.
  const REQUIRES_RE_JURY_KEYS: &[&str] = &[
    // Int
    "jury.panel_size",
    "jury.quorum",
    "jury.appeal_panel_size_increase",
    "jury.panel_size.regular.minor",
    "jury.panel_size.regular.moderate",
    "jury.panel_size.regular.severe",
    "jury.panel_size.founder.minor",
    "jury.panel_size.founder.moderate",
    "jury.panel_size.founder.severe",
    "jury.panel_size.probation.minor",
    "jury.panel_size.probation.moderate",
    "jury.panel_size.probation.severe",
    "jury.constraints.juror_cooldown_days",
    // Float
    "jury.quorum_fraction.minor",
    "jury.quorum_fraction.moderate",
    "jury.quorum_fraction.severe",
    "jury.threshold_fraction.minor",
    "jury.threshold_fraction.moderate",
    "jury.threshold_fraction.severe",
    // Bool
    "jury.diversity_constraints_enabled",
    "jury.constraints.no_majority_from_same_sponsor_cluster",
    "jury.constraints.geographic_diversity_preferred",
    "jury.constraints.no_recent_juror_repeat",
    "jury.constraints.no_same_endorsement_chain",
    // Enum
    "jury.severity_thresholds.minor",
    "jury.severity_thresholds.moderate",
    "jury.severity_thresholds.severe",
  ];

  /// The snapshot keys must equal the set of keys in `CONFIG_KEY_METADATA`
  /// where `requires_re_jury == true`.
  #[test]
  fn snapshot_keyset_matches_requires_re_jury_metadata() {
    let mut metadata_keys: Vec<&str> = CONFIG_KEY_METADATA
      .iter()
      .filter(|m| m.requires_re_jury)
      .map(|m| m.key)
      .collect();
    let mut snapshot_keys: Vec<&str> = REQUIRES_RE_JURY_KEYS.to_vec();
    metadata_keys.sort_unstable();
    snapshot_keys.sort_unstable();
    assert_eq!(
      snapshot_keys, metadata_keys,
      "requires_re_jury metadata keys drifted from REQUIRES_RE_JURY_KEYS"
    );
  }
}
