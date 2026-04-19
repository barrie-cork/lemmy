//! Hash-chained, ed25519-signed governance log writer.
//!
//! The only sanctioned way to append a row to `governance_log`. Every
//! payload is passed through [`crate::governance::redaction::scrub_json`]
//! before insert so no unredacted identifiers ever reach the table —
//! compliance with [ADR-015] and [IMPLEMENTATION-PLAN-v0.md §4.2].
//!
//! `prev_hash` and `entry_hash` are populated by the Postgres trigger
//! `governance_log_hash_chain_before_insert` at INSERT time (see Phase 1
//! migrations). Callers must NEVER populate those columns — the
//! [`GovernanceLogInsertForm`] shape structurally prevents it.
//!
//! ## Signing (Phase 4b)
//!
//! After the INSERT produces an `entry_hash`, this writer signs that
//! hash with the ed25519 key loaded from `GOVERNANCE_LOG_SIGNING_KEY`
//! (32-byte hex-encoded seed) and UPDATEs the row's `signature` column.
//! The UPDATE transitions `signature` from NULL to a 64-byte signature
//! exactly once per row — the `governance_log_signature_gate` trigger
//! (triggers.sql:800-827) forbids any other `signature` transition and
//! forbids any other column from changing on the UPDATE, so we set ONLY
//! `signature`. Per [99 ADR-008] the env-var key is the v0 shape; an
//! external signer + key rotation lands in v2.
//!
//! If `GOVERNANCE_LOG_SIGNING_KEY` is absent or malformed, `append`
//! returns a hard error — silent fallback to unsigned rows defeats the
//! point of the signature.

use crate::governance::redaction::scrub_json;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use ed25519_dalek::{Signer, SigningKey};
use lemmy_db_schema::source::governance::governance_log::{
  GovernanceLog,
  GovernanceLogInsertForm,
};
use lemmy_db_schema_file::schema::governance_log;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::Value;
use std::env;

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
// publisher (DQ-6.6 still pending at the time of writing — see entry id
// 36 in `.claude/decision-queue.json`); inbound emitted by task 75's
// receiver. The four constants are added here regardless of how the
// publisher's home crate resolves so callers in any of `lemmy_api`,
// `lemmy_apub_activities`, or a future `lemmy_api_utils::governance`
// shim can reference the same canonical strings.
pub const ENTRY_KIND_FEDERATION_SANCTION_SENT: &str = "federation_sanction_sent";
pub const ENTRY_KIND_FEDERATION_SANCTION_RECEIVED: &str = "federation_sanction_received";
pub const ENTRY_KIND_FEDERATION_ATTESTATION_SENT: &str = "federation_attestation_sent";
pub const ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED: &str = "federation_attestation_received";

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
///   [`crate::governance::actor_pseudonym_helper::get_or_create`], or
///   `None` for system-generated events (e.g. a scheduled job).
///
/// Returns the inserted row with `entry_hash` populated by the
/// hash-chain trigger and `signature` populated by the signing step.
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

  let row = diesel::insert_into(governance_log::table)
    .values(&form)
    .returning(GovernanceLog::as_returning())
    .get_result::<GovernanceLog>(conn)
    .await?;

  // Sign the trigger-computed entry_hash. The 32-byte SHA-256 digest is
  // what goes on the wire; the resulting 64-byte signature rides in the
  // `signature` column.
  let signature = signing_key.sign(&row.entry_hash).to_bytes().to_vec();

  // The signature-gate trigger allows exactly one NULL→non-NULL
  // transition on `signature` and rejects any other column change, so
  // the .set(...) clause must contain only `signature`.
  diesel::update(governance_log::table.filter(governance_log::id.eq(row.id)))
    .set(governance_log::signature.eq(&signature))
    .execute(conn)
    .await?;

  Ok(GovernanceLog {
    signature: Some(signature),
    ..row
  })
}

/// Load the ed25519 signing key from `GOVERNANCE_LOG_SIGNING_KEY`.
/// Missing / malformed key is a hard error — no silent fallback to
/// unsigned rows per [99 ADR-008].
fn load_signing_key() -> LemmyResult<SigningKey> {
  let hex_str = env::var(SIGNING_KEY_ENV).map_err(|_e| {
    LemmyErrorType::Unknown(format!("{SIGNING_KEY_ENV} not set (required for governance log)"))
  })?;
  let bytes = hex::decode(hex_str.trim()).map_err(|_e| {
    LemmyErrorType::Unknown(format!("{SIGNING_KEY_ENV} is not valid hex"))
  })?;
  let seed: [u8; 32] = bytes.as_slice().try_into().map_err(|_e| {
    LemmyErrorType::Unknown(format!(
      "{SIGNING_KEY_ENV} must decode to exactly 32 bytes (got {})",
      bytes.len()
    ))
  })?;
  Ok(SigningKey::from_bytes(&seed))
}
