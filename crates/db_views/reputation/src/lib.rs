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
use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot;
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
///
/// **Wire-silent raw dimension scores.** Per ADR-005 the public API surfaces
/// capabilities (`jury_eligible`, `trusted_reporter`), not numbers. The four
/// raw score fields (`reporting_accuracy`, `jury_reliability`,
/// `participation_consistency`, `endorsement_strength`) retain their types
/// for in-process consumers (tests, admin-only paths) but are `#[serde(skip)]`
/// + `#[ts(skip)]` so any accidental serialisation of this view through a
/// public handler will not leak them.
pub struct ReputationSummaryView {
  pub person_id: i32,
  pub community_id: Option<i32>,
  #[serde(skip)]
  #[cfg_attr(feature = "ts-rs", ts(skip))]
  pub reporting_accuracy: i32,
  #[serde(skip)]
  #[cfg_attr(feature = "ts-rs", ts(skip))]
  pub jury_reliability: i32,
  #[serde(skip)]
  #[cfg_attr(feature = "ts-rs", ts(skip))]
  pub participation_consistency: i32,
  #[serde(skip)]
  #[cfg_attr(feature = "ts-rs", ts(skip))]
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

/// Map a freshly-loaded `ReputationSnapshot` row into the public view shape.
/// `active_sanctions` is set to `0` here — the caller overwrites it with the
/// `count_active_sanctions` round-trip per the standard read pattern. This
/// impl deliberately does NOT carry `can_sponsor` over from the snapshot
/// per [99 OQ-014] / `scripts/brehon/lint-no-can-sponsor-read.sh`.
impl From<&ReputationSnapshot> for ReputationSummaryView {
  fn from(snapshot: &ReputationSnapshot) -> Self {
    Self {
      person_id: snapshot.person_id.0,
      community_id: snapshot.community_id.map(|c| c.0),
      reporting_accuracy: snapshot.reporting_accuracy,
      jury_reliability: snapshot.jury_reliability,
      participation_consistency: snapshot.participation_consistency,
      endorsement_strength: snapshot.endorsement_strength,
      jury_eligible: snapshot.jury_eligible,
      trusted_reporter: snapshot.trusted_reporter,
      calculated_at: snapshot.calculated_at,
      active_sanctions: 0,
    }
  }
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// Per-person endorsement / surety aggregate. All four counts are derived
/// via separate `SELECT COUNT(*)` round-trips — no source columns exist on
/// any single table.
///
/// **Split surety direction.** `list_endorsements_for_person` populates
/// `active_sureties_inbound` (rows where the subject is the sponsee);
/// `list_sureties_for_person` populates `active_sureties_outbound` (rows
/// where the subject is the sponsor). The two directions are opposite sides
/// of the same `surety` table, so a single `active_sureties` field would be
/// overloaded with different semantics depending on which function produced
/// the view — disambiguating at the shape level keeps consumers from
/// misinterpreting the count.
pub struct EndorsementSummaryView {
  pub person_id: i32,
  pub inbound_endorsements: i64,
  pub outbound_endorsements: i64,
  /// Sureties where this person is the **sponsee** (`surety.sponsored_id`).
  /// Populated by `list_endorsements_for_person`; `0` from
  /// `list_sureties_for_person`.
  pub active_sureties_inbound: i64,
  /// Sureties where this person is the **sponsor** (`surety.sponsor_id`).
  /// Populated by `list_sureties_for_person`; `0` from
  /// `list_endorsements_for_person`.
  pub active_sureties_outbound: i64,
}
