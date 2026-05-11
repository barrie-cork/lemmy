# v1-RT-r1 halt retrospective — Cohort B-final paused

> **Halt retro, not phase-close retro.** Written 2026-05-11 mid-phase to capture
> the recurring divergence + cohort-isolation patterns BEFORE more dispatch
> cycles compound the cost. RT-r1 is paused at phase tip `37a62f9b4` until
> the plan and rules are revised. Phase-close retro (plan §13 Task 11)
> happens later, when phase ships.

## Phase state at halt

**Branch:** `phase-v1-RT-r1` @ `37a62f9b4`
**Plan:** `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` (11 tasks + retro)
**Cohort A (Tasks 1-4):** ✅ SQL migrations shipped + bundled re-validate passed (DQ #200+#201)
**Cohort B-serial (Task 5):** ✅ enum committed; isolation workflow FAIL (DQ #203) historical
**Cohort B-bundled (Tasks 6+7):** ✅ schema.rs + Diesel structs committed; workspace FAIL E0063 (DQ #204) → fix-impl-1 dispatched
**fix-impl-1:** ✅ 2 callsite literals padded; workspace FAIL persists (DQ #206); Junior filed DQ #205 surfacing 8 more callsites out-of-brief-scope
**Cohort B-final (Tasks 8+9+10):** NOT STARTED — halted before dispatch
**Task 11 (canonical retro):** NOT STARTED

## What's pending at halt

| DQ | Kind | State | Notes |
|---|---|---|---|
| #194 | validate-pending | `result: fail` historical | Cohort A Task 3 per-task isolation FAIL (workflow 25636774556) |
| #203 | validate-pending | `result: fail` historical | Cohort B Task 5 per-task isolation FAIL (workflow 25668340599) |
| #204 | validate-pending | `result: fail` historical | Cohort B-bundled 6+7 workspace E0063 FAIL (workflow 25669550114) |
| #205 | blocker | unanswered | Junior asks: queue fix-impl-2 covering 8 more callsites? |
| #206 | validate-pending | `result: null` | Workflow `25698028473` returned `failure` per `gh run view`; not yet mutated to DQ |

**Total pending: 5.** None block phase progression once #205 is answered + fix-impl-2 dispatched + Cohort B-final mega-bundle authored.

## Three signals (per `feedback_four_role_retro_signals.md`)

### Advisor signals

**What worked:**
- §G4 classifier correctly identified DQ #194 as non-allowlist → catch-fire → user-relay path produced the right fix (bundled re-validate).
- DQ #200+#201 re-validate via `junior/revalidate-rt-r1-bundle` worked exactly as designed for the bundled-validation pattern.
- Inline-mutation recovery from ci-watcher resurrection bug (DQ #189-#192 + #195-#196) was clean — Python-script + commit-push pattern was idempotent and auditable.
- Halt-decision: choosing to retro NOW rather than push through Cohort B-final with the same bug pattern about to recur was correct judgment.

**What didn't:**
- **Cohort A `[P]` markers conflated worktree-write disjointness with validation-time independence.** Each Task 1-4 worker branch passed its file-disjointness check but Task 3's migration UPDATE references Task 1's column — workflow failed on a per-task worker branch.
- **Cohort B-serial markers ("Sequential ordering required") didn't fix the underlying gap.** Task 5's enum references the sql_types path Task 6 creates — same isolation-validation bug class.
- **Parallel ci-watcher dispatch off same phase tip caused merge-time resurrection** of already-resolved DQ entries (#189-#192 + #195-#196). 4 ci-watchers each forked off the same tip; sequential finalize-merges reverted earlier mutations.
- **DQ raise + ci-watcher dispatch isn't atomic.** ci-watcher #219 forked before my advisor DQ #200+#201 raise was pushed; correctly filed contract-violation (DQ #197). Lost wallclock + bookkeeping.
- **Shared-checkout race:** shutter session and this RT-r1 advisor session both operate on `C:\Users\barri\Developer\brehon-fork`. Both write `.claude/decision-queue.json`. Three daemon/origin divergence cycles in one session (~3-4 hour bookkeeping overhead).

### Planner signals (from plan §13 task author)

**What worked:**
- Plan §10 mirror skeletons (verbatim SQL + Rust) made Junior tasks mechanical.
- Plan §13 FILES YAML with creates: + modifies: arrays gave clean cohort-overlap checks.
- Plan §5.2 complexity score caught the right magnitude (19/10) for proceed-as-one decision.
- Plan §10.4 + §10.8 cumulative invariant accounting (V1_RT=26 net-new, cumulative=127) correctly anticipated DQ #185/#187 dispute and resolved with audit-trail evidence.

**What didn't:**
- Plan §13 `[P]` semantics conflate two distinct properties (worktree-write disjointness vs validation-time independence).
- Plan §13 "Cohort B barrier. Sequential ordering required." markers don't address the cross-task validation dependency — they only address commit-order.
- No `requires:` field in FILES YAML to declare cross-task data dependencies (Task 3 needs Task 1's column; Task 5 needs Task 6's sql_types).
- fix-impl-1 brief's scope-bound (`crates/api/api/src/governance/` only) was too narrow — Junior couldn't fix all 10 callsites in one task, surfaced as DQ #205 blocker.

### Impl signals (from Junior tasks #208-#226)

**What worked:**
- Junior task #195 (Task 0 pre-flight): clean 11-probe run + WARN handling for missing yamllint.
- Junior tasks #208-#211 (Cohort A migrations): mechanical SQL writing from §10 skeletons; commits + Shape G pushes correct.
- Junior task #223 (Task 5 enum): mechanical, single-file Rust addition; correct shape.
- Junior task #225 (bundled 6+7): correctly executed 3-file bundled commit; bundled validation was the right strategy.
- Junior task #226 (fix-impl-1): correctly handled scope-bound + surfaced blocker DQ #205 with concrete option-a/option-b proposals.

**What didn't:**
- Junior tasks #208-#211 raced on DQ id #189/#190 (4-way collision) — each Junior computed `next_id` against its worker branch's view, not a shared atomic counter. Required advisor renumber (DQ #189-#196) + cohort merge auto-resolution caused resurrection bug.
- Junior tasks #215-#218 (4 parallel ci-watchers): same race class — forked off same phase tip; sequential finalize-merge resurrection.

### BM signals

**What worked:**
- Daemon's bm-cut Junior task #186 cleanly cut `phase-v1-RT-r1` off governance-v0 with runlog seed.

**What didn't:**
- Daemon finalize-merge regularly skips origin-push (3 occurrences this session: planning #182, bm-cut #186, Task 5 push). Required manual `ssh homeserver "git push origin phase-v1-RT-r1"` each time.

## Per-task complexity score (per `feedback_retro_task_complexity_score.md`)

| Task | Files | Commits | Wall-clock | Max log-silence |
|---|---|---|---|---|
| 0 (pre-flight) | 0 | 0 (verify-only) | 5m17s | <1 min |
| 1-4 (Cohort A migrations) | 2 each | 1 feat + 1 DQ each | ~4 min each | <1 min |
| 5 (Rust enum) | 1 | 1 feat + 1 DQ | 5m33s | <1 min |
| 6+7 (bundled) | 3 | 1 feat + 1 DQ | 5m47s | <1 min |
| fix-impl-1 | 2 | 1 fix + 1 DQ | 8m51s | <1 min |
| **Recovery work (advisor wallclock)** | 1 (decision-queue.json) | 7+ commits across 3 reconciles | **~4 hours** | n/a |

Recovery overhead dominated the schedule: 7+ commits to `.claude/decision-queue.json` across 3 reconcile cycles, plus 4 hours of advisor-driven merge resolution and Python scripting.

## Lessons to promote to rules + lessons

### High-priority (lock in BEFORE next phase)

#### L1. `[P]` semantics need explicit validation-dependency layer

Plan template `.claude/PRPs/templates/plan.template.md` §13 FILES YAML should add:

```yaml
creates: [...]
modifies: [...]
requires:           # <-- new field
  - task: <N>       # this task's validation depends on task <N>'s changes
    reason: <one-line> # what specifically is needed
```

Advisor cohort-dispatch check (`.claude/rules/advisor-orchestrator.md` §4.1) adds step 5a:
> For each `[P]`-marked task in the cohort, verify all `requires:` entries are
> already merged on the phase branch — else degrade to serial OR reorder cohort.

Affected ADRs: none. Affected rules: `advisor-orchestrator.md` §4.1, `.claude/PRPs/templates/plan.template.md` §13. Lesson file: `.claude/lessons/feedback_cohort_validation_dependency_check.md` (new).

#### L2. ci-watcher dispatch must be SERIAL (not parallel) per task pair

`.claude/agents/ci-watcher.md` + `.claude/rules/advisor-orchestrator.md` §3.1 stage-shape: when N validate-pending DQ entries exist, dispatch **ONE ci-watcher per logical task** (mutating multiple workflows in sequence), NOT N parallel ci-watchers. Parallel ci-watchers off same phase tip cause merge-time resurrection.

Affected files: `advisor-orchestrator.md` §3.1 "ci-watcher complete" branch + `ci-watcher.md` "Concurrency" section. Lesson file: `.claude/lessons/feedback_ci_watcher_serial_per_task_pair.md` (new).

#### L3. Atomic DQ raise + ci-watcher dispatch

When raising a `validate-pending` DQ entry and queueing a ci-watcher to mutate it: **the DQ raise must be COMMITTED + PUSHED before the ci-watcher Junior task is created**. Otherwise the ci-watcher worker branch may fork before the DQ raise lands → contract violation.

Affected files: `advisor-orchestrator.md` §5.2 validate-pending-laptop handler (mirror pattern to validate-pending dispatch). Lesson file: `.claude/lessons/feedback_dq_raise_before_ci_watcher_queue.md` (new).

#### L4. Single-checkout cross-session work is structurally unsound for active phases

Two advisor sessions writing `.claude/decision-queue.json` on the same `C:\Users\barri\Developer\brehon-fork` checkout produces unavoidable divergence + reconcile cycles. Either:

1. **Lane-strict isolation**: one session locked to RT-r1, another locked to SL-d, with a checkout-level mutex (e.g. `git worktree` per lane), OR
2. **Mid-task push discipline** + cross-session DQ id reservation (advisor sessions reserve id ranges to avoid collision), OR
3. **No cross-session work on active phases** (one phase = one advisor session; serialise phases instead of running in parallel).

Affected: project topology decision. Lesson file: `.claude/lessons/feedback_single_checkout_cross_session_unsound.md` (new). User decision required.

### Medium-priority (capture but don't block)

#### L5. fix-impl brief scope-bound should match full callsite enumeration

§G4 classifier dispatches narrow fix-impl tasks (≤3 file edits per allowlist rule). But when the compile error reveals only 2 of N callsites need padding, the fix-impl-1 brief must enumerate the FULL `rg`-discoverable callsite set (or explicitly limit to N=1 with retry expected). The fix-impl-1 brief here properly listed expected sites but bounded to 2 — forcing a follow-up fix-impl-2.

Affected: §G4 classifier auto-fix recipes. Lesson file: `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` (new).

#### L6. Daemon finalize-merge push regression

Daemon's `claude -p` finalize stage frequently skips the origin push. Symptom: daemon-local tip ahead of origin; advisor must manually `ssh homeserver "git push origin phase-..."`. Three occurrences this session.

Root cause: unclear (claude -p prompt? settings.json finalize hook?). Affected: Junior daemon configuration. Lesson file: `.claude/lessons/feedback_junior_finalize_push_regression.md` (extend or new — `feedback_junior_finalize_skips_when_worker_pre_pushes.md` may already cover; check before authoring).

## Forward plan for RT-r1 (post-halt)

Once L1+L2+L3 are locked in:

1. Answer DQ #205 → option-a (queue fix-impl-2 covering all 8 sites).
2. Mutate DQ #206 → `result: fail` with log_slice (workflow `25698028473` already failed; just bookkeeping).
3. Author **bundled brief: Cohort B-final + fix-impl-2** combined — Tasks 8, 9, 10, plus 8-callsite fix in 3 additional files (`create_endorsement.rs:278+291`, `seed_founders/main.rs:173`, `e2e.rs:2902+2925+3935+5345+7986`). Single Junior task, single commit, single workflow.
4. ci-watcher (serial, single) mutates the resulting validate-pending DQ.
5. On pass → cohort barrier lifts → Task 11 (canonical retro) → bm-pr → CR triage → merge.

## Forward plan for the broader pattern

User decision required on L4 (single-checkout cross-session). Options:

- **(a) Lane-strict via git worktrees:** RT-r1 advisor session works in `C:/Users/barri/Developer/brehon-rt-r1` worktree; SL-d advisor session in `C:/Users/barri/Developer/brehon-sl-d`; cross-session writes to `governance-v0` (briefs, plans) require explicit hand-off via user-relay (mid-task push from the lane-active session).
- **(b) Serialize phases:** finish RT-r1 entirely before opening another phase. SL-d shipped 2026-05-11; future phases run one-at-a-time.
- **(c) DQ id reservation:** lane-strict DQ id ranges (RT-r1 uses #190-#250; SL-d uses #150-#190). Lower bookkeeping overhead but doesn't fix file-level conflict.

## Sign-off

Author: advisor session (this RT-r1 advisor on `C:/Users/barri/Developer/brehon-fork`).
Date: 2026-05-11.
Halt status: RT-r1 paused at `37a62f9b4`. No further Junior dispatches until L1-L4 are locked in (or user-deferred).
