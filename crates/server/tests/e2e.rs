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
