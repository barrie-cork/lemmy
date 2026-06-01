---
phase: v1-quality-r3
role: planning
n: 1
authored: 2026-05-30
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: governance-v0
plan_output: .claude/PRPs/plans/v1-quality-r3.plan.md
---

# [role:planning] v1-quality-r3 — EnvVarGuard C4 sweep (12 LEMMY_INITIALIZE + 11 GOVERNANCE_LOG_SIGNING_KEY sites) + issue #167 (admin_audit_stream context.database_url() fix)

## 1. Role + dispatch line

`[role:planning]` Author `.claude/PRPs/plans/v1-quality-r3.plan.md` (the implementation plan for v1-quality-r3). This is a planning task — produce a plan file only; do NOT author implementation code or commit anything to `crates/`.

## 2. Scope

### What to produce

A complete plan file at `.claude/PRPs/plans/v1-quality-r3.plan.md` conforming to the template at `.claude/PRPs/templates/plan.template.md`.

### Primary scope

**C4 follow-on sweep — ~23 remaining raw `std::env::set_var` calls in `e2e.rs`:**

Two env vars that still use raw `unsafe { std::env::set_var(...) }` without `EnvVarGuard`:
- `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` — 12 raw set_var instances (lines 833, 2568, 3347, 4114, 4459, 4788, 4923, 5047, 5675, 5876, 6146, 16823)
- `GOVERNANCE_LOG_SIGNING_KEY` — 11 raw set_var instances (lines 834, 2569, 3348, 4116, 4461, 4790, 5048, 5677, 5878, 6147, 16824)

These appear in pairs at the same call sites (12 locations total). The `EnvVarGuard` struct is already at the test-crate root (hoisted by T4 of v1-quality-r2 — verify its location on the base-branch tip before task numbering). Line 17474-17475 already use `EnvVarGuard` for these two vars — do NOT re-wrap those.

MIRROR reference for the wrapping pattern: `e2e.rs` lines 17474-17475 (already-wrapped instances) and `crates/server/tests/e2e.rs:10867-11053` (BREHON_DISABLE_APPEAL_WINDOW_JOB pattern, non-EnvVarGuard but shows guard-held-in-test-body discipline).

**Fixture-lifetime discipline (MANDATORY):** Per `feedback_envvarguard_fixture_lifetime_footgun.md` — any site wrapped inside a `bootstrap()` or fixture-helper function MUST either (a) return the guard to the caller, or (b) use raw `set_var` with a SAFETY comment if intentionally process-scoped. The plan MUST call this out in §4 watchpoints for any fixture-body wrap sites identified.

### Secondary scope

**Issue #167 — `admin_audit_stream` context.database_url() fix:**

Fix `admin_audit_stream.rs:125` to resolve the LISTEN connection string from `context` rather than re-reading `LEMMY_DATABASE_URL` from the process environment at request time. Proposed fix shape (from issue #167):

- **(A, resolved — clarify DQ a3d0e9941441-039)** `LemmyContext::create` calls `SETTINGS.get_database_url()` internally at construction (same read `build_db_pool` at `crates/diesel_utils/src/connection.rs:177` makes), stores result as `db_url: String` field, exposes via `pub fn database_url(&self) -> &str`. Zero callsite changes — no new parameter added to `LemmyContext::create`. Admin_audit_stream then calls `context.database_url()` instead of `context.settings().get_database_url()`.
- Files touched: `crates/api/api_utils/src/context.rs` (new `db_url: String` field + `database_url()` accessor), `crates/api/api/src/governance/admin_audit_stream.rs:125` (change call site).
- The plan must note ALL `LemmyContext::create` callsites (context.rs:79, server/src/lib.rs:210, e2e.rs:854+) in §13 IMPLEMENT — they require NO changes (zero-parameter-change approach confirmed by clarify pass).
- This task is independent of the e2e.rs sweep (different files, no YAML overlap) and MAY be marked `[P]` with a non-e2e sweep task IF file overlap allows.

### Out of scope (premature-DRY gate)

**Issue #158 — `emit_reputation_event` helper extraction:** EXCLUDED. Grep confirms only 1 handler (`submit_jury_vote.rs`) calls the shared `emit_reputation_event`; `admin_emergency_remove.rs` has its own local `emit_reputation_event_local`. Premature-DRY gate: 2 total implementations, not 3 consumers → defer. Note this decision in plan §3 with the grep evidence.

### Boundaries

- Do NOT author migration files under `crates/db_schema/migrations/` (scope violation — this phase has no schema changes).
- Do NOT modify `Cargo.toml`, `Cargo.lock`, or `rust-toolchain.toml`.
- e2e.rs tasks are serial by YAML overlap (same file). Issue #167 task may be `[P]` with non-e2e tasks.
- Code changes only in: `crates/server/tests/e2e.rs`, `crates/api/api_utils/src/context.rs`, `crates/api/api/src/governance/admin_audit_stream.rs`.

## 3. Required reading

### Plan template and sibling plans

1. `.claude/PRPs/templates/plan.template.md` — canonical plan structure; follow exactly.
2. `.claude/PRPs/plans/v1-quality-r2.plan.md` — MIRROR for task structure, §13 task-shape, §15 DoD, §16a stories. This phase's tasks are the same shape as T3/T4/T5 from quality-r2.
3. `.claude/PRPs/plans/v1-redaction-r1.plan.md` — MIRROR for a phase that touches both production code and e2e.rs (the `[P]` cohort discipline example).

### Always-applicable lessons

4. `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — mandatory for any e2e.rs-targeting brief; the plan must note that impl briefs MUST pre-locate verbatim `old_string`/`new_string` anchors from the phase-branch tip before editing.
5. `.claude/lessons/feedback_envvarguard_fixture_lifetime_footgun.md` — RAII guard in bootstrap() drops on return; plan §4 must flag fixture-body wrap sites.
6. `.claude/lessons/feedback_gate4_full_e2e_env_refactor_class.md` — gate-4 full-e2e mandatory for env-var-management refactor class; include in §15 DoD.
7. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — any new handler code must use `LemmyResult<()>` with `?`.
8. `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — if issue #167 fix introduces any DB writes, they need transactions (check: the fix only changes connection acquisition, likely no new writes).
9. `.claude/lessons/feedback_async_pool_test_pattern.md` — fixture/pool conventions for any new test.
10. `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md` — plan §15 DoD cargo commands must use `scripts\brehon\cargo-*.bat` wrappers on Windows. (clarify DQ a3d0e9941441-041)

### Issue context

11. GitHub issue #167 text (reproduced in §2 above for reference; planner must read `crates/api/api_utils/src/context.rs` and `crates/utils/src/settings/mod.rs:49-55` to understand the construction chain). Fix shape confirmed in clarify DQ a3d0e9941441-039: option A (internal capture at construction, no new parameter).

### ADRs (confirm no scope violation)

11. `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` — confirm plan stays within v0 scope (no new endpoints, no schema changes).

## 4. Constraints

### Planning constraints

1. **Plan §13 tasks**: Task 0 is a non-`[P]` probe task that enumerates: (a) exact line numbers of all 12 raw `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` + 11 `GOVERNANCE_LOG_SIGNING_KEY` set_var calls from the `phase-v1-quality-r3` tip, (b) the current location of `EnvVarGuard` in `e2e.rs` (confirming T4's hoist is present), (c) all `LemmyContext::create` callsites in `crates/`. This is a read-only probe — no code changes in Task 0.

2. **e2e.rs tasks are serial** (same-file YAML overlap rule). The sweep may be split into sub-tasks by test-module grouping (e.g., T1: lines 833-3348, T2: lines 4114-6147, T3: line 16823) but each sub-task is non-`[P]` due to file overlap.

3. **Issue #167 task** (let's call it T_167): may be `[P]` with a non-e2e sweep task (no file overlap with `e2e.rs`). Must enumerate ALL `LemmyContext::create` callsites in §13 IMPLEMENT to confirm scope.

4. **§15 DoD MUST include:**
   - `cargo check --workspace` (exit 0)
   - `cargo clippy --workspace -- -D warnings` (exit 0)
   - `cargo test --workspace --test e2e --features full` (gate-4 full-e2e, exit 0) — with `scripts\brehon\cargo-test.bat` wrapper on Windows
   - `grep -n "LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS\|GOVERNANCE_LOG_SIGNING_KEY" crates/server/tests/e2e.rs | grep "set_var" | grep -v "EnvVarGuard"` — must return 0 lines (all sites wrapped)
   - `grep -n "get_database_url" crates/api/api/src/governance/admin_audit_stream.rs` — must return 0 lines (no env-var re-read at request time)

5. **§4 watchpoints must cite specific files/lines** (not concepts). Per `feedback_advisor_watchpoint_specificity.md`.

6. **DQ mid-task push discipline**: any DQ entry raised during planning must be committed and pushed to the planning branch immediately (not deferred to finalize).

7. **No `kind: "clarify"` from this task** — that kind is advisor-only. Use `kind: "blocker"` from `from: "planner"` for any ambiguity.

8. **Issue #158 exclusion must be documented in plan §3** with the grep evidence (`grep -n "emit_reputation_event" crates/` confirmed 1 caller + 1 local-variant; 2 total implementations, not 3 consumers).

### impl-task brief discipline (for planner's §3 brief-injection notes)

Per `advisor-orchestrator.md` §2.4, any impl-task brief the planner flags as needed MUST include in its §3:
- All e2e.rs-editing briefs: `feedback_fix_impl_pre_locate_e2e_anchors.md`, `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`, `feedback_envvarguard_fixture_lifetime_footgun.md`, `feedback_gate4_full_e2e_env_refactor_class.md`
- T_167 brief: `feedback_lemmy_error_no_std_error.md`, `feedback_multi_write_handlers_need_transactions.md` (confirm no new writes first)

### Attribution

- Plan file commit subject: `docs(plan): v1-quality-r3 planning — EnvVarGuard C4 sweep + issue #167 admin_audit_stream fix`
- Commit only `.claude/PRPs/plans/v1-quality-r3.plan.md` and any DQ entries (in `decision-queue.json`). Do NOT commit code under `crates/`.
- `answered_by` in any DQ entries: `"planner"` (never `"advisor"` from a non-advisor session).

## 5. Pre-flight before authoring

Before drafting the plan, the planner must:
1. Read `crates/api/api_utils/src/context.rs` to understand `LemmyContext` construction — confirm the URL is NOT currently stored (clarify DQ a3d0e9941441-039 resolved: add `db_url: String` field and `database_url()` accessor, captured via `SETTINGS.get_database_url()` inside `create`).
2. Read `crates/utils/src/settings/mod.rs:49-55` — confirm `get_database_url()` reads the env var at call time.
3. Run conceptually: `grep -n "LemmyContext::create\|LemmyContext::new" crates/` to enumerate callers.
4. Confirm the `EnvVarGuard` struct is at e2e.rs test-crate root (before the first `mod *_fixtures` block) — line number may differ from v1-quality-r2 because subsequent commits may have shifted it.
5. Confirm no migration files exist under `crates/db_schema/migrations/` for this branch (issue #167 fix is pure Rust, not schema).
