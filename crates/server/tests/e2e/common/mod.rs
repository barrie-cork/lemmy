//! Shared e2e test scaffold extracted from `tests/e2e.rs` (Phase 6
//! decomposition, sub-phase 1). Holds the cross-domain fixtures and
//! guards that every per-domain test module under `tests/e2e/` depends
//! on: the Postgres/testcontainer harness (`governance_fixtures`), the
//! process-env RAII guard (`EnvVarGuard`), and the single-column row
//! shape (`SingleI32`).
//!
//! Behaviour-preserving: the bodies below are moved verbatim from
//! `e2e.rs`; the only edits are visibility promotions (`mod` ->
//! `pub(crate) mod`, file-private `struct`/`fn` -> `pub(crate)`) so the
//! symbols resolve across module boundaries now that tests live in
//! sub-modules rather than as file-scope siblings.
//!
//! `#![expect(...)]` lint allowances are inherited from the crate root
//! (`e2e.rs`), which gates the whole test binary.

pub(crate) struct EnvVarGuard {
  key: &'static str,
  prev: Option<String>,
}

impl EnvVarGuard {
  pub(crate) fn set(key: &'static str, value: &str) -> Self {
    let prev = std::env::var(key).ok();
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var(key, value);
    }
    Self { key, prev }
  }
}

impl Drop for EnvVarGuard {
  fn drop(&mut self) {
    // SAFETY: same justification — single-threaded test runner.
    unsafe {
      match &self.prev {
        Some(prev) => std::env::set_var(self.key, prev),
        None => std::env::remove_var(self.key),
      }
    }
  }
}

/// Tiny row shape for `sql_query` probes that return a single `id` column.
#[derive(diesel::QueryableByName)]
pub(crate) struct SingleI32 {
  #[diesel(sql_type = diesel::sql_types::Int4)]
  pub(crate) id: i32,
}

pub(crate) mod governance_fixtures {
  //! **Process-env safety constraint:** every test in this module mutates
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
    // `include_str!` resolves relative to THIS source file. After the Phase 6
    // split this file moved from `tests/e2e.rs` to `tests/e2e/common/mod.rs`
    // (2 dirs deeper), so the original `../../../` prefix gains 2 levels.
    conn.batch_execute(include_str!(
      "../../../../../crates/diesel_utils/replaceable_schema/utils.sql"
    ))?;
    conn.batch_execute(include_str!(
      "../../../../../crates/diesel_utils/replaceable_schema/triggers.sql"
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
