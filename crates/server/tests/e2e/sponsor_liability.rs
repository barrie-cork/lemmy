mod v1_sl_b_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::governance_fixtures;
  use actix_web::web::Json;
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api_common::governance::RevokeEndorsement;
  use lemmy_api_crud::governance::revoke_endorsement::revoke_endorsement;
  use lemmy_db_schema::{
    newtypes::{CommunityId, EndorsementId, ModerationCaseId},
    source::governance::{
      endorsement::EndorsementInsertForm, moderation_case::ModerationCaseInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseStatusTier, CaseTargetType, SeverityTier},
    schema::{endorsement, governance_log, moderation_case, reputation_snapshot, surety},
  };
  use lemmy_utils::error::LemmyResult;
  use serde_json::Value;

  /// Seed an active (non-revoked) endorsement from `sponsor` to `sponsee`.
  /// Mirror of §10.3 helper shape.
  async fn seed_endorsement_active(
    conn: &mut AsyncPgConnection,
    sponsor: PersonId,
    sponsee: PersonId,
  ) -> LemmyResult<EndorsementId> {
    let form = EndorsementInsertForm {
      from_person_id: sponsor,
      to_person_id: sponsee,
      community_id: None,
    };
    let id: EndorsementId = diesel::insert_into(endorsement::table)
      .values(&form)
      .returning(endorsement::id)
      .get_result(conn)
      .await?;
    Ok(id)
  }

  /// Seed a `SponsorLiabilityPending` moderation_case for `sponsee`.
  /// `community` may be None (instance scope). `grace_hours` controls
  /// the grace_expires_at offset from now (positive = future deadline).
  async fn seed_pending_case(
    conn: &mut AsyncPgConnection,
    sponsee: PersonId,
    community: Option<CommunityId>,
    grace_hours: i64,
  ) -> LemmyResult<ModerationCaseId> {
    let form = ModerationCaseInsertForm {
      community_id: community,
      creator_id: None,
      target_type: CaseTargetType::Person,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: Some(sponsee),
      target_community_id: None,
      target_remote_url: None,
      reason_code: "sponsor_liability_pending_test_seed".to_string(),
      severity: CaseSeverity::Medium,
      status: CaseStatus::SponsorLiabilityPending,
      threshold_score: 0,
      applied_config_snapshot: None,
      rule_set_version_id: None,
      severity_tier: Some(SeverityTier::Minor),
      status_tier: Some(CaseStatusTier::Regular),
      panel_size_snapshot: None,
      quorum_snapshot: None,
      threshold_count_snapshot: None,
      appeal_window_expires_at: None,
      winning_decision: None,
      grace_expires_at: Some(Utc::now() + Duration::hours(grace_hours)),
      liability_escape_reason: None,
    };
    let id: ModerationCaseId = diesel::insert_into(moderation_case::table)
      .values(&form)
      .returning(moderation_case::id)
      .get_result(conn)
      .await?;
    Ok(id)
  }

  /// Read endorsement row's revoked_at by id.
  async fn read_endorsement_revoked_at(
    conn: &mut AsyncPgConnection,
    eid: EndorsementId,
  ) -> LemmyResult<Option<DateTime<Utc>>> {
    let v: Option<DateTime<Utc>> = endorsement::table
      .filter(endorsement::id.eq(eid))
      .select(endorsement::revoked_at)
      .first(conn)
      .await?;
    Ok(v)
  }

  /// Read surety row's revoked_at for a (sponsor, sponsee, community) triple.
  async fn read_surety_revoked_at(
    conn: &mut AsyncPgConnection,
    sponsor: PersonId,
    sponsee: PersonId,
    community: Option<CommunityId>,
  ) -> LemmyResult<Option<DateTime<Utc>>> {
    let mut q = surety::table
      .filter(surety::sponsor_id.eq(sponsor))
      .filter(surety::sponsored_id.eq(sponsee))
      .into_boxed();
    q = match community {
      Some(c) => q.filter(surety::community_id.eq(c)),
      None => q.filter(surety::community_id.is_null()),
    };
    let v: Option<DateTime<Utc>> = q.select(surety::revoked_at).first(conn).await?;
    Ok(v)
  }

  /// Read the moderation_case row.
  async fn read_case_status_and_escape(
    conn: &mut AsyncPgConnection,
    case_id: ModerationCaseId,
  ) -> LemmyResult<(CaseStatus, Option<Value>)> {
    let row: (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(conn)
      .await?;
    Ok(row)
  }

  /// Count governance_log entries of a given kind.
  async fn count_log_entries(conn: &mut AsyncPgConnection, kind: &str) -> LemmyResult<i64> {
    let n: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq(kind))
      .count()
      .get_result(conn)
      .await?;
    Ok(n)
  }

  /// Read the most recent payload of a given kind.
  async fn read_log_payload(
    conn: &mut AsyncPgConnection,
    kind: &str,
  ) -> LemmyResult<Option<Value>> {
    let payloads: Vec<Value> = governance_log::table
      .filter(governance_log::entry_kind.eq(kind))
      .order(governance_log::id.desc())
      .select(governance_log::payload)
      .limit(1)
      .load(conn)
      .await?;
    Ok(payloads.into_iter().next())
  }

  /// Read reputation_snapshot.calculated_at for (person, community=None).
  async fn read_snapshot_calculated_at(
    conn: &mut AsyncPgConnection,
    person: PersonId,
  ) -> LemmyResult<Option<DateTime<Utc>>> {
    let rows: Vec<DateTime<Utc>> = reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person))
      .filter(reputation_snapshot::community_id.is_null())
      .select(reputation_snapshot::calculated_at)
      .load(conn)
      .await?;
    Ok(rows.into_iter().next())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 1 (plan §13 Task 4): self-revoke success path.
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_self_succeeds_updates_surety_and_recomputes_snapshots()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsor, sponsor_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_slb1", false).await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb1", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_id = seed_endorsement_active(&mut conn, sponsor, sponsee).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;

    // Pre-call: revoked_at IS NULL on both rows.
    assert!(
      read_endorsement_revoked_at(&mut conn, endorsement_id)
        .await?
        .is_none(),
      "pre-call: endorsement.revoked_at IS NULL",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None)
        .await?
        .is_none(),
      "pre-call: surety.revoked_at IS NULL",
    );

    let test_start = Utc::now();
    let resp = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id,
        reason: "self-revoke test".to_string(),
      }),
      context.clone(),
      sponsor_view,
    )
    .await?
    .into_inner();

    assert_eq!(
      resp.endorsement_id, endorsement_id,
      "response endorsement_id matches"
    );
    assert!(
      (Utc::now() - resp.revoked_at).num_seconds() < 5,
      "response.revoked_at recent (within 5s)",
    );
    assert!(
      resp.liability_chain_severed_for_cases.is_empty(),
      "no pending case → severed empty",
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    assert!(
      read_endorsement_revoked_at(&mut conn, endorsement_id)
        .await?
        .is_some(),
      "post-call: endorsement.revoked_at populated",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None)
        .await?
        .is_some(),
      "post-call: surety.revoked_at populated",
    );

    // Snapshots: both sponsor + sponsee recomputed (calculated_at >= test_start).
    // test_start is captured before revoke_endorsement (above), so the recomputation
    // triggered by that call produces a calculated_at that should be >= test_start.
    // Rare flake: Postgres `now()` at trigger fire time may lag Rust `Utc::now()`
    // by a few hundred milliseconds if the PG and host clocks diverge at sub-ms
    // resolution. The logic is correct; this is a clock-sync edge case.
    let sponsor_calc = read_snapshot_calculated_at(&mut conn, sponsor)
      .await?
      .expect("sponsor snapshot exists post-recompute");
    assert!(
      sponsor_calc >= test_start,
      "sponsor snapshot recomputed: calculated_at {:?} >= test_start {:?}",
      sponsor_calc,
      test_start,
    );
    let sponsee_calc = read_snapshot_calculated_at(&mut conn, sponsee)
      .await?
      .expect("sponsee snapshot exists post-recompute");
    assert!(
      sponsee_calc >= test_start,
      "sponsee snapshot recomputed: calculated_at {:?} >= test_start {:?}",
      sponsee_calc,
      test_start,
    );

    // governance_log: 1 endorsement_revoked, 0 sponsor_liability_escaped.
    assert_eq!(
      count_log_entries(&mut conn, "endorsement_revoked").await?,
      1,
      "exactly one endorsement_revoked entry",
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "no sponsor_liability_escaped entry (no pending case)",
    );

    let payload = read_log_payload(&mut conn, "endorsement_revoked")
      .await?
      .expect("endorsement_revoked payload exists");
    assert!(
      payload["revoker_pseudonym"].is_string(),
      "revoker_pseudonym is a string"
    );
    assert!(
      payload["target_pseudonym"].is_string(),
      "target_pseudonym is a string"
    );
    assert_eq!(
      payload["reason"],
      Value::String("self-revoke test".to_string()),
      "reason matches",
    );
    assert!(
      payload.get("rate_limit_bypassed").is_none(),
      "rate_limit_bypassed field absent under threshold",
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 2 (plan §13 Task 5): admin-revoke success path under threshold.
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_admin_succeeds_under_threshold() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsor, _sponsor_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_slb2", false).await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb2", false).await?;
    let (_admin, admin_view) =
      governance_fixtures::seed_user(&context, instance.id, "admin_slb2", true).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_id = seed_endorsement_active(&mut conn, sponsor, sponsee).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;

    let resp = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id,
        reason: "admin policy intervention".to_string(),
      }),
      context.clone(),
      admin_view,
    )
    .await?
    .into_inner();

    assert_eq!(resp.endorsement_id, endorsement_id);
    assert!(
      (Utc::now() - resp.revoked_at).num_seconds() < 5,
      "revoked_at recent",
    );
    assert!(resp.liability_chain_severed_for_cases.is_empty());

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    assert!(
      read_endorsement_revoked_at(&mut conn, endorsement_id)
        .await?
        .is_some()
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None)
        .await?
        .is_some()
    );

    // governance_log entry: caller is admin, target is sponsee. rate_limit_bypassed
    // ABSENT because admin was under threshold (DQ #141 separation).
    let payload = read_log_payload(&mut conn, "endorsement_revoked")
      .await?
      .expect("payload exists");
    assert!(
      payload.get("rate_limit_bypassed").is_none(),
      "rate_limit_bypassed absent for admin under threshold",
    );
    assert_eq!(
      payload["reason"],
      Value::String("admin policy intervention".to_string()),
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 3 (plan §13 Task 6): re-revoke idempotency — second call returns
  // existing revoked_at, emits no new log, severs no chain.
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_re_revoke_returns_existing_revoked_at_no_log_no_severance()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsor, sponsor_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_slb3", false).await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb3", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_id = seed_endorsement_active(&mut conn, sponsor, sponsee).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;

    // First call.
    let resp1 = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id,
        reason: "first attempt".to_string(),
      }),
      context.clone(),
      sponsor_view.clone(),
    )
    .await?
    .into_inner();
    let t1 = resp1.revoked_at;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let count_after_first = count_log_entries(&mut conn, "endorsement_revoked").await?;
    assert_eq!(count_after_first, 1, "one log entry after first call");

    // Second call (same endorsement, different reason).
    let resp2 = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id,
        reason: "second attempt".to_string(),
      }),
      context.clone(),
      sponsor_view,
    )
    .await?
    .into_inner();

    assert_eq!(resp2.endorsement_id, endorsement_id);
    // Compare at microsecond precision: the first call returns the in-memory
    // `Utc::now()` (nanosecond precision), the second call returns the value
    // round-tripped through Postgres `timestamptz` (truncated to microseconds).
    // Same instant, different precision — strict `==` would fail spuriously.
    assert_eq!(
      resp2.revoked_at.timestamp_micros(),
      t1.timestamp_micros(),
      "second response.revoked_at == first (idempotency, micros precision)",
    );
    assert!(
      resp2.liability_chain_severed_for_cases.is_empty(),
      "second-call severance empty",
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let count_after_second = count_log_entries(&mut conn, "endorsement_revoked").await?;
    assert_eq!(
      count_after_second, count_after_first,
      "no new log entry on re-revoke",
    );
    let final_revoked_at = read_endorsement_revoked_at(&mut conn, endorsement_id).await?;
    // Same precision rationale as the resp2.revoked_at assertion above:
    // `final_revoked_at` is DB-round-tripped (micros); `t1` is in-memory (nanos).
    assert_eq!(
      final_revoked_at.map(|t| t.timestamp_micros()),
      Some(t1.timestamp_micros()),
      "endorsement.revoked_at unchanged (micros precision)",
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 4 (plan §13 Task 7): non-sponsor non-admin caller rejected with
  // NotFound (PRD §5.2 — do NOT leak existence as Unauthorized).
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_non_sponsor_non_admin_rejects_with_not_found() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsor, _sponsor_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_slb4", false).await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb4", false).await?;
    let (_third, third_view) =
      governance_fixtures::seed_user(&context, instance.id, "third_slb4", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_id = seed_endorsement_active(&mut conn, sponsor, sponsee).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;

    let logs_before = count_log_entries(&mut conn, "endorsement_revoked").await?;

    let result = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id,
        reason: "impersonation attempt".to_string(),
      }),
      context.clone(),
      third_view,
    )
    .await;

    let err = result.expect_err("third-party caller must be rejected");
    assert!(
      matches!(err.error_type, lemmy_utils::error::LemmyErrorType::NotFound),
      "expected NotFound, got {:?}",
      err.error_type,
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    assert!(
      read_endorsement_revoked_at(&mut conn, endorsement_id)
        .await?
        .is_none(),
      "endorsement.revoked_at unchanged on rejection",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None)
        .await?
        .is_none(),
      "surety.revoked_at unchanged on rejection",
    );
    assert_eq!(
      count_log_entries(&mut conn, "endorsement_revoked").await?,
      logs_before,
      "no log entry on rejection",
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 5 (plan §13 Task 8): empty / whitespace-only reason rejected.
  // Three sub-cases (single test fn) per DQ #139 resolution.
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_empty_reason_rejects() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsor, sponsor_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_slb5", false).await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb5", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_id = seed_endorsement_active(&mut conn, sponsor, sponsee).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;

    for bad_reason in ["", "   ", "\t\n  "] {
      let result = revoke_endorsement(
        Json(RevokeEndorsement {
          endorsement_id,
          reason: bad_reason.to_string(),
        }),
        context.clone(),
        sponsor_view.clone(),
      )
      .await;
      let err = result.expect_err("empty/whitespace reason must reject");
      let matched = matches!(
        &err.error_type,
        lemmy_utils::error::LemmyErrorType::Unknown(msg)
          if msg == "revoke-endorsement reason required",
      );
      assert!(
        matched,
        "expected Unknown(\"revoke-endorsement reason required\") for reason {:?}, got {:?}",
        bad_reason, err.error_type,
      );
    }

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    assert!(
      read_endorsement_revoked_at(&mut conn, endorsement_id)
        .await?
        .is_none(),
      "no successful revocation across all three rejection sub-cases",
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 6 (plan §13 Task 9): rate-limit enforces unless admin bypasses.
  // Combined positive (regular caller at threshold rejected) + negative
  // (admin caller at threshold succeeds with rate_limit_bypassed: true).
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_rate_limit_enforces_unless_admin_bypasses() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (regular_caller, regular_view) =
      governance_fixtures::seed_user(&context, instance.id, "regular_slb6", false).await?;
    let (admin_caller, admin_view) =
      governance_fixtures::seed_user(&context, instance.id, "admin_slb6", true).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // Helper: seed N revoked endorsements for `caller` to push to threshold.
    async fn seed_prior_revocations(
      conn: &mut AsyncPgConnection,
      ctx: &lemmy_api_utils::context::LemmyContext,
      instance_id: lemmy_db_schema_file::InstanceId,
      caller: PersonId,
      caller_label: &str,
      count: usize,
    ) -> LemmyResult<()> {
      let recent = Utc::now() - Duration::hours(1);
      for i in 0..count {
        let name = format!("{caller_label}_revoked_{i:02}");
        let (target, _) = governance_fixtures::seed_user(ctx, instance_id, &name, false).await?;
        let eid: EndorsementId = diesel::insert_into(endorsement::table)
          .values(&EndorsementInsertForm {
            from_person_id: caller,
            to_person_id: target,
            community_id: None,
          })
          .returning(endorsement::id)
          .get_result(conn)
          .await?;
        diesel::update(endorsement::table.filter(endorsement::id.eq(eid)))
          .set(endorsement::revoked_at.eq(recent))
          .execute(conn)
          .await?;
      }
      Ok(())
    }

    seed_prior_revocations(
      &mut conn,
      &context,
      instance.id,
      regular_caller,
      "regular",
      5,
    )
    .await?;
    let (sponsee_regular, _) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_regular_slb6", false).await?;
    let regular_active_eid =
      seed_endorsement_active(&mut conn, regular_caller, sponsee_regular).await?;

    // Regular caller at threshold (5 prior + 6th attempt) — should reject.
    let result = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id: regular_active_eid,
        reason: "6th attempt".to_string(),
      }),
      context.clone(),
      regular_view,
    )
    .await;
    let err = result.expect_err("regular caller at threshold must reject");
    assert!(
      matches!(
        err.error_type,
        lemmy_utils::error::LemmyErrorType::TooManyRequests
      ),
      "expected TooManyRequests at threshold, got {:?}",
      err.error_type,
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    assert!(
      read_endorsement_revoked_at(&mut conn, regular_active_eid)
        .await?
        .is_none(),
      "regular caller's active endorsement remains untouched after rate-limit reject",
    );

    // Admin at threshold — bypasses, succeeds, log includes rate_limit_bypassed: true.
    seed_prior_revocations(&mut conn, &context, instance.id, admin_caller, "admin", 5).await?;
    let (sponsee_admin, _) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_admin_slb6", false).await?;
    let admin_active_eid = seed_endorsement_active(&mut conn, admin_caller, sponsee_admin).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: admin_caller,
        sponsored_id: sponsee_admin,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;

    let resp = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id: admin_active_eid,
        reason: "admin override".to_string(),
      }),
      context.clone(),
      admin_view,
    )
    .await?
    .into_inner();
    assert_eq!(resp.endorsement_id, admin_active_eid);

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let payload = read_log_payload(&mut conn, "endorsement_revoked")
      .await?
      .expect("admin bypass log payload exists");
    assert_eq!(
      payload["rate_limit_bypassed"],
      Value::Bool(true),
      "admin bypass log carries rate_limit_bypassed: true",
    );
    assert_eq!(
      payload["reason"],
      Value::String("admin override".to_string()),
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 7 (plan §13 Task 10): single-sponsor grace-window severance.
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_severs_grace_window_single_sponsor_case() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsor, sponsor_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_slb7", false).await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb7", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_id = seed_endorsement_active(&mut conn, sponsor, sponsee).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;
    let case_id = seed_pending_case(&mut conn, sponsee, None, 24).await?;

    // Pre-call.
    let (status_before, escape_before) = read_case_status_and_escape(&mut conn, case_id).await?;
    assert_eq!(
      status_before,
      CaseStatus::SponsorLiabilityPending,
      "pre-call: pending"
    );
    assert!(
      escape_before.is_none(),
      "pre-call: liability_escape_reason NULL"
    );

    let resp = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id,
        reason: "prudent withdrawal".to_string(),
      }),
      context.clone(),
      sponsor_view,
    )
    .await?
    .into_inner();

    assert_eq!(
      resp.liability_chain_severed_for_cases,
      vec![case_id],
      "severed contains exactly the seeded case",
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let (status_after, escape_after) = read_case_status_and_escape(&mut conn, case_id).await?;
    assert_eq!(
      status_after,
      CaseStatus::SponsorLiabilityEscaped,
      "case status flipped to escaped",
    );
    let escape_json = escape_after.expect("escape_reason JSONB populated");
    assert_eq!(
      escape_json["version"],
      Value::Number(1.into()),
      "version: 1"
    );
    assert_eq!(
      escape_json["reason"],
      Value::String("sponsor_revoked".to_string()),
      "reason: sponsor_revoked",
    );
    assert!(
      escape_json["actor_pseudonym"].is_string(),
      "actor_pseudonym is a string",
    );
    let actor_pseud = escape_json["actor_pseudonym"].as_str().expect("string");
    let raw_id_str = format!("{}", sponsor.0);
    assert_ne!(
      actor_pseud, raw_id_str,
      "ADR-015: actor_pseudonym must not equal raw caller_id",
    );
    assert_eq!(
      escape_json["endorsement_id"],
      Value::Number(endorsement_id.0.into()),
      "endorsement_id matches",
    );

    assert_eq!(
      count_log_entries(&mut conn, "endorsement_revoked").await?,
      1,
      "1 endorsement_revoked entry",
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      1,
      "1 sponsor_liability_escaped entry",
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 8 (plan §13 Task 11): multi-sponsor any_revocation rule (default)
  // severs chain; only revoking sponsor's surety flips.
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_multi_sponsor_any_revocation_severs_chain() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb8", false).await?;
    let (sponsor_a, sponsor_a_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_a_slb8", false).await?;
    let (sponsor_b, _sponsor_b_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_b_slb8", false).await?;
    let (sponsor_c, _sponsor_c_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_c_slb8", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_a = seed_endorsement_active(&mut conn, sponsor_a, sponsee).await?;
    let _endorsement_b = seed_endorsement_active(&mut conn, sponsor_b, sponsee).await?;
    let _endorsement_c = seed_endorsement_active(&mut conn, sponsor_c, sponsee).await?;
    for sponsor in [sponsor_a, sponsor_b, sponsor_c] {
      diesel::insert_into(surety::table)
        .values(&SuretyInsertForm {
          sponsor_id: sponsor,
          sponsored_id: sponsee,
          community_id: None,
        })
        .execute(&mut conn)
        .await?;
    }
    let case_id = seed_pending_case(&mut conn, sponsee, None, 24).await?;

    let resp = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id: endorsement_a,
        reason: "any-rev test".to_string(),
      }),
      context.clone(),
      sponsor_a_view,
    )
    .await?
    .into_inner();
    assert_eq!(
      resp.liability_chain_severed_for_cases,
      vec![case_id],
      "any_revocation default severs chain on first revoke",
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let (status_after, _) = read_case_status_and_escape(&mut conn, case_id).await?;
    assert_eq!(status_after, CaseStatus::SponsorLiabilityEscaped);

    // Only sponsor_a's surety flipped.
    assert!(
      read_surety_revoked_at(&mut conn, sponsor_a, sponsee, None)
        .await?
        .is_some(),
      "sponsor_a's surety revoked",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor_b, sponsee, None)
        .await?
        .is_none(),
      "sponsor_b's surety untouched",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor_c, sponsee, None)
        .await?
        .is_none(),
      "sponsor_c's surety untouched",
    );

    assert_eq!(
      count_log_entries(&mut conn, "endorsement_revoked").await?,
      1,
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      1,
    );

    Ok(())
  }

  // ─────────────────────────────────────────────────────────────────
  // Test 9 (plan §13 Task 12): no pending case → no severance, only
  // endorsement_revoked log.
  // ─────────────────────────────────────────────────────────────────
  #[tokio::test]
  async fn revoke_endorsement_no_pending_case_no_severance_only_revoked_log() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsor, sponsor_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsor_slb9", false).await?;
    let (sponsee, _sponsee_view) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_slb9", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let endorsement_id = seed_endorsement_active(&mut conn, sponsor, sponsee).await?;
    diesel::insert_into(surety::table)
      .values(&SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;
    // NO seed_pending_case call.

    let resp = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id,
        reason: "standard withdrawal".to_string(),
      }),
      context.clone(),
      sponsor_view,
    )
    .await?
    .into_inner();
    assert_eq!(resp.endorsement_id, endorsement_id);
    assert!(
      (Utc::now() - resp.revoked_at).num_seconds() < 5,
      "revoked_at recent",
    );
    assert!(
      resp.liability_chain_severed_for_cases.is_empty(),
      "no pending case → severed empty",
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    assert!(
      read_endorsement_revoked_at(&mut conn, endorsement_id)
        .await?
        .is_some()
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None)
        .await?
        .is_some()
    );

    assert_eq!(
      count_log_entries(&mut conn, "endorsement_revoked").await?,
      1,
      "1 endorsement_revoked entry",
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "no sponsor_liability_escaped (no pending case)",
    );

    let payload = read_log_payload(&mut conn, "endorsement_revoked")
      .await?
      .expect("payload exists");
    assert_eq!(
      payload["liability_chain_severed_for_cases"],
      Value::Array(vec![]),
      "severed array serializes as []",
    );

    // Defensive: no moderation_case rows for this sponsee.
    let case_count: i64 = moderation_case::table
      .filter(moderation_case::target_person_id.eq(sponsee))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(case_count, 0, "no moderation_case rows for sponsee");

    Ok(())
  }
}

mod v1_sl_c_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::governance_fixtures;
  use chrono::{Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, insert_into, update};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch;
  use lemmy_db_schema::{
    newtypes::{ModerationCaseId, SuretyId},
    source::governance::{
      endorsement::EndorsementInsertForm, moderation_case::ModerationCaseInsertForm,
      sanction::SanctionInsertForm, surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, SanctionAction, SanctionScope},
    schema::{endorsement, governance_log, moderation_case, reputation_event, sanction, surety},
  };
  use lemmy_utils::error::LemmyResult;
  use serde_json::Value;

  async fn count_log_entries(conn: &mut AsyncPgConnection, kind: &str) -> LemmyResult<i64> {
    let n: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq(kind))
      .count()
      .get_result(conn)
      .await?;
    Ok(n)
  }

  async fn read_log_payload(
    conn: &mut AsyncPgConnection,
    kind: &str,
  ) -> LemmyResult<Option<Value>> {
    let payloads: Vec<Value> = governance_log::table
      .filter(governance_log::entry_kind.eq(kind))
      .order(governance_log::id.desc())
      .select(governance_log::payload)
      .limit(1)
      .load(conn)
      .await?;
    Ok(payloads.into_iter().next())
  }

  async fn seed_pending_case(
    conn: &mut AsyncPgConnection,
    sponsee: PersonId,
    grace_offset: Duration,
    sanction_action: Option<SanctionAction>,
  ) -> LemmyResult<ModerationCaseId> {
    let now = Utc::now();
    let decided_at = now - Duration::hours(24);
    let grace_expires_at = now + grace_offset;
    let case_id = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_c_test".to_string(),
        severity: CaseSeverity::Medium,
        status: CaseStatus::SponsorLiabilityPending,
        threshold_score: 100,
        grace_expires_at: Some(grace_expires_at),
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(conn)
      .await?;
    update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::decided_at.eq(Some(decided_at)))
      .execute(conn)
      .await?;
    if let Some(action) = sanction_action {
      insert_into(sanction::table)
        .values(SanctionInsertForm {
          case_id,
          scope: SanctionScope::Community,
          action,
          target_person_id: Some(sponsee),
          ends_at: None,
          active: Some(true),
          ..Default::default()
        })
        .execute(conn)
        .await?;
    }
    Ok(case_id)
  }

  async fn seed_active_surety(
    conn: &mut AsyncPgConnection,
    sponsor: PersonId,
    sponsee: PersonId,
  ) -> LemmyResult<SuretyId> {
    insert_into(surety::table)
      .values(SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .returning(surety::id)
      .get_result::<SuretyId>(conn)
      .await
      .map_err(Into::into)
  }

  #[tokio::test]
  async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries() -> LemmyResult<()>
  {
    let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
    unsafe {
      std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1");
    }

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsee, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc1_sponsee", false).await?;
    let (sponsor1, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc1_sponsor1", false).await?;
    let (sponsor2, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc1_sponsor2", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let case_id = seed_pending_case(
      &mut conn,
      sponsee,
      Duration::minutes(-1),
      Some(SanctionAction::ContentRemoval),
    )
    .await?;
    seed_active_surety(&mut conn, sponsor1, sponsee).await?;
    seed_active_surety(&mut conn, sponsor2, sponsee).await?;

    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(outcome.cases_processed, 1, "1 case processed");
    assert_eq!(outcome.fired, 1, "fire branch: 1 case fired");
    assert_eq!(outcome.escaped, 0, "fire branch: 0 escaped");
    assert_eq!(outcome.skipped, 0, "fire branch: 0 skipped");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let (case_status, escape_reason): (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_status,
      CaseStatus::SponsorLiabilityFired,
      "case transitioned to SponsorLiabilityFired"
    );
    assert!(
      escape_reason.is_none(),
      "fire branch: liability_escape_reason IS NULL"
    );

    let rep_event_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_event_count, 2,
      "2 reputation_event rows (1 per sponsor)"
    );

    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      1,
      "1 fired summary"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      2,
      "2 applied entries"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "0 escaped entries"
    );

    let fired_payload = read_log_payload(&mut conn, "sponsor_liability_fired")
      .await?
      .expect("fired payload exists");
    assert!(
      fired_payload["target_pseudonym"].is_string(),
      "target_pseudonym is a string"
    );
    assert_eq!(
      fired_payload["sponsor_count"].as_u64(),
      Some(2),
      "sponsor_count == 2"
    );
    assert_eq!(
      fired_payload["case_id"].as_i64(),
      Some(i64::from(case_id.0)),
      "case_id matches"
    );

    unsafe {
      match prev_disable {
        Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
        None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
      }
    }
    Ok(())
  }

  #[tokio::test]
  async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at() -> LemmyResult<()> {
    // Per Test #2 (PRD §6.2 step 4 escape branch — "any sponsor
    // revoked since decided_at").
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   1 sponsee + 1 sponsor. Endorsement seeded sponsor→sponsee.
    //   ModerationCase status=SponsorLiabilityPending,
    //     grace_expires_at = now() - 1 minute (expired),
    //     decided_at = now() - 24h.
    //   Sanction row seeded.
    //   Surety seeded then revoked_at = now() - 1h
    //     (revoked AFTER decided_at, BEFORE now()).
    //
    // Drive: run_grace_check_batch(&context).await.
    //
    // Assert:
    //   - outcome.escaped == 1, outcome.fired == 0.
    //   - case.status == SponsorLiabilityEscaped.
    //   - case.liability_escape_reason IS Some(json) with expected shape.
    //   - 0 reputation_event rows (escape branch skips apply_sponsor_liability).
    //   - 1 governance_log "sponsor_liability_escaped".
    //   - 0 governance_log "sponsor_liability_fired".
    //   - 0 governance_log "sponsor_liability_applied".
    let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
    unsafe {
      std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1");
    }

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsee, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc2_escape_sponsee", false).await?;
    let (sponsor, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc2_escape_sponsor", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let now = Utc::now();
    let decided_at = now - Duration::hours(24);
    let grace_exp = now - Duration::minutes(1);

    // Seed endorsement (sponsor → sponsee) before surety so
    // evaluate_escape_conditions' endorsement-id lookup finds a row.
    insert_into(endorsement::table)
      .values(EndorsementInsertForm {
        from_person_id: sponsor,
        to_person_id: sponsee,
        community_id: None,
      })
      .execute(&mut conn)
      .await?;

    // Seed expired pending case.
    let case_id = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_c2_escape_test".to_string(),
        severity: CaseSeverity::Medium,
        status: CaseStatus::SponsorLiabilityPending,
        threshold_score: 100,
        grace_expires_at: Some(grace_exp),
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;
    update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::decided_at.eq(Some(decided_at)))
      .execute(&mut conn)
      .await?;

    // Sanction row required — fire_or_escape_case_inner skips cases without one.
    insert_into(sanction::table)
      .values(SanctionInsertForm {
        case_id,
        scope: SanctionScope::Community,
        action: SanctionAction::ContentRemoval,
        target_person_id: Some(sponsee),
        ends_at: None,
        active: Some(true),
        ..Default::default()
      })
      .execute(&mut conn)
      .await?;

    // Seed surety then set revoked_at = now - 1h (after decided_at = now-24h,
    // before now()) so evaluate_escape_conditions returns EscapeStatus::Escape.
    let surety_id = insert_into(surety::table)
      .values(SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsee,
        community_id: None,
      })
      .returning(surety::id)
      .get_result::<SuretyId>(&mut conn)
      .await?;
    update(surety::table.filter(surety::id.eq(surety_id)))
      .set(surety::revoked_at.eq(Some(now - Duration::hours(1))))
      .execute(&mut conn)
      .await?;

    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(outcome.escaped, 1, "escape branch: 1 case escaped");
    assert_eq!(outcome.fired, 0, "escape branch: 0 cases fired");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let (case_status, escape_reason): (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_status,
      CaseStatus::SponsorLiabilityEscaped,
      "case transitioned to SponsorLiabilityEscaped"
    );

    let json = escape_reason.expect("liability_escape_reason IS Some(json)");
    assert_eq!(json["version"].as_i64(), Some(1), "version == 1");
    assert_eq!(
      json["reason"].as_str(),
      Some("sponsor_revoked"),
      "reason == sponsor_revoked"
    );
    assert!(
      json["actor_pseudonym"].is_string(),
      "actor_pseudonym is a string"
    );
    // ADR-015: actor_pseudonym must NOT equal raw sponsor PersonId.
    assert_ne!(
      json["actor_pseudonym"].as_str().unwrap_or(""),
      &format!("{}", sponsor.0),
      "actor_pseudonym is NOT raw sponsor PersonId (ADR-015)"
    );
    assert!(
      json["endorsement_id"].as_i64().is_some(),
      "endorsement_id present in JSONB"
    );
    assert!(
      json["endorsement_id"].as_i64().unwrap_or(-1) >= 0,
      "endorsement_id >= 0 (endorsement row was seeded)"
    );

    // Escape branch does NOT call apply_sponsor_liability — 0 reputation_event rows.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_count, 0, "0 reputation_event rows (escape branch)");

    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      1,
      "1 escaped log entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      0,
      "0 fired log entries"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      0,
      "0 applied log entries"
    );

    let escaped_payload = read_log_payload(&mut conn, "sponsor_liability_escaped")
      .await?
      .expect("escaped payload exists");
    assert!(
      escaped_payload["actor_pseudonym"].is_string(),
      "log payload actor_pseudonym is a string"
    );

    unsafe {
      match prev_disable {
        Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
        None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
      }
    }
    Ok(())
  }

  #[tokio::test]
  async fn grace_check_no_op_when_grace_expires_at_in_future() -> LemmyResult<()> {
    // Per Test #3 (PRD §6.1 batch-query filter — only cases past
    // grace_expires_at).
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   1 sponsee + 1 sponsor. Active surety.
    //   ModerationCase status=SponsorLiabilityPending,
    //     grace_expires_at = now() + 2 hours (NOT YET EXPIRED),
    //     decided_at = now() - 24h.
    //   sanction row.
    //
    // Drive: run_grace_check_batch(&context).await.
    //
    // Assert:
    //   - outcome.cases_processed == 0 (case not selected by batch query).
    //   - outcome.fired == 0, outcome.escaped == 0.
    //   - case.status STILL == SponsorLiabilityPending (unchanged).
    //   - case.liability_escape_reason IS STILL NULL.
    //   - 0 reputation_event rows for the case.
    //   - 0 governance_log rows of any SL kind.
    let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
    unsafe {
      std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1");
    }

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsee, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc3_noop_sponsee", false).await?;
    let (sponsor, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc3_noop_sponsor", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    // grace_expires_at = now() + 2 hours — NOT yet expired;
    // the batch query filters .le(Some(now)) so this case is skipped.
    let case_id = seed_pending_case(
      &mut conn,
      sponsee,
      Duration::hours(2),
      Some(SanctionAction::ContentRemoval),
    )
    .await?;
    seed_active_surety(&mut conn, sponsor, sponsee).await?;

    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(
      outcome.cases_processed, 0,
      "no-op: future grace_expires_at case not selected by batch query"
    );
    assert_eq!(outcome.fired, 0, "no-op: 0 cases fired");
    assert_eq!(outcome.escaped, 0, "no-op: 0 cases escaped");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let (case_status, escape_reason): (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_status,
      CaseStatus::SponsorLiabilityPending,
      "no-op: case status STILL SponsorLiabilityPending (unchanged)"
    );
    assert!(
      escape_reason.is_none(),
      "no-op: liability_escape_reason STILL NULL"
    );

    let rep_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_count, 0, "no-op: 0 reputation_event rows");

    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      0,
      "no-op: 0 fired log entries"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "no-op: 0 escaped log entries"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      0,
      "no-op: 0 applied log entries"
    );

    unsafe {
      match prev_disable {
        Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
        None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
      }
    }
    Ok(())
  }

  #[tokio::test]
  async fn grace_check_per_case_isolation_skips_bad_case_processes_good_case() -> LemmyResult<()> {
    // Per Test #4 (PRD §6.3 + §4 watchpoint #8 — per-case isolation
    // invariant).
    //
    // case_b (malformed: zero sanction rows, grace_expires_at = -2 min) is
    // ordered FIRST by the batch query's ORDER BY grace_expires_at ASC.
    // case_a (well-formed: one sanction, grace_expires_at = -1 min) is second.
    // case_b's error-skip MUST NOT block case_a from firing (strong assertion).
    let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
    unsafe {
      std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1");
    }

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let (sponsee_a, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc4_isol_sponsee_a", false).await?;
    let (sponsee_b, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc4_isol_sponsee_b", false).await?;
    let (sponsor, _) =
      governance_fixtures::seed_user(&context, instance.id, "slc4_isol_sponsor", false).await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    // case_b: earlier grace_expires_at (-2 min) — processed FIRST; no sanction → skipped.
    let case_b_id = seed_pending_case(&mut conn, sponsee_b, Duration::minutes(-2), None).await?;
    // case_a: later grace_expires_at (-1 min) — processed SECOND; has sanction → fires.
    let case_a_id = seed_pending_case(
      &mut conn,
      sponsee_a,
      Duration::minutes(-1),
      Some(SanctionAction::ContentRemoval),
    )
    .await?;
    // Seed surety for sponsor → sponsee_a only (sponsee_b has no active sponsor).
    seed_active_surety(&mut conn, sponsor, sponsee_a).await?;

    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(
      outcome.cases_processed, 2,
      "2 cases processed (case_b first, case_a second)"
    );
    assert_eq!(outcome.fired, 1, "1 case fired (case_a)");
    assert_eq!(
      outcome.skipped, 1,
      "1 case skipped (case_b — zero sanction rows)"
    );
    assert_eq!(outcome.escaped, 0, "0 cases escaped");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let (case_a_status, _): (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_a_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_a_status,
      CaseStatus::SponsorLiabilityFired,
      "case_a transitioned to SponsorLiabilityFired"
    );

    let (case_b_status, case_b_escape_reason): (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_b_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_b_status,
      CaseStatus::SponsorLiabilityPending,
      "case_b STILL SponsorLiabilityPending (silently skipped)"
    );
    assert!(
      case_b_escape_reason.is_none(),
      "case_b.liability_escape_reason STILL NULL"
    );

    // 1 reputation_event for case_a's sponsor; 0 for case_b.
    let rep_a_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_a_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_a_count, 1,
      "1 reputation_event row for case_a's sponsor"
    );

    let rep_b_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_b_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_b_count, 0,
      "0 reputation_event rows for case_b (skipped)"
    );

    // Governance log: 1 fired (case_a only), 1 applied (case_a's sponsor), 0 escaped.
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      1,
      "1 fired log entry (case_a)"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      1,
      "1 applied log entry (case_a's sponsor)"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "0 escaped log entries"
    );

    unsafe {
      match prev_disable {
        Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
        None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
      }
    }
    Ok(())
  }

  #[tokio::test]
  async fn grace_check_batch_size_config_caps_iteration() -> LemmyResult<()> {
    // Per Test #5 (PRD §6.4 + §4.1 batch_size config).
    //
    // Setup: BREHON_DISABLE_GRACE_CHECK_JOB=1.
    //   INSERT governance_config row "job.grace_check_batch_size" = 2
    //     (instance scope). Default is 100; we override to 2.
    //   5 sponsees + 5 sponsors (1 surety each). All 5 cases:
    //     status=SponsorLiabilityPending, grace_expires_at expired,
    //     sanction inserted. Distinct grace_expires_at via
    //     now() - Duration::minutes(N) for N in 5..1 (ASC = order of
    //     processing per the batch query's ORDER BY grace_expires_at ASC).
    //
    // Drive (first invocation): run_grace_check_batch(&context).await.
    //
    // Assert (first invocation):
    //   - outcome.cases_processed == 2 (batch_size cap honoured).
    //   - outcome.fired == 2.
    //   - 2 cases transitioned to SponsorLiabilityFired.
    //   - 3 cases STILL == SponsorLiabilityPending.
    //
    // Drive (second invocation): run_grace_check_batch(&context).await.
    //
    // Assert (second invocation):
    //   - outcome.cases_processed == 2 (next 2 picked up).
    //   - outcome.fired == 2.
    //   - 4 cases now SponsorLiabilityFired total.
    //   - 1 case STILL == SponsorLiabilityPending.
    //
    // Drive (third invocation): run_grace_check_batch(&context).await.
    //
    // Assert (third invocation):
    //   - outcome.cases_processed == 1 (last remaining).
    //   - outcome.fired == 1.
    //   - all 5 cases now SponsorLiabilityFired.
    let prev_disable = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
    unsafe {
      std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", "1");
    }

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;

    // Seed 5 sponsees + 5 sponsors.
    let mut sponsee_ids = Vec::with_capacity(5);
    let mut sponsor_ids = Vec::with_capacity(5);
    for i in 0..5usize {
      let (sponsee, _) = governance_fixtures::seed_user(
        &context,
        instance.id,
        &format!("slc5_batch_sponsee_{i}"),
        false,
      )
      .await?;
      let (sponsor, _) = governance_fixtures::seed_user(
        &context,
        instance.id,
        &format!("slc5_batch_sponsor_{i}"),
        false,
      )
      .await?;
      sponsee_ids.push(sponsee);
      sponsor_ids.push(sponsor);
    }

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // Override batch_size to 2. The seeded default is 100. Use append-only
    // INSERT (governance_config is not upserted — new row with later
    // valid_from wins per ORDER BY valid_from DESC in fetch_value_at_scope).
    diesel::sql_query(
      "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
       VALUES ('instance', 'job.grace_check_batch_size', 'int', 2, now())",
    )
    .execute(&mut conn)
    .await?;

    // Seed 5 cases with DISTINCT grace_expires_at so ORDER BY grace_expires_at
    // ASC is deterministic. Offsets: -5, -4, -3, -2, -1 minutes.
    // Index 0 → earliest (processed first); index 4 → latest (processed last).
    for i in 0..5usize {
      let offset_minutes = 5 - i64::try_from(i).expect("loop index fits i64");
      seed_pending_case(
        &mut conn,
        sponsee_ids[i],
        Duration::minutes(-offset_minutes),
        Some(SanctionAction::ContentRemoval),
      )
      .await?;
      seed_active_surety(&mut conn, sponsor_ids[i], sponsee_ids[i]).await?;
    }

    // --- First invocation: batch_size=2 → processes cases[0] and cases[1] ---
    let outcome1 = run_grace_check_batch(&context).await?;
    assert_eq!(
      outcome1.cases_processed, 2,
      "invocation 1: 2 cases processed (batch_size cap)"
    );
    assert_eq!(outcome1.fired, 2, "invocation 1: 2 cases fired");
    assert_eq!(outcome1.escaped, 0, "invocation 1: 0 cases escaped");
    assert_eq!(outcome1.skipped, 0, "invocation 1: 0 cases skipped");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let fired_after_1: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityFired))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      fired_after_1, 2,
      "invocation 1: 2 cases total SponsorLiabilityFired"
    );
    let pending_after_1: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      pending_after_1, 3,
      "invocation 1: 3 cases STILL SponsorLiabilityPending"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      2,
      "invocation 1: 2 fired log entries"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      2,
      "invocation 1: 2 applied log entries (1 per sponsor)"
    );

    // --- Second invocation: picks up cases[2] and cases[3] ---
    let outcome2 = run_grace_check_batch(&context).await?;
    assert_eq!(
      outcome2.cases_processed, 2,
      "invocation 2: 2 cases processed"
    );
    assert_eq!(outcome2.fired, 2, "invocation 2: 2 cases fired");
    assert_eq!(outcome2.escaped, 0, "invocation 2: 0 cases escaped");
    assert_eq!(outcome2.skipped, 0, "invocation 2: 0 cases skipped");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let fired_after_2: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityFired))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      fired_after_2, 4,
      "invocation 2: 4 cases total SponsorLiabilityFired"
    );
    let pending_after_2: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      pending_after_2, 1,
      "invocation 2: 1 case STILL SponsorLiabilityPending"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      4,
      "invocation 2: 4 fired log entries total"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      4,
      "invocation 2: 4 applied log entries total"
    );

    // --- Third invocation: picks up cases[4] (last remaining) ---
    let outcome3 = run_grace_check_batch(&context).await?;
    assert_eq!(
      outcome3.cases_processed, 1,
      "invocation 3: 1 case processed (last remaining)"
    );
    assert_eq!(outcome3.fired, 1, "invocation 3: 1 case fired");
    assert_eq!(outcome3.escaped, 0, "invocation 3: 0 cases escaped");
    assert_eq!(outcome3.skipped, 0, "invocation 3: 0 cases skipped");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let fired_after_3: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityFired))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      fired_after_3, 5,
      "invocation 3: all 5 cases SponsorLiabilityFired"
    );
    let pending_after_3: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      pending_after_3, 0,
      "invocation 3: 0 cases STILL SponsorLiabilityPending"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      5,
      "invocation 3: 5 fired log entries total"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      5,
      "invocation 3: 5 applied log entries total (1 per sponsor per case)"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "invocation 3: 0 escaped log entries"
    );

    unsafe {
      match prev_disable {
        Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
        None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
      }
    }
    Ok(())
  }
}

mod v1_sl_d_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::governance_fixtures;
  use activitypub_federation::config::FederationConfig;
  use actix_web::web::{Data, Json};
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, insert_into};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote};
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::{
    newtypes::ModerationCaseId,
    source::governance::{moderation_case::ModerationCaseInsertForm, surety::SuretyInsertForm},
  };
  use lemmy_db_schema_file::{
    InstanceId, PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryDecision, SeverityTier},
    schema::{governance_log, moderation_case, public_case_log, reputation_event, surety},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::LemmyResult;
  use serde_json::Value;

  /// Seed a sponsee + `sponsor_count` active-surety sponsors, inserting surety
  /// rows pointing to the sponsee. Returns (sponsee_id, sponsor_ids).
  async fn seed_target_with_sureties(
    context: &Data<LemmyContext>,
    instance_id: InstanceId,
    conn: &mut AsyncPgConnection,
    sponsor_count: usize,
    prefix: &str,
  ) -> LemmyResult<(PersonId, Vec<PersonId>)> {
    let (sponsee, _) =
      governance_fixtures::seed_user(context, instance_id, &format!("{prefix}_sponsee"), false)
        .await?;
    let mut sponsor_ids = Vec::with_capacity(sponsor_count);
    for i in 0..sponsor_count {
      let (sponsor, _) =
        governance_fixtures::seed_user(context, instance_id, &format!("{prefix}_sp{i}"), false)
          .await?;
      insert_into(surety::table)
        .values(SuretyInsertForm {
          sponsor_id: sponsor,
          sponsored_id: sponsee,
          community_id: None,
        })
        .execute(conn)
        .await?;
      sponsor_ids.push(sponsor);
    }
    Ok((sponsee, sponsor_ids))
  }

  #[tokio::test]
  async fn submit_jury_vote_transitions_to_pending_for_liability_bearing_sponsored_case()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;

    // submit_jury_vote requires activitypub_federation Data context for outbox
    // federation calls — mirror golden-path builder pattern exactly.
    let federation_config = FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data((**context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await?;
    let federation_context = federation_config.to_request_data();

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // Target (sponsee) with 2 active sureties → compute_sponsor_liability returns
    // non-empty → Pending path fires on the decisive vote.
    let (sponsee, _sponsors) =
      seed_target_with_sureties(&context, instance.id, &mut conn, 2, "sld1").await?;

    // 5 jury-eligible persons + 1 admin.
    let mut juror_ids = Vec::with_capacity(5);
    for i in 0..5_usize {
      let (id, _) =
        governance_fixtures::seed_user(&context, instance.id, &format!("sld_juror{i}"), false)
          .await?;
      juror_ids.push(id);
    }
    let (_, admin_view) =
      governance_fixtures::seed_user(&context, instance.id, "sld_admin", true).await?;

    // Reputation snapshots required for the strict eligibility query in
    // admin_assign_jury. Same pattern as v1_jm_b / v1_jm_e fixtures.
    super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &juror_ids).await?;

    // Case: High severity (→ CaseSeverity::High → severity_str "severe" → 168h
    // grace window), Minor severity_tier (→ 5-juror panel).
    let case_id: ModerationCaseId = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_d_test".to_string(),
        severity: CaseSeverity::High,
        severity_tier: Some(SeverityTier::Minor),
        status: CaseStatus::Open,
        threshold_score: 1,
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;

    let assign_resp = admin_assign_jury(
      Json(AdminAssignJury { case_id }),
      context.clone(),
      admin_view,
    )
    .await?
    .into_inner();
    assert_eq!(
      assign_resp.assigned_person_ids.len(),
      5,
      "Minor panel = 5 jurors"
    );

    for &juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }

    // Votes 1–2: below quorum → case_decided = false, no post-decision side effects.
    for &juror_id in &assign_resp.assigned_person_ids[..2] {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      let resp = submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision: JuryDecision::SuspendCommunityMember,
          rationale: None,
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?
      .into_inner();
      assert!(!resp.case_decided, "votes 1-2: not yet at quorum");
    }

    // Vote 3: quorum reached (SuspendCommunityMember × 3 ≥ threshold_count for
    // Minor panel) → sanction inserted → compute_sponsor_liability finds 2 active
    // sureties → SLD Pending path fires → case → SponsorLiabilityPending.
    let before_decisive = Utc::now();
    let juror_view_2 =
      LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2]).await?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::SuspendCommunityMember,
        rationale: None,
      }),
      federation_context.reset_request_count(),
      juror_view_2,
    )
    .await?
    .into_inner();

    assert!(resp.case_decided, "3rd vote decides the case");
    assert_eq!(
      resp.decision,
      Some(JuryDecision::SuspendCommunityMember),
      "winning decision = SuspendCommunityMember"
    );

    // --- DB assertions ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (status, grace_expires_at, appeal_window_expires_at): (
      CaseStatus,
      Option<DateTime<Utc>>,
      Option<DateTime<Utc>>,
    ) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::grace_expires_at,
        moderation_case::appeal_window_expires_at,
      ))
      .first(&mut conn)
      .await?;

    assert!(
      matches!(status, CaseStatus::SponsorLiabilityPending),
      "case must be SponsorLiabilityPending, got {status:?}"
    );
    let grace = grace_expires_at.expect("grace_expires_at set on Pending path");
    let expected_grace = before_decisive + Duration::hours(168);
    assert!(
      (grace - expected_grace).num_seconds().abs() < 5,
      "grace_expires_at ≈ now + 168h (within 5s), got {grace:?}"
    );
    assert!(
      appeal_window_expires_at.is_some(),
      "appeal_window_expires_at set on both Decided and Pending paths"
    );

    // Steps 10–12 (reputation_events, public_case_log) are deferred on Pending path.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_count, 0, "reputation_events deferred on Pending path");

    let plog_count: i64 = public_case_log::table.count().get_result(&mut conn).await?;
    assert_eq!(plog_count, 0, "public_case_log deferred on Pending path");

    // governance_log: case_decided (both paths) + sanction_created + sponsor_liability_pending.
    let decided_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("case_decided"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(decided_count, 1, "1 case_decided log entry");

    let slt_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_pending"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(slt_count, 1, "1 sponsor_liability_pending log entry");

    let sanction_log_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sanction_created"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(sanction_log_count, 1, "1 sanction_created log entry");

    // sponsor_liability_pending payload — ADR-015 pseudonym discipline.
    let payload: Value = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_pending"))
      .select(governance_log::payload)
      .first(&mut conn)
      .await?;

    assert_eq!(
      payload["case_id"],
      serde_json::json!(case_id.0),
      "payload.case_id matches"
    );
    let target_psn = payload["target_pseudonym"]
      .as_str()
      .expect("target_pseudonym is a string");
    assert_eq!(
      target_psn.len(),
      36,
      "target_pseudonym is a UUID (36 chars)"
    );
    assert_ne!(
      target_psn,
      format!("{}", sponsee.0),
      "target_pseudonym != raw person_id (ADR-015)"
    );
    assert_eq!(
      payload["severity"],
      serde_json::json!("severe"),
      "CaseSeverity::High → severity_str = severe"
    );
    assert!(
      payload["grace_expires_at"].as_str().is_some(),
      "grace_expires_at present as ISO 8601 string in payload"
    );
    let psns = payload["sponsors_pseudonyms"]
      .as_array()
      .expect("sponsors_pseudonyms is an array");
    assert_eq!(
      psns.len(),
      2,
      "2 sponsor pseudonyms (one per active surety)"
    );
    for psn in psns {
      assert_eq!(
        psn.as_str().map(str::len),
        Some(36),
        "each sponsor pseudonym is a UUID (36 chars)"
      );
    }

    Ok(())
  }

  #[tokio::test]
  async fn submit_jury_vote_preserves_v0_decided_for_no_sponsor_target() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;

    let federation_config = FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data((**context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await?;
    let federation_context = federation_config.to_request_data();

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // Target person with ZERO active sureties → compute_sponsor_liability returns
    // empty vec → handler stays on Decided path (v0 semantics preserved).
    let (target_id, _) =
      governance_fixtures::seed_user(&context, instance.id, "sld2_target", false).await?;
    // Reporter: sets creator_id so the reporter reputation_event fires on Decided path.
    let (reporter_id, _) =
      governance_fixtures::seed_user(&context, instance.id, "sld2_reporter", false).await?;

    let mut juror_ids = Vec::with_capacity(5);
    for i in 0..5_usize {
      let (id, _) =
        governance_fixtures::seed_user(&context, instance.id, &format!("sld2_juror{i}"), false)
          .await?;
      juror_ids.push(id);
    }
    let (_, admin_view) =
      governance_fixtures::seed_user(&context, instance.id, "sld2_admin", true).await?;

    super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &juror_ids).await?;

    // CaseSeverity::Medium → severity_str "moderate"; SeverityTier::Minor → 5-panel.
    // creator_id set so reporter reputation_event fires on Decided path.
    let case_id: ModerationCaseId = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(target_id),
        creator_id: Some(reporter_id),
        reason_code: "v1_sl_d_test2".to_string(),
        severity: CaseSeverity::Medium,
        severity_tier: Some(SeverityTier::Minor),
        status: CaseStatus::Open,
        threshold_score: 1,
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;

    let assign_resp = admin_assign_jury(
      Json(AdminAssignJury { case_id }),
      context.clone(),
      admin_view,
    )
    .await?
    .into_inner();
    assert_eq!(
      assign_resp.assigned_person_ids.len(),
      5,
      "Minor panel = 5 jurors"
    );

    for &juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }

    // Votes 1–2: below quorum (threshold_count = 3 for Minor panel) → not decided.
    for &juror_id in &assign_resp.assigned_person_ids[..2] {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      let resp = submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision: JuryDecision::RemoveContent,
          rationale: None,
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?
      .into_inner();
      assert!(!resp.case_decided, "votes 1-2: not yet at quorum");
    }

    // Vote 3: quorum reached (RemoveContent × 3 ≥ threshold_count for Minor panel).
    // Target has no sureties → compute_sponsor_liability returns vec![] → Decided path.
    let juror_view_2 =
      LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2]).await?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: None,
      }),
      federation_context.reset_request_count(),
      juror_view_2,
    )
    .await?
    .into_inner();

    assert!(resp.case_decided, "3rd vote decides the case");
    assert_eq!(
      resp.decision,
      Some(JuryDecision::RemoveContent),
      "winning decision = RemoveContent"
    );

    // --- DB assertions ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (status, grace_expires_at, appeal_window_expires_at): (
      CaseStatus,
      Option<DateTime<Utc>>,
      Option<DateTime<Utc>>,
    ) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::grace_expires_at,
        moderation_case::appeal_window_expires_at,
      ))
      .first(&mut conn)
      .await?;

    assert!(
      matches!(status, CaseStatus::Decided),
      "no-sponsor path: case must be Decided (v0 semantics), got {status:?}"
    );
    assert!(
      grace_expires_at.is_none(),
      "grace_expires_at must be NULL on Decided path (no Pending transition)"
    );
    assert!(
      appeal_window_expires_at.is_some(),
      "appeal_window_expires_at set on Decided path"
    );

    // Steps 10–12 fire immediately on Decided path (not deferred like Pending path).
    let plog_count: i64 = public_case_log::table.count().get_result(&mut conn).await?;
    assert_eq!(plog_count, 1, "1 public_case_log row on Decided path");

    // 3 ParticipationConsistency (RT-r3 vote-outcome emit, 3 majority-aligned jurors)
    // + 3 JuryReliability (3 votes cast) + 1 ReportingAccuracy (reporter) = 7 total.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count, 7,
      "3 ParticipationConsistency + 3 JuryReliability + 1 ReportingAccuracy reputation events fire immediately on Decided path (RT-r3 vote-outcome added the 3 ParticipationConsistency rows)"
    );

    // 0 sponsor_liability_pending entries: no sureties → Decided path, not Pending.
    let slt_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_pending"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      slt_count, 0,
      "0 sponsor_liability_pending log entries on no-sponsor path"
    );

    let decided_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("case_decided"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(decided_count, 1, "1 case_decided log entry");

    let sanction_log_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sanction_created"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(sanction_log_count, 1, "1 sanction_created log entry");

    Ok(())
  }

  #[tokio::test]
  async fn submit_jury_vote_no_action_skips_liability_machinery() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;

    let federation_config = FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data((**context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await?;
    let federation_context = federation_config.to_request_data();

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // Target (sponsee) with 2 active sureties — liability machinery would fire on a
    // liability-bearing decision; NoAction → map_decision_to_sanction returns None →
    // the entire if-let block at submit_jury_vote.rs:434 is skipped →
    // compute_sponsor_liability is never called.
    let (sponsee, sponsor_ids) =
      seed_target_with_sureties(&context, instance.id, &mut conn, 2, "sld3").await?;
    let (reporter_id, _) =
      governance_fixtures::seed_user(&context, instance.id, "sld3_reporter", false).await?;

    let mut juror_ids = Vec::with_capacity(5);
    for i in 0..5_usize {
      let (id, _) =
        governance_fixtures::seed_user(&context, instance.id, &format!("sld3_juror{i}"), false)
          .await?;
      juror_ids.push(id);
    }
    let (_, admin_view) =
      governance_fixtures::seed_user(&context, instance.id, "sld3_admin", true).await?;

    super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &juror_ids).await?;

    let case_id: ModerationCaseId = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        creator_id: Some(reporter_id),
        reason_code: "v1_sl_d_test3".to_string(),
        severity: CaseSeverity::High,
        severity_tier: Some(SeverityTier::Minor),
        status: CaseStatus::Open,
        threshold_score: 1,
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;

    let assign_resp = admin_assign_jury(
      Json(AdminAssignJury { case_id }),
      context.clone(),
      admin_view,
    )
    .await?
    .into_inner();
    assert_eq!(
      assign_resp.assigned_person_ids.len(),
      5,
      "Minor panel = 5 jurors"
    );

    for &juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }

    // Votes 1–2: below quorum (threshold_count = 3 for Minor panel) → not decided.
    for &juror_id in &assign_resp.assigned_person_ids[..2] {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      let resp = submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision: JuryDecision::NoAction,
          rationale: None,
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?
      .into_inner();
      assert!(!resp.case_decided, "votes 1-2: not yet at quorum");
    }

    // Vote 3: quorum reached (NoAction × 3 ≥ threshold_count for Minor panel).
    // NoAction → map_decision_to_sanction returns None → if-let block skipped →
    // compute_sponsor_liability never called → case → Decided (not SponsorLiabilityPending).
    let juror_view_2 =
      LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2]).await?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::NoAction,
        rationale: None,
      }),
      federation_context.reset_request_count(),
      juror_view_2,
    )
    .await?
    .into_inner();

    assert!(resp.case_decided, "3rd vote decides the case");
    assert_eq!(
      resp.decision,
      Some(JuryDecision::NoAction),
      "winning decision = NoAction"
    );

    // --- DB assertions ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (status, grace_expires_at): (CaseStatus, Option<DateTime<Utc>>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((moderation_case::status, moderation_case::grace_expires_at))
      .first(&mut conn)
      .await?;

    assert!(
      matches!(status, CaseStatus::Decided),
      "NoAction path: case must be Decided (no liability), got {status:?}"
    );
    assert!(
      grace_expires_at.is_none(),
      "grace_expires_at must be NULL on NoAction path (no Pending transition)"
    );

    // Liability machinery skipped entirely: 0 sponsor_liability_pending entries.
    let slt_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_pending"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      slt_count, 0,
      "0 sponsor_liability_pending log entries on NoAction path"
    );

    // NoAction → map_decision_to_sanction returns None → no sanction row → 0 sanction_created.
    let sanction_log_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sanction_created"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      sanction_log_count, 0,
      "0 sanction_created log entries on NoAction path"
    );

    // case_decided fires unconditionally (path-agnostic, submit_jury_vote.rs:678-688).
    let decided_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("case_decided"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(decided_count, 1, "1 case_decided log entry");

    // Sponsors have 0 reputation_event rows: compute_sponsor_liability never called.
    let sponsor_rep_count: i64 = reputation_event::table
      .filter(reputation_event::person_id.eq_any(&sponsor_ids))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      sponsor_rep_count, 0,
      "0 reputation_event rows for sponsors on NoAction path"
    );

    // Juror events fire for the 3 who voted (3 JuryReliability) + reporter (1 ReportingAccuracy);
    // RT-r3 vote-outcome adds 3 ParticipationConsistency (3 NoAction-aligned jurors).
    // Total = 3 ParticipationConsistency + 3 JuryReliability + 1 ReportingAccuracy = 7.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count, 7,
      "3 ParticipationConsistency + 3 JuryReliability + 1 ReportingAccuracy reputation events fire on NoAction Decided path (RT-r3 vote-outcome added the 3 ParticipationConsistency rows)"
    );

    Ok(())
  }

  #[tokio::test]
  async fn apply_sponsor_liability_wrapper_preserves_v0_outputs() -> LemmyResult<()> {
    use diesel::update;
    use lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch;
    use lemmy_db_schema::source::governance::sanction::SanctionInsertForm;
    use lemmy_db_schema_file::enums::{ReputationDimension, SanctionAction, SanctionScope};
    use lemmy_db_schema_file::schema::sanction;

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // Seed sponsee + 2 sponsors with active sureties.
    let (sponsee, sponsor_ids) =
      seed_target_with_sureties(&context, instance.id, &mut conn, 2, "sld4").await?;

    // Seed reputation_snapshot rows for sponsors (endorsement_strength = 100) so the
    // floor clamp (floor = 0) does not engage on the computed -25 per-sponsor delta.
    super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &sponsor_ids).await?;

    let now = Utc::now();

    // Seed moderation_case in SponsorLiabilityPending with expired grace.
    let case_id: ModerationCaseId = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_d_test4".to_string(),
        severity: CaseSeverity::Medium,
        status: CaseStatus::SponsorLiabilityPending,
        threshold_score: 0,
        grace_expires_at: Some(now - Duration::minutes(1)),
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;

    // fire_or_escape_case_inner skips cases with NULL decided_at — set it explicitly.
    update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::decided_at.eq(Some(now - Duration::hours(24))))
      .execute(&mut conn)
      .await?;

    // Sanction row required — ContentRemoval → Moderate severity → raw_delta = -50.
    // fire_or_escape_case_inner skips cases with no sanction row.
    insert_into(sanction::table)
      .values(SanctionInsertForm {
        case_id,
        scope: SanctionScope::Community,
        action: SanctionAction::ContentRemoval,
        target_person_id: Some(sponsee),
        ends_at: None,
        active: Some(true),
        ..Default::default()
      })
      .execute(&mut conn)
      .await?;

    // Drive: run_grace_check_batch fires apply_sponsor_liability (thin wrapper) internally
    // at sponsor_liability_grace.rs:510 (fire branch of fire_or_escape_case_inner).
    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(outcome.cases_processed, 1, "1 case processed");
    assert_eq!(outcome.fired, 1, "fire branch: 1 case fired");
    assert_eq!(outcome.escaped, 0, "fire branch: 0 escaped");
    assert_eq!(outcome.skipped, 0, "fire branch: 0 skipped");

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // case.status == SponsorLiabilityFired (set by fire branch after wrapper returns).
    let case_status: CaseStatus = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select(moderation_case::status)
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_status,
      CaseStatus::SponsorLiabilityFired,
      "case transitioned to SponsorLiabilityFired"
    );

    // --- reputation_event assertions ---
    // Expected per-sponsor delta: raw_delta = DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE
    // = -50; sponsor_count = 2; per_sponsor_base = -25; remainder = 0 (no bump);
    // regular_multiplier = 1.0 (non-founder); post_multiplier_delta = -25;
    // current_endorsement_strength = 100 (seeded); 100 + (-25) = 75 >= floor 0 => no clamp;
    // final_delta = -25.  BYTE-IDENTICAL to v0 single-pass body output for these seeds.
    let rep_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .filter(reputation_event::dimension.eq(ReputationDimension::EndorsementStrength))
      .filter(reputation_event::reason.eq("sponsor_liability_applied"))
      .filter(reputation_event::delta.eq(-25_i32))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count, 2,
      "2 reputation_event rows: dimension=EndorsementStrength, reason=sponsor_liability_applied, delta=-25"
    );

    // --- governance_log assertions for sponsor_liability_applied ---
    // 2 entries (one per sponsor); payload BYTE-IDENTICAL to v0 payload shape at
    // sponsor_liability.rs:438-448.
    let applied_payloads: Vec<Value> = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_applied"))
      .order_by(governance_log::id.asc())
      .select(governance_log::payload)
      .load(&mut conn)
      .await?;
    assert_eq!(
      applied_payloads.len(),
      2,
      "2 sponsor_liability_applied log entries"
    );
    for payload in &applied_payloads {
      assert!(
        payload["sponsor_pseudonym"].is_string(),
        "sponsor_pseudonym is a string (ADR-015)"
      );
      assert_eq!(
        payload["severity"],
        serde_json::json!("moderate"),
        "severity = moderate (ContentRemoval => Moderate)"
      );
      assert_eq!(
        payload["pre_multiplier_delta"].as_i64(),
        Some(-25),
        "pre_multiplier_delta = -25"
      );
      assert_eq!(
        payload["multiplier"].as_f64(),
        Some(1.0),
        "multiplier = 1.0 (regular_multiplier, non-founder)"
      );
      assert_eq!(
        payload["post_multiplier_delta"].as_i64(),
        Some(-25),
        "post_multiplier_delta = -25"
      );
      assert_eq!(
        payload["final_delta"].as_i64(),
        Some(-25),
        "final_delta = -25 (no clamp: 100 + (-25) = 75 >= floor 0)"
      );
    }

    // 0 governance_log rows with entry_kind == "sponsor_liability_clamped" (no clamp).
    let clamped_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_clamped"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      clamped_count, 0,
      "0 sponsor_liability_clamped entries (no clamp engaged)"
    );

    // 1 governance_log row with entry_kind == "sponsor_liability_fired" (SL-c summary).
    let fired_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_fired"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(fired_count, 1, "1 sponsor_liability_fired summary entry");

    Ok(())
  }
}

mod v1_sl_e_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::governance_fixtures;
  use activitypub_federation::config::FederationConfig;
  use actix_web::web::{Data, Json};
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, insert_into, update};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    sponsor_liability_grace::run_grace_check_batch, submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, RevokeEndorsement, SubmitJuryVote,
  };
  use lemmy_api_crud::governance::revoke_endorsement::revoke_endorsement;
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::{
    newtypes::{EndorsementId, ModerationCaseId},
    source::governance::{
      endorsement::EndorsementInsertForm, moderation_case::ModerationCaseInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    InstanceId, PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryDecision, SeverityTier},
    schema::{
      endorsement, governance_log, moderation_case, public_case_log, reputation_event, surety,
    },
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::LemmyResult;
  use serde_json::Value;

  /// Seed a sponsee + `sponsor_count` sponsors, each with an endorsement + surety row.
  /// Returns (sponsee_id, Vec<(sponsor_id, endorsement_id)>).
  async fn seed_target_with_sureties_and_endorsements(
    context: &Data<LemmyContext>,
    instance_id: InstanceId,
    conn: &mut AsyncPgConnection,
    sponsor_count: usize,
    prefix: &str,
  ) -> LemmyResult<(PersonId, Vec<(PersonId, EndorsementId)>)> {
    let (sponsee, _) =
      governance_fixtures::seed_user(context, instance_id, &format!("{prefix}_sponsee"), false)
        .await?;
    let mut sponsors = Vec::with_capacity(sponsor_count);
    for i in 0..sponsor_count {
      let (sponsor, _) =
        governance_fixtures::seed_user(context, instance_id, &format!("{prefix}_sp{i}"), false)
          .await?;
      let endo_id: EndorsementId = insert_into(endorsement::table)
        .values(EndorsementInsertForm {
          from_person_id: sponsor,
          to_person_id: sponsee,
          community_id: None,
        })
        .returning(endorsement::id)
        .get_result::<EndorsementId>(conn)
        .await?;
      insert_into(surety::table)
        .values(SuretyInsertForm {
          sponsor_id: sponsor,
          sponsored_id: sponsee,
          community_id: None,
        })
        .execute(conn)
        .await?;
      sponsors.push((sponsor, endo_id));
    }
    Ok((sponsee, sponsors))
  }

  async fn count_log_entries(conn: &mut AsyncPgConnection, kind: &str) -> LemmyResult<i64> {
    let n: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq(kind))
      .count()
      .get_result(conn)
      .await?;
    Ok(n)
  }

  async fn read_log_payload(
    conn: &mut AsyncPgConnection,
    kind: &str,
  ) -> LemmyResult<Option<Value>> {
    let payloads: Vec<Value> = governance_log::table
      .filter(governance_log::entry_kind.eq(kind))
      .order(governance_log::id.desc())
      .select(governance_log::payload)
      .limit(1)
      .load(conn)
      .await?;
    Ok(payloads.into_iter().next())
  }

  async fn drive_jury_to_quorum(
    context: &Data<LemmyContext>,
    federation_context: &activitypub_federation::config::Data<LemmyContext>,
    admin_view: LocalUserView,
    case_id: ModerationCaseId,
    decision: JuryDecision,
  ) -> LemmyResult<()> {
    let assign_resp = admin_assign_jury(
      Json(AdminAssignJury { case_id }),
      context.clone(),
      admin_view,
    )
    .await?
    .into_inner();

    for &juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }

    for &juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), juror_id).await?;
      let resp = submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision,
          rationale: None,
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?
      .into_inner();
      if resp.case_decided {
        break;
      }
    }

    Ok(())
  }

  // RAII guard for BREHON_DISABLE_GRACE_CHECK_JOB. Restores prior value on Drop,
  // covering Ok / Err / panic exit paths. Per CR cr-5 + Copilot copilot-1 on PR #127.
  struct GraceCheckDisableGuard {
    prev: Option<std::ffi::OsString>,
  }

  impl GraceCheckDisableGuard {
    fn set(value: &str) -> Self {
      let prev = std::env::var_os("BREHON_DISABLE_GRACE_CHECK_JOB");
      unsafe {
        std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", value);
      }
      Self { prev }
    }
  }

  impl Drop for GraceCheckDisableGuard {
    fn drop(&mut self) {
      unsafe {
        match self.prev.take() {
          Some(val) => std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", val),
          None => std::env::remove_var("BREHON_DISABLE_GRACE_CHECK_JOB"),
        }
      }
    }
  }

  #[tokio::test]
  async fn revocation_during_window_escapes_full_lane() -> LemmyResult<()> {
    let _guard = GraceCheckDisableGuard::set("1");

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;

    let federation_config = FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data((**context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await?;
    let federation_context = federation_config.to_request_data();

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (sponsee, sponsors) =
      seed_target_with_sureties_and_endorsements(&context, instance.id, &mut conn, 2, "sle1")
        .await?;
    let (sponsor2, endo2) = sponsors[1];

    let mut juror_ids = Vec::with_capacity(5);
    for i in 0..5_usize {
      let (id, _) =
        governance_fixtures::seed_user(&context, instance.id, &format!("sle1_juror{i}"), false)
          .await?;
      juror_ids.push(id);
    }
    let (_, admin_view) =
      governance_fixtures::seed_user(&context, instance.id, "sle1_admin", true).await?;

    super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &juror_ids).await?;

    let case_id: ModerationCaseId = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_e_test_revocation".to_string(),
        severity: CaseSeverity::High,
        severity_tier: Some(SeverityTier::Minor),
        status: CaseStatus::Open,
        threshold_score: 1,
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;

    // Drive #1: jury vote to quorum → SponsorLiabilityPending.
    let before_decisive = Utc::now();
    drive_jury_to_quorum(
      &context,
      &federation_context,
      admin_view,
      case_id,
      JuryDecision::SuspendCommunityMember,
    )
    .await?;
    let after_decisive = Utc::now();

    // --- Mid-window assertions ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (status, grace_expires_at): (CaseStatus, Option<DateTime<Utc>>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((moderation_case::status, moderation_case::grace_expires_at))
      .first(&mut conn)
      .await?;

    assert!(
      matches!(status, CaseStatus::SponsorLiabilityPending),
      "case must be SponsorLiabilityPending, got {status:?}"
    );
    let grace = grace_expires_at.expect("grace_expires_at set on Pending path");
    let grace_lower = before_decisive + Duration::hours(168);
    let grace_upper = after_decisive + Duration::hours(168) + Duration::seconds(1);
    assert!(
      grace >= grace_lower && grace <= grace_upper,
      "grace_expires_at in [before_decisive + 168h, after_decisive + 168h + 1s], got {grace:?}"
    );

    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_pending").await?,
      1,
      "1 sponsor_liability_pending entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "case_decided").await?,
      1,
      "1 case_decided entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sanction_created").await?,
      1,
      "1 sanction_created entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "endorsement_revoked").await?,
      0,
      "no endorsement_revoked yet"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "no sponsor_liability_escaped yet"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      0,
      "no sponsor_liability_fired yet"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      0,
      "no sponsor_liability_applied yet"
    );

    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_count, 0, "reputation_events deferred on Pending path");

    let plog_count: i64 = public_case_log::table.count().get_result(&mut conn).await?;
    assert_eq!(plog_count, 0, "public_case_log deferred on Pending path");

    // ADR-015 pseudonym checks on sponsor_liability_pending payload.
    let payload: Value = read_log_payload(&mut conn, "sponsor_liability_pending")
      .await?
      .expect("sponsor_liability_pending payload present");
    assert_eq!(
      payload["case_id"],
      serde_json::json!(case_id.0),
      "payload.case_id matches"
    );
    let target_psn = payload["target_pseudonym"]
      .as_str()
      .expect("target_pseudonym is a string");
    assert!(
      payload["target_pseudonym"].is_string(),
      "target_pseudonym is a string"
    );
    assert_ne!(
      target_psn,
      format!("{}", sponsee.0),
      "target_pseudonym != raw person_id (ADR-015)"
    );
    let sponsors_psns = payload["sponsors_pseudonyms"]
      .as_array()
      .expect("sponsors_pseudonyms is an array");
    assert_eq!(sponsors_psns.len(), 2, "2 sponsors in payload");

    // Drive #2: revoke sponsor2's endorsement during the grace window → escape.
    let sponsor2_view = LocalUserView::read_person(&mut context.pool(), sponsor2).await?;
    let t_revoke_start = Utc::now();
    let revoke_resp = revoke_endorsement(
      Json(RevokeEndorsement {
        endorsement_id: endo2,
        reason: "sl-e test revocation".to_string(),
      }),
      context.clone(),
      sponsor2_view,
    )
    .await?
    .into_inner();
    let t_revoke_end = Utc::now();

    assert_eq!(
      revoke_resp.endorsement_id, endo2,
      "revoke response endorsement_id matches"
    );
    assert!(
      revoke_resp.revoked_at >= t_revoke_start
        && revoke_resp.revoked_at <= t_revoke_end + Duration::seconds(1),
      "revoked_at in [t_revoke_start, now+1s], got {:?}",
      revoke_resp.revoked_at
    );
    assert!(
      revoke_resp
        .liability_chain_severed_for_cases
        .contains(&case_id),
      "case_id in liability_chain_severed_for_cases"
    );

    // --- Post-revocation assertions ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (status, liability_escape_reason): (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(&mut conn)
      .await?;

    assert!(
      matches!(status, CaseStatus::SponsorLiabilityEscaped),
      "case must be SponsorLiabilityEscaped, got {status:?}"
    );
    let escape_reason =
      liability_escape_reason.expect("liability_escape_reason set on escape path");
    assert_eq!(
      escape_reason["version"],
      serde_json::json!(1),
      "escape_reason.version == 1"
    );
    assert_eq!(
      escape_reason["reason"],
      serde_json::json!("sponsor_revoked"),
      "escape_reason.reason == sponsor_revoked"
    );
    assert!(
      escape_reason["actor_pseudonym"].is_string(),
      "actor_pseudonym is a string"
    );
    assert_eq!(
      escape_reason["endorsement_id"],
      serde_json::json!(endo2.0),
      "escape_reason.endorsement_id matches"
    );

    assert_eq!(
      count_log_entries(&mut conn, "endorsement_revoked").await?,
      1,
      "1 endorsement_revoked entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      1,
      "1 sponsor_liability_escaped entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_pending").await?,
      1,
      "sponsor_liability_pending unchanged at 1"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      0,
      "no sponsor_liability_fired"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      0,
      "no sponsor_liability_applied"
    );

    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_count, 0, "reputation_events still 0 after escape");

    let plog_count: i64 = public_case_log::table.count().get_result(&mut conn).await?;
    assert_eq!(plog_count, 0, "public_case_log still 0 after escape");

    // ADR-015 pseudonym checks on endorsement_revoked payload.
    let endo_payload: Value = read_log_payload(&mut conn, "endorsement_revoked")
      .await?
      .expect("endorsement_revoked payload present");
    assert!(
      endo_payload["revoker_pseudonym"].is_string(),
      "revoker_pseudonym is a string"
    );
    assert_ne!(
      endo_payload["revoker_pseudonym"].as_str().unwrap(),
      format!("{}", sponsor2.0).as_str(),
      "revoker_pseudonym != raw sponsor_id (ADR-015)"
    );
    assert!(
      endo_payload["target_pseudonym"].is_string(),
      "target_pseudonym is a string in endorsement_revoked"
    );

    // ADR-015 pseudonym checks on sponsor_liability_escaped payload.
    let escaped_payload: Value = read_log_payload(&mut conn, "sponsor_liability_escaped")
      .await?
      .expect("sponsor_liability_escaped payload present");
    assert!(
      escaped_payload["actor_pseudonym"].is_string(),
      "actor_pseudonym is a string in sponsor_liability_escaped"
    );
    assert_eq!(
      escaped_payload["reason"].as_str().unwrap(),
      "sponsor_revoked",
      "escaped payload reason == sponsor_revoked"
    );

    // Drive #3: scheduler tick — case is already SponsorLiabilityEscaped → batch skips.
    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(
      outcome.cases_processed, 0_usize,
      "scheduler skips already-escaped case"
    );
    assert_eq!(outcome.fired, 0_usize, "no fires");
    assert_eq!(outcome.escaped, 0_usize, "no escapes from scheduler");
    assert_eq!(outcome.skipped, 0_usize, "no skips");

    // Final state unchanged.
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let final_status: CaseStatus = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select(moderation_case::status)
      .first(&mut conn)
      .await?;
    assert!(
      matches!(final_status, CaseStatus::SponsorLiabilityEscaped),
      "final status still SponsorLiabilityEscaped, got {final_status:?}"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      0,
      "no sponsor_liability_fired after scheduler tick"
    );

    Ok(())
  }

  #[tokio::test]
  async fn window_expiry_fires_full_lane() -> LemmyResult<()> {
    let _guard = GraceCheckDisableGuard::set("1");

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;

    let federation_config = FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data((**context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await?;
    let federation_context = federation_config.to_request_data();

    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (sponsee, _sponsors) =
      seed_target_with_sureties_and_endorsements(&context, instance.id, &mut conn, 2, "sle2")
        .await?;

    let mut juror_ids = Vec::with_capacity(5);
    for i in 0..5_usize {
      let (id, _) =
        governance_fixtures::seed_user(&context, instance.id, &format!("sle2_juror{i}"), false)
          .await?;
      juror_ids.push(id);
    }
    let (_, admin_view) =
      governance_fixtures::seed_user(&context, instance.id, "sle2_admin", true).await?;

    super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &juror_ids).await?;

    let case_id: ModerationCaseId = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_e_test_expiry".to_string(),
        severity: CaseSeverity::Low,
        severity_tier: Some(SeverityTier::Minor),
        status: CaseStatus::Open,
        threshold_score: 1,
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;

    // Drive jury to quorum → SponsorLiabilityPending.
    drive_jury_to_quorum(
      &context,
      &federation_context,
      admin_view,
      case_id,
      JuryDecision::SuspendCommunityMember,
    )
    .await?;

    // --- Mid-window assertions: Pending state ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let status: CaseStatus = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select(moderation_case::status)
      .first(&mut conn)
      .await?;
    assert!(
      matches!(status, CaseStatus::SponsorLiabilityPending),
      "case must be SponsorLiabilityPending, got {status:?}"
    );

    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_pending").await?,
      1,
      "1 sponsor_liability_pending entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      0,
      "no sponsor_liability_fired yet"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      0,
      "no sponsor_liability_applied yet"
    );

    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_count, 0, "reputation_events deferred on Pending path");

    let plog_count: i64 = public_case_log::table.count().get_result(&mut conn).await?;
    assert_eq!(plog_count, 0, "public_case_log deferred on Pending path");

    // ADR-015 pseudonym check on sponsor_liability_pending payload.
    let payload: Value = read_log_payload(&mut conn, "sponsor_liability_pending")
      .await?
      .expect("sponsor_liability_pending payload present");
    let target_psn = payload["target_pseudonym"]
      .as_str()
      .expect("target_pseudonym is a string");
    assert!(
      payload["target_pseudonym"].is_string(),
      "target_pseudonym is a string"
    );
    assert_ne!(
      target_psn,
      format!("{}", sponsee.0),
      "target_pseudonym != raw person_id (ADR-015)"
    );

    // Force-rewind grace_expires_at to past so the scheduler picks up the case.
    update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::grace_expires_at.eq(Some(Utc::now() - Duration::minutes(1))))
      .execute(&mut conn)
      .await?;

    // Fire the scheduler — grace window has expired, no revocation occurred.
    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(outcome.cases_processed, 1, "1 case processed");
    assert_eq!(outcome.fired, 1, "fire branch: 1 case fired");
    assert_eq!(outcome.escaped, 0, "fire branch: 0 escaped");
    assert_eq!(outcome.skipped, 0, "fire branch: 0 skipped");

    // --- Post-fire assertions ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (case_status, escape_reason): (CaseStatus, Option<Value>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::liability_escape_reason,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_status,
      CaseStatus::SponsorLiabilityFired,
      "case transitioned to SponsorLiabilityFired"
    );
    assert!(
      escape_reason.is_none(),
      "fire branch: liability_escape_reason IS NULL"
    );

    let rep_event_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_event_count, 2,
      "2 reputation_event rows (1 per sponsor)"
    );

    let plog_count_after: i64 = public_case_log::table.count().get_result(&mut conn).await?;
    assert_eq!(
      plog_count_after, 0,
      "fire path does not write public_case_log"
    );

    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      1,
      "1 sponsor_liability_fired summary"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      2,
      "2 sponsor_liability_applied entries (1 per sponsor)"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "0 sponsor_liability_escaped entries"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_pending").await?,
      1,
      "sponsor_liability_pending unchanged at 1"
    );

    // ADR-015 pseudonym checks on sponsor_liability_fired payload.
    let fired_payload = read_log_payload(&mut conn, "sponsor_liability_fired")
      .await?
      .expect("fired payload exists");
    assert!(
      fired_payload["target_pseudonym"].is_string(),
      "target_pseudonym is a string"
    );
    assert_ne!(
      fired_payload["target_pseudonym"].as_str().unwrap(),
      format!("{}", sponsee.0).as_str(),
      "target_pseudonym != raw person_id (ADR-015)"
    );
    assert_eq!(
      fired_payload["sponsor_count"].as_u64(),
      Some(2),
      "sponsor_count == 2"
    );
    assert_eq!(
      fired_payload["case_id"].as_i64(),
      Some(i64::from(case_id.0)),
      "case_id matches"
    );

    Ok(())
  }

  #[tokio::test]
  async fn backfill_of_mid_flight_v0_to_v1_deploy() -> LemmyResult<()> {
    use diesel::sql_query;
    use lemmy_db_schema::source::governance::sanction::SanctionInsertForm;
    use lemmy_db_schema_file::{
      enums::{SanctionAction, SanctionScope},
      schema::sanction,
    };

    let _guard = GraceCheckDisableGuard::set("1");

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = lemmy_db_schema::source::instance::Instance::read_or_create(
      &mut context.pool(),
      "test.invalid",
    )
    .await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    // Seed sponsee + 1 sponsor (single-sponsor minimal seed for backfill scenario).
    let (sponsee, sponsors) =
      seed_target_with_sureties_and_endorsements(&context, instance.id, &mut conn, 1, "sle3")
        .await?;
    let (sponsor, _endo) = sponsors[0];

    // Insert pre-deploy Decided case (simulating v0 mid-flight at v1 deploy time).
    let case_id: ModerationCaseId = insert_into(moderation_case::table)
      .values(ModerationCaseInsertForm {
        target_type: CaseTargetType::Person,
        target_person_id: Some(sponsee),
        reason_code: "v1_sl_e_test_backfill".to_string(),
        severity: CaseSeverity::Medium,
        severity_tier: Some(SeverityTier::Minor),
        status: CaseStatus::Decided,
        threshold_score: 1,
        ..Default::default()
      })
      .returning(moderation_case::id)
      .get_result::<ModerationCaseId>(&mut conn)
      .await?;

    // Set decided_at via UPDATE post-insert (not in InsertForm — mirror SL-c-2 pattern).
    // decided_at = now - 23h30m → grace_expires_at = now + 30m post-backfill (future).
    let decided_at = Utc::now() - Duration::hours(23) - Duration::minutes(30);
    update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::decided_at.eq(Some(decided_at)))
      .execute(&mut conn)
      .await?;

    // Insert sanction (§8.4 WHERE clause requirement: NOT EXISTS guard on reputation_event).
    insert_into(sanction::table)
      .values(SanctionInsertForm {
        case_id,
        scope: SanctionScope::Community,
        action: SanctionAction::ContentRemoval,
        target_person_id: Some(sponsee),
        active: Some(true),
        ..Default::default()
      })
      .execute(&mut conn)
      .await?;

    // --- Pre-backfill assertions: verify seed satisfies §8.4 WHERE clause ---
    let (pre_status, pre_decided_at, pre_target_pid): (
      CaseStatus,
      Option<DateTime<Utc>>,
      Option<PersonId>,
    ) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::decided_at,
        moderation_case::target_person_id,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      pre_status,
      CaseStatus::Decided,
      "pre-backfill: status is Decided"
    );
    assert!(pre_decided_at.is_some(), "pre-backfill: decided_at is Some");
    assert!(
      pre_decided_at.unwrap() > Utc::now() - Duration::hours(24),
      "pre-backfill: decided_at within 24h window"
    );
    assert_eq!(
      pre_target_pid,
      Some(sponsee),
      "pre-backfill: target_person_id is sponsee"
    );

    let surety_count: i64 = surety::table
      .filter(surety::sponsored_id.eq(sponsee))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(surety_count, 1, "pre-backfill: 1 active surety for sponsee");

    let sanction_count: i64 = sanction::table
      .filter(sanction::case_id.eq(case_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(sanction_count, 1, "pre-backfill: 1 sanction row for case");

    let rep_guard_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .filter(reputation_event::reason.eq("sponsor_liability_applied"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_guard_count, 0,
      "pre-backfill: 0 reputation_events (§8.4 NOT EXISTS guard satisfied)"
    );

    // --- Drive #1: PRD §8.4 backfill UPDATE (verbatim SQL) ---
    // Per PRD §8.4 — verbatim. Comment cites the source location.
    let _affected = sql_query(
      "UPDATE moderation_case
       SET status = 'SponsorLiabilityPending',
           grace_expires_at = decided_at + INTERVAL '24 hours'
       WHERE status = 'Decided'
         AND decided_at IS NOT NULL
         AND decided_at > now() - INTERVAL '24 hours'
         AND target_person_id IS NOT NULL
         AND id IN (
           SELECT mc.id
           FROM moderation_case mc
           WHERE EXISTS (
             SELECT 1 FROM surety s
             WHERE s.sponsored_id = mc.target_person_id
               AND s.revoked_at IS NULL
           )
           AND EXISTS (
             SELECT 1 FROM sanction sa
             WHERE sa.case_id = mc.id
           )
           AND NOT EXISTS (
             SELECT 1 FROM reputation_event re
             WHERE re.source_case_id = mc.id
               AND re.reason = 'sponsor_liability_applied'
           )
         )",
    )
    .execute(&mut conn)
    .await?;

    // --- Post-backfill assertions (Phase A: backfill set Pending + future grace) ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let (post_backfill_status, post_grace): (CaseStatus, Option<DateTime<Utc>>) =
      moderation_case::table
        .filter(moderation_case::id.eq(case_id))
        .select((moderation_case::status, moderation_case::grace_expires_at))
        .first(&mut conn)
        .await?;
    assert_eq!(
      post_backfill_status,
      CaseStatus::SponsorLiabilityPending,
      "post-backfill: status == SponsorLiabilityPending"
    );
    let expected_grace = decided_at + Duration::hours(24);
    let actual_grace = post_grace.expect("grace_expires_at is Some");
    let diff_ms = (actual_grace - expected_grace).num_milliseconds().abs();
    assert!(
      diff_ms < 1000,
      "post-backfill: grace_expires_at == decided_at + 24h (within 1s, diff={diff_ms}ms)"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_pending").await?,
      0,
      "post-backfill: 0 governance_log entries (backfill UPDATE writes no logs)"
    );
    let rep_after_backfill: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_after_backfill, 0,
      "post-backfill: 0 reputation_event rows (no scheduler fire yet)"
    );

    // --- Drive #2: force-rewind grace_expires_at to past ---
    // Test artifact per PRD §8.4 timing note: decided_at + 24h = now + 30m is in
    // the future; rewind so the scheduler picks up the case immediately.
    update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::grace_expires_at.eq(Some(Utc::now() - Duration::minutes(1))))
      .execute(&mut conn)
      .await?;

    // --- Drive #3: scheduler tick (SL-c fires the backfilled case) ---
    let outcome = run_grace_check_batch(&context).await?;
    assert_eq!(outcome.cases_processed, 1, "1 case processed");
    assert_eq!(outcome.fired, 1, "fire branch: 1 case fired");
    assert_eq!(outcome.escaped, 0, "fire branch: 0 escaped");
    assert_eq!(outcome.skipped, 0, "fire branch: 0 skipped");

    // --- Post-fire assertions (Phase B: scheduler resolves backfilled case) ---
    let mut conn = AsyncPgConnection::establish(&db_url).await?;

    let post_fire_status: CaseStatus = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select(moderation_case::status)
      .first(&mut conn)
      .await?;
    assert_eq!(
      post_fire_status,
      CaseStatus::SponsorLiabilityFired,
      "post-fire: case transitioned to SponsorLiabilityFired"
    );

    let rep_event_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_event_count, 1, "1 reputation_event row (1 sponsor)");

    let plog_count: i64 = public_case_log::table.count().get_result(&mut conn).await?;
    assert_eq!(plog_count, 0, "fire path does not write public_case_log");

    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_applied").await?,
      1,
      "1 sponsor_liability_applied entry"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_fired").await?,
      1,
      "1 sponsor_liability_fired summary"
    );
    assert_eq!(
      count_log_entries(&mut conn, "sponsor_liability_escaped").await?,
      0,
      "0 sponsor_liability_escaped entries"
    );

    // ADR-015 pseudonym discipline on sponsor_liability_applied payload.
    let applied_payload = read_log_payload(&mut conn, "sponsor_liability_applied")
      .await?
      .expect("sponsor_liability_applied payload exists");
    assert!(
      applied_payload["sponsor_pseudonym"].is_string(),
      "sponsor_pseudonym is a string"
    );
    assert_ne!(
      applied_payload["sponsor_pseudonym"].as_str().unwrap(),
      format!("{}", sponsor.0).as_str(),
      "sponsor_pseudonym != raw person_id (ADR-015)"
    );

    Ok(())
  }
}

#[path = "federation_inbound_a.rs"]
mod v1_federation_inbound_a_fixtures;

mod v1_ship_2_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::governance_fixtures;
  use actix_web::{App, test, web::Data};
  use chrono::{Duration, Utc};
  use diesel::ExpressionMethods;
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api_common::governance::{
    CreateEndorsementResponse, GetMyReputationResponse, RequestAppealResponse,
  };
  use lemmy_api_utils::{claims::Claims, context::LemmyContext};
  use lemmy_db_schema::{
    newtypes::LocalUserId,
    source::{
      instance::Instance,
      governance::{
        moderation_case::ModerationCaseInsertForm,
        public_case_log::PublicCaseLogInsertForm,
      },
    },
  };
  use lemmy_db_schema_file::{
    enums::{CaseSeverity, CaseStatus, CaseTargetType},
    schema::moderation_case,
  };
  use lemmy_db_views_governance_modlog::GovernanceModlogView;
  use lemmy_routes::middleware::session::SessionMiddleware;
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit};

  async fn mint_jwt(ctx: &LemmyContext, local_user_id: LocalUserId) -> LemmyResult<String> {
    let req = test::TestRequest::default().to_http_request();
    let token = Claims::generate(local_user_id, None, req, ctx).await?;
    Ok(token.into_inner())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn request_appeal_happy_path_and_auth_failure() -> LemmyResult<()> {
    use lemmy_db_schema::newtypes::ModerationCaseId;

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (target_pid, target_lu_view) =
      governance_fixtures::seed_user(&context, instance.id, "ship2_appeal_target", false).await?;

    let case_id: ModerationCaseId = {
      let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
      let decided_form = ModerationCaseInsertForm {
        community_id: None,
        creator_id: None,
        target_type: CaseTargetType::RemoteInstance,
        target_post_id: None,
        target_comment_id: None,
        target_person_id: Some(target_pid),
        target_community_id: None,
        target_remote_url: None,
        reason_code: "v1_ship_2_appeal_probe".to_string(),
        severity: CaseSeverity::Low,
        status: CaseStatus::Decided,
        threshold_score: 1,
        ..Default::default()
      };
      let id: ModerationCaseId = diesel::insert_into(moderation_case::table)
        .values(&decided_form)
        .returning(moderation_case::id)
        .get_result(&mut async_conn)
        .await?;
      let future = Utc::now() + Duration::days(7);
      diesel::update(moderation_case::table)
        .filter(moderation_case::id.eq(id))
        .set((
          moderation_case::appeal_window_expires_at.eq(Some(future)),
          moderation_case::panel_size_snapshot.eq(Some(5_i32)),
        ))
        .execute(&mut async_conn)
        .await?;
      id
    };

    let target_jwt = mint_jwt(&context, target_lu_view.local_user.id).await?;

    let rate_limit = RateLimit::with_debug_config();
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
    let app = test::init_service(
      App::new()
        .app_data(Data::new((**context).clone()))
        .wrap(SessionMiddleware::new((**context).clone()))
        .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
    )
    .await;

    let resp = test::TestRequest::post()
      .uri("/api/v4/governance/appeal")
      .insert_header(("authorization", format!("Bearer {target_jwt}")))
      .insert_header(("content-type", "application/json"))
      .set_payload(format!(
        r#"{{"case_id":{},"reason":"v1_ship_2 appeal probe"}}"#,
        case_id.0
      ))
      .send_request(&app)
      .await;
    assert_eq!(
      resp.status().as_u16(),
      200,
      "appeal expected 200 for authed target on Decided case"
    );
    let body: RequestAppealResponse = test::read_body_json(resp).await;
    assert!(body.appeal_id.0 > 0, "appeal_id must be positive");
    assert_eq!(body.case_id, case_id, "case_id must match inserted case");

    let resp = test::TestRequest::post()
      .uri("/api/v4/governance/appeal")
      .insert_header(("content-type", "application/json"))
      .set_payload(format!(
        r#"{{"case_id":{},"reason":"v1_ship_2 appeal probe"}}"#,
        case_id.0
      ))
      .send_request(&app)
      .await;
    assert_eq!(
      resp.status().as_u16(),
      401,
      "appeal expected 401 for unauthenticated request"
    );

    Ok(())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn list_governance_modlog_returns_seeded_entry() -> LemmyResult<()> {
    use lemmy_db_schema::newtypes::ModerationCaseId;
    use lemmy_db_schema_file::schema::public_case_log;

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    let case_form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::RemoteInstance,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: None,
      target_community_id: None,
      target_remote_url: Some("https://example.invalid/modlog-test".to_string()),
      reason_code: "v1_ship_2_modlog_probe".to_string(),
      severity: CaseSeverity::Low,
      status: CaseStatus::Decided,
      threshold_score: 1,
      ..Default::default()
    };
    let case_id: ModerationCaseId = diesel::insert_into(moderation_case::table)
      .values(&case_form)
      .returning(moderation_case::id)
      .get_result(&mut async_conn)
      .await?;

    let log_form = PublicCaseLogInsertForm {
      case_id,
      community_id: None,
      summary: "v1_ship_2 modlog probe — no identifiers".to_string(),
      rationale_redacted: None,
    };
    diesel::insert_into(public_case_log::table)
      .values(&log_form)
      .execute(&mut async_conn)
      .await?;

    let rate_limit = RateLimit::with_debug_config();
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
    let app = test::init_service(
      App::new()
        .app_data(Data::new((**context).clone()))
        .wrap(SessionMiddleware::new((**context).clone()))
        .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
    )
    .await;

    let resp = test::TestRequest::get()
      .uri("/api/v4/governance/modlog")
      .send_request(&app)
      .await;
    assert_eq!(
      resp.status().as_u16(),
      200,
      "modlog expected 200 without auth"
    );
    let body: Vec<GovernanceModlogView> = test::read_body_json(resp).await;
    assert_eq!(body.len(), 1, "expected exactly one modlog entry");
    assert_eq!(
      body[0].case_id,
      case_id.0,
      "case_id must match inserted case"
    );
    assert_eq!(
      body[0].summary,
      "v1_ship_2 modlog probe — no identifiers",
      "summary must round-trip unchanged"
    );
    assert!(!body[0].appealed, "newly seeded case has no appeal");

    Ok(())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn get_my_reputation_happy_path_and_no_auth() -> LemmyResult<()> {
    let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (_user_pid, user_lu_view) =
      governance_fixtures::seed_user(&context, instance.id, "ship2_rep_user", false).await?;
    let user_jwt = mint_jwt(&context, user_lu_view.local_user.id).await?;

    let rate_limit = RateLimit::with_debug_config();
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
    let app = test::init_service(
      App::new()
        .app_data(Data::new((**context).clone()))
        .wrap(SessionMiddleware::new((**context).clone()))
        .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
    )
    .await;

    // Happy path: authed GET /reputation/me — load_or_compute_snapshot writes a
    // default snapshot row for a first-time caller; fresh user has no sanction
    // rows so active_sanctions == 0. Mirror of sweep test e2e.rs:4326-4337.
    let resp = test::TestRequest::get()
      .uri("/api/v4/governance/reputation/me")
      .insert_header(("authorization", format!("Bearer {user_jwt}")))
      .send_request(&app)
      .await;
    assert_eq!(
      resp.status().as_u16(),
      200,
      "reputation/me expected 200 for authed user"
    );
    let body: GetMyReputationResponse = test::read_body_json(resp).await;
    assert_eq!(
      body.view.active_sanctions,
      0,
      "fresh user must have zero active sanctions"
    );

    // Failure mode: GET /reputation/me without Authorization header → 401.
    // LocalUserView extractor rejects missing JWT before the handler runs.
    let resp = test::TestRequest::get()
      .uri("/api/v4/governance/reputation/me")
      .send_request(&app)
      .await;
    assert_eq!(
      resp.status().as_u16(),
      401,
      "reputation/me expected 401 for unauthenticated request"
    );

    Ok(())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn create_endorsement_happy_path_and_self_endorse_rejects() -> LemmyResult<()> {
    use lemmy_db_schema::source::governance::governance_config::GovernanceConfigInsertForm;
    use lemmy_db_schema_file::schema::governance_config;

    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (sponsor_pid, sponsor_lu_view) =
      governance_fixtures::seed_user(&context, instance.id, "ship2_endorse_sponsor", false).await?;
    let (sponsee_pid, _sponsee_lu_view) =
      governance_fixtures::seed_user(&context, instance.id, "ship2_endorse_sponsee", false).await?;

    // Insert "open" strategy row — fetch_value_at_scope uses ORDER BY valid_from DESC
    // LIMIT 1, so this row (DEFAULT now()) wins over the seed row (2026-04-18T00:00:00Z),
    // bypassing enforce_age_gate before the self-endorse check at line 178.
    {
      let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
      diesel::insert_into(governance_config::table)
        .values(&GovernanceConfigInsertForm {
          scope: "instance".to_string(),
          key: "onboarding.sponsor_gate_strategy".to_string(),
          value_type: "text".to_string(),
          value_text: Some("open".to_string()),
          ..Default::default()
        })
        .execute(&mut async_conn)
        .await?;
    }

    let sponsor_jwt = mint_jwt(&context, sponsor_lu_view.local_user.id).await?;

    let rate_limit = RateLimit::with_debug_config();
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
    let app = test::init_service(
      App::new()
        .app_data(Data::new((**context).clone()))
        .wrap(SessionMiddleware::new((**context).clone()))
        .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
    )
    .await;

    // Happy path: sponsor endorses sponsee (distinct users, "open" gate) → 200.
    let resp = test::TestRequest::post()
      .uri("/api/v4/governance/endorsement")
      .insert_header(("authorization", format!("Bearer {sponsor_jwt}")))
      .insert_header(("content-type", "application/json"))
      .set_payload(format!(r#"{{"person_id":{}}}"#, sponsee_pid.0))
      .send_request(&app)
      .await;
    assert_eq!(
      resp.status().as_u16(),
      200,
      "endorsement expected 200 for distinct sponsor/sponsee"
    );
    let body: CreateEndorsementResponse = test::read_body_json(resp).await;
    assert!(body.endorsement_id.0 > 0, "endorsement_id must be positive");
    assert!(
      body.surety_created,
      "surety_created must be true for fresh sponsee with no active sureties"
    );

    // Failure mode: self-endorsement → 404 (per DQ a3d0e9941441-007).
    // Self-endorse check (create_endorsement.rs:178) fires before the cooldown
    // check (line 195), so the happy-path endorsement above does not interfere.
    let resp = test::TestRequest::post()
      .uri("/api/v4/governance/endorsement")
      .insert_header(("authorization", format!("Bearer {sponsor_jwt}")))
      .insert_header(("content-type", "application/json"))
      .set_payload(format!(r#"{{"person_id":{}}}"#, sponsor_pid.0))
      .send_request(&app)
      .await;
    assert_eq!(
      resp.status().as_u16(),
      404,
      "self-endorsement expected 404 (LemmyErrorType::NotFound at create_endorsement.rs:179)"
    );

    Ok(())
  }
}

#[tokio::test(flavor = "multi_thread")]
async fn agpl_source_disclosure_surface_returns_notice() -> lemmy_utils::error::LemmyResult<()> {
  use activitypub_federation::config::{FederationConfig, FederationMiddleware};
  use actix_web::{App, test, web::Data};
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::source::{
    instance::Instance,
    local_site::{LocalSite, LocalSiteInsertForm},
    local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
    person::{Person, PersonInsertForm},
    site::{Site, SiteInsertForm},
  };
  use lemmy_db_views_site::api::{GetSiteResponse, GetSourceResponse};
  use lemmy_diesel_utils::traits::Crud;
  use lemmy_routes::middleware::idempotency::{IdempotencyMiddleware, IdempotencySet};
  use lemmy_routes::middleware::session::SessionMiddleware;
  use lemmy_utils::rate_limit::RateLimit;
  use std::ops::Deref;

  // ------------------- 1. testcontainer + AGPL surface seed (fix-impl-6 Part B PRESERVED) -------------------
  let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;

  // Seed instance + Site + LocalSite + LocalSiteRateLimit so `SiteView::read_local`
  // (called by `read_site` for GET /api/v4/site) returns a row instead of
  // LocalSiteNotSetup -> HTTP 500. Mirrors the canonical scaffold at e2e.rs:4751-4761
  // (governance_outbox_emits_remote_sanction_notice_on_local_sanction).
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  {
    let pool = &mut context.pool();
    let site_key_pair = activitypub_federation::http_signatures::generate_actor_keypair()?;
    let site_form = SiteInsertForm {
      ap_id: Some(url::Url::parse("https://test.invalid")?.into()),
      last_refreshed_at: Some(chrono::Utc::now()),
      inbox_url: Some(url::Url::parse("https://test.invalid/inbox")?.into()),
      private_key: Some(site_key_pair.private_key),
      public_key: Some(site_key_pair.public_key),
      ..SiteInsertForm::new("agpl test site".to_string(), instance.id)
    };
    let site = Site::create(pool, &site_form).await?;
    // System account: throwaway Person — LocalSite needs a non-null FK.
    let sysacct_form = PersonInsertForm::test_form(instance.id, "agpl_sysacct");
    let sysacct = Person::create(pool, &sysacct_form).await?;
    let local_site_form = LocalSiteInsertForm::new(site.id, sysacct.id);
    let local_site = LocalSite::create(pool, &local_site_form).await?;
    LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site.id)).await?;
  }

  // ------------------- 2. federation_config + inner_context (mirrors lib.rs:228-241 + lib.rs:364 VERBATIM) -------------------
  // §10.5: build FederationConfig from the bootstrap context. `(**context).clone()`
  // derefs Data<LemmyContext> -> LemmyContext (via actix Data's Deref<Target=T>);
  // clone gives a fresh LemmyContext whose ActualDbPool is Arc-shared with the
  // bootstrap's pool — so the AGPL seed (written via context.pool() above) is
  // visible to handler reads (via the inner_context.pool() below).
  let federation_config = FederationConfig::builder()
    .domain((**context).settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;

  // §10.6: lib.rs:364 line-for-line mirror.
  // `FederationConfig<T>: Deref<Target=T>` (config.rs:264-270). `.deref().clone()` gives a
  // LemmyContext sharing the SAME pool as `federation_config.app_data`'s inner clone.
  let inner_context: LemmyContext = federation_config.deref().clone();
  let idempotency_set = IdempotencySet::default();

  // ------------------- 3. App composition (mirrors lib.rs:379-382 VERBATIM) -------------------
  let rate_limit = RateLimit::with_debug_config();
  let app = test::init_service(
    App::new()
      .app_data(Data::new(inner_context.clone())) // lib.rs:379 mirror — actix Data<LemmyContext>
      .wrap(FederationMiddleware::new(federation_config.clone())) // lib.rs:380 mirror
      .wrap(IdempotencyMiddleware::new(idempotency_set.clone())) // lib.rs:381 mirror
      .wrap(SessionMiddleware::new(inner_context.clone())) // lib.rs:382 mirror
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
  )
  .await;

  // ------------------- 4. GET /api/v4/site — assert source_disclosure block (fix-impl-6 Part A PRESERVED) -------------------
  let site_req = test::TestRequest::get().uri("/api/v4/site").to_request();
  let site_resp = test::call_service(&app, site_req).await;
  let site_status = site_resp.status().as_u16();
  let site_body_bytes = test::read_body(site_resp).await;
  assert_eq!(
    site_status,
    200,
    "/api/v4/site must return 200 — body: {}",
    String::from_utf8_lossy(&site_body_bytes)
  );
  let site_body: GetSiteResponse = serde_json::from_slice(&site_body_bytes)?;

  assert_eq!(
    site_body.source_disclosure.license, "AGPL-3.0",
    "source_disclosure.license must be 'AGPL-3.0' per ADR-011"
  );
  assert_eq!(
    site_body.source_disclosure.disclosure_url, "/api/v4/source",
    "source_disclosure.disclosure_url must point to /api/v4/source"
  );
  assert!(
    !site_body.source_disclosure.repo_url.is_empty(),
    "source_disclosure.repo_url must be non-empty"
  );
  assert!(
    !site_body.source_disclosure.fork_commit.is_empty(),
    "source_disclosure.fork_commit must be non-empty (build.rs default 'unknown' is acceptable)"
  );

  // ------------------- 5. GET /api/v4/source — assert AGPL notice body (fix-impl-6 Part A PRESERVED) -------------------
  let source_req = test::TestRequest::get().uri("/api/v4/source").to_request();
  let source_resp = test::call_service(&app, source_req).await;
  let source_status = source_resp.status().as_u16();
  let source_body_bytes = test::read_body(source_resp).await;
  assert_eq!(
    source_status,
    200,
    "/api/v4/source must return 200 — body: {}",
    String::from_utf8_lossy(&source_body_bytes)
  );
  let source_body: GetSourceResponse = serde_json::from_slice(&source_body_bytes)?;

  assert_eq!(source_body.license, "AGPL-3.0");
  assert!(
    source_body
      .notice
      .contains("GNU Affero General Public License"),
    "AGPL-NOTICE.md body must contain the canonical license name"
  );
  assert!(
    source_body.notice.len() > 100,
    "notice body must be substantive (got {} bytes)",
    source_body.notice.len()
  );

  Ok(())
}

// ============================================================================
// v1-AD-e — server-rendered HTML admin pages (Dashboard + Audit)
//
// Mirrored from v1-AD-d admin_dashboard tests at e2e.rs:7296-7343 (Case A:
// uniform LemmyResult<()>, all bare ?, per feedback_lemmy_error_no_std_error
// §"Case A"). Four tests:
//   - `admin_dashboard_html_returns_html_for_admin`  — 200 text/html + heading
//   - `admin_dashboard_html_forbidden_for_non_admin` — capability gate
//   - `admin_html_pages_flag_off_returns_404`        — html_pages_enabled=false
//   - `admin_audit_html_returns_html_for_admin`      — 200 text/html + EventSource
//   - `admin_audit_html_forbidden_for_non_admin`     — capability gate (audit)
//
// Handlers invoked directly (no in-process actix server needed; the handler
// returns LemmyResult<HttpResponse> and the response body is a buffered
// BoxBody accessible via try_into_bytes()).
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_html_returns_html_for_admin() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::{body::MessageBody, http::StatusCode};
  use lemmy_api::governance::admin_dashboard_html::admin_dashboard_html;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "ade_dash_admin", true).await?;

  let resp = admin_dashboard_html(context.clone(), admin_view).await?;
  assert_eq!(
    resp.status(),
    StatusCode::OK,
    "admin gets 200 from /dashboard/view"
  );
  assert!(
    resp
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or_default()
      .contains("text/html"),
    "Content-Type must contain text/html",
  );
  let body_str = String::from_utf8(
    resp
      .into_body()
      .try_into_bytes()
      .unwrap_or_default()
      .to_vec(),
  )?;
  assert!(
    body_str.contains("Governance Admin Dashboard"),
    "body must contain the stable dashboard page heading",
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_html_forbidden_for_non_admin() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_dashboard_html::admin_dashboard_html;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "ade_dash_nonadmin", false).await?;

  let result = admin_dashboard_html(context.clone(), user_view).await;
  let err = result.expect_err("non-admin must be rejected by is_admin()");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotAnAdmin),
    "expected NotAnAdmin, got {:?}",
    err.error_type,
  );

  Ok(())
}

/// R-html-3: when `governance.dashboard.html_pages_enabled` is set to `false`
/// at instance scope, both the dashboard and audit HTML routes return 404
/// (feature-off semantics — not 403, which would indicate an auth failure).
#[tokio::test(flavor = "multi_thread")]
async fn admin_html_pages_flag_off_returns_404() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::{http::StatusCode, web::Json};
  use lemmy_api::governance::{
    admin_config::admin_set_config,
    admin_dashboard_html::{admin_audit_html, admin_dashboard_html},
  };
  use lemmy_api_common::governance::AdminSetConfig;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "ade_flagoff_admin", true).await?;

  // Disable HTML pages via the existing config-write path (R-html-3: do not
  // raw-INSERT; use the handler that mirrors how the gate reads the key).
  admin_set_config(
    Json(AdminSetConfig {
      key: "governance.dashboard.html_pages_enabled".to_string(),
      value_type: "bool".to_string(),
      value: serde_json::json!(false),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "disable HTML pages for 404 test".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  // Both routes must return 404 when the feature flag is off (R-html-3).
  let resp_dash = admin_dashboard_html(context.clone(), admin_view.clone()).await?;
  assert_eq!(
    resp_dash.status(),
    StatusCode::NOT_FOUND,
    "/dashboard/view must return 404 when html_pages_enabled=false",
  );

  let resp_audit = admin_audit_html(context.clone(), admin_view).await?;
  assert_eq!(
    resp_audit.status(),
    StatusCode::NOT_FOUND,
    "/audit/view must return 404 when html_pages_enabled=false",
  );

  Ok(())
}

/// Story 2 structural check: the audit HTML page includes an EventSource
/// pointing at /audit/stream and wires both named-event listeners per the
/// admin_audit_stream.rs frame contract (plan §13 Task 4 GOTCHA — onmessage
/// fires only on unnamed events; addEventListener required for named events).
#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_html_returns_html_for_admin() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::{body::MessageBody, http::StatusCode};
  use lemmy_api::governance::admin_dashboard_html::admin_audit_html;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "ade_audit_admin", true).await?;

  let resp = admin_audit_html(context.clone(), admin_view).await?;
  assert_eq!(
    resp.status(),
    StatusCode::OK,
    "admin gets 200 from /audit/view"
  );
  assert!(
    resp
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or_default()
      .contains("text/html"),
    "Content-Type must contain text/html",
  );
  let body_str = String::from_utf8(
    resp
      .into_body()
      .try_into_bytes()
      .unwrap_or_default()
      .to_vec(),
  )?;
  assert!(
    body_str.contains("Governance Config Audit"),
    "body must contain the stable audit page heading",
  );
  // EventSource wiring: named-event listeners for both governance event kinds.
  assert!(
    body_str.contains("EventSource("),
    "audit page must instantiate an EventSource",
  );
  assert!(
    body_str.contains("admin_config_changed"),
    "audit page must wire the admin_config_changed event listener",
  );
  assert!(
    body_str.contains("admin_config_change_denied"),
    "audit page must wire the admin_config_change_denied event listener",
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_html_forbidden_for_non_admin() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_dashboard_html::admin_audit_html;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "ade_audit_nonadmin", false).await?;

  let result = admin_audit_html(context.clone(), user_view).await;
  let err = result.expect_err("non-admin must be rejected by is_admin()");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotAnAdmin),
    "expected NotAnAdmin, got {:?}",
    err.error_type,
  );

  Ok(())
}

#[path = "federation_inbound_b.rs"]
mod v1_federation_inbound_b_fixtures;

#[path = "federation_inbound_e.rs"]
mod v1_federation_inbound_e_fixtures;

mod v1_ship_3_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use crate::common::{EnvVarGuard, governance_fixtures};
  use actix_web::web::{Data, Json};
  use diesel::{Connection as _, ExpressionMethods, PgConnection, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    reputation_snapshot::recompute_snapshot, sponsor_liability_grace::run_grace_check_batch,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, CreateGovernanceReport, SubmitJuryVote,
  };
  use lemmy_api_crud::governance::create_report::create_report;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::{
      reputation_event::ReputationEventInsertForm,
      reputation_snapshot::ReputationSnapshotInsertForm,
      surety::SuretyInsertForm,
    },
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{
    InstanceId, PersonId,
    enums::{CaseStatus, CaseTargetType, JuryDecision, ReputationDimension},
    schema::{moderation_case, reputation_event, reputation_snapshot, surety},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests, get_conn},
    traits::Crud,
  };
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;

  #[tokio::test(flavor = "multi_thread")]
  async fn two_sponsors_lose_endorsement_strength_on_sanction() -> LemmyResult<()> {
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    let _g_gov = EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);

    let (_container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      governance_fixtures::apply_all_schema(&mut sync_conn)?;
    }

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

    let federation_config = activitypub_federation::config::FederationConfig::builder()
      .domain(context.settings().hostname.clone())
      .app_data((**context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await
      .map_err(|e| anyhow::anyhow!("{e}"))?;
    let federation_context = federation_config.to_request_data();

    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

    let community_form = CommunityInsertForm::new(
      instance.id,
      "ship3comm".to_string(),
      "Ship3 Community".to_string(),
      "comm-pubkey".to_string(),
    );
    let community = Community::create(&mut context.pool(), &community_form).await?;

    async fn seed_person(
      ctx: &LemmyContext,
      instance_id: InstanceId,
      name: &str,
      is_admin: bool,
    ) -> LemmyResult<PersonId> {
      let person_form = PersonInsertForm::test_form(instance_id, name);
      let person = Person::create(&mut ctx.pool(), &person_form).await?;
      let mut lu_form = if is_admin {
        LocalUserInsertForm::test_form_admin(person.id)
      } else {
        LocalUserInsertForm::test_form(person.id)
      };
      lu_form.accepted_application = Some(true);
      LocalUser::create(&mut ctx.pool(), &lu_form, vec![]).await?;
      Ok(person.id)
    }

    let admin = seed_person(&context, instance.id, "ship3_admin", true).await?;
    let reporter = seed_person(&context, instance.id, "ship3_reporter", false).await?;

    let mut jurors: Vec<PersonId> = Vec::new();
    for i in 0..6 {
      jurors.push(
        seed_person(
          &context,
          instance.id,
          &format!("ship3_juror_{i}"),
          false,
        )
        .await?,
      );
    }

    let admin_view = LocalUserView::read_person(&mut context.pool(), admin).await?;
    let reporter_view = LocalUserView::read_person(&mut context.pool(), reporter).await?;

    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    async fn seed_surety(
      conn: &mut AsyncPgConnection,
      sponsor: PersonId,
      sponsored: PersonId,
    ) -> LemmyResult<()> {
      let form = SuretyInsertForm {
        sponsor_id: sponsor,
        sponsored_id: sponsored,
        community_id: None,
      };
      diesel::insert_into(surety::table)
        .values(&form)
        .execute(conn)
        .await?;
      Ok(())
    }

    async fn seed_snapshot(
      conn: &mut AsyncPgConnection,
      person: PersonId,
      endorsement_strength: i32,
    ) -> LemmyResult<()> {
      let form = ReputationSnapshotInsertForm {
        person_id: person,
        community_id: None,
        reporting_accuracy: 0,
        jury_reliability: 0,
        participation_consistency: 0,
        endorsement_strength,
        jury_eligible: false,
        trusted_reporter: false,
        ..Default::default()
      };
      diesel::insert_into(reputation_snapshot::table)
        .values(&form)
        .execute(conn)
        .await?;
      Ok(())
    }

    // Seed the initial endorsement_strength as a reputation_event row so that
    // recompute_snapshot (which sums events, not the snapshot table) reflects
    // the starting balance. Mirrors seed_founder_events pattern at e2e.rs:3424.
    async fn seed_endorsement_event(
      conn: &mut AsyncPgConnection,
      person: PersonId,
      delta: i32,
    ) -> LemmyResult<()> {
      use chrono::{Duration as ChronoDuration, Utc};
      let expiry = Utc::now() + ChronoDuration::days(90);
      let form = ReputationEventInsertForm {
        person_id: person,
        community_id: None,
        dimension: ReputationDimension::EndorsementStrength,
        delta,
        source_case_id: None,
        source_report_id: None,
        reason: "test_seed".to_string(),
        expires_at: Some(expiry),
        dedupe_key: None,
        source_event_type: None,
      };
      diesel::insert_into(reputation_event::table)
        .values(&form)
        .execute(conn)
        .await?;
      Ok(())
    }

    #[expect(
      clippy::too_many_arguments,
      reason = "integration test helper orchestrates a full sanction round; all parameters are required"
    )]
    async fn run_sanction_scenario(
      context: &Data<LemmyContext>,
      federation_context: &activitypub_federation::config::Data<LemmyContext>,
      admin_view: &LocalUserView,
      reporter_view: &LocalUserView,
      jurors: &[PersonId],
      target: PersonId,
      community_id: lemmy_db_schema::newtypes::CommunityId,
      reason_code: &str,
      decision: JuryDecision,
    ) -> LemmyResult<i32> {
      let create_resp = create_report(
        Json(CreateGovernanceReport {
          community_id: Some(community_id),
          target_type: CaseTargetType::Person,
          target_id: target.0,
          reason_code: reason_code.to_string(),
          description: Some(format!("Test report for {reason_code}")),
        }),
        context.clone(),
        reporter_view.clone(),
      )
      .await?
      .into_inner();
      let case_id = create_resp
        .case_id
        .ok_or_else(|| anyhow::anyhow!("case_id missing"))?;

      {
        let mut pool = context.pool();
        let mut conn = get_conn(&mut pool).await?;
        diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id.0)))
          .set(moderation_case::status.eq(CaseStatus::ThresholdMet))
          .execute(&mut *conn)
          .await?;
      }

      let assign_resp = admin_assign_jury(
        Json(AdminAssignJury { case_id }),
        context.clone(),
        admin_view.clone(),
      )
      .await?
      .into_inner();
      assert_eq!(assign_resp.assigned_person_ids.len(), 5, "5 jurors assigned");

      for juror_id in &assign_resp.assigned_person_ids {
        let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
        accept_jury_assignment(
          Json(AcceptJuryAssignment { case_id }),
          context.clone(),
          juror_view,
        )
        .await?;
      }

      let voting: Vec<PersonId> = assign_resp
        .assigned_person_ids
        .iter()
        .copied()
        .take(3)
        .collect();
      for juror in &voting {
        let juror_view = LocalUserView::read_person(&mut context.pool(), *juror).await?;
        submit_jury_vote(
          Json(SubmitJuryVote {
            case_id,
            decision,
            rationale: Some("test".to_string()),
          }),
          federation_context.reset_request_count(),
          juror_view,
        )
        .await?;
      }

      let _ = jurors;
      Ok(case_id.0)
    }

    async fn liability_delta_for(
      conn: &mut AsyncPgConnection,
      person: PersonId,
      case_id: i32,
    ) -> LemmyResult<i32> {
      let delta: i32 = reputation_event::table
        .filter(reputation_event::person_id.eq(person))
        .filter(reputation_event::source_case_id.eq(case_id))
        .filter(reputation_event::reason.eq("sponsor_liability_applied"))
        .select(reputation_event::delta)
        .order_by(reputation_event::id.desc())
        .first(conn)
        .await?;
      Ok(delta)
    }

    let target = seed_person(&context, instance.id, "ship3_target", false).await?;
    let sponsor_1 = seed_person(&context, instance.id, "ship3_sponsor_1", false).await?;
    let sponsor_2 = seed_person(&context, instance.id, "ship3_sponsor_2", false).await?;

    seed_surety(&mut async_conn, sponsor_1, target).await?;
    seed_surety(&mut async_conn, sponsor_2, target).await?;
    seed_snapshot(&mut async_conn, sponsor_1, 10).await?;
    seed_snapshot(&mut async_conn, sponsor_2, 10).await?;
    // Seed backing events so recompute_snapshot reflects the initial 10.
    seed_endorsement_event(&mut async_conn, sponsor_1, 10).await?;
    seed_endorsement_event(&mut async_conn, sponsor_2, 10).await?;

    let case = run_sanction_scenario(
      &context,
      &federation_context,
      &admin_view,
      &reporter_view,
      &jurors,
      target,
      community.id,
      "ship3_moderate",
      JuryDecision::RemoveContent,
    )
    .await?;

    // v1-SL-d: submit_jury_vote transitions the case to SponsorLiabilityPending
    // (grace window) instead of firing liability immediately. Expire the grace
    // window and call run_grace_check_batch to trigger fire_sponsor_liability
    // so reputation_event rows exist for the assertions below.
    // Set grace_expires_at to 1s in the past (same pattern as v1-SL-c test at
    // e2e.rs:14288) so the batch filter `grace_expires_at <= now()` picks it up.
    diesel::sql_query(
      "UPDATE moderation_case \
       SET grace_expires_at = now() - interval '1 second' \
       WHERE id = $1",
    )
    .bind::<diesel::sql_types::Int4, _>(case)
    .execute(&mut async_conn)
    .await?;
    let batch_outcome = run_grace_check_batch(&context).await?;
    assert_eq!(batch_outcome.fired, 1, "grace batch fired 1 case");

    // Math: raw_delta = -50 (moderate), 2 sponsors → per_sponsor = -25,
    // remainder = 0. Non-founder → multiplier = regular_multiplier (1.0
    // default). post_multiplier_delta = -25. current = 10.
    // 10 + (-25) = -15 < floor(0) → clamp: final_delta = 0 - 10 = -10.
    // Final endorsement_strength = 10 + (-10) = 0.
    // Read from runtime config so the assertion survives future RT-r* tuning.
    let mut cache = lemmy_api::governance::config::ConfigCache::new();
    let floor: i64 = lemmy_api::governance::config::get_int(
      &mut cache,
      &mut (&mut async_conn).into(),
      lemmy_api::governance::config::Scope::Instance,
      "liability.sponsor_liability_floor",
    )
    .await?;
    let moderate_delta: i64 = lemmy_api::governance::config::get_int(
      &mut cache,
      &mut (&mut async_conn).into(),
      lemmy_api::governance::config::Scope::Instance,
      "deltas.sponsor_liability_moderate",
    )
    .await?;
    let initial_strength: i64 = 10;
    let per_sponsor_pre = moderate_delta / 2;
    let expected_clamped = std::cmp::max(per_sponsor_pre, floor - initial_strength);
    let expected_final_strength = initial_strength + expected_clamped;

    let d_1 = liability_delta_for(&mut async_conn, sponsor_1, case).await?;
    let d_2 = liability_delta_for(&mut async_conn, sponsor_2, case).await?;
    assert_eq!(
      i64::from(d_1),
      expected_clamped,
      "sponsor_1 delta clamped to floor"
    );
    assert_eq!(
      i64::from(d_2),
      expected_clamped,
      "sponsor_2 delta clamped to floor"
    );

    let snap_1 = recompute_snapshot(&mut async_conn, sponsor_1, None, &mut cache).await?;
    let snap_2 = recompute_snapshot(&mut async_conn, sponsor_2, None, &mut cache).await?;
    assert_eq!(
      i64::from(snap_1.endorsement_strength),
      expected_final_strength,
      "sponsor_1 final endorsement_strength = floor (0) by clamp"
    );
    assert_eq!(
      i64::from(snap_2.endorsement_strength),
      expected_final_strength,
      "sponsor_2 final endorsement_strength = floor (0) by clamp"
    );

    Ok(())
  }
}


// Reputation RT-r3 fixtures extracted to tests/e2e/reputation_rt_r3.rs (sub-phase 3/7).
