---
name: Preserve active worktree — never checkout-switch when pending work exists
description: When the primary worktree has uncommitted phase work, advance other branches via git update-ref or auxiliary worktrees; never checkout-switch (which aborts on dirty tree, and -f would destroy work).
type: feedback
originSessionId: 50ffed48-0dd6-4010-8ffb-d915ca13314c
---
When doing git housekeeping on the Brehon fork, the primary worktree is almost always **actively hosting in-progress phase work** (staged edits, modified scripts, scratch files). Switching branches there is unsafe:

- `git checkout <other-branch>` aborts if the working tree has modifications overlapping with the target branch — correct behaviour, but blocks progress.
- `git checkout -f <other-branch>` (or `git switch -f`) would succeed but **destroy** the uncommitted work.
- `git stash` is acceptable but fragile (stash scope can be hard to reason about when mixing staged/unstaged, and dropped stashes leak work).

**Why:** on 2026-04-18 the user explicitly thanked me for leaving the phase-5c worktree alone during a branch-cleanup session that needed to advance `governance-v0` by 1 commit and open a PR from `governance-v0`. The correct moves were:
1. `git fetch origin governance-v0` + `git update-ref refs/heads/governance-v0 origin/governance-v0` — advances the local branch ref without touching the working tree. Only safe when it's a confirmed fast-forward (`git merge-base --is-ancestor` returns true).
2. For the PR branch, `git worktree add ../brehon-fork-<topic> -b <new-branch> governance-v0` creates an isolated second checkout; cherry-pick / commit / push from there; `git worktree remove` when done.

**How to apply:** in branch-housekeeping sessions, do not issue `git checkout <other-branch>` from the primary worktree if `git status --short` shows anything. Plan the session entirely around `update-ref` (for fast-forwards) and auxiliary worktrees (for anything needing a working-tree checkout). Ask before resorting to `git stash` — it's acceptable but not the first tool.

This is the same pattern that earlier landed the plan-drift CI fix (2026-04-18, commit `e30f93c8b`): the primary worktree was on phase-5c with pending edits, the fix needed to go to governance-v0, so I created `../brehon-fork-govv0` as an isolated worktree, did the work there, pushed, then removed the worktree. The primary worktree was never disturbed. Same pattern used 2026-04-18 for PR #9 (`../brehon-fork-claude-workflows`).
