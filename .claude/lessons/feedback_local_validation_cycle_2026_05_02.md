---
name: Local validation cycle speedup (2026-05-02)
description: Drop cargo check, prefer cargo nextest, use rust-lld, pre-bake Postgres fixtures image — for the brehon-fork local-only validation policy.
type: feedback
---

# Local validation cycle speedup (2026-05-02)

The brehon-fork validation cycle runs entirely on the laptop (Windows + MSVC + vcpkg). GitHub Actions is reserved for milestone-PR CodeRabbit review only. This lesson captures four high-leverage local-cycle changes confirmed by Perplexity research 2026-05-02 (saved at `C:\Users\barri\Downloads\I'm working on a Rust monorepo*.md` + `C:\Users\barri\Downloads\I maintain a Rust monorepo*.md`).

## Rule 1: Drop `cargo check`; run `cargo clippy --no-deps` only

`cargo check` followed by `cargo clippy` **double-compiles workspace members**. Clippy stores artifacts under a different fingerprint hash than check (via `RUSTC_WORKSPACE_WRAPPER`), so non-workspace dependencies are reused but the 45-crate workspace itself is re-analyzed. Running clippy alone is a strict superset (full type-checking PLUS lints) and saves the entire check pass.

**Why:** Confirmed bug in cargo's incremental fingerprinting; no stable rustc fix yet. See <https://www.reddit.com/r/rust/comments/114fdf9/psa_clippy_seems_to_break_incremental_cache/>.

**How to apply:**
- Plan DoDs (`.claude/PRPs/plans/*.plan.md` §15) MUST NOT name both `cargo check` and `cargo clippy`. Pick clippy.
- Validation lessons that historically said "run check then clippy" are obsolete. Clippy alone covers it.
- The `scripts/brehon/cargo-check.bat` wrapper is kept (still useful for hand-debugging) but should not appear in DoD chains.

**Generalises to:** any Rust project. The fingerprint divergence is a cargo-wide invariant, not Lemmy-specific.

**Symptom to recognise:** "validation took 10 min when only one crate changed" — the second compile pass is the lost time.

## Rule 2: Use `cargo nextest` for tests with shared singleton state

The historical `--test-threads=1` constraint on `crates/server/tests/e2e.rs` exists because `LazyLock<Settings>` (and `LEMMY_DATABASE_URL`) are cached on first access. Standard `cargo test` runs all tests in one process — once the singleton is initialised, every subsequent test reads the first test's URL.

`cargo nextest` runs **each test in its own process**. `LazyLock` re-initialises per process; each test's `LEMMY_DATABASE_URL` lives in its own address space. The `--test-threads=1` constraint disappears with **zero changes to e2e.rs**.

**Why:** Per <https://nexte.st/docs/design/why-process-per-test/>: process isolation is nextest's core design point. Singletons across tests in the same process is exactly the use case it solves.

**How to apply:**
- Install once: `cargo install cargo-nextest`.
- Use the wrappers `scripts/brehon/cargo-nextest.bat` / `.sh` (same vcvars + libpq env as cargo-test).
- Concurrency caps live in `.config/nextest.toml` (`threads-required`). Default in this repo is 4 (≈2 concurrent containers on 8-core); raise to 8 if Docker Desktop is comfortable.
- `cargo-test.bat`'s `--test-threads=1` injection is fine (still works for legacy `cargo test` callers); just don't pile it on top of nextest.

**Realistic speedup:** 14 e2e tests × ~90s serial = ~21 min. With 4 concurrent processes: ~5-7 min. With pre-baked image (Rule 4): ~2-3 min.

**Generalises to:** any Rust integration test suite that uses `LazyLock`, `OnceLock`, `lazy_static!`, or static `Mutex<>` to cache test-time configuration.

**Symptom to recognise:** test file documents `--test-threads=1` "for race safety" or sets env vars unsafely with comments about "first test wins".

## Rule 3: Use `rust-lld` for the Windows MSVC linker

`rust-lld` is the Rust-vendored LLD, tuned for the toolchain's LLVM version. It ships with rustup and is on PATH by default. On `x86_64-pc-windows-msvc` it is **not yet the default** (issue #71520) but is safe to enable per-project via `.cargo/config.toml`.

**NOT to confuse with:** `lld-link.exe` from system LLVM (winget/scoop install). That's a different binary with a mixed 2024 track record — some Reddit reports show slowdowns vs MSVC `link.exe`.

**Why:** Test binary linking is a non-trivial fraction of the e2e test cycle (~10-30s on a clean cargo test build). Linker is irrelevant for `cargo check`/`cargo clippy` (codegen skipped) so this only helps test compile time. Modrinth's Labrinth (similar Rust/Actix/Diesel stack) adopted this in 2025: <https://github.com/modrinth/code/commit/c899d9550cb5275f2afef16525bfc9a01cbd064a>.

**How to apply:**
- Land `.cargo/config.toml` at workspace root with:

  ```toml
  [target.x86_64-pc-windows-msvc]
  linker = "rust-lld"
  ```

- Rollback: comment the block; no `target/` invalidation needed beyond cargo's normal config-change rebuild.
- **Do not** add `lld-link.exe` from system LLVM unless `rust-lld` doesn't work for some reason — the two are separate codepaths.

**Generalises to:** any Windows Rust project building large test or binary targets.

**Symptom to recognise:** `cargo test --no-run -p lemmy_server` spending >30s in the link phase (visible as a long pause after "Compiling lemmy_server" before tests start).

## Rule 4: pg_dump+restore template database — scaffolding shipped, throughput parity with nextest legacy

**Status:** SHIPPED 2026-05-03 in tooling-local-validation as **scaffolding for future option-3 work**. Wall-clock parity (not speedup) measured against same-runner baseline:

| Path | Wall | Per-test avg | n |
|---|---|---|---|
| nextest, template path (run-4) | **12m 17s** | 22.0s | 67/67 PASS |
| nextest, legacy `BREHON_E2E_NO_TEMPLATE=1` (Phase 3.2) | **12m 02s** | 21.5s | 67/67 PASS |
| `cargo test --test-threads=1` (single-threaded; advisor docs reference) | ~26 min | ~23s | reference only |

The `~26 min` baseline cited in `advisor-orchestrator.md:166/270` and `v1-jury-mechanics-e.plan.md:1030` is the **single-threaded `cargo test`** baseline, NOT a like-for-like nextest run. Against same-runner nextest with `threads-required=4`, the template path runs **~2.1% slower** than legacy on this PG18 + fast-laptop combination (15s of 722s wall, well within run-to-run variance). The premise of the original Tier 3 plan ("migrations cost 60-90s/test, pg_restore costs 1-3s") was wrong on this hardware — both paths cost ~10-20s per test.

**Why ship anyway** — Tier 3 lays the foundation for option-3 (shared-container, per-test schema isolation) which the plan declined for `search_path` regression risk. With template-database scaffolding in place, an option-3 follow-up needs only the per-test `CREATE SCHEMA test_<uuid>` + `pg_restore --schema=` rewriting; the bootstrap, caching, and dump-capture machinery is reusable. **Tier 3 is plumbing, not throughput.** It should be evaluated when a future contributor pursues option-3 OR when the same code runs on a slower environment (e.g. EliteDesk Junior, GitHub Actions runners) where Diesel migration time genuinely dominates pg_restore time.

**Why this pattern over alternatives:**
- **Multi-stage docker commit** (option 1) — initdb.d scripts re-run on every container start (when data dir is empty, which is per-container under default volume management), so the migrations would re-run per-test anyway. Brittle clean-shutdown semantics. Rejected at plan.
- **Shared container per nextest process + per-test schema isolation** (option 3) — fastest in theory (~10-50ms per test) but introduces silent regression risk if Diesel's `MigrationHarness` doesn't honor `search_path`; cross-test state leak invisible until a pool recycles a connection mid-test. Rejected at plan; reconsider given Tier 3's measured null-result.
- **pg_dump+restore template** (option 2, this rule) — proven pattern, no schema-isolation risk, scaffolding shipped. Throughput-neutral on this laptop; likely value on slower environments and as foundation for option-3.

**How to apply (the scaffolding):**
- `governance_fixtures::pg_template` sub-module in `crates/server/tests/e2e.rs`:
  - `ensure_template()` returns cached dump bytes via 3-layer cache:
    1. In-process `tokio::sync::OnceCell<Vec<u8>>` (same process, second test).
    2. On-disk `target/tmp/brehon-pg-template-<exe-mtime>.dump` (cross-process; cache key is the test binary's mtime so any source/migration change invalidates).
    3. Cold bootstrap (vanilla container + apply_all_schema_legacy + pg_dump). ~30s on this laptop.
  - `pg_restore_into(container_id, &dump)` streams bytes via `docker exec -i pg_restore --clean --if-exists --single-transaction --exit-on-error`.
  - `pg_dump` runs via `docker exec` against the live container — eliminates host-side Postgres client toolchain dependency AND guarantees pg_dump version-match with the container (no skew risk).
- **Must include `-e <ext>` for every Postgres extension migrations install** — `pg_dump` does NOT capture extensions by default. `--clean --if-exists` drops the default `public` schema (which cascades the extensions), so the dump must recreate them. As of 2026-05-03 the migrations install: `pgcrypto`, `ltree`, `pg_trgm`. If a fourth lands, append `-e <name>` to the pg_dump args. Symptom of missing extension: `pg_restore: error: type public.<name> does not exist` on first restore.
- `governance_fixtures::apply_all_schema` gains a sentinel short-circuit: `SELECT COUNT(*) FROM information_schema.tables WHERE table_name='governance_log'` returns `Ok(())` on hit. All 21 caller sites continue to work unchanged — the sentinel makes the legacy-or-template choice transparent.
- `governance_fixtures::start_postgres` dispatches template-by-default. `BREHON_E2E_NO_TEMPLATE=1` env var rolls back to legacy (cold migrations) — load-bearing for debugging dump suspicion or schema-change regressions, AND for measuring same-runner baseline.

**Why nextest matters here:** nextest's process-per-test isolation means the in-process `OnceCell` rebuilds for every test (each test = its own process). The disk cache (layer 2) is therefore the load-bearing tier when running under nextest. Without disk cache, every test pays the ~30s cold bootstrap and the template path runs 22% SLOWER than legacy (run-3 measured 20m 15s). Disk cache brought it back to parity (run-4: 12m 17s ≈ legacy 12m 02s).

**Why not actually faster:** on PG18 + this laptop, pg_restore (binary command replay) takes the same wall-clock as Diesel `MigrationHarness::run_pending_migrations` (SQL replay). Both ~10-20s per test. The Tier 3 plan's `1-3s restore` projection was wrong for this hardware. Reaching a real sub-6-min wall would require option 3 (shared-container, per-test schema isolation) — declined initially for search_path risk, worth reconsidering now that template alone is throughput-neutral.

**Generalises to:** any test suite using testcontainers + a containerised database where schema setup is identical per test. The pattern is portable to any Diesel/Sqlx project; the Postgres-extension footgun (`-e` flag) is universal — pg_dump never captures extensions by default in custom format. The throughput-neutral result is hardware-dependent: on slower environments where migration runner time dominates, the template path will actually speed things up.

**Lessons-within-the-lesson worth promoting elsewhere:**
1. **"Baseline" numbers from rules/docs need a literal probe before being used to justify a refactor.** The 26-min number in `advisor-orchestrator.md` was correct for `--test-threads=1` and got mistakenly applied to nextest comparisons. Same class as `feedback_runbook_audit_drift_post_event_check.md`.
2. **Plan §3 wall-clock targets need a measured baseline first.** The Tier 3 plan §3.1 set "<6 min target" against the wrong baseline. A 5-min baseline measurement run before plan approval would have revealed that nextest legacy ALREADY runs in ~12 min and the "<6 min target" requires architectural changes (option 3), not just template caching.

**Symptom to recognise (during use):** test logs showing "applying migration 2025-..." for every test (legacy mode active when template expected), OR pg_restore errors of the form `schema "public" already exists` (need `--clean --if-exists`) or `type public.X does not exist` (missing `-e <ext>` flag).

## Inner-loop tooling

Also as part of this lesson: **`bacon` over `cargo-watch`**. `cargo-watch` has been in maintenance-only mode since September 2024. `bacon` is the actively maintained successor. Workspace `bacon.toml` defaults to `cargo clippy` on save (Rule 1), with `bacon test-smoke` and `bacon test-full` jobs available.

## See also

- `feedback_pq_sys_stale_cache.md` — the libpq env wiring that all `scripts/brehon/cargo-*` wrappers replicate.
- `feedback_features_full_p_crate_incompatible.md` — `--features full` requires `--workspace`; never combine with `-p <crate>`.
- `feedback_clippy_no_deps_uniform.md` (referenced by `cargo-clippy.sh`) — `--no-deps` keeps clippy output focused on workspace code.
- `feedback_e2e_local_or_dispatch_user_choice.md` — the gate that makes laptop the default e2e runner; this lesson is its execution-side companion.
