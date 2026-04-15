use crate::newtypes::ActorPseudonymId;
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::actor_pseudonym;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = actor_pseudonym))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A pseudonymous handle for a Lemmy person, used in governance-log entries to
/// satisfy GDPR pseudonymisation requirements (ADR-015).
pub struct ActorPseudonym {
  pub id: ActorPseudonymId,
  pub person_id: PersonId,
  pub pseudonym: String,
  pub created_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = actor_pseudonym))]
pub struct ActorPseudonymInsertForm {
  pub person_id: PersonId,
  pub pseudonym: String,
}
