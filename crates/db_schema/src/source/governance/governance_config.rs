use crate::newtypes::GovernanceConfigId;
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::governance_config;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_config))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A typed, versioned config row. `valid_from` lets admin edits insert a new
/// row rather than mutate; the `governance_config_current` view reads the
/// latest per `(scope, key)`. The CHECK constraint `governance_config_typed`
/// ensures exactly one of `value_int`/`value_float`/`value_bool`/`value_text`
/// is non-NULL, matching `value_type`.
pub struct GovernanceConfig {
  pub id: GovernanceConfigId,
  pub scope: String,
  pub key: String,
  pub value_type: String,
  pub value_int: Option<i64>,
  pub value_float: Option<f64>,
  pub value_bool: Option<bool>,
  pub value_text: Option<String>,
  pub valid_from: DateTime<Utc>,
  pub updated_by: Option<PersonId>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = governance_config))]
pub struct GovernanceConfigInsertForm {
  pub scope: String,
  pub key: String,
  pub value_type: String,
  pub value_int: Option<i64>,
  pub value_float: Option<f64>,
  pub value_bool: Option<bool>,
  pub value_text: Option<String>,
  pub updated_by: Option<PersonId>,
}
