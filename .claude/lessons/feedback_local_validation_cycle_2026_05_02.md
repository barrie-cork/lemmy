---
name: Local validation cycle speedup (2026-05-02)
description: Drop cargo check, prefer cargo nextest, use rust-lld, pre-bake Postgres fixtures image — for the brehon-fork local-only validation policy.
type: feedback
---

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

## Rule 4: Pre-bake the Postgres fixtures image

Each e2e test currently spawns a fresh `pgautoupgrade:18-alpine` container, then runs ~40 Diesel migrations + replaceable schema setup. The migrations alone cost ~60-90s per test.

A pre-baked image (built once, cached by Docker, keyed on `sha256(migrations/)`) ships with the schema already applied. Per-test container startup drops to the time it takes Postgres to start accepting connections — ~5-15s.

**Why:** testcontainers-rs 0.27 supports `GenericBuildableImage` with `with_skip_if_exists(true)` — thread-safe under nextest parallelism. The image tag = migrations hash, so Docker rebuilds only when migrations change. No drift risk.

**How to apply:**
- `scripts/brehon/Dockerfile.test-pg` — builds from `pgautoupgrade:18-alpine`, runs `apply-migrations-to-image.sh` in the initdb.d phase.
- `scripts/brehon/apply-migrations-to-image.sh` — replicates `governance_fixtures::apply_all_schema` (acquire `pg_advisory_lock(0)`, run migrations in order, rebuild `r` schema, install replaceable schema utils + triggers).
- `governance_fixtures::start_postgres` (in `e2e.rs`) replaces `GenericImage::new(...)` with `GenericBuildableImage::new("brehon-pg-fixtures", &migrations_hash())`. The function signature `(ContainerAsync<...>, u16)` is preserved.
- `governance_fixtures::apply_all_schema` gets a fast-path probe (check for `governance_log` table existence) — skips re-running migrations on the pre-baked image. Backwards compatible with tests that build their own container.

**Generalises to:** any test suite that runs identical schema setup per test against a containerised database.

**Symptom to recognise:** test logs showing "applying migration 2025-..." 14 times in a row before any test logic runs.

## Inner-loop tooling

Also as part of this lesson: **`bacon` over `cargo-watch`**. `cargo-watch` has been in maintenance-only mode since September 2024. `bacon` is the actively maintained successor. Workspace `bacon.toml` defaults to `cargo clippy` on save (Rule 1), with `bacon test-smoke` and `bacon test-full` jobs available.

## See also

- `feedback_pq_sys_stale_cache.md` — the libpq env wiring that all `scripts/brehon/cargo-*` wrappers replicate.
- `feedback_features_full_p_crate_incompatible.md` — `--features full` requires `--workspace`; never combine with `-p <crate>`.
- `feedback_clippy_no_deps_uniform.md` (referenced by `cargo-clippy.sh`) — `--no-deps` keeps clippy output focused on workspace code.
- `feedback_e2e_local_or_dispatch_user_choice.md` — the gate that makes laptop the default e2e runner; this lesson is its execution-side companion.
