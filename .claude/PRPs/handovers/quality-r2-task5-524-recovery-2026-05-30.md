# RECOVERY NOTE — quality-r2 task-5 (#524) orphaned by cross-lane cancel — 2026-05-30

**Audience:** the v1-quality-r2 advisor session (NOT this rt-r4 session — written here because rt-r4 caused the incident and committing on the rt-r4 phase branch is the only push surface available to the rt-r4 session; quality-r2 should copy this note into its own lane if useful).

## What happened

The **rt-r4 advisor session** (me) mis-tracked Junior task IDs during an unrelated fix-impl-2 dispatch and **erroneously issued `cancel_task` on #524**. #524 was NOT an rt-r4 task — it was **v1-quality-r2 task 5: "LEMMY_DATABASE_URL EnvVarGuard wrap 13 sites (closes #160)"**.

Timeline (UTC, 2026-05-30):
- 09:31:19 — #524 created (quality-r2 session).
- 09:42:15 — #524 run **SUCCEEDED** (commits written to its worker branch).
- 09:42:57 — rt-r4 session's errant `cancel_task #524` took effect → daemon reaped the worker branch ref `junior/role-impl-task-v1-quality-r2-task-5-...-524`.

Because success preceded the cancel, **the commits were written but the branch ref was reaped**. The daemon had NOT yet finalize-merged them to `origin/phase-v1-quality-r2`.

## Damage assessment: NOTHING LOST — work is safe on origin

**UPDATE: the worker branch was already pushed to origin BEFORE the cancel.** Verified:
```
4e5ec7748d298fe9415be4902f3bf3a6141202eb  refs/heads/junior/role-impl-task-v1-quality-r2-task-5-lemmy-database-url-envvarguard-wrap-13-sites-closes-160-see-claude-prps-br-524   (on ORIGIN)
```
The cancel only reaped the daemon-**local** ref + worktree; the origin branch is intact. **The simplest recovery is to fetch the worker branch from origin** — no special handling needed:
```bash
git fetch origin "junior/role-impl-task-v1-quality-r2-task-5-lemmy-database-url-envvarguard-wrap-13-sites-closes-160-see-claude-prps-br-524"
# → FETCH_HEAD = 4e5ec7748, the full task-5 chain
```

The rt-r4 session ALSO preserved a belt-and-suspenders recovery ref at `refs/recovery/quality-r2-task5-524-orphaned` → `4e5ec7748` on the daemon (redundant now that origin is confirmed intact). The commit objects survive (verified `git cat-file -t f0b24f577` = commit):

```
refs/recovery/quality-r2-task5-524-orphaned  →  4e5ec7748  (chore(decision-queue): impl raised DQ 7b0f9dbbb42a-001 — validate-pending-laptop task 5 EnvVarGuard wrap)
                                                 f0b24f577  (refactor(e2e): wrap 13 LEMMY_DATABASE_URL setter sites with EnvVarGuard (closes #160, task 5))
                                                 badd5622f  (Merge governance-v0 into phase-v1-quality-r2 — pull impl-5 brief for Task 5 dispatch (#160))
```

`badd5622f` is the quality-r2 brief-merge that was already the tip of `origin/phase-v1-quality-r2` at dispatch time — so `f0b24f577` + `4e5ec7748` are the only two new commits, and they cleanly descend from the existing phase tip.

## Recovery procedure (quality-r2 session runs this)

The rt-r4 session did NOT finalize this into quality-r2 (not its lane; finalize is the quality-r2 session's call). Options:

**Option A — restore the worker branch ref, let normal flow finalize:**
```bash
ssh homeserver "cd /srv/brehon-fork && \
  git update-ref refs/heads/junior/quality-r2-task5-524-recovered refs/recovery/quality-r2-task5-524-orphaned && \
  git log --oneline -2 refs/heads/junior/quality-r2-task5-524-recovered"
```
Then finalize-merge that branch into `phase-v1-quality-r2` the same way the daemon would have (the quality-r2 session knows its own finalize discipline), OR cherry-pick `f0b24f577` (the code) onto the phase branch and re-raise the validate-pending DQ.

**Option B — cherry-pick just the code commit (simplest; mirrors how rt-r4 handled its own stale-base worker):**
```bash
# from the quality-r2 lane worktree on phase-v1-quality-r2:
git fetch origin phase-v1-quality-r2
git checkout phase-v1-quality-r2 && git pull
# cherry-pick the EnvVarGuard code commit (the DQ commit 4e5ec7748 is stale-shape; re-raise validate-pending fresh instead)
ssh homeserver "cd /srv/brehon-fork && git rev-parse refs/recovery/quality-r2-task5-524-orphaned^"   # = f0b24f577 full SHA
git cherry-pick f0b24f577
# then re-run validate-pending-laptop for task 5 normally
```
Recommend Option B (matches the rt-r4 cherry-pick-the-code pattern; avoids the worker's possibly-stale DQ commit). Verify the 13 EnvVarGuard sites are present after cherry-pick.

## Why the recovery ref will persist

`refs/recovery/*` is a normal ref — `git gc` will not prune objects reachable from it. The work is safe until the quality-r2 session recovers it and deletes the recovery ref. **Do not let rt-r4 delete this ref** — only the quality-r2 session, after confirming recovery.

## Root cause + prevention (rt-r4 side)

rt-r4 cancelled by remembered task-ID without first running `show_task` to confirm identity + status. Task IDs are a **shared cross-lane sequence** on the daemon; #524 belonged to a different lane. Prevention now recorded in `workflow_state_v1_rt_r4.md` KEY LESSONS #2: **always `show_task <id>` to confirm ownership + status BEFORE any `cancel_task`.**
