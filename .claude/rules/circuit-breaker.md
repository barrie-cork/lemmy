# Circuit Breaker — Retry and Fallback Rule

On tool or service failure, retry once, then try an alternative, then stop. Do not loop. Maximum 2 retries per tool per task.

## Retry sequence

### First failure
Wait 5 seconds, then retry the same approach once. Transient errors (network timeouts, SSH drops, rate limits) often resolve on a single retry.

### Second failure
Do not retry the same approach again. Check if an alternative tool or method exists:

| Failed tool/service | Fallback |
|---------------------|----------|
| `serena` code search | Use Grep/Glob directly |
| `junior-mcp` SSH connection | Write findings to local file, report SSH issue |
| `project-memory` MCP | Check `docs/memory/*.md` for promoted patterns |
| `docker-deploy-mcp` | Report the specific error — do NOT attempt manual Docker commands |
| External API / WebFetch timeout | Retry once, then proceed with available information |
| Git operations | Check if worktree lock or index.lock exists, report if so |

If no known fallback exists, proceed to "third failure" behavior.

### Third failure
Stop attempting. Write partial results and a clear error description to the task output. Do not loop.

## Anti-patterns — do NOT do these

- **Infinite retry loops** — cap at 2 retries total, then stop
- **Brute-forcing permission errors** — if you lack permission, escalate (see escalation rule)
- **Retrying the exact same command unchanged** — if the first retry failed, something must change on the second attempt
- **Dangerous workarounds** — never use `--force`, `--no-verify`, `rm -rf`, or `git reset --hard` to bypass a legitimate error
- **Silent error swallowing** — every failure must be logged in your output

## Enforcement

- Apply in all sessions (interactive and Junior `-p` mode)
- Count retries per tool per task — the cap is global across the session, not per invocation
- When falling back, state clearly: what failed, what you're using instead, and any limitations of the fallback

## Skip conditions

- None. This rule always applies.
