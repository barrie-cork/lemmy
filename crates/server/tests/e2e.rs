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
  ///   - 2 Phase 5a migrations (add_governance_config + add_person_membership_state
  ///     — total 8)
  const PHASE_1_MIGRATION_COUNT: u64 = 8;

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
    admin_assign_jury::admin_assign_jury,
    list_modlog::list_modlog,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
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
    assert_eq!(threshold_score, 1, "threshold_score after one report = 1");

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

  // -- 11. Steps 3–5: 3 jurors vote AdvisoryLabel ------------------
  let voting_jurors: Vec<PersonId> = assign_resp
    .assigned_person_ids
    .iter()
    .copied()
    .take(3)
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
      context.clone(),
      juror_view,
    )
    .await?
    .into_inner();
    decided_responses.push(resp);
  }
  assert!(!decided_responses[0].case_decided, "1st vote: not decided");
  assert!(!decided_responses[1].case_decided, "2nd vote: not decided");
  assert!(decided_responses[2].case_decided, "3rd vote: decided");
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
    assert_eq!(vote_count, 3, "3 jury_vote rows");

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
    let rep_total: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_id))
      .count()
      .get_result(conn)
      .await?;
    assert_eq!(rep_total, 4, "4 reputation_event rows total");

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
    assert_eq!(map.get("jury_vote_submitted"), Some(&3));
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
