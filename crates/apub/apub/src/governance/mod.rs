//! Governance federation glue.
//!
//! Houses the cross-cutting helpers that bridge between
//! `lemmy_apub_activities::governance` (the AP wire types and trait impls)
//! and the rest of the lemmy stack:
//!
//! - `verify` — thin wrappers over `activitypub_federation`'s domain
//!   verification helpers, providing a stable extension point for v1
//!   custom verification (jury-panel attestation strength, etc.) without
//!   churning callers ([04 §11]).
//!
//! See plan §task 78.
//!
//! NOTE (DQ-6.6 pending): the inbound receiver (`receive_remote_sanction_notice`)
//! was originally planned for `inbox.rs` here, but the `Activity::receive`
//! trait body lives in `lemmy_apub_activities` (lower crate) and cannot
//! call up. Resolution awaits advisor decision; once landed, either an
//! `inbox.rs` re-export module appears here or the receiver lives in
//! `lemmy_apub_activities::governance::inbox` directly.

pub mod verify;
