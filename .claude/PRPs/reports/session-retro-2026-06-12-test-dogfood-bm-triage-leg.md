# Session retro — 2026-06-12 — test dogfood bm-triage leg

**Harness:** claude-code  
**Session window:** 2026-06-12T08:30Z → 2026-06-12T11:00Z (~150 min active, with gaps and context-compaction)  
**Branch at start:** `a73d4d211` (`governance-v0`)  
**Branch at end:** `6f1275d63` (`governance-v0`)  
**Files touched:** 16 (briefs, runlogs, DQ, auto-state, crates/, lesson)  
**Commits:** 20 (advisor: 8, BM/impl Junior merges: 5, Junior tasks: 3, Race-A reconcile: 1, misc: 3)

## TL;DR

This session ran the back half of the `test` dogfood phase: impl-task-1 validate-pending resolution, bm-pr, bm-poll-cr, and bm-triage dispatch. The two most notable findings: (1) Shape G's `cargo-validate-workspace.yml` workflow is disabled at the API level — workers write `workflow_run_id: 0` and the advisor must fall back to a local throwaway-worktree cargo check, a path that was documented but not exercised before; (2) a Race-A `git push` non-fast-forward on `governance-v0` occurred when BM-pr Junior committed and pushed concurrently with the advisor laptop session — resolved via daemon-side `git merge origin/governance-v0`, but the recovery required cross-machine SSH coordination. Both were handled without catch-fire but added ~25 min of friction. bm-triage #658 is now running; gate 3 (CR triage approval) was surfaced.

---

## What surprised us

**Advisor:**
- `cargo-validate-workspace.yml` is `disabled_manually` at the GitHub API level — not just `workflow_dispatch`-only. `gh workflow run` returns HTTP 422 ("Workflow is disabled"), and `gh workflow list` shows `state: disabled_manually`. The impl worker correctly wrote `workflow_run_id: 0` as the sentinel, but the advisor initially attempted `gh workflow run` expecting it to succeed (the workflow file exists, triggering a false-confidence). This path was documented in the Shape G / RESIDUAL-ONLY memory but the advisor had not previously hit the 422 branch in practice.

- Race-A on `governance-v0`: BM-pr Junior task (#656) committed `chore(bm): test PR opened #195` to `governance-v0` (via daemon finalize-merge) while the advisor laptop was mid-write for DQ + auto-state. The daemon push succeeded first; the laptop's subsequent `git push origin governance-v0` was rejected (non-fast-forward). Resolution: `git merge origin/governance-v0` on the daemon (ort strategy, clean), then push from daemon, then laptop `git pull`. Two sessions, one `governance-v0` branch — this exact race was anticipated in `feedback_cross_session_commit_attribution_collision.md` but the recovery path required SSH to the daemon rather than local-only resolution.

- The bm-poll-cr findings YAML (`pr-195-findings.yaml`) was correctly gitignored and not committed — meaning when bm-triage ran later, the YAML was absent from the worktree. The triage brief needs to include explicit "reconstruct YAML from `gh api` if file absent" instructions. The prior retro's YAML was also ephemeral; this is the expected design, but triage briefs need to carry reconstruction instructions every time.

**BM-task (bm-pr #656):**
- L14 git-sequence held cleanly: runlog commit (`chore(bm): test PR opened #195`) preceded the finalize-merge. This was the first L14 regression-test in the test phase and it passed.

**BM-task (bm-poll-cr #657):**
- Two CR findings ingested correctly. CR correctly identified the `validate-pending` with `workflow_run_id: 0` as a schema concern (cr-1 major). The advisor classified this as `rebut` — the DQ entry's shape IS correct for Shape-G-disabled flows, and it's already in `resolved[]`. This is a known pattern: CR doesn't know about the Shape G RESIDUAL-ONLY convention.

**Impl-task (task-1 #655):**
- `cargo check --workspace` passed on `phase-test` in 7m12s on the laptop. The throwaway worktree approach (separate `brehon-fork-validate-phase-test` worktree) worked cleanly — no contamination of the canonical checkout.

---

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Add `workflow_run_id: 0` advisor-laptop fast-path to the auto-phase skill's validate-pending handler** — when `workflow_run_id == 0` is detected on a new `validate-pending` entry, skip `gh workflow run` entirely and go directly to the throwaway-worktree local cargo check path. Currently the skill may attempt `gh workflow run` before falling back. Document the 422-branch as the expected case under Shape G RESIDUAL-ONLY. | Saves ~5 min per validate-pending occurrence; eliminates false-confidence from the `gh workflow run` attempt. | minor (add an `if workflow_run_id == 0` branch before the GH API call in the validate-pending handler description in `advisor-orchestrator.md` §5.2) | 1× this session (new scenario); prior memory: Shape G RESIDUAL-ONLY established but handler path not codified |
| 2 | **bm-poll-cr brief template: always include "if YAML absent, reconstruct from `gh api` before triage"** — the findings YAML is gitignored and lost after the worker finalizes. The current brief template doesn't mandate reconstruction instructions. bm-triage already has them (added to test-bm-triage-1.md ad-hoc), but the gap means a future bm-poll-cr brief author could omit them. Add a required §5 "Findings YAML recovery" to the `bm-task-brief.template.md` for both poll-cr and triage verbs. | Eliminates the ad-hoc reconstruction step; triage briefs are self-sufficient regardless of YAML state at dispatch time. | minor (update `.claude/PRPs/templates/bm-task-brief.template.md` triage/poll-cr rows; ~10 lines) | 1× this session; structurally recurs every phase (YAML is always gitignored) |
| 3 | **Race-A mitigation: BM Junior task completion → advisor fetches before writing** — when the polling loop detects a BM task transitioning to `done`, the advisor should `git fetch origin governance-v0` BEFORE authoring the next brief or DQ entry. Currently the poll-loop description says "git fetch on every poll" but it was missed before the DQ write that caused the non-fast-forward. Codify explicitly in the `bm-pr-running` → `bm-poll-cr-running` transition in the state-machine. | Prevents Race-A non-fast-forward on `governance-v0` when BM Junior finalize-merges concurrently with advisor writes. | minor (add explicit "fetch before write" step to the bm-pr completion handler in `auto-phase.md` §3.1 transition table) | 1× this session; prior: `feedback_cross_session_commit_attribution_collision.md` documents the class but not the BM-finalize-specific variant |

## What to carry forward

- **Throwaway-worktree validate-pending handler works cleanly.** `git worktree add ../brehon-fork-validate-<id> origin/<branch>` + `cargo check --workspace` + DQ mutation with `answered_by: "advisor-laptop"` + worktree remove is the correct Shape-G-disabled validation flow. No cargo contamination of the canonical checkout. Used once; ready to be the standard handler.
- **bm-triage brief should pre-determine triage decisions** (advisor-approved bucket + rationale in §4) rather than leaving the bm-task to classify independently. This shifts the judgment-heavy classification to the advisor (where it belongs) and makes the bm-task mechanical. The test-bm-triage-1.md brief follows this pattern and is the canonical shape going forward.
- **`git fetch origin governance-v0` before every advisor write** on that branch — especially after any BM or impl Junior task reports done. The finalize-merge can land at any time after the task's `status: complete` transition; a stale local ref causes non-fast-forward on the next push.
- **L14 git-sequence discipline is holding** across bm-pr, bm-poll-cr. The runlog commit-before-finalize pattern fired correctly in both cases.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| bm-pr Junior #656 (bm-task) | 25 | 5 | low | L14 held; Race-A non-fast-forward added ~5 min for SSH recovery |
| bm-poll-cr Junior #657 (bm-task) | 20 | 0 | low | Two findings correctly ingested; YAML reconstruction pattern confirmed |
| bm-triage Junior #658 (bm-task) | 20 | 0 | none | Not yet complete; brief dispatch clean |
| validate-pending advisor-laptop (throwaway worktree) | 10 | 12 | medium | Initial `gh workflow run` attempt hit 422 (Shape G disabled); worktree path worked |
| Race-A resolution (SSH daemon merge) | — | 25 | high | Unplanned; required SSH cross-machine coordination; mitigation identified |
| bm-triage brief authoring (advisor inline) | 15 | 0 | none | Pre-classified buckets in brief §4; clean pattern |

## Complexity scores (heavy tasks only)

Per `.claude/lessons/feedback_retro_task_complexity_score.md`.

| Task | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|---|---:|---:|---:|---:|---|
| impl-task-1 sandbox_clamp (#655) | 2 | 1 | ~8 | ~4 | trivial — complexity: 2/1/8/4 |
| bm-pr #656 | 1 (runlog) | 1 | ~10 | ~5 | complexity: 1/1/10/5 |
| bm-poll-cr #657 | 1 (runlog) | 1 | ~8 | ~4 | complexity: 1/1/8/4 |

All tasks well within the watchdog envelope. No outliers.

## Decisions to revisit

- Whether `validate-pending-laptop` vs `validate-pending` with `workflow_run_id: 0` is the right shape when Shape G is disabled. CR's cr-1 finding has a point — the `kind` field suggests GH-Actions-backed validation, but `answered_by: "advisor-laptop"` says it was local. The RESIDUAL-ONLY convention uses `workflow_run_id: 0` as the sentinel; encoding this in a sub-kind (e.g. `validate-pending-local`) would be cleaner but requires schema work. Filed for next phase retro consideration.

---

## Auto-phase reliability

### 1. Stage-transition correctness

All transitions fired on the right triggers:
- `impl-cohort-1-task1-running` → `bm-pr-running` fired after validate-pending DQ `6b7c4b329002-001` resolved with `result: pass` and barrier_reached confirmed. ✓
- `bm-pr-running` → `bm-poll-cr-running` fired after bm-pr #656 completed + runlog confirmed PR #195 open. ✓
- `bm-poll-cr-running` → `bm-triage-running` fired after bm-poll-cr #657 completed + 2 findings confirmed. ✓

No premature transitions; no missed transitions; cohort barrier held (task-1 member at `status: complete` before advance). `user_gate_history[1].notes` ("approved — Rust + Shape G path") — non-empty and meaningful; captured in prior session retro §1.

### 2. Cadence calibration

This session ran interactively (manual "poll" prompts) rather than via ScheduleWakeup. No wasted polls from over-scheduling. No detection lag beyond human response time. Not applicable for ScheduleWakeup calibration since the session was driven manually throughout.

### 3. Auto-state integrity

`resume_count: 1` (one context-compaction resume). `last_known_phase_tip: "50195e511"` — matches `phase-test` HEAD (`git log -1 --format='%H' origin/phase-test` = `50195e511`, confirmed). No hand-edits required other than normal stage/task-id updates. Auto-state JSON was gitignored (required `git add -f`), which is expected — no integrity issue. ✓

### 4. User-touchpoint count vs target

Two `user_gate_history` entries recorded:
- `plan-file-prereq-override` (gate 0.5, prior session)
- `plan-approval` (gate 1, prior session)

Missing from this session's leg (still in flight):
- Gate 3: CR triage approval — surfaced this session ✓
- Gate 5: merge confirm — pending
- Gate 6: retro sign-off — pending

Two of six mandatory gates remain; both are pending Junior task completion (bm-triage #658) and advisor decision. Touchpoint count is on track. No false-positive AskUserQuestion calls in this leg. ✓

### 5. Catch-fire FP / FN rate

No catch-fires fired in this session leg. No `*-catchfire-*.md` dumps exist under `.claude/auto-state/`. 

Potential FN check: the Race-A non-fast-forward should NOT have been a catch-fire (it's a recoverable git conflict, not a process breach). The advisor handled it correctly without catch-fire — this was the right call. ✓

### 6. §G4 classifier accuracy

One `validate-pending` entry (`6b7c4b329002-001`) was mutated this session:
- `result: pass` — no failure classification needed
- The `workflow_run_id: 0` sentinel was not a §G4 input (§G4 triggers on `result: fail`)

No §G4 classification decisions required. N/A ✓

### 7. L14 / L15 / L16 fixes still holding

- **L14:** `git log -3 governance-v0` shows `chore(bm): test PR opened #195` (e55116d1a) BEFORE the finalize-merge SHA (`576e1069f`). L14 ✓ held.
- **L15:** bm-merge gate-side checks have not yet run (bm-merge pending). L15 will be verified at bm-merge time.
- **L16:** post-merge branch deletion not yet applicable (bm-merge pending). Will verify at bm-merge time.

### 8. Subagent offload effectiveness

No `Agent` tool subagent dispatch under `/auto-phase` in this session. All orchestration work was inline advisor. Not applicable — per skill body Phase 0.6 "deferred per Phase 0.6." N/A

### 9. Plan §13 fidelity vs cohort dispatch

Cohort dispatch from prior session. Task-1 impl completed cleanly with no cohort degrade. The plan had a 2-task cohort (task-0 preflight + task-1 impl) dispatched serially (no `[P]` markers on this dogfood plan). No YAML overlap checks triggered. ✓

### 10. Resume-cycle pain points

`resume_count: 1` — one resume from context compaction. The Phase 0.5 reconciliation on resume found the state file accurately reflected reality (stage = `bm-poll-cr-running`, tip = `50195e511`). No discrepancies needed user input. The context-compaction summary was accurate enough to resume directly into bm-triage brief authoring. ✓

The "continue prompt" friction on resume was zero — the context-compaction summary ended mid-task with enough detail that the advisor resumed directly without a Phase 0.5 AskUserQuestion. This is a win: the summary mechanism worked as a cross-compaction handover.

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ✓ | 0× prior failure |
| 2. Cadence calibration | ✓ (N/A — manual session) | — |
| 3. Auto-state integrity | ✓ | 0× prior failure |
| 4. Touchpoint count | ✓ | on track |
| 5. Catch-fire FP/FN | ✓ | 0 fires |
| 6. §G4 classifier | ✓ (N/A — no failures) | — |
| 7. L14/L15/L16 holding | ✓ (L14 confirmed; L15/L16 pending bm-merge) | — |
| 8. Subagent offload | N/A (deferred per Phase 0.6) | — |
| 9. Plan §13 fidelity | ✓ | 0× degrade |
| 10. Resume cycles | ✓ | `resume_count: 1`; zero discrepancy friction |

No ⚠ or ✗ entries. All three "What to change" proposals above address newly observed friction (Shape G 422 path, YAML absence, Race-A mitigation) — none of them represent automation reliability regressions, just first-exposure learnings.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Change #1** (Shape G disabled `workflow_run_id: 0` fast-path): add to `advisor-orchestrator.md` §5.2 validate-pending-laptop handler — "if `workflow_run_id: 0`, skip `gh workflow run` attempt" — and update `advisor-validation.md` §"validate-pending-laptop handler". Threshold: 1× this session + Shape G RESIDUAL-ONLY memory = meets threshold.
- [ ] **Change #2** (bm-task-brief.template.md YAML recovery section): minor template update. Threshold: structurally recurs every phase. Meets 1×-here + structural-recurrence threshold.
- [ ] **Change #3** (fetch-before-write in BM transition): minor addition to `auto-phase.md` §3.1 or `advisor-orchestrator.md` §3.1. Threshold: 1× this session + `feedback_cross_session_commit_attribution_collision.md` class = meets threshold.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`, `feedback_auto_phase_retro_signals.md`._
