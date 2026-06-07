use crate::newtypes::{SanctionEventId, SanctionId};
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::sanction_event;
use lemmy_db_schema_file::enums::SanctionKind;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = sanction_event))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A published sanction event delivered to registered subscribers (B-publish, ADR-016).
pub struct SanctionEvent {
  pub id: SanctionEventId,
  pub sanction_id: SanctionId,
  pub sanction_kind: SanctionKind,
  /// ADR-015: actor_pseudonym.pseudonym only — never person.name or local_user.email.
  pub subject_actor_pseudonym: String,
  pub effective_from: DateTime<Utc>,
  pub effective_until: Option<DateTime<Utc>>,
  /// Hex-encoded governance_log.entry_hash for the sanction_created entry.
  pub governance_log_entry_hash: String,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = sanction_event))]
pub struct SanctionEventInsertForm {
  pub sanction_id: SanctionId,
  pub sanction_kind: SanctionKind,
  pub subject_actor_pseudonym: String,
  pub effective_from: DateTime<Utc>,
  pub effective_until: Option<DateTime<Utc>>,
  pub governance_log_entry_hash: String,
}
