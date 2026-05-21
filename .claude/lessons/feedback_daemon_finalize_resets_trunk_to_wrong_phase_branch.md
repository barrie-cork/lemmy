# feedback: daemon finalize step hard-resets governance-v0 to a non-trunk phase branch's tip

## TL;DR

The Junior daemon's finalize-merge step on a planning task (#399 — v1-dq-schema-r1) successfully merged the worker branch into daemon-local `governance-v0` as merge commit `2ad835aa4`, then **immediately hard-reset `governance-v0` to `origin/phase-v1-federation-inbound-c`** in the same finalize cycle. The plan-merge commit became unreachable from daemon-local refs; origin was never pushed (the reset happened before push). Recovery required cherry-pick + recovery-branch push from the daemon's reflog. **Status: ROOT CAUSE NOT YET INVESTIGATED — DQ #338 pending (2026-05-21).**

## Why this mattered

The destructive sequence (daemon reflog):

```
@{1}  merge: 2ad835aa4 — daemon merged worker-399 into local governance-v0 (CORRECT)
@{0}  reset: moving to origin/phase-v1-federation-inbound-c (WRONG REF — destructive)
```

Origin/governance-v0 was protected only because the reset happened before push. If the daemon's finalize order were `merge → push → reset`, origin/governance-v0 would have been silently force-pushed to the phase branch's tip, taking the public history with it.

The bug class is: the daemon's finalize step's branch-restoration sub-step (after merge, the daemon presumably checks out trunk to push) picked the wrong ref. The active phase branch `phase-v1-federation-inbound-c` exists on the daemon (cohort 1 dispatched 2026-05-21); finalize's logic appears to read a stale "current branch" pointer or a misconfigured worktree state and reset trunk to that ref instead of to `origin/governance-v0`.

## When to apply

This lesson and the recovery recipe apply whenever:

1. A planning Junior task (`[role:planning]`) finalizes while another `phase-v1-*` branch is concurrently active on the daemon (cohort dispatched, worker branches in `.junior/worktrees/`).
2. The advisor observes the post-finalize `governance-v0` tip on origin is unchanged but the daemon's local `governance-v0` is sitting at a phase branch's tip.
3. `mcp__junior-brehon__show_task` reports `succeeded` for the planning task but `git log origin/governance-v0 --grep '<plan>'` returns empty.

Probability of recurrence: high. Every planning task dispatched while ANY `phase-v1-*` branch is active reproduces the trigger conditions.

## Recovery recipe (verified 2026-05-21 on task #399)

Three-step recovery, ~25 min total:

### Step 1: SSH to EliteDesk daemon and cherry-pick from reflog

```bash
ssh homeserver "cd /srv/brehon-fork && \
    git reflog governance-v0 | head -5 && \
    git cherry-pick <orphaned-merge-sha-from-reflog>~1..<orphaned-merge-sha-from-reflog>"
```

Replace `<orphaned-merge-sha-from-reflog>` with the merge commit visible in the daemon reflog @{1} entry. The cherry-pick produces a new commit (`c02dc8617` in the 2026-05-21 incident) with the same tree as the original plan write.

### Step 2: Push to a recovery branch (NOT trunk)

```bash
ssh homeserver "cd /srv/brehon-fork && \
    git push origin HEAD:refs/heads/recovery/<phase>-plan"
```

Never `git push origin governance-v0` from the daemon side post-reset — at this point daemon-local `governance-v0` is at the wrong ref (the phase branch tip); pushing it would clobber origin/governance-v0.

### Step 3: Pull-cherry-pick on laptop

```bash
git fetch origin recovery/<phase>-plan
git cherry-pick origin/recovery/<phase>-plan
git push origin governance-v0
git push origin :recovery/<phase>-plan  # cleanup
```

The laptop's cherry-pick produces a third SHA (`f746a00b9` in 2026-05-21 incident) but the tree is byte-identical to the plan write. Origin/governance-v0 now has the plan commit; recovery branch is deleted.

### Step 4: Daemon-side trunk re-alignment

After origin/governance-v0 is current, force the daemon's local `governance-v0` back into alignment:

```bash
ssh homeserver "cd /srv/brehon-fork && \
    git fetch origin governance-v0 && \
    git update-ref refs/heads/governance-v0 origin/governance-v0"
```

Use `update-ref` rather than `checkout` + `reset --hard` so the daemon's current worktree state (which may be on a phase branch for an active cohort worker) is not disturbed.

## Investigation pointers (for DQ #338 option-a resolution)

- **Primary suspect:** `/opt/junior-src/src/daemon/executor.ts` finalize-merge code path. Look for any `git reset --hard <ref>` or `git checkout -B <branch> <ref>` step that uses a `<ref>` derived from worktree state rather than the task's `base_branch` field.
- **Secondary suspect:** `/opt/junior-src/src/daemon/worktree.ts` — if the daemon reuses a worktree directory across tasks, finalize may read a stale `HEAD` from a prior cohort worker's worktree.
- **Symptom signature for repro:** dispatch a `[role:planning]` task with `base_branch=governance-v0` while at least one `phase-v1-*` branch is active in the daemon's worktree list. Watch the daemon's `governance-v0` reflog post-finalize.

A safer finalize sequence would be:

```bash
# After merge:
git checkout governance-v0          # NOT reset; preserves any uncommitted state
git merge --ff-only <worker-branch> # FF; aborts if non-FF (which signals the bug)
git push origin governance-v0       # push only on confirmed FF
```

The `--ff-only` flag converts the silent destructive-reset bug into a loud merge-aborts-with-error bug.

## What NOT to do

1. **Never `git push origin governance-v0` from the daemon side after observing the reset.** Daemon-local `governance-v0` is at the wrong ref; pushing it overwrites the public history.
2. **Never `git reset --hard origin/governance-v0` on the daemon-local trunk** while a phase worktree may be in-flight — this can wedge an active cohort worker if the worker reads daemon-local trunk for ff-merge purposes (per `feedback_daemon_local_trunk_stale_multi_lane.md`).
3. **Never restart the daemon as a "fix"** — the daemon's reflog is the only source of truth for the orphaned commit; restart may garbage-collect unreachable refs (default 90d but configurable).

## Companion DQ

- **DQ #338** (2026-05-21, advisor) — root-cause investigation; option-a (structural fix in `executor.ts`) recommended over option-b (track-and-watch). Recurrence cost ~25 min × N planning tasks while phase branches are active.

## See also

- `feedback_daemon_refspec_excludes_meta_phase_branches.md` — adjacent daemon-config defect class (refspec filter), demonstrates the live-edit-then-patch-restore pattern that DQ #338's eventual fix should follow.
- `feedback_daemon_local_trunk_stale_multi_lane.md` — why daemon-local trunk vs origin trunk diverge under multi-lane operation; relevant to recovery Step 4's `update-ref` choice.
- `feedback_junior_finalize_skips_when_worker_pre_pushes.md` — adjacent finalize-step failure mode; in that case finalize is too passive, in this case finalize is too aggressive (overreaches into trunk-reset).
- `feedback_junior_finalize_merge_race_lossless_reconcile.md` — BOTH-RAN reconcile pattern; complementary defense in depth.
