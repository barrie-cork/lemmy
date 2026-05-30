---
name: Gate-4 full e2e mandatory for env-var-management refactor class
description: Compile-only gates (check, clippy, --no-run) cannot catch runtime env-var-lifetime regressions; gate-4 full-e2e run is mandatory for any refactor that touches env-var acquisition, guard scoping, or fixture bootstrap helpers
type: feedback
originSessionId: v1-quality-r2-retro
---

**Pattern:** Any refactor that changes how env vars are set or scoped in `e2e.rs` fixtures — including wrapping raw `set_var` in `EnvVarGuard`, hoisting guards, or changing fixture helper signatures — is structurally invisible to compile-only validation. `cargo check`, `cargo clippy`, `cargo test --no-run`, and `cargo test -p ... --features full -- --test-thread=1 <single-test>` all exit 0 even when the guard's RAII lifetime mismatches the handler's read timing.

**Why:** RAII guard lifetimes and env-var read timing are runtime properties — the compiler enforces that the guard type is used correctly, not that it's held for the right duration relative to a handler's internal read schedule. A guard that drops at function-return is valid Rust; it's only wrong in the context of a handler that reads the env var lazily after the function returns.

**Gate-4 requirement:** For any impl-task that:
1. Wraps `unsafe { set_var(...) }` calls in `EnvVarGuard` in `e2e.rs`, OR
2. Changes the signature of a fixture `bootstrap()` helper to add/remove a guard return value, OR
3. Hoists or restructures `EnvVarGuard` acquisition into or out of fixture functions,

…the plan §15 DoD MUST include a full `cargo test --workspace --test e2e --features full` run (gate-4) before merge. A validate-pending-laptop entry with `--no-run` is NOT sufficient.

**How to apply:** At plan-authorship time, if the plan includes any of the three trigger patterns above, confirm gate-4 is in §15. At brief-authorship time, inject `feedback_gate4_full_e2e_env_refactor_class.md` + `feedback_envvarguard_fixture_lifetime_footgun.md` into §3 Required reading. See also `.claude/rules/advisor-orchestrator.md` §3.2 user gate 4.

**Source:** v1-quality-r2 T5 regression — EnvVarGuard refactor self-regressed with 2 `admin_audit_stream` test failures caught only at gate-4 local e2e run (`119/0/5` → `117/2/5`). Full RCA in v1-quality-r2 retro.
