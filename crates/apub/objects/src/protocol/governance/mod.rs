//! AP protocol structs for governance objects (Phase 6).
//!
//! These mirror Lemmy's `protocol/note.rs` shape: a serde struct with
//! `#[serde(rename_all = "camelCase")]` and a single-variant `kind` enum
//! that serializes to the literal AP `type` discriminator.
//!
//! Outbound-only in v0 — Phase 6 task 74 (Agent D) wires `into_json`;
//! Phase 6 task 75 (Agent E) wires `from_json` for the inbound path.
//! See [docs/brehon-law-inspired-network/04-data-model-and-api.md §11].

pub mod moderation_label;
pub mod sanction_notice;
pub mod trust_attestation;
