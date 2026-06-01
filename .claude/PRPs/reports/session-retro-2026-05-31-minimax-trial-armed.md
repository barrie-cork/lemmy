# Session retro — 2026-05-31 — minimax-trial-armed

**Harness:** claude-code
**Session window:** ~2026-05-31T14:00Z → ~2026-05-31T16:00Z (~120 min est.)
**Branch at start:** `fa091be00` (`governance-v0`)
**Branch at end:** `1785f1032` (`governance-v0`)
**Files touched:** 4
**Commits:** 3 (auto: 0, explicit: 3)

## TL;DR

Session clarified the MiniMax M2.7 A/B trial history, converted its
trigger from a named-phase gate to a rolling cumulative counter, and
wired auto-designation into the plan-approval gate (§3.5a). The key
insight — that e2e tasks self-exclude by criteria, so the trial is safe
to arm broadly — came from the user, not the advisor. The advisor had
been overly conservative about the trial scope. The structural change is
low-risk, reversible, and adds zero manual overhead to the plan-approval
flow going forward.

---

## What surprised us

- **Advisor over-conservatism on trial scope.** The advisor's initial
  response to "can we configure it for remaining RT-r5 tasks?" was to
  list three reasons it wouldn't work — then the user immediately
  identified that e2e exclusion is a criterion, not a blocker, making
  the trial safe to arm broadly. The advisor was pattern-matching to
  "don't run trial on insufficient tasks" without seeing the obvious
  fix: bake the exclusion into the criteria and let tasks accumulate
  across phases.

- **The trial had zero recorded data despite being armed since 2026-05-29.**
  RT-r4 was forced serial by OOM; RT-r5 had only one qualifying task.
  Two full sub-phases elapsed without firing. Without the rolling
  cumulative counter, a future advisor might have reset the trial
  intent entirely rather than recognising the tasks were still worth
  accumulating.

- **The §3.5a step name worked cleanly.** §3.6 was already taken
  (canonical-schema-first gate), so the new step slotted in as §3.5a.
  This is an unusual numbering but the rule file accepted it without
  ambiguity and the stage-shape line updated cleanly.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | At plan approval, run §3.5a designation pass and report `MiniMax trial: N/5` in the approval surface | Advisor never forgets to check; user sees running count without asking | already wired in `advisor-orchestrator.md` §3.5a | 1× this session (first wiring) |
| 2 | When cumulative ✅ count reaches ≥5 at plan approval, surface a clear "trial fires this phase" flag to user before queueing bm-cut | User can confirm or defer before any ab-test branches are cut | minor — add to §3.5a prose | 0× prior — pre-emptive |
| 3 | Add the cumulative ✅ count to MEMORY.md `project_minimax_ab_trial_deferred.md` entry so it's visible at session-start without reading the runbook | Advisor doesn't re-derive count from the table | minor — update memory file when count changes | 0× prior — pre-emptive |

## What to carry forward

- **User identifies the obvious fix, advisor implements it.** When the
  user pushes back on an over-conservative advisor position, assume
  they've identified a real gap in the reasoning — don't defend the
  position, probe it. The e2e-exclusion insight was a 10-second
  observation that unlocked the whole approach.

- **Rolling cumulative counters beat named-phase gates for optional
  work.** Any optional trial or experiment that needs N data points
  but can't guarantee N qualifying events per phase should use a
  cumulative counter tracked in the runbook table. Named-phase gates
  expire silently when the phase doesn't materialise; cumulative
  counters don't.

- **Criteria-driven self-exclusion is more robust than manual
  exclusion lists.** The original trial runbook had an explicit
  "Task 7 excluded (18k-line e2e.rs)" note. The §0.1 qualifying
  criteria now encode the same exclusion structurally — any task
  whose primary file is `e2e.rs` fails criterion 2 without the
  advisor needing to remember the exclusion.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Reading trial history from memory + runbook | 5 | 0 | low | Answered the MiniMax outcome question in one read cycle; memory was current |
| Advisor initial RT-r5 feasibility assessment | 0 | 5 | medium | Over-conservative; user corrected immediately |
| MiniMax endpoint re-verification (curl) | 2 | 0 | none | HTTP 200 on first probe; key still funded 2 days on |
| §3.5a rule authoring + stage-shape update | 15 | 0 | none | Clean insertion at an existing numbered section gap |
| Runbook §3 restructure (rolling table) | 10 | 0 | low | Template row pattern worked well; per-phase history rows make the state legible |
| Memory file update | 5 | 0 | none | Straightforward overwrite; MEMORY.md index line updated in-place |
| Commit 3× (runbook, orchestrator, both) | 5 | 0 | none | All three committed cleanly; no pre-commit hook friction |

## Complexity scores (heavy tasks only)

No Junior tasks dispatched this session. All changes were advisor-side
rule/runbook edits — well within the lightweight category. No complexity
score applicable.

## Decisions to revisit

- **Item 2 from "What to change"** (surface "trial fires this phase"
  flag to user at plan approval when count ≥5): worth a one-line
  addition to §3.5a prose at the next convenient session — low cost,
  prevents the advisor from silently starting the trial without an
  explicit user acknowledgement.

- **Running total in memory:** once the first qualifying tasks
  accumulate (quality-r3c or later), update `project_minimax_ab_trial_deferred.md`
  with the live count so session-start context shows the current
  state without reading the runbook table.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Rolling cumulative counter pattern** (beat named-phase gates
  for optional work): promote to `.claude/lessons/feedback_rolling_cumulative_trial_counter.md`
  — generalises beyond MiniMax to any N-data-point optional experiment.
  *Threshold: 1× here + the RT-r4 deferred-trial incident = 2× effective.
  Promote if a third instance appears (another trial or experiment
  that needs the same pattern).*

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
