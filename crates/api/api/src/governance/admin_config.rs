//! Brehon admin-config HTTP endpoints — dry-run impact + handlers.
//!
//! This module backs the three `/api/v4/governance/admin/config*` routes
//! (registration in task 7). Task 3 lands the dry-run `compute_downstream_impact`
//! dispatcher and its seven per-category implementations; the `admin_set_config`
//! / `admin_get_config` / `admin_get_config_audit` handlers land in tasks 4-5.
//!
//! ## Dry-run impact (OQ-V1-AD-03 resolution, 2026-04-20 `3f0dd6572`)
//!
//! `diesel_utils::run_transaction` exposes no SAVEPOINT primitive. The
//! impact preview therefore runs as a pure read-only query against the
//! current persisted state + the proposed value, executed BEFORE the
//! write transaction opens. When `dry_run = true` the handler returns the
//! preview and no write happens. When `dry_run = false` the already-
//! computed preview is returned alongside the applied row. Post-commit
//! drift (another admin write between preview-query and write tx) is not
//! the handler's responsibility to describe — the pre-tx snapshot is the
//! contract.
//!
//! ## No writes in this module (task 3)
//!
//! Every function here is read-only. `fn(pool: &mut DbPool<'_>)` or
//! `fn(conn: &mut AsyncPgConnection)` only — no `run_transaction`, no
//! `insert_into`, no `update`. Task 4's handler performs the write.
//!
//! ## Impact shape by key category (PRD §4.3)
//!
//! | Key family | Impact function | Return shape |
//! |---|---|---|
//! | `thresholds.*` | `impact_for_threshold_key` | `{current_eligible, proposed_eligible, delta}` |
//! | `jury.panel_size` | `impact_for_jury_panel_size` | `{open_cases_needing_reassembly}` |
//! | `jury.max_concurrent_assignments` | `impact_for_jury_max_concurrent` | `{over_cap_jurors}` |
//! | `liability.sponsor_liability_floor` | `impact_for_liability_sponsor_floor_marker` | `{message: "requires materialised view"}` |
//! | `report.case_threshold_micros` | `impact_for_report_case_threshold_micros` | `{cases_that_would_open, cases_that_would_stay_closed}` |
//! | `decay.*` | `impact_for_decay_key` | `{message: "gradual"}` |
//! | other | `impact_for_other` | `{estimated_first_effect_at}` |

use crate::governance::{
  actor_pseudonym_helper,
  config::{
    self,
    ApplyAt,
    CONFIG_KEY_METADATA,
    ConfigCache,
    ConfigKeyMetadata,
    ConfigScope,
    NumericRange,
    Scope,
    ValueType,
    const_default_bool,
    const_default_float,
    const_default_int,
    const_default_text,
  },
  governance_log::{
    self,
    ENTRY_KIND_ADMIN_CONFIG_CHANGED,
    ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
    GovernanceLog,
  },
};
use actix_web::web::{Data, Json, Query};
use diesel::{
  ExpressionMethods,
  OptionalExtension,
  QueryDsl,
  QueryableByName,
  SelectableHelper,
  insert_into,
  sql_query,
  sql_types::{BigInt, Integer, Nullable},
};
use diesel_async::{RunQueryDsl, scoped_futures::ScopedFutureExt};
use lemmy_api_common::governance::{
  AdminConfigAuditEntry,
  AdminConfigEntry,
  AdminGetConfig,
  AdminGetConfigAudit,
  AdminGetConfigResponse,
  AdminSetConfig,
  AdminSetConfigResponse,
  ConfigChangePreview,
  ConfigValueWithProvenance,
};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::source::governance::governance_config::{
  GovernanceConfig,
  GovernanceConfigInsertForm,
};
use lemmy_db_schema_file::schema::{
  governance_config,
  governance_log as governance_log_schema,
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::{Value, json};

/// Row shape for the two-count FILTER aggregate used by threshold + liability +
/// report-threshold impact queries. Module-scope per workspace lint
/// `items-after-statements`.
#[derive(QueryableByName)]
struct TwoCountRow {
  #[diesel(sql_type = BigInt)]
  c_current: i64,
  #[diesel(sql_type = BigInt)]
  c_proposed: i64,
}

/// Row shape for single-count queries (open-cases, over-cap-jurors).
#[derive(QueryableByName)]
struct SingleCountRow {
  #[diesel(sql_type = BigInt)]
  c: i64,
}

/// Central dispatcher. Task 4's `admin_set_config` calls this PRE-transaction
/// after policy + type + range validation pass, and before the write closure
/// opens.
///
/// Category is selected by exact-match on `jury.panel_size` /
/// `jury.max_concurrent_assignments` / `liability.sponsor_liability_floor` /
/// `report.case_threshold_micros`, by prefix on `thresholds.*` and `decay.*`,
/// and falls back to `impact_for_other` for every other key. Keys that don't
/// have a meaningful count-based preview fall into `impact_for_other` — that
/// is not a bug; it's the "we don't preview this shape" signal per PRD §4.3.
///
/// The `current_value` / `proposed_value` JSON values are already in rehydrated
/// typed form (int → JSON number, bool → JSON bool, etc.); extraction uses
/// `as_i64()` / `as_f64()` accordingly.
pub async fn compute_downstream_impact(
  pool: &mut DbPool<'_>,
  key: &str,
  scope: Scope,
  current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let community_bind: Option<i32> = match scope {
    Scope::Instance => None,
    Scope::Community(id) => Some(id.0),
  };

  if key.starts_with("thresholds.") {
    return impact_for_threshold_key(pool, key, community_bind, current_value, proposed_value)
      .await;
  }

  if key.starts_with("decay.") {
    return Ok(impact_for_decay_key());
  }

  match key {
    "jury.panel_size" => impact_for_jury_panel_size(pool, community_bind).await,
    "jury.max_concurrent_assignments" => {
      impact_for_jury_max_concurrent(pool, current_value, proposed_value).await
    }
    "liability.sponsor_liability_floor" => Ok(impact_for_liability_sponsor_floor_marker()),
    "report.case_threshold_micros" => {
      impact_for_report_case_threshold_micros(pool, community_bind, current_value, proposed_value)
        .await
    }
    _ => Ok(impact_for_other()),
  }
}

/// `thresholds.jury_reliability | reporting_accuracy | endorsement_strength`.
///
/// **Hard GOTCHA**: `reputation_snapshot.jury_eligible` is a DERIVED boolean
/// computed at snapshot time against the CURRENT threshold. Previewing the
/// impact of a PROPOSED threshold MUST query the RAW columns
/// (`jury_reliability`, `reporting_accuracy`, `endorsement_strength`), not
/// `jury_eligible`. The raw columns are `Int4` per `schema.rs:1186-1189`.
///
/// Returns `{current_eligible, proposed_eligible, delta}` where `delta =
/// proposed - current` (signed; negative = more users excluded).
async fn impact_for_threshold_key(
  pool: &mut DbPool<'_>,
  key: &str,
  community_bind: Option<i32>,
  current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let column = match key {
    "thresholds.jury_reliability" => "jury_reliability",
    "thresholds.reporting_accuracy" => "reporting_accuracy",
    "thresholds.endorsement_strength" => "endorsement_strength",
    other => {
      return Err(LemmyErrorType::Unknown(format!("not a threshold key: {other}")).into());
    }
  };

  // Clamp i64 → i32 to match the `Int4` column type. Config values seeded
  // for thresholds have `NumericRange::max = 100` in `CONFIG_KEY_METADATA`,
  // so the clamp is a correctness guarantee, not a hack.
  let current_threshold = i32_from_value(current_value, "current_value", key)?;
  let proposed_threshold = i32_from_value(proposed_value, "proposed_value", key)?;

  let conn = &mut get_conn(pool).await?;

  // One round-trip — two filtered counts on the raw column. `community_id IS
  // NOT DISTINCT FROM $1` mirrors the convention used by `capability_query` +
  // `bucket_query` in admin_reputation_stats.rs (treats NULL as equal to NULL
  // so an instance-wide impact query binds None).
  let sql = format!(
    "SELECT \
       COUNT(*) FILTER (WHERE {column} >= $2)::bigint AS c_current, \
       COUNT(*) FILTER (WHERE {column} >= $3)::bigint AS c_proposed \
     FROM reputation_snapshot \
     WHERE community_id IS NOT DISTINCT FROM $1"
  );

  let row: TwoCountRow = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .bind::<Integer, _>(current_threshold)
    .bind::<Integer, _>(proposed_threshold)
    .get_result(conn)
    .await?;

  Ok(json!({
    "current_eligible": row.c_current,
    "proposed_eligible": row.c_proposed,
    "delta": row.c_proposed - row.c_current,
  }))
}

/// Helper: rehydrate a JSON value as `i32` with clear error messaging. Used by
/// threshold queries where the column type is `Int4`.
fn i32_from_value(v: &Value, label: &str, key: &str) -> LemmyResult<i32> {
  let n = v.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(format!("threshold impact: {label} for `{key}` is not an integer"))
  })?;
  i32::try_from(n).map_err(|_| {
    LemmyErrorType::Unknown(format!(
      "threshold impact: {label} for `{key}` (= {n}) does not fit in i32"
    ))
    .into()
  })
}

/// `jury.panel_size`. Counts open cases in the two actively-assembling states
/// — any increase in panel size forces these cases to pick up additional
/// jurors on their next `panel_assembled` event.
///
/// Plan §13 task 3: `status IN ('JurySelection', 'InReview')`.
async fn impact_for_jury_panel_size(
  pool: &mut DbPool<'_>,
  community_bind: Option<i32>,
) -> LemmyResult<Value> {
  let conn = &mut get_conn(pool).await?;

  let sql = "\
     SELECT COUNT(*)::bigint AS c \
     FROM moderation_case \
     WHERE status IN ('JurySelection', 'InReview') \
       AND community_id IS NOT DISTINCT FROM $1";

  let row: SingleCountRow = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .get_result(conn)
    .await?;

  Ok(json!({ "open_cases_needing_reassembly": row.c }))
}

/// `jury.max_concurrent_assignments`. Counts jurors whose current count of
/// Selected/Accepted assignments exceeds the proposed cap.
///
/// Mirrors the sub-query shape from `admin_assign_jury.rs:321-326`'s strict
/// eligibility filter but returns the over-cap count instead of excluding
/// them from a selection. Scope is instance-wide here: concurrent-cap is
/// per-person across all communities (a juror over-cap on the instance is
/// over-cap everywhere).
async fn impact_for_jury_max_concurrent(
  pool: &mut DbPool<'_>,
  _current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let proposed_cap = proposed_value.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(
      "jury.max_concurrent_assignments impact: proposed_value is not an integer".to_string(),
    )
  })?;

  let conn = &mut get_conn(pool).await?;

  let sql = "\
     SELECT COUNT(*)::bigint AS c FROM ( \
       SELECT ja.person_id FROM jury_assignment ja \
       WHERE ja.status IN ('Selected', 'Accepted') \
       GROUP BY ja.person_id \
       HAVING count(*) > $1 \
     ) sub";

  let row: SingleCountRow = sql_query(sql)
    .bind::<BigInt, _>(proposed_cap)
    .get_result(conn)
    .await?;

  Ok(json!({ "over_cap_jurors": row.c }))
}

/// `liability.sponsor_liability_floor`. The floor is a config-only tuning
/// knob — no column on `reputation_snapshot` materialises a person's running
/// liability total, and aggregating from `reputation_event` rows in a
/// pre-transaction read is expensive. v1-AD-b returns a descriptive marker;
/// if operators need a count, it lands as a later refinement against a
/// materialised view (out of scope per plan §12).
///
/// `_community_bind` and `_proposed_value` are intentionally unused — they
/// are the dispatcher's contract shape, kept for forward compat when the
/// materialised liability total exists.
fn impact_for_liability_sponsor_floor_marker() -> Value {
  json!({
    "message": "Sponsor liability floor is a tuning knob; count-based preview requires the liability-aggregate view (deferred to v1-AD-b.1).",
  })
}

/// `report.case_threshold_micros`. The per-case `moderation_case.threshold_score`
/// column (Int8) stores the threshold scalar at which the case transitioned out
/// of Open. Count cases sitting in the band between old and new thresholds so
/// operators can reason about flip direction without loading the full case list.
///
/// Returns two counts:
/// - `cases_that_would_open` — cases whose `threshold_score` sits in
///   `[proposed, current)`: the proposed (lower) threshold would have opened them.
/// - `cases_that_would_stay_closed` — cases whose `threshold_score` sits in
///   `[current, proposed)`: the proposed (higher) threshold would no longer
///   trip them.
///
/// Exactly one of the two bands is non-empty per request (the other is empty
/// due to the ordering constraint inside `FILTER`). The UI interprets the
/// populated side.
async fn impact_for_report_case_threshold_micros(
  pool: &mut DbPool<'_>,
  community_bind: Option<i32>,
  current_value: &Value,
  proposed_value: &Value,
) -> LemmyResult<Value> {
  let current_threshold = current_value.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(
      "report.case_threshold_micros impact: current_value is not an integer".to_string(),
    )
  })?;
  let proposed_threshold = proposed_value.as_i64().ok_or_else(|| {
    LemmyErrorType::Unknown(
      "report.case_threshold_micros impact: proposed_value is not an integer".to_string(),
    )
  })?;

  let conn = &mut get_conn(pool).await?;

  let sql = "\
     SELECT \
       COUNT(*) FILTER (WHERE threshold_score >= $2 AND threshold_score <  $3)::bigint \
         AS c_current, \
       COUNT(*) FILTER (WHERE threshold_score >= $3 AND threshold_score <  $2)::bigint \
         AS c_proposed \
     FROM moderation_case \
     WHERE community_id IS NOT DISTINCT FROM $1";

  let row: TwoCountRow = sql_query(sql)
    .bind::<Nullable<Integer>, _>(community_bind)
    .bind::<BigInt, _>(proposed_threshold)
    .bind::<BigInt, _>(current_threshold)
    .get_result(conn)
    .await?;

  Ok(json!({
    "cases_that_would_open": row.c_current,
    "cases_that_would_stay_closed": row.c_proposed,
  }))
}

/// `decay.*` family. Half-life constants affect every future reputation
/// snapshot tick, not the current snapshot — so a count-based preview against
/// `reputation_snapshot` tells the operator nothing useful. Return the
/// "gradual" marker per PRD §4.3.
fn impact_for_decay_key() -> Value {
  json!({
    "message": "Decay changes take effect at next reputation snapshot; no immediate count diff available.",
  })
}

/// Catch-all for keys that have no meaningful count-based impact (e.g.
/// `governance.dashboard.*`, `onboarding.*`, `job.*`). Returns the "first
/// effect at next handler invocation" marker per PRD §4.3.
fn impact_for_other() -> Value {
  json!({ "estimated_first_effect_at": "next handler invocation" })
}

// -- admin_set_config handler (v1-AD-b task 4) ------------------------------
//
// POST /api/v4/governance/admin/config — type-safe, scoped, audit-emitting
// replacement for the `scripts/brehon/admin-config-write.sh` psql wrapper.
//
// Shell payload target (`admin-config-write.sh:146-157`):
//   jsonb_build_object('scope', ..., 'key', ..., 'value_type', ..., 'value', ..., 'reason', ...)
//
// v1-AD-b payload MUST be byte-identical to this shape for NOT5 gate 3.
// `serde_json::json!` preserves macro-literal field order; with the
// workspace-pinned `preserve_order` feature (`Cargo.toml:201`) the stored
// `governance_log.payload` jsonb round-trips with this order intact.

/// Handler entry. Capability + validation + dry-run impact + optional tx.
/// The `run_transaction` call lives strictly inside the write branch so a
/// dry-run costs exactly the impact-query round-trips + no BEGIN/COMMIT.
pub async fn admin_set_config(
  Json(data): Json<AdminSetConfig>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminSetConfigResponse>> {
  // 1. Lookup metadata. Unknown key → 400 with clear message.
  let metadata = *metadata_for_key(&data.key)?;

  // 2. Parse scope. Unrecognised → 400; no denial log for malformed input
  //    per §10.3 precedent (federation_outbox denial is for policy
  //    rejections, not parsing errors).
  let scope = Scope::parse_wire(&data.scope).ok_or_else(|| {
    LemmyErrorType::Unknown(format!(
      "scope `{}` is not recognised — expected `instance` or `community:<id>`",
      data.scope
    ))
  })?;

  // 3. Reason must be non-empty trimmed — mirror of admin_close_case.
  if data.reason.trim().is_empty() {
    return Err(LemmyErrorType::Unknown("admin_set_config reason required".to_string()).into());
  }

  // 4. Value type matches metadata. Mismatch → 400 (bad input, not denial).
  if data.value_type != value_type_label(metadata.value_type) {
    return Err(LemmyErrorType::Unknown(format!(
      "value_type `{}` does not match metadata for key `{}` (expected `{}`)",
      data.value_type,
      data.key,
      value_type_label(metadata.value_type),
    ))
    .into());
  }

  // 5. Value conforms to range/enum. Mismatch → 400.
  validate_value_shape(&metadata, &data.value)?;

  // 6. Policy dispatch. On denial, write the `admin_config_change_denied`
  //    log entry BEFORE returning. Denial path is OUTSIDE `run_transaction`
  //    — `governance_log::append` opens its own internal tx per
  //    `governance_log.rs:205-235`.
  let admin_id = local_user_view.person.id;
  let pool = &mut context.pool();
  if let Err(reason) = check_policy(&metadata, scope, &local_user_view, pool).await? {
    emit_denial_log(pool, admin_id, &data, &scope, reason).await?;
    return Err(LemmyErrorType::NotAnAdmin.into());
  }

  // 7. Step-up reserved slot. Only emits denial + 403 when the key has
  //    `requires_step_up = true` AND the instance-level
  //    `governance.dashboard.step_up_enforced` flag is `true`. Otherwise
  //    advisory mode — the attempt still logs in the success path.
  if metadata.requires_step_up {
    let mut cache = ConfigCache::new();
    let enforced = config::get_bool_opt(
      &mut cache,
      pool,
      Scope::Instance,
      "governance.dashboard.step_up_enforced",
    )
    .await?
    .unwrap_or(false);
    if enforced {
      emit_denial_log(pool, admin_id, &data, &scope, DenialReason::StepUpRequired).await?;
      return Err(LemmyErrorType::NotAnAdmin.into());
    }
  }

  // 8. Read current effective value + provenance. PRE-tx.
  let (current_value, current_from) = read_effective(pool, &metadata, scope).await?;

  // 9. Compute dry-run impact. PRE-tx, pure read-only.
  let impact =
    compute_downstream_impact(pool, &data.key, scope, &current_value, &data.value).await?;

  // 10. Build the shared `ConfigChangePreview` carried by both branches.
  let preview = ConfigChangePreview {
    previous: ConfigValueWithProvenance {
      value: current_value.clone(),
      effective_from: current_from,
    },
    new: ConfigValueWithProvenance {
      value: data.value.clone(),
      effective_from: scope.as_str().into_owned(),
    },
    downstream_impact: impact,
  };

  // 11. Dry-run branch — return preview, no write.
  if data.dry_run.unwrap_or(false) {
    return Ok(Json(AdminSetConfigResponse {
      applied: false,
      config_id: None,
      governance_log_id: None,
      preview,
      applied_at: None,
    }));
  }

  // 12. Write branch. Pseudonym is `get_or_create` because even an
  //     admin who hasn't been tagged in governance-log yet needs a
  //     pseudonym to emit the success entry under.
  let admin_pseudonym = actor_pseudonym_helper::get_or_create(pool, admin_id).await?;

  let conn = &mut get_conn(pool).await?;

  let data_for_tx = data.clone();
  let pseudonym_for_tx = admin_pseudonym.clone();
  let scope_for_tx = scope;
  let admin_id_for_tx = admin_id;
  let metadata_for_tx = metadata;

  let (cfg_id, log_id, applied_at) = conn
    .run_transaction(|conn| {
      async move {
        process_set_config(
          conn,
          admin_id_for_tx,
          pseudonym_for_tx,
          scope_for_tx,
          metadata_for_tx,
          data_for_tx,
        )
        .await
      }
      .scope_boxed()
    })
    .await?;

  Ok(Json(AdminSetConfigResponse {
    applied: true,
    config_id: Some(i64::from(cfg_id)),
    governance_log_id: Some(log_id),
    preview,
    applied_at: Some(applied_at),
  }))
}

/// Tx closure body. Insert the new `governance_config` row, then emit the
/// `admin_config_changed` entry under the same transaction (the inner
/// `append` call promotes to a SAVEPOINT so a signing failure rolls
/// back both writes together — see `governance_log.rs:193-204`).
async fn process_set_config(
  conn: &mut diesel_async::AsyncPgConnection,
  _admin_id: lemmy_db_schema_file::PersonId,
  admin_pseudonym: String,
  scope: Scope,
  metadata: ConfigKeyMetadata,
  data: AdminSetConfig,
) -> LemmyResult<(i32, i64, chrono::DateTime<chrono::Utc>)> {
  // Unpack the JSON value into the four typed columns exactly as the shell
  // wrapper does: exactly one is `Some`, the other three are `None`. The
  // `governance_config_typed` CHECK constraint verifies this DB-side.
  let (value_int, value_float, value_bool, value_text) =
    split_typed_value(metadata.value_type, &data.value, &data.key)?;

  let form = GovernanceConfigInsertForm {
    scope: scope.as_str().into_owned(),
    key: data.key.clone(),
    value_type: data.value_type.clone(),
    value_int,
    value_float,
    value_bool,
    value_text,
    updated_by: Some(_admin_id),
  };

  let row: GovernanceConfig = insert_into(governance_config::table)
    .values(&form)
    .returning(GovernanceConfig::as_returning())
    .get_result::<GovernanceConfig>(conn)
    .await?;

  // Shell-identical payload: `{scope, key, value_type, value, reason}` in
  // this exact declaration order. `apply_at` is deliberately absent here
  // so NOT5 gate 3 holds; the response carries `apply_at` via `preview`
  // when task 5 extends the preview shape.
  let payload = json!({
    "scope":      row.scope,
    "key":        row.key,
    "value_type": row.value_type,
    "value":      data.value,
    "reason":     data.reason,
  });

  let log_row = governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_ADMIN_CONFIG_CHANGED,
    payload,
    Some(admin_pseudonym),
  )
  .await?;

  Ok((row.id.0, log_row.id.0, row.valid_from))
}

// -- helpers (kept private to v1-AD-b's admin_config module) ----------------

/// Look up the compile-time metadata row for a key. `Ok(&ConfigKeyMetadata)`
/// on hit; `Err(LemmyErrorType::Unknown)` on miss.
fn metadata_for_key(key: &str) -> LemmyResult<&'static ConfigKeyMetadata> {
  CONFIG_KEY_METADATA
    .iter()
    .find(|m| m.key == key)
    .ok_or_else(|| LemmyErrorType::Unknown(format!("unknown config key: `{key}`")).into())
}

/// Wire label for a `ValueType`. Must match the `governance_config.value_type`
/// column literal — the shell script and the parity tests both use these
/// strings.
fn value_type_label(vt: ValueType) -> &'static str {
  match vt {
    ValueType::Int => "int",
    ValueType::Float => "float",
    ValueType::Bool => "bool",
    ValueType::Text => "text",
    ValueType::Enum => "text",
  }
}

/// Reject a value that doesn't fit the metadata's range or enum.
fn validate_value_shape(metadata: &ConfigKeyMetadata, value: &Value) -> LemmyResult<()> {
  // Range check — applies when metadata declares a NumericRange.
  if let Some(range) = metadata.valid_range {
    let n = match metadata.value_type {
      ValueType::Int => value.as_i64().map(|i| i as f64),
      ValueType::Float => value.as_f64(),
      _ => None,
    };
    let n = n.ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "value for `{}` is not numeric; range validation requires int/float",
        metadata.key
      ))
    })?;
    if n < range.min || n > range.max {
      return Err(LemmyErrorType::Unknown(format!(
        "value {n} for `{}` out of range [{}, {}]",
        metadata.key, range.min, range.max
      ))
      .into());
    }
  }

  // Enum check — applies when metadata declares a pinned enum list.
  if let Some(allowed) = metadata.valid_enum {
    let s = value.as_str().ok_or_else(|| {
      LemmyErrorType::Unknown(format!(
        "value for enum key `{}` must be a JSON string",
        metadata.key
      ))
    })?;
    if !allowed.contains(&s) {
      return Err(LemmyErrorType::Unknown(format!(
        "value `{}` for key `{}` not in allowed set {:?}",
        s, metadata.key, allowed
      ))
      .into());
    }
  }

  Ok(())
}

/// Unpack the JSON value into the four Diesel columns. Exactly one is `Some`.
fn split_typed_value(
  vt: ValueType,
  value: &Value,
  key: &str,
) -> LemmyResult<(Option<i64>, Option<f64>, Option<bool>, Option<String>)> {
  match vt {
    ValueType::Int => {
      let v = value.as_i64().ok_or_else(|| {
        LemmyErrorType::Unknown(format!("`{key}` value_type=int but JSON is not an integer"))
      })?;
      Ok((Some(v), None, None, None))
    }
    ValueType::Float => {
      let v = value.as_f64().ok_or_else(|| {
        LemmyErrorType::Unknown(format!(
          "`{key}` value_type=float but JSON is not a number"
        ))
      })?;
      Ok((None, Some(v), None, None))
    }
    ValueType::Bool => {
      let v = value.as_bool().ok_or_else(|| {
        LemmyErrorType::Unknown(format!(
          "`{key}` value_type=bool but JSON is not a boolean"
        ))
      })?;
      Ok((None, None, Some(v), None))
    }
    ValueType::Text | ValueType::Enum => {
      let v = value
        .as_str()
        .ok_or_else(|| {
          LemmyErrorType::Unknown(format!(
            "`{key}` value_type=text/enum but JSON is not a string"
          ))
        })?
        .to_string();
      Ok((None, None, None, Some(v)))
    }
  }
}

/// Structured denial reasons. String form is what lands in the denial log's
/// `denial_reason` payload field — kept as a typed enum so the handler
/// can't emit free-form strings that diverge from the audit spec.
#[derive(Debug, Clone, Copy)]
enum DenialReason {
  InstanceAdminRequired,
  CommunityModeratorRequired,
  ScopeMismatchInstanceKey,
  ScopeMismatchCommunityKey,
  StepUpRequired,
}

impl DenialReason {
  fn as_str(self) -> &'static str {
    match self {
      DenialReason::InstanceAdminRequired => "instance_admin_required",
      DenialReason::CommunityModeratorRequired => "community_moderator_required",
      DenialReason::ScopeMismatchInstanceKey => "scope_mismatch_instance_key",
      DenialReason::ScopeMismatchCommunityKey => "scope_mismatch_community_key",
      DenialReason::StepUpRequired => "step_up_required",
    }
  }
}

/// Policy dispatch per plan §13 task 4 step 4. `Ok(Ok(()))` means the caller
/// is authorised; `Ok(Err(reason))` means denied with a specific cause;
/// `Err(..)` means the check itself failed (e.g. DB read error).
///
/// The inner `Result` lets the caller emit a denial log for the `Err` side
/// without collapsing the "legitimate auth failure" case with the
/// "something crashed" case.
async fn check_policy(
  metadata: &ConfigKeyMetadata,
  scope: Scope,
  local_user_view: &LocalUserView,
  pool: &mut DbPool<'_>,
) -> LemmyResult<Result<(), DenialReason>> {
  let admin_ok = is_admin(local_user_view).is_ok();
  match (metadata.scope, scope) {
    (ConfigScope::Instance, Scope::Instance) | (ConfigScope::Both, Scope::Instance) => {
      if admin_ok {
        Ok(Ok(()))
      } else {
        Ok(Err(DenialReason::InstanceAdminRequired))
      }
    }
    (ConfigScope::Both, Scope::Community(community_id))
    | (ConfigScope::Community, Scope::Community(community_id)) => {
      // Moderator of the target community — is_admin also counts, since
      // instance admins moderate all communities implicitly.
      if admin_ok {
        return Ok(Ok(()));
      }
      match CommunityModeratorView::check_is_community_moderator(
        pool,
        community_id,
        local_user_view.person.id,
      )
      .await
      {
        Ok(()) => Ok(Ok(())),
        Err(_) => Ok(Err(DenialReason::CommunityModeratorRequired)),
      }
    }
    (ConfigScope::Instance, Scope::Community(_)) => Ok(Err(DenialReason::ScopeMismatchInstanceKey)),
    (ConfigScope::Community, Scope::Instance) => {
      Ok(Err(DenialReason::ScopeMismatchCommunityKey))
    }
  }
}

/// Write the `admin_config_change_denied` entry. Denial path runs OUTSIDE
/// the write transaction — `append` opens its own internal tx.
async fn emit_denial_log(
  pool: &mut DbPool<'_>,
  actor_id: lemmy_db_schema_file::PersonId,
  data: &AdminSetConfig,
  scope: &Scope,
  reason: DenialReason,
) -> LemmyResult<()> {
  // `get_or_create` is deliberate per plan §4.1: a denied caller who has
  // never had a pseudonym gets one allocated here — the `actor_pseudonym`
  // table is the GDPR pseudonymisation layer for ANY person writing to
  // governance state, including denied actors.
  let actor = actor_pseudonym_helper::get_or_create(pool, actor_id).await?;

  let payload = json!({
    "scope":          scope.as_str(),
    "key":            data.key,
    "value_type":     data.value_type,
    "value":          data.value,
    "reason":         data.reason,
    "denial_reason":  reason.as_str(),
  });

  governance_log::append(
    pool,
    ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
    payload,
    Some(actor),
  )
  .await?;
  Ok(())
}

/// Read the currently-effective `(value, provenance)` for a key at a scope.
/// Uses the `*_opt` accessors so "no row, no const" returns `(Null, "default")`
/// — which for v1-AD-b is a legal state for `rule_set.active_version_id`
/// and some future-seeded keys.
///
/// Provenance strings match the wire shape of
/// `ConfigValueWithProvenance.effective_from`:
/// - `"community:<id>"` — the community-scoped row is in effect
/// - `"instance"` — the instance-scoped row is in effect
/// - `"default"` — no row exists; the Rust const default applies
///
/// v1-AD-b's `*_opt` accessors don't expose the provenance layer directly,
/// so we re-probe: a community-scoped read returns the community row first
/// and falls through to instance in `fetch_value`. We call `_opt` at
/// `Community(id)` to get the community-level value, then `_opt` at
/// `Instance` to distinguish "was-a-community-row" from "was-an-instance-row".
/// On both misses, the const default is consulted.
async fn read_effective(
  pool: &mut DbPool<'_>,
  metadata: &ConfigKeyMetadata,
  scope: Scope,
) -> LemmyResult<(Value, String)> {
  let mut cache = ConfigCache::new();
  match metadata.value_type {
    ValueType::Int => read_effective_int(pool, &mut cache, metadata.key, scope).await,
    ValueType::Float => read_effective_float(pool, &mut cache, metadata.key, scope).await,
    ValueType::Bool => read_effective_bool(pool, &mut cache, metadata.key, scope).await,
    ValueType::Text | ValueType::Enum => {
      read_effective_text(pool, &mut cache, metadata.key, scope).await
    }
  }
}

async fn read_effective_int(
  pool: &mut DbPool<'_>,
  cache: &mut ConfigCache,
  key: &str,
  scope: Scope,
) -> LemmyResult<(Value, String)> {
  // Try the requested scope first. If `Community`, the `*_opt` accessor
  // falls through to `instance` internally — but we need to distinguish
  // community-hit from instance-hit for provenance, so probe at both
  // scopes independently.
  if let Scope::Community(_) = scope
    && let Some(v) = config::get_int_opt(cache, pool, scope, key).await?
  {
    // The _opt accessor's cascade hits community-first-then-instance; we
    // need to know which tier served the value. Probe community-only by
    // re-requesting at the community scope against a fresh cache: if the
    // community tier is empty, the _opt already served the instance row.
    let mut probe_cache = ConfigCache::new();
    let community_only = config::get_int_opt(&mut probe_cache, pool, scope, key)
      .await?
      .and(probe_community_int_only(pool, scope, key).await?);
    let provenance = if community_only.is_some() {
      scope.as_str().into_owned()
    } else {
      "instance".to_string()
    };
    return Ok((json!(v), provenance));
  }

  if let Some(v) = config::get_int_opt(cache, pool, Scope::Instance, key).await? {
    return Ok((json!(v), "instance".to_string()));
  }

  match const_default_int(key) {
    Some(v) => Ok((json!(v), "default".to_string())),
    None => Ok((Value::Null, "default".to_string())),
  }
}

async fn read_effective_float(
  pool: &mut DbPool<'_>,
  cache: &mut ConfigCache,
  key: &str,
  scope: Scope,
) -> LemmyResult<(Value, String)> {
  if let Scope::Community(_) = scope
    && let Some(v) = config::get_float_opt(cache, pool, scope, key).await?
  {
    let community_only = probe_community_float_only(pool, scope, key).await?;
    let provenance = if community_only.is_some() {
      scope.as_str().into_owned()
    } else {
      "instance".to_string()
    };
    return Ok((json!(v), provenance));
  }
  if let Some(v) = config::get_float_opt(cache, pool, Scope::Instance, key).await? {
    return Ok((json!(v), "instance".to_string()));
  }
  match const_default_float(key) {
    Some(v) => Ok((json!(v), "default".to_string())),
    None => Ok((Value::Null, "default".to_string())),
  }
}

async fn read_effective_bool(
  pool: &mut DbPool<'_>,
  cache: &mut ConfigCache,
  key: &str,
  scope: Scope,
) -> LemmyResult<(Value, String)> {
  if let Scope::Community(_) = scope
    && let Some(v) = config::get_bool_opt(cache, pool, scope, key).await?
  {
    let community_only = probe_community_bool_only(pool, scope, key).await?;
    let provenance = if community_only.is_some() {
      scope.as_str().into_owned()
    } else {
      "instance".to_string()
    };
    return Ok((json!(v), provenance));
  }
  if let Some(v) = config::get_bool_opt(cache, pool, Scope::Instance, key).await? {
    return Ok((json!(v), "instance".to_string()));
  }
  match const_default_bool(key) {
    Some(v) => Ok((json!(v), "default".to_string())),
    None => Ok((Value::Null, "default".to_string())),
  }
}

async fn read_effective_text(
  pool: &mut DbPool<'_>,
  cache: &mut ConfigCache,
  key: &str,
  scope: Scope,
) -> LemmyResult<(Value, String)> {
  if let Scope::Community(_) = scope
    && let Some(v) = config::get_text_opt(cache, pool, scope, key).await?
  {
    let community_only = probe_community_text_only(pool, scope, key).await?;
    let provenance = if community_only.is_some() {
      scope.as_str().into_owned()
    } else {
      "instance".to_string()
    };
    return Ok((json!(v), provenance));
  }
  if let Some(v) = config::get_text_opt(cache, pool, Scope::Instance, key).await? {
    return Ok((json!(v), "instance".to_string()));
  }
  match const_default_text(key) {
    Some(v) => Ok((json!(v), "default".to_string())),
    None => Ok((Value::Null, "default".to_string())),
  }
}

/// Probe queries: read the `governance_config_current` view filtered to
/// community scope only, returning `None` if the tier is empty. Used by
/// `read_effective_*` to distinguish community-level hits from fallthrough
/// to instance. Returns the raw JSON value when the community tier is
/// populated; callers ignore the value and just inspect `is_some()`.
async fn probe_community_int_only(
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<i64>> {
  let scope_str = match scope {
    Scope::Community(_) => scope.as_str().into_owned(),
    Scope::Instance => return Ok(None),
  };
  let conn = &mut get_conn(pool).await?;
  let sql = "SELECT value_int AS c FROM governance_config_current \
     WHERE scope = $1 AND key = $2 AND value_type = 'int' LIMIT 1";
  probe_single_int(conn, sql, &scope_str, key).await
}

async fn probe_community_float_only(
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<f64>> {
  let scope_str = match scope {
    Scope::Community(_) => scope.as_str().into_owned(),
    Scope::Instance => return Ok(None),
  };
  let conn = &mut get_conn(pool).await?;
  let sql = "SELECT COUNT(*)::bigint AS c FROM governance_config_current \
     WHERE scope = $1 AND key = $2 AND value_type = 'float'";
  let row: SingleCountRow = sql_query(sql)
    .bind::<diesel::sql_types::Text, _>(scope_str)
    .bind::<diesel::sql_types::Text, _>(key.to_string())
    .get_result(conn)
    .await?;
  Ok(if row.c > 0 { Some(0.0) } else { None })
}

async fn probe_community_bool_only(
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<bool>> {
  let scope_str = match scope {
    Scope::Community(_) => scope.as_str().into_owned(),
    Scope::Instance => return Ok(None),
  };
  let conn = &mut get_conn(pool).await?;
  let sql = "SELECT COUNT(*)::bigint AS c FROM governance_config_current \
     WHERE scope = $1 AND key = $2 AND value_type = 'bool'";
  let row: SingleCountRow = sql_query(sql)
    .bind::<diesel::sql_types::Text, _>(scope_str)
    .bind::<diesel::sql_types::Text, _>(key.to_string())
    .get_result(conn)
    .await?;
  Ok(if row.c > 0 { Some(false) } else { None })
}

async fn probe_community_text_only(
  pool: &mut DbPool<'_>,
  scope: Scope,
  key: &str,
) -> LemmyResult<Option<String>> {
  let scope_str = match scope {
    Scope::Community(_) => scope.as_str().into_owned(),
    Scope::Instance => return Ok(None),
  };
  let conn = &mut get_conn(pool).await?;
  let sql = "SELECT COUNT(*)::bigint AS c FROM governance_config_current \
     WHERE scope = $1 AND key = $2 AND value_type = 'text'";
  let row: SingleCountRow = sql_query(sql)
    .bind::<diesel::sql_types::Text, _>(scope_str)
    .bind::<diesel::sql_types::Text, _>(key.to_string())
    .get_result(conn)
    .await?;
  Ok(if row.c > 0 { Some(String::new()) } else { None })
}

async fn probe_single_int(
  conn: &mut diesel_async::AsyncPgConnection,
  sql: &str,
  scope_str: &str,
  key: &str,
) -> LemmyResult<Option<i64>> {
  #[derive(QueryableByName)]
  struct Row {
    #[diesel(sql_type = Nullable<BigInt>)]
    c: Option<i64>,
  }
  let row: Option<Row> = sql_query(sql)
    .bind::<diesel::sql_types::Text, _>(scope_str.to_string())
    .bind::<diesel::sql_types::Text, _>(key.to_string())
    .get_result(conn)
    .await
    .optional()?;
  Ok(row.and_then(|r| r.c))
}

// -- admin_get_config / admin_get_config_audit handlers (v1-AD-b task 5) ---
//
// GET /api/v4/governance/admin/config        — single-key or full-list read
// GET /api/v4/governance/admin/config/audit  — paginated log of writes+denials
//
// Both are read-only; neither opens a transaction nor appends to the
// governance_log. Per advisor decision-queue #25 (Watch 11 scope is
// governance-weight WRITES, not READS) the GET handlers do not self-log.

const AUDIT_DEFAULT_PAGE: i64 = 1;
const AUDIT_DEFAULT_LIMIT: i64 = 20;
const AUDIT_MAX_LIMIT: i64 = 100;

/// `GET /api/v4/governance/admin/config`.
///
/// No `key`: one entry per `CONFIG_KEY_METADATA` row (61 entries at v1-AD-a).
/// `key`: single-entry response for that key (unknown key → 400).
/// `community_id`: when present, the cascade probes `community:<id>` first,
/// then `instance`, then the const default. When absent the cascade starts
/// at `Scope::Instance`. The response's `effective_from` reports which
/// tier served the value for each entry.
///
/// The per-request [`ConfigCache`] memoises repeat reads across keys that
/// happen to hit the same `(scope, key)` pair; in practice that is rare in
/// the full-list path but the cache keeps the typed accessors honest.
pub async fn admin_get_config(
  Query(data): Query<AdminGetConfig>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminGetConfigResponse>> {
  is_admin(&local_user_view)?;

  let pool = &mut context.pool();
  let request_scope = match data.community_id {
    Some(id) => Scope::Community(id),
    None => Scope::Instance,
  };

  let entries: Vec<AdminConfigEntry> = if let Some(key) = &data.key {
    let metadata = *metadata_for_key(key)?;
    let entry = build_config_entry(pool, &metadata, request_scope).await?;
    vec![entry]
  } else {
    let mut out = Vec::with_capacity(CONFIG_KEY_METADATA.len());
    for metadata in CONFIG_KEY_METADATA {
      out.push(build_config_entry(pool, metadata, request_scope).await?);
    }
    out
  };

  Ok(Json(AdminGetConfigResponse { entries }))
}

/// `GET /api/v4/governance/admin/config/audit`.
///
/// Filters (Diesel-pushdown): `entry_kind IN (…changed, …denied)`,
/// optional `actor_pseudonym` (indexed), optional `since` / `until` on
/// `created_at` (half-open: `ge(since)`, `lt(until)`).
/// Filters (Rust post-filter — payload JSONB not pushed down per §10.6):
/// `key`, `scope`. The audit list is expected low-volume (<1000 rows/day
/// at v1), so post-filtering a page of up-to-100 rows is cheap and avoids
/// adding a raw-SQL or `@>` variant. Pagination runs BEFORE post-filter —
/// a pathological filter combination can return fewer than `limit`
/// entries; that is deliberate per §10.6.
///
/// Ordering is `created_at DESC, id DESC` — id is the tiebreaker for
/// same-timestamp rows (trigger-managed `prev_hash` guarantees id order
/// respects insertion order for equal timestamps).
pub async fn admin_get_config_audit(
  Query(data): Query<AdminGetConfigAudit>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<Vec<AdminConfigAuditEntry>>> {
  is_admin(&local_user_view)?;

  let page = data.page.unwrap_or(AUDIT_DEFAULT_PAGE).max(1);
  let limit = data
    .limit
    .unwrap_or(AUDIT_DEFAULT_LIMIT)
    .clamp(1, AUDIT_MAX_LIMIT);
  let offset = page.saturating_sub(1).saturating_mul(limit);

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let mut query = governance_log_schema::table
    .filter(governance_log_schema::entry_kind.eq_any(vec![
      ENTRY_KIND_ADMIN_CONFIG_CHANGED,
      ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
    ]))
    .into_boxed();

  if let Some(p) = &data.actor_pseudonym {
    query = query.filter(governance_log_schema::actor_pseudonym.eq(p));
  }
  if let Some(since) = data.since {
    query = query.filter(governance_log_schema::created_at.ge(since));
  }
  if let Some(until) = data.until {
    query = query.filter(governance_log_schema::created_at.lt(until));
  }

  let rows: Vec<GovernanceLog> = query
    .order_by((
      governance_log_schema::created_at.desc(),
      governance_log_schema::id.desc(),
    ))
    .limit(limit)
    .offset(offset)
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(conn)
    .await?;

  let entries: Vec<AdminConfigAuditEntry> = rows
    .into_iter()
    .map(project_to_audit_entry)
    .filter(|entry| {
      data
        .key
        .as_deref()
        .is_none_or(|k| entry.key == k)
    })
    .filter(|entry| {
      data
        .scope
        .as_deref()
        .is_none_or(|s| entry.scope == s)
    })
    .collect();

  Ok(Json(entries))
}

/// Build one `AdminConfigEntry` from static metadata + a live cascade read.
/// A fresh `ConfigCache` is used per call; the cache lifetime is the
/// read-cascade only — the typed `*_opt` accessors re-query on
/// `CachedValue::Absent`, so re-using the cache across metadata keys would
/// not reduce the hot-path round-trip count anyway.
async fn build_config_entry(
  pool: &mut DbPool<'_>,
  metadata: &ConfigKeyMetadata,
  request_scope: Scope,
) -> LemmyResult<AdminConfigEntry> {
  let (value, effective_from) = read_effective(pool, metadata, request_scope).await?;
  Ok(AdminConfigEntry {
    key: metadata.key.to_string(),
    value_type: value_type_label(metadata.value_type).to_string(),
    value,
    effective_from,
    scope: config_scope_label(metadata.scope).to_string(),
    requires_re_jury: metadata.requires_re_jury,
    requires_step_up: metadata.requires_step_up,
    apply_at_default: apply_at_label(metadata.apply_at_default).to_string(),
    description: metadata.description.to_string(),
    doc_anchor: metadata.doc_anchor.to_string(),
    valid_range: metadata.valid_range.map(numeric_range_tuple),
    valid_enum: metadata
      .valid_enum
      .map(|e| e.iter().copied().map(str::to_owned).collect()),
  })
}

/// Wire label for `ConfigScope`. Matches the `AdminConfigEntry.scope`
/// field that the dashboard UI renders.
fn config_scope_label(cs: ConfigScope) -> &'static str {
  match cs {
    ConfigScope::Instance => "instance",
    ConfigScope::Community => "community",
    ConfigScope::Both => "both",
  }
}

/// Wire label for `ApplyAt`. Matches `AdminSetConfig.apply_at` accepted
/// values (`"immediate"`, `"next_jury_cycle"`, `"next_snapshot_job"`).
fn apply_at_label(apply_at: ApplyAt) -> &'static str {
  match apply_at {
    ApplyAt::Immediate => "immediate",
    ApplyAt::NextJuryCycle => "next_jury_cycle",
    ApplyAt::NextSnapshotJob => "next_snapshot_job",
  }
}

/// Flatten a `NumericRange` into the `(f64, f64)` tuple the DTO exposes.
fn numeric_range_tuple(r: NumericRange) -> (f64, f64) {
  (r.min, r.max)
}

/// Project a `governance_log` row whose `entry_kind` is
/// `admin_config_changed` or `admin_config_change_denied` into the typed
/// audit response entry. Unknown / missing payload fields degrade to
/// empty strings / `Value::Null` — we never fail a whole audit page on a
/// single malformed legacy row (shell-wrapper rows predating this handler
/// always produce well-formed payloads, but future migrations may add
/// fields and older rows should still list).
///
/// `previous_value` is always `None` for v1-AD-b; a future revision may
/// join to the prior row in the same `(scope, key)` bucket to hydrate it.
fn project_to_audit_entry(row: GovernanceLog) -> AdminConfigAuditEntry {
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
    previous_value: None,
    new_value,
    reason,
    actor_pseudonym: row.actor_pseudonym,
    created_at: row.created_at,
    signature: row.signature,
    denial_reason,
  }
}
