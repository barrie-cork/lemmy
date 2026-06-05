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
  ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED, ENTRY_KIND_ADMIN_CONFIG_CHANGED,
  ENTRY_KIND_APPEAL_DECIDED, ENTRY_KIND_APPEAL_PANEL_ASSEMBLED, ENTRY_KIND_APPEAL_REJECTED,
  ENTRY_KIND_APPEAL_REQUESTED, ENTRY_KIND_APPEAL_WINDOW_EXPIRED, ENTRY_KIND_CAPABILITY_CHANGED,
  ENTRY_KIND_CASE_DECIDED, ENTRY_KIND_DECAY_KNOB_CHANGED, ENTRY_KIND_EMERGENCY_REMOVED,
  ENTRY_KIND_ENDORSEMENT_CREATED, ENTRY_KIND_ENDORSEMENT_REVOKED,
  ENTRY_KIND_EVIDENCE_QUALITY_RECORDED, ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED,
  ENTRY_KIND_FEDERATION_ATTESTATION_SENT, ENTRY_KIND_FEDERATION_INBOUND_BLOCKED,
  ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE,
  ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR,
  ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER,
  ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY, ENTRY_KIND_FEDERATION_INBOUND_DROPPED_SCHEMA,
  ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED,
  ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED, ENTRY_KIND_FEDERATION_LABEL_RECEIVED,
  ENTRY_KIND_FEDERATION_PEER_TRUST_CHANGED, ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
  ENTRY_KIND_FEDERATION_SANCTION_SENT, ENTRY_KIND_FOUNDER_SEEDED, ENTRY_KIND_JURY_ACCEPTED,
  ENTRY_KIND_JURY_ASSIGNED, ENTRY_KIND_JURY_CONSTRAINT_RELAXED, ENTRY_KIND_JURY_DEADLOCK,
  ENTRY_KIND_JURY_DECLINED, ENTRY_KIND_JURY_REPLACEMENT_SELECTED, ENTRY_KIND_JURY_VOTED,
  ENTRY_KIND_PANEL_ASSEMBLED, ENTRY_KIND_PARTICIPATION_CRON_TICK, ENTRY_KIND_PUBLIC_LOG_PUBLISHED,
  ENTRY_KIND_REPORT_CREATED, ENTRY_KIND_REPUTATION_DELTA, ENTRY_KIND_RESTORATION_COMPLETED,
  ENTRY_KIND_ROLLUP_RECOMPUTED, ENTRY_KIND_ROOM_ARCHIVED, ENTRY_KIND_ROOM_BRIDGE_ERROR,
  ENTRY_KIND_ROOM_CREATED, ENTRY_KIND_ROOM_DECISION_RELAYED, ENTRY_KIND_ROOM_IDENTITY_REVEALED,
  ENTRY_KIND_ROOM_LIFECYCLE_EVENT, ENTRY_KIND_ROOM_MEMBER_ADDED, ENTRY_KIND_ROOM_MEMBER_REMOVED,
  ENTRY_KIND_ROOM_RECORDING_UPLOADED, ENTRY_KIND_ROOM_TRANSCRIPT_READY,
  ENTRY_KIND_RULE_SET_VERSION_CREATED, ENTRY_KIND_SANCTION_CREATED,
  ENTRY_KIND_SEVERITY_TIER_FROZEN, ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED,
  ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED, ENTRY_KIND_SPONSOR_LIABILITY_APPLIED,
  ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED, ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED,
  ENTRY_KIND_SPONSOR_LIABILITY_FIRED, ENTRY_KIND_SPONSOR_LIABILITY_PENDING,
  ENTRY_KIND_THRESHOLD_MET, ENTRY_KIND_VOTE_OUTCOME_RECORDED, GovernanceLog,
  GovernanceLogInsertForm, append,
};

#[cfg(feature = "full")]
use {
  lemmy_diesel_utils::connection::DbPool,
  lemmy_utils::error::{LemmyErrorType, LemmyResult},
};

/// Payload for a Room lifecycle event written by the bridge daemon via
/// [`append_room_event`]. Integer and opaque-token fields only — no raw
/// usernames, emails, or display names (`scrub_json` inside `append` strips
/// those; bridge daemon owns pseudonym resolution).
///
/// NOTE: `matrix_room_id` uses the Matrix `!room:server` sigil form which
/// may match the `scrub_json` URL/mention regex. Pass an opaque non-sigil
/// token (e.g. the bare room localpart) when round-trip fidelity is
/// required (ADR-008).
#[derive(Debug, serde::Serialize)]
pub struct RoomEventPayload {
  pub case_id: i32,
  pub matrix_room_id: Option<String>,
  pub lifecycle_stage: String,
  pub member_count: Option<i32>,
}

/// The 10 allowed `entry_kind` values for [`append_room_event`].
/// Any kind not in this set is rejected (ADR-008 integrity gate).
#[cfg(feature = "full")]
const ROOM_KINDS: &[&str] = &[
  ENTRY_KIND_ROOM_ARCHIVED,
  ENTRY_KIND_ROOM_BRIDGE_ERROR,
  ENTRY_KIND_ROOM_CREATED,
  ENTRY_KIND_ROOM_DECISION_RELAYED,
  ENTRY_KIND_ROOM_IDENTITY_REVEALED,
  ENTRY_KIND_ROOM_LIFECYCLE_EVENT,
  ENTRY_KIND_ROOM_MEMBER_ADDED,
  ENTRY_KIND_ROOM_MEMBER_REMOVED,
  ENTRY_KIND_ROOM_RECORDING_UPLOADED,
  ENTRY_KIND_ROOM_TRANSCRIPT_READY,
];

/// Typed, kind-validated path to the governance log for Room lifecycle events.
///
/// Accepts only the 10 `ENTRY_KIND_ROOM_*` kinds (ADR-008 integrity gate —
/// the bridge can only write Room::* kinds through this wrapper, never
/// arbitrary entry kinds). Serialises `payload` to JSON and delegates to
/// [`append`], which runs `scrub_json` and the hash-chain + signing steps.
#[cfg(feature = "full")]
pub async fn append_room_event(
  pool: &mut DbPool<'_>,
  kind: &str,
  payload: RoomEventPayload,
  actor_pseudonym: Option<String>,
) -> LemmyResult<GovernanceLog> {
  if !ROOM_KINDS.contains(&kind) {
    return Err(LemmyErrorType::Unknown(format!("not a room entry kind: {kind}")).into());
  }
  let value = serde_json::to_value(&payload)
    .map_err(|e| LemmyErrorType::Unknown(format!("RoomEventPayload serialisation failed: {e}")))?;
  append(pool, kind, value, actor_pseudonym).await
}
