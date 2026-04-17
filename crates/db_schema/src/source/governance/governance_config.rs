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

/// Insert form for `governance_config`. The table is append-only by design —
/// admin edits INSERT a new `(scope, key, valid_from)` row and the
/// `governance_config_current` DISTINCT-ON view surfaces the latest — so this
/// form deliberately does NOT derive `AsChangeset`. Mutating historical rows
/// in place would break the audit trail and the parity between `valid_from`
/// and action time. If a narrow update path is ever needed (e.g. fixing
/// `updated_by` on the most-recent row), define a dedicated changeset struct.
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
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
