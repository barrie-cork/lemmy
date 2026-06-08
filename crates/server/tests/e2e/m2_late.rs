mod m2_late_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::governance_fixtures;
  use actix_web::web::{Data, Json};
  use diesel::{Connection as _, ExpressionMethods, PgConnection, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    sanction_publisher,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment,
    AdminAssignJury,
    CreateGovernanceReport,
    SubmitJuryVote,
  };
  use lemmy_api_crud::governance::create_report::create_report;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    post::{Post, PostInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseTargetType, JuryDecision},
    schema::{governance_log, sanction_event as sanction_event_dsl},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;
  use tokio::io::{AsyncReadExt, AsyncWriteExt};
  use tokio::net::TcpListener;
  use tokio::sync::oneshot;

  /// Minimal mock HTTP server: bind a local TCP listener, spawn a task that
  /// accepts one connection, reads the raw HTTP request body, sends 200 OK,
  /// and returns the body via a `oneshot` channel. Returns the bound URL.
  ///
  /// This avoids adding a `httpmock` / `wiremock` dependency (none are in the
  /// workspace). The single-connection contract matches T8's use: one quorum
  /// trip → one POST to the subscriber.
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

  /// Golden-path test for B-publish sanction propagation (m2-late-1 §10).
  ///
  /// 1. Start a local mock HTTP subscriber server.
  /// 2. Seed the subscriber URL via `seed_sanction_subscriber` (T6 path).
  /// 3. Drive a 3-of-5 quorum vote (AdvisoryLabel → Label → RestrictReach).
  /// 4. Let `enqueue_sanction_event` spawn deliver to the mock subscriber.
  /// 5. Assert: subscriber received a POST with a valid `SanctionEventPayload`
  ///    whose `subject_actor_pseudonym` is a pseudonym (ADR-015), and whose
  ///    `sanction_kind` matches the mapped RestrictReach kind.
  #[tokio::test(flavor = "current_thread")]
  pub async fn sanction_event_delivered_to_subscriber() -> LemmyResult<()> {
    // -- 1. Env guards + Postgres ------------------------------------------
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    let _g_init = crate::EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = crate::EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    let _g_secret = crate::EnvVarGuard::set("BRIDGE_CALLBACK_SECRET", "test-secret-m2late");

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = crate::EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

    // -- 2. Mock subscriber server + subscriber seed -----------------------
    let (mock_url, body_rx) = spawn_mock_subscriber().await?;

    let pool: ActualDbPool = build_db_pool_for_tests();
    {
      let mut pool_ref: DbPool<'_> = (&pool).into();
      sanction_publisher::seed_sanction_subscriber(&mock_url, &mut pool_ref).await?;
    }

    // -- 3. Build LemmyContext + federation Data ---------------------------
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
    let federation_config = activitypub_federation::config::FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data((**context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await?;
    let federation_context = federation_config.to_request_data();

    // -- 4. Seed persons + community ---------------------------------------
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

    async fn seed_person_local(
      ctx: &LemmyContext,
      instance_id: lemmy_db_schema_file::InstanceId,
      name: &str,
      is_admin: bool,
    ) -> LemmyResult<PersonId> {
      let person_form = PersonInsertForm::test_form(instance_id, name);
      let person = Person::create(&mut ctx.pool(), &person_form).await?;
      let lu_form = LocalUserInsertForm {
        admin: Some(is_admin),
        accepted_application: Some(true),
        ..LocalUserInsertForm::test_form(person.id)
      };
      LocalUser::create(&mut ctx.pool(), &lu_form, vec![]).await?;
      Ok(person.id)
    }

    let admin_id = seed_person_local(&context, instance.id, "m2late_admin", true).await?;
    let reporter_id = seed_person_local(&context, instance.id, "m2late_reporter", false).await?;
    let target_id = seed_person_local(&context, instance.id, "m2late_target", false).await?;
    let _juror_a_id = seed_person_local(&context, instance.id, "m2late_juror_a", false).await?;
    let _juror_b_id = seed_person_local(&context, instance.id, "m2late_juror_b", false).await?;
    let _juror_c_id = seed_person_local(&context, instance.id, "m2late_juror_c", false).await?;
    let _juror_d_id = seed_person_local(&context, instance.id, "m2late_juror_d", false).await?;
    let _juror_e_id = seed_person_local(&context, instance.id, "m2late_juror_e", false).await?;

    // Seed reputation snapshots so jury eligibility is satisfied.
    {
      use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshotInsertForm;
      use lemmy_db_schema_file::schema::reputation_snapshot;
      let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
      for person_id in [
        _juror_a_id,
        _juror_b_id,
        _juror_c_id,
        _juror_d_id,
        _juror_e_id,
      ] {
        let form = ReputationSnapshotInsertForm {
          person_id,
          community_id: None,
          reporting_accuracy: 100,
          jury_reliability: 100,
          participation_consistency: 100,
          endorsement_strength: 100,
          jury_eligible: true,
          trusted_reporter: false,
          ..Default::default()
        };
        diesel::insert_into(reputation_snapshot::table)
          .values(&form)
          .execute(&mut async_conn)
          .await?;
      }
    }

    let community_form = CommunityInsertForm::new(
      instance.id,
      "m2late_community".to_string(),
      "m2late_community".to_string(),
      "http://test.invalid/m2late_community".to_string(),
    );
    let community = Community::create(&mut context.pool(), &community_form).await?;

    // Create a post for the report to target.
    let post = Post::create(
      &mut context.pool(),
      &PostInsertForm::new("m2late_test_post".into(), target_id, community.id),
    )
    .await?;

    // -- 5. Create report (opens a case against target_person) -----------
    // CreateGovernanceReport: target_type=Person, target_id=target_id.0.
    // Using a Person-target so enqueue_sanction_event can resolve the pseudonym.
    let reporter_view = LocalUserView::read_person(&mut context.pool(), reporter_id).await?;
    create_report(
      Json(CreateGovernanceReport {
        community_id: Some(community.id),
        target_type: CaseTargetType::Person,
        target_id: target_id.0,
        reason_code: "m2late_e2e_spam".to_string(),
        description: None,
      }),
      context.clone(),
      reporter_view,
    )
    .await?;

    // Suppress unused-variable warning for post (needed to satisfy FK).
    let _ = post;

    // -- 6. Assign jury --------------------------------------------------
    let case_id = {
      let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
      let id: i32 = lemmy_db_schema_file::schema::moderation_case::table
        .select(lemmy_db_schema_file::schema::moderation_case::id)
        .order(lemmy_db_schema_file::schema::moderation_case::id.desc())
        .first(&mut async_conn)
        .await?;
      lemmy_db_schema::newtypes::ModerationCaseId(id)
    };

    let admin_view = LocalUserView::read_person(&mut context.pool(), admin_id).await?;
    let assign_resp = admin_assign_jury(
      Json(AdminAssignJury { case_id }),
      context.clone(),
      admin_view,
    )
    .await?
    .into_inner();

    // -- 7. Jurors accept assignments ------------------------------------
    for person_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *person_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }

    // -- 8. Drive quorum votes (AdvisoryLabel → Label → RestrictReach) --
    // Stop at quorum (3 of 5): votes after quorum re-query the active sanction
    // and each fires another enqueue_sanction_event, inflating sanction_event count.
    let mut decided = false;
    for person_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *person_id).await?;
      let resp = submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision: JuryDecision::AdvisoryLabel,
          rationale: Some("m2late e2e quorum vote".to_string()),
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?
      .into_inner();
      if resp.case_decided {
        decided = true;
        break; // stop at quorum — further votes would fire additional enqueue_sanction_event spawns
      }
    }
    assert!(decided, "quorum should have been reached after 3 votes");

    // -- 9. Wait for the spawned enqueue_sanction_event delivery ----------
    // The spawn fires inside submit_jury_vote on the 3rd (quorum) vote.
    // Allow up to 3 seconds for the async task to POST to our mock server.
    let body = tokio::time::timeout(std::time::Duration::from_secs(3), body_rx)
      .await
      .expect("timed out waiting for mock subscriber POST")
      .expect("mock subscriber body channel dropped before send");

    // -- 10. Assert payload shape (ADR-015 pseudonymity + kind) ----------
    let payload: serde_json::Value =
      serde_json::from_slice(&body).expect("mock subscriber body is not valid JSON");

    // subject_actor_pseudonym must be present and non-empty (ADR-015).
    let subject = payload
      .get("subject_actor_pseudonym")
      .and_then(|v| v.as_str())
      .expect("payload missing subject_actor_pseudonym");
    assert!(
      !subject.is_empty(),
      "subject_actor_pseudonym must be non-empty (ADR-015)"
    );
    // ADR-015: pseudonym must NOT be the person's display name.
    assert_ne!(
      subject, "m2late_target",
      "subject_actor_pseudonym must be a pseudonym, not the person name (ADR-015)"
    );

    // sanction_kind must be "restrict_reach"
    // (AdvisoryLabel → SanctionAction::Label → SanctionKind::RestrictReach, serde snake_case).
    let sanction_kind = payload
      .get("sanction_kind")
      .and_then(|v| v.as_str())
      .expect("payload missing sanction_kind");
    assert_eq!(
      sanction_kind, "restrict_reach",
      "AdvisoryLabel → Label → RestrictReach (serde snake_case)"
    );

    // governance_log_entry_hash must be a non-empty hex string.
    let hash = payload
      .get("governance_log_entry_hash")
      .and_then(|v| v.as_str())
      .expect("payload missing governance_log_entry_hash");
    assert!(!hash.is_empty(), "governance_log_entry_hash must be non-empty");

    // Allow the spawned enqueue_sanction_event task to complete its DB writes
    // (sanction_event insert + governance_log append) after the HTTP POST.
    // body_rx fires when the mock sends 200 OK, but those DB writes happen
    // after the POST returns — a brief yield is sufficient.
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    // -- 11. DB assertions — sanction_event row + governance_log ---------
    {
      let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
      let count: i64 = sanction_event_dsl::table
        .count()
        .get_result(&mut async_conn)
        .await?;
      assert_eq!(count, 1, "exactly one sanction_event row written");

      let gl_count: i64 = governance_log::table
        .filter(governance_log::entry_kind.eq("sanction_published"))
        .count()
        .get_result(&mut async_conn)
        .await?;
      assert_eq!(gl_count, 1, "one sanction_published governance_log entry");
    }

    Ok(())
  }
}
