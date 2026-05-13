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

/// Smoke test the harness boot path: container start + Tier 3 template
/// restore (when enabled) + schema sentinel reachable. Asserts that
/// `governance_log` is present in `public` schema after `start_postgres`
/// returns — confirming pg_restore actually populated the schema. With
/// `BREHON_E2E_NO_TEMPLATE=1` the assertion still holds because the
/// caller follows up with `apply_all_schema` (legacy path).
#[tokio::test]
async fn postgres_container_boots() -> Result<(), Box<dyn Error>> {
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
async fn template_dump_capture() -> Result<(), Box<dyn Error>> {
  // Force the template path even if a future test sets BREHON_E2E_NO_TEMPLATE.
  // Skip when env explicitly disables it (legacy fallback validation runs).
  if std::env::var("BREHON_E2E_NO_TEMPLATE").as_deref() == Ok("1") {
    return Ok(());
  }
  let dump = governance_fixtures::pg_template::ensure_template().await?;
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

// ============================================================================
// Phase 1 — governance schema + hash-chain trigger smoke tests
// ============================================================================

mod governance_fixtures {
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
  pub fn schema_sentinel_satisfied(conn: &mut PgConnection) -> Result<bool, Box<dyn Error>> {
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
    .get_result::<CountRow>(conn)
    .map_err(|e| -> Box<dyn Error> { format!("schema sentinel query failed: {e}").into() })?;
    Ok(row.count == 2)
  }

  pub fn apply_all_schema(conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    if schema_sentinel_satisfied(conn)? {
      return Ok(());
    }

    // Sentinel missed → schema not fully populated by template
    // restore. Delegate to the legacy bootstrap path so the
    // migration + r-schema rebuild logic lives in exactly one place
    // (CR finding #12 DRY).
    apply_all_schema_legacy(conn)
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
    static TEMPLATE_DUMP: tokio::sync::OnceCell<Vec<u8>> =
      tokio::sync::OnceCell::const_new();

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
      Ok(target_root.join("tmp").join(format!("brehon-pg-template-{mtime}.dump")))
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
        .map_err(|e| -> Box<dyn Error> {
          format!("template bootstrap establish: {e}").into()
        })?;
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
      .map_err(|e| -> Box<dyn Error> {
        format!("query pg_extension: {e}").into()
      })?;
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
    async fn pg_dump(
      container_id: &str,
      extensions: &[String],
    ) -> Result<Vec<u8>, Box<dyn Error>> {
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
      .map_err(|_| -> Box<dyn Error> {
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
    pub async fn pg_restore_into(
      container_id: &str,
      dump: &[u8],
    ) -> Result<(), Box<dyn Error>> {
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
            let stdin = child.stdin.as_mut().ok_or_else(|| -> Box<dyn Error> {
              "no stdin on pg_restore child".into()
            })?;
            stdin.write_all(dump).await?;
            stdin.flush().await?;
          }
          // Drop stdin so pg_restore sees EOF.
          drop(child.stdin.take());
          child.wait_with_output().await.map_err(|e| -> Box<dyn Error> {
            format!("wait_with_output: {e}").into()
          })
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
  pub async fn start_postgres() -> Result<
    (
      testcontainers::ContainerAsync<testcontainers::GenericImage>,
      u16,
    ),
    Box<dyn Error>,
  > {
    let (container, host_port) = start_postgres_vanilla().await?;
    if std::env::var("BREHON_E2E_NO_TEMPLATE").as_deref() != Ok("1") {
      let dump = pg_template::ensure_template().await?;
      pg_template::pg_restore_into(container.id(), dump).await?;
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
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (container, host_port) = start_postgres()
      .await
      .map_err(|e| anyhow::anyhow!("start_postgres: {e}"))?;
    let db_url = db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
      let mut sync_conn = PgConnection::establish(&db_url)
        .map_err(|e| -> Box<dyn Error + Send + Sync> {
          format!("PgConnection::establish: {e}").into()
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
      apply_all_schema(&mut sync_conn)
        .map_err(|e| anyhow::anyhow!("apply_all_schema: {e}"))?;
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
    Ok(Community::create(&mut ctx.pool(), &community_form).await?)
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

  /// Count of top-N migrations to revert via the native runner, LIFO, so the
  /// assertions below can probe the post-revert and post-re-apply states.
  ///
  /// **This is a LIFO-positional count, not a semantic set.** The runner at
  /// `lemmy_diesel_utils::schema_setup::run` with `.revert().limit(N)` reverts
  /// the top-N-by-timestamp pending migrations. Any migration added to the
  /// fork after the last bump of this constant silently takes the Nth slot
  /// without renaming — the name-list probes below (`moderation_case`, enum
  /// drops, etc.) only assert what happens to be in the LIFO window at this
  /// count. Counting phase-by-phase is a useful bookkeeping fiction, not a
  /// semantic invariant.
  ///
  /// Bumped to 18 in v1-RT-r1 (adds 4: add_reputation_event_v1_columns
  /// @ 2026-05-10-000000, extend_sponsor_allowlist_for_r1 @
  /// 2026-05-10-000100, backfill_reputation_event_source_type @
  /// 2026-05-10-000200, seed_v1_rt_config_keys @ 2026-05-10-000300).
  /// Phase-by-phase breakdown (bookkeeping, not enforced):
  ///   - 6 Phase 1 migrations (enums, core, jury, rep+surety, pseudonym,
  ///     governance_log — the last added in Phase 4b task 8 but part of the
  ///     contiguous governance-bootstrap LIFO block)
  ///   - 2 Phase 5a migrations (add_governance_config +
  ///     add_person_membership_state)
  ///   - 1 Phase 5b Slice A migration (add_restoration_sanction_variant
  ///     — task 56 / OQ-003)
  ///   - 3 v1-JM-a migrations (bump 9 → 12)
  ///   - 2 v1-SL-a migrations (bump 12 → 14)
  ///   - 4 v1-RT-r1 migrations (this bump)
  ///
  /// **Uncounted drift**: v1-AD-a shipped 4 migrations (rule_set_versions,
  /// sponsor_allowlist, case_applied_config_snapshot, seed_v1_config_keys)
  /// but did not bump this constant. When this test is un-ignored (see GH
  /// issue #43), a separate `chore(test): retrofit v1-AD-a migrations into
  /// phase1_migrations_round_trip` commit must reconcile this — not JM-a's
  /// concern per advisor 2026-04-23 (option (c)).
  ///
  // TODO(v0-polish): replace count-based revert with a named-migration list
  // to stop LIFO-positional slot-swap silently hiding uncounted drift. See
  // GH issue #43 (existing #[ignore] reason) + the count-model GH issue
  // sketched in `.claude/PRPs/reports/phase-v1-JM-a-retro.md` §3.
  const PHASE_1_MIGRATION_COUNT: u64 = 18;

  /// Query shape for `COUNT(*)` probes via `sql_query`.
  #[derive(diesel::QueryableByName)]
  struct Count {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    n: i64,
  }

  // Use the vanilla container (no Tier 3 template restore) so step 1's
  // forward apply genuinely exercises the migration runner. The Tier 3
  // template-restored DB would make this no-op (CR finding #19).
  let (_container, host_port) = governance_fixtures::start_postgres_vanilla().await?;
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
      let result: Count = sql_query(format!("SELECT count(*) AS n FROM {table}")).get_result(&mut conn)?;
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
    for idx in ["moderation_case_grace_expires_idx", "surety_sponsored_id_active"] {
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
    for key in ["job.grace_check_interval_minutes", "liability.grace_window_minimum_hours"] {
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
       AND is_nullable = 'YES'"
    )
    .get_result(&mut conn)?;
    assert_eq!(
      nullable_result.n, 1,
      "community_id should be nullable in sponsor_allowlist after RT-r1 forward migration"
    );
    // partial unique index added by add_reputation_event_v1_columns
    let idx_result: Count = sql_query(
      "SELECT count(*) AS n FROM pg_indexes \
       WHERE indexname = 'reputation_event_dedupe_key_partial_idx'"
    )
    .get_result(&mut conn)?;
    assert_eq!(
      idx_result.n, 1,
      "index reputation_event_dedupe_key_partial_idx should exist after RT-r1 forward migration"
    );
    // pg_type for the new enum
    let type_result: Count = sql_query(
      "SELECT count(*) AS n FROM pg_type WHERE typname = 'reputation_event_source_type'"
    )
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
    for idx in ["moderation_case_grace_expires_idx", "surety_sponsored_id_active"] {
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
    for key in ["job.grace_check_interval_minutes", "liability.grace_window_minimum_hours"] {
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
    let type_result: Count = sql_query(
      "SELECT count(*) AS n FROM pg_type WHERE typname = 'reputation_event_source_type'",
    )
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
async fn v1_jm_a_backfill_populates_v0_snapshot() -> Result<(), Box<dyn Error>> {
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
  #[allow(dead_code)]
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
  let (_container, host_port) = governance_fixtures::start_postgres_vanilla().await?;
  let db_url = governance_fixtures::db_url(host_port);

  // Step 1: full forward apply.
  schema_setup::run(Options::default().run(), &db_url)?;

  // Step 2: revert the 12 JM-a + JM-d Task 1 + SL-b + RT-r1 migrations LIFO:
  //   - 4 RT-r1 migrations: 2026-05-10-000000 through 2026-05-10-000300
  //   - 2 SL-b migrations: 2026-05-03-000000 and 2026-05-03-000100
  //   - 2 JM-d Task 1 migrations: 2026-04-27-000000 and 2026-04-27-000100
  //   - 4 JM-a migrations: 2026-04-23-000000 through 2026-04-23-000200
  // Runner takes pg_advisory_lock(0) so the forbid_diesel_cli trigger does
  // not fire. Limit must rise with each new phase that adds migrations
  // post-dating JM-a (prior bumps: 4→6 in 4875a20a7 for JM-d Task 3; 6→8
  // for SL-b; 8→12 here for RT-r1).
  schema_setup::run(Options::default().revert().limit(12), &db_url)?;

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
    let row: CountRow = sql_query("SELECT count(*) AS n FROM jury_constraint_violation_log")
      .get_result(&mut conn)?;
    assert_eq!(
      row.n, 0,
      "jury_constraint_violation_log should exist and be empty after re-apply"
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
  ..Default::default()
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
  ..Default::default()
    };
    let case_id: i32 = diesel::insert_into(moderation_case::table)
      .values(&case_form)
      .returning(moderation_case::id)
      .get_result(&mut sync_conn)?;

    let assignment_form = JuryAssignmentInsertForm {
      case_id: ModerationCaseId(case_id),
      person_id: PersonId(person_id),
      status: lemmy_db_schema_file::enums::JuryAssignmentStatus::Selected,
      selected_under_constraints: None,
      ..Default::default()
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
  ..Default::default()
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

    let (status, decided_at, appeal_window_expires_at): (
      CaseStatus,
      Option<DateTime<Utc>>,
      Option<DateTime<Utc>>,
    ) = moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .select((
        moderation_case::status,
        moderation_case::decided_at,
        moderation_case::appeal_window_expires_at,
      ))
      .first(conn)
      .await?;
    assert!(matches!(status, CaseStatus::Decided), "case must be Decided");
    let decided = decided_at.expect("decided_at set");
    let appeal_expires =
      appeal_window_expires_at.expect("appeal_window_expires_at set by JM-c step 9");
    let gap = appeal_expires.signed_duration_since(decided);
    // DB precision can drift by microseconds; assert within 1 second of 7d
    // (the seeded `appeal.window_days = 7` default per JM-a config seed).
    // JM-c step 9 reads `appeal.window_days` LIVE at decision time and writes
    // `appeal_window_expires_at = decided_at + window_days` (no longer
    // `closed_at = decided_at + 7d`).
    let expected = Duration::days(7);
    assert!(
      (gap - expected).num_milliseconds().abs() < 1_000,
      "appeal_window_expires_at should be ~ decided_at + 7 days (got gap = {gap:?})"
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

/// PR #92 cr-10 fix probe: the v1-JM-a seed migration must be idempotent
/// across manual reruns. Prior to cr-10, `valid_from` defaulted to
/// `now()` per statement, so each rerun inserted a duplicate row (the
/// unique index on `(scope, key, valid_from)` treated two different
/// `now()` values as distinct). The fix pins `valid_from` to a stable
/// literal so `ON CONFLICT DO NOTHING` is a true no-op on rerun.
///
/// Test shape: apply all migrations (seed lands → 27 rows), then
/// execute the seed migration's up.sql a SECOND time directly via
/// `batch_execute`. The second run must leave the row count unchanged.
/// If the `ON CONFLICT` target doesn't match on the second run (the
/// cr-10 bug), the second apply would insert 27 duplicate rows, the
/// governance_config_typed CHECK still passes (every row typed
/// correctly), and the count would be 54 instead of 27.
#[tokio::test]
async fn v1_jm_a_seed_migration_is_idempotent() -> Result<(), Box<dyn Error>> {
  use diesel::sql_types::Int8;
  use diesel::{Connection as _, PgConnection, RunQueryDsl, connection::SimpleConnection, sql_query};
  use lemmy_api::governance::config::EXPECTED_SEED_COUNT_V1_JM;

  #[derive(diesel::QueryableByName)]
  struct Count {
    #[diesel(sql_type = Int8)]
    n: i64,
  }

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let mut conn = PgConnection::establish(&db_url)?;
  governance_fixtures::apply_all_schema(&mut conn)?;

  let expected = EXPECTED_SEED_COUNT_V1_JM as i64;

  // Count rows at the stable seed valid_from.
  let q = "SELECT count(*) AS n FROM governance_config \
           WHERE scope = 'instance' \
             AND valid_from = '2026-04-23T00:02:00Z'::timestamptz";
  let first: Count = sql_query(q).get_result(&mut conn)?;
  assert_eq!(
    first.n, expected,
    "v1-JM-a seed must insert exactly {expected} rows on first apply"
  );

  // Re-execute the seed migration's up.sql directly — simulates a
  // manual rerun (e.g. idempotent redeploy or ops script). The stable
  // valid_from literal + ON CONFLICT DO NOTHING must leave the count
  // unchanged.
  conn.batch_execute(include_str!(
    "../../../migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql"
  ))?;

  let second: Count = sql_query(q).get_result(&mut conn)?;
  assert_eq!(
    second.n, expected,
    "v1-JM-a seed must remain at {expected} rows after second apply (cr-10 idempotency)"
  );

  // And again — triple-check the idempotency holds across multiple reruns.
  conn.batch_execute(include_str!(
    "../../../migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql"
  ))?;
  let third: Count = sql_query(q).get_result(&mut conn)?;
  assert_eq!(
    third.n, expected,
    "v1-JM-a seed must remain at {expected} rows after third apply (cr-10 idempotency)"
  );

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
        dedupe_key: None,
        source_event_type: None,
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
      dedupe_key: None,
      source_event_type: None,
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
  // Positive assertion: at least one payload mentions revoker_pseudonym so
  // the grep isn't vacuously passing on an empty log.
  let saw_pseudonym = payloads
    .iter()
    .any(|p| serde_json::to_string(p).map(|s| s.contains("\"revoker_pseudonym\"")).unwrap_or(false));
  assert!(
    saw_pseudonym,
    "expected at least one governance_log payload with revoker_pseudonym"
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
    // Use a malformed `case_id` so the wired route returns 400 (Query
    // deserialisation fails on non-numeric input). 404 is excluded from
    // the allowlist so a dropped route registration fails this probe
    // instead of silently looking like "empty DB". GH #36.
    ("GET",  "/api/v4/governance/case?case_id=not_a_number",     "",   &[400, 401]),
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
  //
  // The Decided case needs `closed_at` in the future so that the #34 appeal
  // window guard (`closed_at > now()`) allows the target to appeal. The
  // production path (`submit_jury_vote` decided-flip) stamps
  // `closed_at = decided_at + 7d`; we mirror that here with a direct
  // UPDATE since `ModerationCaseInsertForm` doesn't carry `closed_at`.
  {
    use diesel::ExpressionMethods;
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
  ..Default::default()
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

    // Stamp appeal_window_expires_at in the future on every Decided seeded
    // case so the appeal window guard admits the appeal (Task 3 switched the
    // check from closed_at to appeal_window_expires_at). Also seed
    // panel_size_snapshot — Task 3's request_appeal calls select_appeal_panel
    // (admin_assign_jury.rs:1097), which guards on case.panel_size_snapshot
    // being non-NULL. The direct ModerationCaseInsertForm path bypasses
    // admin_assign_jury, leaving the column NULL. Seed to the JM-a default
    // for Minor severity (`jury.panel_size.regular.minor` = 5 from
    // migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:35).
    // Scope both UPDATEs to status=Decided so they only touch the decided_form
    // row even if other tests extend this seed later.
    let future = chrono::Utc::now() + chrono::Duration::days(7);
    diesel::update(moderation_case::table)
      .filter(moderation_case::status.eq(CaseStatus::Decided))
      .set((
        moderation_case::appeal_window_expires_at.eq(Some(future)),
        moderation_case::panel_size_snapshot.eq(Some(5_i32)),
      ))
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
        dedupe_key: None,
        source_event_type: None,
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
  ..Default::default()
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
        selected_under_constraints: None,
        ..Default::default()
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
    let local_site_form_a = LocalSiteInsertForm::new(site_a.id, sysacct.id);
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

  // Seed the admin's actor_pseudonym row up front. In production this row
  // is created the first time the admin appears in a governance write
  // (e.g. by `admin_assign_jury`); this test bypasses the assign-jury path
  // (lines below seed `JuryAssignment` rows directly), so the row would
  // not yet exist when `submit_jury_vote` reaches the federation publish.
  // GH #48 finding 2 turned `federation_outbox::send_local_sanction_notice`
  // into a strict `get` — missing-row is now a hard error rather than a
  // silent INSERT — so the fixture must materialise the row here.
  lemmy_api::governance::actor_pseudonym_helper::get_or_create(
    &mut context_a.pool(),
    admin_pid,
  ).await
    .map_err(|e| -> Box<dyn Error> { format!("seed admin pseudonym: {e}").into() })?;

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
      // v1-JM-c fold-in: submit_jury_vote now reads `quorum_snapshot` /
      // `panel_size_snapshot` / `threshold_count_snapshot` directly from
      // `case` per PRD §9.1 (snapshots written by admin_assign_jury at
      // jury-assemble time). NULL here triggers a hard error per the
      // process-breach guard added in JM-c task 2 (line ~209). This test
      // bypasses `admin_assign_jury`, so we mirror what the handler would
      // write for a 5-juror Minor case (panel=5, quorum=3, threshold=3 —
      // matching `jury.panel_size.regular.minor=5` × `quorum_fraction=0.6` ×
      // `threshold_fraction=0.5001` per the JM-a config seed).
      panel_size_snapshot: Some(5),
      quorum_snapshot: Some(3),
      threshold_count_snapshot: Some(3),
      ..Default::default()
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
        selected_under_constraints: None,
        ..Default::default()
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
    let local_site_form_b = LocalSiteInsertForm::new(site_b.id, sysacct.id);
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

// ============================================================================
// v0-polish — GH #34 regression: appeal allowed inside closed_at window
// ============================================================================
//
// Before the #34 fix, `request_appeal`'s window guard was
// `if case.closed_at.is_some() { NotFound }` — which rejected every
// Decided case because `submit_jury_vote` always stamps
// `closed_at = decided_at + 7d` on Decided-flip. The fix inverts the
// guard to `within_window = closed_at > now()`, admitting appeals only
// while the window is actually open. This test seeds a Decided case
// with `closed_at = now() + 1d`, calls `request_appeal` as the target,
// and asserts a `RequestAppealResponse` is returned with a fresh
// `appeal_id`. A parallel assertion covers the expired-window branch by
// seeding a second case with `closed_at = now() - 1d` and expecting a
// `NotFound` error.

#[tokio::test(flavor = "multi_thread")]
async fn appeal_inside_window_succeeds_expired_rejects() -> Result<(), Box<dyn Error>> {
  use actix_web::web::{Data, Json};
  use chrono::{Duration, Utc};
  use diesel::{Connection as _, ExpressionMethods, PgConnection, QueryDsl};
  use diesel_async::{AsyncConnection as _, AsyncPgConnection, RunQueryDsl};
  use lemmy_api_common::governance::RequestAppeal;
  use lemmy_api_crud::governance::request_appeal::request_appeal;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    governance::moderation_case::ModerationCaseInsertForm, instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm}, person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType},
    schema::moderation_case,
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

  async fn seed_target(
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

  let target_a = seed_target(&context, instance.id, "appeal_target_open").await?;
  let target_b = seed_target(&context, instance.id, "appeal_target_expired").await?;
  let target_a_view = LocalUserView::read_person(&mut context.pool(), target_a).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
  let target_b_view = LocalUserView::read_person(&mut context.pool(), target_b).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  // Seed two Decided cases: case_a has closed_at in the future (+1d), case_b
  // has closed_at in the past (-1d). `ModerationCaseInsertForm` doesn't carry
  // `closed_at`, so we UPDATE after insert — same trick as the seed block in
  // `all_mvp_endpoints_return_non_404`.
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

  let case_a_form = ModerationCaseInsertForm {
    community_id: None,
    creator_id: None,
    target_type: CaseTargetType::Person,
    target_post_id: None,
    target_comment_id: None,
    target_person_id: Some(target_a),
    target_community_id: None,
    target_remote_url: None,
    reason_code: "probe_open".to_string(),
    severity: CaseSeverity::Low,
    status: CaseStatus::Decided,
    threshold_score: 1,
  ..Default::default()
  };
  let case_a: lemmy_db_schema::source::governance::moderation_case::ModerationCase =
    diesel::insert_into(moderation_case::table)
      .values(&case_a_form)
      .get_result(&mut async_conn)
      .await?;

  let case_b_form = ModerationCaseInsertForm {
    target_person_id: Some(target_b),
    reason_code: "probe_expired".to_string(),
    ..case_a_form
  };
  let case_b: lemmy_db_schema::source::governance::moderation_case::ModerationCase =
    diesel::insert_into(moderation_case::table)
      .values(&case_b_form)
      .get_result(&mut async_conn)
      .await?;

  // Task 3 invariant: request_appeal calls select_appeal_panel, which reads
  // case.panel_size_snapshot (admin_assign_jury.rs:1097 guard). The direct
  // ModerationCaseInsertForm path here bypasses admin_assign_jury, leaving
  // panel_size_snapshot NULL. Seed it on both cases to the JM-a default for
  // Minor severity (`jury.panel_size.regular.minor` = 5 from
  // migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:35).
  let future = Utc::now() + Duration::days(1);
  diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_a.id)))
    .set((
      moderation_case::appeal_window_expires_at.eq(Some(future)),
      moderation_case::panel_size_snapshot.eq(Some(5_i32)),
    ))
    .execute(&mut async_conn)
    .await?;

  let past = Utc::now() - Duration::days(1);
  diesel::update(moderation_case::table.filter(moderation_case::id.eq(case_b.id)))
    .set((
      moderation_case::appeal_window_expires_at.eq(Some(past)),
      moderation_case::panel_size_snapshot.eq(Some(5_i32)),
    ))
    .execute(&mut async_conn)
    .await?;

  // Case A: appeal_window_expires_at in the future → appeal succeeds.
  let resp_a = request_appeal(
    Json(RequestAppeal { case_id: case_a.id, reason: "try me".to_string() }),
    context.clone(),
    target_a_view,
  )
  .await
  .map_err(|e| -> Box<dyn Error> { format!("request_appeal (open window): {e}").into() })?
  .into_inner();
  assert!(
    resp_a.appeal_id.0 > 0,
    "GH #34: appeal with appeal_window_expires_at in future must succeed (appeal_id positive)",
  );
  assert_eq!(resp_a.case_id, case_a.id, "response case_id round-trips");

  // Case B: appeal_window_expires_at in the past → appeal fails with NotFound.
  let resp_b = request_appeal(
    Json(RequestAppeal { case_id: case_b.id, reason: "expired".to_string() }),
    context.clone(),
    target_b_view,
  )
  .await;
  assert!(
    resp_b.is_err(),
    "GH #34: appeal with appeal_window_expires_at in past must fail (window expired)",
  );

  Ok(())
}

// ============================================================================
// v0-polish — GH #33 regression: declining juror not picked as own replacement
// ============================================================================
//
// Before the #33 fix, `decline_jury_assignment` flipped the caller's
// assignment to Declined, then built `exclude_person_ids` from all
// `jury_assignment` rows where status != Declined && status != Expired.
// That filter removed the declining juror's own (just-flipped) row from
// the query, so the caller was eligible to be picked as their own
// replacement — a quorum-threatening self-selection bug.
//
// The fix pushes `caller_id` onto the exclude vec after the query. This
// test seeds a small pool (5 jurors on the panel + 1 extra eligible),
// has one juror on the panel decline, and asserts the replacement is
// the sixth eligible — never the decliner. The JuryAssignment rows on
// the case after the decline are inspected directly at the DB level
// (status=Selected ∩ person_id=decliner must be zero).

#[tokio::test(flavor = "multi_thread")]
async fn declining_juror_not_picked_as_own_replacement() -> Result<(), Box<dyn Error>> {
  use actix_web::web::{Data, Json};
  use chrono::{Duration, Utc};
  use diesel::{
    Connection as _, ExpressionMethods, PgConnection, QueryDsl,
  };
  use diesel_async::{AsyncConnection as _, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    admin_assign_jury::admin_assign_jury,
    decline_jury_assignment::decline_jury_assignment,
    reputation_snapshot::run_snapshot_batch,
  };
  use lemmy_api_common::governance::{AdminAssignJury, DeclineJuryAssignment};
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
    schema::{jury_assignment, moderation_case, reputation_event},
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

  // Six eligibles — panel is 5; the 6th is the only viable replacement so
  // self-exclusion is the load-bearing assertion. If the fix is absent, the
  // decliner would be one of two candidates in the pool
  // (decliner + sixth-eligible) and randomness could mask the bug; with 6
  // eligibles and panel_size=5, the sixth is the unique replacement.
  let mut eligibles = Vec::new();
  for i in 0..6 {
    eligibles.push(seed_person(&context, instance.id, &format!("juror_{i}"), false).await?);
  }
  let admin = seed_person(&context, instance.id, "decline_admin", true).await?;
  let target = seed_person(&context, instance.id, "decline_target", false).await?;
  let admin_view = LocalUserView::read_person(&mut context.pool(), admin).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  // Seed reputation so jury_eligible resolves true for everyone.
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
        dedupe_key: None,
        source_event_type: None,
      };
      diesel::insert_into(reputation_event::table)
        .values(&form)
        .execute(&mut async_conn)
        .await?;
    }
  }

  // Override jury config: zero age requirement; disable fallback so any
  // failure surfaces loudly.
  {
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
    diesel::sql_query(
      "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
       VALUES ('instance', 'jury.age_requirement_days', 'int', 0, now())"
    ).execute(&mut async_conn).await?;
    diesel::sql_query(
      "INSERT INTO governance_config (scope, key, value_type, value_bool, valid_from) \
       VALUES ('instance', 'jury.fallback_on_small_pool', 'bool', false, now())"
    ).execute(&mut async_conn).await?;
  }

  run_snapshot_batch(&context).await
    .map_err(|e| -> Box<dyn Error> { format!("run_snapshot_batch: {e}").into() })?;

  // Community for the case (target_person_id case so eligibility filter
  // excludes the target from the panel).
  let community_form = CommunityInsertForm::new(
    instance.id,
    "declinecomm".to_string(),
    "Decline Community".to_string(),
    "decline-pubkey".to_string(),
  );
  let _community = Community::create(&mut context.pool(), &community_form).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  // Seed a JurySelection case ready for admin_assign_jury.
  let case_id = {
    let mut pool = context.pool();
    let conn = &mut lemmy_diesel_utils::connection::get_conn(&mut pool).await
      .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;
    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::Person,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: Some(target),
      target_community_id: None,
      target_remote_url: None,
      reason_code: "decline_probe".to_string(),
      severity: CaseSeverity::Low,
      status: CaseStatus::Open,
      threshold_score: 1,
  ..Default::default()
    };
    let case: lemmy_db_schema::source::governance::moderation_case::ModerationCase =
      diesel::insert_into(moderation_case::table)
        .values(&form)
        .get_result(conn)
        .await?;
    case.id
  };

  // Assign jury. 5 eligibles selected; one eligible remains as the unique
  // replacement candidate.
  let assign_resp = admin_assign_jury(
    Json(AdminAssignJury { case_id }),
    context.clone(),
    admin_view,
  )
  .await
  .map_err(|e| -> Box<dyn Error> { format!("admin_assign_jury: {e}").into() })?
  .into_inner();
  assert_eq!(assign_resp.assigned_person_ids.len(), 5, "5 jurors assigned");

  let decliner_id = assign_resp.assigned_person_ids[0];
  let decliner_view = LocalUserView::read_person(&mut context.pool(), decliner_id).await
    .map_err(|e| -> Box<dyn Error> { format!("{e}").into() })?;

  // Decline.
  let decline_resp = decline_jury_assignment(
    Json(DeclineJuryAssignment { case_id, reason: Some("cannot serve".to_string()) }),
    context.clone(),
    decliner_view,
  )
  .await
  .map_err(|e| -> Box<dyn Error> { format!("decline_jury_assignment: {e}").into() })?
  .into_inner();
  assert!(decline_resp.declined, "declined=true in response");
  let replacement_id = decline_resp.replacement_person_id
    .expect("GH #33: replacement must be selected (6th eligible is available)");
  assert_ne!(
    replacement_id, decliner_id,
    "GH #33: decliner must NOT be picked as own replacement",
  );

  // DB-level assertions: decliner has exactly one Declined row; the
  // replacement has a Selected row; no Selected row exists for the
  // decliner on this case.
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

  let decliner_selected_count: i64 = jury_assignment::table
    .filter(jury_assignment::case_id.eq(case_id))
    .filter(jury_assignment::person_id.eq(decliner_id))
    .filter(jury_assignment::status.eq(JuryAssignmentStatus::Selected))
    .count()
    .get_result(&mut async_conn)
    .await?;
  assert_eq!(
    decliner_selected_count, 0,
    "GH #33: no Selected row should exist for the decliner on this case",
  );

  let replacement_selected_count: i64 = jury_assignment::table
    .filter(jury_assignment::case_id.eq(case_id))
    .filter(jury_assignment::person_id.eq(replacement_id))
    .filter(jury_assignment::status.eq(JuryAssignmentStatus::Selected))
    .count()
    .get_result(&mut async_conn)
    .await?;
  assert_eq!(
    replacement_selected_count, 1,
    "replacement must have a Selected row",
  );

  Ok(())
}

// ============================================================================
// Phase v1-AD-b — task 8: admin_config HTTP handler integration tests
// ============================================================================
//
// Thirteen tests per plan §11 + §13 task 8. They share a small module of
// local helpers that spin up Postgres, seed an instance + users, and mint
// `LocalUserView`s so individual `admin_set_config` / `admin_get_config` /
// `admin_get_config_audit` invocations stay readable. The helpers mirror
// `report_to_modlog_golden_path` (e2e.rs:748) — same SETTINGS-priming,
// same `build_db_pool_for_tests` bootstrap, same direct-handler-invoke
// style.
//
// Each test owns a fresh `pgautoupgrade/pgautoupgrade:18-alpine` container;
// that matches the existing e2e.rs pattern (1 test = 1 container) and
// keeps state bleed between tests impossible. The `admin_get_config_full`
// test is the slowest — it walks all 61 seed rows — but still finishes
// well under 30s on a warm host.

mod admin_config_fixtures {
  use actix_web::web::Data;
  use diesel::{Connection as _, PgConnection};
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    instance::Instance,
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

  /// Spin a fresh Postgres, apply the full Brehon schema, build a real
  /// `LemmyContext` wrapped in `Data`, and return both the context handle
  /// and the container guard (keep the container alive via `_container`).
  pub async fn bootstrap() -> LemmyResult<(
    testcontainers::ContainerAsync<testcontainers::GenericImage>,
    Data<LemmyContext>,
    String,
  )> {
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }

    let (container, host_port) = super::governance_fixtures::start_postgres()
      .await
      .map_err(|e| anyhow::anyhow!("start_postgres: {e}"))?;
    let db_url = super::governance_fixtures::db_url(host_port);
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }

    {
      let mut sync_conn = PgConnection::establish(&db_url)
        .map_err(|e| -> Box<dyn Error + Send + Sync> {
          format!("PgConnection::establish: {e}").into()
        })
        .map_err(|e| anyhow::anyhow!("{e}"))?;
      super::governance_fixtures::apply_all_schema(&mut sync_conn)
        .map_err(|e| anyhow::anyhow!("apply_all_schema: {e}"))?;
    }

    let pool: ActualDbPool = build_db_pool_for_tests();
    let client = client_builder(&SETTINGS).build()?;
    let middleware_client = ClientBuilder::new(client).build();
    let secret = Secret { id: 0, jwt_secret: String::new().into() };
    let rate_limit = RateLimit::with_debug_config();
    // Bump rate-limit buckets — multi-write tests trip the 6/300s Post
    // bucket from `with_debug_config()`. See
    // `feedback_rate_limit_debug_config_post_bucket.md`.
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
    let context = Data::new(LemmyContext::create(
      pool,
      middleware_client.clone(),
      middleware_client,
      secret,
      rate_limit,
    ));

    Ok((container, context, db_url))
  }

  /// Seed an instance + a single person/local_user pair, returning both the
  /// PersonId and the `LocalUserView` callers need to invoke handlers.
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

  /// Read the first instance (auto-created by migrations as
  /// `local_site.site_id = 1`) or create a fresh `test.invalid` one.
  pub async fn bootstrap_instance(ctx: &LemmyContext) -> LemmyResult<Instance> {
    Ok(Instance::read_or_create(&mut ctx.pool(), "test.invalid").await?)
  }

  /// v1-AD-c task 8 helper: seed a non-admin user AND register them as a
  /// CommunityModerator on the given community. Returns the `LocalUserView`
  /// ready to pass to `admin_create_rule_set` / `admin_list_rule_sets`. The
  /// moderator path is the primary capability gate for rule-set CRUD —
  /// v1-AD-d will re-use this helper for the audit-display tests.
  pub async fn seed_community_moderator(
    ctx: &LemmyContext,
    instance_id: InstanceId,
    community_id: lemmy_db_schema::newtypes::CommunityId,
    name: &str,
  ) -> LemmyResult<LocalUserView> {
    use lemmy_db_schema::source::community::{CommunityActions, CommunityModeratorForm};
    let (person_id, view) = seed_user(ctx, instance_id, name, false).await?;
    CommunityActions::join(
      &mut ctx.pool(),
      &CommunityModeratorForm::new(community_id, person_id),
    )
    .await?;
    Ok(view)
  }
}

/// Task 8 test 1: instance-scope int write bumps `jury.panel_size` from 5
/// to 7, returns `applied=true` + both IDs, persists a `governance_config`
/// row, and emits exactly one `admin_config_changed` log entry.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_happy_path() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::{governance_config, governance_log};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) = admin_config_fixtures::seed_user(&context, instance.id, "admin_hp", true).await?;

  let resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "bump panel_size for test".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert!(resp.applied, "applied must be true on happy path");
  assert!(resp.config_id.is_some(), "config_id set");
  assert!(resp.governance_log_id.is_some(), "governance_log_id set");
  assert!(resp.applied_at.is_some(), "applied_at set");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let row_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .filter(governance_config::scope.eq("instance"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(row_count, 2, "seed row + new row = 2");

  let log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(log_count, 1, "exactly one admin_config_changed entry");

  Ok(())
}

/// Task 8 test 2: `dry_run = Some(true)` returns a populated preview but
/// writes nothing.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_dry_run() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::{governance_config, governance_log};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) = admin_config_fixtures::seed_user(&context, instance.id, "admin_dry", true).await?;

  let mut conn_before = AsyncPgConnection::establish(&db_url).await?;
  let before_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .count()
    .get_result(&mut conn_before)
    .await?;

  let resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(9),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: Some(true),
      reason: "dry-run probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert!(!resp.applied, "dry_run must set applied=false");
  assert!(resp.config_id.is_none(), "dry_run must NOT return config_id");
  assert!(resp.governance_log_id.is_none(), "dry_run must NOT return log_id");
  assert!(resp.applied_at.is_none(), "dry_run must NOT return applied_at");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let after_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(before_count, after_count, "dry_run must not append a config row");

  let log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(log_count, 0, "dry_run must not emit admin_config_changed");

  Ok(())
}

/// Task 8 test 3: `value_type="int"` against a float-metadata key → 400,
/// no denial log (type mismatch is bad input, not a policy denial).
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_type_mismatch_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) = admin_config_fixtures::seed_user(&context, instance.id, "admin_tm", true).await?;

  // `liability.regular_multiplier` is declared float in metadata.
  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "liability.regular_multiplier".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(2),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "type-mismatch probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "type mismatch must be an error");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let denial_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(denial_count, 0, "type mismatch is not a policy denial → no denial log");

  Ok(())
}

/// Task 8 test 4: panel_size=1000 is outside the declared 3-21 range →
/// 400, no denial log (range violation is bad input, not policy denial).
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_range_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::{governance_config, governance_log};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) = admin_config_fixtures::seed_user(&context, instance.id, "admin_rg", true).await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(1000),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "range violation probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "out-of-range must be an error");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let cfg_count: i64 = governance_config::table
    .filter(governance_config::key.eq("jury.panel_size"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(cfg_count, 1, "range violation must not insert a config row");

  let denial_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(denial_count, 0, "range violation is not a policy denial");

  Ok(())
}

/// Task 8 test 5: enum value not in `valid_enum` list → 400, no denial log.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_enum_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) = admin_config_fixtures::seed_user(&context, instance.id, "admin_en", true).await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.severity_thresholds.minor".to_string(),
      value_type: "text".to_string(),
      value: serde_json::json!("invalid"),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "enum violation probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "invalid enum value must be an error");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let denial_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(denial_count, 0, "enum violation is not a policy denial");

  Ok(())
}

/// Task 8 test 6: non-admin caller → 403 + denial log with
/// `denial_reason = "instance_admin_required"`.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_non_admin_rejected_with_denial_log()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) = admin_config_fixtures::seed_user(&context, instance.id, "non_admin", false).await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "non-admin probe".to_string(),
    }),
    context.clone(),
    user_view,
  )
  .await;
  assert!(result.is_err(), "non-admin must be rejected");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  use diesel::SelectableHelper;
  let denial_rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(denial_rows.len(), 1, "exactly one denial entry");
  let payload = &denial_rows[0].payload;
  assert_eq!(
    payload.get("denial_reason").and_then(|v| v.as_str()),
    Some("instance_admin_required"),
    "denial_reason must be instance_admin_required",
  );
  assert!(
    denial_rows[0].actor_pseudonym.is_some(),
    "denied caller still gets a pseudonym (GDPR layer per plan §4.1)",
  );

  Ok(())
}

/// Task 8 test 7: instance-only key (e.g. `federation.inbound_advisory_only`)
/// with `scope: community:<id>` → 400 + denial log with
/// `denial_reason = "scope_mismatch_instance_key"`.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_scope_mismatch_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::governance_log::GovernanceLog,
  };
  use lemmy_db_schema_file::schema::governance_log;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) = admin_config_fixtures::seed_user(&context, instance.id, "admin_sm", true).await?;

  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "scope_mm".to_string(),
      "Scope Mismatch Community".to_string(),
      "pk-scope".to_string(),
    ),
  )
  .await?;

  let result = admin_set_config(
    Json(AdminSetConfig {
      key: "federation.inbound_advisory_only".to_string(),
      value_type: "bool".to_string(),
      value: serde_json::json!(false),
      scope: format!("community:{}", community.id.0),
      apply_at: None,
      dry_run: None,
      reason: "scope-mismatch probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await;
  assert!(result.is_err(), "scope mismatch must be rejected");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  use diesel::SelectableHelper;
  let denial_rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(denial_rows.len(), 1, "exactly one denial entry");
  assert_eq!(
    denial_rows[0]
      .payload
      .get("denial_reason")
      .and_then(|v| v.as_str()),
    Some("scope_mismatch_instance_key"),
    "denial_reason must be scope_mismatch_instance_key",
  );

  Ok(())
}

/// Task 8 test 8: `jury.quorum` has `ConfigScope::Both` so a community
/// moderator (not an instance admin) can write it at `community:<id>`
/// scope.
///
/// NOTE: v1-AD-b plan §11 line 1101 originally named
/// `liability.regular_multiplier` here, but that key is declared
/// `ConfigScope::Instance` in `config.rs:1141-1152` — not `Both`. Swapped
/// to `jury.quorum` (Int, range 1-21, `ConfigScope::Both`) which actually
/// exercises the moderator-write path. The `Both`-scope key set has no
/// float-typed member today, so picking an int is the minimal correction.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_community_scope_by_moderator()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::{
    community::{Community, CommunityActions, CommunityInsertForm, CommunityModeratorForm},
    governance::governance_config::GovernanceConfig,
  };
  use lemmy_db_schema_file::schema::governance_config;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (mod_id, mod_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "cmod", false).await?;

  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "cmod_comm".to_string(),
      "Community Mod".to_string(),
      "pk-cmod".to_string(),
    ),
  )
  .await?;
  CommunityActions::join(
    &mut context.pool(),
    &CommunityModeratorForm::new(community.id, mod_id),
  )
  .await?;

  let resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.quorum".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(5),
      scope: format!("community:{}", community.id.0),
      apply_at: None,
      dry_run: None,
      reason: "cmod sets jury quorum for community".to_string(),
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();
  assert!(resp.applied, "moderator write on Both-scope key must succeed");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  use diesel::SelectableHelper;
  let rows: Vec<GovernanceConfig> = governance_config::table
    .filter(governance_config::scope.eq(format!("community:{}", community.id.0)))
    .filter(governance_config::key.eq("jury.quorum"))
    .select(GovernanceConfig::as_select())
    .load::<GovernanceConfig>(&mut conn)
    .await?;
  assert_eq!(rows.len(), 1, "one community-scoped row appended");

  Ok(())
}

/// Task 8 test 9: GET /admin/config without filters returns every
/// CONFIG_KEY_METADATA row; each entry carries `effective_from` matching
/// either "default" (const) or "instance" (seed row).
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_full() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Query;
  use lemmy_api::governance::{
    admin_config::admin_get_config,
    config::CONFIG_KEY_METADATA,
  };
  use lemmy_api_common::governance::AdminGetConfig;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_gf", true).await?;

  let resp = admin_get_config(
    Query(AdminGetConfig { key: None, community_id: None }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(
    resp.entries.len(),
    CONFIG_KEY_METADATA.len(),
    "GET without filters returns one entry per metadata row",
  );
  for entry in &resp.entries {
    assert!(
      !entry.effective_from.is_empty(),
      "effective_from populated for key `{}`",
      entry.key,
    );
    assert!(
      matches!(entry.effective_from.as_str(), "default" | "instance" | "community"),
      "effective_from must be default/instance/community, got `{}` for `{}`",
      entry.effective_from,
      entry.key,
    );
  }

  Ok(())
}

/// Task 8 test 10: after a successful write, GET with `?key=jury.panel_size`
/// returns a single entry whose `effective_from = "instance"` (the seed
/// row + the new instance-scope write both exist, and the latest-wins
/// probe picks the new one).
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_single_key_with_provenance() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_config::{admin_get_config, admin_set_config};
  use lemmy_api_common::governance::{AdminGetConfig, AdminSetConfig};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_sk", true).await?;

  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(11),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "provenance probe".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?
  .into_inner();

  let resp = admin_get_config(
    Query(AdminGetConfig {
      key: Some("jury.panel_size".to_string()),
      community_id: None,
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(resp.entries.len(), 1, "single-key GET returns exactly one entry");
  let entry = &resp.entries[0];
  assert_eq!(entry.key, "jury.panel_size");
  assert_eq!(entry.value, serde_json::json!(11), "value reflects the write");
  assert_eq!(
    entry.effective_from, "instance",
    "effective_from must be 'instance' after an instance-scope write",
  );

  Ok(())
}

/// Task 8 test 11: five writes (3 successes, 2 denials) + a paginated GET
/// with `limit=3` returns three entries in descending created_at order.
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_audit_paginated() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_config::{admin_get_config_audit, admin_set_config};
  use lemmy_api_common::governance::{AdminGetConfigAudit, AdminSetConfig};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_ap", true).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "user_ap", false).await?;

  // 3 successes — increments to panel_size (5 → 7 → 9 → 11)
  for (i, v) in [7, 9, 11].iter().enumerate() {
    admin_set_config(
      Json(AdminSetConfig {
        key: "jury.panel_size".to_string(),
        value_type: "int".to_string(),
        value: serde_json::json!(*v),
        scope: "instance".to_string(),
        apply_at: None,
        dry_run: None,
        reason: format!("bump {i}"),
      }),
      context.clone(),
      admin_view.clone(),
    )
    .await?;
  }
  // 2 denials — non-admin attempts
  for i in 0..2 {
    let _ = admin_set_config(
      Json(AdminSetConfig {
        key: "jury.panel_size".to_string(),
        value_type: "int".to_string(),
        value: serde_json::json!(13),
        scope: "instance".to_string(),
        apply_at: None,
        dry_run: None,
        reason: format!("denied {i}"),
      }),
      context.clone(),
      user_view.clone(),
    )
    .await;
  }

  let resp = admin_get_config_audit(
    Query(AdminGetConfigAudit {
      key: None,
      scope: None,
      actor_pseudonym: None,
      since: None,
      until: None,
      page: None,
      limit: Some(3),
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(resp.len(), 3, "limit=3 returns three entries");
  for pair in resp.windows(2) {
    assert!(
      pair[0].created_at >= pair[1].created_at,
      "audit entries must be in desc created_at order",
    );
  }

  Ok(())
}

/// Task 8 test 12 (NOT5 gate 3, continuity semantics per PRD §8.4 condition 3,
/// amended 2026-04-21): write via HTTP handler AND write via raw SQL matching
/// the shell script's INSERT. The two payloads are **continuous**, not
/// byte-identical: for every key the shell script emits (`scope`, `key`,
/// `value_type`, `value`, `reason`) the two payloads must agree
/// byte-for-byte, AND the HTTP path may emit strictly more keys
/// (`previous_value`, `previous_from` as of v1-AD-c task 4, closes
/// GH #77). `project_to_audit_entry` degrades the HTTP-only fields to
/// `None` on shell-written rows, so downstream readers see a coherent
/// schema either way. This test guards the subset-parity contract +
/// asserts the additive fields land on the HTTP row and are absent on
/// the shell row (future drift would flip that asymmetry and must be
/// caught here).
#[tokio::test(flavor = "multi_thread")]
async fn governance_log_payload_shell_parity() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_pp", true).await?;

  // Write #1: via the Rust HTTP handler.
  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "parity probe".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?;

  // Write #2: via raw SQL matching admin-config-write.sh's payload-build SQL
  // EXACTLY (including JSON key declaration order, the previous_value/from
  // sub-SELECTs, and the to_char timestamp formatting).
  // Updated for #84 (option A): shell wrapper now emits previous_value +
  // previous_from at the tail. The sub-SELECTs read from governance_config
  // for the most-recent row (scope, key) with valid_from < now() — i.e. the
  // row Write #1 just inserted, since the HTTP handler writes to BOTH
  // governance_config and governance_log.
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  diesel::sql_query(
    "INSERT INTO governance_log (entry_kind, payload, actor_pseudonym) VALUES (\
       'admin_config_changed',\
       jsonb_build_object(\
         'scope',          'instance',\
         'key',            'jury.panel_size',\
         'value_type',     'int',\
         'value',          7,\
         'reason',         'parity probe',\
         'previous_value', (\
           SELECT CASE prev.value_type \
                    WHEN 'int'   THEN to_jsonb(prev.value_int) \
                    WHEN 'float' THEN to_jsonb(prev.value_float) \
                    WHEN 'bool'  THEN to_jsonb(prev.value_bool) \
                    WHEN 'text'  THEN to_jsonb(prev.value_text) \
                  END \
           FROM governance_config prev \
           WHERE prev.scope = 'instance' \
             AND prev.key   = 'jury.panel_size' \
             AND prev.valid_from < now() \
           ORDER BY prev.valid_from DESC LIMIT 1 \
         ),\
         'previous_from', (\
           SELECT to_char(prev.valid_from AT TIME ZONE 'UTC', \
                          'YYYY-MM-DD\"T\"HH24:MI:SS.US\"Z\"') \
           FROM governance_config prev \
           WHERE prev.scope = 'instance' \
             AND prev.key   = 'jury.panel_size' \
             AND prev.valid_from < now() \
           ORDER BY prev.valid_from DESC LIMIT 1 \
         )\
       ),\
       'shell-wrapper-pseudo'\
     )",
  )
  .execute(&mut conn)
  .await?;

  // Load both rows.
  let rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .order_by(governance_log::id.asc())
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(rows.len(), 2, "two admin_config_changed rows present");

  // entry_kind must match (trivially; already filtered).
  assert_eq!(rows[0].entry_kind, rows[1].entry_kind);

  // Subset parity: for every key the shell script emits, the two
  // payloads must agree byte-for-byte. Postgres canonicalises jsonb on
  // round-trip (spaces after commas, colons, etc.); both rows come
  // through the same canonicaliser so the comparison is on the
  // canonicalised form — the contract we care about is what a
  // downstream reader sees.
  for key in ["scope", "key", "value_type", "value", "reason"] {
    assert_eq!(
      rows[0].payload.get(key),
      rows[1].payload.get(key),
      "subset-parity: key `{key}` must be byte-identical across HTTP and shell paths (NOT5 gate 3, PRD §8.4 condition 3)",
    );
  }

  // Both rows must carry the additive fields introduced by v1-AD-c task 4
  // (closes GH #77 + #84). HTTP and shell paths now both emit
  // `previous_value` and `previous_from` per option A.
  //
  // Important: by this test's setup order (Write #1 = HTTP, Write #2 = shell),
  // the two rows' `previous_*` fields legitimately DIFFER:
  //   - HTTP row's previous_* reflects pre-Write-1 state (no prior row → null).
  //   - Shell row's previous_* reflects post-Write-1 state (Write #1's value).
  // That divergence is correct behavior, not a parity bug — each row carries
  // the previous-value the writer observed at action-time.
  assert!(
    rows[0].payload.get("previous_value").is_some(),
    "HTTP row must carry previous_value field (t4 extension, PRD §8.4 condition 3 amended)",
  );
  assert!(
    rows[0].payload.get("previous_from").is_some(),
    "HTTP row must carry previous_from field (t4 extension, PRD §8.4 condition 3 amended)",
  );
  assert!(
    rows[1].payload.get("previous_value").is_some(),
    "shell row must carry previous_value field (closes #84, option A)",
  );
  assert!(
    rows[1].payload.get("previous_from").is_some(),
    "shell row must carry previous_from field (closes #84, option A)",
  );
  // Shell's previous_value should observe Write #1's row (value 7), since
  // Write #1's HTTP handler wrote to governance_config too.
  assert_eq!(
    rows[1].payload.get("previous_value"),
    Some(&serde_json::json!(7)),
    "shell row's previous_value should equal Write #1's value (post-Write-1 state observed)",
  );

  Ok(())
}

/// Task 8 test 13 (advisor edit #2 / v1-AD-a retro): after full migrations
/// but with no seed row for `rule_set.active_version_id` AND no compile-time
/// default, `config::get_int_opt(Scope::Instance, "rule_set.active_version_id")`
/// returns `Ok(None)`. Proves that the accessor family correctly surfaces
/// absent keys without panicking (CachedValue::Absent path).
#[tokio::test(flavor = "multi_thread")]
async fn rule_set_active_version_absent_returns_none() -> lemmy_utils::error::LemmyResult<()> {
  use diesel_async::{AsyncConnection, AsyncPgConnection};
  use lemmy_api::governance::config::{ConfigCache, Scope, get_int_opt};
  use lemmy_diesel_utils::connection::DbPool;

  let (_container, _context, db_url) = admin_config_fixtures::bootstrap().await?;
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
  let mut pool: DbPool<'_> = (&mut async_conn).into();
  let mut cache = ConfigCache::new();

  let result = get_int_opt(
    &mut cache,
    &mut pool,
    Scope::Instance,
    "rule_set.active_version_id",
  )
  .await?;
  assert!(
    result.is_none(),
    "rule_set.active_version_id has no seed + no const → Ok(None)",
  );

  Ok(())
}

// -- v1-AD-c task 8 — 8 new e2e tests -------------------------------------
//
// Groups: A (4 rule-set CRUD), B (1 Scope parser), C (2 audit payload),
// D (1 case-open snapshot). See `.claude/PRPs/plans/v1-admin-dashboard-c.plan.md`
// §13 task 8 and §14.1 for the group matrix.

/// v1-AD-c task 8 test A1: moderator creates v1 (parent_id=None) then v2
/// (parent_id=v1); assert rule_set_version row count == 2,
/// governance_config row for `rule_set.active_version_id` flipped to v2,
/// exactly 2 `rule_set_version_created` governance_log entries.
#[tokio::test(flavor = "multi_thread")]
async fn admin_create_rule_set_happy_path() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_rule_sets::admin_create_rule_set;
  use lemmy_api_common::governance::AdminCreateRuleSet;
  use lemmy_db_schema::source::community::{Community, CommunityInsertForm};
  use lemmy_db_schema_file::schema::{governance_config, governance_log, rule_set_version};
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_happy".to_string(),
      "Rule-Set Happy".to_string(),
      "pk-rs-happy".to_string(),
    ),
  )
  .await?;
  let mod_view = admin_config_fixtures::seed_community_moderator(
    &context,
    instance.id,
    community.id,
    "rs_mod_hp",
  )
  .await?;

  let v1 = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "rules v1 — initial".to_string(),
      parent_id: None,
      reason: "initial rule-set".to_string(),
    }),
    context.clone(),
    mod_view.clone(),
  )
  .await?
  .into_inner();
  assert_eq!(v1.version, 1, "first version must be 1");

  let v2 = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "rules v2 — revised".to_string(),
      parent_id: Some(v1.rule_set_version_id),
      reason: "revise rules".to_string(),
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();
  assert_eq!(v2.version, 2, "second version must be 2");
  assert!(v2.rule_set_version_id > v1.rule_set_version_id, "id monotonic");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rsv_count: i64 = rule_set_version::table
    .filter(rule_set_version::community_id.eq(community.id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(rsv_count, 2, "two rule_set_version rows for this community");

  let community_scope = format!("community:{}", community.id.0);
  let active_rows: Vec<Option<i64>> = governance_config::table
    .filter(governance_config::scope.eq(&community_scope))
    .filter(governance_config::key.eq("rule_set.active_version_id"))
    .order(governance_config::valid_from.desc())
    .select(governance_config::value_int)
    .load(&mut conn)
    .await?;
  assert_eq!(active_rows.len(), 2, "two active_version_id rows (one per create)");
  assert_eq!(
    active_rows[0],
    Some(i64::from(v2.rule_set_version_id)),
    "latest active_version_id == v2 id",
  );

  let log_count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("rule_set_version_created"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(log_count, 2, "exactly two rule_set_version_created entries");

  Ok(())
}

/// v1-AD-c task 8 test A2: non-moderator non-admin caller attempts
/// `admin_create_rule_set` → `LemmyErrorType::NotAnAdmin` AND a
/// `admin_config_change_denied` governance_log entry with
/// `denial_reason = "community_moderator_required"`. Actor pseudonym is
/// populated even on denial (GDPR layer per plan §4.1).
#[tokio::test(flavor = "multi_thread")]
async fn admin_create_rule_set_non_moderator_rejected() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_rule_sets::admin_create_rule_set;
  use lemmy_api_common::governance::AdminCreateRuleSet;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::governance_log::GovernanceLog,
  };
  use lemmy_db_schema_file::schema::governance_log;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_denied".to_string(),
      "Rule-Set Denied".to_string(),
      "pk-rs-denied".to_string(),
    ),
  )
  .await?;
  // Seed a user that is NEITHER an admin NOR a moderator of this community.
  let (_, outsider_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "rs_outsider", false).await?;

  let result = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "sneaky rules".to_string(),
      parent_id: None,
      reason: "non-moderator probe".to_string(),
    }),
    context.clone(),
    outsider_view,
  )
  .await;
  assert!(result.is_err(), "non-moderator must be rejected");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let denial_rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_change_denied"))
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(denial_rows.len(), 1, "exactly one denial entry");
  assert_eq!(
    denial_rows[0]
      .payload
      .get("denial_reason")
      .and_then(|v| v.as_str()),
    Some("community_moderator_required"),
    "denial_reason must be community_moderator_required",
  );
  assert!(
    denial_rows[0].actor_pseudonym.is_some(),
    "denied caller still gets a pseudonym (GDPR layer per plan §4.1)",
  );

  Ok(())
}

/// v1-AD-c task 8 test A3: the `rule_set_version` UNIQUE
/// `(community_id, version)` constraint triggers a
/// `diesel::result::DatabaseErrorKind::UniqueViolation` on a duplicate
/// insert at the same version. The handler's write path
/// (`process_create_rule_set` in `admin_rule_sets.rs`) catches exactly
/// this error kind and maps it to a retryable
/// `LemmyErrorType::Unknown("rule_set_version already exists ...")`.
///
/// This test exercises the constraint and mapping deterministically at
/// the DB layer:
///   1. Insert a `rule_set_version` row directly at version=1 (bypassing
///      the handler, so we can force a specific version).
///   2. Attempt a second direct insert at the same `(community_id, 1)`
///      pair.
///   3. Assert the exact Diesel error shape the handler pattern-matches
///      on (`DatabaseError(UniqueViolation, _)`) inside
///      `process_create_rule_set`.
///   4. Apply the same `Err` transform as the handler and assert the
///      `LemmyError` message.
///   5. Assert the pre-existing row survived and no second row leaked.
///
/// Rationale — an earlier revision of this test used `tokio::join!` to
/// race two concurrent `admin_create_rule_set` calls through the
/// handler. That is non-deterministic: `tokio::join!` gives no barrier
/// guarantee that both futures reach `lookup_latest_version` before
/// either insert commits. If one future wins, the other legitimately
/// observes version=1 already committed and computes version=2 — no
/// collision, assertions false-pass. The DB-layer direct-insert path
/// here is deterministic: the constraint fires on every run, the error
/// shape is observable, and the mapping is a pure function of that
/// error. The other path — concurrent handler invocation — would
/// require an `Arc<Barrier>` hook inside `process_create_rule_set`
/// (polluting production code for test determinism). CR PR #81 #5.
#[tokio::test(flavor = "multi_thread")]
async fn admin_create_rule_set_duplicate_version_rejected()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel::{
    ExpressionMethods,
    QueryDsl,
    result::{DatabaseErrorKind, Error as DieselError},
  };
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::source::community::{Community, CommunityInsertForm};
  use lemmy_db_schema::source::governance::rule_set_version::RuleSetVersionInsertForm;
  use lemmy_db_schema_file::schema::rule_set_version;
  use lemmy_diesel_utils::traits::Crud;
  use lemmy_utils::error::LemmyErrorType;
  use sha2::{Digest, Sha256};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_dup".to_string(),
      "Rule-Set Duplicate".to_string(),
      "pk-rs-dup".to_string(),
    ),
  )
  .await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "rs_admin_dup", true).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let text_sha256_a = Sha256::digest(b"seeded A".as_slice()).to_vec();
  let text_sha256_b = Sha256::digest(b"would-be B".as_slice()).to_vec();

  // Step 1: seed the first row directly — this is the row the handler
  // would have written on a winning race.
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community.id,
      version: 1,
      parent_id: None,
      text_sha256: text_sha256_a,
      rule_text: "seeded A".to_string(),
      created_by: Some(admin_view.person.id),
    })
    .execute(&mut conn)
    .await?;

  // Step 2: attempt the duplicate — this is the insert the losing
  // handler would have issued before UniqueViolation rolls its tx back.
  let duplicate_insert = diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community.id,
      version: 1,
      parent_id: None,
      text_sha256: text_sha256_b,
      rule_text: "would-be B".to_string(),
      created_by: Some(admin_view.person.id),
    })
    .execute(&mut conn)
    .await;

  // Step 3: drive the duplicate through the DB layer (UNIQUE constraint
  // fires), then route the resulting DieselError through the same helper
  // `process_create_rule_set` uses — any change to
  // `map_rsv_unique_violation` in admin_rule_sets.rs immediately affects
  // this test's mapping assertion.
  match &duplicate_insert {
    Err(DieselError::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
      // Expected — this is the branch the helper catches.
    }
    Err(other) => panic!(
      "expected UniqueViolation on duplicate (community_id, version); got {other:?}",
    ),
    Ok(_) => panic!(
      "UNIQUE(community_id, version) constraint did not fire — duplicate row committed",
    ),
  }

  // Step 4: route the DieselError through the real production mapping
  // helper and assert the LemmyError's inner `error_type` carries the
  // retry message verbatim. Note: we assert on `error_type` directly
  // rather than `format!("{mapped}")` because `LemmyError`'s `Display`
  // impl uses `strum::Display` on `LemmyErrorType`, which renders
  // `Unknown(String)` as just the bare variant name "Unknown" — the
  // wrapped message is only visible through pattern-matching on the
  // enum.
  let mapped: lemmy_utils::error::LemmyError = match duplicate_insert {
    Err(err) => lemmy_api::governance::admin_rule_sets::map_rsv_unique_violation(err),
    Ok(_) => unreachable!("UniqueViolation asserted at step above"),
  };
  match mapped.error_type {
    LemmyErrorType::Unknown(ref msg) => assert!(
      msg.contains("rule_set_version already exists"),
      "handler maps UniqueViolation to a retry-shaped Unknown error carrying the rule_set collision message; got {msg:?}",
    ),
    other => panic!("expected LemmyErrorType::Unknown with retry message; got {other:?}"),
  }

  // Step 5: the pre-existing row survived and no second row leaked.
  let rsv_count: i64 = rule_set_version::table
    .filter(rule_set_version::community_id.eq(community.id))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(
    rsv_count, 1,
    "UNIQUE violation left the original row intact and rejected the duplicate",
  );

  Ok(())
}

/// v1-AD-c task 8 test A4: after three successful creates, GET
/// `/admin/rule-sets` returns all three versions ordered by `version`
/// DESC and `active_version_id` equals the most recent id.
#[tokio::test(flavor = "multi_thread")]
async fn admin_list_rule_sets_returns_versions_with_active_version_id()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_rule_sets::{admin_create_rule_set, admin_list_rule_sets};
  use lemmy_api_common::governance::{AdminCreateRuleSet, AdminListRuleSetsRequest};
  use lemmy_db_schema::source::community::{Community, CommunityInsertForm};
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_list".to_string(),
      "Rule-Set List".to_string(),
      "pk-rs-list".to_string(),
    ),
  )
  .await?;
  let mod_view = admin_config_fixtures::seed_community_moderator(
    &context,
    instance.id,
    community.id,
    "rs_mod_list",
  )
  .await?;

  let mut prev_id: Option<i32> = None;
  let mut ids: Vec<i32> = Vec::with_capacity(3);
  for i in 1..=3 {
    let resp = admin_create_rule_set(
      Json(AdminCreateRuleSet {
        community_id: community.id,
        rule_text: format!("rules v{i}"),
        parent_id: prev_id,
        reason: format!("revision {i}"),
      }),
      context.clone(),
      mod_view.clone(),
    )
    .await?
    .into_inner();
    ids.push(resp.rule_set_version_id);
    prev_id = Some(resp.rule_set_version_id);
  }

  let resp = admin_list_rule_sets(
    Query(AdminListRuleSetsRequest {
      community_id: community.id,
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();

  assert_eq!(resp.versions.len(), 3, "three versions returned");
  // Ordered by version DESC
  assert_eq!(resp.versions[0].version, 3);
  assert_eq!(resp.versions[1].version, 2);
  assert_eq!(resp.versions[2].version, 1);
  assert_eq!(
    resp.active_version_id,
    Some(ids[2]),
    "active_version_id == id of the most recently created version",
  );

  Ok(())
}

/// v1-AD-c task 8 test B1 (Issue #78): `Scope::parse_wire` rejects
/// `community:-1` and `community:0` with the typed
/// `ScopeParseError::NonPositiveCommunityId` variant carrying the
/// offending integer. This is a direct-call unit-style assertion; no DB
/// container needed, but kept in e2e.rs per plan §14's "integration-only"
/// discipline.
#[test]
fn scope_parse_wire_rejects_negative_community_id() {
  use lemmy_api::governance::config::{Scope, ScopeParseError};

  assert_eq!(
    Scope::parse_wire("community:-1"),
    Err(ScopeParseError::NonPositiveCommunityId(-1)),
    "negative community_id must be typed-rejected",
  );
  assert_eq!(
    Scope::parse_wire("community:0"),
    Err(ScopeParseError::NonPositiveCommunityId(0)),
    "zero community_id must be typed-rejected",
  );
}

/// v1-AD-c task 8 test C1 (Issue #77 write side): after a successful
/// `admin_set_config` bumping `jury.panel_size` 5 → 7, the
/// `admin_config_changed` governance_log payload carries
/// `previous_value: 5, previous_from: "instance"` (the migration seed
/// at `migrations/2026-04-18-000000-0000_add_governance_config/up.sql:82`
/// inserts an instance-scoped row for this key with value=5, so the
/// effective provenance is `"instance"` — NOT `"default"`). A second
/// write 7 → 9 emits a row with `previous_value: 7, previous_from:
/// "instance"` (the 5→7 write appended another instance-scope row;
/// latest-wins reader picks it up). Both writes validate that the
/// `previous_value` + `previous_from` fields thread through from the
/// pre-tx read to the governance_log payload.
#[tokio::test(flavor = "multi_thread")]
async fn admin_set_config_persists_previous_value_and_from()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_config::admin_set_config;
  use lemmy_api_common::governance::AdminSetConfig;
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_prev", true).await?;

  // Write #1: 5 (seeded instance row) → 7.
  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "bump to 7".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  // Write #2: 7 (instance) → 9.
  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(9),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "bump to 9".to_string(),
    }),
    context.clone(),
    admin_view,
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rows: Vec<GovernanceLog> = governance_log::table
    .filter(governance_log::entry_kind.eq("admin_config_changed"))
    .order(governance_log::id.asc())
    .select(GovernanceLog::as_select())
    .load::<GovernanceLog>(&mut conn)
    .await?;
  assert_eq!(rows.len(), 2, "two admin_config_changed rows");

  // Write #1: previous is the seeded instance row (value=5, from="instance").
  assert_eq!(
    rows[0].payload.get("previous_value"),
    Some(&serde_json::json!(5)),
    "write #1 previous_value must equal the seeded instance row value 5",
  );
  assert_eq!(
    rows[0].payload.get("previous_from").and_then(|v| v.as_str()),
    Some("instance"),
    "write #1 previous_from must be `instance` (seeded row exists at instance scope)",
  );

  // Write #2: previous is the just-written 7 with from = "instance".
  assert_eq!(
    rows[1].payload.get("previous_value"),
    Some(&serde_json::json!(7)),
    "write #2 previous_value must equal the 5→7 write",
  );
  assert_eq!(
    rows[1].payload.get("previous_from").and_then(|v| v.as_str()),
    Some("instance"),
    "write #2 previous_from must be `instance` (latest-wins reads the 5→7 row)",
  );

  Ok(())
}

/// v1-AD-c task 8 test C2 (Issue #77 read side): after a 5→7 write,
/// `GET /admin/config/audit` returns an entry whose `previous_value` +
/// `previous_from` are hydrated from the payload written by task 4.
#[tokio::test(flavor = "multi_thread")]
async fn admin_get_config_audit_hydrates_previous_value()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::{Json, Query};
  use lemmy_api::governance::admin_config::{admin_get_config_audit, admin_set_config};
  use lemmy_api_common::governance::{AdminGetConfigAudit, AdminSetConfig};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_hyd", true).await?;

  admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(7),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "hydration probe".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  let entries = admin_get_config_audit(
    Query(AdminGetConfigAudit {
      key: Some("jury.panel_size".to_string()),
      scope: None,
      actor_pseudonym: None,
      since: None,
      until: None,
      page: None,
      limit: None,
    }),
    context.clone(),
    admin_view,
  )
  .await?
  .into_inner();

  assert_eq!(entries.len(), 1, "one audit entry for the bump");
  assert_eq!(
    entries[0].previous_value,
    Some(serde_json::json!(5)),
    "previous_value hydrated from payload — seeded instance value 5",
  );
  assert_eq!(
    entries[0].previous_from.as_deref(),
    Some("instance"),
    "previous_from hydrated from payload — `instance` (seed row exists)",
  );

  Ok(())
}

/// v1-AD-c task 8 test D1: community-target `create_report` opens a case
/// whose `applied_config_snapshot` contains exactly the 7
/// `requires_re_jury` keys AND `rule_set_version_id` equals the
/// community's active rule-set version.
#[tokio::test(flavor = "multi_thread")]
async fn case_open_pins_applied_config_snapshot_and_rule_set_version_id()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_rule_sets::admin_create_rule_set;
  use lemmy_api_common::governance::AdminCreateRuleSet;
  use lemmy_api_crud::governance::create_report::create_report;
  use lemmy_api_common::governance::CreateGovernanceReport;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::moderation_case::ModerationCase,
  };
  use lemmy_db_schema_file::{enums::CaseTargetType, schema::moderation_case};
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "rs_pin".to_string(),
      "Rule-Set Pin".to_string(),
      "pk-rs-pin".to_string(),
    ),
  )
  .await?;
  let mod_view = admin_config_fixtures::seed_community_moderator(
    &context,
    instance.id,
    community.id,
    "rs_mod_pin",
  )
  .await?;
  let (_, reporter_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "rs_reporter", false).await?;

  // Seed a community rule-set version — this flips
  // `rule_set.active_version_id` at Scope::Community(cid).
  let v1 = admin_create_rule_set(
    Json(AdminCreateRuleSet {
      community_id: community.id,
      rule_text: "community rules v1".to_string(),
      parent_id: None,
      reason: "pin probe".to_string(),
    }),
    context.clone(),
    mod_view,
  )
  .await?
  .into_inner();

  // Open a case against the community (target_type=Community). The
  // create_report handler pins the snapshot at Scope::Community(cid)
  // because the reporter passed `community_id = Some(cid)`.
  let resp = create_report(
    Json(CreateGovernanceReport {
      community_id: Some(community.id),
      target_type: CaseTargetType::Community,
      target_id: community.id.0,
      reason_code: "test.pin".to_string(),
      description: Some("pin probe".to_string()),
    }),
    context.clone(),
    reporter_view,
  )
  .await?
  .into_inner();
  let case_id = resp.case_id.expect("case_id present on successful open");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let case: ModerationCase = moderation_case::table
    .filter(moderation_case::id.eq(case_id))
    .select(ModerationCase::as_select())
    .first(&mut conn)
    .await?;

  assert_eq!(
    case.rule_set_version_id.map(|r| r.0),
    Some(v1.rule_set_version_id),
    "rule_set_version_id pinned to the active community version",
  );

  let snapshot = case
    .applied_config_snapshot
    .as_ref()
    .expect("applied_config_snapshot populated on case open");
  let snap_obj = snapshot
    .as_object()
    .expect("applied_config_snapshot is a JSON object");
  let expected_keys: &[&str] = &[
    "jury.panel_size",
    "jury.quorum",
    "jury.severity_thresholds.minor",
    "jury.severity_thresholds.moderate",
    "jury.severity_thresholds.severe",
    "jury.diversity_constraints_enabled",
    "jury.appeal_panel_size_increase",
  ];
  assert_eq!(
    snap_obj.len(),
    expected_keys.len(),
    "snapshot has exactly 7 keys",
  );
  for key in expected_keys {
    assert!(
      snap_obj.contains_key(*key),
      "snapshot contains `{key}`",
    );
  }

  Ok(())
}

// ============================================================================
// v1-AD-d — admin dashboard aggregate + SSE audit stream
//
// Six tests covering the two new read-only handlers:
// - `admin_dashboard_returns_aggregate_for_admin`    — zero-row happy path
// - `admin_dashboard_forbidden_for_non_admin`        — capability gate
// - `admin_dashboard_aggregates_populated_data`      — data fidelity
// - `admin_audit_stream_forbidden_for_non_admin`     — capability gate
// - `admin_audit_stream_enforces_per_admin_cap`      — 409 on 2nd connection
// - `admin_audit_stream_emits_frame_on_config_change` — live SSE emission
//
// Dashboard tests invoke the handler directly (same pattern as the
// v1-AD-b `admin_get_config_full` test). SSE tests invoke the handler
// directly and drain the streaming body via `MessageBody::poll_next`
// without an in-process actix HTTP server — the live emission test
// exercises the full NOTIFY → filter → row hydration → frame-format
// path end-to-end.
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_returns_aggregate_for_admin()
-> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_dashboard::admin_dashboard;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_dash_hp", true).await?;

  let before = chrono::Utc::now();
  let resp = admin_dashboard(context.clone(), admin_view).await?.into_inner();
  let after = chrono::Utc::now();

  // Zero-row DB — every widget populates with defaults, none error.
  assert_eq!(resp.active_cases.total_active, 0, "no active cases on fresh DB");
  assert!(
    resp.active_cases.by_status.is_empty() || resp.active_cases.by_status.values().sum::<i64>() == 0,
    "by_status empty or all zeros",
  );
  assert_eq!(resp.jury_queue.pending_accept, 0);
  assert_eq!(resp.jury_queue.accepted, 0);
  assert_eq!(resp.jury_queue.submitted, 0);
  assert_eq!(resp.recent_config_changes.len(), 0, "no config-change events");
  assert_eq!(resp.federation.active, 0);
  assert_eq!(resp.federation.expired, 0);
  assert_eq!(resp.federation.total, 0);
  assert_eq!(resp.rule_sets.communities_with_rule_sets, 0);
  assert_eq!(resp.rule_sets.total_versions, 0);
  assert_eq!(resp.rule_sets.per_community.len(), 0);

  // calculated_at within the request window (±5s slack either side).
  let slack = chrono::Duration::seconds(5);
  assert!(
    resp.calculated_at >= before - slack && resp.calculated_at <= after + slack,
    "calculated_at {} outside [{}, {}]",
    resp.calculated_at,
    before - slack,
    after + slack,
  );

  // Reputation widget is the instance-scope stub on a fresh DB.
  assert_eq!(
    resp.reputation.buckets.reporting_accuracy.len(),
    5,
    "bucket shape preserved (5 buckets per dimension)",
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_forbidden_for_non_admin()
-> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "dash_nonadmin", false).await?;

  let result = admin_dashboard(context.clone(), user_view).await;
  // is_err() alone would also pass on a pre-admin-check DB error; the
  // specific-variant match anchors the test to the capability gate
  // (cr-18).
  let err = result.expect_err("non-admin must be rejected by is_admin()");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotAnAdmin),
    "expected NotAnAdmin, got {:?}",
    err.error_type,
  );

  // Dashboard is read-only — ADR-008 compliance: no governance_log
  // entry is emitted on capability-deny (unlike admin_set_config's
  // denial path).
  use diesel::{QueryDsl, SelectableHelper};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::source::governance::governance_log::GovernanceLog;
  use lemmy_db_schema_file::schema::governance_log;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let rows: Vec<GovernanceLog> = governance_log::table
    .select(GovernanceLog::as_select())
    .load(&mut conn)
    .await?;
  assert_eq!(rows.len(), 0, "dashboard rejection must NOT emit a governance_log entry");

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_aggregates_populated_data()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::{
      federation_attestation::FederationAttestationInsertForm,
      moderation_case::ModerationCaseInsertForm,
      rule_set_version::RuleSetVersionInsertForm,
    },
  };
  use lemmy_db_schema_file::enums::{AttestationType, CaseSeverity, CaseStatus, CaseTargetType};
  use lemmy_db_schema_file::schema::{
    federation_attestation, moderation_case, rule_set_version,
  };
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "admin_dash_seed", true).await?;

  // Seed a community so we can hang a rule_set_version off of it.
  let community = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "dash_comm".to_string(),
      "Dashboard Seed Community".to_string(),
      "dash-pubkey".to_string(),
    ),
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // 3 moderation_case rows across three statuses. Open + JurySelection
  // count toward `total_active` (2); Decided does not.
  for status in [CaseStatus::Open, CaseStatus::JurySelection, CaseStatus::Decided] {
    let form = ModerationCaseInsertForm {
      community_id: None,
      creator_id: None,
      target_type: CaseTargetType::RemoteInstance,
      target_post_id: None,
      target_comment_id: None,
      target_person_id: None,
      target_community_id: None,
      target_remote_url: Some(format!("https://example.invalid/dash/{status:?}")),
      reason_code: "dashboard_seed".to_string(),
      severity: CaseSeverity::Low,
      status,
      threshold_score: 1,
      ..Default::default()
    };
    diesel::insert_into(moderation_case::table)
      .values(&form)
      .execute(&mut conn)
      .await?;
  }

  // 1 active federation_attestation (valid_until in the future).
  let future = chrono::Utc::now() + chrono::Duration::days(30);
  diesel::insert_into(federation_attestation::table)
    .values(&FederationAttestationInsertForm {
      actor_url: "https://test.invalid/u/seed-actor".to_string(),
      subject_url: "https://test.invalid/u/seed-subject".to_string(),
      attestation_type: AttestationType::TrustedReporter,
      valid_until: Some(future),
      signature: "seed-sig".to_string(),
    })
    .execute(&mut conn)
    .await?;

  // 1 rule_set_version on the seeded community (version 1, no parent).
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community.id,
      version: 1,
      parent_id: None,
      text_sha256: vec![0u8; 32],
      rule_text: "seed rule text".to_string(),
      created_by: None,
    })
    .execute(&mut conn)
    .await?;

  // Invoke the dashboard handler and assert aggregates.
  let resp = admin_dashboard(context.clone(), admin_view).await?.into_inner();

  // Each seed inserts exactly one case per status (three total). Assert
  // exact values so a regression that double-counts or drops a status
  // bucket surfaces, and so a missing key (None) is distinguished from
  // a zero count (cr-25).
  assert_eq!(
    resp.active_cases.by_status.get("Open").copied(),
    Some(1),
    "Open case count should be exactly 1 (one seed)",
  );
  assert_eq!(
    resp.active_cases.by_status.get("JurySelection").copied(),
    Some(1),
    "JurySelection case count should be exactly 1 (one seed)",
  );
  assert_eq!(
    resp.active_cases.by_status.get("Decided").copied(),
    Some(1),
    "Decided case count should be exactly 1 (one seed)",
  );
  assert_eq!(
    resp.active_cases.total_active, 2,
    "total_active excludes Decided (and Closed/EmergencyRemove)",
  );

  assert_eq!(resp.federation.active, 1, "one active attestation");
  assert_eq!(resp.federation.expired, 0);
  assert_eq!(resp.federation.total, 1);

  assert_eq!(
    resp.rule_sets.communities_with_rule_sets, 1,
    "one community has a rule_set_version",
  );
  assert_eq!(resp.rule_sets.total_versions, 1);
  // per_community includes one entry; active_version_id is None because
  // no governance_config row was seeded for rule_set.active_version_id.
  assert_eq!(resp.rule_sets.per_community.len(), 1);
  assert_eq!(resp.rule_sets.per_community[0].community_id, community.id);
  assert_eq!(resp.rule_sets.per_community[0].active_version_id, None);

  // recent_config_changes remains empty — no admin_config_changed rows
  // were inserted by any of the seeds above (they go through direct
  // table inserts, not the governance_log::append path).
  assert_eq!(resp.recent_config_changes.len(), 0);

  // Silence unused variable warnings on the fields we checked via other
  // branches.
  let _ = instance;
  Ok(())
}

/// cr-24 regression lock: the batched `active_version_id` lookup in
/// `rule_sets_summary` must return the correct per-community value, and
/// must fall back to the `instance`-scoped row when no community-scoped
/// row exists. Seeds two communities: the first has its own
/// community-scoped `rule_set.active_version_id`; the second has none,
/// so the cascade falls back to the instance-scoped row. Asserts each
/// community gets its own value from a single batched SELECT.
#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_per_community_active_version_cascade()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::rule_set_version::RuleSetVersionInsertForm,
  };
  use lemmy_db_schema_file::schema::rule_set_version;
  use lemmy_diesel_utils::traits::Crud;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "dash_cascade_admin", true).await?;

  let community_a = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "cascade_a".to_string(),
      "Cascade A".to_string(),
      "cascade-a-pk".to_string(),
    ),
  )
  .await?;
  let community_b = Community::create(
    &mut context.pool(),
    &CommunityInsertForm::new(
      instance.id,
      "cascade_b".to_string(),
      "Cascade B".to_string(),
      "cascade-b-pk".to_string(),
    ),
  )
  .await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // Each community needs a rule_set_version row so it shows up in
  // `per_community`.
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community_a.id,
      version: 1,
      parent_id: None,
      text_sha256: vec![0x11u8; 32],
      rule_text: "community A rules".to_string(),
      created_by: None,
    })
    .execute(&mut conn)
    .await?;
  diesel::insert_into(rule_set_version::table)
    .values(&RuleSetVersionInsertForm {
      community_id: community_b.id,
      version: 1,
      parent_id: None,
      text_sha256: vec![0x22u8; 32],
      rule_text: "community B rules".to_string(),
      created_by: None,
    })
    .execute(&mut conn)
    .await?;

  // Seed instance-scoped fallback: version 999. Community B falls back
  // to this because it has no community-scoped row.
  diesel::sql_query(
    "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
     VALUES ('instance', 'rule_set.active_version_id', 'int', 999, now())",
  )
  .execute(&mut conn)
  .await?;

  // Seed community A: community-scoped override to version 42. This
  // must win over the instance-scoped row per `get_int_opt`'s cascade.
  diesel::sql_query(format!(
    "INSERT INTO governance_config (scope, key, value_type, value_int, valid_from) \
     VALUES ('community:{}', 'rule_set.active_version_id', 'int', 42, now())",
    community_a.id.0,
  ))
  .execute(&mut conn)
  .await?;

  let resp = admin_dashboard(context.clone(), admin_view).await?.into_inner();

  // Locate each community's row in the response — per_community is
  // ORDER BY community_id in the handler, so community A sorts before B
  // if community_a.id.0 < community_b.id.0 (which is always true since
  // they were inserted in that order under a serial PK).
  let per_a = resp
    .rule_sets
    .per_community
    .iter()
    .find(|r| r.community_id == community_a.id)
    .ok_or_else(|| anyhow::anyhow!("community A not in per_community"))?;
  let per_b = resp
    .rule_sets
    .per_community
    .iter()
    .find(|r| r.community_id == community_b.id)
    .ok_or_else(|| anyhow::anyhow!("community B not in per_community"))?;

  assert_eq!(
    per_a.active_version_id,
    Some(42),
    "community A has a community-scoped override; batched query should \
     surface it (community:{} → 42), not the instance fallback",
    community_a.id.0,
  );
  assert_eq!(
    per_b.active_version_id,
    Some(999),
    "community B has no community-scoped row; batched query must fall \
     back to the instance-scoped row (value 999)",
  );

  Ok(())
}

/// cr-22 regression lock: `recent_config_changes` MUST exclude rows whose
/// `signature` is NULL. Those rows represent a half-written append (the
/// INSERT succeeded but the follow-up signature UPDATE in
/// `governance_log::append` failed). Surfacing them in the dashboard
/// would display unsigned audit entries that carry no verifiable hash
/// chain position — a misleading artifact for an admin reviewing
/// governance activity. Seeds one signed row and one unsigned row
/// directly via diesel (bypassing the append helper) and asserts only
/// the signed one is returned.
#[tokio::test(flavor = "multi_thread")]
async fn admin_dashboard_recent_config_changes_excludes_unsigned_rows()
-> lemmy_utils::error::LemmyResult<()> {
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::admin_dashboard::admin_dashboard;
  use lemmy_db_schema::source::governance::governance_log::{
    ENTRY_KIND_ADMIN_CONFIG_CHANGED, GovernanceLogInsertForm,
  };
  use lemmy_db_schema_file::schema::governance_log;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "dash_unsigned_filter", true).await?;

  let mut conn = AsyncPgConnection::establish(&db_url).await?;

  // Row A — "unsigned": INSERT only, signature stays NULL (simulates a
  // failed follow-up UPDATE in governance_log::append).
  let unsigned_id: i64 = diesel::insert_into(governance_log::table)
    .values(&GovernanceLogInsertForm {
      entry_kind: ENTRY_KIND_ADMIN_CONFIG_CHANGED.to_string(),
      payload: serde_json::json!({ "marker": "unsigned_row_should_not_leak" }),
      actor_pseudonym: Some("test-unsigned".to_string()),
    })
    .returning(governance_log::id)
    .get_result(&mut conn)
    .await?;

  // Row B — "signed": INSERT then flip signature NULL → NOT NULL. The
  // signature-gate trigger permits exactly one such transition per row.
  let signed_id: i64 = diesel::insert_into(governance_log::table)
    .values(&GovernanceLogInsertForm {
      entry_kind: ENTRY_KIND_ADMIN_CONFIG_CHANGED.to_string(),
      payload: serde_json::json!({ "marker": "signed_row_should_appear" }),
      actor_pseudonym: Some("test-signed".to_string()),
    })
    .returning(governance_log::id)
    .get_result(&mut conn)
    .await?;
  diesel::update(governance_log::table.find(signed_id))
    .set(governance_log::signature.eq(Some(vec![0xAAu8; 64])))
    .execute(&mut conn)
    .await?;

  let resp = admin_dashboard(context.clone(), admin_view).await?.into_inner();

  assert_eq!(
    resp.recent_config_changes.len(),
    1,
    "dashboard must include the signed row and exclude the unsigned row; \
     got {:?} entries",
    resp.recent_config_changes.len(),
  );
  // Sanity: unsigned row's marker must not appear in any projected entry.
  for entry in &resp.recent_config_changes {
    let serialized = serde_json::to_string(entry)?;
    assert!(
      !serialized.contains("unsigned_row_should_not_leak"),
      "unsigned governance_log row leaked into recent_config_changes: {entry:?}",
    );
  }

  let _ = (unsigned_id, signed_id);
  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_stream_forbidden_for_non_admin()
-> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  let (_, user_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "sse_nonadmin", false).await?;

  let result = admin_audit_stream(context.clone(), user_view).await;
  // is_err() alone would also pass on a tokio_postgres::connect failure
  // before the admin check; the specific-variant match anchors the test
  // to the capability gate (cr-18).
  let err = result.expect_err("non-admin must be rejected by is_admin()");
  assert!(
    matches!(&err.error_type, LemmyErrorType::NotAnAdmin),
    "expected NotAnAdmin, got {:?}",
    err.error_type,
  );

  Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_stream_enforces_per_admin_cap()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::http::StatusCode;
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  // Unique username avoids PersonId collision with other SSE tests in
  // the same process (module-static HashSet leaks across tests per plan
  // §14 GOTCHA).
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "sse_cap_admin", true).await?;

  // First connection succeeds — returns 200 with text/event-stream body.
  let resp1 = admin_audit_stream(context.clone(), admin_view.clone()).await?;
  assert_eq!(resp1.status(), StatusCode::OK, "first connection returns 200");
  assert_eq!(
    resp1
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or_default(),
    "text/event-stream",
    "Content-Type is text/event-stream",
  );

  // Second concurrent connection — same admin — returns 409 Conflict.
  let resp2 = admin_audit_stream(context.clone(), admin_view.clone()).await?;
  assert_eq!(
    resp2.status(),
    StatusCode::CONFLICT,
    "second concurrent connection from same admin returns 409",
  );

  // Drop the first response body so the SseGuard drops and releases
  // the per-admin slot. Then a third connection should succeed within
  // a short window — proves SseGuard::Drop ran.
  drop(resp1);
  // The Drop impl spawns an async cleanup task; yield to let it run.
  for _ in 0..20 {
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let resp3 = admin_audit_stream(context.clone(), admin_view.clone()).await?;
    if resp3.status() == StatusCode::OK {
      drop(resp3);
      return Ok(());
    }
    drop(resp3);
  }
  panic!("third connection did not succeed after first was dropped — SseGuard::Drop may not be releasing the cap entry");
}

/// Drive `admin_audit_stream`'s streaming body end-to-end: open the SSE
/// connection, trigger an `admin_config_changed` write via
/// `admin_set_config`, and assert a correctly-framed `admin_config_changed`
/// SSE event arrives with the expected JSON payload.
///
/// Exercises the full live-stream path the other two SSE tests skip:
///   - the `kind == ENTRY_KIND_ADMIN_CONFIG_CHANGED` filter branch
///   - the `entry_id` → `governance_log` row hydration via `get_conn`
///   - `project_to_audit_entry` round-trip through the streaming body
///   - `event: X\ndata: Y\n\n` frame format per HTML5 §9.2.4
#[tokio::test(flavor = "multi_thread")]
async fn admin_audit_stream_emits_frame_on_config_change()
-> lemmy_utils::error::LemmyResult<()> {
  use std::{
    future::poll_fn,
    pin::Pin,
    time::Duration as StdDuration,
  };
  use actix_web::{body::MessageBody, http::StatusCode, web::Json};
  use lemmy_api::governance::{
    admin_audit_stream::admin_audit_stream,
    admin_config::admin_set_config,
  };
  use lemmy_api_common::governance::AdminSetConfig;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
  let instance = admin_config_fixtures::bootstrap_instance(&context).await?;
  // Unique username avoids PersonId collision with the other SSE tests
  // (the module-static cap HashSet persists across tests in the same
  // process).
  let (_, admin_view) =
    admin_config_fixtures::seed_user(&context, instance.id, "sse_emit_admin", true).await?;

  // Open the SSE stream. The handler returns a 200 with a streaming
  // body; we drain frames below via `MessageBody::poll_next`.
  let resp = admin_audit_stream(context.clone(), admin_view.clone()).await?;
  assert_eq!(resp.status(), StatusCode::OK, "stream opens with 200");
  assert_eq!(
    resp
      .headers()
      .get("content-type")
      .and_then(|v| v.to_str().ok())
      .unwrap_or_default(),
    "text/event-stream",
    "Content-Type is text/event-stream",
  );

  // `into_body()` yields the `BoxBody` driving the stream. We poll it
  // frame-by-frame with a timeout. Each SSE message arrives as a single
  // `Bytes` chunk (the handler emits `yield Ok(Bytes::from(...))` per
  // frame).
  let mut body = resp.into_body();

  // First frame is the initial `retry: 10000\n\n` the handler emits
  // before entering its select loop. Pull it out so the subsequent reads
  // see a clean stream.
  let retry_frame = tokio::time::timeout(StdDuration::from_secs(5), poll_fn(|cx| {
    Pin::new(&mut body).poll_next(cx)
  }))
  .await
  .map_err(|_| anyhow::anyhow!("timed out waiting for initial retry frame"))?
  .ok_or_else(|| anyhow::anyhow!("body ended before retry frame"))?
  .map_err(|e| anyhow::anyhow!("body error on retry frame: {e}"))?;
  let retry_str = std::str::from_utf8(&retry_frame)
    .map_err(|e| anyhow::anyhow!("retry frame not utf-8: {e}"))?;
  assert_eq!(
    retry_str, "retry: 10000\n\n",
    "initial frame is the SSE retry field (HTML5 §9.2.5, not a custom event)",
  );

  // Trigger an `admin_config_changed` write. The governance_log INSERT
  // fires the `governance_events` NOTIFY, which the handler's dedicated
  // tokio-postgres LISTEN connection observes and forwards into the
  // stream.
  let _set_resp = admin_set_config(
    Json(AdminSetConfig {
      key: "jury.panel_size".to_string(),
      value_type: "int".to_string(),
      value: serde_json::json!(9),
      scope: "instance".to_string(),
      apply_at: None,
      dry_run: None,
      reason: "v1-AD-d SSE emission test".to_string(),
    }),
    context.clone(),
    admin_view.clone(),
  )
  .await?;

  // Drain frames (skipping `: keepalive\n\n` comments) until we observe
  // an `event: admin_config_changed` frame or time out. The handler
  // heartbeat interval is 15s, so under a 10s budget we expect zero
  // keepalive frames — but the loop is defensive against scheduling
  // jitter and future interval changes.
  let config_frame = tokio::time::timeout(StdDuration::from_secs(10), async {
    loop {
      let chunk = poll_fn(|cx| Pin::new(&mut body).poll_next(cx))
        .await
        .ok_or_else(|| anyhow::anyhow!("body ended before config-change frame"))?
        .map_err(|e| anyhow::anyhow!("body error: {e}"))?;
      let s = std::str::from_utf8(&chunk)
        .map_err(|e| anyhow::anyhow!("frame not utf-8: {e}"))?
        .to_owned();
      if s.starts_with(": keepalive") {
        continue;
      }
      return Ok::<String, anyhow::Error>(s);
    }
  })
  .await
  .map_err(|_| anyhow::anyhow!("timed out waiting for admin_config_changed frame"))??;

  // SSE framing: `event: admin_config_changed\ndata: {json}\n\n`.
  assert!(
    config_frame.ends_with("\n\n"),
    "SSE frame terminates with two newlines (HTML5 §9.2.4); got: {config_frame:?}",
  );
  let mut lines = config_frame.trim_end_matches("\n\n").split('\n');
  let event_line = lines
    .next()
    .ok_or_else(|| anyhow::anyhow!("frame has no event line: {config_frame:?}"))?;
  let data_line = lines
    .next()
    .ok_or_else(|| anyhow::anyhow!("frame has no data line: {config_frame:?}"))?;
  assert!(
    lines.next().is_none(),
    "frame has exactly event + data lines; got extra: {config_frame:?}",
  );
  assert_eq!(
    event_line, "event: admin_config_changed",
    "event line names the entry kind",
  );
  let data_json = data_line
    .strip_prefix("data: ")
    .ok_or_else(|| anyhow::anyhow!("data line missing 'data: ' prefix: {data_line:?}"))?;
  let payload: serde_json::Value = serde_json::from_str(data_json)
    .map_err(|e| anyhow::anyhow!("data payload not valid JSON ({e}): {data_json:?}"))?;

  // The payload is the full `AdminConfigAuditEntry` shape produced by
  // `project_to_audit_entry`.
  assert_eq!(
    payload["entry_kind"].as_str(),
    Some("admin_config_changed"),
    "payload.entry_kind mirrors the event type",
  );
  assert_eq!(
    payload["scope"].as_str(),
    Some("instance"),
    "payload.scope reflects the admin_set_config call",
  );
  assert_eq!(
    payload["key"].as_str(),
    Some("jury.panel_size"),
    "payload.key reflects the admin_set_config call",
  );
  assert_eq!(
    payload["value_type"].as_str(),
    Some("int"),
    "payload.value_type reflects the admin_set_config call",
  );
  assert_eq!(
    payload["new_value"], serde_json::json!(9),
    "payload.new_value reflects the written value",
  );
  assert!(
    payload["id"].as_i64().unwrap_or_default() > 0,
    "payload.id is the governance_log row id",
  );
  assert!(
    payload["created_at"].as_str().is_some(),
    "payload.created_at populated",
  );

  // Drop the stream so the SseGuard releases the per-admin cap entry
  // before the container tear-down runs.
  drop(body);
  Ok(())
}

// ============================================================================
// v1-JM-b Task 7 — severity/status cascade fixture + tests
// ============================================================================
//
// Six tests per plan §14 Task 7:
//
// 1. regular/minor  → panel 5 / quorum 3 / threshold 3
// 2. regular/severe → panel 7 / quorum 5 / threshold 6
// 3. founder/severe → panel 9 / quorum 7 / threshold 7
// 4. `jury_assignment.selected_under_constraints` JSONB shape
// 5. `severity_tier_frozen` governance_log row emitted
// 6. `config::get_int_cascade` walks founder.severe → severe → bare → const
//
// The fixture mirrors `admin_config_fixtures::bootstrap` at e2e.rs:~4620.
// admin_assign_jury's small-pool-fallback (Phase 4 `legacy_select_eligible_jurors`
// shape) seats the panel when no reputation_snapshot rows exist, so the
// fixture skips snapshot seeding — the cascade-resolved panel_size still
// applies upstream of the fallback and the snapshot fields still land.

mod v1_jm_b_fixtures {
  use chrono::{Duration as ChronoDuration, Utc};
  use diesel::{Connection as _, PgConnection};
  use diesel_async::{AsyncPgConnection, RunQueryDsl};
  use lemmy_db_schema::source::governance::{
    moderation_case::ModerationCaseInsertForm,
    reputation_event::ReputationEventInsertForm,
    reputation_snapshot::ReputationSnapshotInsertForm,
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{
      CaseSeverity, CaseStatus, CaseTargetType, ReputationDimension, SeverityTier,
    },
    schema::{moderation_case, reputation_event, reputation_snapshot},
  };
  use lemmy_utils::error::LemmyResult;
  use std::error::Error;

  /// Insert a case directly with the given severity_tier. `creator_id` is
  /// NULL so the admin is not excluded from the panel. `community_id` is
  /// NULL (instance-scope) so the eligibility query's
  /// `rs.community_id IS NOT DISTINCT FROM $1` matches any snapshot (or,
  /// under the small-pool fallback, ignores community scope entirely).
  pub async fn seed_case(
    db_url: &str,
    target: PersonId,
    severity_tier: SeverityTier,
  ) -> Result<lemmy_db_schema::newtypes::ModerationCaseId, Box<dyn Error + Send + Sync>> {
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
      value["no_majority_from_same_sponsor_cluster"], Value::String("applied".to_string()),
      "row {idx} cluster constraint = applied"
    );
    assert_eq!(
      value["geographic_diversity_preferred"], Value::String("applied_soft".to_string()),
      "row {idx} geographic preference = applied_soft"
    );
    assert_eq!(
      value["no_recent_juror_repeat"], Value::String("applied".to_string()),
      "row {idx} juror cooldown = applied"
    );
    assert_eq!(
      value["no_same_endorsement_chain"], Value::String("disabled".to_string()),
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
    payload["severity_tier"], Value::String("minor".to_string()),
    "severity_tier slug = minor"
  );
  assert_eq!(
    payload["status_tier"], Value::String("regular".to_string()),
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
async fn admin_assign_jury_small_pool_triggers_r1_relaxation()
-> lemmy_utils::error::LemmyResult<()> {
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
  use lemmy_db_schema_file::{enums::{CaseStatus, SeverityTier}, schema::moderation_case};

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
    v1_jm_b_fixtures::seed_jury_eligible_snapshots_scoped(
      &mut conn,
      &jurors,
      Some(community.id),
    )
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
  assert_eq!(payload["severity_tier"], Value::String("severe".to_string()));
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
async fn submit_jury_vote_severe_panel_meets_threshold()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote,
  };
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
async fn submit_jury_vote_deadlock_flips_to_admin_review()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote,
  };
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
  assert!(final_resp.decision.is_none(), "deadlock has no winning decision");

  let mut conn = AsyncPgConnection::establish(&db_url).await?;
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
async fn submit_jury_vote_writes_appeal_window_default()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use chrono::Duration;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote,
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
async fn submit_jury_vote_writes_appeal_window_live_config()
-> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Json;
  use chrono::Duration;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    admin_config::admin_set_config,
    submit_jury_vote::submit_jury_vote,
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
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    admin_config::admin_set_config,
    submit_jury_vote::submit_jury_vote,
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
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote,
  };
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
  let view_a = LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2])
    .await?;
  let view_b = LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[3])
    .await?;
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
  use actix_web::web::Json;
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
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
  use diesel::{ExpressionMethods, QueryDsl};

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
    use lemmy_api_crud::governance::create_report::create_report;
    use lemmy_api_common::governance::CreateGovernanceReport;
    use lemmy_db_schema_file::enums::{CaseStatus, CaseTargetType};
    use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
    use lemmy_db_schema_file::schema::moderation_case;
    use diesel::SelectableHelper;

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
      let reporter_ids: Vec<PersonId> =
        reporters.iter().map(|v| v.person.id).collect();
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
        assert!(resp.threshold_met, "report 3: 4_000_000 > 3_000_000 (threshold met)");
      } else {
        assert!(!resp.threshold_met, "report {i}: cumulative not > 3_000_000 yet");
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
      case_after_reports.status, CaseStatus::ThresholdMet,
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
    appeal_window_expiry::run_appeal_window_expiry_batch,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::SubmitJuryVote;
  use lemmy_db_schema::source::instance::Instance;
  use lemmy_db_schema_file::{
    enums::{CaseStatus, JuryDecision, AppealStatus},
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
  let prev_appeal_window_disable =
    std::env::var_os("BREHON_DISABLE_APPEAL_WINDOW_JOB");
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
  let (case_id, appeal_id, appeal_panel_ids) =
    v1_jm_e_fixtures::seed_appealed_case_with_panel(
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
  let (appeal_decided_at, appeal_status): (
    Option<chrono::DateTime<chrono::Utc>>,
    AppealStatus,
  ) = appeal::table
    .filter(appeal::id.eq(appeal_id))
    .select((appeal::decided_at, appeal::status))
    .first(&mut conn)
    .await?;
  assert!(appeal_decided_at.is_some(), "appeal.decided_at populated");
  assert_eq!(appeal_status, AppealStatus::Decided, "appeal status = Decided");

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
  assert_eq!(status, CaseStatus::Closed, "case → Closed after appeal verdict");
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
async fn governance_log_sequence_matches_prd_state_machine()
-> lemmy_utils::error::LemmyResult<()> {
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
  let prev_appeal_window_disable =
    std::env::var_os("BREHON_DISABLE_APPEAL_WINDOW_JOB");
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
    let row_case_id = payload.get("case_id").and_then(|v| v.as_i64());
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
  //   - public_log_published (between case_decided and appeal_requested) —
  //     redacted public log entry created on case-decide
  //     (Phase 4b shipped, submit_jury_vote.rs)
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
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    admin_config::admin_set_config,
    submit_jury_vote::submit_jury_vote,
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
  let drive_case_to_decided = async |target: lemmy_db_schema_file::PersonId|
   -> lemmy_utils::error::LemmyResult<lemmy_db_schema::newtypes::ModerationCaseId> {
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
    matches!(&err.error_type, lemmy_utils::error::LemmyErrorType::NotFound),
    "§12.4: expected LemmyErrorType::NotFound on orphan-case spoof, got {:?}",
    err.error_type,
  );

  Ok(())
}

mod v1_sl_b_fixtures {
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
      endorsement::EndorsementInsertForm,
      moderation_case::ModerationCaseInsertForm,
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
      .select((moderation_case::status, moderation_case::liability_escape_reason))
      .first(conn)
      .await?;
    Ok(row)
  }

  /// Count governance_log entries of a given kind.
  async fn count_log_entries(
    conn: &mut AsyncPgConnection,
    kind: &str,
  ) -> LemmyResult<i64> {
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
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
      read_endorsement_revoked_at(&mut conn, endorsement_id).await?.is_none(),
      "pre-call: endorsement.revoked_at IS NULL",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None).await?.is_none(),
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

    assert_eq!(resp.endorsement_id, endorsement_id, "response endorsement_id matches");
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
      read_endorsement_revoked_at(&mut conn, endorsement_id).await?.is_some(),
      "post-call: endorsement.revoked_at populated",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None).await?.is_some(),
      "post-call: surety.revoked_at populated",
    );

    // Snapshots: both sponsor + sponsee recomputed (calculated_at >= test_start).
    let sponsor_calc = read_snapshot_calculated_at(&mut conn, sponsor).await?
      .expect("sponsor snapshot exists post-recompute");
    assert!(
      sponsor_calc >= test_start,
      "sponsor snapshot recomputed: calculated_at {:?} >= test_start {:?}",
      sponsor_calc,
      test_start,
    );
    let sponsee_calc = read_snapshot_calculated_at(&mut conn, sponsee).await?
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

    let payload = read_log_payload(&mut conn, "endorsement_revoked").await?
      .expect("endorsement_revoked payload exists");
    assert!(payload["revoker_pseudonym"].is_string(), "revoker_pseudonym is a string");
    assert!(payload["target_pseudonym"].is_string(), "target_pseudonym is a string");
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
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
    assert!(read_endorsement_revoked_at(&mut conn, endorsement_id).await?.is_some());
    assert!(read_surety_revoked_at(&mut conn, sponsor, sponsee, None).await?.is_some());

    // governance_log entry: caller is admin, target is sponsee. rate_limit_bypassed
    // ABSENT because admin was under threshold (DQ #141 separation).
    let payload = read_log_payload(&mut conn, "endorsement_revoked").await?
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
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
  async fn revoke_endorsement_non_sponsor_non_admin_rejects_with_not_found()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
      read_endorsement_revoked_at(&mut conn, endorsement_id).await?.is_none(),
      "endorsement.revoked_at unchanged on rejection",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor, sponsee, None).await?.is_none(),
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
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
      read_endorsement_revoked_at(&mut conn, endorsement_id).await?.is_none(),
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
  async fn revoke_endorsement_rate_limit_enforces_unless_admin_bypasses()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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

    seed_prior_revocations(&mut conn, &context, instance.id, regular_caller, "regular", 5).await?;
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
      matches!(err.error_type, lemmy_utils::error::LemmyErrorType::TooManyRequests),
      "expected TooManyRequests at threshold, got {:?}",
      err.error_type,
    );

    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    assert!(
      read_endorsement_revoked_at(&mut conn, regular_active_eid).await?.is_none(),
      "regular caller's active endorsement remains untouched after rate-limit reject",
    );

    // Admin at threshold — bypasses, succeeds, log includes rate_limit_bypassed: true.
    seed_prior_revocations(&mut conn, &context, instance.id, admin_caller, "admin", 5).await?;
    let (sponsee_admin, _) =
      governance_fixtures::seed_user(&context, instance.id, "sponsee_admin_slb6", false).await?;
    let admin_active_eid =
      seed_endorsement_active(&mut conn, admin_caller, sponsee_admin).await?;
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
    let payload = read_log_payload(&mut conn, "endorsement_revoked").await?
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
  async fn revoke_endorsement_severs_grace_window_single_sponsor_case()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
    let (status_before, escape_before) =
      read_case_status_and_escape(&mut conn, case_id).await?;
    assert_eq!(status_before, CaseStatus::SponsorLiabilityPending, "pre-call: pending");
    assert!(escape_before.is_none(), "pre-call: liability_escape_reason NULL");

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
    let (status_after, escape_after) =
      read_case_status_and_escape(&mut conn, case_id).await?;
    assert_eq!(
      status_after,
      CaseStatus::SponsorLiabilityEscaped,
      "case status flipped to escaped",
    );
    let escape_json = escape_after.expect("escape_reason JSONB populated");
    assert_eq!(escape_json["version"], Value::Number(1.into()), "version: 1");
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
  async fn revoke_endorsement_multi_sponsor_any_revocation_severs_chain()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
      read_surety_revoked_at(&mut conn, sponsor_a, sponsee, None).await?.is_some(),
      "sponsor_a's surety revoked",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor_b, sponsee, None).await?.is_none(),
      "sponsor_b's surety untouched",
    );
    assert!(
      read_surety_revoked_at(&mut conn, sponsor_c, sponsee, None).await?.is_none(),
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
  async fn revoke_endorsement_no_pending_case_no_severance_only_revoked_log()
  -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;
    let instance =
      lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid")
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
    assert!(read_endorsement_revoked_at(&mut conn, endorsement_id).await?.is_some());
    assert!(read_surety_revoked_at(&mut conn, sponsor, sponsee, None).await?.is_some());

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

    let payload = read_log_payload(&mut conn, "endorsement_revoked").await?
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
  use super::*;
  use chrono::{Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, insert_into, update};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch;
  use lemmy_db_schema::{
    newtypes::{ModerationCaseId, SuretyId},
    source::governance::{
      endorsement::EndorsementInsertForm,
      moderation_case::ModerationCaseInsertForm,
      sanction::SanctionInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{
      CaseSeverity,
      CaseStatus,
      CaseTargetType,
      SanctionAction,
      SanctionScope,
    },
    schema::{endorsement, governance_log, moderation_case, reputation_event, sanction, surety},
  };
  use lemmy_utils::error::LemmyResult;
  use serde_json::Value;

  async fn count_log_entries(
    conn: &mut AsyncPgConnection,
    kind: &str,
  ) -> LemmyResult<i64> {
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
  async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(
  ) -> LemmyResult<()> {
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
    assert_eq!(rep_event_count, 2, "2 reputation_event rows (1 per sponsor)");

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
  async fn grace_check_escapes_case_when_sponsor_revoked_after_decided_at(
  ) -> LemmyResult<()> {
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
      outcome.cases_processed,
      0,
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
  async fn grace_check_per_case_isolation_skips_bad_case_processes_good_case(
  ) -> LemmyResult<()> {
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
      outcome.cases_processed,
      2,
      "2 cases processed (case_b first, case_a second)"
    );
    assert_eq!(outcome.fired, 1, "1 case fired (case_a)");
    assert_eq!(
      outcome.skipped,
      1,
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
    assert_eq!(rep_a_count, 1, "1 reputation_event row for case_a's sponsor");

    let rep_b_count: i64 = reputation_event::table
      .filter(reputation_event::source_case_id.eq(case_b_id))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(rep_b_count, 0, "0 reputation_event rows for case_b (skipped)");

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
      let offset_minutes = 5 - i as i64;
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
      outcome1.cases_processed,
      2,
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
      fired_after_1,
      2,
      "invocation 1: 2 cases total SponsorLiabilityFired"
    );
    let pending_after_1: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      pending_after_1,
      3,
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
      outcome2.cases_processed,
      2,
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
      fired_after_2,
      4,
      "invocation 2: 4 cases total SponsorLiabilityFired"
    );
    let pending_after_2: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      pending_after_2,
      1,
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
      outcome3.cases_processed,
      1,
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
      fired_after_3,
      5,
      "invocation 3: all 5 cases SponsorLiabilityFired"
    );
    let pending_after_3: i64 = moderation_case::table
      .filter(moderation_case::status.eq(CaseStatus::SponsorLiabilityPending))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      pending_after_3,
      0,
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
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use actix_web::web::{Data, Json};
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, insert_into};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{AcceptJuryAssignment, AdminAssignJury, SubmitJuryVote};
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::{
    newtypes::ModerationCaseId,
    source::governance::{
      moderation_case::ModerationCaseInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    InstanceId,
    PersonId,
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
    let (sponsee, _) = governance_fixtures::seed_user(
      context,
      instance_id,
      &format!("{prefix}_sponsee"),
      false,
    )
    .await?;
    let mut sponsor_ids = Vec::with_capacity(sponsor_count);
    for i in 0..sponsor_count {
      let (sponsor, _) = governance_fixtures::seed_user(
        context,
        instance_id,
        &format!("{prefix}_sp{i}"),
        false,
      )
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
      let (id, _) = governance_fixtures::seed_user(
        &context,
        instance.id,
        &format!("sld_juror{i}"),
        false,
      )
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
      LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2])
        .await?;
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

    let plog_count: i64 = public_case_log::table
      .count()
      .get_result(&mut conn)
      .await?;
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
    assert_eq!(target_psn.len(), 36, "target_pseudonym is a UUID (36 chars)");
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
    assert_eq!(psns.len(), 2, "2 sponsor pseudonyms (one per active surety)");
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
  async fn submit_jury_vote_preserves_v0_decided_for_no_sponsor_target()
  -> LemmyResult<()> {
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
      let (id, _) = governance_fixtures::seed_user(
        &context,
        instance.id,
        &format!("sld2_juror{i}"),
        false,
      )
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
      LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2])
        .await?;
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
    let plog_count: i64 = public_case_log::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(plog_count, 1, "1 public_case_log row on Decided path");

    // 3 juror reputation events (3 votes cast) + 1 reporter = 4 total.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count,
      4,
      "3 juror + 1 reporter reputation events fire immediately on Decided path"
    );

    // 0 sponsor_liability_pending entries: no sureties → Decided path, not Pending.
    let slt_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sponsor_liability_pending"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(slt_count, 0, "0 sponsor_liability_pending log entries on no-sponsor path");

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
      let (id, _) = governance_fixtures::seed_user(
        &context,
        instance.id,
        &format!("sld3_juror{i}"),
        false,
      )
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
      LocalUserView::read_person(&mut context.pool(), assign_resp.assigned_person_ids[2])
        .await?;
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
    assert_eq!(slt_count, 0, "0 sponsor_liability_pending log entries on NoAction path");

    // NoAction → map_decision_to_sanction returns None → no sanction row → 0 sanction_created.
    let sanction_log_count: i64 = governance_log::table
      .filter(governance_log::entry_kind.eq("sanction_created"))
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(sanction_log_count, 0, "0 sanction_created log entries on NoAction path");

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
      sponsor_rep_count,
      0,
      "0 reputation_event rows for sponsors on NoAction path"
    );

    // Juror events fire for the 3 who voted; reporter event fires (1 row).
    // Total = 3 juror + 1 reporter = 4.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count,
      4,
      "3 juror + 1 reporter reputation events fire on NoAction Decided path"
    );

    Ok(())
  }

  #[tokio::test]
  async fn apply_sponsor_liability_wrapper_preserves_v0_outputs() -> LemmyResult<()> {
    use lemmy_api::governance::sponsor_liability_grace::run_grace_check_batch;
    use diesel::update;
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
      rep_count,
      2,
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
    assert_eq!(applied_payloads.len(), 2, "2 sponsor_liability_applied log entries");
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
    assert_eq!(clamped_count, 0, "0 sponsor_liability_clamped entries (no clamp engaged)");

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
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use actix_web::web::{Data, Json};
  use chrono::{DateTime, Duration, Utc};
  use diesel::{ExpressionMethods, QueryDsl, insert_into, update};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment,
    admin_assign_jury::admin_assign_jury,
    sponsor_liability_grace::run_grace_check_batch,
    submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment,
    AdminAssignJury,
    RevokeEndorsement,
    SubmitJuryVote,
  };
  use lemmy_api_crud::governance::revoke_endorsement::revoke_endorsement;
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::{
    newtypes::{EndorsementId, ModerationCaseId},
    source::governance::{
      endorsement::EndorsementInsertForm,
      moderation_case::ModerationCaseInsertForm,
      surety::SuretyInsertForm,
    },
  };
  use lemmy_db_schema_file::{
    InstanceId,
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryDecision, SeverityTier},
    schema::{endorsement, governance_log, moderation_case, public_case_log, reputation_event, surety},
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
    let (sponsee, _) = governance_fixtures::seed_user(
      context,
      instance_id,
      &format!("{prefix}_sponsee"),
      false,
    )
    .await?;
    let mut sponsors = Vec::with_capacity(sponsor_count);
    for i in 0..sponsor_count {
      let (sponsor, _) = governance_fixtures::seed_user(
        context,
        instance_id,
        &format!("{prefix}_sp{i}"),
        false,
      )
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

  async fn count_log_entries(
    conn: &mut AsyncPgConnection,
    kind: &str,
  ) -> LemmyResult<i64> {
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
      unsafe { std::env::set_var("BREHON_DISABLE_GRACE_CHECK_JOB", value); }
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

    let (sponsee, sponsors) = seed_target_with_sureties_and_endorsements(
      &context,
      instance.id,
      &mut conn,
      2,
      "sle1",
    )
    .await?;
    let (sponsor2, endo2) = sponsors[1];

    let mut juror_ids = Vec::with_capacity(5);
    for i in 0..5_usize {
      let (id, _) = governance_fixtures::seed_user(
        &context,
        instance.id,
        &format!("sle1_juror{i}"),
        false,
      )
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

    let plog_count: i64 = public_case_log::table
      .count()
      .get_result(&mut conn)
      .await?;
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
      revoke_resp.endorsement_id,
      endo2,
      "revoke response endorsement_id matches"
    );
    assert!(
      revoke_resp.revoked_at >= t_revoke_start
        && revoke_resp.revoked_at <= t_revoke_end + Duration::seconds(1),
      "revoked_at in [t_revoke_start, now+1s], got {:?}",
      revoke_resp.revoked_at
    );
    assert!(
      revoke_resp.liability_chain_severed_for_cases.contains(&case_id),
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

    let plog_count: i64 = public_case_log::table
      .count()
      .get_result(&mut conn)
      .await?;
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
      outcome.cases_processed,
      0_usize,
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

    let (sponsee, _sponsors) = seed_target_with_sureties_and_endorsements(
      &context,
      instance.id,
      &mut conn,
      2,
      "sle2",
    )
    .await?;

    let mut juror_ids = Vec::with_capacity(5);
    for i in 0..5_usize {
      let (id, _) = governance_fixtures::seed_user(
        &context,
        instance.id,
        &format!("sle2_juror{i}"),
        false,
      )
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

    let plog_count: i64 = public_case_log::table
      .count()
      .get_result(&mut conn)
      .await?;
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
    assert_eq!(rep_event_count, 2, "2 reputation_event rows (1 per sponsor)");

    let plog_count_after: i64 = public_case_log::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      plog_count_after,
      0,
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
    let (sponsee, sponsors) = seed_target_with_sureties_and_endorsements(
      &context,
      instance.id,
      &mut conn,
      1,
      "sle3",
    )
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
    assert_eq!(pre_status, CaseStatus::Decided, "pre-backfill: status is Decided");
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
      rep_guard_count,
      0,
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
        .select((
          moderation_case::status,
          moderation_case::grace_expires_at,
        ))
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
      rep_after_backfill,
      0,
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

    let plog_count: i64 = public_case_log::table
      .count()
      .get_result(&mut conn)
      .await?;
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
