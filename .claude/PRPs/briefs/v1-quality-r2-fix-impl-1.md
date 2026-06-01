# v1-quality-r2 fix-impl-1 — restore process-lifetime LEMMY_DATABASE_URL in 2 fixture bootstraps (e2e regression)

[role:impl-task] v1-quality-r2 fix e2e admin_audit_stream regression — see this brief

## Context (why)

The full-e2e gate caught a deterministic regression introduced by THIS phase's T5 (#160, commit `077dadd61`).
T5 wrapped 13 `LEMMY_DATABASE_URL` setter sites with `EnvVarGuard::set(...)` (RAII). **2 of those sites are inside
fixture `bootstrap()` helpers** whose return tuple does NOT carry the guard — so the guard drops the instant
`bootstrap()` returns, unsetting `LEMMY_DATABASE_URL` before the test body runs. The `admin_audit_stream` handler
re-reads that env var LAZILY (via `Settings::get_database_url()`) AFTER bootstrap returns, to open its raw
`tokio_postgres` LISTEN connection → connect fails → 2 tests fail:
`admin_audit_stream_emits_frame_on_config_change`, `admin_audit_stream_enforces_per_admin_cap`.

Full RCA: `.claude/PRPs/debug/qr2-envguard-rca.md`. Architectural follow-up (resolve LISTEN URL from `context`,
not the env var) is tracked separately in issue **#167** — OUT OF SCOPE here.

## Scope

**Restore the pre-#160 process-lifetime set in EXACTLY these 2 fixtures, nothing else.** Replace the
function-scoped RAII guard with a raw `unsafe { std::env::set_var(...) }` (the literal pre-T5 form). Return types
and all 82 caller destructures stay UNCHANGED.

### Edit 1 — `governance_fixtures::bootstrap` (e2e.rs ~839)

The guard line is preceded by a **bare** `let db_url = db_url(host_port);` (no `super::` prefix) — use that to
disambiguate from the identical line in Edit 2.

`old_string` (match exactly, includes the preceding 2 lines for uniqueness):
```
    let (container, host_port) = start_postgres().await?;
    let db_url = db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```
`new_string`:
```
    let (container, host_port) = start_postgres().await?;
    let db_url = db_url(host_port);
    // Intentional process-lifetime set (NOT an EnvVarGuard): `admin_audit_stream`
    // re-reads LEMMY_DATABASE_URL via `Settings::get_database_url()` LAZILY, AFTER
    // this fixture returns, to open its raw tokio_postgres LISTEN connection. A
    // function-scoped RAII guard drops the var on return and breaks that lazy
    // re-read (the v1-quality-r2 e2e regression). Restores pre-#160 behaviour.
    // The architectural fix (resolve the LISTEN URL from `context`) is issue #167.
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }
```

### Edit 2 — `admin_config_fixtures::bootstrap` (e2e.rs ~6152)

The guard line here is preceded by a `super::governance_fixtures::`-prefixed `db_url` line — use that to
disambiguate from Edit 1.

`old_string` (match exactly):
```
    let (container, host_port) = super::governance_fixtures::start_postgres().await?;
    let db_url = super::governance_fixtures::db_url(host_port);
    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```
`new_string`:
```
    let (container, host_port) = super::governance_fixtures::start_postgres().await?;
    let db_url = super::governance_fixtures::db_url(host_port);
    // Intentional process-lifetime set (NOT an EnvVarGuard): `admin_audit_stream`
    // re-reads LEMMY_DATABASE_URL via `Settings::get_database_url()` LAZILY, AFTER
    // this fixture returns, to open its raw tokio_postgres LISTEN connection. A
    // function-scoped RAII guard drops the var on return and breaks that lazy
    // re-read (the v1-quality-r2 e2e regression). Restores pre-#160 behaviour.
    // The architectural fix (resolve the LISTEN URL from `context`) is issue #167.
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }
```

### DO NOT touch
- The 10 test-body-level `EnvVarGuard::set("LEMMY_DATABASE_URL", ...)` sites (correct scope — guard lives for the
  whole test body).
- `boot_context()` (v1_rt_r3_fixtures) — already returns its guards correctly.
- Any caller destructure — return types are unchanged, so callers compile as-is.
- `admin_audit_stream.rs` or any production code — that's issue #167.

## Required reading

1. `.claude/PRPs/debug/qr2-envguard-rca.md` — the full root-cause analysis (READ FIRST).
2. `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate the verbatim `old_string` anchors in
   the live file BEFORE editing (2 edits in e2e.rs). The two `old_string`s above are NEAR-IDENTICAL; the ONLY
   disambiguator is the preceding `db_url` line (`db_url(host_port)` vs `super::governance_fixtures::db_url(host_port)`).
   If your Edit fails on a non-unique match, you anchored on the guard line alone — include both preceding lines.
3. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — e2e.rs error-shape conventions (mandatory for any e2e.rs edit).
4. `.claude/lessons/feedback_async_pool_test_pattern.md` — e2e fixture/pool conventions (mandatory for any e2e.rs edit).

## Constraints

- Commit on `phase-v1-quality-r2` ONLY. One commit, subject: `fix(e2e): restore process-lifetime LEMMY_DATABASE_URL in fixture bootstraps (#160 regression)`.
- This is a 2-edit, ≤30-line change. Do NOT scope-creep into the 10 test-body sites or production code.
- After edits: run the impl-task's own `cargo check` validation gate per §13 DoD. **Do NOT run e2e** (that's the
  laptop-side advisor gate — full e2e on this Windows laptop, ~40 min).
- attribution: impl-task work; never write `answered_by: advisor`. DQ mid-task push if blocked.
- MIRROR-ref: the `unsafe { std::env::set_var(...) }` block must match the style of the OTHER raw set_var blocks
  already in these same fixtures (e.g. the `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` / `GOVERNANCE_LOG_SIGNING_KEY`
  block at the top of each bootstrap — same indentation, same SAFETY comment style).

## Validation gate (advisor runs after, on laptop)
1. `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"` = exit 0 (impl-task runs this).
2. `cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` = exit 0 (advisor; watch EnvVarGuard not-dead-code — it's still used at 10+ sites).
3. Full e2e (advisor, laptop): `126 passed; 0 failed; 5 ignored` — the 2 admin_audit_stream tests now PASS.
