use crate::newtypes::GovernanceLogId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::governance_log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

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
