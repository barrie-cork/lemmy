//! Backwards-compatible re-export of the identifier scrubber.
//!
//! ## History
//!
//! Until Phase 6 the [`scrub`] string helper + [`scrub_json`] tree walk
//! lived here in `crates/api/api/src/governance/redaction.rs`. Phase 6
//! task 75's federation receiver lifted [`scrub_json`] (transitively
//! through [`crate::governance::governance_log::append`]) into
//! `lemmy_db_schema::source::governance::governance_log`, which forced
//! the redaction helpers down to
//! `lemmy_db_schema::source::governance::redaction` so the writer could
//! reach them without a circular dep. See the matching
//! `crate::governance::governance_log` module doc and DQ-6.6-inbound
//! (resolved id 37 in `.claude/decision-queue.json`) for the full
//! rationale.
//!
//! This module is now a thin re-export so the existing call sites in
//! `submit_jury_vote.rs` (and any future callers in this crate) continue
//! to use the familiar `crate::governance::redaction::scrub` /
//! `scrub_json` paths.

pub use lemmy_db_schema::source::governance::redaction::{scrub, scrub_json};
