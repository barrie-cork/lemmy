# Mode B: trunk→phase sync for impl-task brief visibility

## Problem

In Mode B (mobile remote-control), the advisor drives a lane from the canonical `brehon-fork` checkout without a dedicated lane worktree. impl-task briefs are authored and committed on `governance-v0`. But Junior workers fork from `phase-v1-<lane>` — they cannot read a brief that only exists on `governance-v0`.

**Trigger:** Mode B lane + impl-task brief just committed to `governance-v0` → MUST sync to phase branch before `create_task`.

## Solution: surgical single-file checkout (preferred)

Use `git checkout origin/governance-v0 -- <file>` rather than a full merge. Full merges fail when the daemon-local phase branch has diverged from origin (Junior finalize-merge commits land on the daemon before origin receives them — this is the normal finalize flow, not a bug).

```bash
# Step 1 (laptop): author brief on governance-v0, commit, push.
git add .claude/PRPs/briefs/<phase>-impl-<n>.md
git commit -m "chore(advisor): brief <phase> impl-task <n> — <slug>"
git push origin governance-v0

# Step 2a: check if daemon phase branch is ahead of origin.
ssh homeserver "cd /srv/brehon-fork \
  && git fetch origin governance-v0 phase-v1-<lane> \
  && git log origin/phase-v1-<lane>..phase-v1-<lane> --oneline | wc -l"
# → 0: daemon is in sync with origin; proceed to 2b.
# → N>0: daemon is ahead (normal post-finalize state); proceed to 2b anyway —
#         surgical checkout is safe regardless.

# Step 2b (daemon, via SSH): surgical single-file pull from trunk.
ssh homeserver "cd /srv/brehon-fork \
  && git checkout phase-v1-<lane> \
  && git checkout origin/governance-v0 -- .claude/PRPs/briefs/<phase>-impl-<n>.md \
  && git add .claude/PRPs/briefs/<phase>-impl-<n>.md \
  && git commit -m 'chore(sync): Mode B pull impl-<n> brief from governance-v0' \
  && git push origin phase-v1-<lane>"

# Step 3 (laptop): queue the Junior task.
# mcp__junior-brehon__create_task with base_branch=phase-v1-<lane>
```

**Why surgical, not full merge:** `git merge origin/governance-v0` on a daemon-local branch that is ahead of `origin/phase-v1-<lane>` produces a non-fast-forward push rejection. The `--no-edit` merge works but the push fails because origin's tip is behind the daemon's tip. Surgical `git checkout origin/governance-v0 -- <file>` copies only the brief file, commits it on top of whatever the daemon's current tip is, and pushes — which succeeds because it's a fast-forward from origin's perspective only when daemon == origin. When daemon is ahead, `git push` still succeeds because it's advancing origin forward.

**Recurrence:** v1-quality-r3c T1 brief sync (2026-05-31 session 1) and T2 brief sync (2026-05-31 session 2) — both hit push rejection due to daemon-ahead state. Root cause: finalize-merge commits accumulate on the daemon between brief dispatches.

**Precondition:** daemon worktree must be on the phase branch before step 2b. Check:
`ssh homeserver 'cd /srv/brehon-fork && git symbolic-ref HEAD'`.

## See also

`multi-lane-worktree.md §"Mode B — author on trunk, sync to phase via daemon SSH"` — full procedure with the alternative bm-task path for when the daemon worktree is unavailable.
