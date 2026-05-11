---
name: ci-watcher dispatch must be serial per task pair
description: Parallel ci-watcher dispatch off same phase tip causes merge-time resurrection of resolved DQ entries. Dispatch ONE ci-watcher per logical task, sequentially.
type: feedback
---

# ci-watcher dispatch must be serial per task pair

When a cohort has N members each with paired `validate-pending` DQ entries (e.g. workspace-check + migration-round-trip = 2 workflows per task), **do NOT dispatch N parallel ci-watchers**. Dispatch **ONE ci-watcher per logical task**, sequentially, each long-polling 1-2 workflow runs and mutating the paired DQ entries in place.

**Why:** Per retro evidence at `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` (commit `ffa2876e3`). v1-RT-r1 Cohort A dispatched 4 parallel ci-watchers (#215, #216, #217, #218) — one per task — each polling 2 workflow runs. All 4 ci-watcher worker branches forked off the SAME phase-branch tip simultaneously. When the daemon's finalize-merge ran them in sequence, ci-watchers 4 and 3 (later finalize-merges) had stale pre-mutation `.claude/decision-queue.json` snapshots. Git's auto-merge resolution between each ci-watcher's branch and the phase tip RESURRECTED already-resolved DQ entries (#189-#192 + #195-#196) from `result: "pass"` back to `result: null` in `pending[]`. Only ci-watcher #217's mutation of DQ #194 (the Task 3 fail) propagated through; the 6 pass-mutations were undone. Required manual advisor inline-fix via Python-script + commit on phase branch to restore correct state. Total cost: ~30 min unwanted work + audit-trail noise + a separate "resurrection recovery" commit that pollutes retro grain.

**How to apply:** When the advisor reaches the "ci-watcher complete" stage and has multiple validate-pending entries (one per cohort task), author **one ci-watcher brief per task** that lists its 1-2 paired workflow_run_ids. Queue ci-watchers SEQUENTIALLY — wait for ci-watcher #1 to finalize-merge (daemon merges its mutation into phase branch) BEFORE queueing ci-watcher #2. Each ci-watcher's worker branch then forks off the updated phase tip (with #1's mutation already merged), so there's no merge-time conflict to auto-resolve.

**Per-ci-watcher brief shape:**

```markdown
# ci-watcher brief — <phase> task <N> workflow runs <wid-a> + <wid-b>

**Workflow run ids (mutate BOTH — paired):**
- `<wid-a>` (cargo-validate-workspace) → mutate DQ #<id-a> (in pending[])
- `<wid-b>` (cargo-validate-migration)  → mutate DQ #<id-b> (in pending[])

(Same per-workflow mutation protocol as sl-d-ci-watcher-1.md — long-poll, classify, mutate by workflow_run_id match. Polls 2 workflows in sequence inside this ci-watcher. Commit each mutation separately.)
```

**Edge cases:**

- A cohort with 4 tasks × 2 workflows each = 8 paired DQs. Serial dispatch = 4 ci-watcher Juniors, each polling 2 workflows. Total wallclock ≈ 4 × (workflow-runtime + finalize-merge-latency). For workflows that already completed pre-dispatch, ci-watcher long-poll returns immediately and the wallclock is dominated by finalize-merge latency (~30s-2min).
- Single-task cohorts (1 task, 1-2 workflows) — only 1 ci-watcher, no race possible. Use the original per-workflow protocol directly.
- Workflow re-validations (re-runs after fix-impl) — ALWAYS single ci-watcher, never parallel.

**Detection (retro):** a `chore(decision-queue):` commit on the phase branch that re-mutates an entry from `pending` to `resolved` AFTER it was already resolved earlier in the git log is evidence of resurrection. Retro flags it. The cost is mechanical re-mutation; the worse cost is audit-trail noise.

**Companion lessons:**
- `feedback_parallel_agents_one_worktree_per_agent.md` — the broader "one worktree per agent" principle.
- `feedback_dq_raise_before_ci_watcher_queue.md` — sibling lesson on atomic raise-before-dispatch ordering.
- `feedback_cohort_validation_dependency_check.md` — sibling lesson on `[P]` semantics gap.

**Where codified:** `.claude/rules/advisor-orchestrator.md` §3.1 "ci-watcher complete" branch + Phase 1 paragraph.
