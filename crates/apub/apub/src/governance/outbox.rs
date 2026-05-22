//! Public-API re-exports for governance outbound publishers (Phase 6
//! task 74 — Agent D).
//!
//! Per [docs/brehon-law-inspired-network/04-data-model-and-api.md §11],
//! consumers (the API handlers in `lemmy_api`, the optional scheduled
//! jobs in `lemmy_server`) reach the publishers through this module.
//! The actual implementations live in
//! `lemmy_apub_activities::governance::publish_sanction_notice` and
//! `…::publish_trust_attestation`; this file is a thin façade so the
//! consuming crates (`lemmy_api`, `lemmy_server`) only ever depend on
//! `lemmy_apub`, never directly on `lemmy_apub_activities`.
//!
//! ## Architectural note
//!
//! DQ-6.6 (resolved 2026-04-19 as `impl-self-resolved`) settled on
//! splitting Phase 6's outbound publishers into a builder/orchestrator
//! pair. The functions re-exported here are **builders** — they perform
//! DB reads only and return a `*SendPlan` describing what to send. The
//! orchestrator (Agent F's `crate::governance::federation_outbox` module
//! inside `lemmy_api`) opens a `conn.run_transaction`, calls
//! [`enqueue_sanction_notice_activity`] /
//! [`enqueue_trust_attestation_activity`] on the tx conn, then calls
//! `lemmy_api::governance::governance_log::append` on the same conn.
//! The transaction guarantees that the `sent_activity` INSERT and the
//! `governance_log` append commit atomically, satisfying [06 §2.3]
//! ("before the user response returns").
//!
//! See the head-of-module comment in
//! `lemmy_apub_activities::governance::publish_sanction_notice` for the
//! full DQ-6.6 framing.

pub use lemmy_apub_activities::governance::publish_sanction_notice::{
  SanctionNoticeSendPlan, build_local_sanction_notice_plan, enqueue_sanction_notice_activity,
};
pub use lemmy_apub_activities::governance::publish_trust_attestation::{
  TrustAttestationSendPlan, build_local_trust_attestation_plan, enqueue_trust_attestation_activity,
};

// The names below are aliases for the builders; they exist so the
// outbox boundary preserves the brief's `send_local_*_notice` /
// `send_local_*_attestation` naming. The `_plan` suffix on the canonical
// names makes the builder/orchestrator split visible at every call site.
pub use lemmy_apub_activities::governance::publish_sanction_notice::send_local_sanction_notice_plan;
pub use lemmy_apub_activities::governance::publish_trust_attestation::send_local_trust_attestation_plan;
