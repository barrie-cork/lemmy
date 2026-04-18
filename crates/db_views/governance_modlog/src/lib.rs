//! Read model for the public redacted moderation log.
//!
//! Phase 2b ships one plain-struct view per [04 §4.4]:
//!
//! * [`GovernanceModlogView`] — one row per published `public_case_log`
//!   entry, enriched with community name and (in Phase 2b) drift stubs
//!   for `decision`, `sanction_action`, and `appealed`. The struct is a
//!   plain Rust type (no `Queryable`/`Selectable` derives); impls build
//!   it by mapping an explicit Diesel tuple select plus a second
//!   round-trip that resolves `appealed` via an `IN`-list lookup on
//!   `appeal` — same two-round-trip pattern as
//!   `lemmy_db_views_governance_case`'s `submitted_counts_by_case`
//!   helper.
//!
//! **Not the same as `lemmy_db_views_modlog`.** That is upstream Lemmy's
//! content-moderation log (`ModlogCombinedView`). `governance_modlog` is
//! the Brehon public case log — a separate table, a separate view, a
//! separate ADR chain.
//!
//! **ADR-015 redaction lives elsewhere.** Every string that lands in
//! `public_case_log.summary` / `public_case_log.rationale_redacted` is
//! scrubbed at Phase 4 write-time by the `redaction::scrub` wrapper.
//! This view crate reads the columns as-is — no re-scrubbing at read
//! time, no `actor_pseudonym` join in the canonical shape per [04 §4.4].

use chrono::{DateTime, Utc};
use lemmy_db_schema_file::enums::{JuryDecision, SanctionAction};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
/// One row of the public redacted moderation log per [04 §4.4].
///
/// Plain Rust struct, NOT a Diesel `Queryable`/`Selectable`. Built by
/// mapping an explicit tuple select plus an `appealed` aggregation in
/// the impls module — see `impls::list_public_case_log` for the
/// canonical shape.
pub struct GovernanceModlogView {
  pub case_id: i32,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  /// [04 §4.4] drift — no source in Phase 1 schema (tally lives in
  /// Phase 4 jury_vote aggregation). Always `None` in Phase 2b; see
  /// plan §2.1.
  pub decision: Option<JuryDecision>,
  /// [04 §4.4] drift — no source in Phase 1 schema (sanction rows
  /// written by Phase 4 task 42). Always `None` in Phase 2b; see plan
  /// §2.2.
  pub sanction_action: Option<SanctionAction>,
  pub summary: String,
  pub published_at: DateTime<Utc>,
  /// Derived via a separate round-trip in impls — true if any
  /// `appeal` row exists with `appeal.case_id = public_case_log.case_id`.
  /// See plan §2.3.
  pub appealed: bool,
}

/// One row of the `governance_log` table surfaced as a capability-change
/// observability entry. Phase 5c task 63 ships this view alongside the
/// existing `GovernanceModlogView` so admin tooling (and v1's admin
/// dashboard) can list capability gains/losses without re-deriving the
/// payload-shape parsing across consumers.
///
/// Read directly from `governance_log` (NOT from `public_case_log`) per
/// plan §11.3 GOTCHA — `capability_changed` entries are governance-log
/// signals, not public-case-log surfaces.
///
/// **No revocation of in-flight jury assignments.** Per
/// IMPLEMENTATION-PLAN-v0.md line 375, `Selected`/`Accepted`
/// jury_assignment rows are NOT recalled when a threshold edit drops
/// the assignee below `jury_eligible`. Snapshot recompute affects
/// **future** selections only. Consumers reading these entries should
/// not assume one entry corresponds to a panel-membership change.
#[skip_serializing_none]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
pub struct CapabilityChangeLogEntry {
  /// `governance_log.id` (Int8). Carry as i64 so `since_id` paging shape
  /// matches the underlying column.
  pub id: i64,
  pub entry_kind: String,
  pub payload: Value,
  pub actor_pseudonym: Option<String>,
  pub created_at: DateTime<Utc>,
  pub signature: Option<Vec<u8>>,
}
