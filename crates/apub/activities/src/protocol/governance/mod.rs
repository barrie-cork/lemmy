//! Protocol-layer (serde) types for governance Create-wrapper activities.
//!
//! Each `Publish*` activity defined here wraps a governance object
//! (sanction notice, trust attestation, label) as the `object` field of an
//! ActivityStreams `Create` activity. The wrapped object's typed protocol
//! struct lives in `lemmy_apub_objects::protocol::governance::*`.
//!
//! For Phase 6 Agent C we use `serde_json::Value` for the `object` field
//! because Agent B is creating the typed object protocols in parallel
//! (see plan §11 line 882 — separate worktrees, no shared files in
//! Layer 2). Advisor may rewire to typed structs after the layer-2 merge
//! if it improves type safety.
//!
//! v2-cleanup: the typed protocol structs (`SanctionNoticeProtocol`,
//! `TrustAttestationProtocol`, `ModerationLabelProtocol`) exist in
//! `lemmy_apub_objects::protocol::governance::*` (landed `aa159a0f4`).
//! Replacing the `serde_json::Value` stub fields with the typed structs
//! is deferred to v2 (requires Activity-Protocol layer refactor).

pub mod publish_label;
pub mod publish_sanction_notice;
pub mod publish_trust_attestation;
