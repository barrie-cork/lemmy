# Session Awareness

Multiple Claude Code sessions may run concurrently in homeserver. Before write operations, check for conflicts.

## Before write operations (git commit/push, SCP, sync scripts, SSH deploys)

**On a shared working tree, run `git branch --show-current` immediately before every `git add`/`commit`/`push`** — a concurrent session can switch the checkout's branch out from under you, silently landing your commit on the wrong branch. Never assume the branch is where you left it after any gap or hand-off. (Recovery if it lands wrong: cherry-pick the commit onto the intended branch, then `git branch -f <wrong-branch> <its-prior-tip>` to drop the stray.)

1. Check who's active: `bash scripts/agent-activity.sh list`
2. If another session shows `mode: write` on the same repo, **tell the user before proceeding**
3. Claim your work: `bash scripts/agent-activity.sh claim <repo> "<short task description>" write`
4. When write work is done: `bash scripts/agent-activity.sh release`

## Before writing a cross-session handoff memory

When writing a "start here next session" memory (or otherwise persisting state for a future session to resume), **re-read `MEMORY.md` and the memory dir fresh first** — do not rely on the copy injected at session start. `MEMORY.md` is shared mutable state: a concurrent session may have appended a directly-relevant memory since yours began. Fold any new, relevant finding into your handoff and cross-link it (`[[name]]`) rather than duplicating recon.

## Skip for

- File reads, grep, memory searches, planning mode
- Read-only SSH commands (checking status, querying DBs)
- Junior task queuing (Junior isolates in worktrees)

## Notes

- The activity file is at `.claude/agent-activity.json` (gitignored, runtime only)
- Sessions are auto-registered by SessionStart/SessionEnd hooks
- Stale entries (dead PIDs, >60 min idle) are auto-pruned on every read
- This is advisory only — no blocking. The human decides on conflicts.
