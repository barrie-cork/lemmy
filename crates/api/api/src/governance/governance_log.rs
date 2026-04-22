//! Backwards-compatible re-export of the governance log writer + entry kinds.
//!
//! ## History
//!
//! Until Phase 6 the [`append`] writer + entry-kind constants + signing
//! helper lived here in `crates/api/api/src/governance/governance_log.rs`.
//! The model itself (`GovernanceLog` + `GovernanceLogInsertForm`) lived
//! in `lemmy_db_schema::source::governance::governance_log`.
//!
//! Phase 6 task 75's inbound federation receiver
//! (`crates/apub/activities/src/governance/inbox.rs`) needs to call
//! [`append`] from the `lemmy_apub_activities` crate, which is below
//! `lemmy_api` in the dep graph. Lifting the call site to a higher crate
//! is impossible: `Activity::receive` is invoked by the AP framework
//! directly at the `lemmy_apub_activities` layer with no opportunity for
//! a higher-crate orchestrator (unlike outbound publishing, which has the
//! `submit_jury_vote.rs` orchestrator in `lemmy_api`).
//!
//! Per advisor decision DQ-6.6-inbound (resolved id 37 in
//! `.claude/decision-queue.json`) the writer + entry-kind constants +
//! signing helper moved DOWN to
//! `lemmy_db_schema::source::governance::governance_log`. This module is
//! now a thin re-export so the ~14 existing `governance_log::append` /
//! `governance_log::ENTRY_KIND_*` call sites in this crate (and in
//! `lemmy_api_crud`, `lemmy_server`, `lemmy_apub`, the seed_founders
//! tool, etc.) continue to compile unchanged.
//!
//! ## What moved
//!
//! - `pub async fn append(...)` — see
//!   [`lemmy_db_schema::source::governance::governance_log::append`].
//! - All `pub const ENTRY_KIND_*` constants — same path.
//! - `fn load_signing_key()` — moved (private).
//! - `scrub_json` import — now imported from
//!   `lemmy_db_schema::source::governance::redaction` (see
//!   `crate::governance::redaction` for the matching string-helper
//!   re-export).

pub use lemmy_db_schema::source::governance::governance_log::{
  ENTRY_KIND_ADMIN_CONFIG_CHANGED,
  ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
  ENTRY_KIND_APPEAL_REQUESTED,
  ENTRY_KIND_CAPABILITY_CHANGED,
  ENTRY_KIND_CASE_DECIDED,
  ENTRY_KIND_EMERGENCY_REMOVED,
  ENTRY_KIND_ENDORSEMENT_CREATED,
  ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED,
  ENTRY_KIND_FEDERATION_ATTESTATION_SENT,
  ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
  ENTRY_KIND_FEDERATION_SANCTION_SENT,
  ENTRY_KIND_FOUNDER_SEEDED,
  ENTRY_KIND_JURY_ACCEPTED,
  ENTRY_KIND_JURY_ASSIGNED,
  ENTRY_KIND_JURY_DECLINED,
  ENTRY_KIND_JURY_REPLACEMENT_SELECTED,
  ENTRY_KIND_JURY_VOTED,
  ENTRY_KIND_PANEL_ASSEMBLED,
  ENTRY_KIND_PUBLIC_LOG_PUBLISHED,
  ENTRY_KIND_REPORT_CREATED,
  ENTRY_KIND_REPUTATION_DELTA,
  ENTRY_KIND_RULE_SET_VERSION_CREATED,
  ENTRY_KIND_SANCTION_CREATED,
  ENTRY_KIND_SPONSOR_LIABILITY_APPLIED,
  ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED,
  ENTRY_KIND_THRESHOLD_MET,
  GovernanceLog,
  GovernanceLogInsertForm,
  append,
};
