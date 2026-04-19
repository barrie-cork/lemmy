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
//! TODO(merge-1b): once Agent B has landed
//! `lemmy_apub_objects::protocol::governance::{SanctionNoticeProtocol,
//! TrustAttestationProtocol, ModerationLabelProtocol}`, replace
//! `serde_json::Value` on each `object` field with the typed struct.

pub mod publish_label;
pub mod publish_sanction_notice;
pub mod publish_trust_attestation;
