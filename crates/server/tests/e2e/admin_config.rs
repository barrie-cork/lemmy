mod admin_config_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::EnvVarGuard;
  use actix_web::web::Data;
  use diesel::{Connection as _, PgConnection};
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{InstanceId, PersonId};
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;

  /// Spin a fresh Postgres, apply the full Brehon schema, build a real
  /// `LemmyContext` wrapped in `Data`, and return both the context handle
  /// and the container guard (keep the container alive via `_container`).
  pub async fn bootstrap() -> LemmyResult<(
    testcontainers::ContainerAsync<testcontainers::GenericImage>,
    Data<LemmyContext>,
    String,
  )> {
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    // These two env vars are intentionally process-scoped (NOT EnvVarGuard-wrapped):
    // both are constant-valued ("1" / fixed signing seed) and bootstrap() has many
    // callers across this test module — wrapping here would drop the guard at
    // bootstrap() return, unsetting the var before the test body runs (see
    // feedback_envvarguard_fixture_lifetime_footgun.md). LEMMY_DATABASE_URL IS
    // guarded (per-call value) at the _g_db_url binding below.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (container, host_port) = super::governance_fixtures::start_postgres().await?;
    let db_url = super::governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      super::governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    let pool: ActualDbPool = build_db_pool_for_tests();
    let client = client_builder(&SETTINGS).build()?;
    let middleware_client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };
    let rate_limit = RateLimit::with_debug_config();
    // Bump rate-limit buckets — multi-write tests trip the 6/300s Post
    // bucket from `with_debug_config()`. See
    // `feedback_rate_limit_debug_config_post_bucket.md`.
    {
      use enum_map::enum_map;
      use lemmy_utils::rate_limit::{ActionType, BucketConfig};
      rate_limit.set_config(enum_map! {
        ActionType::Message => BucketConfig { max_requests: 10_000, interval: 60 },
        ActionType::Post => BucketConfig { max_requests: 10_000, interval: 60 },
        ActionType::Register => BucketConfig { max_requests: 10_000, interval: 60 },
        ActionType::Image => BucketConfig { max_requests: 10_000, interval: 60 },
        ActionType::Comment => BucketConfig { max_requests: 10_000, interval: 60 },
        ActionType::Search => BucketConfig { max_requests: 10_000, interval: 60 },
        ActionType::ImportUserSettings => BucketConfig { max_requests: 10_000, interval: 60 },
      });
    }
    let context = Data::new(LemmyContext::create(
      pool,
      middleware_client.clone(),
      middleware_client,
      secret,
      rate_limit,
    ));

    Ok((container, context, db_url))
  }

  /// Seed an instance + a single person/local_user pair, returning both the
  /// PersonId and the `LocalUserView` callers need to invoke handlers.
  pub async fn seed_user(
    ctx: &LemmyContext,
    instance_id: InstanceId,
    name: &str,
    is_admin: bool,
  ) -> LemmyResult<(PersonId, LocalUserView)> {
    let person_form = PersonInsertForm::test_form(instance_id, name);
    let person = Person::create(&mut ctx.pool(), &person_form).await?;
    let mut lu_form = if is_admin {
      LocalUserInsertForm::test_form_admin(person.id)
    } else {
      LocalUserInsertForm::test_form(person.id)
    };
    lu_form.accepted_application = Some(true);
    LocalUser::create(&mut ctx.pool(), &lu_form, vec![]).await?;
    let view = LocalUserView::read_person(&mut ctx.pool(), person.id).await?;
    Ok((person.id, view))
  }

  /// Read the first instance (auto-created by migrations as
  /// `local_site.site_id = 1`) or create a fresh `test.invalid` one.
  pub async fn bootstrap_instance(ctx: &LemmyContext) -> LemmyResult<Instance> {
    Instance::read_or_create(&mut ctx.pool(), "test.invalid").await
  }

  /// v1-AD-c task 8 helper: seed a non-admin user AND register them as a
  /// CommunityModerator on the given community. Returns the `LocalUserView`
  /// ready to pass to `admin_create_rule_set` / `admin_list_rule_sets`. The
  /// moderator path is the primary capability gate for rule-set CRUD —
  /// v1-AD-d will re-use this helper for the audit-display tests.
  pub async fn seed_community_moderator(
    ctx: &LemmyContext,
    instance_id: InstanceId,
    community_id: lemmy_db_schema::newtypes::CommunityId,
    name: &str,
  ) -> LemmyResult<LocalUserView> {
    use lemmy_db_schema::source::community::{CommunityActions, CommunityModeratorForm};
    let (person_id, view) = seed_user(ctx, instance_id, name, false).await?;
    CommunityActions::join(
      &mut ctx.pool(),
      &CommunityModeratorForm::new(community_id, person_id),
    )
    .await?;
    Ok(view)
  }
}

/// Task 8 test 1: instance-scope int write bumps `jury.panel_size` from 5
/// to 7, returns `applied=true` + both IDs, persists a `governance_config`
/// row, and emits exactly one `admin_config_changed` log entry.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_happy_path() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::{governance_config, governance_log};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_hp", true).await?;

  let resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "bump panel_size for test".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert!(resp.applied, "applied must be true on happy path");
  assert!(resp.config_id.is_some(), "config_id set");
  assert!(resp.governance_log_id.is_some(), "governance_log_id set");
  assert!(resp.applied_at.is_some(), "applied_at set");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let row_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .filter(governance_config::scope.eq("instance"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(row_count, 2, "seed row + new row = 2");

  let log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(log_count, 1, "exactly one admin_config_changed entry");

  Ok(())
}

/// Task 8 test 2: `dry_run = Some(true)` returns a populated preview but
/// writes nothing.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_dry_run() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::{governance_config, governance_log};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_dry", true).await?;

  let mut conn_before = AsyncPgConnection::establish(&db_url).await?;
  let before_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .count()
    .get_result(&mut conn_before)
    .await?;

  let resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(9),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: Some(true),
      reason: "dry-run probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert!(!resp.applied, "dry_run must set applied=false");
  assert!(
    resp.config_id.is_none(),
    "dry_run must NOT return config_id"
  );
  assert!(
    resp.governance_log_id.is_none(),
    "dry_run must NOT return log_id"
  );
  assert!(
    resp.applied_at.is_none(),
    "dry_run must NOT return applied_at"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let after_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    before_count, after_count,
    "dry_run must not append a config row"
  );

  let log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(log_count, 0, "dry_run must not emit admin_config_changed");

  Ok(())
}

/// Task 8 test 3: `value_type="int"` against a float-metadata key → 400,
/// no denial log (type mismatch is bad input, not a policy denial).
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_type_mismatch_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_tm", true).await?;

  // `liability.regular_multiplier` is declared float in metadata.
  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "liability.regular_multiplier".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(2),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "type-mismatch probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "type mismatch must be an error");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let denial_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    denial_count, 0,
    "type mismatch is not a policy denial → no denial log"
  );

  Ok(())
}

/// Task 8 test 4: panel_size=1000 is outside the declared 3-21 range →
/// 400, no denial log (range violation is bad input, not policy denial).
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_range_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::{governance_config, governance_log};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_rg", true).await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(1000),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "range violation probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "out-of-range must be an error");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let cfg_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(cfg_count, 1, "range violation must not insert a config row");

  let denial_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(denial_count, 0, "range violation is not a policy denial");

  Ok(())
}

/// Task 8 test 5: enum value not in `valid_enum` list → 400, no denial log.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_enum_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_en", true).await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.severity_thresholds.minor".to_string(),
      value_type: "text".to_string(),
      value: serde_json::json!("invalid"),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "enum violation probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "invalid enum value must be an error");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let denial_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(denial_count, 0, "enum violation is not a policy denial");

  Ok(())
}

/// Task 8 test 6: non-admin caller → 403 + denial log with
/// `denial_reason = "instance_admin_required"`.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_non_admin_rejected_with_denial_log() -> lemmy_utils::error::LemmyResult<()>
{
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "non_admin", false).await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "non-admin probe".to_string(),
    }),
    context.clone(),
    user_view,
  )
  .await;
  assert!(result.is_err(), "non-admin must be rejected");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  use diesel::SelectableHelper;
  let denial_rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(denial_rows.len(), 1, "exactly one denial entry");
  let payload = &denial_rows[0].payload;
  assert_eq!(
    payload.get("denial_reason").and_then(|v| v.as_str()),
    Some("instance_admin_required"),
    "denial_reason must be instance_admin_required",
  );
  assert!(
    denial_rows[0].actor_pseudonym.is_some(),
    "denied caller still gets a pseudonym (GDPR layer per plan §4.1)",
  );

  Ok(())
}

/// Task 8 test 7: instance-only key (e.g. `federation.inbound_advisory_only`)
/// with `scope: community:<id>` → 400 + denial log with
/// `denial_reason = "scope_mismatch_instance_key"`.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_scope_mismatch_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::governance_log::GovernanceLog,
  };
  use lemmy_db_schema_file::schema::governance_log;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_sm", true).await?;

  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "scope_mm".to_string(),
      "Scope Mismatch Community".to_string(),
      "pk-scope".to_string(),
    ),
  )
  .await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "federation.inbound_advisory_only".to_string(),
      value_type: "bool".to_string(),
      value: serde_json::json!(false),
      scope: format!("community:{}", community.id.0),
      apply_at: None,
      dry_run: None,
      reason: "scope-mismatch probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "scope mismatch must be rejected");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  use diesel::SelectableHelper;
  let denial_rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(denial_rows.len(), 1, "exactly one denial entry");
  assert_eq!(
    denial_rows[0]
      .payload
      .get("denial_reason")
      .and_then(|v| v.as_str()),
    Some("scope_mismatch_instance_key"),
    "denial_reason must be scope_mismatch_instance_key",
  );

  Ok(())
}

/// Task 8 test 8: `jury.quorum` has `ConfigScope::Both` so a community
/// moderator (not an instance admin) can write it at `community:<id>`
/// scope.
///
/// NOTE: v1-AD-b plan §11 line 1101 originally named
/// `liability.regular_multiplier` here, but that key is declared
/// `ConfigScope::Instance` in `config.rs:1141-1152` — not `Both`. Swapped
/// to `jury.quorum` (Int, range 1-21, `ConfigScope::Both`) which actually
/// exercises the moderator-write path. The `Both`-scope key set has no
/// float-typed member today, so picking an int is the minimal correction.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_community_scope_by_moderator() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::{
    community::{Community, CommunityActions, CommunityInsertForm, CommunityModeratorForm},
    governance::governance_config::GovernanceConfig,
  };
  use lemmy_db_schema_file::schema::governance_config;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (mod_id, mod_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "cmod", false).await?;

  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "cmod_comm".to_string(),
      "Community Mod".to_string(),
      "pk-cmod".to_string(),
    ),
  )
  .await?;
  CommunityActions::join(
    &mut context.pool(),
    &CommunityModeratorForm::new(community.id, mod_id),
  )
  .await?;

  let resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.quorum".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(5),
      scope: format!("community:{}", community.id.0),
      apply_at: None,
      dry_run: None,
      reason: "cmod sets jury quorum for community".to_string(),
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();
  assert!(
    resp.applied,
    "moderator write on Both-scope key must succeed"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  use diesel::SelectableHelper;
  let rows: Vec<GovernanceConfig> = governance_config::table
    .filter(governance_config::scope.eq(format!("community:{}", community.id.0)))
    .filter(governance_config::key.eq("jury.quorum"))
    .select(GovernanceConfig::as_select())
    .load::<GovernanceConfig>(&mut conn)
    .await?;
  assert_eq!(rows.len(), 1, "one community-scoped row appended");

  Ok(())
}

/// M1-b task 7 test 1: with `messaging_enabled` absent (clean posture), the
/// `governance_messaging_config` table has no row for `(instance, messaging_enabled)`.
/// `bridge_notify::notify_if_enabled` reads this via `read_current` and returns
/// `Ok(())` immediately — governance assertions are unaffected by the missing config.
#[tokio::test(flavor = "multi_thread")]
async fn messaging_disabled_preserves_governance_posture() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_db_schema::source::governance::governance_messaging_config::GovernanceMessagingConfig;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_person_id, _admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_cp", true).await?;

  let result =
    GovernanceMessagingConfig::read_current(&mut context.pool(), "instance", "messaging_enabled")
      .await?;
  assert_eq!(
    result.and_then(|row| row.value_bool),
    Some(false),
    "clean posture: migration seeds messaging_enabled=false row → bridge_notify no-op path is active",
  );

  Ok(())
}

/// M1-b task 7 test 2: validator rejects `identity_policy=real_name` for `jury` scope.
/// Admin caller (so rejection is from the validator, not the `is_admin` gate). ADR-015.
#[tokio::test(flavor = "multi_thread")]
async fn messaging_identity_policy_rejects_jury_override() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use lemmy_api::governance::messaging_config::admin_set_messaging_config;
  use lemmy_api_common::governance::AdminSetMessagingConfig;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_ip", true).await?;

  let result = admin_set_messaging_config(
    Json(AdminSetMessagingConfig {
      scope: "jury".to_string(),
      key: "identity_policy".to_string(),
      value: serde_json::json!("real_name"),
    }),
    admin_view,
    context.clone(),
  )
  .await;
  assert!(
    result.is_err(),
    "identity_policy=real_name for jury scope must be rejected (ADR-015)",
  );

  Ok(())
}

/// Task 8 test 9: GET /admin/config without filters returns every
/// CONFIG_KEY_METADATA row; each entry carries `effective_from` matching
/// either "default" (const) or "instance" (seed row).
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_full() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Query;
  use lemmy_api::governance::{admin_config::admin_get_config, config::CONFIG_KEY_METADATA};
  use lemmy_api_common::governance::AdminGetConfig;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_gf", true).await?;

  let resp = admin_get_config(
    Query(AdminGetConfig {
      key: None,
      community_id: None,
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(
    resp.entries.len(),
    CONFIG_KEY_METADATA.len(),
    "GET without filters returns one entry per metadata row",
  );
  for entry in &resp.entries {
    assert!(
      !entry.effective_from.is_empty(),
      "effective_from populated for key `{}`",
      entry.key,
    );
    assert!(
      matches!(
        entry.effective_from.as_str(),
        "default" | "instance" | "community"
      ),
      "effective_from must be default/instance/community, got `{}` for `{}`",
      entry.effective_from,
      entry.key,
    );
  }

  Ok(())
}

/// Task 8 test 10: after a successful write, GET with `?key=jury.panel_size`
/// returns a single entry whose `effective_from = "instance"` (the seed
/// row + the new instance-scope write both exist, and the latest-wins
/// probe picks the new one).
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_single_key_with_provenance() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_config::{admin_get_config, admin_set_config};
  use lemmy_api_common::governance::{AdminGetConfig, AdminSetConfig};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_sk", true).await?;

  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(11),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "provenance probe".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?
  .into_inner();

  let resp = admin_get_config(
    Query(AdminGetConfig {
      key: Some("jury.panel_size".to_string()),
      community_id: None,
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(
    resp.entries.len(),
    1,
    "single-key GET returns exactly one entry"
  );
  let entry = &resp.entries[0];
  assert_eq!(entry.key, "jury.panel_size");
  assert_eq!(
    entry.value,
    serde_json::json!(11),
    "value reflects the write"
  );
  assert_eq!(
    entry.effective_from, "instance",
    "effective_from must be 'instance' after an instance-scope write",
  );

  Ok(())
}

/// Task 8 test 11: five writes (3 successes, 2 denials) + a paginated GET
/// with `limit=3` returns three entries in descending created_at order.
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_audit_paginated() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_config::{admin_get_config_audit, admin_set_config};
  use lemmy_api_common::governance::{AdminGetConfigAudit, AdminSetConfig};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_ap", true).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "user_ap", false).await?;

  // 3 successes — increments to panel_size (5 → 7 → 9 → 11)
  for (i, v) in [7, 9, 11].iter().enumerate() {
    admin_set_config(
      Json(AdminSetConfig {
        key: "jury.panel_size".to_string(),
        value_type: "int".to_string(),
        value: serde_json::json!(*v),
        scope: "instance".to_string(),
        apply_at: None,
        dry_run: None,
        reason: format!("bump {i}"),
      }),
      context.clone(),
      admin_view.clone(),
    )
    .await?;
  }
  // 2 denials — non-admin attempts
  for i in 0..2 {
    let _ = admin_set_config(
      Json(AdminSetConfig {
        key: "jury.panel_size".to_string(),
        value_type: "int".to_string(),
        value: serde_json::json!(13),
        scope: "instance".to_string(),
        apply_at: None,
        dry_run: None,
        reason: format!("denied {i}"),
      }),
      context.clone(),
      user_view.clone(),
    )
    .await;
  }

  let resp = admin_get_config_audit(
    Query(AdminGetConfigAudit {
      key: None,
      scope: None,
      actor_pseudonym: None,
      since: None,
      until: None,
      page: None,
      limit: Some(3),
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(resp.len(), 3, "limit=3 returns three entries");
  for pair in resp.windows(2) {
    assert!(
      pair[0].created_at >= pair[1].created_at,
      "audit entries must be in desc created_at order",
    );
  }

  Ok(())
}

/// Task 8 test 12 (NOT5 gate 3, continuity semantics per PRD §8.4 condition 3,
/// amended 2026-04-21): write via HTTP handler AND write via raw SQL matching
/// the shell script's INSERT. The two payloads are **continuous**, not
/// byte-identical: for every key the shell script emits (`scope`, `key`,
/// `value_type`, `value`, `reason`) the two payloads must agree
/// byte-for-byte, AND the HTTP path may emit strictly more keys
/// (`previous_value`, `previous_from` as of v1-AD-c task 4, closes
/// GH #77). `project_to_audit_entry` degrades the HTTP-only fields to
/// `None` on shell-written rows, so downstream readers see a coherent
/// schema either way. This test guards the subset-parity contract +
/// asserts the additive fields land on the HTTP row and are absent on
/// the shell row (future drift would flip that asymmetry and must be
/// caught here).
#[tokio::test(flavor = "multi_thread")]
async fn governance_log_payload_shell_parity() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_pp", true).await?;

  // Write #1: via the Rust HTTP handler.
  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "parity probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?;

  // Write #2: via raw SQL matching admin-config-write.sh's payload-build SQL
  // EXACTLY (including JSON key declaration order, the previous_value/from
  // sub-SELECTs, and the to_char timestamp formatting).
  // Updated for #84 (option A): shell wrapper now emits previous_value +
  // previous_from at the tail. The sub-SELECTs read from governance_config
  // for the most-recent row (scope, key) with valid_from < now() — i.e. the
  // row Write #1 just inserted, since the HTTP handler writes to BOTH
  // governance_config and governance_log.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  diesel::sql_query(
    "INSERT INTO governance_log (entry_kind, payload, actor_pseudonym) VALUES (\
       'admin_config_changed',\
       jsonb_build_object(\
         'scope',          'instance',\
         'key',            'jury.panel_size',\
         'value_type',     'int',\
         'value',          7,\
         'reason',         'parity probe',\
         'previous_value', (\
           SELECT CASE prev.value_type \
                    WHEN 'int'   THEN to_jsonb(prev.value_int) \
                    WHEN 'float' THEN to_jsonb(prev.value_float) \
                    WHEN 'bool'  THEN to_jsonb(prev.value_bool) \
                    WHEN 'text'  THEN to_jsonb(prev.value_text) \
                  END \
           FROM governance_config prev \
           WHERE prev.scope = 'instance' \
             AND prev.key   = 'jury.panel_size' \
             AND prev.valid_from < now() \
           ORDER BY prev.valid_from DESC LIMIT 1 \
         ),\
         'previous_from', (\
           SELECT to_char(prev.valid_from AT TIME ZONE 'UTC', \
                          'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') \
           FROM governance_config prev \
           WHERE prev.scope = 'instance' \
             AND prev.key   = 'jury.panel_size' \
             AND prev.valid_from < now() \
           ORDER BY prev.valid_from DESC LIMIT 1 \
         )\
       ),\
       'shell-wrapper-pseudo'\
     )",
  )
  .execute(&mut conn)
  .await?;

  // Load both rows.
  let rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .order_by(governance_log::id.asc())
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(rows.len(), 2, "two admin_config_changed rows present");

  // entry_kind must match (trivially; already filtered).
  assert_eq!(rows[0].entry_kind, rows[1].entry_kind);

  // Subset parity: for every key the shell script emits, the two
  // payloads must agree byte-for-byte. Postgres canonicalises jsonb on
  // round-trip (spaces after commas, colons, etc.); both rows come
  // through the same canonicaliser so the comparison is on the
  // canonicalised form — the contract we care about is what a
  // downstream reader sees.
  for key in ["scope", "key", "value_type", "value", "reason"] {
    assert_eq!(
      rows[0].payload.get(key),
      rows[1].payload.get(key),
      "subset-parity: key `{key}` must be byte-identical across HTTP and shell paths (NOT5 gate 3, PRD §8.4 condition 3)",
    );
  }

  // Both rows must carry the additive fields introduced by v1-AD-c task 4
  // (closes GH #77 + #84). HTTP and shell paths now both emit
  // `previous_value` and `previous_from` per option A.
  //
  // Important: by this test's setup order (Write #1 = HTTP, Write #2 = shell),
  // the two rows' `previous_*` fields legitimately DIFFER:
  //   - HTTP row's previous_* reflects pre-Write-1 state (no prior row → null).
  //   - Shell row's previous_* reflects post-Write-1 state (Write #1's value).
  // That divergence is correct behavior, not a parity bug — each row carries
  // the previous-value the writer observed at action-time.
  assert!(
    rows[0].payload.get("previous_value").is_some(),
    "HTTP row must carry previous_value field (t4 extension, PRD §8.4 condition 3 amended)",
  );
  assert!(
    rows[0].payload.get("previous_from").is_some(),
    "HTTP row must carry previous_from field (t4 extension, PRD §8.4 condition 3 amended)",
  );
  assert!(
    rows[1].payload.get("previous_value").is_some(),
    "shell row must carry previous_value field (closes #84, option A)",
  );
  assert!(
    rows[1].payload.get("previous_from").is_some(),
    "shell row must carry previous_from field (closes #84, option A)",
  );
  // Shell's previous_value should observe Write #1's row (value 7), since
  // Write #1's HTTP handler wrote to governance_config too.
  assert_eq!(
    rows[1].payload.get("previous_value"),
    Some(&serde_json::json!(7)),
    "shell row's previous_value should equal Write #1's value (post-Write-1 state observed)",
  );

  Ok(())
}

/// Task 8 test 13 (advisor edit #2 / v1-AD-a retro): after full migrations
/// but with no seed row for `rule_set.active_version_id` AND no compile-time
/// default, `config::get_int_opt(Scope::Instance, "rule_set.active_version_id")`
/// returns `Ok(None)`. Proves that the accessor family correctly surfaces
/// absent keys without panicking (CachedValue::Absent path).
#[tokio::test(flavor = "multi_thread")]
async fn rule_set_active_version_absent_returns_none() -> lemmy_utils::error::LemmyResult<()> {
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_api::governance::config::{ConfigCache, Scope, get_int_opt};
  use lemmy_diesel_utils::connection::DbPool;

  let (_container, _context, db_url) = admin_config_fixtures::bootstrap().await?;
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  let mut pool: DbPool<'_> = (&mut async_conn).into();
  let mut cache = ConfigCache::new();

  let result = get_int_opt(
    &mut cache,
    &mut pool,
    Scope::Instance,
    "rule_set.active_version_id",
  )
  .await?;
  assert!(
    result.is_none(),
    "rule_set.active_version_id has no seed + no const → Ok(None)",
  );

  Ok(())
}

// -- v1-AD-c task 8 — 8 new e2e tests -------------------------------------
//
// Groups: A (4 rule-set CRUD), B (1 Scope parser), C (2 audit payload),
// D (1 case-open snapshot). See `.claude/PRPs/plans/v1-admin-dashboard-c.plan.md`
// §13 task 8 and §14.1 for the group matrix.

/// v1-AD-c task 8 test A1: moderator creates v1 (parent_id=None) then v2
/// (parent_id=v1); assert rule_set_version row count == 2,
/// governance_config row for `rule_set.active_version_id` flipped to v2,
/// exactly 2 `rule_set_version_created` governance_log entries.
#[tokio::test(flavor = "multi_thread")]
async fn admin_create_rule_set_happy_path() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_rule_sets::admin_create_rule_set;
  use lemmy_api_common::governance::AdminCreateRuleSet;
  use lemmy_db_schema::source::community::{Community, CommunityInsertForm};
  use lemmy_db_schema_file::schema::{governance_config, governance_log, rule_set_version};
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_happy".to_string(),
      "Rule-Set Happy".to_string(),
      "pk-rs-happy".to_string(),
    ),
  )
  .await?;
  let mod_view = admin_config_fixtures::seed_community_moderator(
    &context,
    instance.id,
    community.id,
    "rs_mod_hp",
  )
  .await?;

  let v1 = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "rules v1 — initial".to_string(),
      parent_id: None,
      reason: "initial rule-set".to_string(),
    }),
    context.clone(),
    mod_view.clone(),
  )
  .await?
  .into_inner();
  assert_eq!(v1.version, 1, "first version must be 1");

  let v2 = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "rules v2 — revised".to_string(),
      parent_id: Some(v1.rule_set_version_id),
      reason: "revise rules".to_string(),
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();
  assert_eq!(v2.version, 2, "second version must be 2");
  assert!(
    v2.rule_set_version_id > v1.rule_set_version_id,
    "id monotonic"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rsv_count: i64 = rule_set_version::table
    .filter(rule_set_version::community_id.eq(community.id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(rsv_count, 2, "two rule_set_version rows for this community");

  let community_scope = format!("community:{}", community.id.0);
  let active_rows: Vec<Option<i64>> = governance_config::table
    .filter(governance_config::scope.eq(&community_scope))
    .filter(governance_config::key.eq("rule_set.active_version_id"))
    .order(governance_config::valid_from.desc())
    .select(governance_config::value_int)
    .load(&mut conn)
    .await?;
  assert_eq!(
    active_rows.len(),
    2,
    "two active_version_id rows (one per create)"
  );
  assert_eq!(
    active_rows[0],
    Some(i64::from(v2.rule_set_version_id)),
    "latest active_version_id == v2 id",
  );

  let log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("rule_set_version_created"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(log_count, 2, "exactly two rule_set_version_created entries");

  Ok(())
}

/// v1-AD-c task 8 test A2: non-moderator non-admin caller attempts
/// `admin_create_rule_set` → `LemmyErrorType::NotAnAdmin` AND a
/// `admin_config_change_denied` governance_log entry with
/// `denial_reason = "community_moderator_required"`. Actor pseudonym is
/// populated even on denial (GDPR layer per plan §4.1).
#[tokio::test(flavor = "multi_thread")]
async fn admin_create_rule_set_non_moderator_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_rule_sets::admin_create_rule_set;
  use lemmy_api_common::governance::AdminCreateRuleSet;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::governance_log::GovernanceLog,
  };
  use lemmy_db_schema_file::schema::governance_log;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_denied".to_string(),
      "Rule-Set Denied".to_string(),
      "pk-rs-denied".to_string(),
    ),
  )
  .await?;
  // Seed a user that is NEITHER an admin NOR a moderator of this community.
  let (_, outsider_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "rs_outsider", false).await?;

  let result = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "sneaky rules".to_string(),
      parent_id: None,
      reason: "non-moderator probe".to_string(),
    }),
    context.clone(),
    outsider_view,
  )
  .await;
  assert!(result.is_err(), "non-moderator must be rejected");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let denial_rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(denial_rows.len(), 1, "exactly one denial entry");
  assert_eq!(
    denial_rows[0]
      .payload
      .get("denial_reason")
      .and_then(|v| v.as_str()),
    Some("community_moderator_required"),
    "denial_reason must be community_moderator_required",
  );
  assert!(
    denial_rows[0].actor_pseudonym.is_some(),
    "denied caller still gets a pseudonym (GDPR layer per plan §4.1)",
  );

  Ok(())
}

/// v1-AD-c task 8 test A3: the `rule_set_version` UNIQUE
/// `(community_id, version)` constraint triggers a
/// `diesel::result::DatabaseErrorKind::UniqueViolation` on a duplicate
/// insert at the same version. The handler's write path
/// (`process_create_rule_set` in `admin_rule_sets.rs`) catches exactly
/// this error kind and maps it to a retryable
/// `LemmyErrorType::Unknown("rule_set_version already exists ...")`.
///
/// This test exercises the constraint and mapping deterministically at
/// the DB layer:
///   1. Insert a `rule_set_version` row directly at version=1 (bypassing
///      the handler, so we can force a specific version).
///   2. Attempt a second direct insert at the same `(community_id, 1)`
///      pair.
///   3. Assert the exact Diesel error shape the handler pattern-matches
///      on (`DatabaseError(UniqueViolation, _)`) inside
///      `process_create_rule_set`.
///   4. Apply the same `Err` transform as the handler and assert the
///      `LemmyError` message.
///   5. Assert the pre-existing row survived and no second row leaked.
///
/// Rationale — an earlier revision of this test used `tokio::join!` to
/// race two concurrent `admin_create_rule_set` calls through the
/// handler. That is non-deterministic: `tokio::join!` gives no barrier
/// guarantee that both futures reach `lookup_latest_version` before
/// either insert commits. If one future wins, the other legitimately
/// observes version=1 already committed and computes version=2 — no
/// collision, assertions false-pass. The DB-layer direct-insert path
/// here is deterministic: the constraint fires on every run, the error
/// shape is observable, and the mapping is a pure function of that
/// error. The other path — concurrent handler invocation — would
/// require an `Arc<Barrier>` hook inside `process_create_rule_set`
/// (polluting production code for test determinism). CR PR #81 #5.
#[tokio::test(flavor = "multi_thread")]
async fn admin_create_rule_set_duplicate_version_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use diesel::{
    ExpressionMethods, QueryDsl,
    result::{DatabaseErrorKind, Error as DieselError},
  };
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::source::community::{Community, CommunityInsertForm};
  use lemmy_db_schema::source::governance::rule_set_version::RuleSetVersionInsertForm;
  use lemmy_db_schema_file::schema::rule_set_version;
  use lemmy_diesel_utils::traits::Crud;
  use lemmy_utils::error::LemmyErrorType;
  use sha2::{Digest, Sha256};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_dup".to_string(),
      "Rule-Set Duplicate".to_string(),
      "pk-rs-dup".to_string(),
    ),
  )
  .await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "rs_admin_dup", true).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let text_sha256_a = Sha256::digest(b"seeded A".as_slice()).to_vec();
  let text_sha256_b = Sha256::digest(b"would-be B".as_slice()).to_vec();

  // Step 1: seed the first row directly — this is the row the handler
  // would have written on a winning race.
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community.id,
      version: 1,
      parent_id: None,
      text_sha256: text_sha256_a,
      rule_text: "seeded A".to_string(),
      created_by: Some(admin_view.person.id),
    })
    .execute(&mut conn)
    .await?;

  // Step 2: attempt the duplicate — this is the insert the losing
  // handler would have issued before UniqueViolation rolls its tx back.
  let duplicate_insert = diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community.id,
      version: 1,
      parent_id: None,
      text_sha256: text_sha256_b,
      rule_text: "would-be B".to_string(),
      created_by: Some(admin_view.person.id),
    })
    .execute(&mut conn)
    .await;

  // Step 3: drive the duplicate through the DB layer (UNIQUE constraint
  // fires), then route the resulting DieselError through the same helper
  // `process_create_rule_set` uses — any change to
  // `map_rsv_unique_violation` in admin_rule_sets.rs immediately affects
  // this test's mapping assertion.
  match &duplicate_insert {
    Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
      // Expected — this is the branch the helper catches.
    }
    Err(other) => {
      panic!("expected UniqueViolation on duplicate (community_id, version); got {other:?}",)
    }
    Ok(_) => {
      panic!("UNIQUE(community_id, version) constraint did not fire — duplicate row committed",)
    }
  }

  // Step 4: route the DieselError through the real production mapping
  // helper and assert the LemmyError's inner `error_type` carries the
  // retry message verbatim. Note: we assert on `error_type` directly
  // rather than `format!("{mapped}")` because `LemmyError`'s `Display`
  // impl uses `strum::Display` on `LemmyErrorType`, which renders
  // `Unknown(String)` as just the bare variant name "Unknown" — the
  // wrapped message is only visible through pattern-matching on the
  // enum.
  let mapped: lemmy_utils::error::LemmyError = match duplicate_insert {
    Err(err) => lemmy_api::governance::admin_rule_sets::map_rsv_unique_violation(err),
    Ok(_) => unreachable!("UniqueViolation asserted at step above"),
  };
  match mapped.error_type {
    LemmyErrorType::Unknown(msg) => assert!(
      msg.contains("rule_set_version already exists"),
      "handler maps UniqueViolation to a retry-shaped Unknown error carrying the rule_set collision message; got {msg:?}",
    ),
    other => panic!("expected LemmyErrorType::Unknown with retry message; got {other:?}"),
  }

  // Step 5: the pre-existing row survived and no second row leaked.
  let rsv_count: i64 = rule_set_version::table
    .filter(rule_set_version::community_id.eq(community.id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    rsv_count, 1,
    "UNIQUE violation left the original row intact and rejected the duplicate",
  );

  Ok(())
}

/// v1-AD-c task 8 test A4: after three successful creates, GET
/// `/admin/rule-sets` returns all three versions ordered by `version`
/// DESC and `active_version_id` equals the most recent id.
#[tokio::test(flavor = "multi_thread")]
async fn admin_list_rule_sets_returns_versions_with_active_version_id()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_rule_sets::{admin_create_rule_set, admin_list_rule_sets};
  use lemmy_api_common::governance::{AdminCreateRuleSet, AdminListRuleSetsRequest};
  use lemmy_db_schema::source::community::{Community, CommunityInsertForm};
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_list".to_string(),
      "Rule-Set List".to_string(),
      "pk-rs-list".to_string(),
    ),
  )
  .await?;
  let mod_view = admin_config_fixtures::seed_community_moderator(
    &context,
    instance.id,
    community.id,
    "rs_mod_list",
  )
  .await?;

  let mut prev_id: Option<i32> = None;
  let mut ids: Vec<i32> = Vec::with_capacity(3);
  for i in 1..=3 {
    let resp = admin_create_rule_set(
      Json(AdminCreateRuleSet {
        community_id: community.id,
        rule_text: format!("rules v{i}"),
        parent_id: prev_id,
        reason: format!("revision {i}"),
      }),
      context.clone(),
      mod_view.clone(),
    )
    .await?
    .into_inner();
    ids.push(resp.rule_set_version_id);
    prev_id = Some(resp.rule_set_version_id);
  }

  let resp = admin_list_rule_sets(
    Query(AdminListRuleSetsRequest {
      community_id: community.id,
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();

  assert_eq!(resp.versions.len(), 3, "three versions returned");
  // Ordered by version DESC
  assert_eq!(resp.versions[0].version, 3);
  assert_eq!(resp.versions[1].version, 2);
  assert_eq!(resp.versions[2].version, 1);
  assert_eq!(
    resp.active_version_id,
    Some(ids[2]),
    "active_version_id == id of the most recently created version",
  );

  Ok(())
}

/// v1-AD-c task 8 test B1 (Issue #78): `Scope::parse_wire` rejects
/// `community:-1` and `community:0` with the typed
/// `ScopeParseError::NonPositiveCommunityId` variant carrying the
/// offending integer. This is a direct-call unit-style assertion; no DB
/// container needed, but kept in e2e.rs per plan §14's "integration-only"
/// discipline.
#[test]
fn scope_parse_wire_rejects_negative_community_id() {
  use lemmy_api::governance::config::{Scope, ScopeParseError};

  assert_eq!(
    Scope::parse_wire("community:-1"),
    Err(ScopeParseError::NonPositiveCommunityId(-1)),
    "negative community_id must be typed-rejected",
  );
  assert_eq!(
    Scope::parse_wire("community:0"),
    Err(ScopeParseError::NonPositiveCommunityId(0)),
    "zero community_id must be typed-rejected",
  );
}

/// v1-AD-c task 8 test C1 (Issue #77 write side): after a successful
/// `admin_set_config` bumping `jury.panel_size` 5 → 7, the
/// `admin_config_changed` governance_log payload carries
/// `previous_value: 5, previous_from: "instance"` (the migration seed
/// at `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:82`
/// inserts an instance-scoped row for this key with value=5, so the
/// effective provenance is `"instance"` — NOT `"default"`). A second
/// write 7 → 9 emits a row with `previous_value: 7, previous_from:
/// "instance"` (the 5→7 write appended another instance-scope row;
/// latest-wins reader picks it up). Both writes validate that the
/// `previous_value` + `previous_from` fields thread through from the
/// pre-tx read to the governance_log payload.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_persists_previous_value_and_from() -> lemmy_utils::error::LemmyResult<()>
{
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_prev", true).await?;

  // Write #1: 5 (seeded instance row) → 7.
  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "bump to 7".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  // Write #2: 7 (instance) → 9.
  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(9),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "bump to 9".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .order(governance_log::id.asc())
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(rows.len(), 2, "two admin_config_changed rows");

  // Write #1: previous is the seeded instance row (value=5, from="instance").
  assert_eq!(
    rows[0].payload.get("previous_value"),
    Some(&serde_json::json!(5)),
    "write #1 previous_value must equal the seeded instance row value 5",
  );
  assert_eq!(
    rows[0]
      .payload
      .get("previous_from")
      .and_then(|v| v.as_str()),
    Some("instance"),
    "write #1 previous_from must be `instance` (seeded row exists at instance scope)",
  );

  // Write #2: previous is the just-written 7 with from = "instance".
  assert_eq!(
    rows[1].payload.get("previous_value"),
    Some(&serde_json::json!(7)),
    "write #2 previous_value must equal the 5→7 write",
  );
  assert_eq!(
    rows[1]
      .payload
      .get("previous_from")
      .and_then(|v| v.as_str()),
    Some("instance"),
    "write #2 previous_from must be `instance` (latest-wins reads the 5→7 row)",
  );

  Ok(())
}

/// v1-AD-c task 8 test C2 (Issue #77 read side): after a 5→7 write,
/// `GET /admin/config/audit` returns an entry whose `previous_value` +
/// `previous_from` are hydrated from the payload written by task 4.
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_audit_hydrates_previous_value() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_config::{admin_get_config_audit, admin_set_config};
  use lemmy_api_common::governance::{AdminGetConfigAudit, AdminSetConfig};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_hyd", true).await?;

  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "hydration probe".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  let entries = admin_get_config_audit(
    Query(AdminGetConfigAudit {
      key: Some("jury.panel_size".to_string()),
      scope: None,
      actor_pseudonym: None,
      since: None,
      until: None,
      page: None,
      limit: None,
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(entries.len(), 1, "one audit entry for the bump");
  assert_eq!(
    entries[0].previous_value,
    Some(serde_json::json!(5)),
    "previous_value hydrated from payload — seeded instance value 5",
  );
  assert_eq!(
    entries[0].previous_from.as_deref(),
    Some("instance"),
    "previous_from hydrated from payload — `instance` (seed row exists)",
  );

  Ok(())
}

/// v1-AD-c task 8 test D1: community-target `create_report` opens a case
/// whose `applied_config_snapshot` contains exactly the set of
/// `requires_re_jury: true` keys from `CONFIG_KEY_METADATA` AND
/// `rule_set_version_id` equals the community's active rule-set version.
///
/// The expected key set is computed from the metadata at test time
/// (mirrors the `snapshot_keyset_matches_requires_re_jury_metadata`
/// parity test in `lemmy_api`). This is drift-proof: adding a new
/// `requires_re_jury: true` key to `CONFIG_KEY_METADATA` automatically
/// extends the expected set, so no manual list maintenance is required
/// here.
#[tokio::test(flavor = "multi_thread")]
async fn case_open_pins_applied_config_snapshot_and_rule_set_version_id()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_rule_sets::admin_create_rule_set;
  use lemmy_api::governance::config::CONFIG_KEY_METADATA;
  use lemmy_api_common::governance::AdminCreateRuleSet;
  use lemmy_api_common::governance::CreateGovernanceReport;
  use lemmy_api_crud::governance::create_report::create_report;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::moderation_case::ModerationCase,
  };
  use lemmy_db_schema_file::{enums::CaseTargetType, schema::moderation_case};
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_pin".to_string(),
      "Rule-Set Pin".to_string(),
      "pk-rs-pin".to_string(),
    ),
  )
  .await?;
  let mod_view = admin_config_fixtures::seed_community_moderator(
    &context,
    instance.id,
    community.id,
    "rs_mod_pin",
  )
  .await?;
  let (_, reporter_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "rs_reporter", false).await?;

  // Seed a community rule-set version — this flips
  // `rule_set.active_version_id` at Scope::Community(cid).
  let v1 = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "community rules v1".to_string(),
      parent_id: None,
      reason: "pin probe".to_string(),
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();

  // Open a case against the community (target_type=Community). The
  // create_report handler pins the snapshot at Scope::Community(cid)
  // because the reporter passed `community_id = Some(cid)`.
  let resp = create_report(
    Json(CreateGovernanceReport {
      community_id: Some(community.id),
      target_type: CaseTargetType::Community,
      target_id: community.id.0,
      reason_code: "test.pin".to_string(),
      description: Some("pin probe".to_string()),
    }),
    context.clone(),
    reporter_view,
  )
  .await?
  .into_inner();
  let case_id = resp.case_id.expect("case_id present on successful open");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select(ModerationCase::as_select())
    .first(&mut conn)
    .await?;

  assert_eq!(
    case.rule_set_version_id.map(|r| r.0),
    Some(v1.rule_set_version_id),
    "rule_set_version_id pinned to the active community version",
  );

  let snapshot = case
    .applied_config_snapshot
    .as_ref()
    .expect("applied_config_snapshot populated on case open");
  let snap_obj = snapshot
    .as_object()
    .expect("applied_config_snapshot is a JSON object");

  // Source-of-truth: every `requires_re_jury: true` entry in metadata
  // must appear in the case-open snapshot per ADR-010. Computing the
  // expected set from metadata here keeps the assertion drift-proof
  // (no hardcoded key list to maintain).
  let mut expected_keys: Vec<&str> = CONFIG_KEY_METADATA
    .iter()
    .filter(|m| m.requires_re_jury)
    .map(|m| m.key)
    .collect();
  expected_keys.sort_unstable();
  let mut snapshot_keys: Vec<&str> = snap_obj.keys().map(String::as_str).collect();
  snapshot_keys.sort_unstable();
  assert_eq!(
    snapshot_keys, expected_keys,
    "applied_config_snapshot keys must equal the requires_re_jury set in CONFIG_KEY_METADATA",
  );

  Ok(())
}

// ============================================================================
// v1-AD-d — admin dashboard aggregate + SSE audit stream
//
// Six tests covering the two new read-only handlers:
// - `admin_dashboard_returns_aggregate_for_admin`    — zero-row happy path
// - `admin_dashboard_forbidden_for_non_admin`        — capability gate
// - `admin_dashboard_aggregates_populated_data`      — data fidelity
// - `admin_audit_stream_forbidden_for_non_admin`     — capability gate
// - `admin_audit_stream_enforces_per_admin_cap`      — 409 on 2nd connection
// - `admin_audit_stream_emits_frame_on_config_change` — live SSE emission
//
// Dashboard tests invoke the handler directly (same pattern as the
// v1-AD-b `admin_get_config_full` test). SSE tests invoke the handler
// directly and drain the streaming body via `MessageBody::poll_next`
// without an in-process actix HTTP server — the live emission test
// exercises the full NOTIFY → filter → row hydration → frame-format
// path end-to-end.
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_returns_aggregate_for_admin() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_dashboard::admin_dashboard;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_dash_hp", true).await?;

  let before = chrono::Utc::now();
  let resp = admin_dashboard(context.clone(), admin_view)
    .await?
    .into_inner();
  let after = chrono::Utc::now();

  // Zero-row DB — every widget populates with defaults, none error.
  assert_eq!(
    resp.active_cases.total_active, 0,
    "no active cases on fresh DB"
  );
  assert!(
    resp.active_cases.by_status.is_empty()
      || resp.active_cases.by_status.values().sum::<i64>() == 0,
    "by_status empty or all zeros",
  );
  assert_eq!(resp.jury_queue.pending_accept, 0);
  assert_eq!(resp.jury_queue.accepted, 0);
  assert_eq!(resp.jury_queue.submitted, 0);
  assert_eq!(
    resp.recent_config_changes.len(),
    0,
    "no config-change events"
  );
  assert_eq!(resp.federation.active, 0);
  assert_eq!(resp.federation.expired, 0);
  assert_eq!(resp.federation.total, 0);
  assert_eq!(resp.rule_sets.communities_with_rule_sets, 0);
  assert_eq!(resp.rule_sets.total_versions, 0);
  assert_eq!(resp.rule_sets.per_community.len(), 0);

  // calculated_at within the request window (±5s slack either side).
  let slack = chrono::Duration::seconds(5);
  assert!(
    resp.calculated_at >= before - slack && resp.calculated_at <= after + slack,
    "calculated_at {} outside [{}, {}]",
    resp.calculated_at,
    before - slack,
    after + slack,
  );

  // Reputation widget is the instance-scope stub on a fresh DB.
  assert_eq!(
    resp.reputation.buckets.reporting_accuracy.len(),
    5,
    "bucket shape preserved (5 buckets per dimension)",
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_forbidden_for_non_admin() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "dash_nonadmin", false).await?;

  let result = admin_dashboard(context.clone(), user_view).await;
  // is_err() alone would also pass on a pre-admin-check DB error; the
  // specific-variant match anchors the test to the capability gate
  // (cr-18).
  let err = result.expect_err("non-admin must be rejected by is_admin()");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotAnAdmin),
    "expected NotAnAdmin, got {:?}",
    err.error_type,
  );

  // Dashboard is read-only — ADR-008 compliance: no governance_log
  // entry is emitted on capability-deny (unlike admin_set_config's
  // denial path).
  use diesel::{QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rows: Vec<GovernanceLog> = governance_log::table
    .select(GovernanceLog::as_select())
    .load(&mut conn)
    .await?;
  assert_eq!(
    rows.len(),
    0,
    "dashboard rejection must NOT emit a governance_log entry"
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_aggregates_populated_data() -> lemmy_utils::error::LemmyResult<()> {
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::{
      federation_attestation::FederationAttestationInsertForm,
      moderation_case::ModerationCaseInsertForm, rule_set_version::RuleSetVersionInsertForm,
    },
  };
  use lemmy_db_schema_file::enums::{AttestationType, CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::{federation_attestation, moderation_case, rule_set_version};
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_dash_seed", true).await?;

  // Seed a community so we can hang a rule_set_version off of it.
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "dash_comm".to_string(),
      "Dashboard Seed Community".to_string(),
      "dash-pubkey".to_string(),
    ),
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // 3 moderation_case rows across three statuses. Open + JurySelection
  // count toward `total_active` (2); Decided does not.
  for status in [
    CaseStatus::Open,
    CaseStatus::JurySelection,
    CaseStatus::Decided,
  ] {
    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::RemoteInstance,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: None,
      target_community_id: None,
      target_remote_url: Some(format!("https://example.invalid/dash/{status:?}")),
      reason_code: "dashboard_seed".to_string(),
      severity: CaseSeverity::Low,
      status,
      threshold_score: 1,
      ..Default::default()
    };
    diesel::insert_into(moderation_case::table)
      .values(&form)
      .execute(&mut conn)
      .await?;
  }

  // 1 active federation_attestation (valid_until in the future).
  let future = chrono::Utc::now() + chrono::Duration::days(30);
  diesel::insert_into(federation_attestation::table)
    .values(&FederationAttestationInsertForm {
      actor_url: "https://test.invalid/u/seed-actor".to_string(),
      subject_url: "https://test.invalid/u/seed-subject".to_string(),
      attestation_type: AttestationType::TrustedReporter,
      valid_until: Some(future),
      signature: "seed-sig".to_string(),
      ..Default::default()
    })
    .execute(&mut conn)
    .await?;

  // 1 rule_set_version on the seeded community (version 1, no parent).
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community.id,
      version: 1,
      parent_id: None,
      text_sha256: vec![0u8; 32],
      rule_text: "seed rule text".to_string(),
      created_by: None,
    })
    .execute(&mut conn)
    .await?;

  // Invoke the dashboard handler and assert aggregates.
  let resp = admin_dashboard(context.clone(), admin_view)
    .await?
    .into_inner();

  // Each seed inserts exactly one case per status (three total). Assert
  // exact values so a regression that double-counts or drops a status
  // bucket surfaces, and so a missing key (None) is distinguished from
  // a zero count (cr-25).
  assert_eq!(
    resp.active_cases.by_status.get("Open").copied(),
    Some(1),
    "Open case count should be exactly 1 (one seed)",
  );
  assert_eq!(
    resp.active_cases.by_status.get("JurySelection").copied(),
    Some(1),
    "JurySelection case count should be exactly 1 (one seed)",
  );
  assert_eq!(
    resp.active_cases.by_status.get("Decided").copied(),
    Some(1),
    "Decided case count should be exactly 1 (one seed)",
  );
  assert_eq!(
    resp.active_cases.total_active, 2,
    "total_active excludes Decided (and Closed/EmergencyRemove)",
  );

  assert_eq!(resp.federation.active, 1, "one active attestation");
  assert_eq!(resp.federation.expired, 0);
  assert_eq!(resp.federation.total, 1);

  assert_eq!(
    resp.rule_sets.communities_with_rule_sets, 1,
    "one community has a rule_set_version",
  );
  assert_eq!(resp.rule_sets.total_versions, 1);
  // per_community includes one entry; active_version_id is None because
  // no governance_config row was seeded for rule_set.active_version_id.
  assert_eq!(resp.rule_sets.per_community.len(), 1);
  assert_eq!(resp.rule_sets.per_community[0].community_id, community.id);
  assert_eq!(resp.rule_sets.per_community[0].active_version_id, None);

  // recent_config_changes remains empty — no admin_config_changed rows
  // were inserted by any of the seeds above (they go through direct
  // table inserts, not the governance_log::append path).
  assert_eq!(resp.recent_config_changes.len(), 0);

  // Silence unused variable warnings on the fields we checked via other
  // branches.
  let _ = instance;
  Ok(())
}

/// cr-24 regression lock: the batched `active_version_id` lookup in
/// `rule_sets_summary` must return the correct per-community value, and
/// must fall back to the `instance`-scoped row when no community-scoped
/// row exists. Seeds two communities: the first has its own
/// community-scoped `rule_set.active_version_id`; the second has none,
/// so the cascade falls back to the instance-scoped row. Asserts each
/// community gets its own value from a single batched SELECT.
#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_per_community_active_version_cascade()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::rule_set_version::RuleSetVersionInsertForm,
  };
  use lemmy_db_schema_file::schema::rule_set_version;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "dash_cascade_admin", true).await?;

  let community_a = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "cascade_a".to_string(),
      "Cascade A".to_string(),
      "cascade-a-pk".to_string(),
    ),
  )
  .await?;
  let community_b = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "cascade_b".to_string(),
      "Cascade B".to_string(),
      "cascade-b-pk".to_string(),
    ),
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // Each community needs a rule_set_version row so it shows up in
  // `per_community`.
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community_a.id,
      version: 1,
      parent_id: None,
      text_sha256: vec![0x11u8; 32],
      rule_text: "community A rules".to_string(),
      created_by: None,
    })
    .execute(&mut conn)
    .await?;
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community_b.id,
      version: 1,
      parent_id: None,
      text_sha256: vec![0x22u8; 32],
      rule_text: "community B rules".to_string(),
      created_by: None,
    })
    .execute(&mut conn)
    .await?;

  // Seed instance-scoped fallback: version 999. Community B falls back
  // to this because it has no community-scoped row.
  diesel::sql_query(
    "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
     VALUES ('instance', 'rule_set.active_version_id', 'int', 999, now())",
  )
  .execute(&mut conn)
  .await?;

  // Seed community A: community-scoped override to version 42. This
  // must win over the instance-scoped row per `get_int_opt`'s cascade.
  diesel::sql_query(format!(
    "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
     VALUES ('community:{}', 'rule_set.active_version_id', 'int', 42, now())",
    community_a.id.0,
  ))
  .execute(&mut conn)
  .await?;

  let resp = admin_dashboard(context.clone(), admin_view)
    .await?
    .into_inner();

  // Locate each community's row in the response — per_community is
  // ORDER BY community_id in the handler, so community A sorts before B
  // if community_a.id.0 < community_b.id.0 (which is always true since
  // they were inserted in that order under a serial PK).
  let per_a = resp
    .rule_sets
    .per_community
    .iter()
    .find(|r| r.community_id == community_a.id)
    .ok_or_else(|| anyhow::anyhow!("community A not in per_community"))?;
  let per_b = resp
    .rule_sets
    .per_community
    .iter()
    .find(|r| r.community_id == community_b.id)
    .ok_or_else(|| anyhow::anyhow!("community B not in per_community"))?;

  assert_eq!(
    per_a.active_version_id,
    Some(42),
    "community A has a community-scoped override; batched query should \
     surface it (community:{} → 42), not the instance fallback",
    community_a.id.0,
  );
  assert_eq!(
    per_b.active_version_id,
    Some(999),
    "community B has no community-scoped row; batched query must fall \
     back to the instance-scoped row (value 999)",
  );

  Ok(())
}

/// cr-22 regression lock: `recent_config_changes` MUST exclude rows whose
/// `signature` is NULL. Those rows represent a half-written append (the
/// INSERT succeeded but the follow-up signature UPDATE in
/// `governance_log::append` failed). Surfacing them in the dashboard
/// would display unsigned audit entries that carry no verifiable hash
/// chain position — a misleading artifact for an admin reviewing
/// governance activity. Seeds one signed row and one unsigned row
/// directly via diesel (bypassing the append helper) and asserts only
/// the signed one is returned.
#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_recent_config_changes_excludes_unsigned_rows()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_db_schema::source::governance::governance_log::{
    ENTRY_KIND_ADMIN_CONFIG_CHANGED, GovernanceLogInsertForm,
  };
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "dash_unsigned_filter", true).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // Row A — "unsigned": INSERT only, signature stays NULL (simulates a
  // failed follow-up UPDATE in governance_log::append).
  let unsigned_id: i64 = diesel::insert_into(governance_log::table)
    .values(&GovernanceLogInsertForm {
      entry_kind: ENTRY_KIND_ADMIN_CONFIG_CHANGED.to_string(),
      payload: serde_json::json!({ "marker": "unsigned_row_should_not_leak" }),
      actor_pseudonym: Some("test-unsigned".to_string()),
    })
    .returning(governance_log::id)
    .get_result(&mut conn)
    .await?;

  // Row B — "signed": INSERT then flip signature NULL → NOT NULL. The
  // signature-gate trigger permits exactly one such transition per row.
  let signed_id: i64 = diesel::insert_into(governance_log::table)
    .values(&GovernanceLogInsertForm {
      entry_kind: ENTRY_KIND_ADMIN_CONFIG_CHANGED.to_string(),
      payload: serde_json::json!({ "marker": "signed_row_should_appear" }),
      actor_pseudonym: Some("test-signed".to_string()),
    })
    .returning(governance_log::id)
    .get_result(&mut conn)
    .await?;
  diesel::update(governance_log::table.find(signed_id))
    .set(governance_log::signature.eq(Some(vec![0xAAu8; 64])))
    .execute(&mut conn)
    .await?;

  let resp = admin_dashboard(context.clone(), admin_view)
    .await?
    .into_inner();

  assert_eq!(
    resp.recent_config_changes.len(),
    1,
    "dashboard must include the signed row and exclude the unsigned row; \
     got {:?} entries",
    resp.recent_config_changes.len(),
  );
  // Sanity: unsigned row's marker must not appear in any projected entry.
  for entry in &resp.recent_config_changes {
    let serialized = serde_json::to_string(entry)?;
    assert!(
      !serialized.contains("unsigned_row_should_not_leak"),
      "unsigned governance_log row leaked into recent_config_changes: {entry:?}",
    );
  }

  let _ = (unsigned_id, signed_id);
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_stream_forbidden_for_non_admin() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "sse_nonadmin", false).await?;

  let result = admin_audit_stream(context.clone(), user_view).await;
  // is_err() alone would also pass on a tokio_postgres::connect failure
  // before the admin check; the specific-variant match anchors the test
  // to the capability gate (cr-18).
  let err = result.expect_err("non-admin must be rejected by is_admin()");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotAnAdmin),
    "expected NotAnAdmin, got {:?}",
    err.error_type,
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_stream_enforces_per_admin_cap() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::http::StatusCode;
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  // Unique username avoids PersonId collision with other SSE tests in
  // the same process (module-static HashSet leaks across tests per plan
  // §14 GOTCHA).
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "sse_cap_admin", true).await?;

  // First connection succeeds — returns 200 with text/event-stream body.
  let resp1 = admin_audit_stream(context.clone(), admin_view.clone()).await?;
  assert_eq!(
    resp1.status(),
    StatusCode::OK,
    "first connection returns 200"
  );
  assert_eq!(
    resp1
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or_default(),
    "text/event-stream",
    "Content-Type is text/event-stream",
  );

  // Second concurrent connection — same admin — returns 409 Conflict.
  let resp2 = admin_audit_stream(context.clone(), admin_view.clone()).await?;
  assert_eq!(
    resp2.status(),
    StatusCode::CONFLICT,
    "second concurrent connection from same admin returns 409",
  );

  // Drop the first response body so the SseGuard drops and releases
  // the per-admin slot. Then a third connection should succeed within
  // a short window — proves SseGuard::Drop ran.
  drop(resp1);
  // The Drop impl spawns an async cleanup task; yield to let it run.
  for _ in 0..20 {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let resp3 = admin_audit_stream(context.clone(), admin_view.clone()).await?;
    if resp3.status() == StatusCode::OK {
      drop(resp3);
      return Ok(());
    }
    drop(resp3);
  }
  panic!(
    "third connection did not succeed after first was dropped — SseGuard::Drop may not be releasing the cap entry"
  );
}

/// Drive `admin_audit_stream`'s streaming body end-to-end: open the SSE
/// connection, trigger an `admin_config_changed` write via
/// `admin_set_config`, and assert a correctly-framed `admin_config_changed`
/// SSE event arrives with the expected JSON payload.
///
/// Exercises the full live-stream path the other two SSE tests skip:
///   - the `kind == ENTRY_KIND_ADMIN_CONFIG_CHANGED` filter branch
///   - the `entry_id` → `governance_log` row hydration via `get_conn`
///   - `project_to_audit_entry` round-trip through the streaming body
///   - `event: X\ndata: Y\n\n` frame format per HTML5 §9.2.4
#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_stream_emits_frame_on_config_change() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::{body::MessageBody, http::StatusCode, web::Json};
  use lemmy_api::governance::{
    admin_audit_stream::admin_audit_stream, admin_config::admin_set_config,
  };
  use lemmy_api_common::governance::AdminSetConfig;
  use std::{future::poll_fn, pin::Pin, time::Duration as StdDuration};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  // Unique username avoids PersonId collision with the other SSE tests
  // (the module-static cap HashSet persists across tests in the same
  // process).
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "sse_emit_admin", true).await?;

  // Open the SSE stream. The handler returns a 200 with a streaming
  // body; we drain frames below via `MessageBody::poll_next`.
  let resp = admin_audit_stream(context.clone(), admin_view.clone()).await?;
  assert_eq!(resp.status(), StatusCode::OK, "stream opens with 200");
  assert_eq!(
    resp
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or_default(),
    "text/event-stream",
    "Content-Type is text/event-stream",
  );

  // `into_body()` yields the `BoxBody` driving the stream. We poll it
  // frame-by-frame with a timeout. Each SSE message arrives as a single
  // `Bytes` chunk (the handler emits `yield Ok(Bytes::from(...))` per
  // frame).
  let mut body = resp.into_body();

  // First frame is the initial `retry: 10000\n\n` the handler emits
  // before entering its select loop. Pull it out so the subsequent reads
  // see a clean stream.
  let retry_frame = tokio::time::timeout(
    StdDuration::from_secs(5),
    poll_fn(|cx| Pin::new(&mut body).poll_next(cx)),
  )
  .await
  .map_err(|_e| anyhow::anyhow!("timed out waiting for initial retry frame"))?
  .ok_or_else(|| anyhow::anyhow!("body ended before retry frame"))?
  .map_err(|e| anyhow::anyhow!("body error on retry frame: {e}"))?;
  let retry_str =
    std::str::from_utf8(&retry_frame).map_err(|e| anyhow::anyhow!("retry frame not utf-8: {e}"))?;
  assert_eq!(
    retry_str, "retry: 10000\n\n",
    "initial frame is the SSE retry field (HTML5 §9.2.5, not a custom event)",
  );

  // Trigger an `admin_config_changed` write. The governance_log INSERT
  // fires the `governance_events` NOTIFY, which the handler's dedicated
  // tokio-postgres LISTEN connection observes and forwards into the
  // stream.
  let _set_resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(9),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "v1-AD-d SSE emission test".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  // Drain frames (skipping `: keepalive\n\n` comments) until we observe
  // an `event: admin_config_changed` frame or time out. The handler
  // heartbeat interval is 15s, so under a 10s budget we expect zero
  // keepalive frames — but the loop is defensive against scheduling
  // jitter and future interval changes.
  let config_frame = tokio::time::timeout(StdDuration::from_secs(10), async {
    loop {
      let chunk = poll_fn(|cx| Pin::new(&mut body).poll_next(cx))
        .await
        .ok_or_else(|| anyhow::anyhow!("body ended before config-change frame"))?
        .map_err(|e| anyhow::anyhow!("body error: {e}"))?;
      let s = std::str::from_utf8(&chunk)
        .map_err(|e| anyhow::anyhow!("frame not utf-8: {e}"))?
        .to_owned();
      if s.starts_with(": keepalive") {
        continue;
      }
      return Ok::<String, anyhow::Error>(s);
    }
  })
  .await
  .map_err(|_e| anyhow::anyhow!("timed out waiting for admin_config_changed frame"))??;

  // SSE framing: `event: admin_config_changed\ndata: {json}\n\n`.
  assert!(
    config_frame.ends_with("\n\n"),
    "SSE frame terminates with two newlines (HTML5 §9.2.4); got: {config_frame:?}",
  );
  let mut lines = config_frame.trim_end_matches("\n\n").split('\n');
  let event_line = lines
    .next()
    .ok_or_else(|| anyhow::anyhow!("frame has no event line: {config_frame:?}"))?;
  let data_line = lines
    .next()
    .ok_or_else(|| anyhow::anyhow!("frame has no data line: {config_frame:?}"))?;
  assert!(
    lines.next().is_none(),
    "frame has exactly event + data lines; got extra: {config_frame:?}",
  );
  assert_eq!(
    event_line, "event: admin_config_changed",
    "event line names the entry kind",
  );
  let data_json = data_line
    .strip_prefix("data: ")
    .ok_or_else(|| anyhow::anyhow!("data line missing 'data: ' prefix: {data_line:?}"))?;
  let payload: serde_json::Value = serde_json::from_str(data_json)
    .map_err(|e| anyhow::anyhow!("data payload not valid JSON ({e}): {data_json:?}"))?;

  // The payload is the full `AdminConfigAuditEntry` shape produced by
  // `project_to_audit_entry`.
  assert_eq!(
    payload["entry_kind"].as_str(),
    Some("admin_config_changed"),
    "payload.entry_kind mirrors the event type",
  );
  assert_eq!(
    payload["scope"].as_str(),
    Some("instance"),
    "payload.scope reflects the admin_set_config call",
  );
  assert_eq!(
    payload["key"].as_str(),
    Some("jury.panel_size"),
    "payload.key reflects the admin_set_config call",
  );
  assert_eq!(
    payload["value_type"].as_str(),
    Some("int"),
    "payload.value_type reflects the admin_set_config call",
  );
  assert_eq!(
    payload["new_value"],
    serde_json::json!(9),
    "payload.new_value reflects the written value",
  );
  assert!(
    payload["id"].as_i64().unwrap_or_default() > 0,
    "payload.id is the governance_log row id",
  );
  assert!(
    payload["created_at"].as_str().is_some(),
    "payload.created_at populated",
  );

  // Drop the stream so the SseGuard releases the per-admin cap entry
  // before the container tear-down runs.
  drop(body);
  Ok(())
}

// ============================================================================
// v1-JM-b Task 7 — severity/status cascade fixture + tests
// ============================================================================
//
// Six tests per plan §14 Task 7:
//
// 1. regular/minor  → panel 5 / quorum 3 / threshold 3
// 2. regular/severe → panel 7 / quorum 5 / threshold 6
// 3. founder/severe → panel 9 / quorum 7 / threshold 7
// 4. `jury_assignment.selected_under_constraints` JSONB shape
// 5. `severity_tier_frozen` governance_log row emitted
// 6. `config::get_int_cascade` walks founder.severe → severe → bare → const
//
// The fixture mirrors `admin_config_fixtures::bootstrap` at e2e.rs:~4620.
// admin_assign_jury's small-pool-fallback (Phase 4 `legacy_select_eligible_jurors`
// shape) seats the panel when no reputation_snapshot rows exist, so the
// fixture skips snapshot seeding — the cascade-resolved panel_size still
// applies upstream of the fallback and the snapshot fields still land.

