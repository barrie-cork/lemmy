---
name: Auto-phase retros need automation-reliability signals
description: When a sub-phase ran under the /auto-phase skill, retros must capture stage-transition correctness, cadence calibration, auto-state integrity, touchpoint count vs target, catch-fire FP/FN rate, §G4 classifier accuracy, L14/L15/L16 holding, subagent offload effectiveness, plan §13 fidelity vs cohort dispatch, and resume-cycle pain points. Without explicit retro questions for these, automation drift compounds invisibly across phases.
type: feedback
---

When a sub-phase ran under `/auto-phase` (state file at `.claude/auto-state/<phase>.json` exists, or its archived sibling `<phase>-archived-<ts>.json`), the canonical session retro template (`.claude/skills/session-retro/template.md`) is **insufficient on its own**. Three canonical headers (surprised / change / carry forward) plus three-signal scoring plus complexity scores capture friction in *manual* sessions. They do NOT capture the failure modes specific to *automated* orchestration.

**Why this matters now (and not for c-1 and earlier):** Automation amplifies bad signal as well as good. With manual orchestration, the human-in-the-loop catches subtle things — wrong cohort dispatched, advisor mis-routed a DQ, ci-watcher cycle stalled silently. With `/auto-phase` driving transitions autonomously between gates, those failures become invisible unless the retro asks for them explicitly. The user's framing for the skill (2026-05-08): *"reliability and accuracy is more important than speed, especially when the system offers automation"*. The retro is the only feedback loop that can keep automation honest.

## When this lesson applies

This lesson loads when ANY of these artifacts exist at retro time:

- `.claude/auto-state/<phase>.json` (live or archived)
- `.claude/auto-state/<phase>-archived-<ts>.json` (state file archived after `--reset` or `done`)
- `.claude/auto-state/<phase>-catchfire-<ts>.md` (catch-fire dump from prior `/auto-phase` run)
- The retro's session involved at least one `/auto-phase` invocation (visible in transcript)

Otherwise this lesson is dormant — manual session retros use the canonical three-lesson stack only.

## The 10 retro categories

Every `/auto-phase`-driven retro must answer each category below. "Nothing surprising" is a valid empty answer; the structure makes absences visible.

### 1. Stage-transition correctness

Did each `state X → state Y` transition fire on the right trigger? Did any transition fire prematurely? Did any fail to fire when its trigger was met?

**How to check.** Read the auto-state JSON's transition log (or git log of advisor commits with `chore(advisor):` prefix). For each transition:
- Confirm the trigger condition was satisfied at the recorded `last_action_at` timestamp
- Confirm no earlier opportunity to fire was missed
- Confirm cohort barriers held (no advance to N+1 with cohort N members in non-pass state)
- **Read every `user_gate_history[].notes` field** — gates capture durable findings the user shared at decision time (e.g. "DoD smoke 10/11 pass; e2e.rs LOC 11925 higher than planner expected"). These are first-class retro inputs, not freeform decoration. Surface non-empty `notes` as findings in §1 even if the gate decision itself was clean. The notes field is the cross-session communication channel between live `/auto-phase` ticks and retro-time review.

**Symptom of failure.** Cohort barrier crossed early (a non-pass member's failure was masked); a `*-pending-user` gate skipped (advisor inferred the answer instead of asking); a transition deadlocked (state stuck in `*-running` after Junior task completed but advisor missed the status change); a non-empty `user_gate_history[].notes` field that the retro author skipped reading (silent loss of the durable finding).

### 2. Cadence calibration

Were the `ScheduleWakeup` delays well-matched to actual wall-clock? Did the skill sleep too long (work sat idle) or too short (parent context bloated from probe noise that yielded no state changes)?

**How to check.** For each ScheduleWakeup pair (delay → next tick), compare the recorded delay against the actual time until the *next state-changing event*. Compute the "wasted poll" count: ticks that fired with no transition.

**Target.** Per skill body §"Cadence schedule": `ci-watcher` Phase 1 cadence is 270s (cache-warm); `planning-running` is 1200s/600s; `bm-pr-running` is 120s/60s. If any stage's wasted-poll count exceeded ~3 in this phase, the cadence is too aggressive. If any stage's wall-clock-to-detection lag exceeded ~5 min, the cadence is too lax.

**Symptom of failure.** ≥3 wasted polls on a single stage in the same run (over-eager); detection-lag >5 min on a state change the user noticed before the skill did (under-eager); 300s sleeps appearing anywhere (worst-of-both per cache-TTL guidance).

### 3. Auto-state integrity

Was the JSON ever corrupted, raced, or out-of-sync with reality? How many resume cycles happened? What was `resume_count` at phase end?

**How to check.** Read the final auto-state JSON. Inspect `resume_count`, `session_id` history, `last_known_phase_tip` accuracy. Cross-reference with `git log` for any advisor commits that surfaced a "state file inconsistency" line in the polling output.

**Target.** `resume_count` ≤ 3 per sub-phase. Higher values suggest either user friction (closing sessions early) or stage-routing bugs (the skill stalled, user gave up and restarted). `last_known_phase_tip` should always equal `git log -1 --format='%H' phase-<phase>` at retro time.

**Symptom of failure.** `resume_count` ≥ 5; phase tip drift detected mid-session that wasn't a daemon finalize-merge; auto-state JSON had to be hand-edited to recover.

### 4. User-touchpoint count vs target

Skill body says ≤8 interruptions per phase (six mandatory gates + ~2 push grants). Was that hit? Were any of the six mandatory gates accidentally skipped? Were any AskUserQuestion called when the answer was already in scope (false-positive friction)?

**How to check.** Read `user_gate_history` field from auto-state JSON. Count entries. Cross-reference against the six mandatory gates list (plan approval, judgment-heavy DQ, CR triage, Phase 2 e2e local-vs-dispatch, merge confirm, retro sign-off). Plus push-grant interruptions counted separately from advisor commits.

**Target.** 6-8 total. ≥6 = the six gates fired (none skipped). ≤8 = no false-positive friction.

**Symptom of failure.** <6 (a mandatory gate was inferred); >8 (over-asking — friction that the skill was meant to eliminate).

### 5. Catch-fire false-positive / false-negative rate

Did the skill catch-fire on something that shouldn't have stopped the loop? Did it FAIL to catch-fire on something that should have (silent corruption / reliability hit)?

**How to check.** For each `catchfire-<ts>.md` dump in `.claude/auto-state/`, classify: was the cause a *real* hard refusal violation / attribution breach / non-allowlist failure / catch-fire-listed condition? Or was it routine friction the skill could have handled?

Conversely, scan the runlog and DQ resolved entries for issues that surfaced during the phase — was any of them a should-have-been-catch-fire that the skill silently advanced past?

**Target.** Zero false-positives. Zero false-negatives. This category is the heart of automation reliability.

**Symptom of failure.** Even one false-positive (skill stopped when it shouldn't have) erodes user trust. Even one false-negative (skill advanced past a real issue) erodes system safety. Both are retro red-flags worth promoting.

### 6. §G4 classifier accuracy

When the skill auto-classified a `result: fail` as allowlist (auto-fix-impl) or non-allowlist (catch-fire), was the classification correct?

**How to check.** For each `validate-pending` DQ entry mutated to `result: fail` during this phase, read the §G4 classification and the resulting action. Then read the actual root cause (from the fix commit, or the user's resolution if catch-fire fired). Compare.

**Target.** ≥95% classifier accuracy. False-positive (auto-queued fix-impl for a real bug) is the more dangerous error class — it ships incorrect code under autonomous flow.

**Symptom of failure.** Allowlist match auto-queued a fix-impl that the user later had to revert; non-allowlist catch-fire fired on something the existing allowlist patterns covered (allowlist is too narrow); same lint pattern appeared multiple times without being added to the allowlist between phases.

### 7. L14 / L15 / L16 fixes still holding

These three fixes from c-1 retro encode regressions that automation must prevent:

- **L14** — BM Junior commits runlog BEFORE merge per the explicit git-sequence directive in `bm-merge.md` preamble.
- **L15** — Advisor-side gate-only checks run inline (no Junior dispatch); only the mutating action queues a Junior task.
- **L16** — `gh pr merge --delete-branch` silent-skip case is caught by post-merge `git ls-remote` verification + advisor-side `gh api -X DELETE`.

**How to check.** For each fix, look at the actual phase trail:
- L14: `git log -3 governance-v0` post-merge shows a `chore(bm)` commit *before* the merge SHA.
- L15: bm-merge brief was for Phases 5-9 only; gate-side checks ran inline (visible in advisor commits, not Junior task output).
- L16: post-merge `git ls-remote origin refs/heads/<phase-branch>` returned empty (branch deleted on first try); if not, the `gh api -X DELETE` fallback ran without surfacing.

**Target.** All three fixes hold. If any held by-default (the failure mode never reproduced) AND the underlying preventative measure was redundant, that's a candidate to simplify in c-3+.

**Symptom of failure.** A regression class returned (L14: runlog commit missed; L15: pre-confirm Junior dispatch; L16: stale remote branch). Promote any recurrence to a stronger preventative measure or a new lesson.

### 8. Subagent offload effectiveness (when used)

When `/auto-phase` did delegate to a `general-purpose` subagent (per skill body Phase 0.6 "deferred — let retros find the signal"), was the synthesis usable on first read or did it need re-derivation? Did delegation save context vs cost token spend?

**How to check.** Find each Agent invocation in the transcript. For each:
- Did the synthesis answer the question without follow-up reads? (yes / partial / no)
- What was the parent context size before vs after? (rough estimate from `/context` snapshots)
- What was the subagent's token spend? (from telemetry if available)

**Target.** First-read usability. Net-positive context conservation.

**Symptom of failure.** Subagent synthesis vague / required re-derivation; advisor parent context grew despite delegation; subagent burned >50k tokens for a probe inline mode would have done in <5k. If delegation pattern fails consistently, push back — pre-c-2 deferral was correct.

### 9. Plan §13 fidelity vs actual cohort dispatch

The skill computes cohorts from §13 `[P]` markers + YAML overlap check + budget check. Did the planner's `[P]` markers match actual file disjointness? Were cohorts force-degraded to serial mid-flight?

**How to check.** Read the plan's §13. For each cohort the skill dispatched:
- Did all `[P]`-marked tasks in the cohort actually have disjoint file edits? (Read each task's commit + verify no overlap with peers.)
- Did the YAML overlap check fire any "cohort overlap detected" degrade-to-serial messages in advisor output?
- Did the budget check force any degrade-to-serial?

**Target.** Zero degrades. If degrades fired, the planner mis-marked a cohort — feedback signal for the planner's next plan.

**Symptom of failure.** Multiple degrades in one phase (planner systematically over-marks `[P]`); a cohort that the skill dispatched in parallel actually had a hidden file overlap that didn't trigger the YAML check (escape hatch — needs YAML check tightening).

### 10. Resume-cycle pain points

Every Phase 0.5 entry interrupts the autonomous loop with a 'continue' prompt. After this phase, is that friction worth it, or should the skill auto-continue if reconciliation finds zero discrepancies?

**How to check.** For each `resume_count` increment, evaluate:
- Did Phase 0.5 reconciliation find discrepancies that needed user input? (yes / no)
- If no discrepancies found, was the 'continue' prompt informative or noisy?
- Did the user need to override anything via `--start-from`?

**Target.** First c-2 phase: keep the 'continue' friction (reliability over speed). c-3+: if zero discrepancies-needing-input across N phases, propose auto-continue with a "X discrepancies found" header.

**Symptom of failure.** User overrode via `--start-from` more than once (Phase 0.5 routing was wrong); user surfaced "I already saw this state, why are you asking" friction (over-friction).

## How to apply at retro time

The session-retro skill (`.claude/skills/session-retro/SKILL.md` Step 1.5) detects `/auto-phase` artifacts and loads this lesson. The retro author iterates the 10 categories and writes findings into the optional **Auto-phase reliability** block in `.claude/skills/session-retro/template.md`. Findings that meet the recurrence-≥2 threshold (this phase + ≥1 prior, or ≥2 in this phase across multiple gates) get promoted to a dedicated `.claude/lessons/feedback_<topic>.md` per the standard promotion pattern.

Per the skill's existing discipline (`feedback_retro_not_report.md`): "What to change" is the heart. For each of the 10 categories, if the answer is "this didn't go well", the retro must propose a concrete fix — a stage-routing change, a cadence adjustment, a new catch-fire condition, an allowlist addition, a lesson promotion. Empty findings are valid; vague findings are not.

## Generalises to

Any harness-driven autonomous orchestration with persistent state (`/loop`, scheduled cron-driven agents, daemon-spawned long-running tasks). The 10 categories are written for `/auto-phase` specifically, but the underlying discipline — *automation amplifies bad signal as well as good; the retro must explicitly look for automation-class failures or they go invisible* — applies wherever a system advances state without human-in-the-loop on every step.

## Symptom to recognise

A `/auto-phase`-driven phase retro that uses only the canonical three-headers + three-signal-scoring + complexity-scores template. The retro will read like every prior manual-orchestration retro, but the failure signals it ought to surface (stage-transition correctness, cadence drift, catch-fire FP/FN, classifier accuracy) won't appear — not because they didn't happen, but because nobody asked. That silent-skip across one phase is forgivable; across three phases compounding, it's the path to an automation system that nobody trusts.
