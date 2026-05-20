# feedback: briefs land on the branch the worker forks from

## TL;DR

A Junior task's brief MUST be committed on the branch the worker will fork from BEFORE the task is dispatched. Workers `Read` the brief from their worktree, which is rooted at `base_branch`. A brief committed on `governance-v0` is INVISIBLE to a worker that forks from `phase-<X>` unless first cherry-picked or merged forward.

## Why

advisor-orchestrator.md §2.1 historically said "committed on `governance-v0` before the task is created" — wording that worked for planning + bm-cut briefs (which precede any phase branch existing) but quietly broke for impl-task briefs, which workers read from the **phase branch worktree**.

Concrete failure (brehon-conformance-audit, 2026-05-20):

- I authored `.claude/PRPs/briefs/brehon-conformance-audit-impl-1.md` on `governance-v0` at `9b74c0909`.
- Junior task #355 was queued with `base_branch=phase-brehon-conformance-audit` (at `0935c3c92`).
- The phase branch did NOT have the brief — its tree had bm-cut-brief + plan + nothing else.
- Cost: cherry-pick `9b74c0909` → `aca7f4149` on phase branch, then forward-merge trunk → phase branch at `8f8fbd9db` to bring runlog + regex widening + DQ #295 onto the phase branch. ~10 min wallclock recovery.

Fed-in-b (the canonical precedent) authored all impl-task briefs directly on the phase branch via commit `0ea7ab4f7 chore(advisor): author v1-federation-inbound-b Cohort A impl briefs (Tasks 1/2/3)`. The PR merge brought them onto `governance-v0` at sub-phase ship time.

## When to apply

| Brief class | Author on | Visibility to worker |
|---|---|---|
| Planning | `governance-v0` | Planner forks from `governance-v0`; brief visible. |
| bm-cut | `governance-v0` | BM worker forks from `governance-v0`; brief visible. |
| **Impl-task** | **`phase-<X>` directly** | **Impl worker forks from `phase-<X>`; brief MUST be on that branch's tree.** |
| bm-pr / bm-merge | `governance-v0` | BM worker reads from trunk. |
| ci-watcher | `governance-v0` | ci-watcher polls workflow_run_id; brief just names IDs (location flexible). |

## How to apply

When the advisor session needs to dispatch an impl-task:

1. **`git checkout phase-<X>` first** (if not already there).
2. **Author the brief at `.claude/PRPs/briefs/<phase>-impl-<n>.md`** on the phase branch.
3. **`git commit + git push origin phase-<X>`.**
4. **Then `mcp__junior-brehon__create_task` with `base_branch=phase-<X>`.**

OR (if the brief was authored on trunk by mistake — recovery):

1. SSH to daemon (or use git from advisor session). `git checkout phase-<X>`.
2. `git cherry-pick <trunk-brief-commit>` to bring the brief onto the phase branch.
3. Optionally `git merge --no-ff governance-v0` to also bring other trunk advances (DQ updates, lessons) onto the phase branch.
4. `git push origin phase-<X>`.
5. Then dispatch the Junior task.

The cherry-pick + forward-merge recovery costs ~10 min; authoring directly on the phase branch costs 0.

## Cross-references

- `advisor-orchestrator.md` §2.1 (revised 2026-05-20).
- `feedback_check_git_before_junior_queue.md` — sibling discipline for pre-dispatch checks.
- Session retro: `.claude/PRPs/reports/session-retro-2026-05-20-brehon-conformance-audit-bootstrap.md` §2.8.
- Fed-in-b precedent: commit `0ea7ab4f7`.
- Brehon-conformance-audit recovery: `aca7f4149` (cherry-pick) + `8f8fbd9db` (forward-merge).
