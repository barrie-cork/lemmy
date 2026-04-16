//! Hash-chained governance log writer.
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
//! ## v0 signing deferred
//!
//! Phase 4a does NOT sign rows. The `signature` column is left NULL.
//! ed25519 signing + key management is deferred to Phase 4b per the
//! plan's §Design Decision: Signing Deferred to Phase 4b, to avoid
//! introducing `ed25519-dalek` before the full golden-path test exists
//! to exercise the write + sign + verify loop end-to-end. The hash
//! chain (trigger-side) is the critical integrity property and ships now.

use crate::governance::redaction::scrub_json;
use diesel::SelectableHelper;
use diesel_async::RunQueryDsl;
use lemmy_db_schema::source::governance::governance_log::{
  GovernanceLog,
  GovernanceLogInsertForm,
};
use lemmy_db_schema_file::schema::governance_log;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;
use serde_json::Value;

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
pub async fn append(
  pool: &mut DbPool<'_>,
  entry_kind: &str,
  payload: Value,
  actor_pseudonym: Option<String>,
) -> LemmyResult<GovernanceLog> {
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

  // TODO(brehon-fork): Phase 4b — sign `row.entry_hash` with ed25519
  // and UPDATE the `signature` column. See plan §Design Decision:
  // Signing Deferred to Phase 4b.

  Ok(row)
}
