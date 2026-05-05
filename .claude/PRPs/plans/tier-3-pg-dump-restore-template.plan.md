# Plan — Tier 3: pg_dump + pg_restore template (e2e per-test fast path)

**Sub-phase:** `chore/tooling-tier3-pg-template` (cut from `governance-v0`)
**Branch base:** `governance-v0`
**Author:** advisor (this session)
**Date:** 2026-05-02
**Status:** draft (awaiting user gate before queueing)

## 0. Context

Local validation cycle (per Perplexity research 2026-05-02 + the
PR #107 + PR #109 + PR #108 chain shipping today):

- **Tier 1 — DONE.** rust-lld linker + `[profile.dev]` tweaks → PR #109.
- **Tier 2 — DONE.** cargo-nextest + bacon + `BREHON_USE_NEXTEST` dispatch
  → PR #107. The `--test-threads=1` constraint disappears with zero
  `e2e.rs` changes (process-per-test isolation handles `LazyLock<Settings>`).
- **Tier 3 — THIS PLAN.** Drop per-test container-startup cost from
  ~90s (cold migrations) to ~1-3s (pg_restore from dump bytes).

**Selected pattern:** `pg_dump` + `pg_restore` template database.
- Each nextest process spins ONE container at suite startup.
- That container applies all migrations + replaceable schema once.
- Dump captured to in-memory bytes via `LazyLock<Vec<u8>>`.
- Per test: spin a fresh container, restore the dump, return.

**Why not the alternatives** (rationale captured at retro):
- Multi-stage docker commit (option 1) — initdb.d re-runs every container
  start, brittle clean-shutdown semantics, custom image tag rebuild dance.
  The deferred Tier 3 scaffolding `Dockerfile.test-pg` +
  `apply-migrations-to-image.sh` from PR #107 are NOT used by this plan
  and may be removed in a follow-up cleanup PR.
- Shared container + per-test schema (option 3) — fastest (~10-50ms) but
  introduces silent regression mode if Diesel `MigrationHarness` doesn't
  honor `search_path`; cross-test state leak is invisible until a pool
  recycles a connection mid-test. Sharp edge declined.

## 1. Hard constraints (do NOT contradict)

- **Lemmy 1.0-beta fork** — testcontainers-rs already at 0.27.x
- **AGPLv3** inherited — no licensing change
- **v0 simplifications** — 5-juror panels, quorum 3, etc; this is e2e
  harness work, no governance semantic changes
- **No EliteDesk cargo** — laptop is canonical cargo runner per
  `project_laptop_canonical_cargo_runner.md`; e2e.rs Edits are
  advisor-side only per `feedback_junior_worker_e2e_edit_hang.md`
- **No-Junior-Edit zone** — `crates/server/tests/e2e.rs` is 9000+ lines;
  Edit window is bounded to `governance_fixtures::*` ~30 lines max
- **`forbid_diesel_cli` trigger** — migrations require `pg_advisory_lock(0)`;
  the existing `apply_all_schema()` in e2e.rs:81-94 already handles this.
  pg_dump captures the migrations table state, so restore doesn't
  need to re-run migrations or re-acquire the lock — trigger is bypassed
  cleanly.

## 2. Watchpoints (each cites a specific file:line)

1. **`crates/server/tests/e2e.rs:81-94`** — `governance_fixtures::apply_all_schema`
   becomes the no-op fast path (or pg_restore call). Existing migration
   logic kept for the bootstrap process that builds the dump.
2. **`crates/server/tests/e2e.rs:99-123`** — `governance_fixtures::start_postgres`
   signature stays `(ContainerAsync<GenericImage>, u16)`. Internal flow
   changes from "boot container + apply migrations" to "boot container
   + pg_restore template".
3. **`crates/server/tests/e2e.rs:130+`** — every `let (_container, host_port) =
   governance_fixtures::start_postgres().await?;` caller (14 of them)
   stays unchanged. Verify with `rg 'governance_fixtures::start_postgres'
   crates/server/tests/e2e.rs`.
4. **`crates/diesel_utils/replaceable_schema/{utils,triggers}.sql`** —
   `r.*` schema is rebuilt on every cargo run via `apply_all_schema`'s
   step 3-5 (DROP SCHEMA r CASCADE; CREATE SCHEMA r; \i utils; \i triggers).
   The dump must capture both `public` and `r` schemas.
5. **`Cargo.toml [workspace.dependencies]`** — pg_dump/pg_restore are
   shelled-out commands, not Rust crates. The `lemmy_server` test target
   needs no new dependency; the helpers use `tokio::process::Command`.
6. **testcontainers-rs `ContainerAsync<GenericImage>` host port** — already
   exposed by `governance_fixtures::start_postgres`'s return tuple.
   `pg_dump -h localhost -p $port` and `pg_restore -h localhost -p $port`
   work via the host network mapping; no Docker-network gymnastics.
7. **`LazyLock<Vec<u8>>` for the dump bytes** — captured ONCE per
   nextest process. Under nextest's process-per-test model, each test
   process pays the bootstrap cost ONCE (estimate: ~30s for the bootstrap
   container) then per-test cost drops to ~1-3s for restore. Net wall
   clock for 14 tests with `threads-required=4` ≈ 4 × (30s + 4 × 2s)
   ≈ 2.5 min.

## 3. Stories (§16a)

Each story is a verifiable user-visible outcome. Status: `[ ]` until done.

- [ ] **S1: bootstrap dump captured.** `cargo nextest run -E
  'test(template_dump_capture)'` runs a new sentinel test that calls
  `governance_fixtures::TEMPLATE_DUMP.deref()` and asserts the byte
  count is non-zero + > 100 KB (sanity check that all migrations applied).

- [ ] **S2: existing tests use pg_restore path.** `cargo nextest run
  -E 'test(postgres_container_boots)'` PASSES in <30s wall-clock
  (down from ~95s). Restore time visible in test stdout via a
  `tracing::info!` line.

- [ ] **S3: full e2e suite passes via dump path.** `cargo nextest run
  --workspace --features full --test e2e` PASSES with all 64 governance
  tests (or 68 incl. lifecycle), wall-clock under 6 min with
  `threads-required=4` (down from 26 min serial).

- [ ] **S4: replaceable schema rebuild preserved.** A test that depends
  on `r.*` triggers (e.g. any test exercising `governance_log` or
  `score_recompute`) PASSES after the dump-restore path. Specifically
  re-run `case_lifecycle_e2e_decided` and `case_lifecycle_e2e_appealed`.

- [ ] **S5: restore-time fast path observable.** Each test's stdout
  includes a `pg_restore: <Nms>` log line; the bootstrap test
  (TEMPLATE_DUMP build) emits `pg_dump: <Nms>` at suite startup.

- [ ] **S6: rollback works.** Set env `BREHON_E2E_NO_TEMPLATE=1` →
  `governance_fixtures::start_postgres` falls back to the old
  `apply_all_schema` path. Verifies the legacy path still compiles
  and works for debug.

## 4. Implementation phases

### Phase 0 — Pre-flight (no code, advisor-side)

0.1. Confirm `pg_dump` + `pg_restore` are on PATH on the laptop.
     `where pg_dump.exe`. They ship with vcpkg's libpq install at
     `C:\Users\barri\Developer\vcpkg\installed\x64-windows\bin\`.
     If absent, the wrapper scripts already export that PATH segment.

0.2. Confirm `pg_dump --version` reports >= 17 (matches the
     pgautoupgrade:18-alpine container's pg_dump format. Older
     pg_dump can produce dumps that newer pg_restore can't read,
     but newer pg_dump is back-compatible).

0.3. Read MIRROR ref: `crates/server/tests/e2e.rs:81-123` —
     full `governance_fixtures` module body, especially
     `apply_all_schema` and `start_postgres`. The plan's
     watchpoints 1-3 cite these lines; the implementation Edit
     window is exactly this range.

### Phase 1 — Helper module

**Task 1.1: Create `governance_fixtures::pg_template` sub-module**

Files:
- `crates/server/tests/e2e.rs` — add `mod pg_template { ... }` inside
  the existing `mod governance_fixtures { ... }` block (line 76+).

Adds (~80 lines, all inside the existing fixtures module):

```rust
// New sub-module under governance_fixtures
pub(super) mod pg_template {
    use std::sync::LazyLock;
    use tokio::process::Command;
    use testcontainers::{ContainerAsync, GenericImage};

    /// Captured once per nextest process. The bootstrap cost
    /// (container start + migrations + replaceable_schema) runs
    /// in a tokio::sync::OnceCell; tests await it.
    pub(super) static TEMPLATE_DUMP: LazyLock<tokio::sync::OnceCell<Vec<u8>>> =
        LazyLock::new(tokio::sync::OnceCell::new);

    pub(super) async fn ensure_template() -> Result<&'static Vec<u8>, Box<dyn std::error::Error>> {
        TEMPLATE_DUMP.get_or_try_init(|| async {
            // 1. Boot bootstrap container (vanilla pgautoupgrade:18-alpine)
            let (container, port) = super::start_postgres_vanilla().await?;
            // 2. Apply migrations + replaceable schema (existing logic)
            super::apply_all_schema_legacy(&container, port).await?;
            // 3. pg_dump --schema-only --no-owner (we restore data per test;
            //    in v0 there is no static seed data — all data is per-test)
            let dump = pg_dump(port).await?;
            // 4. container drops here; bootstrap is done
            Ok(dump)
        }).await
    }

    async fn pg_dump(port: u16) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let output = Command::new("pg_dump")
            .args(&[
                "-h", "localhost", "-p", &port.to_string(),
                "-U", "lemmy", "-d", "lemmy",
                "--no-owner", "--no-privileges",
                "--schema=public", "--schema=r",
                "--format=custom",  // pg_restore accepts; smaller bytes
            ])
            .env("PGPASSWORD", "password")
            .output().await?;
        if !output.status.success() {
            return Err(format!("pg_dump failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        tracing::info!(
            bytes = output.stdout.len(),
            ms = start.elapsed().as_millis(),
            "pg_dump: bootstrap dump captured"
        );
        Ok(output.stdout)
    }

    pub(super) async fn pg_restore_into(port: u16, dump: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        let start = std::time::Instant::now();
        let mut child = Command::new("pg_restore")
            .args(&[
                "-h", "localhost", "-p", &port.to_string(),
                "-U", "lemmy", "-d", "lemmy",
                "--no-owner", "--no-privileges",
                "--single-transaction",
                "--exit-on-error",
            ])
            .env("PGPASSWORD", "password")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.as_mut().ok_or("no stdin on pg_restore child")?;
        tokio::io::AsyncWriteExt::write_all(stdin, dump).await?;
        drop(child.stdin.take());
        let output = child.wait_with_output().await?;
        if !output.status.success() {
            return Err(format!("pg_restore failed: {}", String::from_utf8_lossy(&output.stderr)).into());
        }
        tracing::info!(ms = start.elapsed().as_millis(), "pg_restore: template applied");
        Ok(())
    }
}
```

DoD §15: `cargo check -p lemmy_server --features full --tests` exits 0
+ `rg 'pg_template' crates/server/tests/e2e.rs | wc -l` ≥ 4.

**Task 1.2: Rename existing helpers** (preserve legacy path for rollback)

Rename:
- `governance_fixtures::start_postgres` → `start_postgres_vanilla` (returns
  bare container, no schema applied — used by both Phase 1 helpers and
  the legacy fallback path)
- `governance_fixtures::apply_all_schema` → `apply_all_schema_legacy`

Add new public helpers:
- `governance_fixtures::start_postgres` — preserves the existing public
  signature; internally calls `start_postgres_vanilla` then either
  `pg_template::ensure_template` + `pg_restore_into` (default) or
  `apply_all_schema_legacy` (if `BREHON_E2E_NO_TEMPLATE=1`).

DoD §15: `cargo check -p lemmy_server --features full --tests` exits 0;
all 14 caller sites in e2e.rs (lines 130+) compile unchanged.

### Phase 2 — Validation tests

**Task 2.1: Sentinel test for bootstrap path**

Add inside `mod e2e { ... }` (line ~150):

```rust
#[tokio::test]
async fn template_dump_capture() -> Result<(), Box<dyn std::error::Error>> {
    let dump = governance_fixtures::pg_template::ensure_template().await?;
    assert!(dump.len() > 100_000, "template dump suspiciously small: {} bytes", dump.len());
    assert!(dump.starts_with(b"PGDMP"), "not a pg_dump custom-format file");
    Ok(())
}
```

DoD §15: `cargo nextest run -E 'test(template_dump_capture)'` PASSES.

**Task 2.2: Modify the existing `postgres_container_boots` test**

The test name stays; add an assertion that the dump path is being used:

```rust
#[tokio::test]
async fn postgres_container_boots() -> Result<(), Box<dyn std::error::Error>> {
    let (container, port) = governance_fixtures::start_postgres().await?;
    // existing: assert container is reachable + governance_log table exists
    // new: assert that ENV BREHON_E2E_NO_TEMPLATE is unset (default path)
    //   AND that governance_log table is in 'public' schema (restore worked)
    let row_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name = 'governance_log'"
    ).fetch_one(&pool).await?;
    assert_eq!(row_count, 1, "governance_log not present after restore");
    Ok(())
}
```

DoD §15: `cargo nextest run -E 'test(postgres_container_boots)'` PASSES
in < 30s wall-clock (target: ~10s including bootstrap on first run).

### Phase 3 — Full suite verification

**Task 3.1: Run full e2e suite under template path**

```
scripts/brehon/cargo-nextest.bat run --workspace --features full --test e2e
```

DoD §15:
- All 14+ tests PASS
- Total wall-clock under 6 min (down from 26 min)
- Per-test pg_restore log line visible

**Task 3.2: Run full suite under legacy fallback**

```
$env:BREHON_E2E_NO_TEMPLATE = "1"
scripts/brehon/cargo-nextest.bat run --workspace --features full --test e2e
```

DoD §15:
- All 14+ tests PASS via legacy path
- Wall-clock equivalent to current ~26 min (no fast path)
- No `pg_restore` log lines emitted

**Task 3.3: Re-run a known r.* dependent test specifically**

```
scripts/brehon/cargo-nextest.bat run --workspace --features full --test e2e \
  -E 'test(case_lifecycle_e2e_decided)'
```

DoD §15: PASSES (covers replaceable schema r.* trigger preservation
via the dump's `--schema=r` capture).

### Phase 4 — Cleanup + lessons

**Task 4.1: Remove deferred pre-bake scaffolding**

Files to delete (these were the deferred Tier 3 option-1 attempt):
- `scripts/brehon/Dockerfile.test-pg`
- `scripts/brehon/apply-migrations-to-image.sh`

DoD §15: `git rm` both; commit subject `chore(local-tooling): remove
deferred Dockerfile.test-pg scaffolding (Tier 3 chose pg_dump path)`.

**Task 4.2: Update lesson**

Edit `.claude/lessons/feedback_local_validation_cycle_2026_05_02.md`:
- Mark Tier 3 as DONE with the pg_dump+restore pattern
- Remove the "needs rework" warning
- Add the actual measurement: wall-clock cycle dropped from
  26 min → <X min (TBD by Phase 3.1 measurement)

DoD §15: lesson committed with the actual measurement filled in.

## 5. Complexity score (§5.1)

| Factor | Weight | Score | Notes |
|---|---|---|---|
| New crates/migrations | 0.3 | 0 | Zero — pure e2e harness work |
| Modified crates | 0.2 | 1 | Only `crates/server/tests/e2e.rs` |
| e2e.rs edit count | 0.2 | 2 | ~80 lines added inside fixtures + ~20 lines moved (rename) + ~10 lines test changes |
| Cargo deps changed | 0.1 | 0 | None — pg_dump/pg_restore shelled out |
| Test runtime impact | 0.1 | 1 | Big — drops from 26 min to <6 min |
| Cross-cutting (rules) | 0.1 | 0 | None |
| **Total (0-10 scale)** | | **3.0** | Below split-or-proceed threshold (8) |

Within the no-Junior-Edit-zone constraint: **all e2e.rs edits are advisor-
side**, ≤ 100-line diff in one localized block (governance_fixtures
module + a few tests). Not split-eligible.

## 6. Sequencing

Single PR titled
**`feat(e2e): pg_dump+restore template database for fast per-test boot`**.

Commits:
1. `feat(e2e): governance_fixtures pg_template helper module` (Task 1.1)
2. `feat(e2e): start_postgres uses template dump by default` (Task 1.2)
3. `test(e2e): template_dump_capture sentinel + boot test schema check` (Tasks 2.1, 2.2)
4. `test(e2e): full suite verification + r.* preservation check` (Tasks 3.1-3.3, capture as runlog)
5. `chore(local-tooling): remove deferred Dockerfile.test-pg scaffolding` (Task 4.1)
6. `docs(lessons): Tier 3 DONE — pg_dump+restore pattern shipped` (Task 4.2)

CodeRabbit auto-review applies (`crates/server/tests/e2e.rs` is in
review-worthy zones). Address findings in standard four-bucket triage
before merge.

## 7. Verification (full implementation)

This is the §16a stories collapsed into a verification matrix.

| ID | What to run | Expected | Catches |
|---|---|---|---|
| V1 | `cargo nextest run -E 'test(template_dump_capture)'` | PASS, dump > 100KB, starts with `PGDMP` | Bootstrap broken |
| V2 | `cargo nextest run -E 'test(postgres_container_boots)'` | PASS in <30s; governance_log in public schema | Restore not applied |
| V3 | `cargo nextest run --test e2e` (full) | All PASS, wall <6 min | Per-test path broken |
| V4 | `cargo nextest run -E 'test(case_lifecycle_e2e_decided)'` | PASS | r.* schema not captured/restored |
| V5 | `BREHON_E2E_NO_TEMPLATE=1 cargo nextest run --test e2e` | All PASS, wall ≈ 26 min | Rollback path broken |
| V6 | `rg pg_restore: <log dir>` | Per-test log lines visible | Observability broken |
| V7 | `rg pg_dump: <log dir>` | One bootstrap line per nextest process | Bootstrap not running once |
| V8 | `cargo clippy --workspace --features full --no-deps -- -D warnings` | exit 0 | Lint regression in helper module |
| V9 | `cargo check --workspace --features full --tests` | exit 0 | Compile broken |

Run V8 + V9 at every commit; V1-V7 at PR-open + after every fix-in-PR
commit.

## 8. Risks + mitigations

- **pg_dump version skew vs container.** pgautoupgrade:18-alpine ships
  PG 18; vcpkg's libpq is bundled with PG 17 client tools. Mitigation:
  use `--format=custom` (back-compatible across minor versions); test
  with `pg_dump --version` matching `pg_restore --version`.
- **pg_restore --single-transaction failure mid-test.** A failure leaves
  the container with no schema; subsequent tests on the same container
  would fail. Mitigation: under testcontainers-rs each test gets its
  own fresh container, so a failed restore aborts the whole test
  cleanly (the container is dropped, next test gets a new one).
- **`tokio::sync::OnceCell` contention under nextest parallelism.**
  Each nextest process is its own OS process with its own LazyLock,
  so contention is per-process (one bootstrap per process). With
  `threads-required=4`, four bootstraps run in parallel; vcpkg's
  Postgres binaries handle this fine.
- **Replaceable schema (`r.*`) regression.** If `pg_dump --schema=r`
  fails to capture trigger functions, tests that exercise governance_log
  triggers will fail. Mitigation: V4 specifically targets a
  trigger-dependent test.
- **Windows path / quoting issues for shelled commands.** Mitigation:
  `tokio::process::Command::args(&[...])` handles arg quoting; PGPASSWORD
  via env avoids interactive auth.

## 9. Out of scope

- Schema-isolation-per-test (option 3). Declined for the
  search_path-discipline regression risk.
- Multi-stage docker commit (option 1). Declined for initdb.d gotcha.
- Pre-baked Docker image variants. The Dockerfile.test-pg scaffolding
  is removed in Phase 4.1.
- Refactoring `LazyLock<Settings>` away from `e2e.rs`. Nextest's
  process-per-test isolation handles this; out of scope for this plan.
- Updating `.github/workflows/cargo-test-e2e.yml`. Per the new local-
  validation-only policy, dispatch e2e is audit-trail-only; bringing
  the dump path to CI is a separate decision.
- Cross-platform (Linux/macOS) verification. Lemmy upstream CI tests
  on Linux runners; this plan ships Windows-first because that's
  where the laptop runs. Cross-platform validation is a future-work
  item if a contributor uses a non-Windows dev box.

## 10. Definition of Done (§15)

Each Phase task carries its own DoD; the plan-level DoD is:

- [ ] All 6 commits land on `chore/tooling-tier3-pg-template`
- [ ] PR opened against `governance-v0`
- [ ] CodeRabbit review addressed (four-bucket triage)
- [ ] V1-V9 verification matrix all green at merge time
- [ ] `feedback_local_validation_cycle_2026_05_02.md` updated with
      actual measurement in Phase 4.2
- [ ] Retro authored at sub-phase close per
      `feedback_retro_not_report.md` discipline

## 11. Dispatch (Junior task descriptions)

Plan executes as a SINGLE advisor-driven implementation session
(no Junior dispatch — `e2e.rs` is in the no-Junior-Edit zone per
`feedback_junior_worker_e2e_edit_hang.md`).

Tasks in order:
1. Phase 0 pre-flight (5 min, advisor-side)
2. Phase 1 (Tasks 1.1 + 1.2) — single sitting, ~30 min coding
3. Phase 2 (Tasks 2.1 + 2.2) — ~10 min
4. Phase 3 (Tasks 3.1 + 3.2 + 3.3) — measurement runs, mostly waiting
5. Phase 4 (Tasks 4.1 + 4.2) — cleanup + lesson
6. PR open + CR triage + merge

Total estimated wall-clock: 1 advisor session (~3-4 hours including
the measurement runs).

## 12. Out-of-band cargo handling

This plan is **pre-Shape-G** (laptop-side cargo). Validation runs on
the laptop via the existing wrappers (`scripts/brehon/cargo-nextest.bat`
or `cargo-clippy.bat`). No `validate-pending-laptop` DQ entries needed
because the implementation session IS the advisor session — the advisor
runs the cargo commands directly inline with the implementation.

If/when this work needs Junior dispatch (it shouldn't — e2e.rs Edit
hang risk per the lesson), `validate-pending-laptop` would apply.
