mod v1_jm_b_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use chrono::{Duration as ChronoDuration, Utc};
  use diesel::{Connection as _, PgConnection};
  use diesel_async::{AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::source::governance::{
    moderation_case::ModerationCaseInsertForm, reputation_event::ReputationEventInsertForm,
    reputation_snapshot::ReputationSnapshotInsertForm,
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, ReputationDimension, SeverityTier},
    schema::{moderation_case, reputation_event, reputation_snapshot},
  };
  use lemmy_utils::error::LemmyResult;

  /// Insert a case directly with the given severity_tier. `creator_id` is
  /// NULL so the admin is not excluded from the panel. `community_id` is
  /// NULL (instance-scope) so the eligibility query's
  /// `rs.community_id IS NOT DISTINCT FROM $1` matches any snapshot (or,
  /// under the small-pool fallback, ignores community scope entirely).
  #[expect(
    clippy::unused_async,
    reason = "callers .await this; body uses sync Diesel but signature must be async for call-site consistency"
  )]
  pub async fn seed_case(
    db_url: &str,
    target: PersonId,
    severity_tier: SeverityTier,
  ) -> lemmy_utils::error::LemmyResult<lemmy_db_schema::newtypes::ModerationCaseId> {
    let severity = match severity_tier {
      SeverityTier::Minor => CaseSeverity::Low,
      SeverityTier::Moderate => CaseSeverity::Medium,
      SeverityTier::Severe => CaseSeverity::High,
    };
    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::Person,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: Some(target),
      target_community_id: None,
      target_remote_url: None,
      reason_code: "v1_jm_b_test".to_string(),
      severity,
      severity_tier: Some(severity_tier),
      status: CaseStatus::Open,
      threshold_score: 1,
      ..Default::default()
    };
    let mut sync_conn = PgConnection::establish(db_url)?;
    let case_id: i32 = diesel::RunQueryDsl::get_result(
      diesel::insert_into(moderation_case::table)
        .values(&form)
        .returning(moderation_case::id),
      &mut sync_conn,
    )?;
    Ok(lemmy_db_schema::newtypes::ModerationCaseId(case_id))
  }

  /// Seed a `reputation_snapshot` row with `jury_eligible = true` for
  /// every person in `persons`. This is what unlocks the strict
  /// eligibility query — without it, admin_assign_jury's small-pool
  /// fallback fires and the constraint record records
  /// `relaxed_small_pool` + `legacy_fallback` instead of the steady-state
  /// `applied` values. Tests that assert on the constraint-record shape
  /// must seed these rows first.
  pub async fn seed_jury_eligible_snapshots(
    conn: &mut AsyncPgConnection,
    persons: &[PersonId],
  ) -> LemmyResult<()> {
    seed_jury_eligible_snapshots_scoped(conn, persons, None).await
  }

  /// Like [`seed_jury_eligible_snapshots`], but with explicit
  /// `community_id` scope. Required when the case being tested is
  /// community-scoped (e.g. emergency-remove with a `Some(community_id)`
  /// argument): the eligibility query joins
  /// `reputation_snapshot` on `community_id IS NOT DISTINCT FROM
  /// case.community_id`, so an instance-scoped (`NULL`) snapshot does
  /// not match a community-scoped case.
  pub async fn seed_jury_eligible_snapshots_scoped(
    conn: &mut AsyncPgConnection,
    persons: &[PersonId],
    community_id: Option<lemmy_db_schema::newtypes::CommunityId>,
  ) -> LemmyResult<()> {
    for person in persons {
      let form = ReputationSnapshotInsertForm {
        person_id: *person,
        community_id,
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
        .execute(conn)
        .await?;
    }
    Ok(())
  }

  /// Insert a `reputation_event` row with `reason = "founder_seed"` and an
  /// unexpired `expires_at`, matching the is_founder probe shape in
  /// `compute_status_tier` (mirror of sponsor_liability.rs:217-228).
  pub async fn seed_founder_event(
    conn: &mut AsyncPgConnection,
    person: PersonId,
  ) -> LemmyResult<()> {
    let expiry = Utc::now() + ChronoDuration::days(90);
    let form = ReputationEventInsertForm {
      person_id: person,
      community_id: None,
      dimension: ReputationDimension::EndorsementStrength,
      delta: 100,
      source_case_id: None,
      source_report_id: None,
      reason: "founder_seed".to_string(),
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
}

/// v1-JM-b Task 7 test 1 — regular/minor → panel_size 5, quorum 3,
/// threshold 3. Asserts both the handler's response count and the
/// `moderation_case.panel_size_snapshot / quorum_snapshot /
/// threshold_count_snapshot` row shape per plan §10.5.
#[tokio::test(flavor = "multi_thread")]
async fn admin_assign_jury_severity_tier_regular_minor_panel_5_jurors()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_assign_jury::admin_assign_jury;
  use lemmy_api_common::governance::AdminAssignJury;
  use lemmy_db_schema_file::{enums::SeverityTier, schema::moderation_case};

  use lemmy_db_schema::source::instance::Instance;
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_rm", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_rm", false).await?;
  let _jurors = governance_fixtures::seed_jurors(&context, instance.id, 9).await?;

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(
    resp.assigned_person_ids.len(),
    5,
    "regular/minor → panel_size = 5 (jury.panel_size.regular.minor seed)"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let snapshot: (Option<i32>, Option<i32>, Option<i32>) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::panel_size_snapshot,
      moderation_case::quorum_snapshot,
      moderation_case::threshold_count_snapshot,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(snapshot.0, Some(5), "panel_size_snapshot = 5");
  assert_eq!(snapshot.1, Some(3), "quorum_snapshot = ceil(5 × 0.6) = 3");
  assert_eq!(
    snapshot.2,
    Some(3),
    "threshold_count_snapshot = ceil(5 × 0.5001) = 3"
  );
  Ok(())
}

/// v1-JM-b Task 7 test 2 — regular/severe → panel_size 7, quorum 5,
/// threshold 6. Same cascade shape as test 1 with the higher severity
/// tier exercising a different branch of `jury.panel_size.<status>.<severity>`.
#[tokio::test(flavor = "multi_thread")]
async fn admin_assign_jury_severity_tier_regular_severe_panel_7_jurors()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_assign_jury::admin_assign_jury;
  use lemmy_api_common::governance::AdminAssignJury;
  use lemmy_db_schema_file::{enums::SeverityTier, schema::moderation_case};

  use lemmy_db_schema::source::instance::Instance;
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_rs", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_rs", false).await?;
  let _jurors = governance_fixtures::seed_jurors(&context, instance.id, 9).await?;

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Severe)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(
    resp.assigned_person_ids.len(),
    7,
    "regular/severe → panel_size = 7 (jury.panel_size.regular.severe seed)"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let snapshot: (Option<i32>, Option<i32>, Option<i32>) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::panel_size_snapshot,
      moderation_case::quorum_snapshot,
      moderation_case::threshold_count_snapshot,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(snapshot.0, Some(7), "panel_size_snapshot = 7");
  assert_eq!(snapshot.1, Some(5), "quorum_snapshot = ceil(7 × 0.71) = 5");
  assert_eq!(
    snapshot.2,
    Some(6),
    "threshold_count_snapshot = ceil(7 × 0.75) = 6"
  );
  Ok(())
}

/// v1-JM-b Task 7 test 3 — founder/severe → panel_size 9, quorum 7,
/// threshold 7. Target has a `reputation_event.reason = 'founder_seed'`
/// row with an unexpired `expires_at`, which `compute_status_tier`
/// resolves to `CaseStatusTier::Founder`. Asserts
/// `moderation_case.status_tier = Founder` is snapshotted alongside the
/// count fields (plan §10.5).
#[tokio::test(flavor = "multi_thread")]
async fn admin_assign_jury_severity_tier_founder_severe_panel_9_jurors()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_assign_jury::admin_assign_jury;
  use lemmy_api_common::governance::AdminAssignJury;
  use lemmy_db_schema_file::{
    enums::{CaseStatusTier, SeverityTier},
    schema::moderation_case,
  };

  use lemmy_db_schema::source::instance::Instance;
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_fs", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_fs", false).await?;
  let _jurors = governance_fixtures::seed_jurors(&context, instance.id, 9).await?;

  // Elevate target to Founder via reputation_event.
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_founder_event(&mut conn, target).await?;
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Severe)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(
    resp.assigned_person_ids.len(),
    9,
    "founder/severe → panel_size = 9 (jury.panel_size.founder.severe seed)"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let row: (Option<i32>, Option<i32>, Option<i32>, CaseStatusTier) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::panel_size_snapshot,
      moderation_case::quorum_snapshot,
      moderation_case::threshold_count_snapshot,
      moderation_case::status_tier,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(row.0, Some(9), "panel_size_snapshot = 9");
  assert_eq!(row.1, Some(7), "quorum_snapshot = ceil(9 × 0.71) = 7");
  assert_eq!(
    row.2,
    Some(7),
    "threshold_count_snapshot = ceil(9 × 0.75) = 7"
  );
  assert_eq!(
    row.3,
    CaseStatusTier::Founder,
    "status_tier snapshotted as Founder (compute_status_tier resolved the founder_seed event)"
  );
  Ok(())
}

/// v1-JM-b Task 7 test 4 — every `jury_assignment` row carries a non-null
/// `selected_under_constraints` JSONB with the PRD §5.1 four-axis status
/// shape. Seeds used: `no_majority_from_same_sponsor_cluster` = true,
/// `geographic_diversity_preferred` = true, `no_recent_juror_repeat` =
/// true, `no_same_endorsement_chain` = false.
#[tokio::test(flavor = "multi_thread")]
async fn admin_assign_jury_writes_selected_under_constraints_jsonb()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_assign_jury::admin_assign_jury;
  use lemmy_api_common::governance::AdminAssignJury;
  use lemmy_db_schema_file::{enums::SeverityTier, schema::jury_assignment};
  use serde_json::Value;

  use lemmy_db_schema::source::instance::Instance;
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_suc", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_suc", false).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 9).await?;

  // Seed reputation_snapshot rows so the strict eligibility path succeeds
  // and the constraint record reflects steady-state "applied" values
  // instead of the small-pool-fallback relaxation cascade.
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();
  assert_eq!(resp.assigned_person_ids.len(), 5, "5-juror panel expected");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let constraint_payloads: Vec<Option<Value>> = jury_assignment::table
    .filter(jury_assignment::case_id.eq(case_id))
    .select(jury_assignment::selected_under_constraints)
    .load(&mut conn)
    .await?;
  assert_eq!(
    constraint_payloads.len(),
    5,
    "exactly 5 jury_assignment rows for the case"
  );
  for (idx, payload) in constraint_payloads.iter().enumerate() {
    let value = payload
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("selected_under_constraints row {idx} is NULL"))?;
    assert_eq!(
      value["no_majority_from_same_sponsor_cluster"],
      Value::String("applied".to_string()),
      "row {idx} cluster constraint = applied"
    );
    assert_eq!(
      value["geographic_diversity_preferred"],
      Value::String("applied_soft".to_string()),
      "row {idx} geographic preference = applied_soft"
    );
    assert_eq!(
      value["no_recent_juror_repeat"],
      Value::String("applied".to_string()),
      "row {idx} juror cooldown = applied"
    );
    assert_eq!(
      value["no_same_endorsement_chain"],
      Value::String("disabled".to_string()),
      "row {idx} endorsement chain = disabled (seed flag = false)"
    );
  }
  Ok(())
}

/// v1-JM-b Task 7 test 5 — `severity_tier_frozen` governance_log entry
/// is emitted once per assign-jury call, with a payload naming the
/// resolved severity_tier slug + status_tier slug per plan §10.6.
#[tokio::test(flavor = "multi_thread")]
async fn admin_assign_jury_emits_severity_tier_frozen_governance_log()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_assign_jury::admin_assign_jury;
  use lemmy_api_common::governance::AdminAssignJury;
  use lemmy_db_schema_file::{enums::SeverityTier, schema::governance_log};
  use serde_json::Value;

  use lemmy_db_schema::source::instance::Instance;
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_stf", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_stf", false).await?;
  let _jurors = governance_fixtures::seed_jurors(&context, instance.id, 9).await?;

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let _ = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rows: Vec<Value> = governance_log::table
    .filter(governance_log::entry_kind.eq("severity_tier_frozen"))
    .select(governance_log::payload)
    .load(&mut conn)
    .await?;
  assert_eq!(
    rows.len(),
    1,
    "exactly one severity_tier_frozen entry per assign-jury"
  );
  let payload = rows
    .get(0)
    .ok_or_else(|| anyhow::anyhow!("no severity_tier_frozen row"))?;
  assert_eq!(
    payload["severity_tier"],
    Value::String("minor".to_string()),
    "severity_tier slug = minor"
  );
  assert_eq!(
    payload["status_tier"],
    Value::String("regular".to_string()),
    "status_tier slug = regular"
  );
  assert_eq!(payload["panel_size_snapshot"], Value::from(5));
  assert_eq!(payload["quorum_snapshot"], Value::from(3));
  assert_eq!(payload["threshold_count_snapshot"], Value::from(3));
  Ok(())
}

/// v1-JM-b Task 7 test 6 — cascade walks
/// `jury.panel_size.founder.severe` → `jury.panel_size.severe` →
/// bare `jury.panel_size` (DB rows). When all DB rows are absent, the
/// const-table fallback walks the SAME candidate list (most-specific
/// first) and returns the per-tier const
/// `DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE = 9`. Uses raw SQL to delete
/// config rows between walks since `governance_config` is append-only
/// via `valid_from` but has no DELETE-forbid trigger.
#[tokio::test(flavor = "multi_thread")]
async fn config_get_int_cascade_resolves_founder_severe_to_bare_then_const()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel::sql_query;
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::config::{ConfigCache, Scope, get_int_cascade};
  use lemmy_diesel_utils::connection::DbPool;

  let (_container, _context, db_url) = governance_fixtures::bootstrap().await?;
  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // ---- Step 1 — seed `jury.panel_size.severe = 7` then assert cascade
  //      with no `jury.panel_size.founder.severe` returns 7 (fallback to
  //      the bare severity key).
  sql_query(
    "DELETE FROM governance_config \
     WHERE scope = 'instance' AND key = 'jury.panel_size.founder.severe'",
  )
  .execute(&mut conn)
  .await?;
  sql_query(
    "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
     VALUES ('instance', 'jury.panel_size.severe', 'int', 7, now())",
  )
  .execute(&mut conn)
  .await?;

  {
    let mut cache = ConfigCache::new();
    let mut pool: DbPool<'_> = (&mut conn).into();
    let got = get_int_cascade(
      &mut cache,
      &mut pool,
      Scope::Instance,
      "jury.panel_size",
      &["founder", "severe"],
    )
    .await?;
    assert_eq!(
      got, 7,
      "cascade resolves founder.severe → severe when per-tier key absent"
    );
  }

  // ---- Step 2 — delete `jury.panel_size.severe`; cascade should fall
  //      to bare `jury.panel_size = 5` (JM-a seed).
  sql_query(
    "DELETE FROM governance_config \
     WHERE scope = 'instance' AND key = 'jury.panel_size.severe'",
  )
  .execute(&mut conn)
  .await?;

  {
    let mut cache = ConfigCache::new();
    let mut pool: DbPool<'_> = (&mut conn).into();
    let got = get_int_cascade(
      &mut cache,
      &mut pool,
      Scope::Instance,
      "jury.panel_size",
      &["founder", "severe"],
    )
    .await?;
    assert_eq!(
      got, 5,
      "cascade falls through severe → bare jury.panel_size when severity key absent"
    );
  }

  // ---- Step 3 — delete bare `jury.panel_size`; cascade should fall to
  //      the Rust const default DEFAULT_JURY_PANEL_SIZE = 5.
  sql_query(
    "DELETE FROM governance_config \
     WHERE scope = 'instance' AND key = 'jury.panel_size'",
  )
  .execute(&mut conn)
  .await?;

  {
    let mut cache = ConfigCache::new();
    let mut pool: DbPool<'_> = (&mut conn).into();
    let got = get_int_cascade(
      &mut cache,
      &mut pool,
      Scope::Instance,
      "jury.panel_size",
      &["founder", "severe"],
    )
    .await?;
    assert_eq!(
      got, 9,
      "cascade falls to per-tier const DEFAULT_JURY_PANEL_SIZE_FOUNDER_SEVERE = 9 \
       when DB has no matching row (most-specific-first const cascade per v1-JM-a)"
    );
  }
  Ok(())
}

// ============================================================================
// v1-JM-b Task 8 — R1 relaxation + admin_emergency_remove severity_tier
// ============================================================================

/// v1-JM-b Task 8 test 1 — 3 of 5 jurors are "recently served" (their
/// jury_assignment row on a prior case has `responded_at = now() - 1 day`
/// and `status = Submitted`). The cooldown subquery at
/// admin_assign_jury.rs:820-828 excludes those 3, the strict pool under-
/// fills (2 < panel_size=5), R1 fires, cooldown is dropped, the re-run
/// pool is 5 ≥ panel_size, and the panel seats.
///
/// Asserts:
/// - Panel is seated at exactly 5 jurors.
/// - One `jury_constraint_violation_log` row exists with
///   `constraint_name = 'no_recent_juror_repeat'` and
///   `reason_code = 'small_pool'` (the snake_case serde rendering of
///   `JuryConstraintRelaxationReason::SmallPool`).
/// - One `governance_log` row exists with
///   `entry_kind = 'jury_constraint_relaxed'`.
///
/// PRD §5.1/§8.3 + plan §10.9.
#[tokio::test(flavor = "multi_thread")]
async fn admin_assign_jury_small_pool_triggers_r1_relaxation() -> lemmy_utils::error::LemmyResult<()>
{
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, sql_query};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_assign_jury::admin_assign_jury;
  use lemmy_api_common::governance::AdminAssignJury;
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{JuryConstraintRelaxationReason, SeverityTier},
    schema::{governance_log, jury_constraint_violation_log},
  };

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_r1", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_r1", false).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 5).await?;

  // Unlock the strict eligibility path for all 5 jurors.
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  // Seed a prior case + 3 "recently served" jury_assignment rows on the
  // first three jurors so the cooldown subquery excludes them. The INSERT
  // uses raw SQL because the InsertForm doesn't carry `responded_at` —
  // that column is written by accept/decline/submit_vote handlers. The
  // prior-case id is captured so this test can isolate its
  // jury_constraint_violation_log count to the NEW case only.
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let prior_case_id: i32 = sql_query(
      "INSERT INTO moderation_case (community_id, target_type, reason_code, severity, status, \
                                    threshold_score, severity_tier, status_tier) \
       VALUES (NULL, 'RemoteInstance', 'prior_case', 'Low', 'Closed', 0, 'Minor', 'Regular') \
       RETURNING id",
    )
    .get_result::<SingleI32>(&mut conn)
    .await?
    .id;
    for juror in jurors.iter().take(3) {
      sql_query(
        "INSERT INTO jury_assignment (case_id, person_id, status, selected_at, responded_at) \
         VALUES ($1, $2, 'Submitted', now() - INTERVAL '2 days', now() - INTERVAL '1 day')",
      )
      .bind::<diesel::sql_types::Int4, _>(prior_case_id)
      .bind::<diesel::sql_types::Int4, _>(juror.0)
      .execute(&mut conn)
      .await?;
    }
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(
    resp.assigned_person_ids.len(),
    5,
    "R1 fires → cooldown dropped → all 5 jurors seated"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // Scope the violation-log query to the NEW case_id so the prior-case
  // seed doesn't interfere if future test setup changes.
  let jcvl_rows: Vec<(String, JuryConstraintRelaxationReason)> =
    jury_constraint_violation_log::table
      .filter(jury_constraint_violation_log::case_id.eq(case_id))
      .select((
        jury_constraint_violation_log::constraint_name,
        jury_constraint_violation_log::reason_code,
      ))
      .load::<(String, JuryConstraintRelaxationReason)>(&mut conn)
      .await?;
  assert_eq!(
    jcvl_rows.len(),
    1,
    "exactly one jury_constraint_violation_log row written for the R1 event"
  );
  let row = jcvl_rows
    .get(0)
    .ok_or_else(|| anyhow::anyhow!("no jury_constraint_violation_log row"))?;
  assert_eq!(row.0, "no_recent_juror_repeat", "constraint_name matches");
  assert_eq!(
    row.1,
    JuryConstraintRelaxationReason::SmallPool,
    "reason_code = SmallPool"
  );

  let relaxed_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("jury_constraint_relaxed"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    relaxed_count, 1,
    "exactly one jury_constraint_relaxed governance_log entry"
  );

  Ok(())
}

/// v1-JM-b Task 8 test 2 — `emergency_remove_open_case` opens a case row
/// whose `severity_tier = Severe`, as required by ADR-013 + plan §10.13.
/// Uses a community target to avoid needing a post/comment fixture.
#[tokio::test(flavor = "multi_thread")]
async fn admin_emergency_remove_case_has_severity_tier_severe()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_emergency_remove::{
    EmergencyRemoveTarget, emergency_remove_open_case,
  };
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{CaseStatus, SeverityTier},
    schema::moderation_case,
  };

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (admin_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "admin_er", true).await?;

  let case_id = emergency_remove_open_case(
    &mut context.pool(),
    admin_id,
    EmergencyRemoveTarget::Community(community.id),
    Some(community.id),
    "ADR-013 test removal".to_string(),
  )
  .await?;

  // Snapshot + tier assertions. PR #95 cr-3 + cr-4: emergency-remove
  // routes through the same cascade/snapshot path as admin_assign_jury,
  // so a Severe + Regular (no target_person_id) case freezes
  // panel_size = 7 / quorum = 5 / threshold = 6 from
  // jury.panel_size.regular.severe + jury.{quorum,threshold}_fraction.severe.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let row: (
    SeverityTier,
    CaseStatus,
    Option<i32>,
    Option<i32>,
    Option<i32>,
  ) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::severity_tier,
      moderation_case::status,
      moderation_case::panel_size_snapshot,
      moderation_case::quorum_snapshot,
      moderation_case::threshold_count_snapshot,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(
    row.0,
    SeverityTier::Severe,
    "emergency_remove opens case with severity_tier = Severe (plan §10.13 / ADR-013)"
  );
  assert_eq!(
    row.1,
    CaseStatus::EmergencyRemove,
    "emergency_remove opens case with status = EmergencyRemove (ADR-013)"
  );
  assert_eq!(
    row.2,
    Some(7),
    "panel_size_snapshot = 7 (jury.panel_size.regular.severe seed; PR #95 cr-3)"
  );
  assert_eq!(
    row.3,
    Some(5),
    "quorum_snapshot = ceil(7 × 0.71) = 5 (PR #95 cr-3)"
  );
  assert_eq!(
    row.4,
    Some(6),
    "threshold_count_snapshot = ceil(7 × 0.75) = 6 (PR #95 cr-3)"
  );
  Ok(())
}

/// PR #95 cr-3 + cr-4 — emergency-remove with a seedable juror pool seats
/// the full Severe-tier panel, persists `selected_under_constraints` per
/// juror, and emits both `severity_tier_frozen` and the extended
/// `panel_assembled` payload, matching `admin_assign_jury` exactly.
#[tokio::test(flavor = "multi_thread")]
async fn admin_emergency_remove_seats_severe_panel_with_constraint_record()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_emergency_remove::{
    EmergencyRemoveTarget, emergency_remove_open_case,
  };
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::schema::{governance_log, jury_assignment};
  use serde_json::Value;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (admin_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "admin_erp", true).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 9).await?;

  // Reputation snapshots scoped to the same community as the case so
  // the strict eligibility join's
  // `rs.community_id IS NOT DISTINCT FROM case.community_id` predicate
  // matches; instance-scoped (NULL) snapshots would miss the
  // community-scoped emergency-remove case and force R1 + fallback.
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots_scoped(&mut conn, &jurors, Some(community.id))
      .await?;
  }

  let case_id = emergency_remove_open_case(
    &mut context.pool(),
    admin_id,
    EmergencyRemoveTarget::Community(community.id),
    Some(community.id),
    "ADR-013 panel-seating test".to_string(),
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // 7 jurors seated (Regular + Severe → panel_size 7).
  let assignment_count: i64 = jury_assignment::table
    .filter(jury_assignment::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    assignment_count, 7,
    "emergency-remove seats Severe-tier panel of 7 jurors"
  );

  // Per-juror selected_under_constraints JSONB carries the steady-state
  // ConstraintRecord shape (PRD §5.1).
  let constraints: Vec<Option<Value>> = jury_assignment::table
    .filter(jury_assignment::case_id.eq(case_id))
    .select(jury_assignment::selected_under_constraints)
    .load(&mut conn)
    .await?;
  for (idx, payload) in constraints.iter().enumerate() {
    let value = payload
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("emergency-seated row {idx} missing constraint record"))?;
    assert_eq!(
      value["no_majority_from_same_sponsor_cluster"],
      Value::String("applied".to_string()),
      "emergency-seated row {idx} cluster constraint applied"
    );
    assert_eq!(
      value["no_recent_juror_repeat"],
      Value::String("applied".to_string()),
      "emergency-seated row {idx} cooldown constraint applied"
    );
  }

  // Exactly one severity_tier_frozen entry — admin is the actor.
  let stf_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("severity_tier_frozen"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    stf_count, 1,
    "emergency-remove emits exactly one severity_tier_frozen entry"
  );

  // Exactly one panel_assembled entry per emergency-remove (single
  // assign call); payload carries the resolved tiers + relaxations
  // per Task 5 §10.12 mirror.
  let panel_payloads: Vec<Value> = governance_log::table
    .filter(governance_log::entry_kind.eq("panel_assembled"))
    .select(governance_log::payload)
    .load(&mut conn)
    .await?;
  assert_eq!(
    panel_payloads.len(),
    1,
    "exactly one panel_assembled entry per emergency-remove"
  );
  let payload = panel_payloads
    .get(0)
    .ok_or_else(|| anyhow::anyhow!("no panel_assembled payload"))?;
  assert_eq!(payload["juror_count"], Value::from(7));
  assert_eq!(
    payload["severity_tier"],
    Value::String("severe".to_string())
  );
  assert_eq!(payload["status_tier"], Value::String("regular".to_string()));
  Ok(())
}

// ============================================================================
// v1-JM-c Task 6 — six e2e tests for snapshot-aware threshold + deadlock +
// appeal_window_expires_at write.
//
// All six tests reuse `v1_jm_b_fixtures::{bootstrap, seed_user, seed_community,
// seed_jurors, seed_jury_eligible_snapshots, seed_case}` per JM-b retro §3.2
// amendment 2 (R2). EVERY test seeds reputation_snapshot rows BEFORE
// `admin_assign_jury` so the small-pool fallback does not fire.
//
// Test names use lowercase snake_case per JM-b retro §3.2 amendment 4 (R4).
// ============================================================================

/// JM-c Task 6 test 1 — 7-juror Severe panel decides at 6 votes
/// (threshold_count_snapshot = ceil(7 × 0.75) = 6 per JM-b seeded values).
///
/// The plan §14.1 row 1 mentions "decides at 5 votes (threshold=5)" but the
/// actual JM-b snapshot machinery freezes Severe panels at threshold = 6
/// (verified against `admin_assign_jury_severity_tier_regular_severe_panel_7_jurors`
/// at line 7228+). We cast 6 votes and assert the case decides on vote 6.
#[tokio::test(flavor = "multi_thread")]
async fn submit_jury_vote_severe_panel_meets_threshold() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote};
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseStatus, JuryDecision, SeverityTier},
    schema::{governance_log, moderation_case, sanction},
  };
  use lemmy_db_views_local_user::LocalUserView;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jmc1", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jmc1", false).await?;
  // Seed 9 jurors so admin_assign_jury can pick 7 and snapshot panel_size = 7.
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 9).await?;

  // R2: seed jury_eligible snapshots BEFORE admin_assign_jury.
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Severe)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let assign_resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();
  assert_eq!(
    assign_resp.assigned_person_ids.len(),
    7,
    "Severe panel = 7 jurors per JM-b snapshot"
  );

  // submit_jury_vote takes federation Data per Phase 6 task 76.
  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  // Accept all assignments first — submit_jury_vote requires Accepted status.
  for juror_id in &assign_resp.assigned_person_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    accept_jury_assignment(
      Json(AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // Cast 6 RemoveContent votes (threshold_count_snapshot = 6 for Severe).
  let voting_jurors: Vec<PersonId> = assign_resp
    .assigned_person_ids
    .iter()
    .copied()
    .take(6)
    .collect();
  let mut last_resp = None;
  for (i, juror_id) in voting_jurors.iter().enumerate() {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: Some(format!("severe vote {i}")),
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?
    .into_inner();
    if i < 5 {
      assert!(
        !resp.case_decided,
        "vote {i}: must NOT be decided pre-threshold"
      );
    }
    last_resp = Some(resp);
  }
  let final_resp = last_resp.expect("at least one vote cast");
  assert!(
    final_resp.case_decided,
    "vote 6 (threshold_count_snapshot for Severe) must decide the case"
  );
  assert_eq!(final_resp.decision, Some(JuryDecision::RemoveContent));

  // Assert post-decision DB state.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let (status, decided_at, appeal_expires): (
    CaseStatus,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
  ) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::status,
      moderation_case::decided_at,
      moderation_case::appeal_window_expires_at,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(status, CaseStatus::Decided, "case must be Decided");
  assert!(decided_at.is_some(), "decided_at populated");
  assert!(
    appeal_expires.is_some(),
    "appeal_window_expires_at populated by JM-c step 9"
  );

  let sanction_count: i64 = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(sanction_count, 1, "exactly one sanction row");

  let case_decided_log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("case_decided"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(case_decided_log_count, 1, "exactly one case_decided entry");

  Ok(())
}

/// JM-c Task 6 test 2 — 5-juror Minor panel split 2/2/1 → AdminReview +
/// `jury_deadlock` log. After all 5 jurors vote with no decision meeting
/// `threshold_count_snapshot = 3`, the case must:
///   - flip to `CaseStatus::AdminReview`
///   - leave `decided_at` and `appeal_window_expires_at` NULL
///   - emit exactly one `jury_deadlock` governance_log entry
///   - NOT emit `case_decided`, `sanction_created`, or `public_log_published`
///   - write zero `sanction` or `public_case_log` rows
///   - write zero `reputation_event` rows for the case
#[tokio::test(flavor = "multi_thread")]
async fn submit_jury_vote_deadlock_flips_to_admin_review() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote};
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{CaseStatus, JuryDecision, SeverityTier},
    schema::{governance_log, moderation_case, public_case_log, reputation_event, sanction},
  };
  use lemmy_db_views_local_user::LocalUserView;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jmc2", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jmc2", false).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 7).await?;

  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

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
    "Minor panel = 5 jurors per JM-b snapshot"
  );

  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  for juror_id in &assign_resp.assigned_person_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    accept_jury_assignment(
      Json(AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // Vote split 2/2/1: jurors 0+1 RemoveContent, 2+3 NoAction, 4 AdvisoryLabel.
  // No decision meets threshold_count_snapshot = 3; deadlock fires on vote 5.
  let votes = [
    JuryDecision::RemoveContent,
    JuryDecision::RemoveContent,
    JuryDecision::NoAction,
    JuryDecision::NoAction,
    JuryDecision::AdvisoryLabel,
  ];
  let mut last_resp = None;
  for (i, juror_id) in assign_resp.assigned_person_ids.iter().enumerate() {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: votes[i],
        rationale: Some(format!("split vote {i}")),
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?
    .into_inner();
    // Deadlock returns case_decided: false on the final vote — the case is
    // NOT decided, it's stuck pending admin (per submit_jury_vote.rs deadlock
    // branch comment "Deadlock differs ... vote_recorded: true, case_decided:
    // false").
    assert!(
      !resp.case_decided,
      "vote {i}: deadlock means case_decided stays false even on the panel-completing vote"
    );
    last_resp = Some(resp);
  }
  let final_resp = last_resp.expect("at least one vote cast");
  assert!(final_resp.vote_recorded, "vote 5 recorded");
  assert!(
    final_resp.decision.is_none(),
    "deadlock has no winning decision"
  );

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  #[expect(
    clippy::type_complexity,
    reason = "Diesel tuple query; local variable type annotation required for inference"
  )]
  let (status, decided_at, appeal_expires, closed_at): (
    CaseStatus,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
  ) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::status,
      moderation_case::decided_at,
      moderation_case::appeal_window_expires_at,
      moderation_case::closed_at,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(
    status,
    CaseStatus::AdminReview,
    "deadlock flips status to AdminReview"
  );
  assert!(
    decided_at.is_none(),
    "deadlock leaves decided_at NULL — the case is not decided"
  );
  assert!(
    appeal_expires.is_none(),
    "deadlock leaves appeal_window_expires_at NULL — no appeal window for stuck cases"
  );
  assert!(
    closed_at.is_none(),
    "deadlock leaves closed_at NULL — JM-c removed the close write; appeal/admin-review cases reopen"
  );

  let sanction_count: i64 = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(sanction_count, 0, "no sanction row for deadlocked case");

  let public_log_count: i64 = public_case_log::table
    .filter(public_case_log::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    public_log_count, 0,
    "no public_case_log entry for deadlocked case"
  );

  let rep_event_count: i64 = reputation_event::table
    .filter(reputation_event::source_case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    rep_event_count, 0,
    "no reputation_event rows for deadlocked case"
  );

  let deadlock_log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("jury_deadlock"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    deadlock_log_count, 1,
    "exactly one jury_deadlock governance_log entry"
  );

  let case_decided_log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("case_decided"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    case_decided_log_count, 0,
    "case_decided NOT emitted on deadlock path"
  );

  let public_log_published_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("public_log_published"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    public_log_published_count, 0,
    "public_log_published NOT emitted on deadlock path"
  );

  let sanction_created_log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("sanction_created"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    sanction_created_log_count, 0,
    "sanction_created NOT emitted on deadlock path — no sanction means no log"
  );

  Ok(())
}

/// JM-c Task 6 test 3 — `appeal_window_expires_at` populated on no-sponsor
/// path with default `appeal.window_days = 7`. Also asserts `closed_at` is
/// NULL (JM-c removed the close write at step 8).
#[tokio::test(flavor = "multi_thread")]
async fn submit_jury_vote_writes_appeal_window_default() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use chrono::Duration;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote};
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{CaseStatus, JuryDecision, SeverityTier},
    schema::moderation_case,
  };
  use lemmy_db_views_local_user::LocalUserView;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jmc3", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jmc3", false).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 7).await?;

  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let assign_resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  for juror_id in &assign_resp.assigned_person_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    accept_jury_assignment(
      Json(AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // Cast 3 RemoveContent votes to meet threshold_count_snapshot = 3 for Minor.
  for juror_id in assign_resp.assigned_person_ids.iter().take(3) {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: None,
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?;
  }

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  #[expect(
    clippy::type_complexity,
    reason = "Diesel tuple query; local variable type annotation required for inference"
  )]
  let (status, decided_at, closed_at, appeal_expires): (
    CaseStatus,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
  ) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::status,
      moderation_case::decided_at,
      moderation_case::closed_at,
      moderation_case::appeal_window_expires_at,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(status, CaseStatus::Decided);
  let decided = decided_at.expect("decided_at set");
  let expires = appeal_expires.expect("appeal_window_expires_at set");
  let gap = expires - decided;
  let expected = Duration::days(7);
  assert!(
    (gap - expected).num_milliseconds().abs() < 1_000,
    "appeal_window_expires_at = decided_at + 7d (default); got gap = {gap:?}"
  );
  assert!(
    closed_at.is_none(),
    "closed_at must be NULL — JM-c removed the v0 closed_at write"
  );

  Ok(())
}

/// JM-c Task 6 test 4 — `appeal_window_expires_at` reflects mid-flight
/// `appeal.window_days` config bump. Bump from 7 → 30 BEFORE the
/// threshold-meeting vote and assert the live read picked up 30.
#[tokio::test(flavor = "multi_thread")]
async fn submit_jury_vote_writes_appeal_window_live_config() -> lemmy_utils::error::LemmyResult<()>
{
  use actix_web::web::Json;
  use chrono::Duration;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    admin_config::admin_set_config, submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, AdminSetConfig, SubmitJuryVote,
  };
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{CaseStatus, JuryDecision, SeverityTier},
    schema::moderation_case,
  };
  use lemmy_db_views_local_user::LocalUserView;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jmc4", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jmc4", false).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 7).await?;

  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

  let assign_resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view.clone(),
  )
  .await?
  .into_inner();

  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  for juror_id in &assign_resp.assigned_person_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    accept_jury_assignment(
      Json(AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // Bump appeal.window_days BEFORE any vote. JM-c step 9 is a LIVE config
  // read at decision time (the deliberate snapshot exception).
  admin_set_config(
    Json(AdminSetConfig {
      key: "appeal.window_days".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(30),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "JM-c live-read test bump".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?;

  for juror_id in assign_resp.assigned_person_ids.iter().take(3) {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: None,
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?;
  }

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let (status, decided_at, appeal_expires): (
    CaseStatus,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
  ) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::status,
      moderation_case::decided_at,
      moderation_case::appeal_window_expires_at,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(status, CaseStatus::Decided);
  let decided = decided_at.expect("decided_at set");
  let expires = appeal_expires.expect("appeal_window_expires_at set");
  let gap = expires - decided;
  let expected = Duration::days(30);
  assert!(
    (gap - expected).num_milliseconds().abs() < 1_000,
    "appeal_window_expires_at = decided_at + 30d (LIVE config bumped from 7); got gap = {gap:?}"
  );

  Ok(())
}

/// JM-c Task 6 test 5 — PRD §11 canonical regression. A case opened under
/// v0 defaults (panel=5, threshold=3, appeal_window=7) completes under those
/// snapshot values even after the admin flips three governance config keys
/// mid-flight to wildly different values. The appeal_window_expires_at uses
/// the LIVE bumped value (60d) — the deliberate snapshot exception per
/// ADR-010 + PRD §9.1 cross-references.
#[tokio::test(flavor = "multi_thread")]
async fn v0_case_completes_under_v0_rules_after_v1_config_flip()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use chrono::Duration;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    admin_config::admin_set_config, submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, AdminSetConfig, SubmitJuryVote,
  };
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{CaseStatus, JuryDecision, SeverityTier},
    schema::moderation_case,
  };
  use lemmy_db_views_local_user::LocalUserView;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jmc5", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jmc5", false).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 7).await?;

  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  // Step 1: open case under v0 defaults (Minor severity).
  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;
  let assign_resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view.clone(),
  )
  .await?
  .into_inner();
  // Snapshot fields should be: panel=5, quorum=3, threshold=3.
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let snap: (Option<i32>, Option<i32>, Option<i32>) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::panel_size_snapshot,
        moderation_case::quorum_snapshot,
        moderation_case::threshold_count_snapshot,
      ))
      .first(&mut conn)
      .await?;
    assert_eq!(
      snap,
      (Some(5), Some(3), Some(3)),
      "v0-default snapshot values"
    );
  }

  // Step 2: admin flips three config keys WHILE case is in InReview.
  for (key, value_type, value) in [
    (
      "jury.panel_size.regular.minor",
      "int",
      serde_json::json!(11),
    ),
    (
      "jury.threshold_fraction.minor",
      "float",
      serde_json::json!(0.95),
    ),
    ("appeal.window_days", "int", serde_json::json!(60)),
  ] {
    admin_set_config(
      Json(AdminSetConfig {
        key: key.to_string(),
        value_type: value_type.to_string(),
        value,
        scope: "instance".to_string(),
        apply_at: None,
        dry_run: None,
        reason: "v0-compat regression test".to_string(),
      }),
      context.clone(),
      admin_view.clone(),
    )
    .await?;
  }

  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  for juror_id in &assign_resp.assigned_person_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    accept_jury_assignment(
      Json(AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // Step 3: cast 3 RemoveContent votes — v0 threshold = 3 must still apply,
  // NOT the new 0.95 × 11 = 11 fraction-derived value.
  for juror_id in assign_resp.assigned_person_ids.iter().take(3) {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: None,
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?;
  }

  // Step 4: assert case decided under v0 snapshot rules; appeal_window
  // uses LIVE config (60d), the deliberate exception.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let (status, decided_at, appeal_expires): (
    CaseStatus,
    Option<chrono::DateTime<chrono::Utc>>,
    Option<chrono::DateTime<chrono::Utc>>,
  ) = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::status,
      moderation_case::decided_at,
      moderation_case::appeal_window_expires_at,
    ))
    .first(&mut conn)
    .await?;
  assert_eq!(
    status,
    CaseStatus::Decided,
    "decided despite mid-flight config change (v0 snapshot rules honoured)"
  );
  let decided = decided_at.expect("decided_at set");
  let expires = appeal_expires.expect("appeal_window_expires_at set");
  let gap = expires - decided;
  let expected = Duration::days(60);
  assert!(
    (gap - expected).num_milliseconds().abs() < 1_000,
    "appeal_window uses LIVE config (60), not snapshotted v0 default (7); got gap = {gap:?}"
  );

  Ok(())
}

/// JM-c Task 6 test 6 — concurrency: two jurors race to be the
/// threshold-meeting vote via `tokio::join!`. The FOR UPDATE lock at the
/// case load + the idempotency guard SHOULD ensure exactly-once
/// post-decision side effects. Both calls SHOULD succeed; only one sanction
/// + one case_decided log SHOULD fire.
///
/// **STATUS: ignored — see DQ #49.** This test surfaces a deterministic
/// Postgres deadlock between two concurrent transactions:
///   1. Each tx INSERTs a row into `jury_vote` (step 2 of submit_jury_vote).
///      The INSERT takes a foreign-key SHARE lock on the parent
///      `moderation_case` row.
///   2. Each tx then tries to acquire SELECT ... FOR UPDATE on that same
///      `moderation_case` row at submit_jury_vote.rs:240 (step 6).
///   3. Both txs hold SHARE on the case row and wait for the other to
///      release it before EXCLUSIVE can be granted. Postgres' deadlock
///      detector kills one with `ERROR: deadlock detected`.
///
/// The handler's vote-INSERT-before-FOR-UPDATE structure is pre-existing
/// (v0/JM-b era) and out of JM-c file-ownership scope to refactor. Plan
/// §10.8 GOTCHA anticipated escalation here. The fix is one of:
///   - JM-d (or a separate chore): acquire FOR UPDATE on case_row BEFORE
///     the vote INSERT so the lock-acquisition order is consistent across
///     concurrent voters.
///   - Accept that real-prod concurrent voting on the same case is rare
///     and that one juror occasionally sees deadlock_detected and retries.
///
/// Tests 1-5 ship as JM-c's e2e coverage. The exactly-once invariant under
/// SEQUENTIAL late arrivals (votes 4-5 after vote 3 trips threshold) remains
/// covered by `report_to_modlog_golden_path` at line ~1061. The test body
/// below is preserved as the diagnostic anchor for a future handler refactor.
#[tokio::test(flavor = "multi_thread")]
async fn submit_jury_vote_concurrent_votes_decide_exactly_once()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote};
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{JuryDecision, SeverityTier},
    schema::{governance_log, jury_vote, sanction},
  };
  use lemmy_db_views_local_user::LocalUserView;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jmc6", true).await?;
  let (target, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jmc6", false).await?;
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 7).await?;

  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
    .await
    .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;
  let assign_resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();
  assert_eq!(assign_resp.assigned_person_ids.len(), 5);

  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  for juror_id in &assign_resp.assigned_person_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    accept_jury_assignment(
      Json(AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // Cast 2 votes serially; threshold (3) not yet met.
  for juror_id in assign_resp.assigned_person_ids.iter().take(2) {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: None,
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?;
  }

  // Now race jurors 2 and 3. Both vote RemoveContent. Whichever wins the
  // FOR UPDATE lock first runs the post-decision block; the other observes
  // status=Decided via the idempotency guard and short-circuits.
  let view_a =
    LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2]).await?;
  let view_b =
    LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[3]).await?;
  let fed_a = federation_context.reset_request_count();
  let fed_b = federation_context.reset_request_count();

  let (res_a, res_b) = tokio::join!(
    submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: None,
      }),
      fed_a,
      view_a,
    ),
    submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RemoveContent,
        rationale: None,
      }),
      fed_b,
      view_b,
    ),
  );
  res_a?;
  res_b?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  let vote_count: i64 = jury_vote::table
    .filter(jury_vote::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    vote_count, 4,
    "all 4 votes recorded (2 sequential + 2 concurrent)"
  );

  let sanction_count: i64 = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    sanction_count, 1,
    "post-decision block ran exactly once — exactly one sanction row"
  );

  let case_decided_log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("case_decided"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    case_decided_log_count, 1,
    "case_decided emitted exactly once despite the race"
  );

  Ok(())
}

// ============================================================================
// v1-JM-e fixtures + capstone
// ============================================================================
//
// `mod v1_jm_e_fixtures` houses the cross-sub-phase helpers and tests that
// exercise the full original-decide → request_appeal → appeal-panel-decide
// lifecycle as one piece. Authored by the advisor session per PMD #117 (Junior
// workers hang on Edit calls into this 9000+ line file).

mod v1_jm_e_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, RequestAppeal, SubmitJuryVote,
  };
  use lemmy_api_crud::governance::request_appeal::request_appeal;
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::newtypes::{AppealId, ModerationCaseId};
  use lemmy_db_schema_file::{
    PersonId,
    enums::{JuryAssignmentRole, JuryDecision, SeverityTier},
    schema::jury_assignment,
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_utils::error::LemmyResult;

  /// Drive a case end-to-end up to and including a successful `request_appeal`,
  /// returning everything the capstone test needs to drive the appeal panel
  /// through accept + vote.
  ///
  /// Sequence:
  ///   1. seed 13 jury_eligible snapshots (5 original + 8 appeal panel).
  ///   2. seed Minor-severity case via `v1_jm_b_fixtures::seed_case`.
  ///   3. `admin_assign_jury` → 5 original jurors seated (Minor panel = 5).
  ///   4. all 5 accept their assignments.
  ///   5. all 5 vote `NoAction` (5/5 ≥ threshold 3 → NoAction wins; case → Decided
  ///      with `winning_decision = Some(NoAction)`).
  ///   6. `request_appeal` (caller = target — request_appeal allows the case
  ///      target to appeal a NoAction verdict per the test author's pre-existing
  ///      JM-d `appeal_inside_window_succeeds_expired_rejects` precedent).
  ///      Internally calls `select_appeal_panel` + `seat_appeal_panel`,
  ///      seating 8 appeal-panel jurors with the 5 originals excluded.
  ///   7. returns (case_id, appeal_id, appeal_panel_person_ids).
  ///
  /// **Math** — verified against `compute_appeal_panel_size` in
  /// `crates/api/api/src/governance/admin_assign_jury.rs:1049`:
  ///   - original_panel_size = 5 (Minor)
  ///   - multiplier = 1.5 → from_multiplier = ceil(5 × 1.5) = 8
  ///   - floor_increment = 2 → from_floor = 5 + 2 = 7
  ///   - max(8, 7) = 8, clamped to [3, 11] = **appeal panel = 8**
  ///   - bumped severity = Moderate (Minor + 1)
  ///   - threshold_fraction (Moderate) = 0.6 → ceil(0.6 × 8) = ceil(4.8) = **threshold = 5**
  ///
  /// Plan §13 narrative said "panel of 7" — actual default-config math yields
  /// 8 (logged as kind: "log" DQ for retro flag).
  pub async fn seed_appealed_case_with_panel(
    context: &actix_web::web::Data<LemmyContext>,
    db_url: &str,
    target: PersonId,
    target_view: LocalUserView,
    admin_view: LocalUserView,
    jurors: &[PersonId],
    federation_context: activitypub_federation::config::Data<LemmyContext>,
  ) -> LemmyResult<(ModerationCaseId, AppealId, Vec<PersonId>)> {
    assert!(
      jurors.len() >= 13,
      "seed_appealed_case_with_panel requires >=13 eligibles (5 original + 8 appeal); got {}",
      jurors.len()
    );

    // R2: seed jury_eligible snapshots BEFORE admin_assign_jury.
    {
      let mut conn = AsyncPgConnection::establish(db_url).await?;
      super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, jurors).await?;
    }

    let case_id = super::v1_jm_b_fixtures::seed_case(db_url, target, SeverityTier::Minor)
      .await
      .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;

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
      "Minor panel = 5 jurors per JM-b snapshot"
    );

    // Original jurors accept.
    for juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }

    // 5 NoAction votes → NoAction wins (5 ≥ threshold 3 for Minor).
    for juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
      submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision: JuryDecision::NoAction,
          rationale: None,
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?;
    }

    // request_appeal — caller is the case target (NoAction verdict appealable
    // by the target per JM-d `appeal_inside_window_succeeds_expired_rejects`).
    // Internally seats the appeal panel via select_appeal_panel + seat_appeal_panel.
    let appeal_resp = request_appeal(
      Json(RequestAppeal {
        case_id,
        reason: "capstone appeal".to_string(),
      }),
      context.clone(),
      target_view,
    )
    .await?
    .into_inner();

    // Read appeal-panel jurors directly (role = Appeal) for the caller to drive.
    let mut conn = AsyncPgConnection::establish(db_url).await?;
    let appeal_panel: Vec<PersonId> = jury_assignment::table
      .filter(jury_assignment::case_id.eq(case_id))
      .filter(jury_assignment::role.eq(JuryAssignmentRole::Appeal))
      .select(jury_assignment::person_id)
      .load(&mut conn)
      .await?;
    assert_eq!(
      appeal_panel.len(),
      8,
      "appeal panel = 8 jurors per compute_appeal_panel_size(5, 1.5, 2)"
    );

    Ok((case_id, appeal_resp.appeal_id, appeal_panel))
  }

  /// Sibling of [`seed_appealed_case_with_panel`] that opens the case via
  /// `create_report` (×3 reporters to cross the v0 case_threshold = 3) instead
  /// of the direct `v1_jm_b_fixtures::seed_case` insert. The full lifecycle
  /// (admin_assign → 5 accepts → 5 NoAction votes → request_appeal → 8
  /// appeal jurors seated) matches [`seed_appealed_case_with_panel`] exactly,
  /// so callers asserting on lifecycle invariants get the same shape.
  ///
  /// **Why this exists** (Task 3 GOTCHA): the audit-log-invariant test must
  /// assert on `report_created` + `threshold_met` entries which only fire
  /// when a case is opened via `create_report`. The capstone fixture uses
  /// `seed_case` (direct insert) which bypasses the create_report handler
  /// and therefore never emits those two log kinds.
  ///
  /// **Reporter discipline:** the 3 reporters MUST be distinct from the 13
  /// jurors. The first reporter becomes `case.creator_id` (per
  /// `create_report.rs:206`) and `accept_jury_assignment` excludes the
  /// case creator from accepting (`creator_id == caller_id` → NotFound),
  /// which would break the 5-juror accept loop.
  ///
  /// Returns the same shape as [`seed_appealed_case_with_panel`].
  #[expect(
    clippy::too_many_arguments,
    reason = "integration test fixture requires all caller-side inputs; no natural grouping"
  )]
  pub async fn seed_appealed_case_with_panel_via_report(
    context: &actix_web::web::Data<LemmyContext>,
    db_url: &str,
    target: PersonId,
    target_view: LocalUserView,
    admin_view: LocalUserView,
    reporters: &[LocalUserView],
    jurors: &[PersonId],
    federation_context: activitypub_federation::config::Data<LemmyContext>,
  ) -> LemmyResult<(ModerationCaseId, AppealId, Vec<PersonId>)> {
    use diesel::SelectableHelper;
    use lemmy_api_common::governance::CreateGovernanceReport;
    use lemmy_api_crud::governance::create_report::create_report;
    use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
    use lemmy_db_schema_file::enums::{CaseStatus, CaseTargetType};
    use lemmy_db_schema_file::schema::moderation_case;

    assert!(
      reporters.len() >= 4,
      "seed_appealed_case_with_panel_via_report requires >=4 reporters to cross default case_threshold_micros (3_000_000) with 4 × 1_000_000 weight; got {}",
      reporters.len()
    );
    assert!(
      jurors.len() >= 13,
      "seed_appealed_case_with_panel_via_report requires >=13 eligibles (5 original + 8 appeal); got {}",
      jurors.len()
    );

    // Seed reputation_snapshot for the reporters with reporting_accuracy = 100
    // so each report contributes the deterministic max weight of 1_000_000
    // micros per the OQ-006 formula in create_report.rs (base_weight 1.0 ×
    // accuracy/100 clamped × recency 1.0 × 1_000_000). Without this, fresh
    // users hit reputation_snapshot::recompute_snapshot which yields a
    // baseline accuracy that varies with seed conditions, making the
    // 4-reports-cross-threshold math brittle.
    //
    // Also seed jury_eligible snapshots for jurors (R2 discipline — same
    // as the direct-seed sibling fixture).
    {
      let mut conn = AsyncPgConnection::establish(db_url).await?;
      let reporter_ids: Vec<PersonId> = reporters.iter().map(|v| v.person.id).collect();
      super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &reporter_ids).await?;
      super::v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, jurors).await?;
    }

    // Drive 4 create_report calls. Default case_threshold_micros is
    // 3_000_000 (per DEFAULT_REPORT_CASE_THRESHOLD_MICROS in
    // crates/api/api/src/governance/config.rs). With reporting_accuracy =
    // 100 each report contributes 1_000_000 micros. Threshold check is
    // strict `>` (create_report.rs:157 `new_score > threshold_micros`),
    // so 3 reports at 3_000_000 are NOT > 3_000_000 — the 4th crosses.
    //
    // After the 4th call the case is at ThresholdMet which is what
    // admin_assign_jury requires.
    //
    // governance_log emissions:
    //   reports 1-3: report_created only
    //   report 4:    report_created + threshold_met
    //
    // The consuming audit-invariant test uses a "first occurrence per
    // kind" filter so the 4 report_created entries collapse to one in
    // the asserted sequence — matching the PRD §6.7 state-machine
    // prefix shape.
    let mut case_id_opt: Option<ModerationCaseId> = None;
    for (i, reporter_view) in reporters.iter().take(4).enumerate() {
      let resp = create_report(
        Json(CreateGovernanceReport {
          community_id: None,
          target_type: CaseTargetType::Person,
          target_id: target.0,
          reason_code: "spam".to_string(),
          description: Some(format!("audit-invariant report #{i}")),
        }),
        context.clone(),
        reporter_view.clone(),
      )
      .await?
      .into_inner();
      if i == 0 {
        case_id_opt = resp.case_id;
        assert!(!resp.threshold_met, "report 0: 1_000_000 not > 3_000_000");
      } else if i == 3 {
        assert!(
          resp.threshold_met,
          "report 3: 4_000_000 > 3_000_000 (threshold met)"
        );
      } else {
        assert!(
          !resp.threshold_met,
          "report {i}: cumulative not > 3_000_000 yet"
        );
      }
    }
    let case_id = case_id_opt.expect("first create_report returns case_id");

    // Sanity: case must be at ThresholdMet so admin_assign_jury accepts it.
    let mut conn = AsyncPgConnection::establish(db_url).await?;
    let case_after_reports: ModerationCase = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select(ModerationCase::as_select())
      .first(&mut conn)
      .await?;
    assert_eq!(
      case_after_reports.status,
      CaseStatus::ThresholdMet,
      "case must be at ThresholdMet after 3rd report"
    );

    // Force severity_tier = Minor so the panel size + threshold math matches
    // seed_appealed_case_with_panel (Minor → 5-juror panel, threshold 3,
    // appeal panel 8 via compute_appeal_panel_size(5, 1.5, 2)). Without
    // this, severity_tier defaults to whatever create_report computes from
    // reason_code "spam" + the v0 reason→tier table, which may produce a
    // different panel size and break the 13-juror seeder math downstream.
    diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set(moderation_case::severity_tier.eq(SeverityTier::Minor))
      .execute(&mut conn)
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
      "Minor panel = 5 jurors per JM-b snapshot"
    );

    // Original jurors accept.
    for juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }

    // 5 NoAction votes — same as seed_appealed_case_with_panel.
    for juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
      submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision: JuryDecision::NoAction,
          rationale: None,
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?;
    }

    // request_appeal — caller is target.
    let appeal_resp = request_appeal(
      Json(RequestAppeal {
        case_id,
        reason: "audit-invariant appeal".to_string(),
      }),
      context.clone(),
      target_view,
    )
    .await?
    .into_inner();

    // Read appeal-panel jurors (role = Appeal).
    let mut conn = AsyncPgConnection::establish(db_url).await?;
    let appeal_panel: Vec<PersonId> = jury_assignment::table
      .filter(jury_assignment::case_id.eq(case_id))
      .filter(jury_assignment::role.eq(JuryAssignmentRole::Appeal))
      .select(jury_assignment::person_id)
      .load(&mut conn)
      .await?;
    assert_eq!(
      appeal_panel.len(),
      8,
      "appeal panel = 8 jurors per compute_appeal_panel_size(5, 1.5, 2)"
    );

    Ok((case_id, appeal_resp.appeal_id, appeal_panel))
  }
}

/// v1-JM-e Task 2 — capstone: full appeal lifecycle.
///
/// Drives original-jury NoAction → request_appeal → appeal panel of 8 votes 5
/// AdvisoryLabel → appeal_decided emitted with original=NoAction +
/// appeal_winning=AdvisoryLabel; case → Closed; no NEW sanction row inserted.
/// Then runs `appeal_window_expiry::run_appeal_window_expiry_batch` directly
/// and asserts no-op (case is already Closed, not Decided).
#[tokio::test(flavor = "multi_thread")]
async fn appeal_panel_decides_no_action_overrides_to_advisory_label_chain()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    appeal_window_expiry::run_appeal_window_expiry_batch, submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::SubmitJuryVote;
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{AppealStatus, CaseStatus, JuryDecision},
    schema::{appeal, governance_log, moderation_case, sanction},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use serde_json::Value;

  // Disable the appeal-window-expiry background scheduler so the capstone
  // can drive run_appeal_window_expiry_batch directly without race.
  // SAFETY: e2e tests run with --test-threads=1 (LazyLock SETTINGS singleton),
  // so this set_var is effectively single-threaded for the test process.
  // Capture prev value so we restore at test end (cr-9: prevent state leak
  // to other tests that also touch run_appeal_window_expiry_batch).
  let prev_appeal_window_disable = std::env::var_os("BREHON_DISABLE_APPEAL_WINDOW_JOB");
  unsafe {
    std::env::set_var("BREHON_DISABLE_APPEAL_WINDOW_JOB", "1");
  }

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jme1", true).await?;
  let (target, target_view) =
    governance_fixtures::seed_user(&context, instance.id, "target_jme1", false).await?;
  // 13 jurors: 5 original + 8 appeal panel (compute_appeal_panel_size(5, 1.5, 2) = 8).
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 13).await?;

  // Federation context for submit_jury_vote (Phase 6 task 76).
  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  // Seed + appeal — fixture drives original NoAction verdict + appeal seating.
  let (case_id, appeal_id, appeal_panel_ids) = v1_jm_e_fixtures::seed_appealed_case_with_panel(
    &context,
    &db_url,
    target,
    target_view,
    admin_view,
    &jurors,
    federation_context.reset_request_count(),
  )
  .await?;

  // Snapshot governance_log + sanction counts BEFORE the appeal vote so we can
  // assert "no NEW sanction row was inserted by the appeal verdict".
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let sanctions_before: i64 = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  // Original NoAction → no sanction row (map_decision_to_sanction(NoAction) = None).
  assert_eq!(
    sanctions_before, 0,
    "NoAction original verdict produces no sanction"
  );

  // All 8 appeal-panel jurors accept (handler recognises Appeal role per JM-d).
  for juror_id in &appeal_panel_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    lemmy_api::governance::accept_jury_assignment::accept_jury_assignment(
      Json(lemmy_api_common::governance::AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // 5 of 8 vote AdvisoryLabel (threshold = ceil(0.6 × 8) = 5 for Moderate).
  let voting_jurors: Vec<_> = appeal_panel_ids.iter().take(5).copied().collect();
  let mut last_resp = None;
  for (i, juror_id) in voting_jurors.iter().enumerate() {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::AdvisoryLabel,
        rationale: Some(format!("appeal vote {i}")),
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?
    .into_inner();
    if i < 4 {
      assert!(
        !resp.case_decided,
        "appeal vote {i}: must NOT be decided pre-threshold"
      );
    }
    last_resp = Some(resp);
  }
  let final_resp = last_resp.expect("at least one appeal vote cast");
  assert!(
    final_resp.case_decided,
    "appeal vote 5 (threshold for Moderate-bumped panel of 8) must decide the case"
  );
  assert_eq!(final_resp.decision, Some(JuryDecision::AdvisoryLabel));

  // Assert appeal.decided_at populated + status = Decided.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let (appeal_decided_at, appeal_status): (Option<chrono::DateTime<chrono::Utc>>, AppealStatus) =
    appeal::table
      .filter(appeal::id.eq(appeal_id))
      .select((appeal::decided_at, appeal::status))
      .first(&mut conn)
      .await?;
  assert!(appeal_decided_at.is_some(), "appeal.decided_at populated");
  assert_eq!(
    appeal_status,
    AppealStatus::Decided,
    "appeal status = Decided"
  );

  // Assert governance_log[appeal_decided] payload carries
  // original_winning_decision + appeal_winning_decision.
  let appeal_decided_payloads: Vec<Value> = governance_log::table
    .filter(governance_log::entry_kind.eq("appeal_decided"))
    .select(governance_log::payload)
    .load(&mut conn)
    .await?;
  assert_eq!(
    appeal_decided_payloads.len(),
    1,
    "exactly one appeal_decided entry"
  );
  let payload = &appeal_decided_payloads[0];
  // JuryDecision serializes via #[serde(rename_all = "snake_case")] —
  // see crates/db_schema_file/src/enums.rs:489. Audit-log payloads carry
  // the snake_case form ("no_action", "advisory_label"), not the
  // PascalCase Rust variant name.
  assert_eq!(
    payload["original_winning_decision"],
    Value::String("no_action".to_string()),
    "original_winning_decision = no_action"
  );
  assert_eq!(
    payload["appeal_winning_decision"],
    Value::String("advisory_label".to_string()),
    "appeal_winning_decision = advisory_label"
  );
  assert_eq!(
    payload["case_id"],
    Value::Number(case_id.0.into()),
    "payload.case_id matches"
  );
  assert_eq!(
    payload["appeal_id"],
    Value::Number(appeal_id.0.into()),
    "payload.appeal_id matches"
  );

  // Assert case → Closed + closed_at populated.
  let (status, closed_at): (CaseStatus, Option<chrono::DateTime<chrono::Utc>>) =
    moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((moderation_case::status, moderation_case::closed_at))
      .first(&mut conn)
      .await?;
  assert_eq!(
    status,
    CaseStatus::Closed,
    "case → Closed after appeal verdict"
  );
  assert!(closed_at.is_some(), "closed_at populated");

  // No NEW sanction row inserted by the appeal verdict.
  let sanctions_after: i64 = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    sanctions_after, 0,
    "appeal verdict creates no sanction (PRD §4.1 v1 simplification + ADR-014)"
  );

  // Drive run_appeal_window_expiry_batch directly — case is already Closed,
  // so the filter (status = Decided) does not match → no-op.
  let outcome = run_appeal_window_expiry_batch(&context).await?;
  assert_eq!(
    outcome.cases_processed, 0,
    "appeal_window_expiry on Closed case is a no-op"
  );

  // cr-9: restore BREHON_DISABLE_APPEAL_WINDOW_JOB to its pre-test state so
  // other tests in the same --test-threads=1 process don't inherit the flag.
  unsafe {
    match prev_appeal_window_disable {
      Some(val) => std::env::set_var("BREHON_DISABLE_APPEAL_WINDOW_JOB", val),
      None => std::env::remove_var("BREHON_DISABLE_APPEAL_WINDOW_JOB"),
    }
  }

  Ok(())
}

/// v1-JM-e Task 3 — audit-log invariant (PRD §6.7 state-machine prefix).
///
/// Drives a full original-decide → request_appeal → appeal-decide lifecycle
/// against a case opened via `create_report` (×3 reporters to cross v0
/// threshold), captures every governance_log row for the case in
/// chronological order, and asserts the **first occurrence** of each
/// distinct entry_kind matches the PRD §6.7 prefix:
///
///     report_created → threshold_met → jury_assigned → panel_assembled →
///     case_decided → appeal_requested → appeal_panel_assembled → appeal_decided
///
/// `jury_voted` and `jury_vote_submitted` rows are filtered out — they
/// fire once per juror per panel (5 original + 5 appeal) and are not
/// part of the lifecycle-shape invariant under test. `report_created`
/// fires 3 times (once per reporter); the assertion uses a "first-seen
/// per kind" filter so the recurrence does not break the prefix shape.
///
/// Authored by the advisor session per PMD #117 (Junior workers hang on
/// Edit calls into this 9000+ line file).
#[tokio::test(flavor = "multi_thread")]
async fn governance_log_sequence_matches_prd_state_machine() -> lemmy_utils::error::LemmyResult<()>
{
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::submit_jury_vote::submit_jury_vote;
  use lemmy_api_common::governance::SubmitJuryVote;
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{enums::JuryDecision, schema::governance_log};
  use lemmy_db_views_local_user::LocalUserView;
  use std::collections::HashSet;

  // Disable the appeal-window-expiry background scheduler to avoid races
  // between the test-driven flow and the cron tick.
  // SAFETY: e2e tests run with --test-threads=1 (LazyLock SETTINGS singleton),
  // so this set_var is effectively single-threaded for the test process.
  // cr-9 round 2 (CR re-review): capture prev value so we restore at test
  // end (mirror of the capstone test fix at line ~9415).
  let prev_appeal_window_disable = std::env::var_os("BREHON_DISABLE_APPEAL_WINDOW_JOB");
  unsafe {
    std::env::set_var("BREHON_DISABLE_APPEAL_WINDOW_JOB", "1");
  }

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jme3", true).await?;
  let (target, target_view) =
    governance_fixtures::seed_user(&context, instance.id, "target_jme3", false).await?;

  // 4 reporters (distinct from jurors so case.creator_id doesn't collide
  // with the jury-accept caller-exclusion at accept_jury_assignment.rs:141).
  // 4 not 3 because case_threshold_micros default is 3_000_000 and the
  // check is strict `>` — see seed_appealed_case_with_panel_via_report
  // for the threshold math.
  let mut reporter_views = Vec::with_capacity(4);
  for i in 0..4 {
    let (_, view) =
      governance_fixtures::seed_user(&context, instance.id, &format!("reporter_jme3_{i}"), false)
        .await?;
    reporter_views.push(view);
  }

  // 13 jurors: 5 original + 8 appeal panel.
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 13).await?;

  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  // Seed via the report-driven sibling fixture so report_created +
  // threshold_met log entries are emitted by the real handler.
  let (case_id, _appeal_id, appeal_panel_ids) =
    v1_jm_e_fixtures::seed_appealed_case_with_panel_via_report(
      &context,
      &db_url,
      target,
      target_view,
      admin_view,
      &reporter_views,
      &jurors,
      federation_context.reset_request_count(),
    )
    .await?;

  // 8 appeal-panel jurors accept (handler recognises Appeal role per JM-d).
  for juror_id in &appeal_panel_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    lemmy_api::governance::accept_jury_assignment::accept_jury_assignment(
      Json(lemmy_api_common::governance::AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?;
  }

  // 5 of 8 vote AdvisoryLabel (threshold = ceil(0.6 × 8) = 5 for Moderate).
  let voting_jurors: Vec<_> = appeal_panel_ids.iter().take(5).copied().collect();
  for (i, juror_id) in voting_jurors.iter().enumerate() {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::AdvisoryLabel,
        rationale: Some(format!("audit-invariant appeal vote {i}")),
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?;
  }

  // Capture every governance_log row in chronological order. Client-side
  // filter on payload.case_id avoids needing diesel-async JSONB containment
  // surface (per Plan §10.7 GOTCHA — alternative client-side filter is the
  // supported path). Per-test DB has only one case so the filter walks ~30
  // rows max.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let all_entries: Vec<(String, serde_json::Value)> = governance_log::table
    .order_by(governance_log::id.asc())
    .select((governance_log::entry_kind, governance_log::payload))
    .load(&mut conn)
    .await?;

  let case_id_i64 = i64::from(case_id.0);
  // Build the chronological "first occurrence per kind" sequence for this
  // case, with vote-fan-out kinds filtered out.
  let mut seen: HashSet<&str> = HashSet::new();
  let mut sequence: Vec<&str> = Vec::new();
  for (kind, payload) in &all_entries {
    let row_case_id = payload.get("case_id").and_then(serde_json::Value::as_i64);
    if row_case_id != Some(case_id_i64) {
      continue;
    }
    let k = kind.as_str();
    if k == "jury_voted" || k == "jury_vote_submitted" {
      continue;
    }
    if seen.insert(k) {
      sequence.push(k);
    }
  }

  // Empirically-verified PRD §6.7 state-machine prefix as emitted by current
  // handlers. The plan §10.7 spec lists the original 8-entry shape (the
  // "happy-path lifecycle" view), but the actual state machine also emits:
  //
  //   - severity_tier_frozen (between threshold_met and jury_assigned) —
  //     v1-JM-a snapshot of moderation_case.severity_tier at admin_assign
  //     time per PRD §9.2
  //   - jury_accepted (between panel_assembled and case_decided) — first
  //     juror's accept_jury_assignment (Phase 5c task 64). Subsequent
  //     accepts also emit this kind but the first-occurrence filter
  //     collapses them.
  //   - public_log_published (between jury_accepted and vote_outcome_recorded,
  //     i.e. BEFORE case_decided) — redacted public log entry created on
  //     case-decide (Phase 4b shipped, submit_jury_vote.rs)
  //   - vote_outcome_recorded (between public_log_published and case_decided) —
  //     RT-r3 (996765cae) per-vote outcome emit on submit_jury_vote; first
  //     occurrence is the decision-time write.
  //
  // Test catches future state-machine drift (a new const dropping in or an
  // existing emission disappearing). The plan §10.7 spec is the
  // "lifecycle-shape" view; this is the "every-emission" view.
  let expected_prefix = vec![
    "report_created",
    "threshold_met",
    "severity_tier_frozen",
    "jury_assigned",
    "panel_assembled",
    "jury_accepted",
    "public_log_published",
    "vote_outcome_recorded",
    "case_decided",
    "appeal_requested",
    "appeal_panel_assembled",
    "appeal_decided",
  ];

  assert_eq!(
    sequence, expected_prefix,
    "governance_log first-occurrence sequence must match PRD §6.7 state-machine prefix; got {sequence:?}"
  );

  // cr-9 round 2: restore BREHON_DISABLE_APPEAL_WINDOW_JOB (mirror of the
  // capstone test cleanup at line ~9596).
  unsafe {
    match prev_appeal_window_disable {
      Some(val) => std::env::set_var("BREHON_DISABLE_APPEAL_WINDOW_JOB", val),
      None => std::env::remove_var("BREHON_DISABLE_APPEAL_WINDOW_JOB"),
    }
  }

  Ok(())
}

/// v1-JM-e Task 4 — config-churn regression: appeal.window_days.
///
/// Asserts that flipping `appeal.window_days` AFTER cases A + B reach
/// `Decided` does NOT retroactively change their `appeal_window_expires_at`.
/// Case C (decided AFTER the flip) gets the new window. This is the
/// inverse of the JM-c happy-path snapshot test
/// (`v0_case_completes_under_v0_rules_after_v1_config_flip` at e2e.rs:8689),
/// which asserted the deliberate exception that `appeal.window_days` reads
/// LIVE config at decision time. JM-e Task 4 makes the consequence explicit:
/// once a case is Decided, its `appeal_window_expires_at` is FROZEN — a
/// later config flip cannot retroactively expire (or extend) the window.
///
/// Sequence:
///   1. Seed 5 jurors + admin + 3 targets (A, B, C). Use Minor severity
///      (panel = 5, threshold = 3).
///   2. Drive case A through report → admin_assign → 5 accepts → 3
///      RemoveContent votes → Decided. `appeal_window_expires_at - decided_at
///      ≈ 7 days` (default).
///   3. Drive case B identically. Same gap.
///   4. Flip `appeal.window_days = 30` via `admin_set_config` (instance
///      scope).
///   5. Drive case C through the same lifecycle. New gap ≈ 30 days.
///   6. Re-read A + B from DB AFTER C completes; confirm their gaps are
///      STILL ≈ 7 days (not retroactively extended to 30).
///
/// Authored by the advisor session per PMD #117 (Junior workers hang on
/// Edit calls into this 9000+ line file). Mirrors the JM-c v0-compat
/// pattern's lifecycle drive but in sibling-test form (per plan §13
/// "GOTCHA: sibling test, NOT in-place patch").
#[tokio::test(flavor = "multi_thread")]
async fn config_churn_appeal_window_days_does_not_invalidate_decided_cases()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use chrono::Duration;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    admin_config::admin_set_config, submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, AdminSetConfig, SubmitJuryVote,
  };
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{CaseStatus, JuryDecision, SeverityTier},
    schema::moderation_case,
  };
  use lemmy_db_views_local_user::LocalUserView;

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let _community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jme4", true).await?;
  let (target_a, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jme4_a", false).await?;
  let (target_b, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jme4_b", false).await?;
  let (target_c, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jme4_c", false).await?;
  // 5 jurors — same panel can serve all three cases (no eligibility-filter
  // collision; previously-decided cases don't bar a juror from a new panel).
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 5).await?;

  // R2: seed jury_eligible snapshots BEFORE admin_assign_jury (per JM-b
  // discipline; without this the small-pool fallback fires).
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    v1_jm_b_fixtures::seed_jury_eligible_snapshots(&mut conn, &jurors).await?;
  }

  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  // Helper closure to drive one case from seed → Decided. Reused 3×.
  // Returns the case_id so the test body can re-read decided_at +
  // appeal_window_expires_at from the DB.
  // NOTE: async closure (not async fn) so it inherits the outer scope's
  // `use` imports for LemmyContext, PersonId, etc.
  let drive_case_to_decided = async |target: lemmy_db_schema_file::PersonId| -> lemmy_utils::error::LemmyResult<
    lemmy_db_schema::newtypes::ModerationCaseId,
  > {
    let case_id = v1_jm_b_fixtures::seed_case(&db_url, target, SeverityTier::Minor)
      .await
      .map_err(|e| anyhow::anyhow!("seed_case: {e}"))?;
    let assign_resp = admin_assign_jury(
      Json(AdminAssignJury { case_id }),
      context.clone(),
      admin_view.clone(),
    )
    .await?
    .into_inner();
    assert_eq!(
      assign_resp.assigned_person_ids.len(),
      5,
      "Minor panel = 5 jurors per JM-b snapshot"
    );
    for juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await?;
    }
    // Cast 3 RemoveContent votes — Minor threshold = 3 → case Decided on
    // the 3rd vote.
    for juror_id in assign_resp.assigned_person_ids.iter().take(3) {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
      submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision: JuryDecision::RemoveContent,
          rationale: None,
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await?;
    }
    Ok(case_id)
  };

  // Step 2 + 3: drive A + B to Decided BEFORE the config flip.
  let case_a = drive_case_to_decided(target_a).await?;
  let case_b = drive_case_to_decided(target_b).await?;

  // Snapshot A + B's appeal_window gap RIGHT NOW so we have a baseline
  // independent of any later mutation. Both should be ≈ 7 days (default
  // appeal.window_days).
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let read_gap = async |conn: &mut AsyncPgConnection,
                        case_id: lemmy_db_schema::newtypes::ModerationCaseId|
         -> lemmy_utils::error::LemmyResult<(Duration, CaseStatus)> {
    let (status, decided_at, expires): (
      CaseStatus,
      Option<chrono::DateTime<chrono::Utc>>,
      Option<chrono::DateTime<chrono::Utc>>,
    ) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::decided_at,
        moderation_case::appeal_window_expires_at,
      ))
      .first(conn)
      .await?;
    let decided = decided_at.expect("decided_at set");
    let expires_at = expires.expect("appeal_window_expires_at set");
    Ok((expires_at - decided, status))
  };

  let (gap_a_before, status_a_before) = read_gap(&mut conn, case_a).await?;
  let (gap_b_before, status_b_before) = read_gap(&mut conn, case_b).await?;
  assert_eq!(status_a_before, CaseStatus::Decided, "case A Decided");
  assert_eq!(status_b_before, CaseStatus::Decided, "case B Decided");
  let default_window = Duration::days(7);
  assert!(
    (gap_a_before - default_window).num_milliseconds().abs() < 1_000,
    "case A appeal_window ≈ 7 days at decision (default); got {gap_a_before:?}"
  );
  assert!(
    (gap_b_before - default_window).num_milliseconds().abs() < 1_000,
    "case B appeal_window ≈ 7 days at decision (default); got {gap_b_before:?}"
  );

  // Step 4: flip appeal.window_days = 30 (instance scope).
  admin_set_config(
    Json(AdminSetConfig {
      key: "appeal.window_days".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(30),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "Task 4 config-churn regression test".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  // Step 5: drive case C to Decided AFTER the flip. Should pick up the
  // new live value.
  let case_c = drive_case_to_decided(target_c).await?;

  // Step 6: re-read A + B (their gaps must NOT have moved) and read C
  // (its gap should be ≈ 30 days).
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let (gap_a_after, _) = read_gap(&mut conn, case_a).await?;
  let (gap_b_after, _) = read_gap(&mut conn, case_b).await?;
  let (gap_c, status_c) = read_gap(&mut conn, case_c).await?;
  assert_eq!(status_c, CaseStatus::Decided, "case C Decided");

  let new_window = Duration::days(30);
  assert!(
    (gap_c - new_window).num_milliseconds().abs() < 1_000,
    "case C appeal_window ≈ 30 days (LIVE config at decision time per JM-c discovery); got {gap_c:?}"
  );
  assert_eq!(
    gap_a_after.num_milliseconds(),
    gap_a_before.num_milliseconds(),
    "case A appeal_window UNCHANGED by appeal.window_days flip (frozen at decision time)"
  );
  assert_eq!(
    gap_b_after.num_milliseconds(),
    gap_b_before.num_milliseconds(),
    "case B appeal_window UNCHANGED by appeal.window_days flip (frozen at decision time)"
  );

  Ok(())
}

// ============================================================================
// v1-JM-e Task 5 — §12 security cluster: admin-visibility + spoofing-protection
// ============================================================================
//
// Bundles two §12 assertions per plan §10.9 + §10.10:
//
// (1) §12.2 admin-visibility — when a small-pool case fires R1 relaxation
//     (4 jurors available, Minor severity needs 5), `admin_assign_jury`
//     writes a `jury_constraint_violation_log` row. A community admin
//     issuing a Diesel query joining `jury_constraint_violation_log` ⨝
//     `moderation_case` on `community_id` must see that row.
//
// (2) §12.4 spoofing-protection — `request_appeal.rs:120-131`
//     elif-branch falls through to `Err(LemmyErrorType::NotFound)` when
//     the caller is neither defendant nor original-reporter. An orphaned
//     Decided case (`creator_id = NULL`) must reject `request_appeal`
//     from any non-defendant caller — the original-reporter branch
//     can't fire because there's no creator to match.
//
// Authored as a sibling test (NOT in-place patch) per PMD #117. Uses
// the existing `governance_fixtures::*` and `v1_jm_b_fixtures::*`
// helpers, mirrors the small-pool R1 setup at e2e.rs:7730 and the
// orphaned-case `ModerationCaseInsertForm` direct-insert pattern at
// e2e.rs:4381.
#[tokio::test(flavor = "multi_thread")]
async fn constraint_relaxation_visible_to_community_admin_orphan_case_blocks_spoofing()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use chrono::{Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_assign_jury::admin_assign_jury;
  use lemmy_api_common::governance::{AdminAssignJury, RequestAppeal};
  use lemmy_api_crud::governance::request_appeal::request_appeal;
  use lemmy_db_schema::source::{
    governance::moderation_case::ModerationCaseInsertForm, instance::Instance,
  };
  use lemmy_db_schema_file::{
    enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryDecision, SeverityTier},
    schema::{jury_constraint_violation_log, moderation_case},
  };

  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let community = governance_fixtures::seed_community(&context, instance.id).await?;
  let (_, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "admin_jme5", true).await?;
  let (target_relax, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jme5_relax", false).await?;
  let (target_orphan, _) =
    governance_fixtures::seed_user(&context, instance.id, "target_jme5_orphan", false).await?;
  let (_, spoofer_view) =
    governance_fixtures::seed_user(&context, instance.id, "spoofer_jme5", false).await?;

  // ============================================================================
  // (1) §12.2 admin-visibility: small-pool case → R1 relaxation row →
  //     community-admin Diesel query sees it.
  // ============================================================================

  // Seed only 4 jurors — Minor severity needs 5 → small_pool relaxation
  // fires per JM-b R1.
  let jurors = governance_fixtures::seed_jurors(&context, instance.id, 4).await?;
  {
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    // Scope eligibility to the community (case is community-scoped, so
    // the strict eligibility join requires community_id-matched rows).
    v1_jm_b_fixtures::seed_jury_eligible_snapshots_scoped(&mut conn, &jurors, Some(community.id))
      .await?;
  }

  // Insert a community-scoped Minor case (the §12.2 query filter is
  // `community_id = $admin_community`).
  let small_pool_case_id = {
    let form = ModerationCaseInsertForm {
      community_id: Some(community.id),
      creator_id: None,
      target_type: CaseTargetType::Person,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: Some(target_relax),
      target_community_id: None,
      target_remote_url: None,
      reason_code: "small_pool_admin_visibility".to_string(),
      severity: CaseSeverity::Low,
      severity_tier: Some(SeverityTier::Minor),
      status: CaseStatus::Open,
      threshold_score: 1,
      ..Default::default()
    };
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    diesel::insert_into(moderation_case::table)
      .values(&form)
      .returning(moderation_case::id)
      .get_result::<lemmy_db_schema::newtypes::ModerationCaseId>(&mut conn)
      .await?
  };

  // Drive admin_assign_jury — small_pool relaxation fires (4 < 5
  // eligible) → writes jury_constraint_violation_log row.
  admin_assign_jury(
    Json(AdminAssignJury {
      case_id: small_pool_case_id,
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  // §12.2: community-admin query — count relaxation rows for cases in
  // their community via eq_any(subquery). Per plan §10.10 pattern.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let admin_visible_relaxations: i64 = jury_constraint_violation_log::table
    .filter(
      jury_constraint_violation_log::case_id.eq_any(
        moderation_case::table
          .filter(moderation_case::community_id.eq(Some(community.id)))
          .select(moderation_case::id),
      ),
    )
    .count()
    .get_result(&mut conn)
    .await?;
  assert!(
    admin_visible_relaxations > 0,
    "§12.2: community-admin Diesel query must see relaxation rows for cases in their community; got {admin_visible_relaxations}"
  );

  // ============================================================================
  // (2) §12.4 spoofing-protection: orphaned Decided case → request_appeal
  //     from non-defendant non-creator returns NotFound.
  // ============================================================================

  // Insert an orphaned Decided case (creator_id = NULL, no community).
  // Mirrors the JM-d orphan pattern at e2e.rs:4381 — the request_appeal
  // status guard requires Decided + non-expired window +
  // panel_size_snapshot, so we UPDATE those after insert.
  let orphan_case_id = {
    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::Person,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: Some(target_orphan),
      target_community_id: None,
      target_remote_url: None,
      reason_code: "orphan_appeal_spoof".to_string(),
      severity: CaseSeverity::Low,
      severity_tier: Some(SeverityTier::Minor),
      status: CaseStatus::Decided,
      threshold_score: 1,
      ..Default::default()
    };
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let case_id = diesel::insert_into(moderation_case::table)
      .values(&form)
      .returning(moderation_case::id)
      .get_result::<lemmy_db_schema::newtypes::ModerationCaseId>(&mut conn)
      .await?;
    let future = Utc::now() + Duration::days(7);
    diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id)))
      .set((
        moderation_case::appeal_window_expires_at.eq(Some(future)),
        moderation_case::panel_size_snapshot.eq(Some(5_i32)),
        // Set winning_decision to a value that WOULD enable the
        // OriginalReporter branch IF the caller had been the creator —
        // the test then proves the spoofer (not creator, not defendant)
        // still falls through to NotFound.
        moderation_case::winning_decision.eq(Some(JuryDecision::NoAction)),
      ))
      .execute(&mut conn)
      .await?;
    case_id
  };

  // Spoofer is neither target_orphan nor case.creator (NULL). The
  // request_appeal eligibility branch must fall through to NotFound.
  // cr-18: assert specifically LemmyErrorType::NotFound — `is_err()` alone
  // would also pass on a pre-eligibility DB error (e.g. case-not-found,
  // window-expired); the variant match anchors the test to the
  // pseudo-403 spoofing-protection branch at request_appeal.rs:130.
  let resp = request_appeal(
    Json(RequestAppeal {
      case_id: orphan_case_id,
      reason: "spoof attempt".to_string(),
    }),
    context.clone(),
    spoofer_view,
  )
  .await;
  let err = resp.expect_err(
    "§12.4: orphaned-case (creator_id=NULL) appeal-rights cannot be spoofed by a non-defendant; request_appeal must Err(NotFound)"
  );
  assert!(
    matches!(
      &err.error_type,
      lemmy_utils::error::LemmyErrorType::NotFound
    ),
    "§12.4: expected LemmyErrorType::NotFound on orphan-case spoof, got {:?}",
    err.error_type,
  );

  Ok(())
}

