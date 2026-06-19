//! Admin handler for messaging-config: single-write `admin_set_messaging_config` +
//! read-only `admin_get_messaging_config`.
//!
//! Design: SINGLE insert, NO `run_transaction`, NO `governance_log::append`.
//! Per plan §10.3 GOTCHA: `admin_config.rs` wraps in a transaction only because
//! it appends a governance_log row (two writes need atomicity). M1 drops the
//! append; a single insert needs no transaction.

use actix_web::web::{Data, Json, Query};
use lemmy_api_common::governance::{
  AdminSetMessagingConfig,
  AdminSetMessagingConfigResponse,
  ConfigValueWithProvenance,
};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::governance_messaging_config::{
  GovernanceMessagingConfig,
  GovernanceMessagingConfigInsertForm,
};
use lemmy_db_views_local_user::LocalUserView;
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde::Deserialize;
use serde_json::Value;

/// Unpack a JSON value into (value_type, value_int, value_bool, value_text).
/// Mirrors `admin_config.rs:682` minus the Float arm (M1 has no value_float column).
/// Rejects float / array / object / null per plan §4.1.
// ponytail: a one-use private 4-tuple return; a named type alias would be churn, not clarity.
#[expect(clippy::type_complexity)]
fn split_typed_value(
  value: &Value,
  key: &str,
) -> LemmyResult<(String, Option<i64>, Option<bool>, Option<String>)> {
  if value.is_boolean() {
    let v = value.as_bool().ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "messaging-config value for key `{key}` must be bool, integer, or string"
      ))
    })?;
    Ok(("bool".to_string(), None, Some(v), None))
  } else if value.is_i64() || value.is_u64() {
    let v = value.as_i64().ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "messaging-config value for key `{key}` must be bool, integer, or string"
      ))
    })?;
    Ok(("int".to_string(), Some(v), None, None))
  } else if value.is_string() {
    let v = value
      .as_str()
      .ok_or_else(|| {
        LemmyErrorType::Unknown(format!(
          "messaging-config value for key `{key}` must be bool, integer, or string"
        ))
      })?
      .to_string();
    Ok(("text".to_string(), None, None, Some(v)))
  } else {
    Err(
      LemmyErrorType::Unknown(format!(
        "messaging-config value for key `{key}` must be bool, integer, or string"
      ))
      .into(),
    )
  }
}

// Rejects any attempt to set a jury/appeals room-type identity_policy to a non-pseudonymous value.
// Vacuously satisfied in M1 (jury/appeals rooms arrive in M2) — the gate exists NOW so M2 cannot
// introduce a non-pseudonymous default. ADR-015.
fn validate_identity_policy(data: &AdminSetMessagingConfig) -> LemmyResult<()> {
  if data.key == "identity_policy"
    && (data.scope.starts_with("jury") || data.scope.starts_with("appeal"))
    && data.value.as_str() != Some("pseudonymous")
  {
    return Err(
      LemmyErrorType::Unknown(format!(
        "identity_policy for scope `{}` must be `pseudonymous` (ADR-015 — jury/appeals rooms may never be non-pseudonymous)",
        data.scope
      ))
      .into(),
    );
  }
  Ok(())
}

/// Convert a stored config row back to a typed `serde_json::Value` for wire output.
fn row_to_value(row: &GovernanceMessagingConfig) -> Value {
  match row.value_type.as_str() {
    "int" => row
      .value_int
      .map(|v| Value::Number(v.into()))
      .unwrap_or(Value::Null),
    "bool" => row.value_bool.map(Value::Bool).unwrap_or(Value::Null),
    "text" => row
      .value_text
      .as_deref()
      .map(|s| Value::String(s.to_string()))
      .unwrap_or(Value::Null),
    _ => Value::Null,
  }
}

/// `POST /api/v4/governance/admin/messaging-config`
///
/// Admin write: derive value_type from JSON shape → single insert into
/// `governance_messaging_config`. SINGLE write, no transaction, no
/// governance_log append (plan §10.3 + §12).
pub async fn admin_set_messaging_config(
  Json(data): Json<AdminSetMessagingConfig>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<AdminSetMessagingConfigResponse>> {
  // 1. Admin capability gate (mirrors admin_config.rs:1099 `is_admin` pattern).
  is_admin(&local_user_view)?;
  validate_identity_policy(&data)?; // §10.4 — ADR-015 pin

  let pool = &mut context.pool();

  // 2. Derive value_type + split into typed columns. Reject float/array/object/null.
  let (value_type, value_int, value_bool, value_text) =
    split_typed_value(&data.value, &data.key)?;

  // 3. Read CURRENT value BEFORE the write → build `previous`.
  let previous =
    match GovernanceMessagingConfig::read_current(pool, &data.scope, &data.key).await? {
      Some(row) => ConfigValueWithProvenance {
        value: row_to_value(&row),
        effective_from: row.scope.clone(),
      },
      None => ConfigValueWithProvenance::default(),
    };

  // 4. Single insert. No transaction — one write needs no atomicity wrapper.
  let form = GovernanceMessagingConfigInsertForm {
    scope: data.scope.clone(),
    key: data.key.clone(),
    value_type,
    value_int,
    value_bool,
    value_text,
    updated_by: Some(local_user_view.person.id),
  };
  GovernanceMessagingConfig::create(pool, &form).await?;

  // 5. Build `new` from the written data (scope is the provenance string).
  let new = ConfigValueWithProvenance {
    value: data.value.clone(),
    effective_from: data.scope.clone(),
  };

  Ok(Json(AdminSetMessagingConfigResponse { previous, new }))
}

/// Query params for the GET endpoint.
#[derive(Deserialize)]
pub struct GetMessagingConfigQuery {
  scope: String,
  key: String,
}

/// `GET /api/v4/governance/admin/messaging-config`
///
/// Returns the current effective value for a `(scope, key)` pair from
/// `governance_messaging_config_current`, or a default
/// `ConfigValueWithProvenance` if no row exists yet.
pub async fn admin_get_messaging_config(
  Query(params): Query<GetMessagingConfigQuery>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<ConfigValueWithProvenance>> {
  is_admin(&local_user_view)?;

  let pool = &mut context.pool();
  let result =
    match GovernanceMessagingConfig::read_current(pool, &params.scope, &params.key).await? {
      Some(row) => ConfigValueWithProvenance {
        value: row_to_value(&row),
        effective_from: row.scope.clone(),
      },
      None => ConfigValueWithProvenance::default(),
    };

  Ok(Json(result))
}
