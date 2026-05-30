---
name: EnvVarGuard fixture lifetime footgun
description: RAII guard acquired inside a fixture bootstrap() that doesn't return the guard drops on function return, unsetting the env var before the test body runs
type: feedback
originSessionId: v1-quality-r2-retro
---

When a fixture `bootstrap()` helper acquires an `EnvVarGuard` for an env var and returns only a partial result (e.g., a tuple without the guard), the guard drops at the end of `bootstrap()` — unsetting the env var before the test body ever runs. The pre-guard raw `unsafe { set_var(...) }` was a load-bearing process-lifetime "leak" that downstream handlers (e.g., `admin_audit_stream`'s lazy `get_database_url()` re-read) depended on.

**Root cause (v1-quality-r2 incident):** T5's 13-site LEMMY_DATABASE_URL wrap introduced `EnvVarGuard` in 2 `admin_audit_stream` fixture `bootstrap()` helpers but returned a 3-tuple, dropping the guard. The `admin_audit_stream` handler reads `Settings::get_database_url()` lazily (not at connection time) — so the LISTEN channel URL was fetched after the guard had already dropped. Gate-4 full-e2e caught 2 test failures that cargo check, clippy, and `--no-run` could not detect.

**Forward discipline:** Any `bootstrap()` helper that sets an env var via `EnvVarGuard` MUST either:
- (a) **Return the guard to the caller** — so the guard's lifetime extends into the test body (like `boot_context()` after T4's hoist).
- (b) **Document explicitly** that the var is intentionally process-scoped and switch to raw `unsafe { std::env::set_var(...) }` with a `// SAFETY: process-scoped; handler reads lazily` comment.

**How to apply:** Before writing any impl-task brief targeting `e2e.rs` bootstrap helpers, check if any targeted bootstrap()-call returns the guard. If not, flag as a catch-fire condition in the brief and require the worker to choose (a) or (b) explicitly. See also `feedback_gate4_full_e2e_env_refactor_class.md` — this footgun is invisible to compile-only gates.

**Source:** v1-quality-r2 retro + fix-impl-1 cycle (commit `75ca61251`). The actual fix used approach (a) at the test-body level: guards held directly in each of the 3 `admin_audit_stream` test bodies for the duration of each test.
