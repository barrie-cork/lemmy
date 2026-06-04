//! v1-RT-r4 sponsor-gate strategy + admin allowlist e2e fixtures.
//!
//! 2 stories / 7 tests per plan §16a:
//!   Story A — strategy arms gate endorsement correctly (6 tests):
//!     age_or_surety pass + deny, reputation pass + deny, allowlist pass + deny
//!   Story B — admin allowlist add/remove round-trip + governance log (1 test)
//!
//! Case A error shape: outer `LemmyResult<()>` + helpers `LemmyResult<T>`.
//! Mirror: v1_rt_r3_fixtures (e2e.rs:17107) for boot + helper shape.

use crate::common::governance_fixtures;
use actix_web::web::Json;
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
use lemmy_api::governance::admin_sponsor_allowlist::{add, remove};
use lemmy_api_common::governance::{AddSponsorAllowlist, CreateEndorsement, RemoveSponsorAllowlist};
use lemmy_api_crud::governance::create_endorsement::create_endorsement;
use lemmy_db_schema::source::{
  governance::{
    governance_config::GovernanceConfigInsertForm,
    reputation_snapshot::ReputationSnapshotInsertForm,
    sponsor_allowlist::sponsor_allowlist_exists,
    surety::SuretyInsertForm,
  },
  instance::Instance,
};
use lemmy_db_schema_file::schema::{
  governance_config, governance_log, reputation_snapshot as rs_table, surety,
};
use lemmy_utils::error::{LemmyErrorType, LemmyResult};

// ──────────────────────────── helpers ────────────────────────────

/// Insert a `governance_config` row setting `onboarding.sponsor_gate_strategy`.
/// A new row with `DEFAULT now()` beats any seed row (ORDER BY valid_from DESC LIMIT 1).
async fn set_gate_strategy(conn: &mut AsyncPgConnection, strategy: &str) -> LemmyResult<()> {
  diesel::insert_into(governance_config::table)
    .values(&GovernanceConfigInsertForm {
      scope: "instance".to_string(),
      key: "onboarding.sponsor_gate_strategy".to_string(),
      value_type: "text".to_string(),
      value_text: Some(strategy.to_string()),
      ..Default::default()
    })
    .execute(conn)
    .await?;
  Ok(())
}

// ──────────────────────────── Story A — strategy gate tests ────────────────────────────

/// age_or_surety: fresh account (age 0) has a surety row where it is the
/// sponsored party → age gate fails but surety count > 0 → gate passes.
#[tokio::test(flavor = "multi_thread")]
async fn age_or_surety_gate_passes_via_surety() -> LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  let (sponsor_id, sponsor_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_aos_pass_sponsor", false).await?;
  let (sponsee_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_aos_pass_sponsee", false).await?;
  let (super_sponsor_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_aos_pass_super", false).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  set_gate_strategy(&mut conn, "age_or_surety").await?;
  // super_sponsor vouches for the fresh sponsor (sponsored_id = sponsor_id, revoked_at IS NULL).
  diesel::insert_into(surety::table)
    .values(&SuretyInsertForm {
      sponsor_id: super_sponsor_id,
      sponsored_id: sponsor_id,
      community_id: None,
    })
    .execute(&mut conn)
    .await?;
  drop(conn);

  let result = create_endorsement(
    Json(CreateEndorsement { person_id: sponsee_id, community_id: None }),
    context.clone(),
    sponsor_view,
  )
  .await;
  assert!(result.is_ok(), "age_or_surety: fresh sponsor with active surety should pass");
  Ok(())
}

/// age_or_surety: fresh account with no surety row → both age and surety
/// arms fail → gate denies.
#[tokio::test(flavor = "multi_thread")]
async fn age_or_surety_gate_denies_without_surety() -> LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  let (_sponsor_id, sponsor_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_aos_deny_sponsor", false).await?;
  let (sponsee_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_aos_deny_sponsee", false).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  set_gate_strategy(&mut conn, "age_or_surety").await?;
  drop(conn);

  let err = create_endorsement(
    Json(CreateEndorsement { person_id: sponsee_id, community_id: None }),
    context.clone(),
    sponsor_view,
  )
  .await
  .expect_err("age_or_surety: fresh sponsor with no surety should be denied");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotFound),
    "age_or_surety deny: expected LemmyErrorType::NotFound, got {:?}",
    err.error_type,
  );
  Ok(())
}

/// reputation: sponsor has a snapshot row with `can_sponsor = true` →
/// gate passes.
#[tokio::test(flavor = "multi_thread")]
async fn reputation_gate_passes_with_can_sponsor() -> LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  let (sponsor_id, sponsor_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_rep_pass_sponsor", false).await?;
  let (sponsee_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_rep_pass_sponsee", false).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  set_gate_strategy(&mut conn, "reputation").await?;
  diesel::insert_into(rs_table::table)
    .values(&ReputationSnapshotInsertForm {
      person_id: sponsor_id,
      community_id: None,
      can_sponsor: true,
      reporting_accuracy: 0,
      jury_reliability: 0,
      participation_consistency: 0,
      endorsement_strength: 0,
      jury_eligible: false,
      trusted_reporter: false,
    })
    .execute(&mut conn)
    .await?;
  drop(conn);

  let result = create_endorsement(
    Json(CreateEndorsement { person_id: sponsee_id, community_id: None }),
    context.clone(),
    sponsor_view,
  )
  .await;
  assert!(result.is_ok(), "reputation: sponsor with can_sponsor=true should pass");
  Ok(())
}

/// reputation: no snapshot row for sponsor → `can_sponsor.unwrap_or(false)`
/// is false → gate denies.
#[tokio::test(flavor = "multi_thread")]
async fn reputation_gate_denies_without_snapshot() -> LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  let (_sponsor_id, sponsor_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_rep_deny_sponsor", false).await?;
  let (sponsee_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_rep_deny_sponsee", false).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  set_gate_strategy(&mut conn, "reputation").await?;
  drop(conn);

  let err = create_endorsement(
    Json(CreateEndorsement { person_id: sponsee_id, community_id: None }),
    context.clone(),
    sponsor_view,
  )
  .await
  .expect_err("reputation: sponsor with no snapshot should be denied");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotFound),
    "reputation deny: expected LemmyErrorType::NotFound, got {:?}",
    err.error_type,
  );
  Ok(())
}

/// allowlist: sponsor has been admin-added to the allowlist → gate passes.
#[tokio::test(flavor = "multi_thread")]
async fn allowlist_gate_passes_when_on_list() -> LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  let (sponsor_id, sponsor_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_al_pass_sponsor", false).await?;
  let (sponsee_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_al_pass_sponsee", false).await?;
  let (_admin_id, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_al_pass_admin", true).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  set_gate_strategy(&mut conn, "allowlist").await?;
  drop(conn);

  add(
    Json(AddSponsorAllowlist { person_id: sponsor_id, community_id: None, note: None }),
    context.clone(),
    admin_view,
  )
  .await?;

  let result = create_endorsement(
    Json(CreateEndorsement { person_id: sponsee_id, community_id: None }),
    context.clone(),
    sponsor_view,
  )
  .await;
  assert!(result.is_ok(), "allowlist: sponsor on allowlist should pass");
  Ok(())
}

/// allowlist: sponsor has NOT been added to the allowlist →
/// `sponsor_allowlist_exists` returns false → gate denies.
#[tokio::test(flavor = "multi_thread")]
async fn allowlist_gate_denies_when_not_on_list() -> LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  let (_sponsor_id, sponsor_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_al_deny_sponsor", false).await?;
  let (sponsee_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_al_deny_sponsee", false).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  set_gate_strategy(&mut conn, "allowlist").await?;
  drop(conn);

  let err = create_endorsement(
    Json(CreateEndorsement { person_id: sponsee_id, community_id: None }),
    context.clone(),
    sponsor_view,
  )
  .await
  .expect_err("allowlist: sponsor not on allowlist should be denied");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotFound),
    "allowlist deny: expected LemmyErrorType::NotFound, got {:?}",
    err.error_type,
  );
  Ok(())
}

// ──────────────────────────── Story B — admin allowlist round-trip ────────────────────────────

/// Admin adds a person to the allowlist then removes them. Verifies the
/// add and remove governance-log entries are emitted.
#[tokio::test(flavor = "multi_thread")]
async fn admin_allowlist_add_remove_round_trip() -> LemmyResult<()> {
  let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  let (person_id, _) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_rtrip_person", false).await?;
  let (_admin_id, admin_view) =
    governance_fixtures::seed_user(&context, instance.id, "rt4_rtrip_admin", true).await?;

  let add_resp = add(
    Json(AddSponsorAllowlist {
      person_id,
      community_id: None,
      note: Some("round-trip test".to_string()),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?
  .into_inner();
  assert!(add_resp.allowlist_id.0 > 0, "allowlist_id must be positive");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let add_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("sponsor_allowlist_added"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(add_count, 1, "exactly one sponsor_allowlist_added governance log entry");

  let remove_resp = remove(
    Json(RemoveSponsorAllowlist { person_id, community_id: None }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();
  assert!(remove_resp.success, "remove must return success=true");

  let remove_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("sponsor_allowlist_removed"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(remove_count, 1, "exactly one sponsor_allowlist_removed governance log entry");

  // Verify the underlying row was actually deleted, not just the log entry.
  let still_exists = sponsor_allowlist_exists(person_id, None, &mut conn).await?;
  assert!(!still_exists, "sponsor_allowlist row must be absent after remove");

  Ok(())
}
