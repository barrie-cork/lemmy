---
name: Daemon has no local branch for a newly-cut Mode-A lane
description: A phase branch cut in a Mode-A lane worktree leaves the Junior daemon clone with no local branch; the first task with base_branch=phase-v1-<lane> fails with "fatal: invalid reference". Create the daemon-local branch from the fetched origin ref before dispatching the first task.
type: feedback
---

# Daemon has NO local branch for a newly-cut Mode-A lane → first Junior task fails `invalid reference`

**Rule:** When a phase branch is cut in a **Mode-A lane worktree** (`git checkout -b phase-v1-<lane>` + `git push origin` from `C:/Users/barri/Developer/brehon-fork-<lane>`), the Junior daemon's separate clone at `/srv/brehon-fork` gets **no local branch** for it. The first `[role:*]` task dispatched with `base_branch=phase-v1-<lane>` fails instantly at `git worktree add ... phase-v1-<lane>` with `fatal: invalid reference: phase-v1-<lane>`. Before dispatching the first Junior task on any freshly-cut lane, create the daemon-local branch from the fetched origin ref.

## Why this is distinct from the adjacent lessons

This is NOT the "daemon-local ref lags origin" family — the ref does not exist **at all**:

- `feedback_daemon_local_trunk_stale_multi_lane.md` / `feedback_daemon_stale_bm_verb_brief_miss.md` — daemon-local branch EXISTS but its tip is STALE. Fix = FF (`git fetch origin <b>:<b>`).
- `feedback_cross_lane_daemon_ref_contamination.md` — daemon-local branch EXISTS but points at the WRONG lane's commits. Fix = detach + `git branch -f`.
- `feedback_junior_292_stale_base_recover_recipe.md` — workers fork from a STALE-but-existing daemon-local ref.

All three assume the daemon-local branch already exists. **Here it never did.** A Mode-A lane cut happens entirely on the laptop (bm-cut runs in the lane worktree, pushes to origin); the daemon clone is never told about the branch until something fetches it AND a local branch is created. `git fetch origin` alone is insufficient — it creates only `refs/remotes/origin/phase-v1-<lane>`; the worker's `git worktree add ... -b junior/... phase-v1-<lane>` uses the BARE name as the start-point, which does NOT DWIM-resolve to the remote-tracking ref on this daemon (`fatal: ambiguous argument 'phase-v1-<lane>'`).

## Confirmed

v1-RT-r5, 2026-05-31. bm-cut ran in lane worktree `brehon-fork-rt-r5` (`560eda9b1`), pushed `phase-v1-RT-r5` to origin. Planning task #544 dispatched with `base_branch=phase-v1-RT-r5` → failed in <1s: `git worktree add /srv/brehon-fork/.junior/worktrees/job-544 -b junior/... phase-v1-RT-r5 failed: fatal: invalid reference: phase-v1-RT-r5`. Daemon had zero refs matching `*RT-r5*` (no local branch, no remote-tracking ref). Re-dispatched #545 after the fix below → ran cleanly.

## Pre-dispatch check (mandatory on the FIRST Junior task of any freshly-cut lane)

```bash
ssh homeserver "cd /srv/brehon-fork && git rev-parse phase-v1-<lane> 2>&1"
# If this prints 'fatal: ... unknown revision' or 'invalid reference' → apply the fix below
# before create_task. If it prints a SHA → daemon-local branch exists; proceed.
```

## Fix (lane-safe — no daemon HEAD switch)

```bash
ssh homeserver "cd /srv/brehon-fork && \
  git fetch origin && \
  git branch phase-v1-<lane> origin/phase-v1-<lane>"
# Verify:
ssh homeserver "cd /srv/brehon-fork && git rev-parse phase-v1-<lane>"   # must = your pushed tip
```

`git branch <name> origin/<name>` creates the local branch as a worktree start-point WITHOUT checking it out — the daemon's HEAD stays on whatever other lane it was on (in the RT-r5 case, `phase-v1-quality-r3b`). The worker's per-job worktree forks from the new local branch; daemon HEAD is irrelevant to it. After this, re-dispatch the task (the failed task is dead — create a fresh one; do not retry the failed id).

## Why bm-cut doesn't do this

bm-cut (Mode A) runs entirely in the lane worktree on the laptop — it has no reason to touch the daemon. The gap only manifests when the FIRST daemon-side Junior task tries to fork from the new branch. Modes differ:
- **Mode A (this lesson):** branch cut on laptop lane worktree → daemon never gets a local branch → first task fails.
- **Mode B:** the trunk→phase SSH-merge (per `multi-lane-worktree.md` §"Brief location and trunk→phase sync") runs ON the daemon's main worktree, which is already on the phase branch post-bm-cut — so the daemon-local branch exists by construction. Mode B does not hit this.

## Structural fix (unshipped — daemon-side)

The real cure is the same one tracked broadly under DQ #338 + `feedback_junior_292_stale_base_recover_recipe.md` "Upstream fix": the daemon should run `git fetch origin <base_branch>` and ensure a local branch exists (`git branch -f <base_branch> origin/<base_branch>` or fork the worktree directly from `origin/<base_branch>`) BEFORE `git worktree add`. That single change subsumes this lesson AND the three stale-ref lessons above — they are all "daemon-local ref not current/present for the requested base_branch." Until it lands, the pre-dispatch check above is mandatory for the first task of every Mode-A lane.

## See also

- `.claude/rules/multi-lane-worktree.md` §"Daemon side (EliteDesk)" + §"Lane modes"
- `feedback_daemon_local_trunk_stale_multi_lane.md` — stale-tip variant (ref exists)
- `feedback_cross_lane_daemon_ref_contamination.md` — wrong-lane variant (ref exists)
- `feedback_junior_292_stale_base_recover_recipe.md` — stale-base worker recovery + the shared upstream fix
- CLAUDE.md "Pre-queue git pre-flight (mandatory)" — the `/precheck` discipline this extends to the daemon-local-branch-existence case
