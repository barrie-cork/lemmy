---
paths:
  - "scripts/**"
  - ".github/**"
---

# Cross-Repo Coordination

When your task changes something that affects other repos, proactively queue a follow-up task on the affected repo rather than relying on a human to notice.

## When to queue a cross-repo task

Queue a follow-up task via `junior-add-task` when your changes affect another repo:

- **Shared dependency updated** — a Python package, npm module, or API client used by multiple repos
- **API contract changed** — endpoint renamed, request/response schema modified, new required field
- **DB schema changed** — migration that affects consumers reading the same tables
- **Shared config changed** — Docker network, env var format, port assignment, volume mount
- **Infra-drift detected** (post-task-retro Step 5a) — queue a homeserver doc update task

## Before queuing

1. **Check daemon status**: call `junior-all-status` to confirm the target repo's daemon is active
2. **Never queue on paused repos** (dog-shelter, my-food-system) without explicit human instruction
3. **homeserver as target**: homeserver's Junior daemon is manual-start on the Mac. Check status first. If it is not running, fall back to writing an `issue-note` memory tagged `infra-drift,homeserver` (the existing passive mechanism)

## Safeguards

- **Max 1 cross-repo task per source task** — if your task affects 3 repos, pick the most critical one. Note the others in your eval memory for human follow-up.
- **No cascading** — if YOUR task was itself triggered by a cross-repo task (description contains `[from:`), do NOT queue further cross-repo tasks. Report downstream impacts in your eval memory instead.
- **Include provenance** in the task description: `[from:<source-repo>#<task-id>] <description>`
- **Keep descriptions actionable** — "Update API client for new /users endpoint schema" not "Something changed in agent-grey"

## Skip if

- Changes are purely internal to this repo (app logic, UI, tests with no external contracts)
- The downstream repo is not in Junior's repo list
- You are running a weekly-review or scheduled audit task
