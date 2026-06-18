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
  ENTRY_KIND_ACTOR_APP_LINK_CREATED, ENTRY_KIND_ACTOR_APP_LINK_REVOKED,
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
  ENTRY_KIND_ROOM_CHAIR_OVERRIDE, ENTRY_KIND_ROOM_CHAIR_TRANSFERRED,
  ENTRY_KIND_ROOM_CREATED, ENTRY_KIND_ROOM_DECISION_RELAYED, ENTRY_KIND_ROOM_IDENTITY_REVEALED,
  ENTRY_KIND_ROOM_LIFECYCLE_EVENT, ENTRY_KIND_ROOM_MEMBER_ADDED, ENTRY_KIND_ROOM_MEMBER_REMOVED,
  ENTRY_KIND_ROOM_MUTE_ALL,
  ENTRY_KIND_ROOM_RECORDING_UPLOADED, ENTRY_KIND_ROOM_TRANSCRIPT_READY,
  ENTRY_KIND_RULE_SET_VERSION_CREATED, ENTRY_KIND_SANCTION_CREATED,
  ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED, ENTRY_KIND_SANCTION_PUBLISHED,
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
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomEventPayload {
  pub case_id: i32,
  pub matrix_room_id: Option<String>,
  pub lifecycle_stage: String,
  pub member_count: Option<i32>,
  // M3 stage-mode chair-action metadata (opt; only chair entries set them).
  // ADR-015: pseudonyms only — never person_id/username/MXID.
  // ADR-016: metadata only — never room content.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub action: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub target_pseudonym: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub from_pseudonym: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub to_pseudonym: Option<String>,
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub at: Option<String>,
}

#[cfg(test)]
mod tests {
  use super::RoomEventPayload;

  #[test]
  fn room_event_payload_chair_fields_skip_serializing_if_none() -> Result<(), serde_json::Error> {
    // room_chair_override: action + target_pseudonym present; transfer fields omitted
    let override_payload = RoomEventPayload {
      case_id: 1,
      matrix_room_id: None,
      lifecycle_stage: "room_chair_override".to_string(),
      member_count: None,
      action: Some("force_demote".to_string()),
      target_pseudonym: Some("pseu_x".to_string()),
      from_pseudonym: None,
      to_pseudonym: None,
      at: None,
    };
    let v = serde_json::to_value(&override_payload)?;
    assert!(v.get("action").is_some());
    assert!(v.get("target_pseudonym").is_some());
    assert!(v.get("from_pseudonym").is_none());
    assert!(v.get("to_pseudonym").is_none());
    assert!(v.get("at").is_none());

    // room_chair_transferred: transfer fields present; action + target_pseudonym omitted
    let transfer_payload = RoomEventPayload {
      case_id: 2,
      matrix_room_id: None,
      lifecycle_stage: "room_chair_transferred".to_string(),
      member_count: None,
      action: None,
      target_pseudonym: None,
      from_pseudonym: Some("pseu_a".to_string()),
      to_pseudonym: Some("pseu_b".to_string()),
      at: Some("2026-06-18T00:00:00Z".to_string()),
    };
    let v = serde_json::to_value(&transfer_payload)?;
    assert!(v.get("from_pseudonym").is_some());
    assert!(v.get("to_pseudonym").is_some());
    assert!(v.get("at").is_some());
    assert!(v.get("action").is_none());
    assert!(v.get("target_pseudonym").is_none());

    Ok(())
  }
}

/// The 13 allowed `entry_kind` values for [`append_room_event`].
/// Any kind not in this set is rejected (ADR-008 integrity gate).
#[cfg(feature = "full")]
const ROOM_KINDS: &[&str] = &[
  ENTRY_KIND_ROOM_ARCHIVED,
  ENTRY_KIND_ROOM_BRIDGE_ERROR,
  ENTRY_KIND_ROOM_CHAIR_OVERRIDE,
  ENTRY_KIND_ROOM_CHAIR_TRANSFERRED,
  ENTRY_KIND_ROOM_CREATED,
  ENTRY_KIND_ROOM_DECISION_RELAYED,
  ENTRY_KIND_ROOM_IDENTITY_REVEALED,
  ENTRY_KIND_ROOM_LIFECYCLE_EVENT,
  ENTRY_KIND_ROOM_MEMBER_ADDED,
  ENTRY_KIND_ROOM_MEMBER_REMOVED,
  ENTRY_KIND_ROOM_MUTE_ALL,
  ENTRY_KIND_ROOM_RECORDING_UPLOADED,
  ENTRY_KIND_ROOM_TRANSCRIPT_READY,
];

/// Typed, kind-validated path to the governance log for Room lifecycle events.
///
/// Accepts only the 13 `ENTRY_KIND_ROOM_*` kinds (ADR-008 integrity gate —
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
