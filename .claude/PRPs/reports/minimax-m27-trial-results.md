# MiniMax M2.7 vs Sonnet 4.6 — impl-task A/B trial — v1-RT-r4 (NOT RUN)

**Status:** DEFERRED — trial armed at v1-RT-r4 cut but not executed.

**Why not run:** v1-RT-r4 hit an early OOM on the cohort-1 parallel dispatch
attempt, forcing a switch to **serial one-task-at-a-time** dispatch for tasks
1-7 (per user directive). The A/B trial design requires parallel impl arms
(same task, two models, compared) — serial dispatch left no parallel arm to
compare against. The trial cannot run meaningfully under serial dispatch.

**Designated trial tasks (from cut):** 5 MIRROR-ref §13 tasks were tagged at
planning for the A/B comparison (per `project_minimax_ab_trial_deferred.md` +
`.claude/PRPs/briefs/minimax-m27-trial-1.md`). All ran as Sonnet-only in the
serial dispatch; no MiniMax arm executed.

**Carry-forward:** the trial re-arms at the next MIRROR-ref-heavy sub-phase
with parallel-safe cohorts. Runbook `.claude/PRPs/briefs/minimax-m27-trial-1.md`
is unchanged and reusable. Designation criteria (5 MIRROR-ref tasks, impl-task
only, M2.7 not M2.5) carry forward verbatim.

**No data produced.** Nothing to compare; no winner; no model-tiering change.
The `feedback_brehon_anthropic_only.md` posture is unchanged (MiniMax remains
an untested candidate, not adopted).

---

_Authored by advisor at v1-RT-r4 closeout 2026-05-30 to record the deferred
trial so the next MIRROR-heavy phase re-arms it rather than re-discovering the
intent._
