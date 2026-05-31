---
name: Finalize-merge results are daemon-local-first, origin-push-second — where to look
description: When a base_branch=governance-v0 (or phase-branch) Junior task finishes, the daemon's finalize-merge lands on the DAEMON-LOCAL ref first and pushes to origin asynchronously second. Checking origin first shows a stale pre-merge tip. Canonical observation order: daemon-local log → ls-remote origin → local fetch.
type: feedback
---

## TL;DR

When a Junior task with `base_branch=governance-v0` (or any phase branch) reports `done`, the daemon's finalize-merge commits to the **daemon-local** ref first and pushes to origin **asynchronously afterward**. If you check `origin` first, you see the stale pre-merge tip — triggering a multi-probe hunt for a merge that already happened. Always look daemon-local first.

**Canonical observation order (non-destructive):**

1. `ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"` — the merge lands HERE first
2. `git ls-remote origin governance-v0` — compare: did the daemon push yet?
3. If daemon-local is ahead of origin: `ssh homeserver "cd /srv/brehon-fork && git push origin governance-v0"`, then locally `git fetch origin governance-v0 && git pull`

## Why (incident 2026-05-31, task #533)

Junior task #533 (`base_branch=governance-v0`) completed normally. Finalize-merge produced commit `a47a85e16` on daemon-local `governance-v0`. At the time the advisor polled, `origin/governance-v0` still pointed at the pre-merge brief commit `0a117e30e` — the daemon's async push had not landed yet.

The advisor checked `origin` first → saw stale tip → spent 4 probes (local log → daemon log → origin fetch → manual daemon push) before reconciling. User flagged this recurs ("always happening"). 2× threshold met (this session + user recurrence report) → promotes to lesson.

**Root cause:** the advisor's default post-task poll reads `git log origin/governance-v0` or `git ls-remote origin governance-v0`. The daemon push is asynchronous — a 10–60 second window exists between the finalize-merge commit on daemon-local and the `git push origin governance-v0` completing. Checking origin first during this window is always wrong.

## How to apply — canonical lookup order

```bash
# Step 1: daemon-local ref (the merge lands here first)
ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"

# Step 2: compare with origin (did the daemon push yet?)
git ls-remote origin governance-v0
# If the SHAs differ and daemon-local is ahead → daemon hasn't pushed yet.
# If SHAs match → daemon already pushed; do a local fetch and continue.

# Step 3a: if daemon-local is ahead, push it manually, then pull locally
ssh homeserver "cd /srv/brehon-fork && git push origin governance-v0"
git fetch origin governance-v0 && git pull
```

**Substitute `governance-v0` with `phase-v1-<lane>` for phase-branch finalize-merges** (same ordering applies — daemon-local first, origin second).

## Scope boundary — this is the clean-result observation case

This lesson covers **normal finalize completions** — the merge happened cleanly on the daemon, the advisor is just looking in the wrong place first. If what you find instead is one of these, you're in a different problem:

- **Daemon-local ref is non-fast-forward vs origin** (extra commits, divergence): that is the redundant-finalize divergence case → `feedback_junior_finalize_skips_when_worker_pre_pushes.md` §"Sibling shape: multi-lane redundant-finalize divergence".
- **Daemon-local `governance-v0` moved to a phase branch's tip** (destructive reset clobber): → `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md`.
- **Junior reports `done` but no merge commit exists anywhere**: → `feedback_junior_finalize_skips_when_worker_pre_pushes.md` §"finalize-skip" (worker pre-pushed, daemon saw no local work).

## See also / Companion lessons

- `feedback_junior_finalize_skips_when_worker_pre_pushes.md` — the "No changes to merge" finalize-skip gap (worker pre-pushed) + the divergent redundant-finalize case; different problems sharing the same symptom (phase tip unchanged after task `done`).
- `feedback_daemon_finalize_resets_trunk_to_wrong_phase_branch.md` — clobbered/orphaned merge from a lane-agent `reset --hard`; destructive, different problem.
- `feedback_planner_dq_id_via_origin_not_daemon_local.md` — related daemon-local-vs-origin distinction for reading DQ ids: read via origin, not daemon-local.
- `.claude/rules/advisor-orchestrator.md` §3.1 — the post-finalize check codified in the stage-shape orchestration bullet list.
