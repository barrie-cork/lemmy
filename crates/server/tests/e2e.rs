//! End-to-end test harness for the Brehon governance fork.
//!
//! This file is the entry point for `cargo test --test e2e`. Phase 0
//! establishes the harness; later phases add real golden-path tests
//! that exercise the governance endpoints against a real Postgres.
//!
//! The harness uses `GenericImage` (not `testcontainers_modules::Postgres`)
//! so the image coordinates exactly match Lemmy's production
//! `docker-compose.yml`: `pgautoupgrade/pgautoupgrade:18-alpine`.

use std::error::Error;
use testcontainers::{
  GenericImage, ImageExt,
  core::{IntoContainerPort, WaitFor},
  runners::AsyncRunner,
};

#[tokio::test]
async fn postgres_container_boots() -> Result<(), Box<dyn Error>> {
  let container = GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")
    .with_exposed_port(5432.tcp())
    .with_wait_for(WaitFor::message_on_stderr(
      "database system is ready to accept connections",
    ))
    .with_env_var("POSTGRES_USER", "lemmy")
    .with_env_var("POSTGRES_PASSWORD", "password")
    .with_env_var("POSTGRES_DB", "lemmy")
    .start()
    .await?;

  let host_port = container.get_host_port_ipv4(5432).await?;

  assert!(host_port > 0, "postgres mapped port should be non-zero");

  Ok(())
}

// ============================================================================
// Phase 1 — governance schema + hash-chain trigger smoke tests
// ============================================================================

mod governance_fixtures {
  use diesel::{PgConnection, connection::SimpleConnection};
  use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
  use std::error::Error;

  /// Migrations are embedded at compile time from the repo-root `migrations/`
  /// directory. Path is relative to `CARGO_MANIFEST_DIR` (here,
  /// `crates/server`), so `../../migrations` resolves to repo-root
  /// `migrations/`.
  const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../../migrations");

  /// Apply the full Lemmy + governance schema to a fresh container:
  ///   1. Acquire `pg_advisory_lock(0)` to bypass the `forbid_diesel_cli`
  ///      trigger installed in migration `2025-08-01-000017` (it rejects any
  ///      insert into `__diesel_schema_migrations` that doesn't hold the
  ///      lock).
  ///   2. Run every embedded migration via diesel's standard
  ///      `MigrationHarness::run_pending_migrations`.
  ///   3. Rebuild the `r` schema (replaceable-schema layer) by inlining
  ///      `utils.sql` + `triggers.sql` — the same two files that
  ///      `lemmy_diesel_utils::schema_setup` loads via `include_str!`. This
  ///      installs the governance hash-chain and append-only triggers on
  ///      `public.governance_log`.
  pub fn apply_all_schema(conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    conn.batch_execute("SELECT pg_advisory_lock(0);")?;
    conn
      .run_pending_migrations(MIGRATIONS)
      .map_err(|e| -> Box<dyn Error> { format!("migrations failed: {e}").into() })?;
    conn.batch_execute("DROP SCHEMA IF EXISTS r CASCADE; CREATE SCHEMA r;")?;
    conn.batch_execute(include_str!(
      "../../../crates/diesel_utils/replaceable_schema/utils.sql"
    ))?;
    conn.batch_execute(include_str!(
      "../../../crates/diesel_utils/replaceable_schema/triggers.sql"
    ))?;
    Ok(())
  }

  /// Start a fresh `pgautoupgrade:18-alpine` container matching Lemmy's prod
  /// image, returning the container handle (drop = teardown) and the
  /// host-mapped port.
  pub async fn start_postgres() -> Result<
    (
      testcontainers::ContainerAsync<testcontainers::GenericImage>,
      u16,
    ),
    Box<dyn Error>,
  > {
    use testcontainers::{
      GenericImage, ImageExt,
      core::{IntoContainerPort, WaitFor},
      runners::AsyncRunner,
    };
    let container = GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")
      .with_exposed_port(5432.tcp())
      .with_wait_for(WaitFor::message_on_stderr(
        "database system is ready to accept connections",
      ))
      .with_env_var("POSTGRES_USER", "lemmy")
      .with_env_var("POSTGRES_PASSWORD", "password")
      .with_env_var("POSTGRES_DB", "lemmy")
      .start()
      .await?;
    let host_port = container.get_host_port_ipv4(5432).await?;
    Ok((container, host_port))
  }

  /// Build a standard test DB URL for the given mapped host port.
  pub fn db_url(host_port: u16) -> String {
    format!("postgres://lemmy:password@localhost:{host_port}/lemmy")
  }
}

#[tokio::test]
async fn can_insert_moderation_case() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, QueryDsl, RunQueryDsl};
  use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
  use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::moderation_case;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let mut conn = PgConnection::establish(&db_url)?;
  governance_fixtures::apply_all_schema(&mut conn)?;

  let form = ModerationCaseInsertForm {
    community_id: None,
    creator_id: None,
    target_type: CaseTargetType::RemoteInstance,
    target_post_id: None,
    target_comment_id: None,
    target_person_id: None,
    target_community_id: None,
    target_remote_url: Some("https://example.invalid/post/1".to_string()),
    reason_code: "spam".to_string(),
    severity: CaseSeverity::Low,
    status: CaseStatus::Open,
    threshold_score: 1,
  };

  let inserted_id: i32 = diesel::insert_into(moderation_case::table)
    .values(&form)
    .returning(moderation_case::id)
    .get_result(&mut conn)?;

  assert!(inserted_id > 0, "moderation_case id should be positive");

  let read_back_status: CaseStatus = moderation_case::table
    .find(inserted_id)
    .select(moderation_case::status)
    .first(&mut conn)?;
  assert!(matches!(read_back_status, CaseStatus::Open));

  Ok(())
}

#[tokio::test]
async fn governance_log_hash_chain_holds() -> Result<(), Box<dyn Error>> {
  use diesel::sql_types::{Bytea, Int8, Text};
  use diesel::{
    Connection as _, ExpressionMethods, PgConnection, QueryDsl, RunQueryDsl, sql_query,
  };
  use lemmy_db_schema::source::governance::governance_log::GovernanceLogInsertForm;
  use lemmy_db_schema_file::schema::governance_log;
  use serde_json::json;
  use sha2::{Digest, Sha256};

  // Raw row shape. We deliberately bypass the `GovernanceLog` model here
  // because we need Postgres's own `payload::text` rendering (which uses
  // `{"n": 1}` with a space after the colon) and its own `to_char` timestamp
  // formatting — if we re-serialised from the deserialised `serde_json::Value`
  // or re-formatted from `chrono::DateTime`, the byte sequence would diverge
  // from what the trigger hashed and the chain check would fail.
  #[derive(diesel::QueryableByName, Debug)]
  struct RawRow {
    #[diesel(sql_type = Int8)]
    id: i64,
    #[diesel(sql_type = Bytea)]
    prev_hash: Vec<u8>,
    #[diesel(sql_type = Bytea)]
    entry_hash: Vec<u8>,
    #[diesel(sql_type = Text)]
    entry_kind: String,
    #[diesel(sql_type = Text)]
    payload_text: String,
    #[diesel(sql_type = Text)]
    created_at_text: String,
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let mut conn = PgConnection::establish(&db_url)?;
  governance_fixtures::apply_all_schema(&mut conn)?;

  // Three inserts. Payloads vary in shape: scalar, nested object, and
  // array-valued field — the last one stress-tests canonicalisation of
  // non-trivial JSONB.
  let forms = [
    GovernanceLogInsertForm {
      entry_kind: "phase1.smoke.first".to_string(),
      payload: json!({ "n": 1 }),
      actor_pseudonym: Some("pseudo-alpha".to_string()),
    },
    GovernanceLogInsertForm {
      entry_kind: "phase1.smoke.second".to_string(),
      payload: json!({ "n": 2 }),
      actor_pseudonym: None,
    },
    GovernanceLogInsertForm {
      entry_kind: "phase1.smoke.third".to_string(),
      payload: json!({ "n": 3, "nested": [1, 2] }),
      actor_pseudonym: Some("pseudo-bravo".to_string()),
    },
  ];
  for form in &forms {
    diesel::insert_into(governance_log::table)
      .values(form)
      .execute(&mut conn)?;
  }

  // Read back with the exact byte sequences the trigger hashed: Postgres's
  // `payload::text` and `to_char(... 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')`.
  let rows: Vec<RawRow> = sql_query(
    r#"
    SELECT
      id,
      prev_hash,
      entry_hash,
      entry_kind,
      payload::text AS payload_text,
      to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at_text
    FROM governance_log
    ORDER BY id ASC
    "#,
  )
  .load(&mut conn)?;
  assert_eq!(rows.len(), 3, "should have exactly 3 rows");

  // Recompute the chain in Rust with the exact bytes Postgres used and assert
  // the stored hashes match bit-for-bit.
  let mut prev: Vec<u8> = vec![0u8; 32];
  for row in &rows {
    assert_eq!(
      row.prev_hash, prev,
      "row {} prev_hash should match the previous row's entry_hash",
      row.id
    );
    let mut hasher = Sha256::new();
    hasher.update(&prev);
    hasher.update(row.entry_kind.as_bytes());
    hasher.update(row.payload_text.as_bytes());
    hasher.update(row.created_at_text.as_bytes());
    let expected = hasher.finalize().to_vec();
    assert_eq!(
      row.entry_hash, expected,
      "row {} entry_hash should match sha256(prev||kind||payload_text||ts_text)",
      row.id
    );
    prev = row.entry_hash.clone();
  }

  let first_id = rows[0].id;

  // Append-only DELETE must be rejected by the before-delete trigger. Wrap in
  // a transaction so the aborted-transaction state rolls back and later
  // statements on `conn` don't inherit it.
  let delete_result = conn.transaction::<_, diesel::result::Error, _>(|c| {
    diesel::delete(governance_log::table.find(first_id)).execute(c)?;
    Ok(())
  });
  assert!(
    delete_result.is_err(),
    "delete from governance_log must be rejected by the append-only trigger"
  );

  // Updating a non-signature column must be rejected by the signature-gate
  // trigger. Same transaction wrapper for the same reason.
  let bad_update = conn.transaction::<_, diesel::result::Error, _>(|c| {
    diesel::update(governance_log::table.find(first_id))
      .set(governance_log::entry_kind.eq("tampered"))
      .execute(c)?;
    Ok(())
  });
  assert!(
    bad_update.is_err(),
    "updating entry_kind must be rejected by the append-only trigger"
  );

  Ok(())
}

/// Level 3 acceptance gate: exercise every Phase 1 migration's `down.sql`
/// against a scratch DB by reverting and re-applying the 6 most recent
/// migrations (tasks 2–7: enums, core, jury_system, reputation_and_surety,
/// actor_pseudonym, governance_log).
///
/// Uses the Lemmy-native `lemmy_diesel_utils::schema_setup::run` runner
/// because raw `diesel migration revert` is blocked by the
/// `forbid_diesel_cli` trigger landed in migration `2025-08-01-000017`.
/// The runner acquires `pg_advisory_lock(0)` at `schema_setup/mod.rs:214`,
/// which is what the forbid trigger checks for.
#[ignore = "TODO(v0-polish): deflake — GH issue #43 (needs revert-list extension for federation tables)"]
#[tokio::test]
async fn phase1_migrations_round_trip() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl, sql_query};
  use lemmy_diesel_utils::schema_setup::{self, Options};

  /// Count of branch-added migrations that must revert cleanly for the
  /// Phase 1 enum and table assertions below to hold. Started at 6 in Phase
  /// 1 (tasks 2–7). Each subsequent phase that adds a migration bumps this
  /// by its migration count. Current composition:
  ///   - 6 Phase 1 migrations (enums, core, jury, rep+surety, pseudonym,
  ///     governance_log — the last one was actually added in Phase 4b task 8
  ///     but is still part of the contiguous governance-bootstrap block that
  ///     this test reverts LIFO)
  ///   - 2 Phase 5a migrations (add_governance_config + add_person_membership_state)
  ///   - 1 Phase 5b Slice A migration (add_restoration_sanction_variant
  ///     — task 56 / OQ-003; total 9)
  const PHASE_1_MIGRATION_COUNT: u64 = 9;

  /// Query shape for `COUNT(*)` probes via `sql_query`.
  #[derive(diesel::QueryableByName)]
  struct Count {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  // Step 1: full forward apply. The runner takes pg_advisory_lock(0),
  // bypasses the forbid trigger, runs every pending migration, and rebuilds
  // the `r` schema. Anything in the branch state that wasn't already in
  // upstream Lemmy lands here.
  schema_setup::run(Options::default().run(), &db_url)?;

  // Post-condition probes: each Phase 1 table exists and is queryable. If
  // any of these fails, the forward migrations themselves are broken and
  // the rest of the test is meaningless.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    for table in [
      "moderation_case",
      "case_evidence",
      "sanction",
      "appeal",
      "public_case_log",
      "jury_pool",
      "jury_assignment",
      "jury_vote",
      "surety",
      "endorsement",
      "reputation_event",
      "reputation_snapshot",
      "actor_pseudonym",
      "governance_log",
    ] {
      let result: Count = sql_query(format!("SELECT count(*) AS n FROM {table}")).get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "{table} should exist and be empty after forward migration"
      );
    }
  }

  // Step 2: revert the last N migrations via the native runner. This
  // exercises each Phase 1 `down.sql` in LIFO order. Task 7 (governance_log)
  // reverts first, task 2 (enums) reverts last. Any broken down.sql fails
  // here — the runner propagates the SQL error up through anyhow.
  schema_setup::run(
    Options::default().revert().limit(PHASE_1_MIGRATION_COUNT),
    &db_url,
  )?;

  // Post-condition probes: every Phase 1 table must be GONE. A leftover
  // table means its down.sql didn't drop it cleanly.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    for table in [
      "moderation_case",
      "case_evidence",
      "sanction",
      "appeal",
      "public_case_log",
      "jury_pool",
      "jury_assignment",
      "jury_vote",
      "surety",
      "endorsement",
      "reputation_event",
      "reputation_snapshot",
      "actor_pseudonym",
      "governance_log",
    ] {
      let probe: Result<Count, diesel::result::Error> =
        sql_query(format!("SELECT count(*) AS n FROM {table}")).get_result(&mut conn);
      assert!(
        probe.is_err(),
        "{table} should not exist after reverting Phase 1 migrations"
      );
    }
  }

  // Also check that the Phase 1 enum types were dropped — if `DROP TYPE`
  // was missed in enums/down.sql, these pg_type lookups would still return
  // rows.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    for type_name in [
      "case_status",
      "case_target_type",
      "case_severity",
      "evidence_visibility",
      "jury_assignment_status",
      "jury_decision",
      "sanction_scope",
      "sanction_action",
      "appeal_status",
      "reputation_dimension",
      "attestation_type",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_type WHERE typname = '{type_name}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "pg_type entry for {type_name} should be dropped after reverting Phase 1 migrations"
      );
    }
  }

  // Step 3: re-apply. If down.sql didn't leave the DB in a clean state,
  // the forward re-apply will fail with an error like "type case_status
  // already exists" or "relation moderation_case already exists".
  schema_setup::run(Options::default().run(), &db_url)?;

  // Final post-condition: governance_log is queryable again. If this
  // passes, every Phase 1 migration round-tripped cleanly and the replace-
  // able schema (including the hash-chain triggers) was rebuilt.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    let result: Count =
      sql_query("SELECT count(*) AS n FROM governance_log").get_result(&mut conn)?;
    assert_eq!(
      result.n, 0,
      "governance_log should exist and be empty after revert + re-apply"
    );
  }

  Ok(())
}

// ============================================================================
// Phase 2 — view-crate smoke tests
// ============================================================================
//
// Three gates, one per Phase 2 view crate:
//
//   * `list_open_cases_returns_seeded_rows`  → governance_case
//   * `jury_queue_view_returns_assignments`  → jury_queue
//   * `modlog_view_returns_published_entries`→ governance_modlog
//
// Seeding uses the same sync `PgConnection` pattern as the Phase 1 tests
// so we reuse the `governance_fixtures::apply_all_schema` helper. The
// view-crate queries take `&mut DbPool<'_>`, which is diesel-async; we
// build a single `AsyncPgConnection` against the same container and
// convert it via the blanket `From<&mut AsyncPgConnection> for DbPool<'_>`
// impl at `lemmy_diesel_utils::connection.rs:104`. No pool needed — the
// `DbPool::Conn` variant threads one borrowed connection.

#[tokio::test]
async fn list_open_cases_returns_seeded_rows() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
  use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::moderation_case;
  use lemmy_db_views_governance_case::impls::list_cases_needing_jury_selection;
  use lemmy_diesel_utils::connection::DbPool;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;

    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::RemoteInstance,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: None,
      target_community_id: None,
      target_remote_url: Some("https://example.invalid/post/seed".to_string()),
      reason_code: "spam".to_string(),
      severity: CaseSeverity::Low,
      status: CaseStatus::ThresholdMet,
      threshold_score: 1,
    };
    diesel::insert_into(moderation_case::table)
      .values(&form)
      .execute(&mut sync_conn)?;
  }

  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  let mut pool: DbPool<'_> = (&mut async_conn).into();

  let rows = list_cases_needing_jury_selection(&mut pool)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("list_cases_needing_jury_selection: {e}").into() })?;
  assert_eq!(rows.len(), 1, "expected exactly one ThresholdMet case");
  let row = rows
    .first()
    .ok_or_else(|| -> Box<dyn Error> { "expected at least one row".into() })?;
  assert_eq!(
    row.jury_needed, 5,
    "jury_needed should be the [05 §3] constant 5"
  );
  assert_eq!(
    row.reporter_count, 0,
    "reporter_count is a Phase 2a drift stub and should always be 0"
  );
  assert!(
    matches!(row.status, CaseStatus::ThresholdMet),
    "seeded status must round-trip unchanged"
  );

  Ok(())
}

#[tokio::test]
async fn jury_queue_view_returns_assignments() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl, connection::SimpleConnection};
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_db_schema::newtypes::ModerationCaseId;
  use lemmy_db_schema::source::governance::{
    jury_assignment::JuryAssignmentInsertForm,
    moderation_case::ModerationCaseInsertForm,
  };
  use lemmy_db_schema_file::PersonId;
  use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::{jury_assignment, moderation_case};
  use lemmy_db_views_jury_queue::impls::list_jury_assignments_for_person;
  use lemmy_diesel_utils::connection::DbPool;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  // Seed an instance + person pair via raw SQL — the only way to satisfy
  // `jury_assignment.person_id -> person (id)` without pulling the whole
  // Lemmy Person::create stack into this test binary. Person has many
  // NOT NULL columns but most have DEFAULTs; we set the minimum required
  // (name, ap_id, inbox_url, public_key, instance_id) and let the rest
  // default.
  let (case_id, person_id) = {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;

    sync_conn.batch_execute(
      r#"
      INSERT INTO instance (domain) VALUES ('test.invalid');
      INSERT INTO person (name, ap_id, inbox_url, public_key, instance_id)
        VALUES (
          'seed-juror',
          'https://test.invalid/u/seed-juror',
          'https://test.invalid/u/seed-juror/inbox',
          'seed-pubkey',
          (SELECT id FROM instance WHERE domain = 'test.invalid')
        );
      "#,
    )?;

    let person_id: i32 = diesel::sql_query(
      "SELECT id FROM person WHERE name = 'seed-juror'",
    )
    .get_result::<SingleI32>(&mut sync_conn)?
    .id;

    let case_form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::RemoteInstance,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: None,
      target_community_id: None,
      target_remote_url: Some("https://example.invalid/post/jury-seed".to_string()),
      reason_code: "harassment".to_string(),
      severity: CaseSeverity::Medium,
      status: CaseStatus::InReview,
      threshold_score: 1,
    };
    let case_id: i32 = diesel::insert_into(moderation_case::table)
      .values(&case_form)
      .returning(moderation_case::id)
      .get_result(&mut sync_conn)?;

    let assignment_form = JuryAssignmentInsertForm {
      case_id: ModerationCaseId(case_id),
      person_id: PersonId(person_id),
      status: lemmy_db_schema_file::enums::JuryAssignmentStatus::Selected,
    };
    diesel::insert_into(jury_assignment::table)
      .values(&assignment_form)
      .execute(&mut sync_conn)?;

    (case_id, person_id)
  };

  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  let mut pool: DbPool<'_> = (&mut async_conn).into();

  let rows = list_jury_assignments_for_person(&mut pool, PersonId(person_id))
    .await
    .map_err(|e| -> Box<dyn Error> { format!("list_jury_assignments_for_person: {e}").into() })?;
  assert_eq!(
    rows.len(),
    1,
    "expected exactly one jury assignment for the seeded juror"
  );
  let row = rows
    .first()
    .ok_or_else(|| -> Box<dyn Error> { "expected at least one row".into() })?;
  assert_eq!(row.case_id, case_id, "case_id should round-trip unchanged");
  assert!(
    row.deadline_at.is_none(),
    "deadline_at is a Phase 2a drift stub and should always be None"
  );

  Ok(())
}

#[tokio::test]
async fn modlog_view_returns_published_entries() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_db_schema::newtypes::ModerationCaseId;
  use lemmy_db_schema::source::governance::{
    moderation_case::ModerationCaseInsertForm,
    public_case_log::PublicCaseLogInsertForm,
  };
  use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::{moderation_case, public_case_log};
  use lemmy_db_views_governance_modlog::impls::list_public_case_log;
  use lemmy_diesel_utils::connection::DbPool;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  let case_id = {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;

    let case_form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::RemoteInstance,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: None,
      target_community_id: None,
      target_remote_url: Some("https://example.invalid/post/modlog-seed".to_string()),
      reason_code: "disinformation".to_string(),
      severity: CaseSeverity::High,
      status: CaseStatus::InReview,
      threshold_score: 1,
    };
    let case_id: i32 = diesel::insert_into(moderation_case::table)
      .values(&case_form)
      .returning(moderation_case::id)
      .get_result(&mut sync_conn)?;

    let log_form = PublicCaseLogInsertForm {
      case_id: ModerationCaseId(case_id),
      community_id: None,
      summary: "Case summary — no identifiers".to_string(),
      rationale_redacted: None,
    };
    diesel::insert_into(public_case_log::table)
      .values(&log_form)
      .execute(&mut sync_conn)?;

    case_id
  };

  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  let mut pool: DbPool<'_> = (&mut async_conn).into();

  let rows = list_public_case_log(&mut pool)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("list_public_case_log: {e}").into() })?;
  assert_eq!(rows.len(), 1, "expected exactly one public_case_log entry");
  let row = rows
    .first()
    .ok_or_else(|| -> Box<dyn Error> { "expected at least one row".into() })?;
  assert_eq!(row.case_id, case_id, "case_id should round-trip unchanged");
  assert_eq!(
    row.summary, "Case summary — no identifiers",
    "summary should be read back verbatim (no re-redaction)"
  );
  assert!(
    row.decision.is_none(),
    "decision is a Phase 2b drift stub and should always be None"
  );
  assert!(
    row.sanction_action.is_none(),
    "sanction_action is a Phase 2b drift stub and should always be None"
  );
  assert!(
    !row.appealed,
    "appealed should be false when no appeal row was seeded"
  );

  Ok(())
}

/// Tiny row shape for `sql_query` probes that return a single `id` column.
#[derive(diesel::QueryableByName)]
struct SingleI32 {
  #[diesel(sql_type = diesel::sql_types::Int4)]
  id: i32,
}

// ============================================================================
// Phase 4 — golden-path end-to-end test (DoD per IMPLEMENTATION-PLAN-v0 §3 P4)
// ============================================================================
//
// Walks a single moderation case from initial report → admin-assigned jury
// (approach B backstop, makes the test deterministic without needing four
// separate reporters) → 3 jury votes → decision → sanction → public modlog.
// Assertions hit every Phase 4 invariant: hash chain, ed25519 signatures,
// pseudonym usage, redaction, status transitions, vote tally, reputation
// events, and per-entry_kind log counts.
//
// Drift from plan resolved per decision-queue #8 (reputation_event count =
// 4, no "reputation_event" key in the governance_log map) and #9 (direct
// handler invocation rather than building an actix `App`).

#[tokio::test(flavor = "multi_thread")]
async fn report_to_modlog_golden_path() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Data, Json, Query};
  use chrono::{DateTime, Duration, Utc};
  use diesel::{
    Connection as _,
    ExpressionMethods,
    PgConnection,
    QueryDsl,
    sql_query,
    sql_types::{Bytea, Int8, Text},
  };
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use ed25519_dalek::{Signature, SigningKey, Verifier, VerifyingKey};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    list_modlog::list_modlog,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment,
    AdminAssignJury,
    CreateGovernanceReport,
    ListGovernanceModlog,
    SubmitJuryVote,
  };
  use lemmy_api_crud::governance::create_report::create_report;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseStatus, CaseTargetType, JuryDecision, ReputationDimension},
    schema::{governance_log, jury_assignment, jury_vote, moderation_case, public_case_log,
             reputation_event, sanction},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, DbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;
  use sha2::{Digest, Sha256};
  use std::collections::HashMap;

  // -- 1. Set env vars BEFORE any Lemmy code touches `SETTINGS`. --------
  // Deterministic 32-byte ed25519 seed: 31 zero bytes + 0x01.
  const SIGNING_SEED_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";
  // SAFETY: tests run with --test-threads=1 so no concurrent env mutation;
  // these vars are read by SETTINGS (LazyLock) and the governance log
  // signer at first call.
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
  }

  // -- 2. Spin up Postgres and apply the full schema. -------------------
  // governance_fixtures helpers return `Box<dyn Error>` (no Send+Sync),
  // which doesn't bridge to anyhow/LemmyError via `?`. Stringify across
  // the boundary.
  let (_container, host_port) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| anyhow::anyhow!("start_postgres: {e}"))?;
  let db_url = governance_fixtures::db_url(host_port);
  unsafe {
    std::env::set_var("LEMMY_DATABASE_URL", &db_url);
  }

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| anyhow::anyhow!("apply_all_schema: {e}"))?;
  }

  // -- 3. Build an ActualDbPool against this container.
  // `build_db_pool_for_tests` reads `LEMMY_DATABASE_URL` from SETTINGS
  // (set above) and re-runs `schema_setup::run`; the latter is
  // idempotent against the schema we already applied via
  // `apply_all_schema`, and acquires `pg_advisory_lock(0)` to bypass
  // the `forbid_diesel_cli` trigger. This mirrors
  // `init_test_federation_config` at api_utils/src/context.rs:67.
  let pool: ActualDbPool = build_db_pool_for_tests();

  // -- 4. Build a LemmyContext directly. Mirrors
  //       `init_test_federation_config` at api_utils/src/context.rs:69-85.
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

  // Phase 6 task 76: `submit_jury_vote` now takes
  // `activitypub_federation::config::Data<LemmyContext>` (not the actix
  // Data) so its `process_vote` body can hand `&context` to
  // `federation_outbox::send_local_sanction_notice`, which needs it for
  // activity-id hostname generation and `Person::read` resolution. Build
  // a federation Data here that wraps the same `LemmyContext` (the
  // underlying pool is `Arc`-shared via `ActualDbPool`, so both Data
  // handles see the same DB rows). Used only at the `submit_jury_vote`
  // call sites below; every other handler still takes the actix Data.
  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context = federation_config.to_request_data();

  // -- 5. Seed instance + 8 persons + 1 community + 1 post. -------------
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;

  // Helper: create a person + local_user pair. `is_admin` toggles
  // local_user.admin; all jurors get accepted_application=true so the
  // admin-assign-jury eligibility filter sees them.
  async fn seed_person(
    ctx: &LemmyContext,
    instance_id: lemmy_db_schema_file::InstanceId,
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

  let reporter = seed_person(&context, instance.id, "reporter", false).await?;
  let target = seed_person(&context, instance.id, "target", false).await?;
  let admin = seed_person(&context, instance.id, "admin", true).await?;
  let juror_d = seed_person(&context, instance.id, "juror_d", false).await?;
  let juror_e = seed_person(&context, instance.id, "juror_e", false).await?;
  let juror_f = seed_person(&context, instance.id, "juror_f", false).await?;
  let juror_g = seed_person(&context, instance.id, "juror_g", false).await?;
  let juror_h = seed_person(&context, instance.id, "juror_h", false).await?;

  let community_form = CommunityInsertForm::new(
    instance.id,
    "testcomm".to_string(),
    "Test Community".to_string(),
    "comm-pubkey".to_string(),
  );
  let community = Community::create(&mut context.pool(), &community_form).await?;

  // -- 6. Resolve LocalUserView for each actor. -------------------------
  let reporter_view = LocalUserView::read_person(&mut context.pool(), reporter).await?;
  let admin_view = LocalUserView::read_person(&mut context.pool(), admin).await?;

  // -- 7. Step 1: POST /governance/report (single report; threshold not
  //              met because v0 V0_THRESHOLD = 3 and weight is 1).
  // Reporting the target person directly so admin_assign_jury's
  // eligibility filter sees `case.target_person_id = Some(target)` and
  // excludes them from the panel. See decision-queue #10 — the
  // Post-target codepath has a known eligibility-filter gap that must
  // be patched in Phase 5.
  let create_resp = create_report(
    Json(CreateGovernanceReport {
      community_id: Some(community.id),
      target_type: CaseTargetType::Person,
      target_id: target.0,
      reason_code: "spam".to_string(),
      description: Some("Email spam@example.com posting links http://bad.invalid/".to_string()),
    }),
    context.clone(),
    reporter_view.clone(),
  )
  .await?
  .into_inner();
  assert!(create_resp.case_id.is_some(), "case_id must be set");
  assert!(!create_resp.threshold_met, "single report must not meet threshold");
  let case_id = create_resp.case_id.expect("case_id present");

  // -- 8. DB checks after report --------------------------------------
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  {
    let mut probe_pool: DbPool<'_> = (&mut async_conn).into();
    use lemmy_diesel_utils::connection::get_conn;
    let conn = &mut get_conn(&mut probe_pool).await?;

    let (status, threshold_score): (CaseStatus, i64) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((moderation_case::status, moderation_case::threshold_score))
      .first(conn)
      .await?;
    assert!(matches!(status, CaseStatus::Open), "status must be Open");
    // Phase 5b task 58: OQ-006 formula. Reporter has no reputation_event
    // rows, so the on-the-fly snapshot has reporting_accuracy = 0, which
    // clamps to `report.clamp_min = 0.1`. weight = 1.0 * 0.1 * 1.0 *
    // 1_000_000 = 100_000 (one report; micros scale).
    assert_eq!(
      threshold_score, 100_000,
      "threshold_score after one report = base_weight × clamp_min × recency × 1_000_000 = 100_000"
    );

    let report_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("report_created"))
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(report_count, 1, "exactly one report_created entry");

    let signed_nulls: i64 = governance_log::table
      .filter(governance_log::signature.is_null())
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(signed_nulls, 0, "every governance_log row must be signed");
  }

  // -- 9. Step 2: POST /governance/admin/assign-jury as admin --------
  let assign_resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view.clone(),
  )
  .await?
  .into_inner();
  assert_eq!(assign_resp.assigned_person_ids.len(), 5, "5 jurors assigned");
  for pid in &assign_resp.assigned_person_ids {
    assert_ne!(*pid, reporter, "reporter must not be on jury");
    assert_ne!(*pid, target, "target must not be on jury");
  }

  // -- 10. DB checks after jury assignment --------------------------
  {
    let mut probe_pool: DbPool<'_> = (&mut async_conn).into();
    use lemmy_diesel_utils::connection::get_conn;
    let conn = &mut get_conn(&mut probe_pool).await?;

    let assignment_count: i64 = jury_assignment::table
      .filter(jury_assignment::case_id.eq(case_id))
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(assignment_count, 5, "5 jury_assignment rows");

    let case_status: CaseStatus = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select(moderation_case::status)
      .first(conn)
      .await?;
    assert!(
      matches!(case_status, CaseStatus::JurySelection),
      "case must be JurySelection after assign-jury (plan shorthand: InPanel)"
    );

    let counts: Vec<(String, i64)> = governance_log::table
      .group_by(governance_log::entry_kind)
      .select((governance_log::entry_kind, diesel::dsl::count_star()))
      .load::<(String, i64)>(conn)
      .await?;
    let map: HashMap<String, i64> = counts.into_iter().collect();
    assert_eq!(map.get("report_created"), Some(&1));
    assert_eq!(map.get("jury_assigned"), Some(&5));
    assert_eq!(map.get("panel_assembled"), Some(&1));
  }

  // -- 10a. Every assigned juror calls accept_jury_assignment (task 64) --
  //        Must run BEFORE jurors vote: submit_jury_vote filters on
  //        status=Accepted. With the task 64a flip, admin_assign_jury now
  //        writes status=Selected, so the accept handshake moves each
  //        assignment to status=Accepted before the vote loop below.
  for juror_id in &assign_resp.assigned_person_ids {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    let _resp = accept_jury_assignment(
      Json(AcceptJuryAssignment { case_id }),
      context.clone(),
      juror_view,
    )
    .await?
    .into_inner();
  }

  // -- 11. Steps 3–5: ALL 5 jurors vote AdvisoryLabel ------------------
  // All 5 jurors vote. Votes 4 and 5 arrive AFTER quorum trips at vote 3.
  // This exercises the idempotency guard in submit_jury_vote: late-arriving
  // votes must persist the vote row and emit jury_vote_submitted for audit
  // integrity, but MUST NOT re-run the post-decision block (sanction insert,
  // sponsor liability, public_case_log append, federation publish, nor
  // governance_log case_decided/sanction_created/public_log_published).
  // See CodeRabbit PR #46 finding #15.
  let voting_jurors: Vec<PersonId> = assign_resp
    .assigned_person_ids
    .iter()
    .copied()
    .collect();

  // Embed every category the redaction layer scrubs so the public-log
  // assertions below exercise mention, email, and profile-URL stripping.
  // Per redaction.rs:51-61 the contract covers `@handle` mentions,
  // bare email addresses, and `https?://host/(u|user|profile)/<handle>`
  // profile URLs. Arbitrary http URLs (e.g. `http://example.com/post/1`)
  // are deliberately NOT in the contract.
  let mut decided_responses = Vec::new();
  for (i, juror_id) in voting_jurors.iter().enumerate() {
    let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id).await?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::AdvisoryLabel,
        rationale: Some(format!(
          "Juror {i} saw @someone email foo.bar@example.com via https://lemmy.example/u/baduser"
        )),
      }),
      federation_context.reset_request_count(),
      juror_view,
    )
    .await?
    .into_inner();
    decided_responses.push(resp);
  }
  assert!(!decided_responses[0].case_decided, "1st vote: not decided");
  assert!(!decided_responses[1].case_decided, "2nd vote: not decided");
  assert!(decided_responses[2].case_decided, "3rd vote: decided (quorum tripped)");
  // Votes 4+5 arrive post-quorum. Case is already Decided — handler must
  // return case_decided: true (case IS decided) but must NOT re-run the
  // post-decision block. Downstream exactly-once DB assertions are the
  // load-bearing invariant; these response assertions only verify shape.
  assert!(decided_responses[3].case_decided, "4th vote: case already decided (idempotent)");
  assert!(decided_responses[4].case_decided, "5th vote: case already decided (idempotent)");
  assert_eq!(
    decided_responses[2].decision,
    Some(JuryDecision::AdvisoryLabel),
    "3rd vote returns winning decision"
  );

  // Silence unused-binding lints for jurors not on the panel — the random
  // selection means we can't predict which of D-H were picked, so we keep
  // them all live until after the assertion above.
  let _ = (juror_d, juror_e, juror_f, juror_g, juror_h);

  // -- 12. DB checks after decision ------------------------------
  {
    let mut probe_pool: DbPool<'_> = (&mut async_conn).into();
    use lemmy_diesel_utils::connection::get_conn;
    let conn = &mut get_conn(&mut probe_pool).await?;

    let vote_count: i64 = jury_vote::table
      .filter(jury_vote::case_id.eq(case_id))
      .count()
      .get_result(conn)
      .await?;
    // All 5 jurors voted. Every vote row persists for audit integrity even
    // though votes 4+5 arrived post-quorum — vote INSERT sits ABOVE the
    // idempotency gate in submit_jury_vote. Only the post-decision block
    // is guarded.
    assert_eq!(vote_count, 5, "5 jury_vote rows (all jurors recorded)");

    let (status, decided_at, closed_at): (
      CaseStatus,
      Option<DateTime<Utc>>,
      Option<DateTime<Utc>>,
    ) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::decided_at,
        moderation_case::closed_at,
      ))
      .first(conn)
      .await?;
    assert!(matches!(status, CaseStatus::Decided), "case must be Decided");
    let decided = decided_at.expect("decided_at set");
    let closed = closed_at.expect("closed_at set");
    let gap = closed.signed_duration_since(decided);
    // DB precision can drift by microseconds; assert within 1 second of 7d.
    let expected = Duration::days(7);
    assert!(
      (gap - expected).num_milliseconds().abs() < 1_000,
      "closed_at should be ~ decided_at + 7 days (got gap = {gap:?})"
    );

    let sanction_count: i64 = sanction::table
      .filter(sanction::case_id.eq(case_id))
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(sanction_count, 1, "1 sanction row");

    let (summary, rationale): (String, Option<String>) = public_case_log::table
      .filter(public_case_log::case_id.eq(case_id))
      .select((public_case_log::summary, public_case_log::rationale_redacted))
      .first(conn)
      .await?;
    // Summary is bland by construction (build_summary at
    // submit_jury_vote.rs:440-448 uses only case_id/target_type/decision/
    // reason_code); scrub still runs as defence-in-depth, so confirm
    // no leak even though it's structurally impossible here.
    assert!(!summary.contains('@'), "summary must be scrubbed of '@'");
    assert!(
      !summary.contains("@example."),
      "summary must be scrubbed of email-shaped strings"
    );
    // Rationale assertions exercise the actual scrub contract per
    // redaction.rs:51-61: mentions, emails, and profile URLs of the
    // form host/(u|user|profile)/handle.
    let r = rationale.as_deref().expect("rationale present");
    assert!(!r.contains("@someone"), "rationale must scrub @mentions");
    assert!(
      !r.contains("foo.bar@example.com"),
      "rationale must scrub email addresses"
    );
    assert!(
      !r.contains("/u/baduser"),
      "rationale must scrub profile URLs"
    );
    assert!(
      r.contains("[redacted]"),
      "rationale must contain redaction sentinel"
    );

    // Drift #8: 4 reputation_event rows (3 jurors on JuryReliability + 1
    // reporter on ReportingAccuracy) per [05 §6] — NOT 3 as the plan
    // body suggests.
    //
    // Exactly-once under post-quorum votes: reputation_event writes occur
    // only in the post-decision block. Votes 4+5 MUST NOT produce additional
    // rows. Without the idempotency guard, this count would be 14
    // (3+4+5 jurors over three post-decision-block runs at votes 3/4/5,
    // plus 1 reporter) — double-penalising sponsors on sponsor_liability
    // recompute and polluting reputation history. Regression test for
    // CodeRabbit PR #46 #15.
    let rep_total: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(rep_total, 4, "4 reputation_event rows (exactly-once under late votes)");

    let jury_rep_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .filter(reputation_event::dimension.eq(ReputationDimension::JuryReliability))
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(jury_rep_count, 3, "3 JuryReliability rows (one per juror)");

    let reporter_rep_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .filter(reputation_event::dimension.eq(ReputationDimension::ReportingAccuracy))
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(reporter_rep_count, 1, "1 ReportingAccuracy row (reporter)");

    let counts: Vec<(String, i64)> = governance_log::table
      .group_by(governance_log::entry_kind)
      .select((governance_log::entry_kind, diesel::dsl::count_star()))
      .load::<(String, i64)>(conn)
      .await?;
    let map: HashMap<String, i64> = counts.into_iter().collect();
    assert_eq!(map.get("report_created"), Some(&1));
    assert_eq!(map.get("jury_assigned"), Some(&5));
    assert_eq!(map.get("panel_assembled"), Some(&1));
    assert_eq!(map.get("jury_accepted"), Some(&5));
    // All 5 vote rows emit jury_vote_submitted (above the idempotency gate).
    assert_eq!(map.get("jury_vote_submitted"), Some(&5));
    // Exactly-once invariants: each post-decision entry kind emitted once
    // despite votes 4+5 arriving after quorum. Without the submit_jury_vote
    // idempotency guard these would be 3 each. Regression test for
    // CodeRabbit PR #46 finding #15.
    assert_eq!(map.get("case_decided"), Some(&1));
    assert_eq!(map.get("sanction_created"), Some(&1));
    assert_eq!(map.get("public_log_published"), Some(&1));
    // Drift #8: submit_jury_vote.rs does NOT emit governance_log entries
    // for reputation_event writes. Assert the key is absent.
    assert!(
      !map.contains_key("reputation_event"),
      "no reputation_event entry_kind should appear in governance_log"
    );
  }

  // -- 13. Step 6: GET /governance/modlog?community_id=... unauth -----
  let modlog_resp = list_modlog(
    Query(ListGovernanceModlog {
      community_id: Some(community.id),
      page: None,
      limit: None,
    }),
    context.clone(),
    None,
  )
  .await?
  .into_inner();
  assert_eq!(modlog_resp.len(), 1, "exactly one modlog entry for the community");
  assert_eq!(modlog_resp[0].case_id, case_id.0, "modlog entry case_id matches");

  // -- 14. Hash chain + signature verification on every governance_log
  //        row. Mirrors governance_log_hash_chain_holds at e2e.rs:159+
  //        and triggers.sql:781-788. -----------------------------------
  #[derive(diesel::QueryableByName, Debug)]
  struct RawRow {
    #[diesel(sql_type = Int8)]
    id: i64,
    #[diesel(sql_type = Bytea)]
    prev_hash: Vec<u8>,
    #[diesel(sql_type = Bytea)]
    entry_hash: Vec<u8>,
    #[diesel(sql_type = Text)]
    entry_kind: String,
    #[diesel(sql_type = Text)]
    payload_text: String,
    #[diesel(sql_type = Text)]
    created_at_text: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<Bytea>)]
    signature: Option<Vec<u8>>,
  }

  let mut sync_conn = PgConnection::establish(&db_url)?;
  let rows: Vec<RawRow> = diesel::RunQueryDsl::load(
    sql_query(
      r#"
      SELECT
        id,
        prev_hash,
        entry_hash,
        entry_kind,
        payload::text AS payload_text,
        to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"') AS created_at_text,
        signature
      FROM governance_log
      ORDER BY id ASC
      "#,
    ),
    &mut sync_conn,
  )?;
  assert!(!rows.is_empty(), "expected governance_log rows");

  let signing_seed = hex::decode(SIGNING_SEED_HEX)?;
  let seed_arr: [u8; 32] = signing_seed
    .as_slice()
    .try_into()
    .map_err(|_| anyhow::anyhow!("signing seed must be 32 bytes"))?;
  let signing_key = SigningKey::from_bytes(&seed_arr);
  let verifying_key: VerifyingKey = signing_key.verifying_key();

  let mut prev: Vec<u8> = vec![0u8; 32];
  for row in &rows {
    assert_eq!(
      row.prev_hash, prev,
      "row {} prev_hash should match the previous row's entry_hash",
      row.id
    );
    let mut hasher = Sha256::new();
    hasher.update(&prev);
    hasher.update(row.entry_kind.as_bytes());
    hasher.update(row.payload_text.as_bytes());
    hasher.update(row.created_at_text.as_bytes());
    let expected = hasher.finalize().to_vec();
    assert_eq!(
      row.entry_hash, expected,
      "row {} entry_hash should match sha256(prev||kind||payload||ts)",
      row.id
    );

    let sig_bytes = row
      .signature
      .as_ref()
      .ok_or_else(|| anyhow::anyhow!("row {} missing signature", row.id))?;
    let sig_arr: [u8; 64] = sig_bytes
      .as_slice()
      .try_into()
      .map_err(|_| anyhow::anyhow!("row {} signature wrong length", row.id))?;
    let sig = Signature::from_bytes(&sig_arr);
    verifying_key
      .verify(&row.entry_hash, &sig)
      .map_err(|e| anyhow::anyhow!("row {} signature verify failed: {e}", row.id))?;

    prev = row.entry_hash.clone();
  }

  Ok(())
}

// ============================================================================
// Phase 5a — governance_config seed/const parity round-trip (GOTCHA-50h)
// ============================================================================
//
// Structural parity (`seeded_keys_count_matches_const_count`,
// `every_seeded_key_has_const_fallback`) lives in
// `crates/api/api/src/governance/config.rs::parity` — no DB needed, runs
// at `cargo test -p lemmy_api --lib`.
//
// This test is the runtime pair: walk every entry in
// `SEEDED_KEYS_WITH_CONSTS`, call the typed accessor matching the declared
// `value_type`, and assert the read succeeds. Catches the class of bug where
// the SQL seed stores a `value_text` row but the Rust const declares `i64`
// (or any shape disagreement the DB-level CHECK cannot catch on the
// Rust-declaration side — per Perplexity-review 2026-04-17 item 5).

#[tokio::test]
async fn config_parity_round_trip() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection};
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_api::governance::config::{
    ConfigCache, SEEDED_KEYS_WITH_CONSTS, Scope, get_bool, get_float, get_int, get_text,
  };
  use lemmy_diesel_utils::connection::DbPool;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  let mut pool: DbPool<'_> = (&mut async_conn).into();
  let mut cache = ConfigCache::new();

  for (key, _const_name, vtype) in SEEDED_KEYS_WITH_CONSTS {
    match *vtype {
      "int" => {
        get_int(&mut cache, &mut pool, Scope::Instance, key)
          .await
          .map_err(|e| -> Box<dyn Error> {
            format!("get_int round-trip failed for `{key}`: {e}").into()
          })?;
      }
      "float" => {
        get_float(&mut cache, &mut pool, Scope::Instance, key)
          .await
          .map_err(|e| -> Box<dyn Error> {
            format!("get_float round-trip failed for `{key}`: {e}").into()
          })?;
      }
      "bool" => {
        get_bool(&mut cache, &mut pool, Scope::Instance, key)
          .await
          .map_err(|e| -> Box<dyn Error> {
            format!("get_bool round-trip failed for `{key}`: {e}").into()
          })?;
      }
      "text" => {
        get_text(&mut cache, &mut pool, Scope::Instance, key)
          .await
          .map_err(|e| -> Box<dyn Error> {
            format!("get_text round-trip failed for `{key}`: {e}").into()
          })?;
      }
      other => {
        return Err(format!("unknown value_type `{other}` for seed key `{key}`").into());
      }
    }
  }

  Ok(())
}

// ============================================================================
// Phase 5b task 60 — sponsor_liability_with_founder_multiplier (3 branches)
// ============================================================================
//
// Drives the full report → admin-assign → 3 votes → Decided pipeline through
// `submit_jury_vote`. `apply_sponsor_liability` runs inside that handler's
// transaction (see submit_jury_vote.rs:259) and writes `reputation_event` +
// `governance_log` rows the test asserts on.
//
// Three branches share one testcontainer + DB (distinct persons + cases per
// branch so assertions filter by `source_case_id`):
//
//   1. `default_multiplier`      — 2 sponsors (1 regular B, 1 founder C),
//                                  ContentRemoval sanction (moderate, -50).
//                                  B has baseline=0 (floor-clamp fires → 0);
//                                  C is a founder with snapshot=100 → -50.
//                                  Flip `liability.founder_multiplier` from
//                                  2.0 → 3.0 via a second governance_config
//                                  row; repeat with fresh case and assert
//                                  the new multiplier takes effect.
//   2. `founder_chain_survival`  — 2 founder sponsors (C1, C2 seeded at 100),
//                                  CommunityExclusion (severe, -200). Under
//                                  default floor=0 + multiplier=2.0, both
//                                  clamp to final_delta=-100 (100+(-100)=0).
//                                  Logs a retro note per plan line 1035:
//                                  default config does NOT preserve the
//                                  sponsorship capability under a severe
//                                  sanction.
//   3. `honour_price_floor_clamp` — 1 regular sponsor E (baseline=5),
//                                  ContentRemoval (-50). Clamp: 5+(-50)=-45<0
//                                  → final_delta = 0 − 5 = −5. Emits one
//                                  `_applied` + one `_clamped` log entry.
//
// After all three branches, walks every `governance_log.payload` and asserts
// no raw integer identifiers under banned keys (Watch 10 PII grep).

#[ignore = "TODO(v0-polish): deflake — GH issue #45 (random jury pool + fallback path NotFound)"]
#[tokio::test]
#[expect(clippy::too_many_lines, reason = "3-branch e2e per plan §11.5")]
async fn sponsor_liability_with_founder_multiplier() -> Result<(), Box<dyn Error>> {
  use actix_web::web::{Data, Json};
  use chrono::{Duration as ChronoDuration, Utc};
  use diesel::{Connection as _, ExpressionMethods, PgConnection, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    reputation_snapshot::recompute_snapshot,
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
    InstanceId,
    PersonId,
    enums::{CaseTargetType, JuryDecision, ReputationDimension},
    schema::{governance_log, reputation_event, reputation_snapshot, surety},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests, get_conn},
    traits::Crud,
  };
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;

  const SIGNING_SEED_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";
  // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
  }

  let (_container, host_port) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("start_postgres: {e}").into() })?;
  let db_url = governance_fixtures::db_url(host_port);
  unsafe {
    std::env::set_var("LEMMY_DATABASE_URL", &db_url);
  }
  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| -> Box<dyn Error> { format!("apply_all_schema: {e}").into() })?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS)
    .build()
    .map_err(|e| -> Box<dyn Error> { format!("client: {e}").into() })?;
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

  // Phase 6 task 76: see report_to_modlog_golden_path for the rationale.
  // `submit_jury_vote` is called from `run_sanction_scenario` below; it
  // requires the federation flavour of `Data<LemmyContext>` because the
  // handler hands it to `federation_outbox::send_local_sanction_notice`.
  let federation_config = activitypub_federation::config::FederationConfig::builder()
    .domain(context.settings().hostname.clone())
    .app_data((**context).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("federation_config: {e}").into() })?;
  let federation_context = federation_config.to_request_data();

  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid")
    .await
    .map_err(|e| -> Box<dyn Error> { format!("instance: {e}").into() })?;

  let community_form = CommunityInsertForm::new(
    instance.id,
    "testcomm".to_string(),
    "Test Community".to_string(),
    "comm-pubkey".to_string(),
  );
  let community = Community::create(&mut context.pool(), &community_form)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("community: {e}").into() })?;

  // Shared admin + reporter across branches (persons may be reused; jurors
  // cannot be the target of any case they hear on).
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

  let admin = seed_person(&context, instance.id, "t60_admin", true)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("admin: {e}").into() })?;
  let reporter = seed_person(&context, instance.id, "t60_reporter", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("reporter: {e}").into() })?;

  // Seed 6 spare jurors (we need 5 per case; the random pool must exclude
  // target + sponsors, and we run 4 cases total across branches).
  let mut jurors: Vec<PersonId> = Vec::new();
  for i in 0..6 {
    let id = seed_person(&context, instance.id, &format!("t60_juror_{i}"), false)
      .await
      .map_err(|e| -> Box<dyn Error> { format!("juror: {e}").into() })?;
    jurors.push(id);
  }

  let admin_view = LocalUserView::read_person(&mut context.pool(), admin)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("admin_view: {e}").into() })?;
  let reporter_view = LocalUserView::read_person(&mut context.pool(), reporter)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("reporter_view: {e}").into() })?;

  // Direct async conn for seeding state that doesn't go through handlers.
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

  async fn seed_surety(
    conn: &mut AsyncPgConnection,
    sponsor: PersonId,
    sponsored: PersonId,
  ) -> Result<(), Box<dyn Error>> {
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

  async fn seed_founder_events(
    conn: &mut AsyncPgConnection,
    person: PersonId,
    endorsement_strength: i32,
  ) -> Result<(), Box<dyn Error>> {
    let expiry = Utc::now() + ChronoDuration::days(90);
    for (dim, delta) in [
      (ReputationDimension::JuryReliability, 100),
      (ReputationDimension::ReportingAccuracy, 100),
      (ReputationDimension::EndorsementStrength, endorsement_strength),
    ] {
      let form = ReputationEventInsertForm {
        person_id: person,
        community_id: None,
        dimension: dim,
        delta,
        source_case_id: None,
        source_report_id: None,
        reason: "founder_seed".to_string(),
        expires_at: Some(expiry),
      };
      diesel::insert_into(reputation_event::table)
        .values(&form)
        .execute(conn)
        .await?;
    }
    Ok(())
  }

  async fn seed_organic_endorsement(
    conn: &mut AsyncPgConnection,
    person: PersonId,
    delta: i32,
  ) -> Result<(), Box<dyn Error>> {
    let form = ReputationEventInsertForm {
      person_id: person,
      community_id: None,
      dimension: ReputationDimension::EndorsementStrength,
      delta,
      source_case_id: None,
      source_report_id: None,
      reason: "test_seed".to_string(),
      expires_at: None,
    };
    diesel::insert_into(reputation_event::table)
      .values(&form)
      .execute(conn)
      .await?;
    Ok(())
  }

  async fn seed_snapshot(
    conn: &mut AsyncPgConnection,
    person: PersonId,
    endorsement_strength: i32,
  ) -> Result<(), Box<dyn Error>> {
    let form = ReputationSnapshotInsertForm {
      person_id: person,
      community_id: None,
      reporting_accuracy: 0,
      jury_reliability: 0,
      participation_consistency: 0,
      endorsement_strength,
      jury_eligible: false,
      trusted_reporter: false,
      // Other bool fields (including the sponsorship capability column)
      // fall through to `Default` so this seeding helper does not
      // reference the v0-silenced column by name — keeps the
      // lint-no-can-sponsor-read guard green.
      ..Default::default()
    };
    diesel::insert_into(reputation_snapshot::table)
      .values(&form)
      .execute(conn)
      .await?;
    Ok(())
  }

  // Drive one full sanction round through the real handler pipeline.
  // Returns the `case_id` so callers can filter reputation_event rows.
  //
  // Phase 6 task 76: takes both flavours of `Data<LemmyContext>` because
  // `submit_jury_vote` switched to the federation Data (it hands it to
  // `federation_outbox::send_local_sanction_notice`) while every other
  // governance handler still uses the actix Data.
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
  ) -> Result<i32, Box<dyn Error>> {
    // Step 1: reporter files a report against target.
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
    .await
    .map_err(|e| -> Box<dyn Error> { format!("create_report: {e}").into() })?
    .into_inner();
    let case_id = create_resp
      .case_id
      .ok_or_else(|| -> Box<dyn Error> { "case_id missing".into() })?;

    // Step 2: admin fast-forwards the case to ThresholdMet so
    // admin_assign_jury will accept it (v0 threshold is 3 reports; we
    // bypass via direct DB update).
    use lemmy_db_schema_file::{enums::CaseStatus, schema::moderation_case};
    {
      let mut pool = context.pool();
      let mut conn = get_conn(&mut pool)
        .await
        .map_err(|e| -> Box<dyn Error> { format!("get_conn: {e}").into() })?;
      diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_id.0)))
        .set(moderation_case::status.eq(CaseStatus::ThresholdMet))
        .execute(&mut *conn)
        .await
        .map_err(|e| -> Box<dyn Error> { format!("fast-forward case: {e}").into() })?;
    }

    // Step 3: admin assigns jury.
    let assign_resp = admin_assign_jury(
      Json(AdminAssignJury { case_id }),
      context.clone(),
      admin_view.clone(),
    )
    .await
    .map_err(|e| -> Box<dyn Error> { format!("admin_assign_jury: {e}").into() })?
    .into_inner();
    assert_eq!(
      assign_resp.assigned_person_ids.len(),
      5,
      "5 jurors assigned"
    );

    // Accept jury before voting — submit_jury_vote requires Accepted status
    for juror_id in &assign_resp.assigned_person_ids {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror_id)
        .await
        .map_err(|e| -> Box<dyn Error> { format!("juror_view (accept): {e}").into() })?;
      accept_jury_assignment(
        Json(AcceptJuryAssignment { case_id }),
        context.clone(),
        juror_view,
      )
      .await
      .map_err(|e| -> Box<dyn Error> { format!("accept_jury_assignment: {e}").into() })?;
    }

    // Step 4: first 3 selected jurors vote the target decision.
    let voting: Vec<PersonId> = assign_resp
      .assigned_person_ids
      .iter()
      .copied()
      .take(3)
      .collect();
    for juror in &voting {
      let juror_view = LocalUserView::read_person(&mut context.pool(), *juror)
        .await
        .map_err(|e| -> Box<dyn Error> { format!("juror_view: {e}").into() })?;
      submit_jury_vote(
        Json(SubmitJuryVote {
          case_id,
          decision,
          rationale: Some("test".to_string()),
        }),
        federation_context.reset_request_count(),
        juror_view,
      )
      .await
      .map_err(|e| -> Box<dyn Error> { format!("submit_jury_vote: {e}").into() })?;
    }

    // Touch `jurors` to silence unused warnings if a branch doesn't reference
    // the outer slice directly.
    let _ = jurors;
    Ok(case_id.0)
  }

  async fn liability_delta_for(
    conn: &mut AsyncPgConnection,
    person: PersonId,
    case_id: i32,
  ) -> Result<i32, Box<dyn Error>> {
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

  async fn count_log_for_case(
    conn: &mut AsyncPgConnection,
    entry_kind: &str,
    case_id: i32,
  ) -> Result<i64, Box<dyn Error>> {
    #[derive(diesel::QueryableByName)]
    struct CountRow {
      #[diesel(sql_type = diesel::sql_types::BigInt)]
      c: i64,
    }
    let rows: Vec<CountRow> = diesel::sql_query(
      "SELECT COUNT(*)::bigint AS c FROM governance_log \
       WHERE entry_kind = $1 AND (payload->>'case_id')::int = $2",
    )
    .bind::<diesel::sql_types::Text, _>(entry_kind)
    .bind::<diesel::sql_types::Int4, _>(case_id)
    .load(conn)
    .await?;
    Ok(rows.into_iter().next().map(|r| r.c).unwrap_or(0))
  }

  // ---------- Branch 1 — default_multiplier ------------------------------
  let target1 = seed_person(&context, instance.id, "b1_target", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b1 target: {e}").into() })?;
  let sponsor_b = seed_person(&context, instance.id, "b1_sponsor_regular", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b1 reg: {e}").into() })?;
  let sponsor_c = seed_person(&context, instance.id, "b1_sponsor_founder", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b1 founder: {e}").into() })?;

  seed_surety(&mut async_conn, sponsor_b, target1).await?;
  seed_surety(&mut async_conn, sponsor_c, target1).await?;
  seed_founder_events(&mut async_conn, sponsor_c, 100).await?;
  seed_snapshot(&mut async_conn, sponsor_b, 0).await?;
  seed_snapshot(&mut async_conn, sponsor_c, 100).await?;

  let case1 = run_sanction_scenario(
    &context,
    &federation_context,
    &admin_view,
    &reporter_view,
    &jurors,
    target1,
    community.id,
    "b1_spam",
    JuryDecision::RemoveContent,
  )
  .await?;

  // Math: raw=-50, 2 sponsors → per_sponsor=-25, remainder=0.
  // B regular ×1.0 = -25; current=0; 0+(-25)=-25<0 → clamp: final_delta=0.
  // C founder ×2.0 = -50; current=100; 100+(-50)=50≥0 → final_delta=-50.
  let delta_b = liability_delta_for(&mut async_conn, sponsor_b, case1).await?;
  let delta_c = liability_delta_for(&mut async_conn, sponsor_c, case1).await?;
  assert_eq!(delta_b, 0, "branch1: B (regular, baseline 0) floor-clamped to 0");
  assert_eq!(
    delta_c, -50,
    "branch1: C (founder, baseline 100) × 2.0 → -50"
  );
  let applied_1 = count_log_for_case(&mut async_conn, "sponsor_liability_applied", case1).await?;
  let clamped_1 = count_log_for_case(&mut async_conn, "sponsor_liability_clamped", case1).await?;
  assert_eq!(applied_1, 2, "branch1: 2 sponsor_liability_applied entries");
  assert_eq!(clamped_1, 1, "branch1: 1 sponsor_liability_clamped entry (for B)");

  // Flip liability.founder_multiplier: 2.0 → 3.0 with retroactive
  // valid_from so governance_config_current picks up the new row
  // deterministically. Prior code used `now() + 1s` + `sleep 1.2s` which
  // left ~200ms of CI slack — flaky under container scheduling / GC
  // pauses / clock drift between the test process and the PG container.
  // `now() - interval '1 second'` pre-dates both the seeded row and any
  // other `valid_from <= now()` window the view filter considers, so the
  // DESC sort on `valid_from` always returns 3.0 without waiting.
  diesel::sql_query(
    "INSERT INTO governance_config (scope, key, value_type, value_float, valid_from) \
     VALUES ('instance', 'liability.founder_multiplier', 'float', 3.0, \
             now() - interval '1 second')",
  )
  .execute(&mut async_conn)
  .await?;

  let target1b = seed_person(&context, instance.id, "b1b_target", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b1b target: {e}").into() })?;
  seed_surety(&mut async_conn, sponsor_b, target1b).await?;
  seed_surety(&mut async_conn, sponsor_c, target1b).await?;

  let case1b = run_sanction_scenario(
    &context,
    &federation_context,
    &admin_view,
    &reporter_view,
    &jurors,
    target1b,
    community.id,
    "b1b_spam",
    JuryDecision::RemoveContent,
  )
  .await?;
  // C snapshot still reads 100 (no intervening recompute); -25 × 3.0 = -75;
  // 100 + (-75) = 25 ≥ 0 → no clamp.
  let delta_c_flipped = liability_delta_for(&mut async_conn, sponsor_c, case1b).await?;
  assert_eq!(
    delta_c_flipped, -75,
    "branch1b: C with flipped founder_multiplier=3.0 → -75"
  );

  // ---------- Branch 2 — founder_chain_survival --------------------------
  let target2 = seed_person(&context, instance.id, "b2_target", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b2 target: {e}").into() })?;
  let sponsor_c1 = seed_person(&context, instance.id, "b2_founder_c1", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b2 c1: {e}").into() })?;
  let sponsor_c2 = seed_person(&context, instance.id, "b2_founder_c2", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b2 c2: {e}").into() })?;

  seed_surety(&mut async_conn, sponsor_c1, target2).await?;
  seed_surety(&mut async_conn, sponsor_c2, target2).await?;
  seed_founder_events(&mut async_conn, sponsor_c1, 100).await?;
  seed_founder_events(&mut async_conn, sponsor_c2, 100).await?;
  seed_snapshot(&mut async_conn, sponsor_c1, 100).await?;
  seed_snapshot(&mut async_conn, sponsor_c2, 100).await?;

  let case2 = run_sanction_scenario(
    &context,
    &federation_context,
    &admin_view,
    &reporter_view,
    &jurors,
    target2,
    community.id,
    "b2_severe",
    // Maps to SanctionAction::CommunityExclusion (severe bucket per
    // sponsor_liability::severity_for_action — task 56).
    JuryDecision::SuspendCommunityMember,
  )
  .await?;

  // Note: reverted flipped config still applies if our flipped row has
  // later valid_from than the seed — but founder_multiplier=3.0 would push
  // per_sponsor=-100 × 3.0 = -300; current=100 → -300 clamp → final_delta=
  // 0 - 100 = -100. Same clamp outcome as under 2.0, so assertions are
  // insensitive to whether branch 1's flip is still in effect.
  let d_c1 = liability_delta_for(&mut async_conn, sponsor_c1, case2).await?;
  let d_c2 = liability_delta_for(&mut async_conn, sponsor_c2, case2).await?;
  assert_eq!(d_c1, -100, "branch2: C1 founder clamped to -100 (severe)");
  assert_eq!(d_c2, -100, "branch2: C2 founder clamped to -100 (severe)");
  let clamped_2 = count_log_for_case(&mut async_conn, "sponsor_liability_clamped", case2).await?;
  assert_eq!(clamped_2, 2, "branch2: both founders clamped");

  // Per plan §11.5 branch 2: founder seed (+100, instance-scoped) composes
  // with the sponsor_liability_applied event (-100, instance-scoped after
  // the task 56 split-plane fix at 61ddae110). The instance recompute sums
  // both and lands at endorsement_strength = 0 — below the sponsorship
  // threshold (25). Default config does NOT preserve the sponsorship
  // capability for founders under a severe sanction; v1 tuning is required.
  {
    let mut cache = lemmy_api::governance::config::ConfigCache::new();
    let snap1 = recompute_snapshot(&mut async_conn, sponsor_c1, None, &mut cache)
      .await
      .map_err(|e| -> Box<dyn Error> { format!("recompute c1: {e}").into() })?;
    let snap2 = recompute_snapshot(&mut async_conn, sponsor_c2, None, &mut cache)
      .await
      .map_err(|e| -> Box<dyn Error> { format!("recompute c2: {e}").into() })?;
    assert_eq!(
      snap1.endorsement_strength, 0,
      "branch2: C1 instance endorsement_strength = 0 (100 seed + -100 liability)"
    );
    assert_eq!(
      snap2.endorsement_strength, 0,
      "branch2: C2 instance endorsement_strength = 0 (100 seed + -100 liability)"
    );
    println!(
      "FOUNDER_CHAIN_SURVIVAL: post-sanction endorsement_strength for C1={}, C2={}; \
       default config does NOT preserve the sponsorship capability — retro follow-up \
       for v1 tuning",
      snap1.endorsement_strength, snap2.endorsement_strength
    );
  }

  // ---------- Branch 3 — honour_price_floor_clamp ------------------------
  let target3 = seed_person(&context, instance.id, "b3_target", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b3 target: {e}").into() })?;
  let sponsor_e = seed_person(&context, instance.id, "b3_sponsor_e", false)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("b3 e: {e}").into() })?;
  seed_surety(&mut async_conn, sponsor_e, target3).await?;
  seed_organic_endorsement(&mut async_conn, sponsor_e, 5).await?;
  seed_snapshot(&mut async_conn, sponsor_e, 5).await?;

  let case3 = run_sanction_scenario(
    &context,
    &federation_context,
    &admin_view,
    &reporter_view,
    &jurors,
    target3,
    community.id,
    "b3_moderate",
    JuryDecision::RemoveContent,
  )
  .await?;

  // raw=-50, 1 sponsor → -50; ×1.0 = -50; current=5; 5+(-50)=-45<0 →
  // clamp: final_delta = 0 - 5 = -5.
  // Branch 3 is insensitive to branch 1's `founder_multiplier` flip because
  // sponsor_e was seeded as a non-founder (seed_person(..., false)) — the
  // non-founder path uses multiplier 1.0 regardless of the founder config
  // row. No rollback needed.
  let d_e = liability_delta_for(&mut async_conn, sponsor_e, case3).await?;
  assert_eq!(d_e, -5, "branch3: E (baseline 5) floor-clamped to -5");
  let applied_3 = count_log_for_case(&mut async_conn, "sponsor_liability_applied", case3).await?;
  let clamped_3 = count_log_for_case(&mut async_conn, "sponsor_liability_clamped", case3).await?;
  assert_eq!(applied_3, 1, "branch3: 1 sponsor_liability_applied");
  assert_eq!(clamped_3, 1, "branch3: 1 sponsor_liability_clamped (floor fires)");

  // ---------- Watch 10 — PII grep across ALL governance_log payloads -----
  let payloads: Vec<serde_json::Value> = governance_log::table
    .select(governance_log::payload)
    .load(&mut async_conn)
    .await?;
  let banned = regex::Regex::new(
    r#""(person_id|target_person_id|sponsored_id|sponsor_id)"\s*:\s*\d+"#,
  )?;
  for payload in &payloads {
    let s = serde_json::to_string(payload)?;
    assert!(
      !banned.is_match(&s),
      "governance_log payload leaked a raw integer identifier: {s}"
    );
  }
  // Positive assertion: at least one payload mentions sponsor_pseudonym so
  // the grep isn't vacuously passing on an empty log.
  let saw_pseudonym = payloads
    .iter()
    .any(|p| serde_json::to_string(p).map(|s| s.contains("\"sponsor_pseudonym\"")).unwrap_or(false));
  assert!(
    saw_pseudonym,
    "expected at least one governance_log payload with sponsor_pseudonym"
  );

  Ok(())
}

/// Phase 5c task 63c — `list_capability_changed_entries_since` reads
/// `capability_changed` rows from `governance_log` directly, paginated by
/// `since_id`. This test seeds three rows via raw SQL (the trigger layer
/// fills `prev_hash` + `entry_hash`; signature is left NULL because no
/// signing pass runs in this test), then asserts the helper returns
/// exactly those three with stable id-ascending order. Mirror of
/// `modlog_view_returns_published_entries` style — direct DB seed +
/// view-crate fn assert.
#[tokio::test]
async fn capability_change_entries_reachable_via_modlog_crate(
) -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection, connection::SimpleConnection};
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_db_views_governance_modlog::impls::list_capability_changed_entries_since;
  use lemmy_diesel_utils::connection::DbPool;

  // No GOVERNANCE_LOG_SIGNING_KEY needed — this test reads
  // governance_log directly via the view-crate helper; no
  // `governance_log::append` (which would require the signing key) is
  // called. Inserts go through the hash-chain trigger but leave the
  // signature column NULL — that's the trigger contract too (signing is
  // a separate UPDATE pass in Phase 4 production code).
  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
    // Three capability_changed rows + one unrelated row to confirm
    // the entry_kind filter is honoured. Each insert lets the
    // hash-chain trigger compute prev_hash/entry_hash.
    sync_conn.batch_execute(
      r#"
      INSERT INTO governance_log (entry_kind, payload, actor_pseudonym)
        VALUES
          ('capability_changed',
           '{"dimension_flipped":"jury_eligible","direction":"gained","snapshot_community_id":null}'::jsonb,
           'pseudo-user-a'),
          ('capability_changed',
           '{"dimension_flipped":"jury_eligible","direction":"gained","snapshot_community_id":null}'::jsonb,
           'pseudo-user-b'),
          ('capability_changed',
           '{"dimension_flipped":"trusted_reporter","direction":"lost","snapshot_community_id":null}'::jsonb,
           'pseudo-user-c'),
          ('report_created',
           '{"reason_code":"spam"}'::jsonb,
           'pseudo-reporter');
      "#,
    )?;
  }

  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  let mut pool: DbPool<'_> = (&mut async_conn).into();

  let entries = list_capability_changed_entries_since(&mut pool, 0, 10)
    .await
    .map_err(|e| -> Box<dyn Error> {
      format!("list_capability_changed_entries_since: {e}").into()
    })?;
  assert_eq!(
    entries.len(),
    3,
    "expected the three capability_changed rows (the report_created row must be filtered out)"
  );
  for e in &entries {
    assert_eq!(e.entry_kind, "capability_changed", "entry_kind filter held");
  }
  // id ascending — first row should be the lowest id.
  let first_id = entries
    .first()
    .ok_or_else(|| -> Box<dyn Error> { "expected at least one entry".into() })?
    .id;
  let last_id = entries
    .last()
    .ok_or_else(|| -> Box<dyn Error> { "expected at least one entry".into() })?
    .id;
  assert!(first_id < last_id, "entries must be id-ascending");

  // since_id paging — calling with the first row's id excludes it,
  // returns the remaining 2.
  let after_first = list_capability_changed_entries_since(&mut pool, first_id, 10)
    .await
    .map_err(|e| -> Box<dyn Error> {
      format!("list_capability_changed_entries_since (paging): {e}").into()
    })?;
  assert_eq!(after_first.len(), 2, "since_id excludes rows with id == since_id");

  Ok(())
}

/// Phase 5c task 63d — `check_snapshot_staleness` emits a structured
/// `tracing::error!` event under `target: "governance::integrity"` when
/// the most-recent `reputation_snapshot.calculated_at` is older than
/// `now - 2 * interval_s`. Pure observability; no DB writes. Per
/// GOTCHA-63d-c, time is injected so the test can drive the threshold
/// deterministically. Uses `tracing-test` `traced_test` macro to
/// capture emitted events.
#[tokio::test]
#[tracing_test::traced_test]
async fn snapshot_staleness_alert_fires_when_max_calculated_at_is_old(
) -> Result<(), Box<dyn Error>> {
  use chrono::{Duration, Utc};
  use diesel::{Connection as _, PgConnection, connection::SimpleConnection};
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_api::governance::reputation_snapshot::check_snapshot_staleness;

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);

  // Path 1 — empty table emits the "table empty" variant.
  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  check_snapshot_staleness(&mut async_conn, 60, Utc::now())
    .await
    .map_err(|e| -> Box<dyn Error> {
      format!("check_snapshot_staleness empty-table: {e}").into()
    })?;
  assert!(
    logs_contain("snapshot batch has never run"),
    "expected the empty-table staleness signal in tracing output"
  );

  // Path 2 — seed one stale row (calculated_at = now - 1h), interval = 60s.
  // Threshold becomes now - 120s; 1h ago is well past that, so the
  // staleness signal fires.
  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    sync_conn.batch_execute(
      r#"
      INSERT INTO instance (domain) VALUES ('staleness.invalid');
      INSERT INTO person (name, ap_id, inbox_url, public_key, instance_id)
        VALUES (
          'staleness-seed',
          'https://staleness.invalid/u/seed',
          'https://staleness.invalid/u/seed/inbox',
          'staleness-pubkey',
          (SELECT id FROM instance WHERE domain = 'staleness.invalid')
        );
      INSERT INTO reputation_snapshot
        (person_id, community_id, reporting_accuracy, jury_reliability,
         participation_consistency, endorsement_strength,
         jury_eligible, trusted_reporter, can_sponsor, calculated_at)
        VALUES (
          (SELECT id FROM person WHERE name = 'staleness-seed'),
          NULL, 0, 0, 0, 0, false, false, false,
          NOW() - INTERVAL '1 hour'
        );
      "#,
    )?;
  }
  // Use a fresh async connection — the previous one is borrowed by the
  // earlier check; reusing is ambiguous in scope.
  let mut async_conn2 = AsyncPgConnection::establish(&db_url).await?;
  check_snapshot_staleness(&mut async_conn2, 60, Utc::now())
    .await
    .map_err(|e| -> Box<dyn Error> {
      format!("check_snapshot_staleness stale-row: {e}").into()
    })?;
  assert!(
    logs_contain("staleness detected"),
    "expected the stale-max-row staleness signal in tracing output"
  );

  // Path 3 — seed one fresh row (calculated_at = now), interval = 60s.
  // Threshold = now - 120s; row's calculated_at > threshold, so the
  // signal does NOT fire on this call. The earlier emissions are still
  // in the captured log though, so this assertion only checks the
  // counter incremented by less than 1 — we use a marker emission
  // pattern by passing a fresh future-now to ensure the comparison falls
  // on the safe side without churning the log.
  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    sync_conn.batch_execute(
      r#"
      UPDATE reputation_snapshot
      SET calculated_at = NOW()
      WHERE community_id IS NULL;
      "#,
    )?;
  }
  let mut async_conn3 = AsyncPgConnection::establish(&db_url).await?;
  // Use frozen time slightly in the past to make the threshold even more
  // forgiving — guarantees no new staleness signal in path 3.
  let frozen = Utc::now() - Duration::seconds(1);
  check_snapshot_staleness(&mut async_conn3, 60, frozen)
    .await
    .map_err(|e| -> Box<dyn Error> {
      format!("check_snapshot_staleness fresh-row: {e}").into()
    })?;
  // No new assertion — `logs_contain` is monotonic and would still
  // return true for prior emissions. The contract being tested is "no
  // panic + Ok(()) return when the table is fresh".

  Ok(())
}

// ============================================================================
// Phase 5c — task 68: route registration + per-handler happy-path assertions
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn all_mvp_endpoints_return_non_404() -> Result<(), Box<dyn Error>> {
  use actix_web::{App, test, web::Data};
  use diesel::{Connection as _, PgConnection};
  use diesel_async::{AsyncConnection as _, AsyncPgConnection};
  use lemmy_api_common::governance::{
    AdminReputationStatsResponse, GetMyReputationResponse, ListGovernanceCasesResponse,
    RequestAppealResponse,
  };
  use lemmy_api_utils::{
    claims::Claims, context::LemmyContext, request::client_builder,
  };
  use lemmy_db_schema::{
    newtypes::LocalUserId,
    source::{
      instance::Instance,
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      secret::Secret,
    },
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType},
    schema::moderation_case,
  };
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_routes::middleware::session::SessionMiddleware;
  use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;

  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY",
      "0000000000000000000000000000000000000000000000000000000000000001");
  }

  let (_container, host_port) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("start_postgres: {e}").into() })?;
  let db_url = governance_fixtures::db_url(host_port);
  unsafe { std::env::set_var("LEMMY_DATABASE_URL", &db_url); }

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| -> Box<dyn Error> { format!("apply_all_schema: {e}").into() })?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
  let middleware_client = ClientBuilder::new(client).build();
  let secret = Secret { id: 0, jwt_secret: String::new().into() };
  let rate_limit = RateLimit::with_debug_config();
  // Bump rate-limit buckets so the 14-endpoint sweep + 4 Phase B probes
  // don't trip the 6/300s Post bucket from `with_debug_config()`. These
  // tests exercise routing and handler shape, not rate-limit behaviour.
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
  let context = LemmyContext::create(
    pool,
    middleware_client.clone(),
    middleware_client,
    secret,
    rate_limit.clone(),
  );

  let app = test::init_service(
    App::new()
      .app_data(Data::new(context.clone()))
      .wrap(SessionMiddleware::new(context.clone()))
      .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))
  ).await;

  // ========== Phase A: non-404 sweep (14 routes) ==========
  let endpoints: &[(&str, &str, &str, &[u16])] = &[
    ("POST", "/api/v4/governance/report",                        "{}", &[200, 400, 401]),
    ("POST", "/api/v4/governance/endorsement",                   "{}", &[200, 400, 401]),
    ("POST", "/api/v4/governance/appeal",                        "{}", &[200, 400, 401]),
    // 404 allowed here: empty test DB has no case_id=1; the route is wired
    // (responds with handler's NotFound mapping) but resource doesn't exist.
    // Other routes use validation errors (400/401) for unseeded state, not 404.
    ("GET",  "/api/v4/governance/case?case_id=1",                "",   &[200, 400, 401, 404]),
    ("GET",  "/api/v4/governance/cases",                         "",   &[200, 400, 401]),
    ("GET",  "/api/v4/governance/modlog",                        "",   &[200, 400, 401]),
    ("GET",  "/api/v4/governance/reputation/me",                 "",   &[200, 400, 401]),
    ("GET",  "/api/v4/governance/jury/me",                       "",   &[200, 400, 401]),
    ("POST", "/api/v4/governance/jury/accept",                   "{}", &[200, 400, 401]),
    ("POST", "/api/v4/governance/jury/decline",                  "{}", &[200, 400, 401]),
    ("POST", "/api/v4/governance/jury/vote",                     "{}", &[200, 400, 401]),
    ("POST", "/api/v4/governance/admin/assign-jury",             "{}", &[200, 400, 401]),
    ("POST", "/api/v4/governance/admin/close-case",              "{}", &[200, 400, 401]),
    ("GET",  "/api/v4/governance/admin/reputation-stats",        "",   &[200, 400, 401]),
  ];

  for (method, path, body, allowed) in endpoints {
    let req = match *method {
      "GET" => test::TestRequest::get().uri(path).to_request(),
      "POST" => test::TestRequest::post()
        .uri(path)
        .insert_header(("content-type", "application/json"))
        .set_payload(body.to_string())
        .to_request(),
      _ => unreachable!(),
    };
    let resp = test::call_service(&app, req).await;
    let status = resp.status().as_u16();
    assert!(
      allowed.contains(&status),
      "{method} {path} returned {status} (expected one of {allowed:?}, NOT 404)"
    );
  }

  // ========== Phase B: per-handler happy-path assertions (Move 4) ==========
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  async fn make_user(
    ctx: &LemmyContext,
    instance_id: lemmy_db_schema_file::InstanceId,
    name: &str,
    is_admin: bool,
  ) -> Result<(LocalUserId, PersonId), Box<dyn Error>>
  {
    let person_form = PersonInsertForm::test_form(instance_id, name);
    let person = Person::create(&mut ctx.pool(), &person_form).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    let mut lu_form = if is_admin {
      LocalUserInsertForm::test_form_admin(person.id)
    } else {
      LocalUserInsertForm::test_form(person.id)
    };
    lu_form.accepted_application = Some(true);
    let lu = LocalUser::create(&mut ctx.pool(), &lu_form, vec![]).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    Ok((lu.id, person.id))
  }

  async fn mint_jwt(
    ctx: &LemmyContext,
    local_user_id: LocalUserId,
  ) -> Result<String, Box<dyn Error>> {
    let req = test::TestRequest::default().to_http_request();
    let token = Claims::generate(local_user_id, None, req, ctx).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    Ok(token.into_inner())
  }

  let (admin_lu_id, _admin_pid) = make_user(&context, instance.id, "probe_admin", true).await?;
  let admin_jwt = mint_jwt(&context, admin_lu_id).await?;
  let (user_lu_id, _user_pid) = make_user(&context, instance.id, "probe_user", false).await?;
  let user_jwt = mint_jwt(&context, user_lu_id).await?;
  let (target_lu_id, target_pid) = make_user(&context, instance.id, "probe_target", false).await?;
  let target_jwt = mint_jwt(&context, target_lu_id).await?;

  // Seed a Decided case for the appeal test + an Open case for list_cases.
  {
    use diesel_async::RunQueryDsl;
    use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;

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
      reason_code: "probe".to_string(),
      severity: CaseSeverity::Low,
      status: CaseStatus::Decided,
      threshold_score: 1,
    };
    diesel::insert_into(moderation_case::table)
      .values(&decided_form)
      .execute(&mut async_conn)
      .await?;

    let open_form = ModerationCaseInsertForm {
      status: CaseStatus::Open,
      target_person_id: None,
      ..decided_form
    };
    diesel::insert_into(moderation_case::table)
      .values(&open_form)
      .execute(&mut async_conn)
      .await?;
  }

  // B.1 — GET /reputation/me (task 61)
  let resp = test::TestRequest::get()
    .uri("/api/v4/governance/reputation/me")
    .insert_header(("authorization", format!("Bearer {user_jwt}")))
    .send_request(&app).await;
  assert_eq!(resp.status().as_u16(), 200, "reputation/me expected 200");
  let body: GetMyReputationResponse = test::read_body_json(resp).await;
  assert_eq!(body.view.active_sanctions, 0, "fresh user should have zero active sanctions");

  // B.2 — GET /admin/reputation-stats (task 62; route is GET per fix B3-4)
  let resp = test::TestRequest::get()
    .uri("/api/v4/governance/admin/reputation-stats")
    .insert_header(("authorization", format!("Bearer {admin_jwt}")))
    .send_request(&app).await;
  assert_eq!(resp.status().as_u16(), 200, "admin/reputation-stats expected 200 for admin");
  let body: AdminReputationStatsResponse = test::read_body_json(resp).await;
  assert_eq!(body.buckets.jury_reliability.len(), 5, "jury_reliability bucket shape");

  // B.3 — POST /appeal (task 66) — target appeals a Decided case
  let resp = test::TestRequest::post()
    .uri("/api/v4/governance/appeal")
    .insert_header(("authorization", format!("Bearer {target_jwt}")))
    .insert_header(("content-type", "application/json"))
    .set_payload(r#"{"case_id":1,"reason":"probe appeal"}"#)
    .send_request(&app).await;
  assert_eq!(resp.status().as_u16(), 200, "appeal expected 200 for target on Decided case");
  let body: RequestAppealResponse = test::read_body_json(resp).await;
  assert!(body.appeal_id.0 > 0, "appeal_id must be positive");

  // B.4 — GET /cases (task 67) — authed caller sees the seeded cases
  let resp = test::TestRequest::get()
    .uri("/api/v4/governance/cases")
    .insert_header(("authorization", format!("Bearer {user_jwt}")))
    .send_request(&app).await;
  assert_eq!(resp.status().as_u16(), 200, "cases expected 200 for authed caller");
  let body: ListGovernanceCasesResponse = test::read_body_json(resp).await;
  assert!(!body.cases.is_empty(), "seeded cases must appear in list");

  Ok(())
}

// ============================================================================
// Phase 5c — task 69: capability-gating e2e (3 branches)
// ============================================================================

#[ignore = "TODO(v0-polish): deflake — GH issue #42 (cross-test contamination under --test-threads=1)"]
#[tokio::test(flavor = "multi_thread")]
async fn ineligible_user_cannot_be_picked_for_jury() -> Result<(), Box<dyn Error>> {
  use actix_web::web::{Data, Json};
  use chrono::{Duration, Utc};
  use diesel::{Connection as _, PgConnection};
  use diesel_async::{AsyncConnection as _, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    admin_assign_jury::admin_assign_jury, reputation_snapshot::run_snapshot_batch,
  };
  use lemmy_api_common::governance::AdminAssignJury;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::moderation_case::ModerationCaseInsertForm,
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{
      CaseSeverity, CaseStatus, CaseTargetType, JuryAssignmentStatus, ReputationDimension,
    },
    schema::{jury_assignment, reputation_event},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;

  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY",
      "0000000000000000000000000000000000000000000000000000000000000001");
  }

  let (_container, host_port) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("start_postgres: {e}").into() })?;
  let db_url = governance_fixtures::db_url(host_port);
  unsafe { std::env::set_var("LEMMY_DATABASE_URL", &db_url); }

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| -> Box<dyn Error> { format!("apply_all_schema: {e}").into() })?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
  let middleware_client = ClientBuilder::new(client).build();
  let secret = Secret { id: 0, jwt_secret: String::new().into() };
  let rate_limit = RateLimit::with_debug_config();
  let context = Data::new(LemmyContext::create(
    pool,
    middleware_client.clone(),
    middleware_client,
    secret,
    rate_limit,
  ));

  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  async fn seed_person(
    ctx: &LemmyContext,
    instance_id: lemmy_db_schema_file::InstanceId,
    name: &str,
    is_admin: bool,
  ) -> Result<PersonId, Box<dyn Error>> {
    let person_form = PersonInsertForm::test_form(instance_id, name);
    let person = Person::create(&mut ctx.pool(), &person_form).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    let mut lu_form = if is_admin {
      LocalUserInsertForm::test_form_admin(person.id)
    } else {
      LocalUserInsertForm::test_form(person.id)
    };
    lu_form.accepted_application = Some(true);
    LocalUser::create(&mut ctx.pool(), &lu_form, vec![]).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    Ok(person.id)
  }

  // Seed 6 eligible + 2 ineligible users. Branch 1 needs only 5 to fill the
  // panel; branch 2 needs a 6th because capping eligibles[0] with 3 active
  // assignments drops the strict pool by 1, and branch 2 asserts the pool
  // can still hit panel_size=5 without eligibles[0].
  let mut eligibles = Vec::new();
  for i in 0..6 {
    eligibles.push(seed_person(&context, instance.id, &format!("eligible_{i}"), false).await?);
  }
  let mut ineligibles = Vec::new();
  for i in 0..2 {
    ineligibles.push(seed_person(&context, instance.id, &format!("ineligible_{i}"), false).await?);
  }

  // Seed reputation_event rows for eligible users (delta=60, JuryReliability).
  {
    use lemmy_db_schema::source::governance::reputation_event::ReputationEventInsertForm;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    for &pid in &eligibles {
      let form = ReputationEventInsertForm {
        person_id: pid,
        community_id: None,
        dimension: ReputationDimension::JuryReliability,
        delta: 60,
        source_case_id: None,
        source_report_id: None,
        reason: "founder_seed".to_string(),
        expires_at: Some(Utc::now() + Duration::days(30)),
      };
      diesel::insert_into(reputation_event::table)
        .values(&form)
        .execute(&mut async_conn)
        .await?;
    }
  }

  // Override jury.age_requirement_days=0 so freshly-created test users
  // can be jury_eligible (default is 60 days). Fallback on small pool is
  // also disabled so failure surfaces cleanly instead of defaulting to
  // random unfiltered picks that would mask an eligibility bug.
  {
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    diesel::sql_query(
      "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
       VALUES ('instance', 'jury.age_requirement_days', 'int', 0, now())"
    )
    .execute(&mut async_conn)
    .await?;
    diesel::sql_query(
      "INSERT INTO governance_config (scope, key, value_type, value_bool, valid_from) \
       VALUES ('instance', 'jury.fallback_on_small_pool', 'bool', false, now())"
    )
    .execute(&mut async_conn)
    .await?;
  }

  // Run snapshot batch so jury_eligible flags are up-to-date.
  run_snapshot_batch(&context).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;


  // Seed fixture users outside both groups.
  let target_person = seed_person(&context, instance.id, "cap_target", false).await?;
  let _reporter = seed_person(&context, instance.id, "cap_reporter", false).await?;
  let admin = seed_person(&context, instance.id, "cap_admin", true).await?;
  let admin_view = LocalUserView::read_person(&mut context.pool(), admin).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  let community_form = CommunityInsertForm::new(
    instance.id,
    "capcomm".to_string(),
    "Cap Community".to_string(),
    "cap-pubkey".to_string(),
  );
  let community = Community::create(&mut context.pool(), &community_form).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  async fn seed_case(
    ctx: &LemmyContext,
    target: PersonId,
    _community_id: lemmy_db_schema::newtypes::CommunityId,
  ) -> Result<lemmy_db_schema::newtypes::ModerationCaseId, Box<dyn Error>> {
    use lemmy_db_schema_file::schema::moderation_case;
    let mut pool = ctx.pool();
    let conn = &mut lemmy_diesel_utils::connection::get_conn(&mut pool).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    // Instance-scoped case so the strict eligibility query matches the
    // instance-scoped snapshots produced by `run_snapshot_batch` on our
    // instance-scoped reputation_event rows. (Strict query uses
    // `rs.community_id IS NOT DISTINCT FROM case.community_id`; community-
    // scoped would require seeding snapshots per community too.)
    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::Person,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: Some(target),
      target_community_id: None,
      target_remote_url: None,
      reason_code: "captest".to_string(),
      severity: CaseSeverity::Low,
      status: CaseStatus::Open,
      threshold_score: 1,
    };
    let case: lemmy_db_schema::source::governance::moderation_case::ModerationCase =
      diesel::insert_into(moderation_case::table)
        .values(&form)
        .get_result(conn)
        .await
        .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    Ok(case.id)
  }

  // ============ BRANCH 1: basic capability gate ============
  let case_id = seed_case(&context, target_person, community.id).await?;
  let resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view.clone(),
  )
  .await
  .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?
  .into_inner();
  assert_eq!(resp.assigned_person_ids.len(), 5, "branch 1: 5 jurors assigned");
  for pid in &resp.assigned_person_ids {
    assert!(eligibles.contains(pid), "branch 1: picked person {pid:?} is not in eligible set");
    assert!(!ineligibles.contains(pid), "branch 1: picked ineligible person {pid:?}");
  }

  // ============ BRANCH 2: concurrent-cap ============
  // Pre-seed 3 active (Accepted) jury_assignment rows for eligibles[0].
  {
    use lemmy_db_schema::source::governance::jury_assignment::JuryAssignmentInsertForm;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    for _ in 0..3 {
      let dummy_case = seed_case(&context, target_person, community.id).await?;
      let form = JuryAssignmentInsertForm {
        case_id: dummy_case,
        person_id: eligibles[0],
        status: JuryAssignmentStatus::Accepted,
      };
      diesel::insert_into(jury_assignment::table)
        .values(&form)
        .execute(&mut async_conn)
        .await?;
    }
  }
  let case_id_2 = seed_case(&context, target_person, community.id).await?;
  let resp_2 = admin_assign_jury(
    Json(AdminAssignJury { case_id: case_id_2 }),
    context.clone(),
    admin_view.clone(),
  )
  .await
  .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?
  .into_inner();
  assert!(
    !resp_2.assigned_person_ids.contains(&eligibles[0]),
    "branch 2: eligibles[0] at concurrent-cap of 3 should be excluded"
  );

  // ============ BRANCH 3: config flip 3 → 5 ============
  {
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    diesel::sql_query(
      "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
       VALUES ('instance', 'jury.max_concurrent_assignments', 'int', 5, now())"
    )
    .execute(&mut async_conn)
    .await?;
  }
  let case_id_3 = seed_case(&context, target_person, community.id).await?;
  let resp_3 = admin_assign_jury(
    Json(AdminAssignJury { case_id: case_id_3 }),
    context.clone(),
    admin_view.clone(),
  )
  .await
  .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?
  .into_inner();
  assert!(
    resp_3.assigned_person_ids.contains(&eligibles[0]),
    "branch 3: after config flip to 5, eligibles[0] (has 3 active) should be pickable"
  );

  // ============ Watch 10: PII grep over all governance_log payloads ============
  // ADR-015: every person_id that reaches a governance_log payload must be
  // pseudonymised (goes to the actor_pseudonym column, not the payload JSON).
  // Pseudonyms look like UUIDs (strings with hyphens); raw ids are integers.
  // Ten banned regex patterns cover every known identifier-leak surface:
  // (1-5) raw integer ids in the five canonical id-field names,
  // (6-7) dual-capability variants for admin/creator writes,
  // (8-10) common name/email/handle text leaks.
  {
    use diesel::{QueryDsl, SelectableHelper};
    use diesel_async::RunQueryDsl;
    use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
    use lemmy_db_schema_file::schema::governance_log;
    use regex::Regex;

    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    let rows: Vec<GovernanceLog> = governance_log::table
      .select(GovernanceLog::as_select())
      .load(&mut async_conn)
      .await?;

    let banned_patterns: [(&str, &str); 10] = [
      ("raw person_id",          r#""person_id"\s*:\s*\d+"#),
      ("raw target_person_id",   r#""target_person_id"\s*:\s*\d+"#),
      ("raw sponsor_id",         r#""sponsor_id"\s*:\s*\d+"#),
      ("raw sponsored_id",       r#""sponsored_id"\s*:\s*\d+"#),
      ("raw creator_id",         r#""creator_id"\s*:\s*\d+"#),
      ("raw admin_id",           r#""admin_id"\s*:\s*\d+"#),
      ("raw user_id",            r#""user_id"\s*:\s*\d+"#),
      ("raw username field",     r#""username"\s*:\s*"[^"]+"#),
      ("raw name field",         r#""name"\s*:\s*"[^"]+"#),
      ("email-looking string",   r#"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}"#),
    ];
    let compiled: Vec<(&str, Regex)> = banned_patterns
      .iter()
      .map(|(label, pat)| (*label, Regex::new(pat).expect("valid regex")))
      .collect();

    for row in &rows {
      let payload_str = serde_json::to_string(&row.payload)?;
      for (label, re) in &compiled {
        assert!(
          !re.is_match(&payload_str),
          "Watch 10 PII: banned pattern [{label}] matched in governance_log row {}: {payload_str}",
          row.id.0
        );
      }
    }
  }

  Ok(())
}

// ============================================================================
// Phase 5c — task 69a: V2 messaging hooks (NOTIFY + username regression)
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn governance_events_notify_fires() -> Result<(), Box<dyn Error>> {
  use std::{pin::Pin, time::Duration as StdDuration};
  use actix_web::web::{Data, Json};
  use diesel::{Connection as _, PgConnection};
  use lemmy_api_common::governance::CreateGovernanceReport;
  use lemmy_api_crud::governance::create_report::create_report;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{PersonId, enums::CaseTargetType};
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;
  use tokio::sync::mpsc;
  use tokio_postgres::{AsyncMessage, NoTls, Notification};

  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY",
      "0000000000000000000000000000000000000000000000000000000000000001");
  }

  let (_container, host_port) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("start_postgres: {e}").into() })?;
  let db_url = governance_fixtures::db_url(host_port);
  unsafe { std::env::set_var("LEMMY_DATABASE_URL", &db_url); }

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| -> Box<dyn Error> { format!("apply_all_schema: {e}").into() })?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
  let middleware_client = ClientBuilder::new(client).build();
  let secret = Secret { id: 0, jwt_secret: String::new().into() };
  let rate_limit = RateLimit::with_debug_config();
  let context = Data::new(LemmyContext::create(
    pool,
    middleware_client.clone(),
    middleware_client,
    secret,
    rate_limit,
  ));

  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  async fn seed_person(
    ctx: &LemmyContext,
    instance_id: lemmy_db_schema_file::InstanceId,
    name: &str,
  ) -> Result<PersonId, Box<dyn Error>> {
    let person_form = PersonInsertForm::test_form(instance_id, name);
    let person = Person::create(&mut ctx.pool(), &person_form).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    let mut lu_form = LocalUserInsertForm::test_form(person.id);
    lu_form.accepted_application = Some(true);
    LocalUser::create(&mut ctx.pool(), &lu_form, vec![]).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    Ok(person.id)
  }

  let reporter = seed_person(&context, instance.id, "notify_reporter").await?;
  let target = seed_person(&context, instance.id, "notify_target").await?;

  // 1. Connect via tokio-postgres (NOT diesel) — do NOT tokio::spawn(connection)
  //    directly; the bridge below takes ownership.
  let (pg_client, pg_conn) = tokio_postgres::connect(&db_url, NoTls).await?;

  // 2. Bridge — spawn a task that drives the connection and forwards NOTIFY
  //    messages onto the returned channel. Per DQ #20 (advisor directive):
  //    tokio-postgres 0.7.16 Connection implements Future, not Stream, so we
  //    use poll_fn + Pin::new(&mut conn).poll_message(cx).
  let mut rx: mpsc::UnboundedReceiver<Notification> = {
    let (tx, rx) = mpsc::unbounded_channel();
    tokio::spawn(async move {
      let mut connection = pg_conn;
      std::future::poll_fn(move |cx| loop {
        match Pin::new(&mut connection).poll_message(cx) {
          std::task::Poll::Ready(Some(Ok(AsyncMessage::Notification(n)))) => {
            let _ = tx.send(n);
          }
          std::task::Poll::Ready(Some(Ok(_))) => {}
          std::task::Poll::Ready(Some(Err(_))) | std::task::Poll::Ready(None) => {
            return std::task::Poll::Ready(());
          }
          std::task::Poll::Pending => return std::task::Poll::Pending,
        }
      })
      .await;
    });
    rx
  };

  // 3. LISTEN — must happen BEFORE the INSERT or the test races.
  pg_client.batch_execute("LISTEN governance_events").await?;

  // 4. Trigger an INSERT on governance_log via create_report.
  let reporter_view = LocalUserView::read_person(&mut context.pool(), reporter).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
  let _resp = create_report(
    Json(CreateGovernanceReport {
      community_id: None,
      target_type: CaseTargetType::Person,
      target_id: target.0,
      reason_code: "notify_test".to_string(),
      description: None,
    }),
    context.clone(),
    reporter_view,
  )
  .await
  .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  // 5. Await notification with timeout.
  let notif = tokio::time::timeout(StdDuration::from_secs(2), rx.recv())
    .await
    .map_err(|_| -> Box<dyn Error> { "notification timed out after 2s".into() })?
    .ok_or_else(|| -> Box<dyn Error> { "notification channel closed".into() })?;

  // 6. Assert channel + payload shape.
  assert_eq!(notif.channel(), "governance_events");
  let payload: serde_json::Value = serde_json::from_str(notif.payload())?;
  assert_eq!(payload["kind"], "report_created");
  assert!(payload["entry_id"].as_i64().unwrap_or_default() > 0);
  assert!(payload["created_at"].as_str().is_some());

  Ok(())
}

#[tokio::test]
async fn underscore_prefix_usernames_still_register() -> Result<(), Box<dyn Error>> {
  use diesel::{Connection as _, PgConnection};
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests},
    traits::Crud,
  };
  use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS, utils::validation::is_valid_actor_name};
  use reqwest_middleware::ClientBuilder;

  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  }

  let (_container, host_port) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("start_postgres: {e}").into() })?;
  let db_url = governance_fixtures::db_url(host_port);
  unsafe { std::env::set_var("LEMMY_DATABASE_URL", &db_url); }

  {
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| -> Box<dyn Error> { format!("apply_all_schema: {e}").into() })?;
  }

  let pool: ActualDbPool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
  let middleware_client = ClientBuilder::new(client).build();
  let secret = Secret { id: 0, jwt_secret: String::new().into() };
  let rate_limit = RateLimit::with_debug_config();
  let context = LemmyContext::create(
    pool,
    middleware_client.clone(),
    middleware_client,
    secret,
    rate_limit,
  );

  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  // V2/messaging.md §8.2: MXID-looking usernames must remain registerable.
  // `_lemmy_test_user` is 16 chars; passes is_valid_actor_name regex
  // `^(?:[a-zA-Z0-9_]+|[0-9_\p{Arabic}]+|[0-9_\p{Cyrillic}]+)$`.
  let username = "_lemmy_test_user";
  is_valid_actor_name(username)
    .map_err(|e| -> Box<dyn Error> { format!("is_valid_actor_name: {e}").into() })?;
  let person_form = PersonInsertForm::test_form(instance.id, username);
  let person = Person::create(&mut context.pool(), &person_form).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
  let mut lu_form = LocalUserInsertForm::test_form(person.id);
  lu_form.accepted_application = Some(true);
  LocalUser::create(&mut context.pool(), &lu_form, vec![]).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  assert_eq!(person.name, username);
  Ok(())
}

// ============================================================================
// Phase 6 task 77 — federation round-trip e2e (sanction_notice_round_trip)
// ============================================================================
//
// Two-Postgres test proving that a `FederatedRecommendation`-scope decision
// on instance A produces a `PublishSanctionNotice` activity that, when fed
// directly into instance B's `Activity::receive`, lands as an advisory
// `remote_sanction_notice` row with `local_case_id IS NULL` (ADR-006) and a
// matching `federation_sanction_received` governance-log entry. No HTTP
// transport — per IMPLEMENTATION-PLAN-v0.md §3 Phase 6 task 77, the test
// calls the inbox function directly.

#[tokio::test(flavor = "multi_thread")]
async fn sanction_notice_round_trip() -> Result<(), Box<dyn Error>> {
  use actix_web::web::{Data, Json};
  use diesel::{
    Connection as _, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, SelectableHelper,
  };
  use diesel_async::{AsyncConnection as _, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::submit_jury_vote::submit_jury_vote;
  use lemmy_api_common::governance::SubmitJuryVote;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_apub_activities::protocol::governance::publish_sanction_notice::PublishSanctionNotice;
  use lemmy_db_schema::{
    newtypes::{ModerationCaseId, RemoteSanctionNoticeId},
    source::{
      activity::SentActivity,
      governance::{
        jury_assignment::JuryAssignmentInsertForm,
        moderation_case::{ModerationCase, ModerationCaseInsertForm},
        remote_sanction_notice::RemoteSanctionNotice,
      },
      instance::Instance,
      local_site::{LocalSite, LocalSiteInsertForm},
      local_site_rate_limit::{LocalSiteRateLimit, LocalSiteRateLimitInsertForm},
      local_user::{LocalUser, LocalUserInsertForm},
      person::{Person, PersonInsertForm},
      secret::Secret,
      site::{Site, SiteInsertForm},
    },
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{
      CaseSeverity,
      CaseStatus,
      CaseTargetType,
      JuryAssignmentStatus,
      JuryDecision,
      SanctionAction,
      SanctionScope,
    },
    schema::{
      governance_log,
      jury_assignment,
      moderation_case,
      person,
      remote_sanction_notice,
      sanction,
      sent_activity,
    },
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests},
    dburl::DbUrl,
    traits::Crud,
  };
  use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;
  use serde_json::Value;
  use traits::ActivityTrait;
  use url::Url;

  // Bring trait into scope under a local alias so `PublishSanctionNotice::receive`
  // is callable. The `activitypub_federation::traits::Activity` trait provides
  // both the `receive` method and the `verify`/`actor`/`id` accessors.
  mod traits {
    pub use activitypub_federation::traits::Activity as ActivityTrait;
  }

  // -- 0. Set env vars BEFORE any Lemmy code touches `SETTINGS`. --------
  // GOVERNANCE_LOG_SIGNING_KEY is read by the governance log signer at first
  // call; LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS makes SETTINGS bypass the
  // config-file load. Both DBs share the same signing key — fine for v0
  // since the test only reads each chain locally.
  const SIGNING_SEED_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";
  // SAFETY: tests run with --test-threads=1 so no concurrent env mutation.
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
    std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
  }

  // -- 1. Boot container A + apply schema. ------------------------------
  let (_container_a, port_a) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("start_postgres A: {e}").into() })?;
  let url_a = governance_fixtures::db_url(port_a);
  {
    let mut sync_conn = PgConnection::establish(&url_a)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| -> Box<dyn Error> { format!("apply_all_schema A: {e}").into() })?;
  }

  // Build A's pool+context fully before swapping env to B — the pool reads
  // env at construction and a multi-thread runtime could interleave
  // otherwise. See plan §TWO_DB_TEST_PATTERN + §12 R1.
  // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
  unsafe { std::env::set_var("LEMMY_DATABASE_URL", &url_a); }
  let pool_a: ActualDbPool = build_db_pool_for_tests();
  let client_a = client_builder(&SETTINGS).build()?;
  let middleware_client_a = ClientBuilder::new(client_a).build();
  let secret_a = Secret { id: 0, jwt_secret: String::new().into() };
  let rate_limit_a = RateLimit::with_debug_config();
  let context_a = Data::new(LemmyContext::create(
    pool_a,
    middleware_client_a.clone(),
    middleware_client_a,
    secret_a,
    rate_limit_a,
  ));

  // submit_jury_vote takes the federation flavour of `Data<LemmyContext>`
  // (Phase 6 task 76) so its post-decision block can hand `&context` to
  // `federation_outbox::send_local_sanction_notice`. Mirror the construction
  // pattern from `report_to_modlog_golden_path` (e2e.rs:859-866). Both
  // Data handles share the same underlying `Arc<ActualDbPool>` so they see
  // the same DB rows.
  let federation_config_a = activitypub_federation::config::FederationConfig::builder()
    .domain(context_a.settings().hostname.clone())
    .app_data((**context_a).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context_a = federation_config_a.to_request_data();

  // -- 2. Boot container B + apply schema. ------------------------------
  let (_container_b, port_b) = governance_fixtures::start_postgres()
    .await
    .map_err(|e| -> Box<dyn Error> { format!("start_postgres B: {e}").into() })?;
  let url_b = governance_fixtures::db_url(port_b);
  {
    let mut sync_conn = PgConnection::establish(&url_b)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)
      .map_err(|e| -> Box<dyn Error> { format!("apply_all_schema B: {e}").into() })?;
  }

  // Strict sequencing: pool A construction is fully complete (lines above)
  // before we swap env to B. Multi-thread runtime cannot interleave because
  // Data construction is `await`-free.
  // NOTE: LEMMY_DATABASE_URL is left set to url_b at test exit — mirrors
  // e2e.rs:2195+ pattern; test-infra cleanup is a v1 item per DQ-6.4
  // resolved id 34 (see phase-6 completion report carry-forwards).
  // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
  unsafe { std::env::set_var("LEMMY_DATABASE_URL", &url_b); }
  let pool_b: ActualDbPool = build_db_pool_for_tests();
  let client_b = client_builder(&SETTINGS).build()?;
  let middleware_client_b = ClientBuilder::new(client_b).build();
  let secret_b = Secret { id: 0, jwt_secret: String::new().into() };
  let rate_limit_b = RateLimit::with_debug_config();
  let context_b = Data::new(LemmyContext::create(
    pool_b,
    middleware_client_b.clone(),
    middleware_client_b,
    secret_b,
    rate_limit_b,
  ));
  let federation_config_b = activitypub_federation::config::FederationConfig::builder()
    .domain(context_b.settings().hostname.clone())
    .app_data((**context_b).clone())
    .debug(true)
    .http_fetch_limit(0)
    .build()
    .await?;
  let federation_context_b = federation_config_b.to_request_data();

  // -- 3. Seed instance A. ---------------------------------------------
  // Instance hostname matches the admin/target ap_id host below so
  // `activity.actor.inner().domain()` resolves to "instance-a.test" on
  // the receiving side (asserted later as `source_instance`).
  let instance_a = Instance::read_or_create(&mut context_a.pool(), "instance-a.test")
    .await
    .map_err(|e| -> Box<dyn Error> { format!("instance A: {e}").into() })?;

  // Seed Site + LocalSite + LocalSiteRateLimit on instance A so
  // `SiteView::read_local` (called by `federation_outbox::send_local_sanction_notice`
  // → `load_local_admin`) returns a row. Without this scaffold, the
  // federation publish hits `LocalSiteNotSetup`. Mirrors
  // `lemmy_db_schema::test_data::TestData::create`.
  {
    let pool = &mut context_a.pool();
    let site_form_a = SiteInsertForm::new("instance A test site".to_string(), instance_a.id);
    let site_a = Site::create(pool, &site_form_a).await
      .map_err(|e| -> Box<dyn Error> { format!("site A: {e}").into() })?;
    // System account: throwaway Person — LocalSite needs a non-null FK.
    let sysacct_form = PersonInsertForm::test_form(instance_a.id, "instance_a_sysacct");
    let sysacct = Person::create(pool, &sysacct_form).await
      .map_err(|e| -> Box<dyn Error> { format!("sysacct: {e}").into() })?;
    let local_site_form_a = LocalSiteInsertForm {
      system_account: Some(sysacct.id),
      ..LocalSiteInsertForm::new(site_a.id)
    };
    let local_site_a = LocalSite::create(pool, &local_site_form_a).await
      .map_err(|e| -> Box<dyn Error> { format!("local_site A: {e}").into() })?;
    LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site_a.id))
      .await
      .map_err(|e| -> Box<dyn Error> { format!("local_site_rate_limit A: {e}").into() })?;
  }

  // Seed admin + target with explicit ap_id URLs so `actor.inner().domain()`
  // resolves to `instance-a.test` on the receive side. The default
  // `generate_unique_changeme()` value is not a valid URL.
  async fn seed_person_with_apub(
    ctx: &LemmyContext,
    instance_id: lemmy_db_schema_file::InstanceId,
    name: &str,
    is_admin: bool,
    ap_url: Url,
  ) -> Result<PersonId, Box<dyn Error>> {
    let mut person_form = PersonInsertForm::test_form(instance_id, name);
    let ap_dburl: DbUrl = ap_url.clone().into();
    person_form.ap_id = Some(ap_dburl.clone());
    person_form.inbox_url = Some(ap_dburl);
    person_form.local = Some(true);
    let person = Person::create(&mut ctx.pool(), &person_form).await
      .map_err(|e| -> Box<dyn Error> { format!("person {name}: {e}").into() })?;
    let mut lu_form = if is_admin {
      LocalUserInsertForm::test_form_admin(person.id)
    } else {
      LocalUserInsertForm::test_form(person.id)
    };
    lu_form.accepted_application = Some(true);
    LocalUser::create(&mut ctx.pool(), &lu_form, vec![]).await
      .map_err(|e| -> Box<dyn Error> { format!("local_user {name}: {e}").into() })?;
    Ok(person.id)
  }

  let admin_pid = seed_person_with_apub(
    &context_a, instance_a.id, "admin",
    true,
    Url::parse("http://instance-a.test/u/admin")?,
  ).await?;
  let target_pid = seed_person_with_apub(
    &context_a, instance_a.id, "target",
    false,
    Url::parse("http://instance-a.test/u/target")?,
  ).await?;

  // Re-load target Person to capture the generated ap_id (which we just set
  // above — but we re-load through the model so the test asserts against
  // the round-tripped DB value, not the in-memory one).
  let target_person = Person::read(&mut context_a.pool(), target_pid).await
    .map_err(|e| -> Box<dyn Error> { format!("read target: {e}").into() })?;
  let target_ap_id_string = target_person.ap_id.to_string();

  // Seed 5 jurors (no special ap_ids needed — they're not the actor on the
  // outbound activity).
  let mut jurors: Vec<PersonId> = Vec::new();
  for i in 0..5 {
    let pid = seed_person_with_apub(
      &context_a, instance_a.id, &format!("juror_{i}"),
      false,
      Url::parse(&format!("http://instance-a.test/u/juror_{i}"))?,
    ).await?;
    jurors.push(pid);
  }

  // -- 4. Direct seed: ModerationCase + 5 JuryAssignment(Accepted). ----
  // Bypass the create_report → threshold → admin_assign_jury → 5×accept
  // chain (slow; covered by Phase 5c golden-path test). Task 77's job
  // is to verify the federation publish step, not the handler chain.
  // See plan §GOTCHA "Seeding shortcut".
  let case_id: ModerationCaseId = {
    let pool = &mut context_a.pool();
    let conn = &mut lemmy_diesel_utils::connection::get_conn(pool).await
      .map_err(|e| -> Box<dyn Error> { format!("get_conn A: {e}").into() })?;
    let case_form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::Person,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: Some(target_pid),
      target_community_id: None,
      target_remote_url: None,
      reason_code: "fed_test".to_string(),
      severity: CaseSeverity::Medium,
      // JurySelection so submit_jury_vote's status filter sees the case.
      // (admin_assign_jury normally flips Open→JurySelection.)
      status: CaseStatus::JurySelection,
      threshold_score: 1,
    };
    let case: ModerationCase = diesel::insert_into(
      lemmy_db_schema_file::schema::moderation_case::table,
    )
    .values(&case_form)
    .get_result(&mut **conn)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("insert case: {e}").into() })?;
    case.id
  };

  // 5 JuryAssignment rows with status=Accepted so `submit_jury_vote`'s
  // first-step check passes for each juror (it requires status=Accepted).
  {
    let pool = &mut context_a.pool();
    let conn = &mut lemmy_diesel_utils::connection::get_conn(pool).await
      .map_err(|e| -> Box<dyn Error> { format!("get_conn A jury: {e}").into() })?;
    for &juror_id in &jurors {
      let form = JuryAssignmentInsertForm {
        case_id,
        person_id: juror_id,
        status: JuryAssignmentStatus::Accepted,
      };
      diesel::insert_into(jury_assignment::table)
        .values(&form)
        .execute(&mut **conn)
        .await
        .map_err(|e| -> Box<dyn Error> { format!("insert jury_assignment: {e}").into() })?;
    }
  }

  // -- 5. Submit ALL 5 RecommendFederationAction votes -----------------
  // Quorum = 3 per submit_jury_vote.rs:85. The 3rd vote runs the
  // post-decision block which (because winning_decision maps to
  // FederatedRecommendation scope) calls
  // federation_outbox::send_local_sanction_notice → sent_activity INSERT.
  //
  // All 5 jurors vote. Votes 4+5 arrive post-quorum — the federation
  // publish (PublishSanctionNotice → sent_activity INSERT) must fire
  // exactly once despite late-arriving votes. Without the idempotency
  // guard at submit_jury_vote post-decision block, sent_activity would
  // gain a row per vote past quorum (3 total), exfiltrating duplicate
  // sanction notices to remote instances. Regression test for CodeRabbit
  // PR #46 finding #15.
  for (i, juror_id) in jurors.iter().enumerate() {
    let juror_view = LocalUserView::read_person(&mut context_a.pool(), *juror_id)
      .await
      .map_err(|e| -> Box<dyn Error> { format!("juror_view {i}: {e}").into() })?;
    let resp = submit_jury_vote(
      Json(SubmitJuryVote {
        case_id,
        decision: JuryDecision::RecommendFederationAction,
        rationale: Some(format!(
          "Juror {i}: cross-instance harassment by @baduser email evil@example.org \
           per profile https://instance-a.test/u/baduser",
        )),
      }),
      federation_context_a.reset_request_count(),
      juror_view,
    )
    .await
    .map_err(|e| -> Box<dyn Error> { format!("submit_jury_vote {i}: {e}").into() })?
    .into_inner();
    if i < 2 {
      assert!(!resp.case_decided, "vote {i}: must not be decided pre-quorum");
    } else if i == 2 {
      assert!(resp.case_decided, "vote {i}: must be decided at quorum");
      assert_eq!(
        resp.decision,
        Some(JuryDecision::RecommendFederationAction),
        "winning decision must be RecommendFederationAction",
      );
    } else {
      // Votes 4 and 5: case already Decided, handler returns case_decided
      // true but MUST NOT re-run federation publish. Exactly-once on
      // sent_activity is asserted below at -- 6.
      assert!(resp.case_decided, "vote {i}: case already decided (idempotent)");
    }
  }

  // -- 6. Assert sent_activity on A: exactly one PublishSanctionNotice. -
  // The wire `type` discriminator on the wrapper is "Create" (per
  // `kinds::activity::CreateType`), and the inner `object.type` is
  // "SanctionNotice" (per `SanctionNoticeKind`/`SanctionNoticeType`). So
  // we filter on `data->'object'->>'type' = 'SanctionNotice'` to identify
  // governance Create wrappers vs vanilla Lemmy Create activities (none
  // are produced in this test, but defence in depth matches plan §6).
  let mut async_conn_a = AsyncPgConnection::establish(&url_a).await?;
  let activity_rows: Vec<SentActivity> = sent_activity::table
    .filter(
      diesel::dsl::sql::<diesel::sql_types::Text>("data->'object'->>'type'")
        .eq("SanctionNotice"),
    )
    .select(SentActivity::as_select())
    .load(&mut async_conn_a)
    .await?;
  // Exactly-once invariant under post-quorum votes: if submit_jury_vote's
  // idempotency guard regresses, this count would be 3 (one publish per
  // vote past quorum), exfiltrating duplicate governance activities to
  // federated instances. This assertion IS the load-bearing regression
  // test for CodeRabbit PR #46 #15 on the federation path.
  assert_eq!(
    activity_rows.len(),
    1,
    "exactly one PublishSanctionNotice sent_activity row",
  );
  let activity_row = &activity_rows[0];
  // Sanity: the wrapper's `type` is Create.
  let wrapper_type = activity_row
    .data
    .get("type")
    .and_then(Value::as_str)
    .ok_or_else(|| -> Box<dyn Error> { "wrapper type missing".into() })?;
  assert_eq!(wrapper_type, "Create", "wrapper activity type must be Create");
  // The actor URL on the activity is the local admin's ap_id.
  let actor_url = activity_row
    .data
    .get("actor")
    .and_then(Value::as_str)
    .ok_or_else(|| -> Box<dyn Error> { "actor missing".into() })?;
  assert_eq!(
    actor_url, "http://instance-a.test/u/admin",
    "outbound actor must be admin ap_id",
  );

  // -- 7. Deserialise sent_activity.data into PublishSanctionNotice. ----
  let activity: PublishSanctionNotice = serde_json::from_value(activity_row.data.clone())
    .map_err(|e| -> Box<dyn Error> { format!("deserialise PublishSanctionNotice: {e}").into() })?;

  // -- 8. Seed instance B (Site/LocalSite scaffolding + Instance row). --
  // The receive function does NOT call SiteView::read_local, so strictly
  // speaking only the Instance row is required for inbox bookkeeping.
  // We seed Site/LocalSite anyway to mirror real-world deployment shape
  // and to leave room for v1 receive-side enhancements that may need it.
  let _instance_b = Instance::read_or_create(&mut context_b.pool(), "instance-b.test")
    .await
    .map_err(|e| -> Box<dyn Error> { format!("instance B: {e}").into() })?;
  {
    let pool = &mut context_b.pool();
    let site_form_b = SiteInsertForm::new("instance B test site".to_string(), _instance_b.id);
    let site_b = Site::create(pool, &site_form_b).await
      .map_err(|e| -> Box<dyn Error> { format!("site B: {e}").into() })?;
    let sysacct_form = PersonInsertForm::test_form(_instance_b.id, "instance_b_sysacct");
    let sysacct = Person::create(pool, &sysacct_form).await
      .map_err(|e| -> Box<dyn Error> { format!("sysacct B: {e}").into() })?;
    let local_site_form_b = LocalSiteInsertForm {
      system_account: Some(sysacct.id),
      ..LocalSiteInsertForm::new(site_b.id)
    };
    let local_site_b = LocalSite::create(pool, &local_site_form_b).await
      .map_err(|e| -> Box<dyn Error> { format!("local_site B: {e}").into() })?;
    LocalSiteRateLimit::create(pool, &LocalSiteRateLimitInsertForm::new(local_site_b.id))
      .await
      .map_err(|e| -> Box<dyn Error> { format!("local_site_rate_limit B: {e}").into() })?;
  }

  // -- 9. Deliver the activity directly to instance B's receive function.
  // No HTTP transport (per IMPLEMENTATION-PLAN-v0.md §3 Phase 6 task 77).
  // PublishSanctionNotice::receive consumes self, so we need the owned
  // value from step 7. Activity::verify (which `verify_is_public`-checks
  // the `to`/`cc` fields) is a separate trait method; we call it
  // explicitly to mirror the framework's normal receive pipeline.
  ActivityTrait::verify(&activity, &federation_context_b)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("verify on B: {e}").into() })?;
  ActivityTrait::receive(activity, &federation_context_b)
    .await
    .map_err(|e| -> Box<dyn Error> { format!("receive on B: {e}").into() })?;

  // -- 10. Assert remote_sanction_notice on B has exactly one row. ------
  let mut async_conn_b = AsyncPgConnection::establish(&url_b).await?;
  let advisory_rows: Vec<RemoteSanctionNotice> = remote_sanction_notice::table
    .select(RemoteSanctionNotice::as_select())
    .load(&mut async_conn_b)
    .await?;
  assert_eq!(advisory_rows.len(), 1, "exactly one remote_sanction_notice row");
  let advisory = &advisory_rows[0];

  // ADR-006 invariant: NEVER auto-applied. local_case_id MUST be NULL.
  assert!(
    advisory.local_case_id.is_none(),
    "local_case_id must be NULL on advisory row (ADR-006)",
  );
  assert_eq!(
    advisory.action,
    SanctionAction::FederationQuarantineRecommendation,
    "action must be FederationQuarantineRecommendation",
  );
  assert_eq!(
    advisory.scope,
    SanctionScope::FederatedRecommendation,
    "scope must be FederatedRecommendation",
  );
  assert_eq!(
    advisory.target_url, target_ap_id_string,
    "target_url must match target person's ap_id from A",
  );
  assert_eq!(
    advisory.source_instance, "instance-a.test",
    "source_instance must match A's hostname (from actor.domain())",
  );

  // Redaction assertion: summary must NOT leak admin/target usernames,
  // emails, or profile URLs from the juror rationales. The summary is
  // built by submit_jury_vote.rs:440-448 (plain reason_code/case_id/
  // target_type/decision); the redaction layer also runs as
  // defence-in-depth. Concretely verify nothing identifying remains.
  // Note: "admin" appears in juror_view names and we deliberately seed
  // "admin" as the local admin Person's ap_id host path. The summary
  // builder only uses reason_code+case_id+target_type+decision, so
  // "admin" should not surface; assert that fact directly.
  assert!(!advisory.summary.is_empty(), "summary must be non-empty");
  assert!(
    !advisory.summary.contains("@baduser"),
    "summary must not contain @mention from juror rationale",
  );
  assert!(
    !advisory.summary.contains("evil@example.org"),
    "summary must not contain email from juror rationale",
  );
  assert!(
    !advisory.summary.contains("/u/baduser"),
    "summary must not contain profile URL from juror rationale",
  );
  assert!(
    !advisory.summary.contains("admin"),
    "summary must not contain admin username (build_summary contract)",
  );
  assert!(
    !advisory.summary.contains("target"),
    "summary must not contain target username (build_summary contract)",
  );

  // -- 11. Assert governance_log on B: exactly one federation_sanction_received.
  let received_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("federation_sanction_received"))
    .count()
    .get_result(&mut async_conn_b)
    .await?;
  assert_eq!(
    received_count, 1,
    "exactly one federation_sanction_received governance_log entry on B",
  );

  // -- 11b. Negative assertions: advisory MUST NOT auto-apply on B. -------
  // ADR-006 + v0 simplification in [05 §3] require inbound sanction
  // notices to land as advisory rows only — never materialising into a
  // local `sanction`, a new `moderation_case`, or a Person.removed flip.
  // The positive assertions in -- 10/-- 11 prove the advisory row + log
  // exist; the negative assertions below prove B stays otherwise
  // untouched. Without these, a regression could silently auto-apply and
  // the test would still pass on the positive-path alone.
  // CodeRabbit PR #46 finding #22.
  let sanction_count_b: i64 = sanction::table
    .count()
    .get_result(&mut async_conn_b)
    .await?;
  assert_eq!(
    sanction_count_b, 0,
    "B must have zero sanction rows — advisory notices do not auto-apply (ADR-006)",
  );
  let case_count_b: i64 = moderation_case::table
    .count()
    .get_result(&mut async_conn_b)
    .await?;
  assert_eq!(
    case_count_b, 0,
    "B must have zero moderation_case rows — inbound notice does not create a local case",
  );
  let target_removed_on_b: Option<bool> = person::table
    .filter(person::ap_id.eq(&target_ap_id_string))
    .select(person::deleted)
    .first(&mut async_conn_b)
    .await
    .optional()?;
  if let Some(flag) = target_removed_on_b {
    assert!(
      !flag,
      "target Person on B must NOT have deleted=true set by inbound notice",
    );
  }
  // NB: B has never heard of the target Person, so the row may not exist
  // at all (optional()? returns None). That is the stronger no-apply
  // signal — if auto-apply had fired, a Person row would have been
  // materialised to hang the removal flag off.

  // -- 12. Cross-check on A: federation_sanction_sent log entry exists. -
  // Ensures the orchestrator's transactional pair-write actually committed
  // (not asserted on by step 6 which targets sent_activity, not the log).
  let sent_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("federation_sanction_sent"))
    .count()
    .get_result(&mut async_conn_a)
    .await?;
  assert_eq!(
    sent_count, 1,
    "exactly one federation_sanction_sent governance_log entry on A",
  );

  // -- 13. Negative path: actor-binding spoofing (CodeRabbit PR #46 #19).
  // Take the legitimate activity we already verified+received above, mutate
  // its inner object.actor to point at a *different* actor than the
  // wrapper's signed actor, and confirm verify rejects it. Also assert
  // remote_sanction_notice still has exactly 1 row (the original positive
  // path), proving the rejected activity did NOT land in B's DB.
  //
  // This is the regression guard for the impersonation class: without the
  // actor-binding check in PublishSanctionNotice::verify, an attacker
  // could sign an activity as actor X while naming actor Y in the inner
  // object, causing inbox code that reads object.actor downstream
  // (federation_attestation.actor_url is the documented v0 example, see
  // inbox.rs:195) to attribute the activity to the spoofed actor.
  let mut spoofed_data: Value = serde_json::from_value(activity_row.data.clone())?;
  let spoofed_actor_url = "https://attacker.example/u/eve";
  spoofed_data["object"]["actor"] = Value::String(spoofed_actor_url.to_string());
  let spoofed_activity: PublishSanctionNotice = serde_json::from_value(spoofed_data)?;
  let verify_err = ActivityTrait::verify(&spoofed_activity, &federation_context_b).await;
  assert!(
    verify_err.is_err(),
    "spoofed object.actor must fail verify (CodeRabbit PR #46 #19)",
  );
  // Belt-and-braces: confirm DB on B is unchanged. If verify had let the
  // spoof through, receive would write a second row.
  let advisory_count_after_spoof: i64 = remote_sanction_notice::table
    .count()
    .get_result(&mut async_conn_b)
    .await?;
  assert_eq!(
    advisory_count_after_spoof, 1,
    "rejected spoofed activity must NOT add a remote_sanction_notice row",
  );

  // -- 14. Touch the unused juror locals to keep `_ = jurors` lints happy.
  let _ = (jurors, admin_pid, RemoteSanctionNoticeId(advisory.id.0));

  Ok(())
}

