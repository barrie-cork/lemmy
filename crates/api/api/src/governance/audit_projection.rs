//! Shared projection from a `governance_log` row into the
//! `AdminConfigAuditEntry` wire shape used by the admin-config audit list,
//! the v1-AD-d dashboard `recent_config_changes` widget, and the v1-AD-d
//! SSE audit stream.

use crate::governance::governance_log::ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED;
use lemmy_api_common::governance::AdminConfigAuditEntry;
use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
use serde_json::Value;

/// Project a `governance_log` row whose `entry_kind` is
/// `admin_config_changed` or `admin_config_change_denied` into the typed
/// audit response entry. Unknown / missing payload fields degrade to
/// empty strings / `Value::Null` — we never fail a whole audit page on a
/// single malformed legacy row (shell-wrapper rows predating this handler
/// always produce well-formed payloads, but future migrations may add
/// fields and older rows should still list).
///
/// `previous_value` + `previous_from` are hydrated from the payload's
/// v1-AD-c tail fields via `.get(...).cloned()` / `.and_then(as_str)`.
/// Shell-written rows and pre-v1-AD-c HTTP rows omit those keys; this
/// projection returns `None` for them. See Issue #77.
pub(crate) fn project_to_audit_entry(row: GovernanceLog) -> AdminConfigAuditEntry {
  let payload = &row.payload;
  let scope = payload
    .get("scope")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
  let key = payload
    .get("key")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
  let value_type = payload
    .get("value_type")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
  let new_value = payload.get("value").cloned().unwrap_or(Value::Null);
  let previous_value = payload.get("previous_value").cloned();
  let previous_from = payload
    .get("previous_from")
    .and_then(|v| v.as_str())
    .map(str::to_owned);
  let reason = payload
    .get("reason")
    .and_then(|v| v.as_str())
    .unwrap_or("")
    .to_string();
  let denial_reason = if row.entry_kind == ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED {
    payload
      .get("denial_reason")
      .and_then(|v| v.as_str())
      .map(str::to_owned)
  } else {
    None
  };

  AdminConfigAuditEntry {
    id: row.id.0,
    entry_kind: row.entry_kind,
    scope,
    key,
    value_type,
    previous_value,
    previous_from,
    new_value,
    reason,
    actor_pseudonym: row.actor_pseudonym,
    created_at: row.created_at,
    signature: row.signature,
    denial_reason,
  }
}
