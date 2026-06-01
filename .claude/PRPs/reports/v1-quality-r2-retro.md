# Retro — v1-quality-r2

**Phase:** v1-quality-r2 (PR #155 carry-forward bundle, r2-half)
**Date:** 2026-05-30
**PR target:** `governance-v0` (not yet opened — retro precedes bm-pr)
**Tasks shipped:** T3 (#156 doc-comment audit), T4 (#159 boot_context EnvVarGuard hoist), T5 (#160 LEMMY_DATABASE_URL 13-site wrap) + fix-impl-1 (T5 regression fix)
**Commit range:** `phase-v1-quality-r2` tip `75ca61251`

---

## What surprised us

**T5 self-regressed in the same session it shipped.** The EnvVarGuard refactor (the entire point of the phase) introduced a 2-test deterministic e2e regression that only the full gate-4 e2e RUN caught — not cargo check, not clippy, not `e2e --no-run`. Root cause: 2 fixture `bootstrap()` helpers acquired an `EnvVarGuard` locally but returned only a 3-tuple (without the guard), so the guard dropped on return and unset `LEMMY_DATABASE_URL` before the `admin_audit_stream` handler's lazy `get_database_url()` re-read. The pre-T5 raw `unsafe { set_var(...) }` was a load-bearing process-lifetime "leak" that the SSE handler depended on. This is counterintuitive: the "correct" RAII pattern broke 2 tests; the "unsafe leak" was the correct behaviour.

**Worker independently derived a cleaner fix shape.** The fix-impl brief prescribed restoring raw `unsafe { set_var(...) }` in the 2 defective fixtures (minimal restore, zero caller churn). The worker couldn't find the brief on its branch and reasoned from the RCA directly — it chose instead to hold an `EnvVarGuard` **in each of the 3 `admin_audit_stream` test bodies** for the duration of each test. This is actually cleaner: no process-lifetime leak, properly scoped, and architecturally closer to the eventual fix (issue #167). The brief/RCA/resume-handover now describe a superseded approach; the retro is the correction-of-record.

**Origin/daemon branch diverged at T5.** The T5 commit landed on daemon-local `phase-v1-quality-r2` but was a different SHA from origin (two independent cherry-picks of the same logical content). Reconciling without force-push required cherry-picking only the fix commit onto origin tip via a temp branch — an extra git surgery step not in the nominal close sequence.

---

## What to change

**Fixture `bootstrap()` helpers that set env vars need a return-guard discipline.** The RAII guard in a fixture that doesn't return the guard is a lifetime footgun. Forward rule: any `bootstrap()` helper that sets an env var via `EnvVarGuard` MUST either (a) return the guard to the caller (like `boot_context()` after T4), or (b) document explicitly that the var is intentionally process-scoped and switch to raw `set_var` with a SAFETY comment. The RCA file (`qr2-envguard-rca.md`) is the canonical reference; promote it as a lesson for e2e-fixture authors.

**Implement issue #167 (architectural fix).** The `admin_audit_stream` handler reads `Settings::get_database_url()` — a process-global env read — to open its raw `tokio_postgres` LISTEN connection instead of deriving the URL from the injected `context`. This is an architectural smell that makes any e2e fixture env-var management more fragile. Issue #167 is filed; track it.

**One-action-at-a-time on the git critical path (advisor discipline).** Two SHA hallucinations occurred in this session (guessing `57bf80c5e`, then `9fd24f5f3`) when resolving the origin/daemon divergence. Both caused batch failures requiring restart. The rule is already captured at `feedback_cross_session_commit_attribution_collision.md` and in the rt-r4 advisor defect record — this is a second confirmed recurrence. Discipline: never guess a SHA; always read from git log before using it in a command.

**Batch over-firing on the git critical path.** Parallel tool batches on git operations (where each call's output gates the next) caused repeated cancellation cascades. Independent reads can still be batched; git-state-mutating or SHA-dependent calls must be sequential.

---

## What to carry forward

**Gate-4 full-e2e is load-bearing; do not skip it on refactors touching env-var management.** This phase's whole-point refactor (EnvVarGuard) self-regressed in a runtime-only way that compile-only gates are structurally unable to catch. The env-var-hygiene refactor class is particularly prone to this: compile-only checks only verify the guard type is valid, not that the guard's lifetime matches the handler's read timing. Any future refactor in this class requires gate-4.

**fix-impl-1 RCA file (`qr2-envguard-rca.md`) describes a SUPERSEDED fix shape.** The "chosen fix" section (§"Fix (chosen: minimal restore)") reflects the brief-authored approach, not what shipped. The actual fix is recorded in commit `75ca61251` — guard held in the 3 `admin_audit_stream` test bodies. Future readers should trust the commit, not the RCA doc.

**`admin_audit_stream` architectural debt (issue #167) carries forward.** The lazy `get_database_url()` re-read in `admin_audit_stream.rs:125-126` is an isolation footgun for e2e. Filed as `#167`; follow-on phase should resolve the LISTEN URL from `context` rather than the process-global env.

---

## Four-role retro signals

### Advisor
- Defect: SHA-hallucination ×2 in this session (guessing SHAs before reading git log); batch over-firing on git critical path. Both are recurrences of the one-action-at-a-time-on-git discipline captured in rt-r4 retro. Severity: caused ~30 min of recovery per incident.
- Positive: gate-4 full-e2e caught the regression that gate-3 (validate-pending-laptop-e2e-no-run) could not. The gate-4 call was advisor-driven (local dispatch per feedback_windows_e2e_requires_bat_wrapper.md). The regression would have shipped to PR without it.
- Positive: origin/daemon divergence diagnosed and reconciled without force-push; linear history preserved.

### Impl (Junior workers)
- T3 (#518): mechanical doc-comment sweep — clean, no DQ. Exit without issues.
- T4 (#519 area): boot_context refactor — clean. DQ raised + resolved (validate-pending-laptop). No surprises.
- T5 (#520 area / initial run): LEMMY_DATABASE_URL 13-site wrap — committed correctly, but the fix introduced the runtime regression caught at gate-4. No blame signal — the regression required a full e2e run to surface; the worker ran only cargo check per the DoD.
- fix-impl-1 (#527): worker couldn't find the brief (brief not visible on its fork point) but reasoned from the RCA and produced a **cleaner** fix shape than briefed. Brief-not-visible is a recurring pattern when the brief is committed after bm-cut; T5's Mode B sync-to-phase procedure was followed but the fix-impl-1 brief was authored after the branch already existed on origin — the worker forked a tip that didn't carry the brief.

### Planning (planner)
- No significant signals. Plan §4 task shape map was accurate; T3/T4/T5 complexity matched prediction. The complexity score (10, re-scoped r2-half) was correctly flagged but not acted on further — the split was already executed at r2a.

### BM
- bm-cut non-idempotent (#508): the `git checkout -b` step failed when the branch already existed locally from a prior run. Recovery was straightforward but required a manual workaround step. Carry forward: bm-cut script should handle pre-existing local branch gracefully (use `git checkout <existing>` when branch exists, not fail).

---

## Lessons promoted this phase

1. **`feedback_envvarguard_fixture_lifetime_footgun.md`** (new) — RAII guard in a fixture `bootstrap()` that doesn't return the guard drops on return; env var unset before test body runs. Return the guard, or use process-scoped raw `set_var` with SAFETY comment.
2. **`feedback_gate4_full_e2e_env_refactor_class.md`** (new) — compile-only gates cannot catch runtime env-var-lifetime regressions; gate-4 is mandatory for any env-var-management refactor class.

(Promotion: lesson files to author in governance-v0 session before phase transition.)

---

## Complexity score (per-task actuals)

| Task | Files | Commits | Runtime (min est.) | Max log silence (min) |
|---|---|---|---|---|
| T3 (doc audit) | 1 | 1 | ~15 | — |
| T4 (boot_context hoist) | 1 | 1 | ~20 | — |
| T5 (13-site wrap) | 1 | 1 | ~25 | — |
| fix-impl-1 (regression fix) | 1 | 1 | ~20 | — |
| Gate-4 full-e2e (advisor) | — | — | ~40 | — |
| **Total** | **1 crate** | **4** | **~120** | — |

Aggregate phase complexity: 4 impl commits, 1 crate, gate-4 validation, 1 unplanned fix-impl cycle. Within expectations for a complexity-10 phase post-split.
