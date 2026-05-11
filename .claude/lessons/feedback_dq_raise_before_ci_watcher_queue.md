---
name: atomic DQ raise before ci-watcher dispatch
description: Commit + push the validate-pending DQ entry BEFORE queueing the ci-watcher Junior task. Reverse order causes contract-violation timing race.
type: feedback
---

# Atomic DQ raise before ci-watcher dispatch

The `validate-pending` DQ entry MUST be committed AND pushed BEFORE the `[role:ci-watcher]` Junior task is created. Reverse order causes a timing race: the ci-watcher worker branch forks from the current `phase-<phase>` tip at task-creation time, and if the DQ raise hasn't reached the daemon's view of origin yet, the ci-watcher's brief will not find a paired entry to mutate — and correctly files a `kind: "blocker"` contract-violation per its hard refusal rules.

**Why:** Per retro evidence at `.claude/PRPs/reports/v1-RT-r1-halt-retro.md` (commit `ffa2876e3`). v1-RT-r1 ci-watcher #219 was queued at 2026-05-10T21:35:41Z; the advisor's DQ #200+#201 raise was committed locally before that but the **push to origin landed ~10 min later** (~21:45Z). Junior daemon forked the ci-watcher worker branch from origin/phase-v1-RT-r1 at 21:35:42Z — at that moment, origin still showed the pre-raise tip. ci-watcher's brief read pointed at workflow runs `25640324087` + `25640324089` but its DQ scan found NO paired `validate-pending` entries with those `workflow_run_id`s. Per `.claude/agents/ci-watcher.md` Hard refusal #5 ("never write a NEW validate-pending entry"), the ci-watcher correctly filed `kind: "blocker"` DQ #197 — "dispatch contract violated" — and exited. Required advisor inline-mutation to recover (DQ #202 supersession entry + workflow-success-from-gh-run-view evidence). Cost: ~10 min unwanted ci-watcher cycle + extra audit-trail commit.

**How to apply:** The atomic dispatch sequence is:

```bash
# (a) Write the validate-pending DQ entry to .claude/decision-queue.json on phase-<phase>
git add .claude/decision-queue.json

# (b) Commit with the canonical subject pattern
git commit -m "chore(decision-queue): advisor raised DQ #<id> + #<id+1> — <slug>"

# (c) PUSH FIRST — block until origin reflects the raise
git push origin <phase-branch>

# (d) ONLY NOW queue the ci-watcher Junior task
mcp__junior-brehon__create_task base_branch=<phase-branch> description="[role:ci-watcher] <slug> — see <brief-path>"
```

NEVER reverse step (c) and step (d). NEVER queue the Junior task with an unpushed local DQ raise. The push is the synchronization barrier between the advisor's view and Junior's worker-branch fork point.

**Edge cases:**

- impl-task subagents that raise validate-pending DQs from THEIR worker branch (mid-task push per `decision-queue.md` "Mid-task visibility") followed by ci-watcher dispatch FROM ADVISOR don't have this problem — Junior's mid-task push lands on the worker branch (which ci-watcher's brief references by name + workflow_run_id), and Junior's finalize-merge brings it to phase-branch tip before the advisor's next poll cycle queues the ci-watcher.
- Advisor-raised DQs (Phase 2 e2e dispatch, fix-impl pre-dispatch validate-pending entries) ARE susceptible. These are the load-bearing case for this lesson.
- Forced re-validation (`gh workflow run`-triggered runs that aren't tied to a fresh impl-task push): advisor writes the validate-pending entry inline on `phase-<phase>`, pushes, THEN queues ci-watcher. Same ordering.

**Detection (retro):** a `chore(decision-queue): impl/ci-watcher filed DQ #<N> — dispatch contract violation` commit on a worker branch is evidence of this timing race. Retro flags it. The fix is process-level (advisor must follow the atomic ordering), not in Junior — Junior correctly refused the contract violation.

**Companion lessons:**
- `feedback_ci_watcher_serial_per_task_pair.md` — sibling lesson on ci-watcher dispatch ordering.
- `feedback_check_git_before_junior_queue.md` — general pre-queue git-state check.
- `decision-queue.md` "Mid-task visibility" — broader mid-task push discipline.

**Where codified:** `.claude/rules/advisor-orchestrator.md` §3.1 "Phase 1 (workspace-check on `junior/*`)" paragraph + "Atomic raise-before-dispatch rule".
