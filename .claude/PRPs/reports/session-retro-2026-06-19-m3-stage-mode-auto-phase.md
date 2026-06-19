# Session retro — 2026-06-19 — m3-stage-mode-auto-phase

**Harness:** claude-code
**Session window:** 2026-06-18T21:26Z → 2026-06-19T10:05Z (~12.5h wall-clock, mostly idle/sleeping between ScheduleWakeup ticks)
**Branch at start:** `eb1bcaca0` (`governance-v0`)
**Branch at end:** `09107020e` (`governance-v0`)
**Files touched:** ~14 source/test files on the phase branch (stage.rs, room_event_client.rs, room_provisioner.rs, livekit_jwt.rs, bridge_room.rs, config.rs, main.rs, governance.rs, governance_log.rs, bridge_notify.rs, stage_mode.rs) + ~18 advisor meta-files (briefs, DQ, retro, lesson, bootstrap)
**Commits:** 47 on gov-v0 (all advisor/BM meta) + 14 on the phase branch (impl/fix)

> **Scope note.** This is the *session-scope* cross-harness retro for the whole m3-core-stage-mode phase, with the required **Auto-phase reliability** 10-category section. The per-role *sub-phase* retro (`m3-core-stage-mode-retro.md`, gate-6 signed off) is complementary — it carries the domain/impl signals; this one carries the orchestration-reliability signals.

## TL;DR

`/auto-phase` drove M3 town-halls Phase 3 (chair-controlled stage mode, bridge-side) end-to-end across a `/compact` boundary and ~dozen ScheduleWakeup ticks: 6 impl tasks (Cohort A `[P]` + 4 serial), 5 mechanical fix-impls, 1 CR fix-impl, PR #202 merged @ `a8713dd75`, retro + lesson + phase-transition to m3-core-emergency-mute. All 6 user gates honoured (4 fired; e2e-mode correctly N/A for a unit-test DoD; push-grants 0). The PR/CR flow earned its keep: CodeRabbit cr-4 caught a REAL MAJOR single-presenter-invariant gap that cargo + the task's own test both passed green. Two recurring daemon footguns (long-name refspec, daemon-local-first finalize) handled deterministically; one promoted to a lesson (4× recurrence).

## What surprised us

- **The single-presenter gap was invisible to the test that should have caught it.** The marquee `fifo_mic_pass_in_sequence` asserted grant *order* only, so 4 back-to-back `promote_next` calls fired 4 grants with 0 revokes and stayed green — a genuine concurrent-publisher bug that compiled, passed clippy, and passed its own test. Only CodeRabbit's cr-4 surfaced it. The grace path already revoked; the asymmetry between grace-path and direct-promote-path was the trap.
- **The daemon long-name refspec quirk recurred 4× in one phase.** `git rev-parse origin/<worker>` resolves empty for `junior/role-*` branch names past some length threshold — every finalize-merge needed the explicit-local-ref fetch. Frequent enough this phase alone to promote.
- **The daemon ran its OWN finalize-merge of bm-merge #723** AFTER the GitHub PR merge already landed, producing 3 divergent daemon-local commits — one of which was a *legitimate* bootstrap tombstone I had to preserve, not discard. Origin was a clean ancestor so the FF push kept the tombstone, but it was a near-miss for a reset-to-origin clobber.
- **`resume_count` stayed 0** despite the session spanning a `/compact` + many wakeups. The field never incremented — the schema-upgrade backfill in Phase 0.5 didn't fire because the resume path used the live invocation, not the fresh-init path. Auto-state under-reported its own resume cycles.

## What to change

1. **Marquee tests on authz-shaped state machines must assert the negative invariant, not just positive ordering.** The cr-4 fix added `windows(2)` revoke-before-grant assertions; the *original* test should have had them. **Proposal:** add a one-line note to `.claude/lessons/feedback_governance_type_state_handlers.md` (or a new `feedback_authz_state_machine_test_asserts_negative.md`): "a single-X invariant (single presenter, single chair, single lock-holder) test must assert no second holder remains, not just that the right holder was granted." File path + the cr-4 commit `0258fef07` as the worked example.
2. **`resume_count` is not tracking actual resume cycles** — it read 0 after a `/compact` + ~dozen wakeups. **Proposal:** the auto-phase skill body's Phase 0.5 should increment `resume_count` on EVERY resume entry (compact-resume + wakeup-resume), not only on the cold-init schema-upgrade path. Without it, retro category §3 (auto-state integrity) and §10 (resume-cycle pain) can't be measured. File: `~/.claude/commands/auto-phase.md` Phase 0.5 Step A.
3. **The daemon long-name refspec quirk is now a lesson** (`feedback_daemon_long_name_refspec_finalize.md`, written this phase) — make the explicit-local-ref fetch the *default* for finalize-merges, not a fallback. Already promoted; the residual change is to fold the recipe into the auto-phase skill's finalize-merge step so future phases don't rediscover it.

## What to carry forward

- **The PR/CR flow is load-bearing on bridge code** — it caught cr-4 (a real correctness gap green on cargo + unit test). Keep CR review on every bridge PR; do not skip "because the diff is small / bridge-only."
- **The fix-impl-skips-validate-DQ recurrence-watch HELD** this phase (#722 wrote its DQ). Keep verify-via-DoD-grep regardless — 1 clean phase ≠ resolved (it was 2× on m3-core-infra). Don't downgrade the watch yet.
- **The emit-intent→drain seam** (sync `pending_emits` accumulate / async `drain_emits`) kept the state machine unit-testable while deferring async I/O — reusable "keep core testable, defer I/O" pattern. `room_mute_all` (Phase 4) likely uses it.
- **Falsifiable-hypothesis gate on CR findings paid off** — reading `promote_next` confirmed cr-4 was REAL (not a CR false positive) and confirmed cr-3 was an intentional scaffold (rebut). Both verified against code before triaging.

## Three-signal scoring

| Invocation | Saved (min) | Wasted (min) | Surprise |
|---|---|---|---|
| `/auto-phase` (whole-phase orchestration, 18 dispatches, ~dozen wakeups) | ~180 (no re-derivation of stage shape per transition) | ~10 (worktree-add footgun + daemon-merge reconciliation analysis) | medium (daemon-local-first finalize + resume_count=0) |
| CR triage + falsifiable-hypothesis gate (4 findings) | ~25 (verified cr-4 real / cr-3 scaffold before acting) | 0 | medium (cr-4 was a real green-passing bug) |
| `/brehon-verify` re-run after cr-4 | ~8 (mechanical story re-confirm) | 0 | none |
| `/brehon-phase-transition` (bootstrap + workflow-state + DQ cleanup) | ~30 (vs hand-authoring handoff) | 0 | low (3 stale validate DQ entries needed cleanup) |
| bridge Linux validation (cargo-linux, 3 worktrees: 718/719/722) | n/a (mandatory gate) | ~5 (cold Docker re-compile per fresh worktree) | none |

## Complexity scores (heavy tasks only)

`<files>/<commits>/<runtime-min>/<silence-min>` — silence-min not instrumented (no telemetry CSV).

| Task | complexity | Note |
|---|---|---|
| Task 3 — stage core (FIFO + mic-pass) #711 | 3/2/15/— | marquee; +1 fix (unwrap_or_default) |
| Task 5 — room-event client + emit seam #717 | 6/2/~16/— | largest file count; +1 fix (dead_code) |
| Task 6 — provisioning + drain #719 | 5/1/~18/— | removed 3 dead_code allows |
| cr-4 — single-presenter fix #722 | 1/1/2.5/— | the only post-PR fix; smallest, highest-leverage |

No task breached the >55min / >8-files carry-forward thresholds. 5 fix-impls across 6 tasks — all mechanical (clippy/feature-gate/dead_code), 0 logic re-plans.

## Decisions to revisit

- **Should `room_mute_all` emission reuse the `EmitIntent`/`pending_emits` seam, or does cross-instance mute need a different path?** Carried into the m3-core-emergency-mute bootstrap watchlist (item 3: power-levels ≠ LiveKit grants). Decide at Phase 4 plan time.

---

## Auto-phase reliability

### 1. Stage-transition correctness
All transitions fired on the right trigger; cohort barriers held (Cohort A Tasks 1+2 both `pass` before advancing; serial Tasks 3-6 each validated before the next). No premature fires, no deadlocks. **`user_gate_history[].notes` reviewed** — all 4 substantive: plan-approval recorded the 2 ratified planner scope DQs + DoD smoke results; cr-triage recorded the full 4-finding falsifiable assessment + the bm-poll-cr daemon-local-only-merge catch; merge-confirm recorded the UNSTABLE=CR-non-blocking reasoning; retro-sign-off recorded all-6-gates-cleared. No durable finding lost. ✓

### 2. Cadence calibration
ScheduleWakeup delays: 420s (cr-4 dispatch), 300s (bm-merge), 1200s (cr-4 validation fallback). Most ticks were superseded by the daemon completion-hook re-invocation (background-task notification fired before the fallback in every case). 0 wasted polls — every wake landed on a real state change or was pre-empted by a completion hook. No 300s-exactly sleeps except the bm-merge one (bm-merge is a ~1-2min gh op, 300s slightly lax but pre-empted by the hook). ✓ (one borderline 300s — acceptable, hook pre-empted)

### 3. Auto-state integrity
JSON never corrupted or hand-edited to recover; all edits were clean surgical Edits. `last_known_phase_tip` accurately tracked (9152aa4d6 → 8b8c877a7 across the cr-4 merge). **`resume_count: 0` is WRONG** — the session spanned a `/compact` + ~dozen wakeups; the field never incremented. ⚠ (see §"What to change" #2)

### 4. User-touchpoint count vs target
4 gates fired (plan-approval, cr-triage, merge-confirm, retro-sign-off). Gate 4 (e2e local-vs-dispatch) correctly N/A — the stage-mode DoD is deterministic unit tests, no e2e. Gate 2 (judgment-heavy ADR/scope DQ) didn't fire — no ADR-affecting DQ arose (the planner scope DQs were ratified at plan-approval). Push-grants: 0. Total 4 ≤ 8, no false-positive AskUserQuestions, no mandatory gate skipped. ✓

### 5. Catch-fire FP / FN rate
0 catch-fires this phase (the `m3-core-entry-kinds-catchfire-2026-06-18.md` dump is from the PRIOR phase, not this one). No false-positives (loop never stopped on routine friction). No false-negatives — cr-4 was caught by CR (the intended channel), not silently advanced past; the daemon-local-first finalize divergences were caught by the mandatory finalize-hazard check, not missed. ✓

### 6. §G4 classifier accuracy
No `validate-pending` failures this phase — every cargo-linux validation passed first try (the 5 fix-impls were dispatched off CR/clippy findings, not §G4 auto-classification). The §G4 classifier was not exercised. N/A this phase.

### 7. L14 / L15 / L16 fixes still holding
- **L14 (BM commits runlog before merge):** ✓ — bm-merge #723 wrote the runlog entry (`592a1d3bf`/`63315f069`) with the merge SHA.
- **L15 (gate-side checks inline, no pre-confirm Junior dispatch):** ✓ — merge-gate mergeability poll + status-check-rollup ran inline in the advisor session; only the mutating `gh pr merge` was dispatched to Junior #723.
- **L16 (post-merge branch deleted):** ✓ — BM runlog confirms "remote branch deleted? yes".

### 8. Subagent offload effectiveness (when used)
No `Agent`-tool subagents dispatched this segment (the resume path ran inline; the post-compact resume was a thin ScheduleWakeup prompt, not a Phase-0.5 Explore-subagent reconciliation). N/A — deferred; the inline path was cheap enough given the work was already well-scoped by the resume prompt.

### 9. Plan §13 fidelity vs cohort dispatch
One cohort dispatched: Cohort A (Tasks 1+2, `[P]`). The `[P]` markers were genuinely file-disjoint — Task 1 = crates-side DTO (Windows validation), Task 2 = bridge-side livekit (Linux validation), zero file overlap. No YAML-overlap degrade-to-serial, no budget degrade. Tasks 3-6 correctly serial (each depends on the prior's stage.rs state). ✓

### 10. Resume-cycle pain points
The session resumed across a `/compact` + multiple ScheduleWakeup ticks. Each resume picked up cleanly from the thin wakeup prompt + auto-state — no Phase-0.5 discrepancy needed user input, no `--start-from` override. The `/compact` resume re-read the brief + auto-state and continued without friction. BUT `resume_count` didn't track these (see §3) so the "zero discrepancies across N phases → propose auto-continue" trend can't be measured yet. ⚠ (instrument resume_count first)

### Aggregate auto-phase reliability score

| Category | Status | Recurrence |
|---|---|---|
| 1. Stage-transition correctness | ✓ | 0× prior |
| 2. Cadence calibration | ✓ | 0× prior |
| 3. Auto-state integrity | ⚠ | resume_count=0 — 1× this phase (new finding) |
| 4. Touchpoint count | ✓ | 0× prior |
| 5. Catch-fire FP/FN | ✓ | 0× prior |
| 6. §G4 classifier | N/A | not exercised |
| 7. L14/L15/L16 holding | ✓ | 0× prior |
| 8. Subagent offload | N/A | deferred |
| 9. Plan §13 fidelity | ✓ | 0× prior |
| 10. Resume cycles | ⚠ | resume_count untracked — same root as §3 |

Two ⚠ (§3 + §10) share one root cause: `resume_count` not incrementing. Both fold into §"What to change" #2. Single-phase occurrence — recorded, not yet promoted to a lesson (1× — promote if it recurs next phase).

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [x] **Daemon long-name refspec finalize** → ALREADY promoted to `.claude/lessons/feedback_daemon_long_name_refspec_finalize.md` (4× this phase). Done.
- [ ] **Authz-state-machine test asserts negative invariant** (cr-4): add note to `feedback_governance_type_state_handlers.md` OR new `feedback_authz_state_machine_test_asserts_negative.md` — recurrence 1× here; promote if a second single-X-invariant gap appears.
- [ ] **`resume_count` not incrementing on compact/wakeup resume**: update `~/.claude/commands/auto-phase.md` Phase 0.5 Step A to increment on every resume entry — recurrence 1× here (new finding); fix is cheap, recommend doing now rather than waiting for recurrence since it blocks §3/§10 measurement.
- [ ] **Fold the daemon long-name refspec recipe into the auto-phase finalize-merge step** so it's the default, not rediscovered per phase — `~/.claude/commands/auto-phase.md` finalize-merge section.
