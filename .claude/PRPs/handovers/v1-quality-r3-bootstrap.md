---
phase: v1-quality-r3
plan: .claude/PRPs/plans/v1-quality-r3.plan.md   # not yet authored
phase_branch: phase-v1-quality-r3                  # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-quality-r3   # created at bm-cut; until then canonical brehon-fork
authored: 2026-05-30
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-quality-r3 advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-quality-r3.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-quality-r3` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane.
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `207d606c8` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 207d606c8..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` for any pending entries since handoff; compare against the §"Decision-queue snapshot" below (empty at handoff).
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_quality_r3.md` is the running-state scratchpad. Read `workflow_state_v1_quality_r2.md` once at this session's start for carry-forward context, then do not re-read.

## Next concrete action

Author `.claude/PRPs/briefs/v1-quality-r3-planning-1.md` (scope: ~28 remaining env-var hygiene sites + issue #167 + issue #158 conditional — see §1) → `/brehon-clarify` → queue planning Junior.

Before authoring the brief, promote the two nominated lessons from v1-quality-r2 retro to `.claude/lessons/`:
1. `feedback_envvarguard_fixture_lifetime_footgun.md` — RAII guard in a fixture `bootstrap()` that doesn't return the guard drops on return; env var unset before test body runs.
2. `feedback_gate4_full_e2e_env_refactor_class.md` — compile-only gates cannot catch runtime env-var-lifetime regressions; gate-4 is mandatory for env-var-management refactor class.
Then run `scripts/sync-lessons-to-pmd.sh` to index them.

---

## 1. v1-quality-r3 in one paragraph

v1-quality-r3 retires the remaining PR #155 carry-forwards that v1-quality-r2 deferred. Primary scope: the ~28 sites across the e2e test suite that set `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` and `GOVERNANCE_LOG_SIGNING_KEY` via raw `unsafe { std::env::set_var(...) }` without `EnvVarGuard` wrapping (C4 follow-on, plan §12 pre-seeded planner-DQ). Secondary: issue #167 — refactor `admin_audit_stream` to resolve its raw `tokio_postgres` LISTEN URL from the injected `context` object rather than re-reading `Settings::get_database_url()` lazily (architectural fix for the env-var dependency that caused the v1-quality-r2 regression). Conditional: issue #158 (`emit_reputation_event` helper extraction) — only if a third reputation-event emitter has materialised since v1-quality-r2 shipped (premature-DRY gate: 2 consumers = defer, 3 = extract). DoD: workspace cargo gates exit 0, full e2e suite passes locally (no regression vs v1-quality-r2 baseline, 119/0/5 or better), all targeted env-var setter sites in `e2e.rs` wrapped by `EnvVarGuard`, `admin_audit_stream` no longer reads the process-global env var for its LISTEN URL.

## 2. Why v1-quality-r3 is easier/harder than v1-quality-r2

**Easier:** The `EnvVarGuard` struct and pattern are already established and visible at the test-crate root (T4 hoisted it). The mechanical sweep pattern (T5 in v1-quality-r2) is now proven and repeatable; the planner has a clear MIRROR reference. Issue #167 has a clear scope (one handler in `crates/api/`).

**Not easier:** Issue #167 touches production code (`crates/api/`), not just `e2e.rs` — it crosses the test/prod boundary and goes through full CR review. The `admin_audit_stream` handler uses `tokio_postgres::connect` directly (not the pool), so the fix requires either passing a `DbPool` reference or resolving the URL from `context.pool()` — the planner needs to decide the shape. The ~28 remaining env-var sites span multiple env var names and may be interleaved with the already-wrapped `LEMMY_DATABASE_URL` sites — verbatim anchors MANDATORY (per `feedback_fix_impl_pre_locate_e2e_anchors.md`).

## 3. Lessons from v1-quality-r2 that apply to v1-quality-r3

**All e2e.rs edits:**
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — mandatory for any e2e.rs-targeting impl-task brief; pre-locate verbatim `old_string`/`new_string` anchors from the live phase-branch tip before writing the brief. This is non-negotiable after the 2× recurrence in v1-RT-r3 + v1-quality-r2.
- `feedback_lemmy_error_no_std_error.md` — e2e error-shape conventions.
- `feedback_async_pool_test_pattern.md` — fixture/pool conventions for any new test fn.

**New (nominated from v1-quality-r2 retro — promote before brief authorship):**
- `feedback_envvarguard_fixture_lifetime_footgun.md` — RAII guard inside `bootstrap()` that isn't returned drops on return and unsets the env var. Forward discipline: any `bootstrap()` helper setting an env var via `EnvVarGuard` MUST either return the guard or document it's intentionally process-scoped (raw `set_var` + SAFETY comment).
- `feedback_gate4_full_e2e_env_refactor_class.md` — compile-only gates (check, clippy, `--no-run`) are structurally blind to RAII guard lifetime mismatches. Gate-4 full-e2e is mandatory for any env-var-management refactor.

**For issue #167 (production code edit):**
- `feedback_multi_write_handlers_need_transactions.md` — any handler doing 2+ DB writes needs a transaction. Check if the #167 fix introduces any new writes.
- `feedback_lemmy_error_no_std_error.md` — any new handler code must use `LemmyResult<()>` with `?` (not `Box<dyn Error>`).

**Advisor discipline (recurrences in v1-quality-r2):**
- Never guess a SHA; always read from `git log` before using a SHA in a command (two SHA hallucinations in v1-quality-r2 caused batch failures and ~30 min recovery each).
- Sequential (not parallel) tool calls when each call's output gates the next git operation.

## 4. v1-quality-r3-specific watchlist

1. **`admin_audit_stream.rs:125-126` (`crates/api/src/governance/admin_audit_stream.rs`)** — the lazy `get_database_url()` read that issue #167 must fix. Plan §13 must show the impl-task IMPLEMENT file as `crates/api/src/governance/admin_audit_stream.rs`; the fix must replace the raw `tokio_postgres::connect(Settings::get_database_url())` call with a connection derived from `context` or an injected pool. Confirm at DoD smoke test that the function no longer calls `get_database_url()`.
2. **`crates/server/tests/e2e.rs` env-var sites — expect ~28 raw `set_var` calls for `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` + `GOVERNANCE_LOG_SIGNING_KEY`** — planner must re-enumerate at plan-time from the phase-branch tip (redaction-r1 lane may have shifted line numbers since v1-quality-r2 captured them). Task 0 probe must confirm the count before dispatch.
3. **`EnvVarGuard` already at test-crate root (hoisted by T4)** — verify it's at the correct scope (before the first `mod *_fixtures` block) on the phase-branch tip before authoring T3-equivalent brief; do NOT re-hoist.
4. **Issue #158 premature-DRY gate** — search `crates/` for `emit_reputation_event` call sites before plan authorship. If count ≥ 3, include #158 in scope; if still 2, exclude and note in plan §3.
5. **Gate-4 full-e2e baseline** — v1-quality-r2 shipped with `119/0/5`; v1-quality-r3 must match or improve. Any regression from the C4-follow-on sweep must be caught at gate-4 before merge.

## 5. Operational rules

- **Polling cadence:** ~10 min, `mcp__junior-brehon__list_tasks`.
- **Brief discipline:** briefs at `.claude/PRPs/briefs/v1-quality-r3-<role>-<n>.md`, committed to governance-v0 before dispatch; pre-queue `memory_search_hybrid` + `/precheck` + §2.4 mandatory file-class lesson injection. Impl-task briefs on e2e.rs MUST include `feedback_fix_impl_pre_locate_e2e_anchors.md` + `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` in §3 Required reading.
- **Shape G status:** check current DQ — Shape G was RE-ENABLED 2026-05-30 (commit `9bd933fe2`; GH Actions resets ~2026-06-01). If Actions budget is restored, validate-pending uses Shape G (ci-watcher); if still exhausted, use validate-pending-laptop (laptop bat-wrapper). Check before first impl dispatch.
- **Windows e2e bat-wrapper:** `cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true`. Never bare `cargo test` on Windows (libpq.dll missing). See `feedback_windows_e2e_requires_bat_wrapper.md`.
- **Serial cohort:** T3/T4/T5-class (all `e2e.rs`-touching) are serial by YAML overlap rule (same file). The #167 production fix is a separate task and may be `[P]` with non-e2e tasks if file overlap permits.
- **Model tiering:** Planning → Opus 4.8, Impl → Sonnet 4.6, BM/ci-watcher → Haiku 4.5.
- **Clarify gate:** run `/brehon-clarify` on each planning brief before dispatch.
- **Six user gates:** plan approval (gate 1), judgment-heavy DQ (gate 2), CR triage (gate 3), Phase-2 e2e local-vs-dispatch (gate 4), merge confirm (gate 5), retro sign-off (gate 6). Never skip.
- **DQ attribution:** advisor-side commits must match `^(chore|docs)\((advisor|decision-queue)\)`.
- **Lesson promotion:** promote the two nominated lessons (`feedback_envvarguard_fixture_lifetime_footgun.md` + `feedback_gate4_full_e2e_env_refactor_class.md`) before authoring the planning brief.

## 6. What changed from v1-quality-r2's rule set

- **Two new lessons to apply** from v1-quality-r2 retro (promote them first, then they auto-load as mandatory §3 injections for e2e.rs edits).
- **Issue #167 crosses into production code** (`crates/api/`) — first time the quality lane touches production code, not just `e2e.rs`. This means CR gets a real semantic review target; expect CodeRabbit to engage on the handler shape.
- **Shape G may be active again** (GH Actions billing reset ~2026-06-01) — check before first dispatch; adapt validate-pending kind accordingly.
- **v1-quality-r2 fixture-lifetime footgun** is now a named lesson; the planning brief must reference it in §3 Required reading for any e2e.rs bootstrap()-touching edit.

## 7. Catch-fire procedures

Universal triggers (per `.claude/rules/advisor-orchestrator.md` §5.5`):
- Junior subagent commits to `governance-v0` or `main` directly.
- `bm-task` opens PR into `main`.
- Phase branch has uncommitted state when Junior reports complete.
- ci-watcher exit code not in the empirical table.
- Cancelling a Junior task whose worker log shows uncommitted code → SSH tar before cancel.

Phase-specific additions:
- **Stop if: issue #167 fix removes `tokio_postgres::connect` from `admin_audit_stream.rs` but does NOT update the handler's e2e test to verify the new URL resolution path** — the whole point of the fix is to remove the env-var dependency; the e2e test must assert the LISTEN connection works without `LEMMY_DATABASE_URL` being set.
- **Stop if: the C4-follow-on sweep wraps `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` + `GOVERNANCE_LOG_SIGNING_KEY` sites INSIDE a `bootstrap()` helper that returns a tuple WITHOUT the guards** — that's the exact v1-quality-r2 footgun. The planner must flag any fixture-function-level wraps in §4 watchpoints.
- **Stop if: gate-4 e2e shows > 0 failed** — no merge until 0 failed confirmed.
- **Stop if: `emit_reputation_event` count is still 2 but plan includes #158 scope** — premature-DRY gate must be re-run at plan-time, not assumed from v1-quality-r2's assessment.

## 8. Archive after v1-quality-r3

Run `/brehon-phase-transition v1-quality-r3 v1-quality-r4` (or whatever the next quality slice is). This skill will: close `workflow_state_v1_quality_r3.md`, delete `workflow_state_v1_quality_r2.md` (the two-ago record at that point), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. This bootstrap file stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `207d606c8` (captured 2026-05-30) — `docs(refs): compact-prompt approach + optimisation log`
- Phase branch HEAD: not yet created (branch `phase-v1-quality-r3` cut at bm-cut)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  207d606c8 docs(refs): compact-prompt approach + optimisation log
  f98ee77a6 Merge pull request #168 from barrie-cork/phase-v1-quality-r2
  8273f1423 docs(retro): v1-quality-r2 retro — EnvVarGuard fixture-lifetime regression + gate-4 gate-value signal
  75ca61251 fix(e2e): keep LEMMY_DATABASE_URL live in admin_audit_stream tests
  8a9cb2b0f chore(advisor): correct follow-up issue ref #168→#167 in fix-impl-1 brief + RCA
  ```

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff — 0 pending entries on governance-v0 at 2026-05-30T19:51Z)
```

## Stop-and-ask tripwires

- `Stop and ask if:` the plan includes any migration under `crates/db_schema/migrations/` — this phase is e2e.rs + one API handler refactor only; a new migration is a scope violation.
- `Stop and ask if:` the e2e.rs env-var sweep wraps a site inside a `bootstrap()` helper function body and the brief does NOT specify that the guard must be returned to the caller — that's the v1-quality-r2 footgun pattern; escalate before dispatch.
- `Stop and ask if:` the issue #167 fix shape involves adding a new function parameter to `admin_audit_stream()` that changes its public signature — that could ripple into the AP federation layer; confirm scope is confined to the internal LISTEN channel only.
- `Stop and ask if:` gate-4 full-e2e shows any test count regression below 119 passed (not just > 0 failed) — even a passing-but-fewer count means a test moved to ignored unexpectedly; verify before merge.
- `Stop and ask if:` issue #158 scope is included in the plan but `emit_reputation_event` call-site count is < 3 — run `grep -rn "emit_reputation_event" crates/` from the phase-branch tip and count callers before confirming.
