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
  //! process-wide env vars (e.g. `LEMMY_DATABASE_URL`, `BREHON_DISABLE_*`).
  //! Safety of those mutations is contingent on the Cargo runner flag
  //! `--test-threads=1`. Running these tests with concurrent threads is
  //! undefined behaviour and is forbidden — see the `EnvVarGuard` RAII guard.
  use super::EnvVarGuard;
  use actix_web::web::Data;
  use diesel::{Connection as _, PgConnection, RunQueryDsl, connection::SimpleConnection};
  use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
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
  ///
  /// **Tier 3 fast path:** when `start_postgres` has already restored the
  /// template dump into this container, the schema is fully populated and
  /// steps 1-3 would re-do work the dump already captured. The sentinel
  /// short-circuits to `Ok(())` so existing call sites that pair
  /// `start_postgres()` + `apply_all_schema(&mut conn)` get the speedup
  /// transparently.
  ///
  /// **Sentinel correctness (CR finding #6):** `governance_log` (in `public`)
  /// is introduced by a mid-set migration, so a partial-init DB could have
  /// `governance_log` but be missing later migrations or the `r` schema —
  /// short-circuiting on `governance_log` alone would mask that broken
  /// state. The fix is to require BOTH `public.governance_log` (proves
  /// all migrations ran through that point) AND `r.parent_comment_ids`
  /// (proves the last replaceable-schema rebuild completed — `triggers.sql`
  /// is the last include in `apply_all_schema_legacy`). If either is
  /// missing, fall through to the legacy bootstrap rather than risking
  /// silent partial-init.
  ///
  /// To force the legacy path (cold migrations) for debugging, set
  /// `BREHON_E2E_NO_TEMPLATE=1` before the test run — `start_postgres`
  /// then skips the restore and the sentinel here misses.
  /// Two-signal schema sentinel (CR finding #6 + #16). Returns true
  /// only when BOTH `public.governance_log` (proves all migrations
  /// ran through that point) AND `r.parent_comment_ids` (proves the
  /// LAST step of `apply_all_schema_legacy` — `triggers.sql` —
  /// completed) are present. Either alone indicates a partial-init
  /// DB that must be re-bootstrapped, not skipped.
  ///
  /// Single source of truth — used by `apply_all_schema` for the
  /// fast-path short-circuit AND by `postgres_container_boots` for
  /// the smoke-test assertion. Extracting the SQL ensures the smoke
  /// test and the runtime gate stay in sync if this check ever
  /// gains a third signal or moves to a different invariant.
  pub fn schema_sentinel_satisfied(conn: &mut PgConnection) -> LemmyResult<bool> {
    use diesel::sql_types::BigInt;
    #[derive(diesel::QueryableByName)]
    struct CountRow {
      #[diesel(sql_type = BigInt)]
      count: i64,
    }
    let row: CountRow = diesel::sql_query(
      "SELECT \
         (SELECT COUNT(*) FROM information_schema.tables \
          WHERE table_schema = 'public' AND table_name = 'governance_log') \
       + (SELECT COUNT(*) FROM information_schema.routines \
          WHERE routine_schema = 'r' AND routine_name = 'parent_comment_ids') \
       AS count",
    )
    .get_result::<CountRow>(conn)?;
    Ok(row.count == 2)
  }

  pub fn apply_all_schema(conn: &mut PgConnection) -> LemmyResult<()> {
    if schema_sentinel_satisfied(conn)? {
      return Ok(());
    }

    // Sentinel missed → schema not fully populated by template
    // restore. Delegate to the legacy bootstrap path so the
    // migration + r-schema rebuild logic lives in exactly one place
    // (CR finding #12 DRY).
    apply_all_schema_legacy(conn).map_err(|e| anyhow::anyhow!("{e}"))?;
    Ok(())
  }

  /// Tier 3 — pg_dump template captured once per nextest process (and
  /// cached on disk across processes), then pg_restore'd into every
  /// fresh container.
  ///
  /// **Throughput (PG18 + this laptop, nextest threads-required=4):**
  /// template path 12m17s vs nextest legacy 12m02s for 67/67 e2e tests
  /// — i.e. measurably **NEUTRAL**, not a 60× speedup as the original
  /// plan projected. pg_restore (binary command replay) takes the same
  /// wall-clock as Diesel `MigrationHarness::run_pending_migrations`
  /// (SQL replay) on this hardware. Tier 3 ships as scaffolding for
  /// future option-3 (per-test schema isolation) work, NOT as a
  /// throughput improvement on this laptop. Likely speedup on slower
  /// environments where Diesel migration runner overhead dominates.
  /// Full discussion: `.claude/lessons/feedback_local_validation_cycle_2026_05_02.md`
  /// Rule 4.
  ///
  /// Both `pg_dump` and `pg_restore` shell out via `docker exec` against
  /// the live container, so no host-side Postgres client toolchain is
  /// required. Container ships pg_dump 18.3 + pg_restore 18.3 (exact
  /// version match for `pgautoupgrade:18-alpine`), eliminating the
  /// version-skew risk.
  pub mod pg_template {
    use std::error::Error;
    use std::path::PathBuf;
    use tokio::process::Command;

    /// In-process LazyLock for the dump bytes. Saves a disk read on
    /// the second-and-subsequent test in the SAME nextest process.
    /// Under nextest's process-per-test isolation this rarely fires
    /// (each test = its own process), but it's cheap insurance and
    /// load-bearing if anyone runs `cargo test` (one-process-many-tests).
    static TEMPLATE_DUMP: tokio::sync::OnceCell<Vec<u8>> = tokio::sync::OnceCell::const_new();

    /// Track whether `ensure_docker_cli()` has run successfully in
    /// this process. Both the cold-bootstrap path AND the cache-hit
    /// `pg_restore_into` path need a working `docker` binary, so the
    /// preflight must run on whichever code path fires first (CR
    /// finding #10). OnceCell ensures we pay the ~30ms `docker
    /// --version` cost at most once per nextest process.
    static DOCKER_CHECKED: tokio::sync::OnceCell<()> = tokio::sync::OnceCell::const_new();

    /// Smallest plausible byte count for a healthy custom-format
    /// pg_dump of the full Brehon schema. Used to reject:
    /// - Truncated/half-written disk cache files in `load_or_build`
    ///   (CR finding #18) — even a file starting with `PGDMP` could
    ///   be corrupt and bypass the rebuild path.
    /// - Implausibly empty bootstrap dumps in `template_dump_capture`
    ///   (CR finding #15) — catches the "container produced no
    ///   schema" failure mode.
    ///
    /// 1 KiB is well below any healthy dump (real dumps are
    /// ~400 KiB+) but well above any truncation that would still
    /// register as "looks like a file".
    pub const MIN_TEMPLATE_DUMP_BYTES: usize = 1_024;

    /// Compute the disk cache path. Lives under `target/tmp/` so
    /// `cargo clean` clears it; suffixed with the test binary's mtime
    /// (nanosecond precision) so any source-or-migration change
    /// invalidates the cache (the `embed_migrations!` macro recompiles
    /// on `migrations/` changes → new test binary → new mtime → cache
    /// miss → rebuild).
    ///
    /// **Cache key precision (CR finding #3):** earlier draft used
    /// `as_secs()` which collides on rebuilds within the same second
    /// — incremental cargo can finish a touch-and-rebuild cycle under
    /// 1s on a hot cache, producing a stale-dump-reuse hazard. Nanos
    /// is overkill but cheap; collisions are now astronomically
    /// unlikely (filesystem mtime resolution is the floor anyway).
    fn cache_path() -> Result<PathBuf, Box<dyn Error>> {
      let exe = std::env::current_exe()?;
      let mtime = std::fs::metadata(&exe)?
        .modified()?
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| -> Box<dyn Error> { format!("clock skew: {e}").into() })?
        .as_nanos();
      // Walk up from the e2e test binary
      // (target/<profile>/deps/e2e-<hash>.exe) to find the workspace
      // target directory: deps/ → debug/ or release/ → target/.
      let target_root = exe
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .ok_or_else(|| -> Box<dyn Error> {
          format!("cannot derive target root from {exe:?}").into()
        })?;
      Ok(
        target_root
          .join("tmp")
          .join(format!("brehon-pg-template-{mtime}.dump")),
      )
    }

    /// Lazily build (or return cached) bootstrap dump bytes. Cache
    /// layers (fastest first):
    ///   1. In-process `OnceCell` — same nextest process, second test.
    ///   2. On-disk `target/tmp/...dump` — cross-process, keyed by test
    ///      binary's mtime so source/migration changes invalidate.
    ///   3. Cold bootstrap — ~30s container start + migrations + dump.
    pub async fn ensure_template() -> Result<&'static Vec<u8>, Box<dyn Error>> {
      TEMPLATE_DUMP.get_or_try_init(load_or_build).await
    }

    /// Pre-flight check that the `docker` CLI is on PATH. Tier 3
    /// shells out via `docker exec` for pg_dump/pg_restore (rather
    /// than using the bollard API like testcontainers-rs does
    /// elsewhere), so the binary is a hard dependency. Surface a
    /// clear actionable error here instead of letting `Command::new`
    /// fail with `program not found` mid-run (CR findings #1, #10).
    ///
    /// OnceCell-gated so the ~30ms `docker --version` runs at most
    /// once per nextest process, regardless of whether the first
    /// caller is the cold-bootstrap path or a cache-hit
    /// `pg_restore_into`.
    pub(super) async fn ensure_docker_cli() -> Result<(), Box<dyn Error>> {
      // Cloning the error path: OnceCell::get_or_try_init caches only
      // success. On failure the next call retries, which is what we
      // want — if the user installs docker after a missed first call,
      // subsequent ensure_docker_cli() calls succeed.
      DOCKER_CHECKED
        .get_or_try_init(|| async {
          let out = Command::new("docker")
            .arg("--version")
            .kill_on_drop(true)
            .output()
            .await
            .map_err(|e| -> Box<dyn Error> {
              format!(
                "Tier 3 needs `docker` on PATH for pg_dump/pg_restore \
                 via `docker exec`, but spawning failed: {e}. \
                 Either install Docker CLI, or set \
                 BREHON_E2E_NO_TEMPLATE=1 to bypass the template path \
                 entirely (legacy migration runner)."
              )
              .into()
            })?;
          if !out.status.success() {
            return Err::<(), Box<dyn Error>>(
              format!(
                "`docker --version` exited {:?}: {}. Tier 3 needs a \
                 working docker CLI (or BREHON_E2E_NO_TEMPLATE=1).",
                out.status.code(),
                String::from_utf8_lossy(&out.stderr)
              )
              .into(),
            );
          }
          Ok(())
        })
        .await
        .map(|_| ())
    }

    async fn load_or_build() -> Result<Vec<u8>, Box<dyn Error>> {
      let path = cache_path()?;
      // Layer 2: disk cache hit. Validate the magic header AND the
      // size before returning (CR findings #14 + #18). A truncated
      // dump can still start with `PGDMP`, so the magic check alone
      // isn't enough to prove integrity. Reuse `MIN_TEMPLATE_DUMP_BYTES`
      // (the same floor `template_dump_capture` uses) to reject
      // partially-written cache files. Either failure → delete +
      // fall through to cold bootstrap (self-healing).
      if let Ok(bytes) = tokio::fs::read(&path).await {
        if bytes.starts_with(b"PGDMP") && bytes.len() >= MIN_TEMPLATE_DUMP_BYTES {
          tracing::info!(
            bytes = bytes.len(),
            path = %path.display(),
            "pg_template: disk cache hit (skipping bootstrap)"
          );
          return Ok(bytes);
        }
        tracing::warn!(
          path = %path.display(),
          bytes = bytes.len(),
          first_bytes = ?bytes.get(..bytes.len().min(8)),
          "pg_template: invalid disk cache (header or size below \
           MIN_TEMPLATE_DUMP_BYTES), rebuilding"
        );
        let _ = tokio::fs::remove_file(&path).await;
      }
      // Layer 3: cold bootstrap (preflight docker first so the error
      // surfaces before we boot a container that we can't dump).
      ensure_docker_cli().await?;
      let dump = build_template().await?;
      // Best-effort write. Atomic via tmp-per-pid + rename. On race:
      // - Linux/macOS rename atomically replaces the destination.
      // - Windows rename FAILS with EEXIST when the destination already
      //   exists (CR finding #4). On either OS, if a peer beat us to
      //   write a fully-formed cache file, that file is correct and we
      //   can simply discard our tmp and use the in-memory dump for
      //   this process. Treating EEXIST/AlreadyExists as success keeps
      //   the warning channel clean on Windows.
      if let Some(parent) = path.parent() {
        let _ = tokio::fs::create_dir_all(parent).await;
      }
      let tmp_path = path.with_extension(format!("dump.tmp.{}", std::process::id()));
      if let Err(e) = tokio::fs::write(&tmp_path, &dump).await {
        tracing::warn!(
          error = %e,
          tmp = %tmp_path.display(),
          "pg_template: tmp write failed (continuing with in-memory dump)"
        );
      } else {
        match tokio::fs::rename(&tmp_path, &path).await {
          Ok(()) => {
            tracing::info!(
              bytes = dump.len(),
              path = %path.display(),
              "pg_template: disk cache populated"
            );
          }
          Err(e) if path.exists() => {
            // Peer already populated the cache. Our tmp is redundant.
            // Covers the Windows EEXIST case AND the rare Linux race
            // where a peer wrote between our `tokio::fs::read` miss
            // above and our rename here.
            let _ = tokio::fs::remove_file(&tmp_path).await;
            tracing::debug!(
              error = %e,
              path = %path.display(),
              "pg_template: peer populated cache between our read and rename — using in-memory dump"
            );
          }
          Err(e) => {
            // Genuine rename failure (permissions, cross-device,
            // disk full). Keep the in-memory dump for this process;
            // peers will pay their own bootstrap.
            tracing::warn!(
              error = %e,
              tmp = %tmp_path.display(),
              path = %path.display(),
              "pg_template: rename to cache path failed (continuing with in-memory dump)"
            );
            let _ = tokio::fs::remove_file(&tmp_path).await;
          }
        }
      }
      Ok(dump)
    }

    async fn build_template() -> Result<Vec<u8>, Box<dyn Error>> {
      use diesel::{Connection as _, PgConnection, RunQueryDsl, sql_types::Text};
      let total = std::time::Instant::now();
      let (container, host_port) = super::start_postgres_vanilla().await?;
      let db_url = super::db_url(host_port);
      let mut conn = PgConnection::establish(&db_url)
        .map_err(|e| -> Box<dyn Error> { format!("template bootstrap establish: {e}").into() })?;
      super::apply_all_schema_legacy(&mut conn)?;

      // Query pg_extension for every non-builtin extension installed
      // by the migrations, so the pg_dump command captures all of
      // them dynamically (CR finding #17). The previous version hard-
      // coded `pgcrypto`, `ltree`, `pg_trgm` — easy to forget when a
      // new migration adds an extension. Excluding `plpgsql` (built-
      // in to every Postgres database since 9.x; pg_dump skips it
      // automatically and including it produces a benign warning).
      #[derive(diesel::QueryableByName)]
      struct ExtRow {
        #[diesel(sql_type = Text)]
        extname: String,
      }
      let ext_rows: Vec<ExtRow> = diesel::sql_query(
        "SELECT extname FROM pg_extension \
         WHERE extname <> 'plpgsql' \
         ORDER BY extname",
      )
      .get_results(&mut conn)
      .map_err(|e| -> Box<dyn Error> { format!("query pg_extension: {e}").into() })?;
      let extensions: Vec<String> = ext_rows.into_iter().map(|r| r.extname).collect();
      tracing::info!(
        extensions = ?extensions,
        "pg_template: extensions discovered for dump"
      );

      let dump = pg_dump(container.id(), &extensions).await?;
      tracing::info!(
        bytes = dump.len(),
        ms = total.elapsed().as_millis(),
        "pg_dump: template captured (bootstrap end-to-end)"
      );
      // container drops here; bootstrap done.
      Ok(dump)
    }

    /// Hard timeout for pg_dump bootstrap. On healthy hardware the
    /// dump completes in under 30s; 120s gives ~4× headroom for slow
    /// laptops / contended Docker daemons. Without this timeout a
    /// stalled docker exec would wedge the entire e2e suite with no
    /// bounded failure (CR finding #7).
    const PG_DUMP_TIMEOUT_SECS: u64 = 120;

    /// Hard timeout for per-test pg_restore. Restore is faster than
    /// pg_dump (no SQL parsing, just binary command replay); 90s is
    /// ~6× the observed worst case.
    const PG_RESTORE_TIMEOUT_SECS: u64 = 90;

    /// Capture a custom-format pg_dump of `public` + `r` schemas + the
    /// supplied extension list via `docker exec`. Custom format is more
    /// compact than plain SQL and pg_restore's `--single-transaction
    /// --exit-on-error` semantics give clean failure on per-test
    /// restore.
    ///
    /// **Extensions:** `-e <name>` flags are required because
    /// pg_restore `--clean --if-exists` drops the default `public`
    /// schema, which cascades the extensions installed there. Without
    /// these, restore fails with `type public.<X> does not exist` when
    /// r.* functions reference extension-provided types (e.g.
    /// `public.ltree`). The list is queried dynamically from
    /// `pg_extension` in `build_template` (CR finding #17), so a
    /// future migration that adds an extension automatically gets
    /// captured without code changes here.
    async fn pg_dump(container_id: &str, extensions: &[String]) -> Result<Vec<u8>, Box<dyn Error>> {
      let start = std::time::Instant::now();
      // Build the args list dynamically: fixed prefix + per-extension
      // -e flags + format=custom suffix.
      let mut args: Vec<String> = vec![
        "exec".into(),
        "-u".into(),
        "postgres".into(),
        container_id.into(),
        "pg_dump".into(),
        "-U".into(),
        "lemmy".into(),
        "-d".into(),
        "lemmy".into(),
        "--no-owner".into(),
        "--no-privileges".into(),
        "--schema=public".into(),
        "--schema=r".into(),
      ];
      for ext in extensions {
        args.push("-e".into());
        args.push(ext.clone());
      }
      args.push("--format=custom".into());
      let dump_future = Command::new("docker")
        .args(&args)
        .kill_on_drop(true)
        .output();
      let output = tokio::time::timeout(
        std::time::Duration::from_secs(PG_DUMP_TIMEOUT_SECS),
        dump_future,
      )
      .await
      .map_err(|_e| -> Box<dyn Error> {
        format!(
          "pg_dump timed out after {PG_DUMP_TIMEOUT_SECS}s — \
           docker daemon may be stalled. Check `docker ps` and consider \
           BREHON_E2E_NO_TEMPLATE=1 as a workaround."
        )
        .into()
      })??;
      if !output.status.success() {
        return Err(
          format!(
            "pg_dump failed (exit {:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
          )
          .into(),
        );
      }
      tracing::info!(
        bytes = output.stdout.len(),
        ms = start.elapsed().as_millis(),
        "pg_dump: bootstrap dump captured"
      );
      Ok(output.stdout)
    }

    /// Restore the template dump into a fresh container via `docker
    /// exec -i pg_restore` (dump bytes streamed on stdin). The fresh
    /// container has POSTGRES_DB=lemmy already created (by entrypoint);
    /// pg_restore loads schema into that empty DB.
    ///
    /// `--clean --if-exists` is paired with the dump's `-e` flags
    /// (extensions captured) so DROP+CREATE for `public` + extensions
    /// + tables + r.* objects all replay cleanly in one transaction.
    pub async fn pg_restore_into(container_id: &str, dump: &[u8]) -> Result<(), Box<dyn Error>> {
      use tokio::io::AsyncWriteExt;
      // CR finding #10: cache-hit paths land here without going through
      // load_or_build, so the docker CLI preflight must run here too.
      // OnceCell makes it free on every call after the first.
      ensure_docker_cli().await?;
      let start = std::time::Instant::now();
      let mut child = Command::new("docker")
        .args([
          "exec",
          "-i",
          "-u",
          "postgres",
          container_id,
          "pg_restore",
          "-U",
          "lemmy",
          "-d",
          "lemmy",
          "--no-owner",
          "--no-privileges",
          "--clean",
          "--if-exists",
          "--single-transaction",
          "--exit-on-error",
        ])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        // kill_on_drop is essential for the timeout to actually
        // terminate the underlying docker exec when we cancel the
        // wait_with_output future. Without it, the OS process
        // outlives the cancellation and ties up the docker daemon.
        .kill_on_drop(true)
        .spawn()?;
      // Hard timeout (CR findings #7 + #11). Wraps the ENTIRE
      // stdin-write + drop + wait flow because if the child becomes
      // unresponsive while consuming stdin, the buffer fills and
      // write_all stalls — we need that stall to count against the
      // timeout. kill_on_drop on the Command above ensures the
      // underlying docker exec is killed when this future is
      // cancelled mid-write.
      let output = match tokio::time::timeout(
        std::time::Duration::from_secs(PG_RESTORE_TIMEOUT_SECS),
        async {
          {
            let stdin = child
              .stdin
              .as_mut()
              .ok_or_else(|| -> Box<dyn Error> { "no stdin on pg_restore child".into() })?;
            stdin.write_all(dump).await?;
            stdin.flush().await?;
          }
          // Drop stdin so pg_restore sees EOF.
          drop(child.stdin.take());
          child
            .wait_with_output()
            .await
            .map_err(|e| -> Box<dyn Error> { format!("wait_with_output: {e}").into() })
        },
      )
      .await
      {
        Ok(result) => result?,
        Err(_) => {
          return Err(
            format!(
              "pg_restore timed out after {PG_RESTORE_TIMEOUT_SECS}s — \
               docker daemon or postgres may be stalled, or stdin \
               write blocked because the child stopped consuming. \
               Container ID: {container_id}. Consider \
               BREHON_E2E_NO_TEMPLATE=1 as a workaround."
            )
            .into(),
          );
        }
      };
      if !output.status.success() {
        return Err(
          format!(
            "pg_restore failed (exit {:?}): {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr)
          )
          .into(),
        );
      }
      tracing::info!(
        ms = start.elapsed().as_millis(),
        "pg_restore: template applied"
      );
      Ok(())
    }
  }

  /// Bare container with no schema applied. Equivalent to the
  /// historical `start_postgres` (pre-Tier-3). Used by:
  ///   - `pg_template::build_template` for the bootstrap dump.
  ///   - Tests that explicitly need to exercise cold migrations
  ///     (`phase1_migrations_round_trip`,
  ///     `v1_jm_a_backfill_populates_v0_snapshot`) — these pair the
  ///     vanilla container with `schema_setup::run` to test the
  ///     migration runner itself, which would no-op against a
  ///     template-restored DB. CR finding #19.
  pub async fn start_postgres_vanilla() -> Result<
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

  /// Legacy alias for the historical synchronous schema-apply path
  /// — same body as the original `apply_all_schema`, kept for the
  /// template-bootstrap path (where the sentinel deliberately misses)
  /// and for `BREHON_E2E_NO_TEMPLATE=1` debug runs.
  fn apply_all_schema_legacy(conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
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
  ///
  /// **Tier 3 default path:** boots the container, then `pg_restore`s
  /// the bootstrap template dump into it. The first caller in any
  /// nextest process pays the ~30s bootstrap (vanilla container +
  /// migrations + pg_dump → cached `Vec<u8>`); every other caller in
  /// the same process pays only ~1-3s (fresh container + restore).
  /// Existing call sites that pair this with `apply_all_schema(&mut
  /// conn)` continue to compile and run unchanged — the sentinel in
  /// `apply_all_schema` short-circuits when the restore already
  /// populated the schema.
  ///
  /// **Legacy fallback:** set `BREHON_E2E_NO_TEMPLATE=1` to skip the
  /// restore and rely on `apply_all_schema`'s legacy migration path.
  /// Useful for debugging schema regressions where the dump might be
  /// suspect.
  pub async fn start_postgres() -> LemmyResult<(
    testcontainers::ContainerAsync<testcontainers::GenericImage>,
    u16,
  )> {
    let (container, host_port) = start_postgres_vanilla()
      .await
      .map_err(|e| anyhow::anyhow!("{e}"))?;
    if std::env::var("BREHON_E2E_NO_TEMPLATE").as_deref() != Ok("1") {
      let dump = pg_template::ensure_template()
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
      pg_template::pg_restore_into(container.id(), dump)
        .await
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    Ok((container, host_port))
  }

  /// Build a standard test DB URL for the given mapped host port.
  pub fn db_url(host_port: u16) -> String {
    format!("postgres://lemmy:password@localhost:{host_port}/lemmy")
  }

  pub const SIGNING_SEED_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";

  /// Spin a fresh Postgres, apply the full Brehon schema, build a real
  /// `LemmyContext` wrapped in `Data`, and return the context handle,
  /// the container guard (keep alive via `_container`), and the db URL.
  pub async fn bootstrap() -> LemmyResult<(
    testcontainers::ContainerAsync<testcontainers::GenericImage>,
    Data<LemmyContext>,
    String,
  )> {
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

    let (container, host_port) = start_postgres().await?;
    let db_url = db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

    {
      let mut sync_conn = PgConnection::establish(&db_url)?;
      apply_all_schema(&mut sync_conn)?;
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

    Ok((container, context, db_url))
  }

  /// Seed a person/local_user pair, returning both the PersonId and the
  /// `LocalUserView` callers need to invoke handlers.
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

  /// Seed a community for use in governance tests.
  pub async fn seed_community(
    ctx: &LemmyContext,
    instance_id: InstanceId,
  ) -> LemmyResult<Community> {
    let community_form = CommunityInsertForm::new(
      instance_id,
      "testcomm".to_string(),
      "Test Community".to_string(),
      "comm-pubkey".to_string(),
    );
    Community::create(&mut ctx.pool(), &community_form).await
  }

  /// Seed `count` jurors and return their PersonIds in insertion order.
  /// Names are `juror_<i>` zero-padded to ensure sort stability in lookups.
  pub async fn seed_jurors(
    ctx: &LemmyContext,
    instance_id: InstanceId,
    count: usize,
  ) -> LemmyResult<Vec<PersonId>> {
    let mut ids = Vec::with_capacity(count);
    for i in 0..count {
      let name = format!("juror_{i:02}");
      let (pid, _) = seed_user(ctx, instance_id, &name, false).await?;
      ids.push(pid);
    }
    Ok(ids)
  }
}

#[tokio::test]
async fn can_insert_moderation_case() -> lemmy_utils::error::LemmyResult<()> {
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
    ..Default::default()
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
async fn governance_log_hash_chain_holds() -> lemmy_utils::error::LemmyResult<()> {
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

/// Explicit, newest-first list of migration directory basenames the
/// Phase-1 round-trip tests revert. The list itself is authoritative:
/// per-group counts and bump ranges are documented inline beside each
/// group below (the previous hand-maintained phase-by-phase tally is
/// gone — it had drifted out of sync with the entries, the exact failure
/// mode audit 3.E.4 set out to remove).
///
/// The migration runner (`lemmy_diesel_utils::schema_setup::Options::limit`)
/// supports only count-based revert, not named revert, so the runner is
/// driven by `MIGRATIONS_TO_REVERT_PHASE_1.len()`. Because `.len()` alone
/// would let a stale / typo'd / mis-ordered basename pass silently (the
/// names being inert decoration was itself an audit-3.E.4 gap CR flagged
/// on PR #132), each round-trip test runs `assert_revert_list_matches_disk`
/// as a pre-flight: it derives the actual newest-N migration directory
/// basenames from disk and asserts they equal this slice, so the list is
/// now executable, not merely documentary.
///
/// **Known uncounted drift**: v1-AD-a shipped 4 migrations
/// (rule_set_versions, sponsor_allowlist, case_applied_config_snapshot,
/// seed_v1_config_keys) that predate this list's introduction. The
/// round-trip tests are `#[ignore]`d pending GH issue #43; the pre-flight
/// assertion will enforce list⇄disk parity once they are un-ignored.
const MIGRATIONS_TO_REVERT_PHASE_1: &[&str] = &[
  // ADR-017 author-as-defendant backfill (1 migration, bump 19 → 20).
  // Data-only backfill of moderation_case.target_person_id; reverting it
  // (down.sql nulls the backfilled rows) is a no-op on empty governance
  // tables, so it slots cleanly into the newest-first revert window.
  "2026-06-01-000000-0000_backfill_author_defendant",
  // v1-federation-inbound-a (1 migration, bump 18 → 19)
  "2026-05-17-000000-0000_add_federation_inbound_v1",
  // v1-RT-r1 (4 migrations, bump 14 → 18)
  "2026-05-10-000300-0000_seed_v1_rt_config_keys",
  "2026-05-10-000200-0000_backfill_reputation_event_source_type",
  "2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1",
  "2026-05-10-000000-0000_add_reputation_event_v1_columns",
  // v1-SL-a (2 migrations, bump 12 → 14)
  "2026-05-03-000100-0000_add_sponsor_liability_grace_window",
  "2026-05-03-000000-0000_add_case_status_sponsor_liability_variants",
  // v1-SL-b (2 appeal migrations, bump 10 → 12)
  "2026-04-27-000100-0000_add_appeals_v1_columns",
  "2026-04-27-000000-0000_add_appeal_requester_role_enum",
  // v1-JM-a (4 migrations, bump 6 → 10)
  "2026-04-23-000200-0000_seed_v1_jm_config_keys",
  "2026-04-23-000100-0000_add_jury_mechanics_columns",
  "2026-04-23-000050-0000_add_jury_constraint_relaxation_reason_enum",
  "2026-04-23-000000-0000_add_jury_mechanics_enums",
  // v1-AD-a uncounted drift (4 migrations, present in LIFO window but not
  // yet formally counted — see uncounted-drift note above)
  "2026-04-22-005541-0000_update_modlog_check_constraint",
  "2026-04-22-000300-0000_seed_v1_config_keys",
  "2026-04-22-000200-0000_add_case_applied_config_snapshot",
  "2026-04-22-000100-0000_add_sponsor_allowlist",
  "2026-04-22-000000-0000_add_rule_set_versions",
  // Phase 1 (1 migration at this boundary)
  "2026-04-21-000000-0000_add_federation_attestations",
];

/// Pre-flight for the Phase-1 round-trip tests: assert that
/// `MIGRATIONS_TO_REVERT_PHASE_1` actually matches the newest-N migration
/// directories on disk, newest-first.
///
/// The revert/reapply runners only consume `MIGRATIONS_TO_REVERT_PHASE_1`
/// via `.len()`, so without this check a stale, typo'd, or mis-ordered
/// basename would change nothing observable — the named list would be
/// inert decoration and audit 3.E.4's drift-resistance goal would not be
/// met (CR flagged exactly this on PR #132). Reading the real directory
/// names and asserting equality makes the list executable: any future
/// migration that lands without updating this list (or any reordering /
/// rename) fails the round-trip tests with a precise diff instead of
/// silently reverting the wrong window.
///
/// `migrations/` is the same repo-root directory the harness embeds via
/// `embed_migrations!("../../migrations")`; resolved here at runtime
/// relative to this crate's `CARGO_MANIFEST_DIR` (`crates/server`).
fn assert_revert_list_matches_disk() {
  let migrations_dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../migrations");
  let mut dirs: Vec<String> = std::fs::read_dir(migrations_dir)
    .unwrap_or_else(|e| panic!("cannot read migrations dir {migrations_dir}: {e}"))
    .filter_map(|entry| {
      let entry = entry.expect("readable migrations dir entry");
      entry
        .file_type()
        .expect("entry file_type")
        .is_dir()
        .then(|| entry.file_name().to_string_lossy().into_owned())
    })
    .collect();
  // Migration directory names are `YYYY-MM-DD-HHMMSS-NNNN_slug`, so
  // lexicographic descending order == chronological newest-first, matching
  // the LIFO order the runner reverts in.
  dirs.sort_unstable();
  dirs.reverse();

  let n = MIGRATIONS_TO_REVERT_PHASE_1.len();
  let newest_n: Vec<&str> = dirs.iter().take(n).map(String::as_str).collect();

  assert_eq!(
    newest_n.len(),
    n,
    "migrations/ has only {} directories but \
     MIGRATIONS_TO_REVERT_PHASE_1 expects at least {n} \
     (newest-first list drifted from disk)",
    newest_n.len()
  );
  assert_eq!(
    newest_n, MIGRATIONS_TO_REVERT_PHASE_1,
    "MIGRATIONS_TO_REVERT_PHASE_1 is out of sync with the newest {n} \
     migration directories on disk (newest-first). The named revert list \
     must equal the actual disk window or the runner reverts the wrong \
     migrations — update MIGRATIONS_TO_REVERT_PHASE_1 to match (audit \
     3.E.4 drift-resistance)."
  );
}

/// Standalone, non-`#[ignore]`d guard so the audit-3.E.4 revert-list
/// parity check actually runs in the default e2e suite. The three
/// round-trip tests below also call `assert_revert_list_matches_disk()`
/// as a pre-flight, but they are all `#[ignore]`d pending GH issue #43,
/// so without this wrapper a stale/misordered `MIGRATIONS_TO_REVERT_PHASE_1`
/// would drift unnoticed in normal CI. The helper is pure `std::fs` +
/// `assert_eq!` (no DB / tokio / testcontainer), so a plain `#[test]`
/// runs it in milliseconds by default.
#[test]
fn phase1_revert_list_matches_disk() {
  assert_revert_list_matches_disk();
}

/// Step 1 of the Phase-1 round-trip: apply all migrations, assert
/// post-forward schema invariants (Phase-1 tables, SL-a columns/indexes,
/// RT-r1 columns/indexes/types). Kept separate from the revert and
/// re-apply steps so CI can attribute failures to a specific sub-phase.
#[ignore = "TODO(v0-polish): deflake — GH issue #43 (needs revert-list extension for federation tables)"]
#[tokio::test]
async fn test_phase1_migrations_forward() -> lemmy_utils::error::LemmyResult<()> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl, sql_query};
  use lemmy_diesel_utils::schema_setup::{self, Options};

  // Pre-flight: the named revert list must match the newest-N migration
  // directories on disk before any runner consumes its `.len()` (audit
  // 3.E.4 drift-resistance — see `assert_revert_list_matches_disk`).
  assert_revert_list_matches_disk();

  #[derive(diesel::QueryableByName)]
  struct Count {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
  }

  // Use the vanilla container (no Tier 3 template restore) so step 1's
  // forward apply genuinely exercises the migration runner. The Tier 3
  // template-restored DB would make this no-op (CR finding #19).
  let (_container, host_port) = governance_fixtures::start_postgres_vanilla()
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
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
      "jury_constraint_violation_log",
      "surety",
      "endorsement",
      "reputation_event",
      "reputation_snapshot",
      "actor_pseudonym",
      "governance_log",
    ] {
      let result: Count =
        sql_query(format!("SELECT count(*) AS n FROM {table}")).get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "{table} should exist and be empty after forward migration"
      );
    }
  }

  // Post-condition probes for v1-SL-a schema effects (post-forward): columns,
  // indexes, pg_enum values, and governance_config key presence (plan §10.8).
  {
    let mut conn = PgConnection::establish(&db_url)?;
    // 2 new columns added to moderation_case by add_sponsor_liability_grace_window
    for col in ["grace_expires_at", "liability_escape_reason"] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'moderation_case' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "column {col} should exist in moderation_case after SL-a forward migration"
      );
    }
    // 2 new indexes added by add_sponsor_liability_grace_window
    for idx in [
      "moderation_case_grace_expires_idx",
      "surety_sponsored_id_active",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_indexes WHERE indexname = '{idx}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "index {idx} should exist after SL-a forward migration"
      );
    }
    // 3 new CaseStatus enum values added by add_case_status_sponsor_liability_variants
    for val in [
      "SponsorLiabilityPending",
      "SponsorLiabilityFired",
      "SponsorLiabilityEscaped",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_enum e \
         JOIN pg_type t ON e.enumtypid = t.oid \
         WHERE t.typname = 'case_status' AND e.enumlabel = '{val}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "pg_enum value {val} should exist for case_status after SL-a forward migration"
      );
    }
    // governance_config: check that v1-SL-a-specific keys exist after forward migration.
    // Using key-presence checks instead of a brittle total-count assertion —
    // the total grows with future phases; the specific keys are the invariant.
    for key in [
      "job.grace_check_interval_minutes",
      "liability.grace_window_minimum_hours",
    ] {
      let kc: Count = sql_query(format!(
        "SELECT count(*) AS n FROM governance_config WHERE key = '{key}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        kc.n, 1,
        "governance_config key {key} should exist after v1-SL-a forward migration"
      );
    }
  }

  // Post-condition probes for v1-RT-r1 schema effects (post-forward): columns,
  // indexes, pg_type, and governance_config key count delta (plan §10.10).
  {
    let mut conn = PgConnection::establish(&db_url)?;
    // 2 new columns added to reputation_event by add_reputation_event_v1_columns
    for col in ["dedupe_key", "source_event_type"] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'reputation_event' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "column {col} should exist in reputation_event after RT-r1 forward migration"
      );
    }
    // 2 new columns added to sponsor_allowlist by extend_sponsor_allowlist_for_r1
    for col in ["added_by_admin_id", "note"] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'sponsor_allowlist' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "column {col} should exist in sponsor_allowlist after RT-r1 forward migration"
      );
    }
    // community_id should now be nullable in sponsor_allowlist
    let nullable_result: Count = sql_query(
      "SELECT count(*) AS n FROM information_schema.columns \
       WHERE table_name = 'sponsor_allowlist' AND column_name = 'community_id' \
       AND is_nullable = 'YES'",
    )
    .get_result(&mut conn)?;
    assert_eq!(
      nullable_result.n, 1,
      "community_id should be nullable in sponsor_allowlist after RT-r1 forward migration"
    );
    // partial unique index added by add_reputation_event_v1_columns
    let idx_result: Count = sql_query(
      "SELECT count(*) AS n FROM pg_indexes \
       WHERE indexname = 'reputation_event_dedupe_key_partial_idx'",
    )
    .get_result(&mut conn)?;
    assert_eq!(
      idx_result.n, 1,
      "index reputation_event_dedupe_key_partial_idx should exist after RT-r1 forward migration"
    );
    // pg_type for the new enum
    let type_result: Count =
      sql_query("SELECT count(*) AS n FROM pg_type WHERE typname = 'reputation_event_source_type'")
        .get_result(&mut conn)?;
    assert_eq!(
      type_result.n, 1,
      "pg_type reputation_event_source_type should exist after RT-r1 forward migration"
    );
    // governance_config row count delta: +26 from seed_v1_rt_config_keys
    for key in [
      "feature.reputation_v1_decay_enabled",
      "decay.reporting_accuracy.positive_half_life_days",
      "bounds.endorsement_strength.ceiling",
    ] {
      let kc: Count = sql_query(format!(
        "SELECT count(*) AS n FROM governance_config WHERE key = '{key}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        kc.n, 1,
        "governance_config key {key} should exist after v1-RT-r1 forward migration"
      );
    }
  }

  // Post-condition probes for v1-federation-inbound-a schema effects (post-forward):
  // tables, pg_type, columns, indexes, and governance_config key presence (plan §10.8).
  {
    let mut conn = PgConnection::establish(&db_url)?;
    // 4 new tables created by add_federation_inbound_v1
    for tbl in [
      "federation_peer",
      "federation_inbox_dropped_log",
      "federation_inbox_nonce",
      "remote_moderation_label",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.tables \
         WHERE table_name = '{tbl}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "table {tbl} should exist after federation-inbound-a forward migration"
      );
    }
    // 2 new enum types created by add_federation_inbound_v1
    for typname in [
      "federation_peer_trust_enum",
      "federation_inbox_admin_action_enum",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_type WHERE typname = '{typname}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "pg_type {typname} should exist after federation-inbound-a forward migration"
      );
    }
    // 4 new columns added to remote_sanction_notice by add_federation_inbound_v1
    for col in [
      "peer_trust_level_at_receipt",
      "admin_reviewed_at",
      "admin_action",
      "dismissal_rationale",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'remote_sanction_notice' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "column {col} should exist in remote_sanction_notice after federation-inbound-a forward migration"
      );
    }
    // 6 new columns added to federation_attestation by add_federation_inbound_v1
    for col in [
      "source_instance",
      "received_at",
      "peer_trust_level_at_receipt",
      "admin_reviewed_at",
      "admin_action",
      "dismissal_rationale",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'federation_attestation' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "column {col} should exist in federation_attestation after federation-inbound-a forward migration"
      );
    }
    // 10 new indexes created by add_federation_inbound_v1
    for idx in [
      "idx_federation_peer_trust",
      "idx_fil_drop_source",
      "idx_fil_drop_reason",
      "idx_fin_nonce_seen_at",
      "idx_rml_target",
      "idx_rml_source",
      "idx_rml_admin_action",
      "idx_rsn_admin_action",
      "idx_fa_admin_action",
      "idx_fa_source",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_indexes WHERE indexname = '{idx}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 1,
        "index {idx} should exist after federation-inbound-a forward migration"
      );
    }
    // governance_config: spot-check 2 of the 11 new federation-inbound-a config keys
    for key in [
      "federation.inbound.default_trust_for_new_peers",
      "federation.inbound.replay_window_days",
    ] {
      let kc: Count = sql_query(format!(
        "SELECT count(*) AS n FROM governance_config WHERE key = '{key}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        kc.n, 1,
        "governance_config key {key} should exist after federation-inbound-a forward migration"
      );
    }
  }

  Ok(())
}

/// Step 2 of the Phase-1 round-trip: forward-apply → revert all Phase-1
/// migrations → assert that each Phase-1 table, enum type, column, index,
/// and governance_config key is absent after the revert.
#[ignore = "TODO(v0-polish): deflake — GH issue #43 (needs revert-list extension for federation tables)"]
#[tokio::test]
async fn test_phase1_migrations_revert() -> lemmy_utils::error::LemmyResult<()> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl, sql_query};
  use lemmy_diesel_utils::schema_setup::{self, Options};

  // Pre-flight: the named revert list must match the newest-N migration
  // directories on disk before `.limit(MIGRATIONS_TO_REVERT_PHASE_1.len())`
  // reverts a window (audit 3.E.4 — see `assert_revert_list_matches_disk`).
  assert_revert_list_matches_disk();

  #[derive(diesel::QueryableByName)]
  struct Count {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
  }

  let (_container, host_port) = governance_fixtures::start_postgres_vanilla()
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
  let db_url = governance_fixtures::db_url(host_port);
  schema_setup::run(Options::default().run(), &db_url)?;

  // Step 2: revert the last N migrations via the native runner. This
  // exercises each Phase 1 `down.sql` in LIFO order. Task 7 (governance_log)
  // reverts first, task 2 (enums) reverts last. Any broken down.sql fails
  // here — the runner propagates the SQL error up through anyhow.
  schema_setup::run(
    Options::default()
      .revert()
      .limit(u64::try_from(MIGRATIONS_TO_REVERT_PHASE_1.len()).expect("len fits u64")),
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
      "jury_constraint_violation_log",
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
      "severity_tier",
      "case_status_tier",
      "jury_assignment_role",
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

  // Post-condition probes for v1-SL-a schema effects after LIFO-14 revert
  // (plan §10.8): columns absent, indexes absent, specific governance_config
  // keys absent. (SL-a enum label persistence not checked — see inline note.)
  {
    let mut conn = PgConnection::establish(&db_url)?;
    // 2 SL-a columns must be absent after revert
    for col in ["grace_expires_at", "liability_escape_reason"] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'moderation_case' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "column {col} should not exist in moderation_case after reverting SL-a migrations"
      );
    }
    // 2 SL-a indexes must be absent after revert
    for idx in [
      "moderation_case_grace_expires_idx",
      "surety_sponsored_id_active",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_indexes WHERE indexname = '{idx}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "index {idx} should not exist after reverting SL-a migrations"
      );
    }
    // Note: SL-a `case_status` enum labels (SponsorLiabilityPending, etc.) are not
    // checked for persistence here. The LIFO-14 revert includes reverting the
    // Phase 1 enum migration, which DROP TYPEs `case_status` entirely. When the type
    // is dropped all its labels go with it — the "Postgres ALTER TYPE DROP VALUE
    // is unsupported" caveat applies only when reverting an ADD VALUE migration
    // without dropping the type. Asserting label persistence after a DROP TYPE
    // would always fail.
    // governance_config: v1-SL-a-specific keys must be absent after LIFO-14 revert.
    // Using key-absence checks instead of a brittle (post_up_count - 67) assertion.
    for key in [
      "job.grace_check_interval_minutes",
      "liability.grace_window_minimum_hours",
    ] {
      let kc: Count = sql_query(format!(
        "SELECT count(*) AS n FROM governance_config WHERE key = '{key}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        kc.n, 0,
        "governance_config key {key} should be absent after reverting SL-a migrations"
      );
    }
  }

  // Post-condition probes for v1-RT-r1 schema effects after LIFO-18 revert
  // (plan §10.10): columns absent, index absent, pg_type absent, and
  // governance_config keys absent.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    // RT-r1 columns must be absent from reputation_event after revert.
    // (The table itself is dropped by Phase 1 revert in LIFO order after
    // RT-r1 down.sql removes the columns — this probe guards LIFO correctness.)
    for col in ["dedupe_key", "source_event_type"] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'reputation_event' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "column {col} should not exist in reputation_event after reverting RT-r1 migrations"
      );
    }
    // RT-r1 columns must be absent from sponsor_allowlist after revert.
    for col in ["added_by_admin_id", "note"] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'sponsor_allowlist' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "column {col} should not exist in sponsor_allowlist after reverting RT-r1 migrations"
      );
    }
    // Partial unique index must be absent after revert.
    let idx_result: Count = sql_query(
      "SELECT count(*) AS n FROM pg_indexes \
       WHERE indexname = 'reputation_event_dedupe_key_partial_idx'",
    )
    .get_result(&mut conn)?;
    assert_eq!(
      idx_result.n, 0,
      "index reputation_event_dedupe_key_partial_idx should not exist after reverting RT-r1 migrations"
    );
    // GOTCHA (plan §10.10): CREATE TYPE / DROP TYPE cycle must be clean.
    // If down.sql omits DROP TYPE, the type persists in pg_type even after
    // the table is dropped, and Step 3 re-apply fails with "type already
    // exists". This probe catches that before the re-apply.
    let type_result: Count =
      sql_query("SELECT count(*) AS n FROM pg_type WHERE typname = 'reputation_event_source_type'")
        .get_result(&mut conn)?;
    assert_eq!(
      type_result.n, 0,
      "pg_type reputation_event_source_type should not exist after reverting RT-r1 migrations"
    );
    // governance_config: v1-RT-r1-specific keys must be absent after LIFO-18 revert.
    for key in [
      "feature.reputation_v1_decay_enabled",
      "decay.reporting_accuracy.positive_half_life_days",
      "bounds.endorsement_strength.ceiling",
    ] {
      let kc: Count = sql_query(format!(
        "SELECT count(*) AS n FROM governance_config WHERE key = '{key}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        kc.n, 0,
        "governance_config key {key} should be absent after reverting RT-r1 migrations"
      );
    }
  }

  // Post-condition probes for v1-federation-inbound-a schema effects after LIFO-19 revert
  // (plan §10.8): tables absent, types absent, columns absent, indexes absent,
  // governance_config keys absent.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    // 4 tables must be absent after revert
    for tbl in [
      "federation_peer",
      "federation_inbox_dropped_log",
      "federation_inbox_nonce",
      "remote_moderation_label",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.tables \
         WHERE table_name = '{tbl}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "table {tbl} should not exist after reverting federation-inbound-a migrations"
      );
    }
    // 2 enum types must be absent after revert
    for typname in [
      "federation_peer_trust_enum",
      "federation_inbox_admin_action_enum",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_type WHERE typname = '{typname}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "pg_type {typname} should not exist after reverting federation-inbound-a migrations"
      );
    }
    // 4 columns on remote_sanction_notice must be absent after revert
    for col in [
      "peer_trust_level_at_receipt",
      "admin_reviewed_at",
      "admin_action",
      "dismissal_rationale",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'remote_sanction_notice' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "column {col} should not exist in remote_sanction_notice after reverting federation-inbound-a migrations"
      );
    }
    // 6 columns on federation_attestation must be absent after revert
    for col in [
      "source_instance",
      "received_at",
      "peer_trust_level_at_receipt",
      "admin_reviewed_at",
      "admin_action",
      "dismissal_rationale",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM information_schema.columns \
         WHERE table_name = 'federation_attestation' AND column_name = '{col}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "column {col} should not exist in federation_attestation after reverting federation-inbound-a migrations"
      );
    }
    // 10 indexes must be absent after revert
    for idx in [
      "idx_federation_peer_trust",
      "idx_fil_drop_source",
      "idx_fil_drop_reason",
      "idx_fin_nonce_seen_at",
      "idx_rml_target",
      "idx_rml_source",
      "idx_rml_admin_action",
      "idx_rsn_admin_action",
      "idx_fa_admin_action",
      "idx_fa_source",
    ] {
      let result: Count = sql_query(format!(
        "SELECT count(*) AS n FROM pg_indexes WHERE indexname = '{idx}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        result.n, 0,
        "index {idx} should not exist after reverting federation-inbound-a migrations"
      );
    }
    // governance_config: spot-check 2 federation-inbound-a keys absent after revert
    for key in [
      "federation.inbound.default_trust_for_new_peers",
      "federation.inbound.replay_window_days",
    ] {
      let kc: Count = sql_query(format!(
        "SELECT count(*) AS n FROM governance_config WHERE key = '{key}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        kc.n, 0,
        "governance_config key {key} should be absent after reverting federation-inbound-a migrations"
      );
    }
  }

  Ok(())
}

/// Step 3 of the Phase-1 round-trip: forward-apply → revert → re-apply.
/// Verifies that `down.sql` leaves no artifacts that would prevent a clean
/// second apply (no lingering types, tables, or constraints).
#[ignore = "TODO(v0-polish): deflake — GH issue #43 (needs revert-list extension for federation tables)"]
#[tokio::test]
async fn test_phase1_migrations_reapply() -> lemmy_utils::error::LemmyResult<()> {
  use diesel::{Connection as _, PgConnection, RunQueryDsl, sql_query};
  use lemmy_diesel_utils::schema_setup::{self, Options};

  // Pre-flight: the named revert list must match the newest-N migration
  // directories on disk before the revert→reapply window is computed from
  // its `.len()` (audit 3.E.4 — see `assert_revert_list_matches_disk`).
  assert_revert_list_matches_disk();

  #[derive(diesel::QueryableByName)]
  struct Count {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
  }

  let (_container, host_port) = governance_fixtures::start_postgres_vanilla()
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
  let db_url = governance_fixtures::db_url(host_port);
  schema_setup::run(Options::default().run(), &db_url)?;
  schema_setup::run(
    Options::default()
      .revert()
      .limit(u64::try_from(MIGRATIONS_TO_REVERT_PHASE_1.len()).expect("len fits u64")),
    &db_url,
  )?;

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

/// v1-JM-a backfill smoke test (PRD §8.4 + plan §13 Task 10 sub-edit 3).
///
/// Exercises the exact up/down/up cycle production will see if an admin
/// deploys JM-a, rolls it back, and re-deploys:
///   1. Start fresh Postgres container, apply ALL migrations (JM-a included).
///   2. Revert the 4 JM-a migrations LIFO (seed_v1_jm_config_keys,
///      add_jury_mechanics_columns, add_jury_constraint_relaxation_reason_enum,
///      add_jury_mechanics_enums).
///   3. Insert two `moderation_case` rows in the pre-JM-a shape — no
///      JM-a columns exist because their migration is reverted. One row
///      has `decided_at` set (simulates a v0 case that was Decided before
///      JM-a shipped); the other has only `opened_at` (simulates a v0
///      Open case in flight at migration time).
///   4. Re-apply the 4 JM-a migrations — backfill UPDATE fires.
///   5. Query both rows; assert snapshot columns are Minor/Regular/5/3/3
///      per PRD §8.4, and `appeal_window_expires_at` semantics match the
///      migration's `COALESCE(closed_at, decided_at + '7 days', NULL)`
///      branches.
///
/// Raw SQL throughout — the Diesel `moderation_case` struct has JM-a
/// columns after re-apply, but the test must also operate between revert
/// and re-apply when those columns do not exist, so a typed model read
/// would not compile against both states. `sql_query` + `QueryableByName`
/// keeps the probe column-set flexible.
#[tokio::test]
async fn v1_jm_a_backfill_populates_v0_snapshot() -> lemmy_utils::error::LemmyResult<()> {
  use diesel::sql_types::{Int4, Int8, Nullable, Text, Timestamptz};
  use diesel::{Connection as _, PgConnection, RunQueryDsl, sql_query};
  use lemmy_diesel_utils::schema_setup::{self, Options};

  /// Snapshot read after re-apply. Columns only exist post-JM-a, so this
  /// shape is only valid in the step-5 probe. The three `_snapshot`
  /// columns are INTEGER NOT NULL after backfill. `severity_tier` and
  /// `status_tier` read as TEXT because the enum values print as their
  /// label (`"Minor"`, `"Regular"`) via pg's implicit enum→text cast
  /// (using explicit `::text` in the SELECT avoids a Diesel type-binding
  /// issue for the new enum sql_types).
  #[derive(diesel::QueryableByName, Debug)]
  #[expect(
    dead_code,
    reason = "struct fields accessed via Diesel QueryableByName reflection"
  )]
  struct BackfilledRow {
    #[diesel(sql_type = Int4)]
    id: i32,
    #[diesel(sql_type = Text)]
    severity_tier: String,
    #[diesel(sql_type = Text)]
    status_tier: String,
    #[diesel(sql_type = Int4)]
    panel_size_snapshot: i32,
    #[diesel(sql_type = Int4)]
    quorum_snapshot: i32,
    #[diesel(sql_type = Int4)]
    threshold_count_snapshot: i32,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    appeal_window_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    decided_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = Nullable<Timestamptz>)]
    closed_at: Option<chrono::DateTime<chrono::Utc>>,
  }

  /// Shape of `RETURNING id` from the raw INSERTs in step 3.
  #[derive(diesel::QueryableByName)]
  struct IdRow {
    #[diesel(sql_type = Int4)]
    id: i32,
  }

  /// Probe shape for asserting JM-a columns are absent after revert.
  #[derive(diesel::QueryableByName)]
  struct CountRow {
    #[diesel(sql_type = Int8)]
    n: i64,
  }

  // Use vanilla container so step 1's forward apply genuinely
  // exercises the migration runner. Tier 3 template restore would
  // make step 1 no-op against an already-populated schema (CR
  // finding #19).
  let (_container, host_port) = governance_fixtures::start_postgres_vanilla()
    .await
    .map_err(|e| anyhow::anyhow!("{e}"))?;
  let db_url = governance_fixtures::db_url(host_port);

  // Step 1: full forward apply.
  schema_setup::run(Options::default().run(), &db_url)?;

  // Step 2: revert the 13 JM-a + JM-d Task 1 + SL-b + RT-r1 + federation-inbound-a
  //         migrations LIFO:
  //   - 1 federation-inbound-a migration: 2026-05-17-000000 (newest; slot 1)
  //   - 4 RT-r1 migrations: 2026-05-10-000000 through 2026-05-10-000300
  //   - 2 SL-b migrations: 2026-05-03-000000 and 2026-05-03-000100
  //   - 2 JM-d Task 1 migrations: 2026-04-27-000000 and 2026-04-27-000100
  //   - 4 JM-a migrations: 2026-04-23-000000 through 2026-04-23-000200
  // Runner takes pg_advisory_lock(0) so the forbid_diesel_cli trigger does
  // not fire. Limit must rise with each new phase that adds migrations
  // post-dating JM-a (prior bumps: 4→6 in 4875a20a7 for JM-d Task 3; 6→8
  // for SL-b; 8→12 here for RT-r1; 12→13 here for federation-inbound-a).
  schema_setup::run(Options::default().revert().limit(13), &db_url)?;

  // Sanity: the 3 JM-a columns really are gone — otherwise the step-3
  // INSERTs below would still see DEFAULT 'Minor' / DEFAULT 'Regular'
  // and the backfill branch would be untested (step 5's assertion would
  // pass even if the migration's UPDATE did nothing).
  {
    let mut conn = PgConnection::establish(&db_url)?;
    let row: CountRow = sql_query(
      "SELECT count(*) AS n FROM information_schema.columns \
       WHERE table_name = 'moderation_case' \
       AND column_name = 'severity_tier'",
    )
    .get_result(&mut conn)?;
    assert_eq!(
      row.n, 0,
      "severity_tier column should be absent between revert and re-apply"
    );
    let row: CountRow = sql_query(
      "SELECT count(*) AS n FROM information_schema.tables \
       WHERE table_name = 'jury_constraint_violation_log'",
    )
    .get_result(&mut conn)?;
    assert_eq!(
      row.n, 0,
      "jury_constraint_violation_log should be absent between revert and re-apply"
    );
    // PR #92 cr-5: also assert the 4 JM-a PG enum types are absent post-revert.
    // Protects against a future schema_setup::revert() that drops a table but
    // leaves its backing enum type dangling (which would make the re-apply step
    // fail with "type already exists").
    for pg_type in [
      "severity_tier",
      "case_status_tier",
      "jury_assignment_role",
      "jury_constraint_relaxation_reason",
    ] {
      let row: CountRow = sql_query(format!(
        "SELECT count(*) AS n FROM pg_type WHERE typname = '{pg_type}'"
      ))
      .get_result(&mut conn)?;
      assert_eq!(
        row.n, 0,
        "pg_type '{pg_type}' should be absent between revert and re-apply"
      );
    }
    // cr-14: verify JM-a seed rows are also absent after revert.
    let row: CountRow = sql_query(
      "SELECT count(*) AS n FROM governance_config \
       WHERE scope = 'instance' \
         AND valid_from = '2026-04-23T00:02:00Z'::timestamptz",
    )
    .get_result(&mut conn)?;
    assert_eq!(
      row.n, 0,
      "v1-JM-a governance_config seed rows should be absent between revert and re-apply"
    );
  }

  // Step 3: seed two v0-shape rows via raw SQL. Any column that the
  // pre-JM-a `moderation_case` schema requires (target_type, reason_code)
  // must be explicit; everything else falls to NOT NULL DEFAULTs.
  //
  // Row A: Decided case with decided_at = 2026-04-20T00:00:00Z and no
  // closed_at. Post-backfill: appeal_window_expires_at should be
  // decided_at + 7 days (= 2026-04-27T00:00:00Z).
  //
  // Row B: Open case with neither decided_at nor closed_at. Post-backfill:
  // appeal_window_expires_at should be NULL (the migration's CASE
  // expression returns NULL when both timestamps are NULL).
  let (case_a_id, case_b_id) = {
    let mut conn = PgConnection::establish(&db_url)?;
    let row_a: IdRow = sql_query(
      "INSERT INTO moderation_case \
        (target_type, target_remote_url, reason_code, severity, status, \
         threshold_score, opened_at, decided_at) \
       VALUES \
        ('RemoteInstance', 'https://example.invalid/a', 'spam', 'Medium', 'Decided', \
         1, '2026-04-19T00:00:00Z', '2026-04-20T00:00:00Z') \
       RETURNING id",
    )
    .get_result(&mut conn)?;

    let row_b: IdRow = sql_query(
      "INSERT INTO moderation_case \
        (target_type, target_remote_url, reason_code, severity, status, \
         threshold_score, opened_at) \
       VALUES \
        ('RemoteInstance', 'https://example.invalid/b', 'harassment', 'Low', 'Open', \
         1, '2026-04-19T00:00:00Z') \
       RETURNING id",
    )
    .get_result(&mut conn)?;
    (row_a.id, row_b.id)
  };

  // Step 4: re-apply the 3 JM-a migrations. The `UPDATE moderation_case
  // SET ... WHERE panel_size_snapshot IS NULL` backfill in
  // add_jury_mechanics_columns/up.sql runs over BOTH rows (neither row
  // had the column before, so attmissingval fills NULL, matching the
  // WHERE clause).
  schema_setup::run(Options::default().run(), &db_url)?;

  // Step 5: assert both rows have the expected v0-equivalent snapshot
  // per PRD §8.4: severity_tier='Minor', status_tier='Regular',
  // panel_size=5, quorum=3, threshold_count=3.
  //
  // Enum values are SELECTed as `::text` because our QueryableByName
  // derive uses the plain `Text` sql_type — this avoids needing to
  // register the new severity_tier / case_status_tier Diesel sql_type
  // bindings in the test scope.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    let rows: Vec<BackfilledRow> = sql_query(
      "SELECT id, \
              severity_tier::text AS severity_tier, \
              status_tier::text AS status_tier, \
              panel_size_snapshot, \
              quorum_snapshot, \
              threshold_count_snapshot, \
              appeal_window_expires_at, \
              decided_at, \
              closed_at \
       FROM moderation_case \
       ORDER BY id ASC",
    )
    .load(&mut conn)?;
    assert_eq!(rows.len(), 2, "both seeded rows should be present");

    for row in &rows {
      assert_eq!(
        row.severity_tier, "Minor",
        "case id={}: severity_tier should backfill to Minor per PRD §8.4",
        row.id
      );
      assert_eq!(
        row.status_tier, "Regular",
        "case id={}: status_tier should backfill to Regular per PRD §8.4",
        row.id
      );
      assert_eq!(
        row.panel_size_snapshot, 5,
        "case id={}: panel_size_snapshot should backfill to 5 per PRD §8.4",
        row.id
      );
      assert_eq!(
        row.quorum_snapshot, 3,
        "case id={}: quorum_snapshot should backfill to 3 per PRD §8.4",
        row.id
      );
      assert_eq!(
        row.threshold_count_snapshot, 3,
        "case id={}: threshold_count_snapshot should backfill to 3 per PRD §8.4",
        row.id
      );
    }

    // Row A (Decided, no closed_at): appeal window = decided_at + 7d.
    let row_a = rows
      .iter()
      .find(|r| r.id == case_a_id)
      .expect("Row A (Decided) should be present");
    let decided_a = row_a.decided_at.expect("Row A seeded with decided_at");
    let expected_a = decided_a + chrono::Duration::days(7);
    let actual_a = row_a
      .appeal_window_expires_at
      .expect("Row A should have appeal_window_expires_at = decided_at + 7d");
    assert_eq!(
      actual_a, expected_a,
      "Row A: appeal_window_expires_at should be decided_at + 7 days"
    );

    // Row B (Open, no decided_at, no closed_at): appeal window is NULL.
    let row_b = rows
      .iter()
      .find(|r| r.id == case_b_id)
      .expect("Row B (Open) should be present");
    assert!(
      row_b.appeal_window_expires_at.is_none(),
      "Row B: appeal_window_expires_at should remain NULL when neither decided_at \
       nor closed_at is set at backfill time"
    );
  }

  // jury_constraint_violation_log must exist and be empty post-re-apply.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    let row: CountRow =
      sql_query("SELECT count(*) AS n FROM jury_constraint_violation_log").get_result(&mut conn)?;
    assert_eq!(
      row.n, 0,
      "jury_constraint_violation_log should exist and be empty after re-apply"
    );
  }

  // federation_peer table must exist and be empty post re-apply.
  {
    let mut conn = PgConnection::establish(&db_url)?;
    let row: CountRow =
      sql_query("SELECT count(*) AS n FROM federation_peer").get_result(&mut conn)?;
    assert_eq!(
      row.n, 0,
      "federation_peer should exist and be empty after federation-inbound-a re-apply"
    );
  }

  Ok(())
}

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


// Jury mechanics fixtures + tests (sub-phase 6/7 decomposition):
// Extracted to tests/e2e/jury_mechanics.rs; include! expands inline at crate scope.
include!("e2e/jury_mechanics.rs");


// Sponsor liability fixtures + tests (sub-phase 7/7 decomposition):
// Extracted to tests/e2e/sponsor_liability.rs; include! expands inline at crate scope.
include!("e2e/sponsor_liability.rs");

#[path = "e2e/reputation_rt_r3.rs"]
mod v1_rt_r3_fixtures;


// Reputation RT-r4 fixtures extracted to tests/e2e/reputation_rt_r4.rs (sub-phase 3/7).
#[path = "e2e/reputation_rt_r4.rs"]
mod v1_rt_r4_fixtures;
