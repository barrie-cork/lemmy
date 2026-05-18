use crate::newtypes::FederationInboxDroppedLogId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::federation_inbox_dropped_log;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_dropped_log))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct FederationInboxDroppedLog {
  pub id: FederationInboxDroppedLogId,
  pub source_instance: String,
  pub activity_id: Option<String>,
  pub drop_reason: String,
  pub payload_excerpt: Option<String>,
  pub dropped_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = federation_inbox_dropped_log))]
pub struct FederationInboxDroppedLogInsertForm {
  pub source_instance: String,
  pub activity_id: Option<String>,
  pub drop_reason: String,
  pub payload_excerpt: Option<String>,
}
