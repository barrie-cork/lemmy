---
name: Junior workers hang at 0% CPU after raising their validate-pending-laptop DQ entry
description: After pushing the feat() + chore(decision-queue) commits to origin, some Junior worker sessions enter an Sl (light sleep) state with sub-5% CPU and no log output for 10+ minutes. The work is complete on origin; the hang is downstream of useful output. Cancel + advisor FF-merge + lane-side cargo is the recovery template.
type: feedback
---

# Worker hang post-DQ-raise

## The pattern

A Junior `impl-task` worker:
1. Authors its declared output files (lesson / hook / spec / rule / doc).
2. Commits `feat(<phase>): ... (task N)`.
3. Computes `next_id` and raises a `kind: "validate-pending-laptop"` DQ entry.
4. Commits `chore(decision-queue): impl raised DQ #N — validate-pending-laptop task N`.
5. Pushes both commits to `origin/junior/...`.
6. **Hangs.** Process stays alive (`Sl` state on Linux), sub-5% CPU, no stderr output, no cargo invocation observed via `ps`/`find`.

Observed 2× during v1-rls-r1 (Tasks 6 + 8). Both workers reached the same end state — origin had complete work; daemon had unfinished worker process. Both required `cancel_task` + advisor recovery.

## Diagnosis recipe

Confirm the pattern (per `feedback_junior_task_liveness_check.md`):
- `ssh homeserver "ps -p <pid> -o pid,etime,pcpu,stat"` — `Sl` state + low CPU + long elapsed = hang.
- `ssh homeserver "find /srv/brehon-fork/.junior/worktrees/job-<id>/.claude/PRPs/debug/ -mtime -10 -type f"` — no recent log files = no cargo invocation in flight.
- `git fetch origin "+refs/heads/junior/*:refs/remotes/origin/junior/*"` + `git log phase-<phase>..origin/junior/<branch> --oneline` — if BOTH `feat()` and `chore(decision-queue)` commits are visible, the worker is past its useful output.

## Recovery template

1. `mcp__junior-brehon__cancel_task` on the worker id.
2. `git fetch origin "+refs/heads/junior/*:refs/remotes/origin/junior/*"` (full refspec — wildcard glob may fail per the v1-rls-r1 incident).
3. `git merge --ff-only origin/junior/<worker-branch>` on the lane worktree's phase branch.
4. Verify VALIDATE probes pass on the merged tree.
5. Run `bash scripts/brehon/cargo-check.sh --workspace --features full` on the lane worktree.
6. Mutate the DQ entry in place: `result: "pass"`, `answered_by: "advisor-laptop"`, `resolved_at: <NOW>`, `answer: "<context citing the hang + advisor recovery>"`.
7. Commit + push the mutation. Subject: `chore(decision-queue): advisor-laptop mutated DQ #N — pass validate-pending-laptop`.

## How to apply

- **Brief constraint:** for any task where this pattern has fired in the same phase, the next brief's §6 KNOWN harness limitations section should explicitly say "Worker hang post-DQ-raise observed N times this phase; do NOT block on cargo invocation > 2 min — push first, then attempt cargo, then exit either way".
- **Watchdog:** daemon's `--max-turns 150` cap is too loose at 60 min. A 10-min `Sl`-state-with-no-log-output watchdog would catch the hang at lowest cost.
- **Inflection:** the second hang in a phase triggers the advisor-authoring decision per `feedback_advisor_authoring_under_daemon_stress.md`.

## See also

- `feedback_junior_task_liveness_check.md` — the broader liveness pattern
- `feedback_advisor_authoring_under_daemon_stress.md` — the inflection criterion
- `feedback_junior_finalize_skips_when_worker_pre_pushes.md` — adjacent failure mode (daemon finalize-merge skip)
- `.claude/PRPs/reports/v1-rls-r1-retro.md` — first formal observation; complexity scores include hang notes
