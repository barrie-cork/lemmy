# Session Awareness

Multiple Claude Code sessions may run concurrently in tanglewood-hive. Before write operations, check for conflicts.

## Before write operations (git commit/push, SCP, sync scripts, SSH deploys)

1. Check who's active: `bash scripts/agent-activity.sh list`
2. If another session shows `mode: write` on the same repo, **tell the user before proceeding**
3. Claim your work: `bash scripts/agent-activity.sh claim <repo> "<short task description>" write`
4. When write work is done: `bash scripts/agent-activity.sh release`

## Skip for

- File reads, grep, memory searches, planning mode
- Read-only SSH commands (checking status, querying DBs)
- Junior task queuing (Junior isolates in worktrees)

## Notes

- The activity file is at `.claude/agent-activity.json` (gitignored, runtime only)
- Sessions are auto-registered by SessionStart/SessionEnd hooks
- Stale entries (dead PIDs, >60 min idle) are auto-pruned on every read
- This is advisory only — no blocking. The human decides on conflicts.
