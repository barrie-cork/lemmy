//! Governance Create-wrapper activities.
//!
//! This module hosts the `Activity` trait implementations for the three
//! Brehon governance Create wrappers:
//!
//! - `PublishSanctionNotice` — broadcast that a finalised case applied
//!   a sanction (Phase 6 task 73 type, task 74 outbound, task 75 inbound)
//! - `PublishTrustAttestation` — broadcast that a person gained or lost
//!   a trust attestation (plumbed but unwired in v0)
//! - `PublishLabel` — broadcast a moderation label (stub, not wired in
//!   v0; reserved for v1 to avoid `SharedInboxActivities` churn later)
//!
//! See plan §task 73 for the layered execution rationale and ADR-014 for
//! the fork-only AP type policy.
//!
//! The [`inbox`] module hosts the inbound receiver bodies that the
//! `Activity::receive` impls in `publish_sanction_notice` and
//! `publish_trust_attestation` delegate to. It lives in this crate (not
//! in `lemmy_apub`) per advisor decision DQ-6.6-inbound — see the module
//! doc on `inbox.rs` for the dep-graph rationale.

pub mod inbox;
pub mod publish_label;
pub mod publish_sanction_notice;
pub mod publish_trust_attestation;
