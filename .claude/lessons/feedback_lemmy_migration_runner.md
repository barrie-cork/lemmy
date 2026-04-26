---
name: Lemmy migration runner — not raw diesel CLI
description: In this workspace, raw `diesel migration run/redo/revert` is blocked by an upstream forbid-trigger; use `cargo run -p lemmy_diesel_utils --features full` or call `lemmy_diesel_utils::schema_setup::run` from a test
type: feedback
originSessionId: 98b0fb05-df8a-4d08-a152-27b3f01323e4
---
Migration `2025-08-01-000017_forbid_diesel_cli` (upstream Lemmy) installs a trigger on `__diesel_schema_migrations` that raises `'migrations must be managed using lemmy_server instead of diesel CLI'` on any insert/update/delete unless `pg_advisory_lock(0)` is held by the current session. Raw `diesel migration run`, `diesel migration redo`, and `diesel migration revert` do not take that lock, so all three fail on this workspace.

**Why**: Phase 1 task 10 hit this when the plan's instruction to `diesel migration run` after standing up a fresh container failed with the forbid-trigger error. Spent one iteration cycle diagnosing before landing on the right runner.

**How to apply**:
- To apply migrations against a scratch container, use `cargo run -p lemmy_diesel_utils --features full` with `LEMMY_DATABASE_URL=postgres://...` set. The binary wraps `lemmy_diesel_utils::schema_setup::run(Options::default().run(), &url)` and that function acquires `pg_advisory_lock(0)` at `crates/diesel_utils/src/schema_setup/mod.rs:214` before running migrations.
- To revert migrations (for testing `down.sql`), call `lemmy_diesel_utils::schema_setup::run(Options::default().revert().limit(N), &url)` directly from a test. `Options` exposes `pub fn run()`, `pub fn revert()`, and `pub fn limit(u64)` chainable — read past `run()` next time, they're right there. The `phase1_migrations_round_trip` e2e test at `crates/server/tests/e2e.rs` is the canonical usage example and should not be broken by any new migration work.
- `diesel print-schema` is fine — it's read-only and doesn't touch `__diesel_schema_migrations`, so the forbid-trigger doesn't apply.
- `diesel_cli` itself is fine to use for introspection (`diesel print-schema`, `diesel --version`). Just not the migration-mutating subcommands.
- Windows gotcha: diesel.exe built inside a vcvars shell needs UCRT on PATH at runtime. Git-bash invocations of the bare binary fail with `api-ms-win-crt-heap-l1-1-0.dll not found`. Always invoke diesel.exe from inside a cmd+vcvars wrapper batch script, never directly from git-bash.
