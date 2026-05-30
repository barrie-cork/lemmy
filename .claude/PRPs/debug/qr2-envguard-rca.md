# v1-quality-r2 — EnvVarGuard lifetime regression RCA + fix scope

**Date:** 2026-05-30
**Trigger:** full-e2e (gate-4) run #1 + isolation rerun — 2/126 deterministic fails.
**Failing tests:** `admin_audit_stream_emits_frame_on_config_change`, `admin_audit_stream_enforces_per_admin_cap`.
**Error (both):** `LemmyError { Unknown("tokio_postgres connect failed: db error"), caller: admin_audit_stream.rs:130:89 }`

## Root cause (CONFIRMED)

T5 (#160, commit `077dadd61`) replaced `unsafe { std::env::set_var("LEMMY_DATABASE_URL", &db_url) }` with
`let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url)` at 13 sites. **2 of those sites are inside
fixture `bootstrap()` helpers** that return a 3-tuple `(container, context, db_url)` WITHOUT the guard:

| Fixture | def line | guard site | callers (verified ×3 methods) |
|---|---|---|---|
| `governance_fixtures::bootstrap` | e2e.rs:827 | e2e.rs:839 | **49** |
| `admin_config_fixtures::bootstrap` | e2e.rs:6138 | e2e.rs:6152 | **33** |
| **TOTAL** | | | **82 caller destructures** |

`EnvVarGuard` is RAII: `Drop` restores prev (here `None`) → **removes** `LEMMY_DATABASE_URL`. Because the guard is a
function-scoped local NOT returned to the caller, it drops the instant `bootstrap()` returns — so the env var is
**unset before the test body runs**.

**Why only 2 tests break:** `Settings::get_database_url()` (utils/settings/mod.rs:49-55) reads `env::var("LEMMY_DATABASE_URL")`,
else falls through to `self.database.connection` (the localhost default — no PG there). The DB **pool** is built eagerly
*inside* `bootstrap()` while the guard is alive, so `context.pool()` works for every test. The ONLY production handler that
**lazily re-reads** `get_database_url()` after bootstrap returns is `admin_audit_stream` (its raw `tokio_postgres::connect`
LISTEN channel at :125-126). Confirmed: `get_database_url()` appears in `crates/api` only in `admin_audit_stream.rs`.
So only the 2 SSE tests that reach that connect line fail. (`admin_audit_stream_forbidden_for_non_admin` passes — `is_admin()`
rejects before the connect.)

**Why check/clippy/e2e-no-run passed:** all compile-only. The defect is purely runtime (env var lifetime). Only a full
e2e RUN surfaces it. → strong retro signal: gate-4 full-e2e is load-bearing; the env-var-hygiene refactor self-regressed.

## The OLD behaviour was a load-bearing leak

Pre-T5, `bootstrap()` did `unsafe { set_var(...) }` with NO guard — so the env var **persisted for the process lifetime**
(leaked across tests, but always SET). The SSE tests depended on that leak. T5 "fixed" the leak and broke the lazy re-read.

## NOT in scope (verified correct already)

- The 10 test-body-level `_g_db_url = EnvVarGuard::set(LEMMY_DATABASE_URL)` sites (2575, 3353, 4123, 4468, 4797, 4928,
  5062, 5108, 5684, 5885) — guard lives for the whole test body = correct scope. Leave untouched.
- `boot_context()` (v1_rt_r3_fixtures, site 17475) — T4 already returns `Vec<EnvVarGuard>`. Correct. Leave untouched.
- site 16826 is a test body (`two_sponsors_lose_endorsement_strength_on_sanction`), not a fixture. Correct scope. Leave.
- `v1_ship_3_fixtures` has NO `bootstrap()` fn (0 callers). Not involved.

## Fix (chosen: minimal restore — pre-#160 behaviour; user-confirmed 2026-05-30)

Restore the raw process-lifetime set in the **2 defective fixtures only**. The SSE handler re-reads the env var lazily
after bootstrap returns, so the var must outlive bootstrap — i.e. process-lifetime, exactly as pre-#160. Use a raw
`unsafe { std::env::set_var(...) }` (the literal pre-T5 form that passed clippy for months — avoids any
`clippy::mem_forget` lint risk). 2 edits, **return types + all 82 callers UNCHANGED.**

```rust
// governance_fixtures::bootstrap (e2e.rs:839) AND admin_config_fixtures::bootstrap (e2e.rs:6152)
-    let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
+    // Intentional process-lifetime set (NOT an EnvVarGuard): `admin_audit_stream`
+    // re-reads LEMMY_DATABASE_URL via `Settings::get_database_url()` LAZILY, AFTER
+    // this fixture returns, to open its raw tokio_postgres LISTEN connection. A
+    // function-scoped RAII guard drops the var on return and breaks that lazy
+    // re-read (the v1-quality-r2 e2e regression). This restores the pre-#160
+    // process-lifetime behaviour. The architectural fix (resolve the LISTEN URL
+    // from `context`, not the env var) is tracked in issue #168.
+    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
+    unsafe {
+      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
+    }
```

**Clippy watch:** after this edit, `EnvVarGuard` may have one fewer use site. It is STILL used (10 test-body sites +
boot_context) so it will NOT become dead code — no `dead_code` warning expected. Confirm at clippy gate.

## Validation gate for the fix
1. `cmd //c "scripts\brehon\cargo-check.bat --workspace --features full"` = exit 0
2. `cmd //c "scripts\brehon\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` = exit 0
3. Full e2e (gate-4) — the 2 admin_audit_stream tests now PASS, suite returns `126 passed; 0 failed; 5 ignored`.

## Verdict
T5 regression. Minimal restore = 2 edits in e2e.rs (same file T5 owns), zero caller churn, provably pre-#160 behaviour.
Architectural follow-up (DB URL from `context`) tracked in **issue #168** (filed on trunk `c328d9d34`). Advisor
dispatches a fix-impl-task; advisor never authors crate code.
