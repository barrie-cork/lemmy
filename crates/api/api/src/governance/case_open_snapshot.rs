//! Case-open config snapshot — pins the 7 `requires_re_jury` keys at
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

/// Read the 7 `requires_re_jury` keys at `scope` and return a JSONB-ready
/// object mapping each key to its effective value.
///
/// Numeric keys are stored as `i64`, bool as `bool`, enum/text keys as
/// their raw string value (e.g. `"majority"`, `"60%"`) — `config::get_text`
/// is the correct reader for `ValueType::Enum`, NOT `serde_json`.
/// Consumers in jury-mechanics-v1 dispatch on `.as_i64()` / `.as_bool()` /
/// `.as_str()` based on the key.
pub async fn build_applied_config_snapshot(
  pool: &mut DbPool<'_>,
  scope: Scope,
) -> LemmyResult<Value> {
  let mut cache = ConfigCache::new();
  let panel_size = config::get_int(&mut cache, pool, scope, "jury.panel_size").await?;
  let quorum = config::get_int(&mut cache, pool, scope, "jury.quorum").await?;
  let sev_minor =
    config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.minor").await?;
  let sev_moderate =
    config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.moderate").await?;
  let sev_severe =
    config::get_text(&mut cache, pool, scope, "jury.severity_thresholds.severe").await?;
  let diversity_enabled =
    config::get_bool(&mut cache, pool, scope, "jury.diversity_constraints_enabled").await?;
  let appeal_increase =
    config::get_int(&mut cache, pool, scope, "jury.appeal_panel_size_increase").await?;

  Ok(json!({
    "jury.panel_size":                    panel_size,
    "jury.quorum":                        quorum,
    "jury.severity_thresholds.minor":     sev_minor,
    "jury.severity_thresholds.moderate":  sev_moderate,
    "jury.severity_thresholds.severe":    sev_severe,
    "jury.diversity_constraints_enabled": diversity_enabled,
    "jury.appeal_panel_size_increase":    appeal_increase,
  }))
}

#[cfg(test)]
mod parity {
  use crate::governance::config::CONFIG_KEY_METADATA;

  /// The canonical list of the 7 keys with `requires_re_jury: true` in
  /// `CONFIG_KEY_METADATA`. Kept alongside the test because the helper's
  /// `json!({...})` block is the canonical authority at runtime; this
  /// list is test-only drift-guard. If the metadata adds a new
  /// `requires_re_jury` key without the snapshot helper's `json!` being
  /// updated, the test fails and prevents the pin from drifting silently.
  const REQUIRES_RE_JURY_KEYS: &[&str] = &[
    "jury.panel_size",
    "jury.quorum",
    "jury.severity_thresholds.minor",
    "jury.severity_thresholds.moderate",
    "jury.severity_thresholds.severe",
    "jury.diversity_constraints_enabled",
    "jury.appeal_panel_size_increase",
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
