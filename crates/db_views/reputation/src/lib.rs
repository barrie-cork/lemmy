//! Read models for reputation + endorsement aggregates.
//!
//! Phase 5a task 52 ships two plain-struct views per [04 §4.3]:
//!
//! * [`ReputationSummaryView`] — one row per snapshot-person-community
//!   aggregate, with `active_sanctions: i64` derived via a second round-trip
//!   on `sanction`. Cannot derive `Selectable` because `active_sanctions` is
//!   a bare-scalar kind (c) field per
//!   `.claude/rules/view-crate-selectable-template.md` — tuple-load +
//!   `build_view` is the only shape that compiles.
//! * [`EndorsementSummaryView`] — per-person endorsement / surety counts.
//!
//! **`can_sponsor` is intentionally NOT in `ReputationSummaryView`.** v0
//! does not read it ([99 OQ-014]); v1 adds the boolean as a fourth field
//! when `config.sponsorship.require_reputation_gate` flips. Meanwhile
//! `scripts/brehon/lint-no-can-sponsor-read.sh` (task 51 sibling) fails the
//! 5a phase-close if any crate reads the column outside the authorised
//! writer/schema sites.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// Per-(person, community) reputation snapshot plus a derived
/// `active_sanctions` count. Built by the impls module via tuple-load on
/// `reputation_snapshot` + a second round-trip for the count.
pub struct ReputationSummaryView {
  pub person_id: i32,
  pub community_id: Option<i32>,
  pub reporting_accuracy: i32,
  pub jury_reliability: i32,
  pub participation_consistency: i32,
  pub endorsement_strength: i32,
  pub jury_eligible: bool,
  pub trusted_reporter: bool,
  pub calculated_at: DateTime<Utc>,
  /// Derived via a separate `SELECT COUNT(*) FROM sanction ...` round-trip
  /// at read time — the column has no source in `reputation_snapshot`. The
  /// bare-scalar kind (c) shape is why this struct can't derive
  /// `Selectable`.
  pub active_sanctions: i64,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// Per-person endorsement / surety aggregate. All three counts are derived
/// via separate `SELECT COUNT(*)` round-trips — no source columns exist on
/// any single table.
pub struct EndorsementSummaryView {
  pub person_id: i32,
  pub inbound_endorsements: i64,
  pub outbound_endorsements: i64,
  pub active_sureties: i64,
}
