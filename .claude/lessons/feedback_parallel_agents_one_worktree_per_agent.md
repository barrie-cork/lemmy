---
name: Parallel Agents race in a shared worktree
description: Spawning N parallel Agents in one git worktree causes staging/reset/reflog races even with per-file scope; give each agent its own worktree
type: feedback
originSessionId: d92bc0e2-7ffc-463f-9a0b-2a726d8a50d6
---
When launching multiple parallel `Agent` tool calls that edit files in the same git worktree, the agents race on git index operations (`git add`, `git commit`, `git reset`) even when their file scopes are disjoint. Observed on 2026-04-17 with 4 parallel general-purpose agents fixing CodeRabbit findings across 4 different files in `brehon-fork/.claude/worktrees/ops-pr2-followup`:

- Two agents swept sibling agents' staged changes into their own commits and had to `git reset --soft HEAD~1` to unwind
- One agent's commit (oq-sweep) was orphaned entirely by a later agent's reset and had to be recovered from reflog via cherry-pick
- All 4 flagged the race themselves in their reports (because the briefs told them to)

**Why:** disjoint file-scope doesn't help because the git index is a single-writer structure. `git add <narrow>` followed by `git commit` can still capture whatever else was staged in the window between the two calls.

**How to apply:** when spawning 2+ parallel Agents that will each commit, create a separate worktree per agent up-front. Either:
- `git worktree add .claude/worktrees/agent-<role>-<branch>` per agent, and have each agent cherry-pick / rebase onto a shared collector branch at the end
- or force agents to produce diffs only (not commits) and have the parent session serialise commits

Second-best: tell the agents explicitly to use `git add <specific-path>` + `git diff --cached --name-only` verification + `git commit -- <specific-path>` (pathspec on commit), AND have them hold a file-based mutex during the add→commit window. The briefs in the 2026-04-17 run already told agents to add narrowly; the race still happened because the commit call captures everything staged at the moment it fires.

If using a single shared worktree despite this, instruct agents to flag scope violations (they did, and it saved the run — reflog cherry-pick recovered the orphaned commit).
