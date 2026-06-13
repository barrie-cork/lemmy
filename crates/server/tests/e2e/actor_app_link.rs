mod actor_app_link_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BRIDGE_LINK_CLAIM_URL`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::governance_fixtures;
  use actix_web::web::{Data, Json, Query};
  use diesel::{Connection as _, ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use ed25519_dalek::{Signer, SigningKey};
  use lemmy_api::governance::actor_app_link::{link_actor, link_confirm, revoke_link};
  use lemmy_api_common::governance::{LinkActorRequest, LinkConfirmRequest};
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::schema::{actor_app_link, governance_log};
  use lemmy_utils::error::LemmyResult;
  use tokio::io::{AsyncReadExt, AsyncWriteExt};
  use tokio::net::TcpListener;
  use tokio::sync::oneshot;

  const SIGNING_SEED_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";

  // APP test keypair — fixed seed for deterministic test countersignatures.
  // The test owns both halves of the dual signature.
  const APP_SIGNING_SEED_HEX: &str =
    "0101010101010101010101010101010101010101010101010101010101010101";

  /// Minimal mock HTTP server: bind a local TCP listener, spawn a task that
  /// accepts one connection, reads the raw HTTP request body, sends 200 OK,
  /// and returns the body via a `oneshot` channel. Returns the bound URL.
  ///
  /// Verbatim mirror of the `spawn_mock_subscriber` in `m2_late.rs` — same
  /// single-connection contract: one POST to the bridge callback → one body.
  async fn spawn_mock_subscriber() -> LemmyResult<(String, oneshot::Receiver<Vec<u8>>)> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let url = format!("http://127.0.0.1:{}", addr.port());
    let (tx, rx) = oneshot::channel::<Vec<u8>>();
    tokio::spawn(async move {
      let Ok((mut stream, _)) = listener.accept().await else {
        return;
      };
      // Read until the blank line separating headers from body.
      let mut raw = Vec::new();
      let mut buf = [0u8; 4096];
      loop {
        let n = stream.read(&mut buf).await.unwrap_or(0);
        if n == 0 {
          break;
        }
        raw.extend_from_slice(&buf[..n]);
        if raw.windows(4).any(|w| w == b"\r\n\r\n") {
          break;
        }
      }
      // Read any remaining body bytes declared in Content-Length.
      let content_length = std::str::from_utf8(&raw)
        .unwrap_or("")
        .lines()
        .find(|l| l.to_ascii_lowercase().starts_with("content-length:"))
        .and_then(|l| l.splitn(2, ':').nth(1)?.trim().parse::<usize>().ok())
        .unwrap_or(0);
      let header_end = raw
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .map(|p| p + 4)
        .unwrap_or(raw.len());
      let already_read = raw.len().saturating_sub(header_end);
      let remaining = content_length.saturating_sub(already_read);
      if remaining > 0 {
        let mut body_tail = vec![0u8; remaining];
        let _ = stream.read_exact(&mut body_tail).await;
        raw.extend_from_slice(&body_tail);
      }
      let body = raw[header_end..].to_vec();
      // Respond 200 OK so reqwest treats the POST as successful.
      let _ = stream
        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}")
        .await;
      let _ = tx.send(body);
    });
    Ok((url, rx))
  }

  /// Golden-path test for B-actor portable-ID link flow (m2-late-b-actor §10).
  ///
  /// 1. Boot postgres, apply schema, build context.
  /// 2. Spawn mock bridge callback server + set `BRIDGE_LINK_CLAIM_URL`.
  /// 3. Seed a user; call `link_actor` — POSTs a `LinkClaimPayload` to mock.
  /// 4. Parse payload; assert pseudonym (ADR-015) + non-empty `brehon_signature`.
  /// 5. Countersign with app keypair; call `link_confirm` with valid bearer.
  /// 6. Assert one `actor_app_link` row + one `actor_app_link_created` log entry.
  #[tokio::test(flavor = "current_thread")]
  pub async fn link_actor_creates_dual_signed_claim_and_logs() -> LemmyResult<()> {
    // -- 1. Env guards + Postgres ------------------------------------------
    let _g_init = crate::EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = crate::EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    let _g_secret = crate::EnvVarGuard::set("BRIDGE_CALLBACK_SECRET", "test-secret-actor-link");

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = crate::EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = diesel::PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    // -- 2. Mock bridge callback server + pool ----------------------------
    let (mock_url, body_rx) = spawn_mock_subscriber().await?;
    let _g_bridge_url = crate::EnvVarGuard::set("BRIDGE_LINK_CLAIM_URL", &mock_url);

    use lemmy_api_utils::{context::LemmyContext, request::client_builder};
    use lemmy_db_schema::source::secret::Secret;
    use lemmy_diesel_utils::connection::{ActualDbPool, build_db_pool_for_tests};
    use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
    use reqwest_middleware::ClientBuilder;

    let pool: ActualDbPool = build_db_pool_for_tests();
    let client = client_builder(&SETTINGS).build()?;
    let middleware_client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };
    let rate_limit = RateLimit::with_debug_config();
    let context = Data::new(LemmyContext::create(
      pool,
      middleware_client.clone(),
      middleware_client,
      secret,
      rate_limit,
    ));

    // -- 3. Seed user -------------------------------------------------------
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (_, local_user_view) =
      governance_fixtures::seed_user(&context, instance.id, "actor_link_user", false).await?;

    // -- 4. Call link_actor — fires mock POST --------------------------------
    link_actor(
      Query(LinkActorRequest {
        app_id: "matrix".into(),
        app_local_id: "@alice:test.invalid".into(),
      }),
      context.clone(),
      local_user_view,
    )
    .await?;

    // -- 5. Wait for mock bridge POST (up to 3s) ----------------------------
    let body = tokio::time::timeout(std::time::Duration::from_secs(3), body_rx)
      .await
      .expect("timed out waiting for mock bridge POST")
      .expect("mock bridge body channel dropped before send");

    // -- 6. Assert payload shape (ADR-015 pseudonymity + signature) ----------
    let payload: serde_json::Value =
      serde_json::from_slice(&body).expect("mock bridge body is not valid JSON");

    let brehon_actor_id = payload
      .get("brehon_actor_id")
      .and_then(|v| v.as_str())
      .expect("payload missing brehon_actor_id");
    assert!(!brehon_actor_id.is_empty(), "brehon_actor_id must be non-empty");
    assert_ne!(
      brehon_actor_id, "actor_link_user",
      "brehon_actor_id must be a pseudonym, not the person name (ADR-015)"
    );

    let brehon_signature = payload
      .get("brehon_signature")
      .and_then(|v| v.as_array())
      .expect("payload missing brehon_signature array");
    assert!(!brehon_signature.is_empty(), "brehon_signature must be non-empty");

    let nonce = payload
      .get("nonce")
      .and_then(|v| v.as_str())
      .expect("payload missing nonce")
      .to_string();
    let app_id = payload
      .get("app_id")
      .and_then(|v| v.as_str())
      .expect("payload missing app_id")
      .to_string();
    let app_local_id = payload
      .get("app_local_id")
      .and_then(|v| v.as_str())
      .expect("payload missing app_local_id")
      .to_string();
    let brehon_actor_id = brehon_actor_id.to_string();

    // -- 7. Set app pubkey env var + countersign ----------------------------
    let app_signing_key = SigningKey::from_bytes(
      &hex::decode(APP_SIGNING_SEED_HEX)
        .unwrap()
        .try_into()
        .unwrap(),
    );
    let app_pubkey_hex = hex::encode(app_signing_key.verifying_key().to_bytes());
    let _g_app_pubkey = crate::EnvVarGuard::set("BRIDGE_LINK_APP_PUBKEY", &app_pubkey_hex);

    let signed_bytes = format!("{}\n{}", nonce, app_local_id);
    let app_signature: Vec<u8> = app_signing_key
      .sign(signed_bytes.as_bytes())
      .to_bytes()
      .to_vec();

    // -- 8. Call link_confirm with valid bearer + valid countersig ----------
    let req = actix_web::test::TestRequest::default()
      .insert_header(("Authorization", "Bearer test-secret-actor-link"))
      .to_http_request();

    link_confirm(
      Json(LinkConfirmRequest {
        nonce,
        app_signature,
        app_local_id,
        brehon_actor_id: brehon_actor_id.clone(),
        app_id,
      }),
      context.clone(),
      req,
    )
    .await?;

    // Allow transaction to settle
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // -- 9. DB assertions: one actor_app_link row + actor_app_link_created --
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    let link_count: i64 = actor_app_link::table
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(link_count, 1, "exactly one actor_app_link row");

    let gl_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("actor_app_link_created"))
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(gl_count, 1, "one actor_app_link_created governance_log entry");

    Ok(())
  }

  /// Dual-signature defence: a garbage app countersignature must cause
  /// `link_confirm` to reject with an error and write ZERO `actor_app_link` rows.
  #[tokio::test(flavor = "current_thread")]
  pub async fn link_confirm_rejects_bad_app_signature() -> LemmyResult<()> {
    // -- 1. Env guards + Postgres ------------------------------------------
    let _g_init = crate::EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = crate::EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    let _g_secret = crate::EnvVarGuard::set("BRIDGE_CALLBACK_SECRET", "test-secret-actor-link");

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = crate::EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = diesel::PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    // -- 2. Mock bridge callback server + pool ----------------------------
    let (mock_url, body_rx) = spawn_mock_subscriber().await?;
    let _g_bridge_url = crate::EnvVarGuard::set("BRIDGE_LINK_CLAIM_URL", &mock_url);

    use lemmy_api_utils::{context::LemmyContext, request::client_builder};
    use lemmy_db_schema::source::secret::Secret;
    use lemmy_diesel_utils::connection::{ActualDbPool, build_db_pool_for_tests};
    use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
    use reqwest_middleware::ClientBuilder;

    let pool: ActualDbPool = build_db_pool_for_tests();
    let client = client_builder(&SETTINGS).build()?;
    let middleware_client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };
    let rate_limit = RateLimit::with_debug_config();
    let context = Data::new(LemmyContext::create(
      pool,
      middleware_client.clone(),
      middleware_client,
      secret,
      rate_limit,
    ));

    // -- 3. Seed user -------------------------------------------------------
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (_, local_user_view) =
      governance_fixtures::seed_user(&context, instance.id, "bad_sig_user", false).await?;

    // -- 4. Call link_actor to get a valid claim ----------------------------
    link_actor(
      Query(LinkActorRequest {
        app_id: "matrix".into(),
        app_local_id: "@alice:test.invalid".into(),
      }),
      context.clone(),
      local_user_view,
    )
    .await?;

    let body = tokio::time::timeout(std::time::Duration::from_secs(3), body_rx)
      .await
      .expect("timed out waiting for mock bridge POST")
      .expect("mock bridge body channel dropped");

    let payload: serde_json::Value =
      serde_json::from_slice(&body).expect("not valid JSON");

    let brehon_actor_id = payload
      .get("brehon_actor_id")
      .and_then(|v| v.as_str())
      .expect("missing brehon_actor_id")
      .to_string();
    let nonce = payload
      .get("nonce")
      .and_then(|v| v.as_str())
      .expect("missing nonce")
      .to_string();
    let app_id = payload
      .get("app_id")
      .and_then(|v| v.as_str())
      .expect("missing app_id")
      .to_string();
    let app_local_id = payload
      .get("app_local_id")
      .and_then(|v| v.as_str())
      .expect("missing app_local_id")
      .to_string();

    // -- 5. Set app pubkey env var (needed before InvalidQuery check) -------
    let app_signing_key = SigningKey::from_bytes(
      &hex::decode(APP_SIGNING_SEED_HEX)
        .unwrap()
        .try_into()
        .unwrap(),
    );
    let app_pubkey_hex = hex::encode(app_signing_key.verifying_key().to_bytes());
    let _g_app_pubkey = crate::EnvVarGuard::set("BRIDGE_LINK_APP_PUBKEY", &app_pubkey_hex);

    // -- 6. Call link_confirm with a garbage app_signature (64 zero bytes) --
    let req = actix_web::test::TestRequest::default()
      .insert_header(("Authorization", "Bearer test-secret-actor-link"))
      .to_http_request();

    let result = link_confirm(
      Json(LinkConfirmRequest {
        nonce,
        app_signature: vec![0u8; 64], // garbage — will fail verify_strict
        app_local_id,
        brehon_actor_id,
        app_id,
      }),
      context.clone(),
      req,
    )
    .await;

    // -- 7. Assert error + zero DB rows ------------------------------------
    assert!(result.is_err(), "link_confirm with bad app_signature must return Err");

    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    let link_count: i64 = actor_app_link::table
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(link_count, 0, "zero actor_app_link rows after rejected countersig");

    Ok(())
  }

  /// Service-principal auth: a request without the correct bearer token must
  /// cause `link_confirm` to return a 401-equivalent error with no DB writes.
  #[tokio::test(flavor = "current_thread")]
  pub async fn link_confirm_rejects_bad_bearer() -> LemmyResult<()> {
    // -- 1. Env guards + Postgres ------------------------------------------
    let _g_init = crate::EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = crate::EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    let _g_secret = crate::EnvVarGuard::set("BRIDGE_CALLBACK_SECRET", "test-secret-actor-link");

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = crate::EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = diesel::PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    use lemmy_api_utils::{context::LemmyContext, request::client_builder};
    use lemmy_db_schema::source::secret::Secret;
    use lemmy_diesel_utils::connection::{ActualDbPool, build_db_pool_for_tests};
    use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
    use reqwest_middleware::ClientBuilder;

    let pool: ActualDbPool = build_db_pool_for_tests();
    let client = client_builder(&SETTINGS).build()?;
    let middleware_client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };
    let rate_limit = RateLimit::with_debug_config();
    let context = Data::new(LemmyContext::create(
      pool,
      middleware_client.clone(),
      middleware_client,
      secret,
      rate_limit,
    ));

    // -- 2. Call link_confirm with wrong bearer (checked before any DB work) --
    // Bearer auth is the very first gate in link_confirm; dummy data is fine.
    let req = actix_web::test::TestRequest::default()
      .insert_header(("Authorization", "Bearer WRONG-SECRET"))
      .to_http_request();

    let result = link_confirm(
      Json(LinkConfirmRequest {
        nonce: "dummy-nonce".into(),
        app_signature: vec![0u8; 64],
        app_local_id: "@dummy:test.invalid".into(),
        brehon_actor_id: "00000000-0000-0000-0000-000000000000".into(),
        app_id: "matrix".into(),
      }),
      context.clone(),
      req,
    )
    .await;

    // -- 3. Assert error (401 equivalent) + zero DB rows ------------------
    assert!(
      result.is_err(),
      "link_confirm with wrong bearer must return Err"
    );

    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    let link_count: i64 = actor_app_link::table
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(link_count, 0, "zero actor_app_link rows after bearer rejection");

    Ok(())
  }

  /// Prospective revoke: revoke sets `revoked_at`, does NOT rewrite historical
  /// chain entries (ADR-008), and appends one new `actor_app_link_revoked` log.
  #[tokio::test(flavor = "current_thread")]
  pub async fn revoke_link_is_prospective_and_logged() -> LemmyResult<()> {
    // -- 1. Env guards + Postgres ------------------------------------------
    let _g_init = crate::EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = crate::EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    let _g_secret = crate::EnvVarGuard::set("BRIDGE_CALLBACK_SECRET", "test-secret-actor-link");

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = crate::EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = diesel::PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    // -- 2. Mock bridge callback server + pool ----------------------------
    let (mock_url, body_rx) = spawn_mock_subscriber().await?;
    let _g_bridge_url = crate::EnvVarGuard::set("BRIDGE_LINK_CLAIM_URL", &mock_url);

    use lemmy_api_utils::{context::LemmyContext, request::client_builder};
    use lemmy_db_schema::source::secret::Secret;
    use lemmy_db_schema_file::schema::actor_app_link as aal;
    use lemmy_diesel_utils::connection::{ActualDbPool, build_db_pool_for_tests};
    use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
    use reqwest_middleware::ClientBuilder;

    let pool: ActualDbPool = build_db_pool_for_tests();
    let client = client_builder(&SETTINGS).build()?;
    let middleware_client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };
    let rate_limit = RateLimit::with_debug_config();
    let context = Data::new(LemmyContext::create(
      pool,
      middleware_client.clone(),
      middleware_client,
      secret,
      rate_limit,
    ));

    // -- 3. Seed user -------------------------------------------------------
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (_, local_user_view) =
      governance_fixtures::seed_user(&context, instance.id, "revoke_test_user", false).await?;

    // -- 4. Create a valid link (drive test-1 flow) -------------------------
    link_actor(
      Query(LinkActorRequest {
        app_id: "matrix".into(),
        app_local_id: "@alice:test.invalid".into(),
      }),
      context.clone(),
      local_user_view.clone(),
    )
    .await?;

    let body = tokio::time::timeout(std::time::Duration::from_secs(3), body_rx)
      .await
      .expect("timed out waiting for mock bridge POST")
      .expect("mock bridge body channel dropped");

    let payload: serde_json::Value =
      serde_json::from_slice(&body).expect("not valid JSON");

    let brehon_actor_id = payload
      .get("brehon_actor_id")
      .and_then(|v| v.as_str())
      .expect("missing brehon_actor_id")
      .to_string();
    let nonce = payload
      .get("nonce")
      .and_then(|v| v.as_str())
      .expect("missing nonce")
      .to_string();
    let app_id = payload
      .get("app_id")
      .and_then(|v| v.as_str())
      .expect("missing app_id")
      .to_string();
    let app_local_id = payload
      .get("app_local_id")
      .and_then(|v| v.as_str())
      .expect("missing app_local_id")
      .to_string();

    let app_signing_key = SigningKey::from_bytes(
      &hex::decode(APP_SIGNING_SEED_HEX)
        .unwrap()
        .try_into()
        .unwrap(),
    );
    let app_pubkey_hex = hex::encode(app_signing_key.verifying_key().to_bytes());
    let _g_app_pubkey = crate::EnvVarGuard::set("BRIDGE_LINK_APP_PUBKEY", &app_pubkey_hex);

    let signed_bytes = format!("{}\n{}", nonce, app_local_id);
    let app_signature: Vec<u8> = app_signing_key
      .sign(signed_bytes.as_bytes())
      .to_bytes()
      .to_vec();

    let req = actix_web::test::TestRequest::default()
      .insert_header(("Authorization", "Bearer test-secret-actor-link"))
      .to_http_request();

    link_confirm(
      Json(LinkConfirmRequest {
        nonce,
        app_signature,
        app_local_id: app_local_id.clone(),
        brehon_actor_id: brehon_actor_id.clone(),
        app_id: app_id.clone(),
      }),
      context.clone(),
      req,
    )
    .await?;

    // -- 5. Revoke the link (JWT-authed, no bearer needed) ------------------
    revoke_link(
      Json(LinkActorRequest {
        app_id,
        app_local_id,
      }),
      context.clone(),
      local_user_view,
    )
    .await?;

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    // -- 6. DB assertions ---------------------------------------------------
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    // revoked_at IS NOT NULL
    let revoked_count: i64 = aal::table
      .filter(aal::revoked_at.is_not_null())
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(revoked_count, 1, "actor_app_link row must have revoked_at set");

    // actor_app_link_created entry still present (chain not rewritten — ADR-008)
    let created_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("actor_app_link_created"))
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(
      created_count,
      1,
      "actor_app_link_created governance_log entry must be retained after revoke (ADR-008)"
    );

    // New actor_app_link_revoked entry appended
    let revoked_gl_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("actor_app_link_revoked"))
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(revoked_gl_count, 1, "one actor_app_link_revoked governance_log entry");

    Ok(())
  }

  /// ADR-015 lock: the `brehon_actor_id` POSTed to the bridge callback must be
  /// a UUID pseudonym, never the raw `person.name` of the seeded user.
  #[tokio::test(flavor = "current_thread")]
  pub async fn pseudonym_not_raw_identity_in_payload() -> LemmyResult<()> {
    // -- 1. Env guards + Postgres ------------------------------------------
    let _g_init = crate::EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = crate::EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    let _g_secret = crate::EnvVarGuard::set("BRIDGE_CALLBACK_SECRET", "test-secret-actor-link");

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = crate::EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = diesel::PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    // -- 2. Mock bridge callback server + pool ----------------------------
    let (mock_url, body_rx) = spawn_mock_subscriber().await?;
    let _g_bridge_url = crate::EnvVarGuard::set("BRIDGE_LINK_CLAIM_URL", &mock_url);

    use lemmy_api_utils::{context::LemmyContext, request::client_builder};
    use lemmy_db_schema::source::secret::Secret;
    use lemmy_diesel_utils::connection::{ActualDbPool, build_db_pool_for_tests};
    use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
    use reqwest_middleware::ClientBuilder;

    let pool: ActualDbPool = build_db_pool_for_tests();
    let client = client_builder(&SETTINGS).build()?;
    let middleware_client = ClientBuilder::new(client).build();
    let secret = Secret {
      id: 0,
      jwt_secret: String::new().into(),
    };
    let rate_limit = RateLimit::with_debug_config();
    let context = Data::new(LemmyContext::create(
      pool,
      middleware_client.clone(),
      middleware_client,
      secret,
      rate_limit,
    ));

    // -- 3. Seed user with a recognisable name ------------------------------
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (_, local_user_view) =
      governance_fixtures::seed_user(&context, instance.id, "link_pseudonym_test_user", false)
        .await?;

    // -- 4. Call link_actor; capture payload --------------------------------
    link_actor(
      Query(LinkActorRequest {
        app_id: "matrix".into(),
        app_local_id: "@pseudonym_test:test.invalid".into(),
      }),
      context.clone(),
      local_user_view,
    )
    .await?;

    let body = tokio::time::timeout(std::time::Duration::from_secs(3), body_rx)
      .await
      .expect("timed out waiting for mock bridge POST")
      .expect("mock bridge body channel dropped");

    // -- 5. ADR-015 assertions: brehon_actor_id must be UUID, not username --
    let payload: serde_json::Value =
      serde_json::from_slice(&body).expect("not valid JSON");

    let brehon_actor_id = payload
      .get("brehon_actor_id")
      .and_then(|v| v.as_str())
      .expect("payload missing brehon_actor_id");

    assert_ne!(
      brehon_actor_id, "link_pseudonym_test_user",
      "brehon_actor_id must be a pseudonym, not the person name (ADR-015)"
    );

    // Must parse as a valid UUID string
    uuid::Uuid::parse_str(brehon_actor_id).expect("brehon_actor_id must be a valid UUID string");

    Ok(())
  }
}
