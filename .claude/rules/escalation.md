# Escalation — Human Handoff Rule

When you hit repeated failures, ambiguous requirements, or security-critical decisions, stop and escalate to a human. Do not attempt increasingly creative workarounds.

## When to escalate

### Repeated failures
After 2 or more consecutive failures of the same tool or approach, stop trying and escalate. Do not retry a third time with the same method.

### Ambiguous requirements
When the task requirements are unclear and cannot be resolved from CLAUDE.md, project-memory, or existing code context, escalate rather than guess. Wrong guesses create more work than pausing to ask.

### Security-critical changes
When a task would modify security-critical files, pause and describe the intended change before executing. Security-critical files include:
- `.env` files and secrets
- Docker compose files and Dockerfiles
- Firewall rules (`ufw`, `iptables`)
- systemd service files
- SSH keys and authorized_keys
- Nginx/reverse proxy configs
- Cron jobs that run as root

### Unverifiable assumptions
When a task depends on external state you cannot verify (e.g., "the API should return X" but you can't test it), flag the assumption explicitly.

## Escalation format

Write a structured note containing:
1. **What was attempted** — the specific actions taken
2. **What failed** — the exact error or ambiguity
3. **What is needed** — decision, access, clarification, or approval
4. **Suggested next steps** — your best recommendation

## Where to escalate

- **Junior tasks:** write the escalation note to task output (visible via `junior-show-task`)
- **Interactive sessions:** output directly to the user

## Enforcement

- Apply in all sessions (interactive and Junior `-p` mode)
- Never silently fail a task — if you cannot complete it, the escalation note IS the deliverable
- The escalation note must be actionable: a human reading it should know exactly what to do next

## Skip conditions

- None. This rule always applies.
