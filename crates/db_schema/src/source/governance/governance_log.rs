//! Governance log model + hash-chained, ed25519-signed writer.
//!
//! Originally split: the model lived here (`lemmy_db_schema::source::...`)
//! and the writer ([`append`]) lived in
//! `crates/api/api/src/governance/governance_log.rs`. The split was an
//! accident of layering, not a design decision — and it broke down in
//! Phase 6 when the inbound federation receiver
//! (`crates/apub/activities/src/governance/inbox.rs`) needed to call
//! [`append`] from a crate (`lemmy_apub_activities`) that cannot depend on
//! `lemmy_api` (lower in the dep graph; cycle would form with the planned
//! `lemmy_api → lemmy_apub_activities` edge for Agent F's task 76 wrapper).
//!
//! Per advisor decision DQ-6.6-inbound (resolved id 37 in
//! `.claude/decision-queue.json`) the writer + entry-kind constants +
//! signing helper move DOWN to `lemmy_db_schema`. The api crate
//! re-exports the same public names from
//! `lemmy_api::governance::governance_log` so existing call sites
//! (submit_jury_vote.rs, ban.rs, accept_jury_assignment.rs, …) continue
//! to compile unchanged.
//!
//! ## Append invariants
//!
//! [`append`] is the only sanctioned way to add a row to `governance_log`.
//! Every payload is passed through [`super::redaction::scrub_json`] before
//! insert so no unredacted identifiers ever reach the table — compliance
//! with [ADR-015] and [IMPLEMENTATION-PLAN-v0.md §4.2].
//!
//! `prev_hash` and `entry_hash` are populated by the Postgres trigger
//! `governance_log_hash_chain_before_insert` at INSERT time (see Phase 1
//! migrations). Callers must NEVER populate those columns — the
//! [`GovernanceLogInsertForm`] shape structurally prevents it.
//!
//! ## Signing (Phase 4b)
//!
//! After the INSERT produces an `entry_hash`, [`append`] signs that hash
//! with the ed25519 key loaded from `GOVERNANCE_LOG_SIGNING_KEY`
//! (32-byte hex-encoded seed) and UPDATEs the row's `signature` column.
//! The UPDATE transitions `signature` from NULL to a 64-byte signature
//! exactly once per row — the `governance_log_signature_gate` trigger
//! (triggers.sql:800-827) forbids any other `signature` transition and
//! forbids any other column from changing on the UPDATE, so we set ONLY
//! `signature`. Per [99 ADR-008] the env-var key is the v0 shape; an
//! external signer + key rotation lands in v2.
//!
//! If `GOVERNANCE_LOG_SIGNING_KEY` is absent or malformed, [`append`]
//! returns a hard error — silent fallback to unsigned rows defeats the
//! point of the signature.

use crate::newtypes::GovernanceLogId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::governance_log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use {
  crate::source::governance::redaction::scrub_json,
  diesel::{ExpressionMethods, QueryDsl, SelectableHelper},
  diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt},
  ed25519_dalek::{Signer, SigningKey},
  lemmy_diesel_utils::connection::{DbPool, get_conn},
  lemmy_utils::error::{LemmyErrorType, LemmyResult},
  std::env,
};

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_log))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// Append-only, hash-chained governance log entry.
///
/// `prev_hash`, `entry_hash`, and `signature` are **trigger-populated**:
/// `prev_hash` and `entry_hash` are filled by
/// `r.governance_log_hash_chain_before_insert()` at INSERT time; `signature`
/// is filled by a single-shot UPDATE in Phase 4, gated by
/// `r.governance_log_signature_gate_before_update()`. Callers must never set
/// these three fields directly — construct rows via
/// [`GovernanceLogInsertForm`] and let the trigger layer do its job.
pub struct GovernanceLog {
  pub id: GovernanceLogId,
  pub prev_hash: Vec<u8>,
  pub entry_hash: Vec<u8>,
  pub entry_kind: String,
  pub payload: Value,
  pub actor_pseudonym: Option<String>,
  pub created_at: DateTime<Utc>,
  pub signature: Option<Vec<u8>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_log))]
/// **Do not populate `prev_hash`, `entry_hash`, or `signature` — they are
/// trigger-managed.** Phase 4's helper wraps this form and is the only
/// sanctioned insert path.
pub struct GovernanceLogInsertForm {
  pub entry_kind: String,
  pub payload: Value,
  pub actor_pseudonym: Option<String>,
}

// -- Canonical v0 entry_kind strings ---------------------------------------
//
// Using a const at every call site turns typos into compile-time errors.
// Existing Phase 4 call sites still use string literals; new Phase 5a/5b/5c
// call sites MUST use the consts. Migrating the older literals is a
// Phase 5c cosmetic task.

pub const ENTRY_KIND_REPORT_CREATED: &str = "report_created";
pub const ENTRY_KIND_THRESHOLD_MET: &str = "threshold_met";
pub const ENTRY_KIND_JURY_ASSIGNED: &str = "jury_assigned";
pub const ENTRY_KIND_PANEL_ASSEMBLED: &str = "panel_assembled";
pub const ENTRY_KIND_JURY_VOTED: &str = "jury_voted";
pub const ENTRY_KIND_SANCTION_CREATED: &str = "sanction_created";
pub const ENTRY_KIND_PUBLIC_LOG_PUBLISHED: &str = "public_log_published";
pub const ENTRY_KIND_REPUTATION_DELTA: &str = "reputation_delta";
pub const ENTRY_KIND_CASE_DECIDED: &str = "case_decided";
pub const ENTRY_KIND_CAPABILITY_CHANGED: &str = "capability_changed";
pub const ENTRY_KIND_SPONSOR_LIABILITY_APPLIED: &str = "sponsor_liability_applied";
pub const ENTRY_KIND_SPONSOR_LIABILITY_CLAMPED: &str = "sponsor_liability_clamped";
pub const ENTRY_KIND_FOUNDER_SEEDED: &str = "founder_seeded";
pub const ENTRY_KIND_ENDORSEMENT_CREATED: &str = "endorsement_created";
pub const ENTRY_KIND_EMERGENCY_REMOVED: &str = "emergency_removed";
pub const ENTRY_KIND_JURY_ACCEPTED: &str = "jury_accepted";
pub const ENTRY_KIND_JURY_DECLINED: &str = "jury_declined";
pub const ENTRY_KIND_JURY_REPLACEMENT_SELECTED: &str = "jury_replacement_selected";
pub const ENTRY_KIND_APPEAL_REQUESTED: &str = "appeal_requested";

// Phase 6 — federation entry kinds. Outbound emitted by task 74's
// publisher (Agent D); inbound emitted by task 75's receiver in
// `lemmy_apub_activities::governance::inbox` (Agent E). The four
// constants live alongside the existing kinds so callers in any of
// `lemmy_api`, `lemmy_apub_activities`, `lemmy_apub`, or any future
// crate can import the same canonical strings via the
// `lemmy_db_schema::source::governance::governance_log` path (or its
// `lemmy_api::governance::governance_log` re-export).
pub const ENTRY_KIND_FEDERATION_SANCTION_SENT: &str = "federation_sanction_sent";
pub const ENTRY_KIND_FEDERATION_SANCTION_RECEIVED: &str = "federation_sanction_received";
pub const ENTRY_KIND_FEDERATION_ATTESTATION_SENT: &str = "federation_attestation_sent";
pub const ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED: &str = "federation_attestation_received";

// v1-AD-a additions (task 7): admin config-edit audit. The
// `admin_config_changed` literal MUST equal the string written by the
// v0 shell wrapper `scripts/brehon/admin-config-write.sh:148` so v1-AD-b's
// NOT5 deprecation gate 3 ("byte-identical governance_log rows from both
// paths") holds when the Rust handler lands alongside the CLI path. The
// `_denied` variant has no v0 caller — it's reserved for v1-AD-b's
// capability-check reject path.
pub const ENTRY_KIND_ADMIN_CONFIG_CHANGED: &str = "admin_config_changed";
pub const ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED: &str = "admin_config_change_denied";

// v1-AD-c additions (task 3): rule-set version creation. Emitted from
// `crates/api/api/src/governance/admin_rule_sets.rs::admin_create_rule_set`
// inside the three-row atomic write (rule_set_version INSERT +
// governance_config rule_set.active_version_id INSERT + governance_log
// append). Per the registry rule's invariant, the literal must be unique
// across all ENTRY_KIND_* string values in this file.
pub const ENTRY_KIND_RULE_SET_VERSION_CREATED: &str = "rule_set_version_created";

// v1-JM-a additions (v1 jury-mechanics sub-phase A). All six emitting
// call sites land in v1-JM-b/c/d per PRD §9.2/§9.3/§9.4/§9.5. v1-JM-a
// ships the const declarations and the registry entry only; actual
// governance_log::append calls land with the handler edits in later
// sub-phases (b: constraint_relaxed + severity_tier_frozen;
// d: appeal_* kinds incl. window_expired scheduler tick).
pub const ENTRY_KIND_JURY_CONSTRAINT_RELAXED: &str = "jury_constraint_relaxed";
/// Jury panel reached `panel_size_snapshot` votes but no `JuryDecision`
/// variant met `threshold_count_snapshot`. Case is flipped to
/// `CaseStatus::AdminReview` for human resolution. Payload:
/// `{ case_id, panel_size_snapshot, threshold_count_snapshot,
///   tally: {<JuryDecision>: count, ...} }`. Emitted exactly once per
/// case at the deadlock-detection moment (PRD §9.1 step 5). Per ADR-010,
/// AdminReview is a procedurally-mandatory terminal state; admin
/// intervention is required to resume. No subsequent `case_decided`,
/// `sanction_created`, or `public_log_published` fires for a deadlocked
/// case.
pub const ENTRY_KIND_JURY_DEADLOCK: &str = "jury_deadlock";
pub const ENTRY_KIND_APPEAL_PANEL_ASSEMBLED: &str = "appeal_panel_assembled";
pub const ENTRY_KIND_APPEAL_DECIDED: &str = "appeal_decided";
pub const ENTRY_KIND_APPEAL_REJECTED: &str = "appeal_rejected";
pub const ENTRY_KIND_APPEAL_WINDOW_EXPIRED: &str = "appeal_window_expired";
pub const ENTRY_KIND_SEVERITY_TIER_FROZEN: &str = "severity_tier_frozen";

// v1-SL-a additions (v1 sponsor-liability sub-phase A). All five
// emitting call sites land in v1-SL-b/c/d + restorative-mechanics-v1
// per the registry rule's pre-landed-const exemption. v1-SL-a writes
// the const declarations only; actual governance_log::append calls
// land with the handler edits in later sub-phases (b: endorsement_revoked
// + sponsor_liability_escaped (revocation branch); c: sponsor_liability_fired
// + sponsor_liability_escaped (scheduler escape branch); d:
// sponsor_liability_pending; restorative-mechanics-v1: restoration_completed).
pub const ENTRY_KIND_ENDORSEMENT_REVOKED: &str = "endorsement_revoked";
pub const ENTRY_KIND_RESTORATION_COMPLETED: &str = "restoration_completed";
pub const ENTRY_KIND_SPONSOR_LIABILITY_ESCAPED: &str = "sponsor_liability_escaped";
pub const ENTRY_KIND_SPONSOR_LIABILITY_FIRED: &str = "sponsor_liability_fired";
pub const ENTRY_KIND_SPONSOR_LIABILITY_PENDING: &str = "sponsor_liability_pending";

// v1-RT-r1 additions (v1 reputation-tuning sub-phase r1). All seven
// emitting call sites land in r2/r3/r4/r5 per PRD §7 + pre-landed-const
// exemption.
//   - PARTICIPATION_CRON_TICK -> r3 scheduled_tasks.rs (pending).
//   - VOTE_OUTCOME_RECORDED -> r3 submit_jury_vote.rs (pending).
//   - EVIDENCE_QUALITY_RECORDED -> r3 submit_jury_vote.rs + r3 admin
//     flag-bad-faith endpoint (pending).
//   - ROLLUP_RECOMPUTED -> r5 reputation_rollup_cron (pending).
//   - DECAY_KNOB_CHANGED -> r2 admin_config.rs (pending).
//   - SPONSOR_ALLOWLIST_ADDED -> r4 add endpoint (pending).
//   - SPONSOR_ALLOWLIST_REMOVED -> r4 remove endpoint (pending).
pub const ENTRY_KIND_PARTICIPATION_CRON_TICK: &str = "participation_cron_tick";
pub const ENTRY_KIND_VOTE_OUTCOME_RECORDED: &str = "vote_outcome_recorded";
pub const ENTRY_KIND_EVIDENCE_QUALITY_RECORDED: &str = "evidence_quality_recorded";
pub const ENTRY_KIND_ROLLUP_RECOMPUTED: &str = "rollup_recomputed";
pub const ENTRY_KIND_DECAY_KNOB_CHANGED: &str = "decay_knob_changed";
pub const ENTRY_KIND_SPONSOR_ALLOWLIST_ADDED: &str = "sponsor_allowlist_added";
pub const ENTRY_KIND_SPONSOR_ALLOWLIST_REMOVED: &str = "sponsor_allowlist_removed";

// v1-federation-inbound-a consts (9). Call sites land in -b/-c per registry.
pub const ENTRY_KIND_FEDERATION_INBOUND_BLOCKED: &str = "federation_inbound_blocked";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_OVERSIZE: &str =
  "federation_inbound_dropped_oversize";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_ACTOR: &str =
  "federation_inbound_dropped_rate_limit_actor";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_RATE_LIMIT_PEER: &str =
  "federation_inbound_dropped_rate_limit_peer";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_REPLAY: &str = "federation_inbound_dropped_replay";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_SCHEMA: &str = "federation_inbound_dropped_schema";
pub const ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED: &str =
  "federation_inbound_dropped_storage_cap_evicted";
pub const ENTRY_KIND_FEDERATION_INBOUND_PERSIST_FAILED: &str = "federation_inbound_persist_failed";
pub const ENTRY_KIND_FEDERATION_LABEL_RECEIVED: &str = "federation_label_received";
pub const ENTRY_KIND_FEDERATION_PEER_TRUST_CHANGED: &str = "federation_peer_trust_changed";

#[cfg(feature = "full")]
const SIGNING_KEY_ENV: &str = "GOVERNANCE_LOG_SIGNING_KEY";

/// Append a single row to `governance_log`.
///
/// - `entry_kind` — a short enum-like tag such as `"report_created"`,
///   `"jury_vote_submitted"`, `"case_decided"`. Not enforced at the type
///   level in v0 (the column is plain TEXT); Phase 5 may promote it to
///   a Diesel enum.
/// - `payload` — an arbitrary JSON value describing the event. Passed
///   through [`scrub_json`] before insert; callers do NOT need to scrub
///   their payloads manually.
/// - `actor_pseudonym` — the stable pseudonym from
///   `lemmy_api::governance::actor_pseudonym_helper::get_or_create`, or
///   `None` for system-generated events (e.g. a scheduled job, or a
///   remote actor whose pseudonym does not exist locally).
///
/// Returns the inserted row with `entry_hash` populated by the
/// hash-chain trigger and `signature` populated by the signing step.
#[cfg(feature = "full")]
pub async fn append(
  pool: &mut DbPool<'_>,
  entry_kind: &str,
  payload: Value,
  actor_pseudonym: Option<String>,
) -> LemmyResult<GovernanceLog> {
  let signing_key = load_signing_key()?;

  let conn = &mut get_conn(pool).await?;

  let form = GovernanceLogInsertForm {
    entry_kind: entry_kind.to_string(),
    payload: scrub_json(&payload),
    actor_pseudonym,
  };

  // ADR-008 atomicity: INSERT + signature UPDATE must commit together
  // so subscribers never observe a row with `signature = NULL`, and a
  // crash between the two writes can't leave a permanent unsigned row.
  // When `append` is called from inside a caller's outer tx (via
  // `&mut (&mut *conn).into()` reborrow — see federation_outbox.rs:187
  // or admin_assign_jury.rs:165), diesel-async promotes this inner
  // `run_transaction` to a SAVEPOINT, so a signing failure here rolls
  // back the two log writes while preserving the caller's other work
  // up to their own retry/rollback decision.
  //
  // The NOTIFY trigger (`governance_log_notify_trigger`) already fires
  // AFTER UPDATE OF signature per migration `2026-04-20-000100`, so
  // atomic INSERT+UPDATE also resolves GH #35: subscribers observe the
  // row only once the transaction commits with signature populated.
  conn
    .run_transaction(|conn| {
      async move {
        let row = diesel::insert_into(governance_log::table)
          .values(&form)
          .returning(GovernanceLog::as_returning())
          .get_result::<GovernanceLog>(conn)
          .await?;

        // Sign the trigger-computed entry_hash. The 32-byte SHA-256
        // digest is what goes on the wire; the resulting 64-byte
        // signature rides in the `signature` column.
        let signature = signing_key.sign(&row.entry_hash).to_bytes().to_vec();

        // The signature-gate trigger allows exactly one NULL→non-NULL
        // transition on `signature` and rejects any other column
        // change, so the .set(...) clause must contain only
        // `signature`.
        diesel::update(governance_log::table.filter(governance_log::id.eq(row.id)))
          .set(governance_log::signature.eq(&signature))
          .execute(conn)
          .await?;

        Ok(GovernanceLog {
          signature: Some(signature),
          ..row
        })
      }
      .scope_boxed()
    })
    .await
}

/// Load the ed25519 signing key from `GOVERNANCE_LOG_SIGNING_KEY`.
/// Missing / malformed key is a hard error — no silent fallback to
/// unsigned rows per [99 ADR-008].
#[cfg(feature = "full")]
fn load_signing_key() -> LemmyResult<SigningKey> {
  let hex_str = env::var(SIGNING_KEY_ENV).map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "{SIGNING_KEY_ENV} not set (required for governance log)"
    ))
  })?;
  let bytes = hex::decode(hex_str.trim())
    .map_err(|_e| LemmyErrorType::Unknown(format!("{SIGNING_KEY_ENV} is not valid hex")))?;
  let seed: [u8; 32] = bytes.as_slice().try_into().map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "{SIGNING_KEY_ENV} must decode to exactly 32 bytes (got {})",
      bytes.len()
    ))
  })?;
  Ok(SigningKey::from_bytes(&seed))
}
