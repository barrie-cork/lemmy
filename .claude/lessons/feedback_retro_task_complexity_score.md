---
name: Per-task complexity score in every impl-task retro section
description: One-line metric per impl-task to surface bundling drift — files-touched / commits / runtime-min / max-log-silence-min. Aggregated across a sub-phase, reveals whether planning is bundling tasks too aggressively for the Sonnet+watchdog envelope.
type: feedback
---

Every retro that has per-task sections (which is "every retro" under the four-role model per `feedback_four_role_retro_signals.md`) must record a one-line complexity score for each impl-task that ran during the sub-phase. The score is mechanical to compute and surfaces planning-side bundling drift before it costs a watchdog kill.

## The metric

`complexity: <files-touched>/<commits>/<runtime-min>/<max-log-silence-min>`

Example: `complexity: 11/1/65/43` reads as "11 files touched, 1 commit, 65 minutes total runtime, 43 minutes maximum log-silence window".

Each component is mechanical:

- **files-touched**: `git show --stat <impl-commit>` line count, or sum across multiple commits if the task chained.
- **commits**: number of commits in the impl-task's commit chain (typically 1 per the "one commit per task" rule).
- **runtime-min**: time from `mcp__junior-brehon__show_task` `created_at` to `completed_at`, rounded to whole minutes.
- **max-log-silence-min**: the longest gap between consecutive stdout lines in `/srv/brehon-fork/.junior/logs/job-N-run-N.log`, captured during polling via `junior-telemetry-tick.sh`. The retro author reads the per-task telemetry CSV (`/tmp/junior-telemetry-brehon-fork-job-<N>.csv`) and takes the max `log_age_seconds` value, divided by 60.

## Why

**The Sonnet impl-task envelope has hard limits the planning subagent doesn't see.** The 60-minute Junior watchdog kicks if no stdout line emits — large multi-file Edit batches can stall the worker before the model finishes composing. The 200k Sonnet context window fills more slowly but can be hit on dense plan-cited reads.

When the planning subagent bundles aggressively (e.g. a single task that touches an enum + schema regen + 3 Diesel models + 6 R3 sweep sites), the resulting impl-task may take 60+ minutes with 40+ minutes of log silence. That envelope is invisible at plan-write time — the planner sees "11 file edits, all conceptually one operation" and writes one task. The complexity score makes the envelope visible at retro time.

**Aggregated across sub-phases, the metric reveals trends:**

- A sub-phase whose impl-tasks score `2/1/15/4`, `3/1/22/8`, `4/1/30/12` is in the comfortable zone — small, fast, low-silence.
- A sub-phase whose tasks score `11/1/65/43`, `8/1/55/35`, `9/1/60/40` is repeatedly stressing the watchdog envelope. The planner should split future plans more aggressively.
- A sub-phase with mixed scores `2/1/15/4`, `11/1/65/43`, `1/1/8/2` reveals a single outlier task — usually the schema/migration task in a Diesel-extension sub-phase. Worth flagging in §3 (carry-forward for next plan).

## How to apply

Under each task's H3 sub-section in §1 (What worked) or §2 (What surprised), add the complexity line as the first sentence. Example:

```markdown
### 1.2 Task 2 — Diesel models + R3 sweep
complexity: 11/1/65/43

The R3 sweep discipline from `feedback_insertform_default_propagation.md` worked
cleanly. Six call sites enumerated correctly via grep before edit; no drift
from plan §13's cited count.
```

For sub-phases that ran on a four-role model with multiple impl-tasks, the per-task complexity scores feed a §5 aggregate:

```markdown
## 5. Quantified outcomes vs confidence score

Plan's §20 predicted 8.5/10. Actual: 8/10.

### 5.1 Complexity scores

| Task | complexity | watchdog risk |
|---|---|---|
| Task 1 (migrations) | 3/1/12/4 | low |
| Task 2 (Diesel models + R3) | 11/1/65/43 | high — outlier |
| Task 3 (request_appeal rewrite) | 5/1/28/15 | medium |

Median: 5/1/28/15. Outlier: Task 2 — bundle the schema regen separately in
future plans (carry-forward §3.x).
```

## Generalises to

Any orchestration model where worker subagents have wall-clock or context-budget limits invisible to the planner. The complexity score is the planner's feedback signal — without it, planning drifts toward "bundle for conceptual cohesion" at the cost of "fit the execution envelope." The metric can be extended to bm-task and planning-task retros (with different thresholds), but impl-task is the canonical use case.

## Symptom to recognise

In review of a finished sub-phase, look for: a task that took >55min total runtime, >40min of any single log-silence window, or >8 files touched in one commit. Any one is an early-warning signal; two together is a near-miss; three is a planning bug to carry forward.