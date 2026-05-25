//! Rule-set version CRUD per [05 §2] task v1-AD-c.
//!
//! `admin_create_rule_set` writes a new `rule_set_version` row plus a
//! `governance_config` row flipping `rule_set.active_version_id` for the
//! community, plus a `governance_log` entry — all in a single
//! `run_transaction` so a partial failure can never leave the community
//! with an active-version pointer to a non-existent row, nor a row in
//! the version table that no log entry attests to. The SAVEPOINT
//! semantics inside `governance_log::append` (see
//! [`lemmy_db_schema::source::governance::governance_log::append`])
//! ensure a signing-step failure rolls the whole tx back.
//!
//! `admin_list_rule_sets` is a plain read — moderator/admin gate, then
//! `SELECT * FROM rule_set_version WHERE community_id = ? ORDER BY
//! version DESC`, augmented with the active version id from
//! `governance_config_current`. v1-AD-c leaves `created_by_pseudonym`
//! as `None`; the secondary lookup against `actor_pseudonym` is v1-AD-d
//! work.

use actix_web::web::{Data, Json, Query};
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncPgConnection, RunQueryDsl};
use lemmy_api_common::governance::{
  AdminCreateRuleSet, AdminCreateRuleSetResponse, AdminListRuleSetsRequest,
  AdminListRuleSetsResponse, RuleSetVersionView,
};
use lemmy_api_utils::{context::LemmyContext, utils::is_admin};
use lemmy_db_schema::{
  newtypes::{CommunityId, RuleSetVersionId},
  source::governance::{
    governance_config::{GovernanceConfig, GovernanceConfigInsertForm},
    governance_log::ENTRY_KIND_RULE_SET_VERSION_CREATED,
    rule_set_version::{RuleSetVersion, RuleSetVersionInsertForm},
  },
};
use lemmy_db_schema_file::{
  PersonId,
  schema::{governance_config, rule_set_version},
};
use lemmy_db_views_community_moderator::CommunityModeratorView;
use lemmy_db_views_local_user::LocalUserView;
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};
use serde_json::json;
use sha2::{Digest, Sha256};

use crate::governance::{
  actor_pseudonym_helper,
  config::{self, Scope},
  governance_log,
  redaction::scrub,
};

pub async fn admin_create_rule_set(
  data: Json<AdminCreateRuleSet>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminCreateRuleSetResponse>> {
  let data = data.into_inner();

  // 1. Capability gate — community moderator OR instance admin. Match
  //    `NotAModerator` explicitly; any other error (transient DB/read
  //    failure) is propagated so a denial log is not falsely written.
  let is_mod = match CommunityModeratorView::check_is_community_moderator(
    &mut context.pool(),
    data.community_id,
    local_user_view.person.id,
  )
  .await
  {
    Ok(()) => true,
    Err(e) if matches!(e.error_type, LemmyErrorType::NotAModerator) => false,
    Err(e) => return Err(e),
  };
  let is_admin_ok = is_admin(&local_user_view).is_ok();
  if !(is_mod || is_admin_ok) {
    emit_rule_set_denial_log(
      &mut context.pool(),
      local_user_view.person.id,
      data.community_id,
      &data.reason,
      "community_moderator_required",
    )
    .await?;
    return Err(LemmyErrorType::NotAnAdmin.into());
  }

  // 2. Validate rule_text length against instance config.
  let mut cache = config::ConfigCache::new();
  let max_bytes = config::get_int_opt(
    &mut cache,
    &mut context.pool(),
    Scope::Instance,
    "rule_set.text_max_bytes",
  )
  .await?
  .unwrap_or(65_536);
  if i64::try_from(data.rule_text.len()).unwrap_or(i64::MAX) > max_bytes {
    return Err(
      LemmyErrorType::Unknown(format!(
        "rule_text exceeds rule_set.text_max_bytes = {max_bytes} bytes"
      ))
      .into(),
    );
  }
  if data.rule_text.is_empty() {
    return Err(LemmyErrorType::Unknown("rule_text cannot be empty".to_string()).into());
  }

  // 3. Validate parent_id (if provided): must exist and belong to the same
  //    community. Pre-tx check so a malformed request never opens a tx.
  if let Some(pid) = data.parent_id {
    validate_parent_id(&mut context.pool(), pid, data.community_id).await?;
  }

  // 4. Compute version + canonical SHA-256 over UTF-8 bytes.
  let previous_version = lookup_latest_version(&mut context.pool(), data.community_id).await?;
  let new_version = previous_version.map_or(1, |v| v + 1);
  let text_sha256 = Sha256::digest(data.rule_text.as_bytes()).to_vec();

  // 5. Pseudonym for the actor (idempotent on first use per phase 5b).
  let admin_pseudonym =
    actor_pseudonym_helper::get_or_create(&mut context.pool(), local_user_view.person.id).await?;

  // 6. Three-row atomic write — see process_create_rule_set.
  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let admin_id = local_user_view.person.id;
  let pseudonym_for_tx = admin_pseudonym.clone();
  let rule_text_for_tx = data.rule_text.clone();
  let parent_id_for_tx = data.parent_id.map(RuleSetVersionId);
  let community_id_for_tx = data.community_id;
  let text_sha256_for_tx = text_sha256.clone();

  let (rsv_row, cfg_id, log_id, created_at) = conn
    .run_transaction(async |conn| {
      let args = CreateRuleSetTxArgs {
        admin_id,
        admin_pseudonym: pseudonym_for_tx.clone(),
        community_id: community_id_for_tx,
        new_version,
        parent_id: parent_id_for_tx,
        text_sha256: text_sha256_for_tx.clone(),
        rule_text: rule_text_for_tx.clone(),
      };
      process_create_rule_set(conn, args).await
    })
    .await?;

  Ok(Json(AdminCreateRuleSetResponse {
    rule_set_version_id: rsv_row.id.0,
    version: rsv_row.version,
    config_id: Some(i64::from(cfg_id)),
    governance_log_id: log_id,
    created_at,
  }))
}

struct CreateRuleSetTxArgs {
  admin_id: PersonId,
  admin_pseudonym: String,
  community_id: CommunityId,
  new_version: i32,
  parent_id: Option<RuleSetVersionId>,
  text_sha256: Vec<u8>,
  rule_text: String,
}

async fn process_create_rule_set(
  conn: &mut AsyncPgConnection,
  args: CreateRuleSetTxArgs,
) -> LemmyResult<(RuleSetVersion, i32, i64, DateTime<Utc>)> {
  let CreateRuleSetTxArgs {
    admin_id,
    admin_pseudonym,
    community_id,
    new_version,
    parent_id,
    text_sha256,
    rule_text,
  } = args;

  // Write 1 — rule_set_version. UNIQUE(community_id, version) catches a
  // concurrent insert with the same version; surface as a retryable error.
  let rsv_form = RuleSetVersionInsertForm {
    community_id,
    version: new_version,
    parent_id,
    text_sha256: text_sha256.clone(),
    rule_text,
    created_by: Some(admin_id),
  };
  let rsv: RuleSetVersion = diesel::insert_into(rule_set_version::table)
    .values(&rsv_form)
    .returning(RuleSetVersion::as_returning())
    .get_result(conn)
    .await
    .map_err(map_rsv_unique_violation)?;

  // Write 2 — governance_config row flipping rule_set.active_version_id.
  let cfg_form = GovernanceConfigInsertForm {
    scope: Scope::Community(community_id).as_str().into_owned(),
    key: "rule_set.active_version_id".to_string(),
    value_type: "int".to_string(),
    value_int: Some(i64::from(rsv.id.0)),
    value_float: None,
    value_bool: None,
    value_text: None,
    updated_by: Some(admin_id),
  };
  let cfg: GovernanceConfig = diesel::insert_into(governance_config::table)
    .values(&cfg_form)
    .returning(GovernanceConfig::as_returning())
    .get_result(conn)
    .await?;

  // Write 3 — governance_log via append. SAVEPOINT-promoted inside the
  // outer tx; signing failure rolls all three writes back.
  let payload = json!({
    "community_id":        community_id.0,
    "version":             rsv.version,
    "parent_id":           rsv.parent_id.map(|p| p.0),
    "text_sha256":         hex::encode(&text_sha256),
    "rule_set_version_id": rsv.id.0,
    "config_id":           cfg.id.0,
    "activated_at":        rsv.created_at,
  });
  let log_row = governance_log::append(
    &mut conn.into(),
    ENTRY_KIND_RULE_SET_VERSION_CREATED,
    payload,
    Some(admin_pseudonym),
  )
  .await?;

  let created_at = rsv.created_at;
  Ok((rsv, cfg.id.0, log_row.id.0, created_at))
}

/// Map a diesel error from the `rule_set_version` INSERT to a `LemmyError`.
/// `UniqueViolation` on `(community_id, version)` is a retry-shaped `Unknown`;
/// all other errors pass through via the standard `.into()`.
///
/// Extracted so that `tests/e2e.rs::admin_create_rule_set_duplicate_version_rejected`
/// exercises the real production mapping rather than recreating it inline
/// (CR PR #81 round 2 finding D).
pub fn map_rsv_unique_violation(err: diesel::result::Error) -> lemmy_utils::error::LemmyError {
  match err {
    diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, _) => {
      LemmyErrorType::Unknown(
        "rule_set_version already exists for this community + version — retry".to_string(),
      )
      .into()
    }
    other => other.into(),
  }
}

pub async fn admin_list_rule_sets(
  data: Query<AdminListRuleSetsRequest>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminListRuleSetsResponse>> {
  let data = data.into_inner();

  // Capability gate — match `NotAModerator` explicitly; propagate other
  // errors (transient DB/read failures) rather than masking as NotAnAdmin.
  let is_mod = match CommunityModeratorView::check_is_community_moderator(
    &mut context.pool(),
    data.community_id,
    local_user_view.person.id,
  )
  .await
  {
    Ok(()) => true,
    Err(e) if matches!(e.error_type, LemmyErrorType::NotAModerator) => false,
    Err(e) => return Err(e),
  };
  let is_admin_ok = is_admin(&local_user_view).is_ok();
  if !(is_mod || is_admin_ok) {
    return Err(LemmyErrorType::NotAnAdmin.into());
  }

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  let community_id = data.community_id;
  let (rows, active_version_id): (Vec<RuleSetVersion>, Option<i32>) = conn
    .run_transaction(async |conn| {
      let rows: Vec<RuleSetVersion> = rule_set_version::table
        .filter(rule_set_version::community_id.eq(community_id))
        .order(rule_set_version::version.desc())
        .select(RuleSetVersion::as_select())
        .load(conn)
        .await?;

      let mut cache = config::ConfigCache::new();
      let active_version_id = config::get_int_opt(
        &mut cache,
        &mut conn.into(),
        Scope::Community(community_id),
        "rule_set.active_version_id",
      )
      .await?
      .and_then(|i| i32::try_from(i).ok());

      Ok::<_, lemmy_utils::error::LemmyError>((rows, active_version_id))
    })
    .await?;

  let versions = rows
    .into_iter()
    .map(|rsv| RuleSetVersionView {
      id: rsv.id.0,
      community_id: rsv.community_id,
      version: rsv.version,
      parent_id: rsv.parent_id.map(|p| p.0),
      text_sha256_hex: hex::encode(&rsv.text_sha256),
      rule_text: scrub(&rsv.rule_text),
      created_at: rsv.created_at,
      created_by_pseudonym: None,
    })
    .collect();

  Ok(Json(AdminListRuleSetsResponse {
    versions,
    active_version_id,
  }))
}

async fn lookup_latest_version(
  pool: &mut DbPool<'_>,
  community_id: CommunityId,
) -> LemmyResult<Option<i32>> {
  let conn = &mut get_conn(pool).await?;
  let v: Option<i32> = rule_set_version::table
    .filter(rule_set_version::community_id.eq(community_id))
    .order(rule_set_version::version.desc())
    .select(rule_set_version::version)
    .first(conn)
    .await
    .optional()?;
  Ok(v)
}

async fn validate_parent_id(
  pool: &mut DbPool<'_>,
  parent_wire_id: i32,
  community_id: CommunityId,
) -> LemmyResult<()> {
  let conn = &mut get_conn(pool).await?;
  let parent_community: Option<CommunityId> = rule_set_version::table
    .filter(rule_set_version::id.eq(RuleSetVersionId(parent_wire_id)))
    .select(rule_set_version::community_id)
    .first(conn)
    .await
    .optional()?;
  match parent_community {
    Some(cid) if cid == community_id => Ok(()),
    Some(_) => {
      Err(LemmyErrorType::Unknown("parent_id belongs to a different community".to_string()).into())
    }
    None => Err(LemmyErrorType::Unknown(format!("parent_id {parent_wire_id} not found")).into()),
  }
}

async fn emit_rule_set_denial_log(
  pool: &mut DbPool<'_>,
  admin_id: PersonId,
  community_id: CommunityId,
  reason: &str,
  denial_reason: &str,
) -> LemmyResult<()> {
  let pseudonym = actor_pseudonym_helper::get_or_create(pool, admin_id).await?;
  let payload = json!({
    "scope":         Scope::Community(community_id).as_str(),
    "key":           "rule_set.active_version_id",
    "value_type":    "int",
    "value":         null,
    "reason":        reason,
    "denial_reason": denial_reason,
  });
  governance_log::append(
    pool,
    governance_log::ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED,
    payload,
    Some(pseudonym),
  )
  .await?;
  Ok(())
}
