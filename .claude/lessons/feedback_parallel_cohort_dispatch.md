---
name: Parallel cohort dispatch via §13 [P] markers (spec-kit pattern adoption)
description: Annotate plan §13 tasks with `[P]` markers when their IMPLEMENT files are disjoint within a cohort. The advisor groups consecutive `[P]` tasks and queues them simultaneously to Junior, each on its own worktree. Non-`[P]` tasks are barriers (queue alone). Budget check degrades a cohort to serial when cumulative cargo memory exceeds the EliteDesk cap.
type: feedback
---

In `.claude/PRPs/templates/plan.template.md` §13 (Step-by-step tasks), each task header may carry a `[P]` marker — `### Task N [P]: <title>` — when the task's IMPLEMENT files are disjoint from every other `[P]`-marked task in the same cohort. The advisor's stage-shape orchestration computes the cohort (consecutive `[P]` tasks from the next pending forward) and dispatches the cohort simultaneously, each task on its own Junior worktree.

**Why:** Brehon plans go from "phase scope" straight to file-level tasks without indicating which can fan out to parallel worktrees. `feedback_parallel_agents_one_worktree_per_agent.md` already wants per-agent worktrees; making `[P]` mechanical in §13 makes cohort dispatch a deterministic decision rather than a per-phase advisor judgment call. Spec-kit's `tasks.md` annotates the same way. The Brehon adaptation gates parallelism on a budget check (per `feedback_resource_budget_pre_queue.md` — EliteDesk's `MemoryMax=10G` cap means 2 simultaneous `cargo check --workspace --features full` is the practical max).

**How to apply (planner side — see `.claude/agents/planning.md`):**

- Walk §13 in task order. Build the file-set for each task by reading its `**IMPLEMENT (file N of M):** in <file>` lines.
- Two tasks are cohort-compatible if their file-sets share zero paths.
- Task 0 (pre-flight harness) is always non-`[P]` (verification barrier).
- Retro task is always non-`[P]` (depends on every prior commit).
- Migrations: the up/down pair of one migration is one logical unit; two different migrations are cohort-compatible.
- Mark `[P]` only when disjoint-files holds. Conservative is correct — missing `[P]` means serial dispatch (slower but safe); incorrect `[P]` causes worktree merge conflicts (broken).
- Plans without parallelisable tasks omit `[P]` entirely. Don't sprinkle markers prophylactically.

**How to apply (advisor side — see `.claude/rules/advisor-orchestrator.md` "Cohort dispatch"):**

- After bm-cut completes, walk §13 from Task 1.
- If the next pending task is non-`[P]`: queue alone, wait for complete/failed.
- If the next pending task is `[P]`: collect consecutive `[P]` tasks until a non-`[P]` boundary. That's the cohort.
- **Budget check:** estimate `cohort_size × per_task_peak`. Each `cargo check --workspace --features full` ≈ 6 GB peak. Above 10 GB total, degrade to serial and log the degrade in polling output.
- **Forbidden-window check:** re-evaluate for the cohort's expected start. If any cohort task would start in a forbidden window, defer the entire cohort.
- Queue every cohort task simultaneously via parallel `mcp__junior-brehon__create_task` calls (single message, multiple tool uses). Each task gets its own Junior worktree.
- Wait for **all cohort members** to reach complete/failed before computing next cohort.

**Refusals:**

- Never queue a `[P]` task whose IMPLEMENT files overlap a non-`[P]` task that's still running (`[P]` is a within-cohort promise, not cross-cohort).
- Never re-queue a cohort task that already shows `running`.
- Never queue partial cohorts during forbidden windows — defer all or none.

**Symptom to recognise in retrospect:** a phase that took N×task-duration to ship when several tasks were file-disjoint and could have run in parallel. If the retro shows wall-clock dominated by serial dispatch and the plan §13 has no `[P]` markers, the planner missed the parallelism opportunity. Re-audit at retro and update the planning agent contract if a class of plans keeps missing `[P]`.

**Generalizes to:** any plan-driven workflow with multiple file-disjoint tasks. Spec-kit's `[P]` is the same primitive. The Brehon-specific addition is the budget check — without it, parallel cohorts can OOM-cascade the box (per `project_elitedesk_hung_2026_04_27.md`).
