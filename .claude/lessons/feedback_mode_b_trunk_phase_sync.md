# Mode B: trunk→phase sync for impl-task brief visibility

## Problem

In Mode B (mobile remote-control), the advisor drives a lane from the canonical `brehon-fork` checkout without a dedicated lane worktree. impl-task briefs are authored and committed on `governance-v0`. But Junior workers fork from `phase-v1-<lane>` — they cannot read a brief that only exists on `governance-v0`.

**Trigger:** Mode B lane + impl-task brief just committed to `governance-v0` → MUST sync to phase branch before `create_task`.

## Solution: 3-step SSH merge

The daemon's main worktree is already on `phase-v1-<lane>` post-bm-cut. Use it:

```bash
# Step 1 (laptop): author brief on governance-v0, commit, push.
git add .claude/PRPs/briefs/<phase>-impl-<n>.md
git commit -m "chore(advisor): brief <phase> impl-task <n> — <slug>"
git push origin governance-v0

# Step 2 (daemon, via SSH): merge trunk into phase branch.
ssh homeserver "cd /srv/brehon-fork \
  && git fetch origin governance-v0 \
  && git merge origin/governance-v0 --no-edit -m 'Merge governance-v0 into <phase> — pull impl-<n> brief for Task <n> dispatch' \
  && git push origin phase-v1-<lane>"

# Step 3 (laptop): queue the Junior task.
# mcp__junior-brehon__create_task with base_branch=phase-v1-<lane>
```

**Precondition:** daemon worktree must be on the phase branch. Check: `ssh homeserver 'cd /srv/brehon-fork && git symbolic-ref HEAD'`. If not on the phase branch, `git checkout phase-v1-<lane>` on the daemon first.

## See also

`multi-lane-worktree.md §"Mode B — author on trunk, sync to phase via daemon SSH"` — full procedure with the alternative bm-task path for when the daemon worktree is unavailable.
