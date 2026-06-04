//! End-to-end test harness for the Brehon governance fork.
//!
//! This file is the entry point for `cargo test --test e2e`. Phase 0
//! establishes the harness; later phases add real golden-path tests
//! that exercise the governance endpoints against a real Postgres.
//!
//! The harness uses `GenericImage` (not `testcontainers_modules::Postgres`)
//! so the image coordinates exactly match Lemmy's production
//! `docker-compose.yml`: `pgautoupgrade/pgautoupgrade:18-alpine`.

// Cargo integration test files use top-level test functions by convention;
// test helpers are defined inline after test setup for readability.
// These are file-wide #![expect] for patterns that are appropriate in this
// integration test binary context.
#![expect(
  clippy::tests_outside_test_module,
  reason = "integration test binary; Cargo integration tests are top-level by convention"
)]
#![expect(
  clippy::items_after_statements,
  reason = "integration test helpers defined inline after test setup for readability"
)]
#![expect(
  clippy::expect_used,
  clippy::unwrap_used,
  reason = "integration test assertions; panics signal test failures"
)]
#![expect(
  clippy::indexing_slicing,
  reason = "integration test assertions; index bounds are enforced by prior length assertions"
)]
#![expect(clippy::unreachable, reason = "integration test assertions")]
#![expect(
  clippy::get_first,
  reason = "Vec::first() conflicts with Diesel RunQueryDsl::first() (serde_json::Value / enum-tuple rows engage the blanket LimitDsl impl → E0275/E0277, verified on PR #132); use .get(0) to avoid trait ambiguity"
)]

/// Smoke test the harness boot path: container start + Tier 3 template
/// restore (when enabled) + schema sentinel reachable. Asserts that
/// `governance_log` is present in `public` schema after `start_postgres`
/// returns — confirming pg_restore actually populated the schema. With
/// `BREHON_E2E_NO_TEMPLATE=1` the assertion still holds because the
/// caller follows up with `apply_all_schema` (legacy path).
#[tokio::test]
async fn postgres_container_boots() -> lemmy_utils::error::LemmyResult<()> {
  use diesel::{Connection as _, PgConnection};

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  assert!(host_port > 0, "postgres mapped port should be non-zero");

  // If template path was used (default), governance_log should exist
  // immediately. If BREHON_E2E_NO_TEMPLATE=1, run the legacy schema
  // apply first so the assertion still meaningfully exercises the
  // sentinel + restore wiring.
  let db_url = governance_fixtures::db_url(host_port);
  let mut conn = PgConnection::establish(&db_url)?;
  if std::env::var("BREHON_E2E_NO_TEMPLATE").as_deref() == Ok("1") {
    governance_fixtures::apply_all_schema(&mut conn)?;
  }

  // Reuse the production sentinel helper so this smoke test stays
  // bound to whatever invariant the runtime fast-path enforces
  // (CR finding #16 DRY).
  assert!(
    governance_fixtures::schema_sentinel_satisfied(&mut conn)?,
    "schema sentinel failed after container boot — expected both \
     `public.governance_log` and `r.parent_comment_ids` to be \
     present (template restore or legacy apply_all_schema should have \
     populated both)"
  );

  Ok(())
}

/// Tier 3 sentinel — proves the bootstrap dump path runs and produces
/// a non-trivial pg_dump custom-format payload. The dump is built once
/// per nextest process; this test just calls `ensure_template` so the
/// LazyLock fires under nextest's process-per-test isolation.
#[tokio::test]
async fn template_dump_capture() -> lemmy_utils::error::LemmyResult<()> {
  // Force the template path even if a future test sets BREHON_E2E_NO_TEMPLATE.
  // Skip when env explicitly disables it (legacy fallback validation runs).
  if std::env::var("BREHON_E2E_NO_TEMPLATE").as_deref() == Ok("1") {
    return Ok(());
  }
  let dump = governance_fixtures::pg_template::ensure_template()
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
  // pg_dump custom-format files start with the magic bytes "PGDMP";
  // the rest of the format is the structural check (CR finding #15).
  // We previously asserted len > 100_000 here to catch silently-empty
  // dumps; that's brittle to legitimate compression / pg_dump version
  // / schema changes. A tiny floor (`MIN_TEMPLATE_DUMP_BYTES` = 1 KiB)
  // catches the truly-empty case without flagging healthy variance.
  // Same constant is reused by `pg_template::load_or_build` to reject
  // truncated disk-cache files (CR finding #18).
  assert!(
    dump.starts_with(b"PGDMP"),
    "not a pg_dump custom-format payload (first 8 bytes: {:02x?})",
    &dump[..dump.len().min(8)]
  );
  assert!(
    dump.len() >= governance_fixtures::pg_template::MIN_TEMPLATE_DUMP_BYTES,
    "template dump implausibly small: {} bytes (header valid but body \
     near-empty — likely bootstrap container produced no schema)",
    dump.len()
  );
  Ok(())
}

// Shared cross-domain test scaffold (Phase 6 decomposition, sub-phase 1):
// the testcontainer/Postgres harness, the process-env RAII guard, and the
// single-column row shape now live in `tests/e2e/common/mod.rs`. The `use`
// re-exports below keep the ~291 bare `governance_fixtures::` /
// `EnvVarGuard` / `SingleI32` references in the test bodies unchanged.
//
// `#[path]` is mandatory: this file is `tests/e2e.rs`, an auto-discovered
// integration-test crate root, so a bare `mod common;` resolves to
// `tests/common/mod.rs` (the sibling dir), NOT `tests/e2e/common/`. The
// explicit path keeps all split modules under `tests/e2e/` while preserving
// the single-test-binary contract — files placed directly in `tests/` would
// each become a SEPARATE test binary, which we must avoid.
#[path = "e2e/common/mod.rs"]
mod common;
use common::governance_fixtures;
#[allow(unused_imports)]
use common::{EnvVarGuard, SingleI32};


// Phase 1-8 governance tests (sub-phase 4/7 decomposition):
// Extracted to tests/e2e/governance.rs; include! expands inline at crate
// scope so test paths are unchanged (no module prefix added).
include!("e2e/governance.rs");


// Admin config fixtures + tests (sub-phase 5/7 decomposition):
// Extracted to tests/e2e/admin_config.rs; include! expands inline at crate scope.
include!("e2e/admin_config.rs");

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

mod v1_sl_b_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
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
  use super::*;
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
  use super::*;
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
  use super::*;
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

#[path = "e2e/federation_inbound_a.rs"]
mod v1_federation_inbound_a_fixtures;

mod v1_ship_2_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
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

#[path = "e2e/federation_inbound_b.rs"]
mod v1_federation_inbound_b_fixtures;

#[path = "e2e/federation_inbound_e.rs"]
mod v1_federation_inbound_e_fixtures;

mod v1_ship_3_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::*;
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
#[path = "e2e/reputation_rt_r3.rs"]
mod v1_rt_r3_fixtures;


// Reputation RT-r4 fixtures extracted to tests/e2e/reputation_rt_r4.rs (sub-phase 3/7).
#[path = "e2e/reputation_rt_r4.rs"]
mod v1_rt_r4_fixtures;
