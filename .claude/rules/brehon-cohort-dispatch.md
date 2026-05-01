---
paths:
  - ".claude/PRPs/briefs/**"
  - ".claude/PRPs/plans/**"
---

# Brehon cohort dispatch (impl-task parallelism)

> Path-scoped sub-rule of `advisor-orchestrator.md`. Loads when the advisor opens brief or plan files — exactly when cohort decisions get made. Brehon-fork canonical mirror at `brehon-fork/.claude/rules/advisor-orchestrator.md` (one file, all sections inline).

Per `.claude/PRPs/templates/plan.template.md` §13 (`[P]` markers) and `feedback_parallel_cohort_dispatch.md`:

When a plan §13 task carries `[P]` and is the next pending task, the advisor computes the **cohort** — all consecutive `[P]`-marked tasks from the next pending forward, until a non-`[P]` boundary (the barrier task). Task 0 (pre-flight harness audit) is **always** non-`[P]`, so it queues alone.

## Cohort dispatch sequence

1. Read plan §13. Locate the next pending task by id (smallest-numbered task whose impl commit is not yet on the phase branch).
2. If that task is non-`[P]` (or it is Task 0): queue it alone via `mcp__junior-brehon__create_task` and wait for complete/failed before computing next.
3. If that task is `[P]`: walk §13 forward collecting consecutive `[P]` tasks until a non-`[P]` boundary or end-of-list. The collected list is the **cohort**.
4. **YAML overlap check** (per `feedback_explicit_file_arrays_on_tasks.md`): for each cohort task, parse the **FILES** YAML block from §13 (`creates:` + `modifies:` arrays — the planner asserts `union(creates, modifies) == set(IMPLEMENT files)`). Compute pairwise intersections across cohort members. If any intersection is non-empty, **refuse the cohort and degrade to serial**: queue cohort members one at a time. Surface the overlap as: `cohort overlap detected: tasks <A>+<B> share <path> — degrading to serial`. Do not file a DQ for this — the planner's `[P]` marker was wrong, and the next retro should flag the planner miss; in-flight, serial dispatch is correct + safe. If the YAML block is missing on any cohort member (pre-V1 plans, or planner mistake), back-compat applies: skip the YAML check and trust the `[P]` marker (the cohort can still merge-conflict at finalize, which is the failure mode this check exists to prevent forward-going).
5. **Budget check**: estimate cumulative cargo memory for the cohort (each `cargo check --workspace --features full` ≈ 6 GB peak per the EliteDesk's deployed cap; serial budget is `MemoryMax=10G`). If `cohort_size × per_task_peak > 10 GB`, **degrade to serial** — queue the cohort tasks one at a time as if non-`[P]`. Per `feedback_resource_budget_pre_queue.md`. Note the degrade in the polling-loop output: `cohort degraded to serial: budget exceeded (<size> tasks × <peak> GB > 10 GB)`. Under Shape G the budget check is non-binding (cargo runs off-box).
6. **Forbidden-window check**: re-evaluate the forbidden-windows table for the cohort's expected start time. If any cohort task would start in a forbidden window, defer the entire cohort per the existing self-defer rule. Cohort dispatch and forbidden-window deferral compose naturally — the advisor defers the whole cohort, not individual tasks.
7. **Queue every cohort task simultaneously** via parallel `mcp__junior-brehon__create_task` calls (single message, multiple tool uses). Each task gets its own Junior worktree per `feedback_parallel_agents_one_worktree_per_agent.md`. Brief paths are unique per task (`.claude/PRPs/briefs/<phase>-impl-<N>.md`).
8. **Wait for all cohort members to reach complete or failed** before computing the next cohort. A failed task in the cohort blocks advancement — the advisor surfaces the failure (catch-fire if it's a hard-refusal violation) and does not queue beyond the failure boundary until resolved.
9. **On cohort completion (all members `complete` and validated):** run the cohort handover aggregation step (next sub-section) before computing the *next* cohort. The aggregated handover populates the next cohort's brief §3a "Handover from prior cohort" before any of those tasks gets queued.

## Cohort dispatch refusals

- **Never queue a cohort whose tasks have not all been clarified.** The clarify gate runs once per planning brief, but if a cohort's tasks reference §13 entries that surfaced new ambiguity post-clarify (e.g. a brief edit introduced overlap), file a DQ pending entry and re-run `/brehon-clarify` on the affected brief.
- **Never queue a cohort during a forbidden window**, even partially. Either the entire cohort defers or none does.
- **Never re-queue a cohort task that already shows running.** Junior's task IDs are unique per worktree; re-queueing creates a duplicate worktree and conflicting branch names.
- **Never queue a `[P]` task whose IMPLEMENT files overlap a non-`[P]` task that's still running**. The `[P]` marker is a planner-side promise of file disjointness within the cohort, not across cohort boundaries — if the prior cohort's barrier hasn't completed, wait.
- **Never queue a cohort with a non-empty YAML overlap intersection** without first degrading to serial. Per the YAML overlap check above (Cohort dispatch sequence step 4). Mechanical: `intersect(union(creates, modifies)_taskA, union(creates, modifies)_taskB) != ∅` → degrade. Trust the YAML over the `[P]` marker when they disagree.

## Cohort handover aggregation

Per `feedback_handover_trailer_cohort_propagation.md`. Once all cohort members reach `complete` (and under Shape G, all corresponding `validate-pending` DQ entries mutated to `result: "pass"` for both Phase 1 workspace and Phase 2 e2e per the option-b two-phase validation), the advisor populates the *next* cohort's brief §3a "Handover from prior cohort" before queueing any task in that next cohort.

Sequence:

1. For each cohort member's commit on `phase-<phase>`, parse the commit body for the `HANDOVER:` YAML trailer (per `.claude/agents/impl-task.md` "Per-task commit shape"). Use `git log -1 --format=%B <sha>` and a YAML parser. If a cohort member's commit has no trailer, that's a no-op for the trailer (single-task non-`[P]` exception, or the impl-task subagent skipped it — note in polling output but do not catch-fire; missing trailer is degraded handover, not failure).
2. Aggregate the parsed trailers into one block matching the §3a schema in `impl-task-brief.template.md`:

```yaml
prior_cohort_tasks:
  - task: <N>
    commit: <sha>
    filesCreated: [...]
    filesModified: [...]
    keyDecisions: [...]
    notes: <verbatim from trailer>
  - task: <N+1>
    ...
```

3. Locate the next cohort's brief paths (one per cohort task — `.claude/PRPs/briefs/<phase>-impl-<M>.md`). For each, Edit §3a in place, replacing `(none — first cohort)` or `(none — prior task non-[P])` with the aggregated block. The advisor commits the brief edits with subject `chore(advisor): inject prior-cohort handover for <next-cohort-tasks>` (matches `^(chore|docs)\((advisor|decision-queue)\)` per attribution-integrity).
4. Push the brief commits to `governance-v0` so Junior worktrees pick them up cleanly.
5. Proceed to next-cohort dispatch (Cohort dispatch sequence step 1, restarting with the next pending §13 task).

**Skip the aggregation step if** the next cohort is empty (i.e. the prior cohort was the last cohort before the retro task). The retro task reads §3a as `(none — last cohort)`.

**Single-task cohorts** (where one §13 task with `[P]` is followed by a non-`[P]` task and the cohort collapses to just that one `[P]` task) still aggregate handover — the next non-`[P]` task gains the prior `[P]` task's keyDecisions. The trailer is the unit of handover; cohort size doesn't change the rule.

## Plans without `[P]` markers (back-compat)

If a plan §13 has no `[P]` annotations (legacy plans pre-this-rule, or plans where the planner judged no parallelism was safe), every task is treated as non-`[P]` and dispatched serially. The cohort-dispatch logic does not broaden serial dispatch into accidental parallel — `[P]` must be explicit.

## Cohort dispatch under Shape G (parallel validate-pending)

Under Shape G (v1-JM-e onward), cohort members each enter
`kind: "validate-pending"` simultaneously after their respective
push — one Phase-1 workspace-check workflow run per cohort task,
fanned out on GitHub-hosted runners. The advisor dispatches one
`[role:ci-watcher]` Junior task per `validate-pending` entry. Each
ci-watcher mutates its paired entry on completion (per option 2;
the entry's `kind` stays `"validate-pending"`, but `result`/
`log_slice`/`failed_jobs`/`answer`/`answered_by`/`resolved_at` are
populated, and the entry moves `pending[]` → `resolved[]` only on
`result: "pass"`).

Cohort advancement waits for **all** Phase-1 cohort members to reach
`result: "pass"` (entries in `resolved[]`). A single member with
`result: "fail" | "cancelled" | "timed_out"` (entry remaining in
`pending[]`) blocks advancement and triggers the §G4 classifier
per the validate-stage Stage-shape rule. If multiple cohort members
fail simultaneously, classify each independently — auto-queue
allowlist matches as parallel fix-impl-tasks (each forming its own
[P]-marker degenerate cohort), surface non-allowlist failures to
user as a single catch-fire bundle.

After all Phase-1 cohort members pass and the daemon finalize-merges
each into the phase branch, the advisor raises a SINGLE Phase-2 e2e
`validate-pending` DQ entry for the post-finalize phase-branch tip
(one e2e run per cohort barrier, not per cohort member — option (b)
e2e fires once when the phase-branch tip moves). Cohort advancement
to the *next* cohort waits on this e2e ci-watcher resolving with
`result: "pass"` as well.
