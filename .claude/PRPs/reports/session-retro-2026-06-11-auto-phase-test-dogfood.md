# Session retro — 2026-06-11 — auto-phase test dogfood

**Harness:** claude-code  
**Session window:** 2026-06-11T20:27Z → 2026-06-12T00:06Z (~220 min active, with gaps)  
**Branch at start:** `2bd989568` (`governance-v0`)  
**Branch at end:** `4cfb9cb78` (`governance-v0`)  
**Files touched:** 14 (briefs, handovers, plan + DQ reconcile, auto-state JSON)  
**Commits:** 10 (advisor: 6 chore/docs, Junior merge: 2, bm-task: 1, planner: 1)

## TL;DR

This session ran the first real `/auto-phase test` dogfood cycle — bm-cut → planning → impl-cohort-1 dispatch — to exercise the complete orchestration pipeline on a trivial `sandbox_clamp` Rust helper. The orchestration mostly worked: all stage transitions fired correctly, the planning Junior reconstructed the brief correctly under brief-absent conditions, task-0 pre-flight passed all 7 probes, and task-1 impl was queued with clean state before the session ended. One notable surprise: the planning brief was committed on `governance-v0` but was NOT visible on `phase-test` at planning-task dispatch time — the planner reconstructed scope from the sibling bm-cut brief (self-healed, but fragile). The top change proposal is to make the Mode-A/Mode-B brief-visibility check a pre-dispatch probe in the auto-phase skill rather than a silent recovery.

---

## What surprised us

**Advisor:**
- The planning brief (`test-planning-1.md`) was committed on `governance-v0` but was NOT present on the `phase-test` branch when the planning Junior forked from `phase-test` HEAD. The planner found `test-bm-cut-1.md` as a sibling, reconstructed intent from it, wrote a `kind: "log"` DQ transparently, and produced a correct plan. This is correct behaviour — the planner self-healed — but the advisor-side pre-dispatch check did not catch the visibility gap before queuing. Planning subagent found the brief absent and fixed it autonomously, which is good robustness, but the gap should not have existed.

- The `plan-file-prereq-override` gate was needed at init time because `/auto-phase test` started fresh and no plan existed yet. The gate fired correctly and the user waived it, but the init stage doesn't have explicit guidance for the "plan will be authored by the planning Junior, so waive the prereq" path — the advisor author wrote the waive-justification ad-hoc.

- The auto-state JSON accumulated four `stage_digests` cleanly, including the `impl-cohort-1-task1-running` digest that wasn't a registered stage name — the advisor wrote it anyway as an informal extension. This works but reveals the digest is not validated against a stage enum.

**Planning subagent:**
- Reconstructed a 20-section plan from a sibling brief (not the named brief) and emitted a transparent `kind: "log"` DQ. Plan structure and §13 task shapes were correct and matched the Shape-G pattern from `v1-jury-mechanics-e.plan.md`. The planner correctly identified `crates/utils/src/sandbox.rs` as the simplest possible real Rust change target.

**Impl-task (task-0 pre-flight):**
- All 7 probes passed on first run, including the Shape-G workflow-present check. No blockers.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Pre-dispatch brief-visibility probe in `/auto-phase` skill** — before queuing planning Junior, run `ssh homeserver "git -C /srv/brehon-fork ls-files .claude/PRPs/briefs/<brief>"` (or equivalent `git ls-tree` on the phase branch tip on daemon). If absent, stall with a AskUserQuestion offering to sync trunk → phase first. | Catches Mode-A/Mode-B brief-sync gap before it reaches the planning subagent — converts silent self-heal into explicit pre-flight. | medium (add probe step to skill body planning-running stage) | 1× this session; 1× prior (Mode-B brief docs already address it but the auto-phase skill doesn't enforce it) |
| 2 | **Add `plan-file-prereq-waive` as a first-class gate** in the auto-phase state-machine `init` stage — with the waive justification pre-filled when `brief mentions planning Junior will author plan`. Currently the waive-justification is ad-hoc prose at the AskUserQuestion. | Reduces cognitive load at init time; makes the waive rationale durable in user_gate_history and retrospectable. | minor (expand the AskUserQuestion options at init → bm-cut-running transition) | 1× this session |
| 3 | **Validate `stage_digests` stage names against a known enum** — or document that informal extension digests are OK and note them with a `kind: "extended"` flag. Currently `impl-cohort-1-task1-running` isn't in any registry and future code reading the digest list might mishandle it. | Either prevents informal extension and forces proper stage naming, or makes the extension intent explicit. | minor (either add to stage enum in auto-phase skill body, or add `kind: "extended"` field to digest schema) | 1× this session |

## What to carry forward

- **Planning self-heal via sibling brief is robust** — the planner's transparent `kind: "log"` DQ and sibling-reconstruction behaviour worked cleanly. This is a good failure mode to preserve; it converts an advisor brief-authoring miss into a durable finding rather than a task failure.
- **task-0 pre-flight probe structure** — the seven-probe sequence (submodule, branch, absent-target, anchor-present, anchor-unique, clippy-deny, workflow, concurrent-PR) is a clean dogfood template for future Shape-G phases. §13 Task 0 can be replicated almost verbatim in any new phase using the `test.plan.md` as a MIRROR ref.
- **Serial cohort with two members works correctly under the cross-lane total cap** — task-0 completed then task-1 queued, no `.git/index.lock` contention, auto-state `current_cohort.members[1].status = "running"` accurately reflects live state.
- **Brief-on-governance-v0 + plan-on-phase-test reconcile pattern** — the `chore: merge advisor briefs into governance-v0 (planning-done reconcile)` merge commit at `a73d4d211` is the correct mechanic for reconciling plan output back to trunk. Carry forward for all future `planning-running` → `impl-cohort-1-running` transitions.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/auto-phase test` (bm-cut → planning → impl) | ~90 | ~15 | medium | Orchestration ran mostly clean; surprise = brief-visibility gap (planner self-healed). The 15 min wasted is the session pause to diagnose the DQ and verify the self-heal was complete. |
| Planning Junior #653 (7 min wall-clock) | ~25 | 0 | low | Reconstructed scope correctly from sibling. Output was a well-formed 20-section plan. |
| bm-cut Junior #652 (89s wall-clock) | ~8 | 0 | none | Fast, clean. Branch `phase-test` created and visible on remote in under 2 min. |
| task-0 pre-flight Junior #654 | ~10 | 0 | none | All 7 probes passed on first run. |
| PMD investigation (cfg test issue) | 0 | 5 | low | Issue was already resolved (ensure_default_settings already correct); PMD hit was a historical lesson note. 5 min reading to confirm. |
| Explore subagent (cfg investigation) | 5 | 0 | none | Correctly identified the fix was already in place, saved manual grep work. |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`. No impl commits landed this session (task-1 was queued but not completed). Planning Junior produced 1 commit (`docs(plan): test plan written`). No heavy file-edit tasks completed.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Planning Junior #653 | 2 (plan + DQ) | 1 | 7 | ~3 |
| bm-cut Junior #652 | 1 (runlog) | 1 | ~2 | <1 |
| task-0 pre-flight Junior #654 | 0 (no commit) | 0 | ~5 | ~3 |

## Decisions to revisit

- **Phase-test pipeline completion** — task-1 (#655) was queued at session end, still `status: "running"` in auto-state. The next session picks up at `impl-cohort-1-task1-running`, waits for task-1 to push `sandbox.rs` + write `validate-pending` DQ, dispatches ci-watcher. Pipeline is mid-flight; no restart needed.
- **`last_session_ended_at: null`** in auto-state — the field was not populated at session end. Minor schema discipline gap; consider adding a session-end hook that writes `last_session_ended_at` to the JSON before exit.

---

## Auto-phase reliability

### 1. Stage-transition correctness

All transitions fired on correct triggers:
- `init → bm-cut-running`: user waived plan-file prereq (gate logged in `user_gate_history[0]`).
- `bm-cut-running → planning-running`: Junior #652 done in 89s; phase-test branch verified; planning brief committed.
- `planning-running → impl-cohort-1-running`: planning #653 done in ~7 min; plan approved (gate `user_gate_history[1]`); governance-v0 synced to phase-test; impl briefs committed.
- `impl-cohort-1-running → impl-cohort-1-task1-running`: task-0 #654 PROBES:ALL PASS; task-1 #655 queued.

No premature fires, no missed fires, cohort barrier is maintained (task-1 queued only after task-0 `status: "complete"`).

`user_gate_history[1].notes`: *"user chose 'Approve — Rust + Shape G (Recommended)'"*; notes also record "planner diverged from brief: sandbox_clamp in crates/utils/ instead of non-cargo meta tasks." This divergence was correctly approved at plan-approval time.

### 2. Cadence calibration

Session had significant gaps (session ran in non-continuous intervals with human-driven polling). Auto-state shows `last_poll_at: 2026-06-12T00:05:00Z` which is the final poll for this session — the planning-running wait and the task-0 pre-flight wait were covered by previous-session handovers and manual poll calls. No automated ScheduleWakeup was running in this session (auto-phase was being driven interactively, not in loop mode). Cadence N/A for this session.

### 3. Auto-state integrity

`resume_count: 1` — one resume at the impl-cohort-1-running stage start.  
`last_known_phase_tip: "0a745c56d"` — matches `git log origin/phase-test -1 --format='%h'` (verified during task dispatch).  
No hand-edits to recover; no JSON corruption observed.

### 4. User-touchpoint count vs target

Gates fired this session: 2 (`plan-file-prereq-override` + `plan-approval`). Target for a full phase is 6-8; this session covered the first two gates. Remaining gates (CR triage, Phase-2 e2e local-vs-dispatch, merge confirm, retro sign-off) will fire in subsequent sessions. No false-positive AskUserQuestions observed.

### 5. Catch-fire FP/FN rate

No catch-fire dumps. No catch-fire conditions triggered. No silent-advance past a should-have-been-catch-fire observed. ✓

### 6. §G4 classifier accuracy

No `validate-pending` DQ entries mutated this session (task-1 not yet complete). N/A.

### 7. L14 / L15 / L16 fixes still holding

L14 (BM runlog-before-merge): bm-cut ran; no merge yet — will check at bm-merge.  
L15 (gate-side checks inline): the plan-approval gate ran inline in advisor session, not as a Junior dispatch. ✓  
L16 (branch-delete post-merge): no merge yet — will check at bm-merge.

### 8. Subagent offload effectiveness

One `Explore` subagent dispatched for the PMD cfg investigation. Synthesis was correct and usable on first read (identified fix already in place). Net-positive context conservation: the grep work stayed out of main context. ✓

### 9. Plan §13 fidelity vs cohort dispatch

Plan §13 has two tasks (task-0 preflight [non-P], task-1 impl [non-P]). Both are serial (no `[P]` markers). Cohort dispatched as `serial-impl` matching the plan intent. No degrades. ✓

### 10. Resume-cycle pain points

`resume_count: 1`. The one resume was clean — Phase 0.5 found no discrepancies at the `impl-cohort-1-running` entry. The 'continue' prompt at resume provided useful state confirmation (task-0 had passed, task-1 was ready to queue). No `--start-from` overrides used. ✓ (first c-2 phase — keep the friction).

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ✓ | 1× this phase, 0× prior |
| 2. Cadence calibration | N/A (interactive session) | — |
| 3. Auto-state integrity | ✓ | 1× clean |
| 4. Touchpoint count | ✓ (2/6 gates so far, on track) | — |
| 5. Catch-fire FP/FN | ✓ (0 fires) | — |
| 6. §G4 classifier | N/A (no validate-fail yet) | — |
| 7. L14/L15/L16 | ✓ (L15 holding; L14/L16 deferred to bm-merge) | — |
| 8. Subagent offload | ✓ (Explore subagent usable on first read) | — |
| 9. Plan §13 fidelity | ✓ (serial-impl, no degrades) | — |
| 10. Resume cycles | ✓ (1 resume, no friction) | — |

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Brief-visibility pre-dispatch probe**: promote to auto-phase skill body as a Stage `planning-running` pre-queue check. Command: add `ssh homeserver "git -C /srv/brehon-fork ls-tree HEAD:.claude/PRPs/briefs"` check before `create_task` for planning. Recurrence: 1× this session + 1× in Mode-B brief docs (already documented as a gap, now needs enforcement). Cost: medium. Update `.claude/commands/auto-phase.md` planning-running stage.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`._
