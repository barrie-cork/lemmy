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

  /// Count of Phase 1 migrations that this branch adds. Tasks 2–7 each
  /// create one migration. If Phase 1 ever adds or drops a migration this
  /// constant has to move with it.
  const PHASE_1_MIGRATION_COUNT: u64 = 6;

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
