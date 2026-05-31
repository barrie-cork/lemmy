# Universal guards

Four rules that apply in every session without exception. Skip conditions: None for all four.

---

## 1. Circuit breaker — retry and fallback

On tool or service failure: retry once (wait 5s), try fallback, then stop. Max 2 retries per tool per task.

| Failed tool/service | Fallback |
|---|---|
| `serena` code search | Use Grep/Glob directly |
| `junior-mcp` SSH connection | Write findings to local file, report SSH issue |
| `project-memory` MCP | Check `docs/memory/*.md` for promoted patterns |
| `docker-deploy-mcp` | Report the specific error — do NOT attempt manual Docker commands |
| External API / WebFetch timeout | Retry once, then proceed with available information |
| Git operations | Check if worktree lock or index.lock exists, report if so |

**Do NOT:**
- Retry the exact same command unchanged
- Use `--force`, `--no-verify`, `rm -rf`, or `git reset --hard` to bypass a legitimate error
- Loop past 2 retries
- Swallow errors silently — every failure must be visible in output

Enforcement: all sessions (interactive and Junior `-p` mode). Count retries per tool per task — cap is global across the session.

---

## 2. Escalation — human handoff

Stop and escalate when: 2+ consecutive failures of the same approach; ambiguous requirements not resolvable from CLAUDE.md/project-memory/code; security-critical file changes; unverifiable external state assumptions.

**Security-critical files requiring pause-and-describe before executing:**
- `.env` files and secrets, Docker compose/Dockerfiles, firewall rules (`ufw`, `iptables`), systemd service files, SSH keys and authorized_keys, Nginx/reverse proxy configs, cron jobs running as root

**Escalation note must contain:**
1. What was attempted
2. What failed (exact error or ambiguity)
3. What is needed (decision / access / clarification / approval)
4. Suggested next steps

**Where:** Junior tasks → write to task output (visible via `junior-show-task`). Interactive → output directly to user.

Enforcement: all sessions. The escalation note IS the deliverable when a task cannot complete.

---

## 3. Input validation — integrator rule

Before reasoning on tool output, validate it. Do not process broken, empty, or contradictory data as if correct.

| Situation | Action |
|---|---|
| MCP tool returns error/empty/timeout | Flag explicitly; do not reason over error message as data |
| SSH/Bash mixes stderr with valid output | Separate noise from result before processing |
| File unexpectedly empty/binary/malformed | Stop and report; do not hallucinate content |
| Tool results contradict each other or prior knowledge | Acknowledge contradiction; state what you expected, what you got, which source you trust |
| Search returns 0 results | Try at least one alternative search before concluding absence |

Enforcement: all sessions, all tool results. Every validation failure must be visible in output.

---

## 4. Post-task retrospective — mandatory

Before exiting ANY task, call `memory_write_eval`. Read `.claude/skills/post-task-retro/SKILL.md` and follow it. Do NOT use a slash command.

**Minimum required `memory_write_eval` fields:**
- Title starting with `"Task retro: "`
- Score (0.0–1.0) using 3-signal rubric: goal achieved (0.40) + tests pass (0.30) + clean execution (0.30)
- Confidence estimate recorded BEFORE scoring
- Tags including outcome (`success`/`partial`/`failure`) and repo name

**Required content lines:**
```
SCORE: <composite>
CONFIDENCE: <pre-scoring estimate>
Goal achieved: <yes/partial/no>
Tests: <pass/fail/none>
Clean execution: <yes/no>
```
On partial/failure, also: `ROOT_CAUSE: <category> — <explanation>` + write a lesson memory.

**Stop hook enforcement (`retro-check.sh`):**
- On `junior/*` branch: requires `Task retro:%` row with `source_ref = <exact branch>` within last 30 min
- On any other branch: any `Task retro:%` row within last 15 min

Skill step order: commit → scoring → root cause → lesson → auto-promote → doc drift → blast radius → **memory_write_eval LAST**.

**Skip ONLY if:** task cancelled externally; weekly-review task; scheduled audit task. Do NOT skip for failures, trivial tasks, or "didn't change much".

Score calibration: typical range 0.45–0.70. Above 0.85 should be rare.
