use crate::newtypes::MessagingConfigId;
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::PersonId;
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::governance_messaging_config;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[cfg(feature = "full")]
use {
  diesel::{ExpressionMethods, OptionalExtension, QueryDsl, dsl::insert_into},
  diesel_async::RunQueryDsl,
  lemmy_diesel_utils::connection::{DbPool, get_conn},
  lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult},
};

#[skip_serializing_none]
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_messaging_config))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A typed, versioned messaging config row. `valid_from` lets admin edits insert a new
/// row rather than mutate; the `governance_messaging_config_current` view reads the
/// latest per `(scope, key)`. The CHECK constraint `governance_messaging_config_typed`
/// ensures exactly one of `value_int`/`value_bool`/`value_text` is non-NULL, matching
/// `value_type`. No `value_float` column (M1 config keys are int/bool/text only).
pub struct GovernanceMessagingConfig {
  pub id: MessagingConfigId,
  pub scope: String,
  pub key: String,
  pub value_type: String,
  pub value_int: Option<i64>,
  pub value_bool: Option<bool>,
  pub value_text: Option<String>,
  pub valid_from: DateTime<Utc>,
  pub updated_by: Option<PersonId>,
}

/// Insert form for `governance_messaging_config`. The table is append-only by design —
/// admin edits INSERT a new `(scope, key, valid_from)` row and the
/// `governance_messaging_config_current` DISTINCT-ON view surfaces the latest — so this
/// form deliberately does NOT derive `AsChangeset`. Mutating historical rows
/// in place would break the audit trail and the parity between `valid_from`
/// and action time. If a narrow update path is ever needed (e.g. fixing
/// `updated_by` on the most-recent row), define a dedicated changeset struct.
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_messaging_config))]
pub struct GovernanceMessagingConfigInsertForm {
  pub scope: String,
  pub key: String,
  pub value_type: String,
  pub value_int: Option<i64>,
  pub value_bool: Option<bool>,
  pub value_text: Option<String>,
  pub updated_by: Option<PersonId>,
}

#[cfg(feature = "full")]
impl GovernanceMessagingConfig {
  pub async fn create(
    pool: &mut DbPool<'_>,
    form: &GovernanceMessagingConfigInsertForm,
  ) -> LemmyResult<Self> {
    let conn = &mut get_conn(pool).await?;
    insert_into(governance_messaging_config::table)
      .values(form)
      .get_result::<Self>(conn)
      .await
      .with_lemmy_type(LemmyErrorType::CouldntCreate)
  }

  pub async fn read_current(
    pool: &mut DbPool<'_>,
    scope: &str,
    key: &str,
  ) -> LemmyResult<Option<Self>> {
    let conn = &mut get_conn(pool).await?;
    Ok(
      governance_messaging_config::table
        .filter(governance_messaging_config::scope.eq(scope))
        .filter(governance_messaging_config::key.eq(key))
        .order_by(governance_messaging_config::valid_from.desc())
        .first::<Self>(conn)
        .await
        .optional()?,
    )
  }
}
