---
name: check-dq
description: "Poll the Brehon decision-queue on brehon-fork governance-v0 for new pending blockers + recently-resolved entries, cross-referenced with running Junior tasks. Read-only. Use this every time the user asks to check the DQ, the decision queue, advisor queue, Brehon DQ, or pending decisions; or when running a poll loop on a Brehon impl-task; or after queueing a JM/AD impl-task to verify mid-task DQ pushes are flowing. Invoke when the user types /check-dq, mentions polling the queue, or asks 'any new DQ entries'. Also use this inside /loop /check-dq for periodic polling per the advisor-orchestrator rule's polling-loop discipline."
---

# Check Brehon DQ

Read-only poll of the Brehon decision-queue (`.claude/decision-queue.json`) on the `brehon-fork` repo's `governance-v0` branch. Reports pending blockers, recent resolutions, and running Junior tasks in a single concise report — designed to satisfy the advisor session's polling-loop discipline (per `brehon-fork/.claude/rules/advisor-orchestrator.md`) without flooding context.

## When to use

- The user types `/check-dq` directly
- Inside `/loop /check-dq <interval>` for periodic polling during a Brehon sub-phase
- After queueing a JM/AD impl-task to verify mid-task DQ entries are flowing through to `governance-v0`
- The user asks "any new DQ?", "anything pending?", "what's the DQ status?", "check brehon decision queue"
- The user asks about advisor-loop state without naming the file specifically
- **Escalated from `/start-brehon --fast <N>`** when its probe 5 (DQ pending count) returns `pending > 0` — fast-mode does the cheap count probe, this skill does the full triage. Don't run `/check-dq` redundantly if `/start-brehon --fast` already returned `pending=0`; their DQ probes read the same source.

## What this skill does NOT do

- It does not write to `decision-queue.json` (use the manual flow per `brehon-fork/.claude/rules/decision-queue.md` for that)
- It does not commit or push anything (read-only)
- It does not queue Junior tasks (that's a separate decision the user makes)
- It does not poll arbitrary repos — scoped narrowly to `brehon-fork governance-v0`. To poll a phase branch, pass `--branch <name>`.

## Inputs

`$ARGUMENTS` (all optional):
- `--branch <name>` — defaults to `governance-v0`. Only override for advanced cases (e.g. polling a phase branch when the advisor's polling target shifts).
- `--since <id>` — only show resolved entries with id strictly greater than this number. Useful for "what's new since I last checked DQ #54".

## Steps

### 1. Fetch the latest governance-v0

```bash
cd C:/Users/barri/Developer/brehon-fork
MSYS_NO_PATHCONV=1 git fetch origin governance-v0
```

**Why `MSYS_NO_PATHCONV=1`:** Git Bash on Windows mangles arguments containing `/` and `:` (it tries to translate them as Unix paths). Without this env var, `origin/governance-v0:.claude/decision-queue.json` becomes `origin\governance-v0;.claude\decision-queue.json` and git errors with "ambiguous argument". The flag suppresses the path translation. Confirmed by today's session (3 retries before finding the right invocation).

### 2. Capture the last-known governance-v0 SHA

```bash
git log origin/governance-v0 --oneline -1
```

Use this for the "new commits since" check at the end. If you want to detect new commits across polls, save the SHA to a state file (see "State file" section below) and compare on the next call.

### 3. Read the DQ JSON via git show

```bash
MSYS_NO_PATHCONV=1 git show "origin/governance-v0:.claude/decision-queue.json" > C:/Users/barri/AppData/Local/Temp/dq.json
```

**Why the Windows-accessible temp path:** Python on Windows is a Windows-native binary and does not see Git Bash's `/tmp/` directory (which lives inside the MSYS root, invisible to non-MSYS tools). Use `C:/Users/barri/AppData/Local/Temp/dq.json` or any path under `%TEMP%`.

### 4. Parse with Python

```python
import json, datetime
data = json.load(open(r'C:\Users\barri\AppData\Local\Temp\dq.json'))
schema_version = data.get('schema_version', 1)  # missing = v1
pending = data.get('pending', [])
resolved = data.get('resolved', [])
now = datetime.datetime.now(datetime.UTC).strftime('%H:%M')  # use timezone-aware UTC, not utcnow() which is deprecated as of Python 3.12
```

**Schema reference:** the canonical schema lives in `C:/Users/barri/Developer/brehon-fork/.claude/rules/decision-queue.md`. Read this if you need the full field list. As of v2:
- Top-level keys: `pending` (array), `resolved` (array), `schema_version` (int).
- Entries have `kind: "blocker" | "log"` (v2 only). Default for v1 entries (no kind field) is `"blocker"` — preserves prior intent.
- Pending entries should always be `kind: "blocker"` (`kind: "log"` entries land directly in resolved per the rule).

### 5. Filter and summarise

```python
# Pending blockers (the advisor-loop stops)
pending_blockers = [e for e in pending if e.get('kind', 'blocker') == 'blocker']

# Recent resolved (top 3 by id, descending)
recent_resolved = sorted(resolved, key=lambda x: x.get('id', 0), reverse=True)[:3]

# Log-kind entries (retro-harvest candidates, no immediate action)
log_count = sum(1 for e in resolved if e.get('kind') == 'log')
```

If the caller passed `--since N`, filter `recent_resolved` to entries with `id > N`.

### 6. Get running Junior tasks

```
mcp__junior-brehon__list_tasks(status: "running")
```

Cross-reference: any pending DQ entry's `from` field likely correlates with one of the running tasks (the impl-task subagent that wrote it). Surface this correlation in the report.

### 7. Emit the report

**If there are pending blockers OR a Junior task transition since the last poll → emit the full report (≤8 lines):**

```
DQ poll @ <UTC HH:MM>
Pending blockers: <N>
  #<id> from=<from> q=<question[:90]>
  ... (one line per blocker)
Recent resolved (top 3 by id): #<id1>, #<id2>, #<id3>
Log-kind entries in resolved: <N total> (retro-harvest only, no action)
Running Junior tasks: <count>
  #<id> "<title[:60]>"
New commits on governance-v0 since last poll: <list of short SHAs, or "none">
```

**Otherwise (no pending blockers, no commit changes, no task transitions) emit a single line:**

```
no change @ <UTC HH:MM>
```

This satisfies the orchestrator-rule's model-efficient discipline: a quiet poll costs almost no context.

## State file (optional)

For multi-poll sessions (e.g. inside `/loop /check-dq 10m`), maintain a small state file to detect what's actually new:

Path: `C:/Users/barri/.claude/projects/C--Users-barri-Developer-homeserver/memory/check-dq-state.json`

```json
{
  "last_checked": "2026-04-27T12:35:00Z",
  "last_governance_v0_sha": "abce1a57c",
  "last_resolved_id": 54,
  "last_pending_ids": [],
  "last_running_task_ids": [12]
}
```

On each call:
1. Read the state file (treat missing as first run — bootstrap by snapshotting current state, report "first run, baseline captured")
2. Compute deltas: any new pending blocker id, any task transition (running → done/failed), any new commit SHA
3. Update state file with current snapshot before emitting the report
4. The report's "since last poll" wording uses the timestamp from the previous state

If you are running outside a loop (one-shot `/check-dq`), the state file is optional — just emit a current snapshot.

## Hard constraints

- **Read-only.** No `git commit`, no `git push`, no edits to `decision-queue.json`, no `mcp__junior-brehon__create_task` calls, no DQ writes of any kind. If you find yourself wanting to do any of these, stop and surface the finding to the user — they decide whether to act.
- **Always use `MSYS_NO_PATHCONV=1`** for the `git show` command. The flag is the only known way to make Git Bash on Windows pass `origin/<branch>:<path>` arguments correctly. Forgetting it produces a confusing "ambiguous argument" error.
- **Always use a Windows-accessible temp path** (under `%TEMP%`, e.g. `C:/Users/barri/AppData/Local/Temp/`) for the JSON dump. Git Bash's `/tmp/` is invisible to Windows-native Python.
- **Tolerate v1 entries.** Some historical entries lack `kind` and have inconsistent resolved-timestamp keys (`answered_at` / `ts_resolved` / `resolved_timestamp`). Default missing `kind` to `"blocker"`. Don't backfill — the historical drift is the audit trail.
- **Tolerate missing `schema_version`.** Treat as v1 if absent.
- **Never exceed 8 lines on quiet polls.** A loop firing every 10 minutes for an hour produces 6 polls — staying tight on context is load-bearing for advisor-session memory budget.
- **Don't recurse on the schema rule.** Do not call Read on `decision-queue.md` in steady-state polls — that's a 200-line file and reading it every poll would defeat the model-efficient design. Only read it if a parse error occurs and the schema may have changed.

## Common mistakes (from session 2026-04-27)

1. **Forgetting `MSYS_NO_PATHCONV=1`** — produces `fatal: ambiguous argument 'origin\governance-v0;.claude\decision-queue.json'`. Three retries today before this was found.
2. **Writing the JSON dump to `/tmp/`** — works inside Git Bash but Python on Windows can't see it. Use `C:/Users/barri/AppData/Local/Temp/`.
3. **Assuming the schema has `entries` key** — it doesn't. Top-level keys are `pending` and `resolved`. This is a common mistake from generic queue patterns.
4. **Listing all 51 resolved entries** — defeats the point. Only the top 3 by id (recent activity) are interesting on a poll.
5. **Polling the wrong branch** — the canonical advisor poll target is `governance-v0`, not the active phase branch. Phase branches have stale DQ state until the bm-task merges back.

## Reference docs (read on demand, not every poll)

- `C:/Users/barri/Developer/brehon-fork/.claude/rules/decision-queue.md` — v2 schema definition and attribution rules. Read if you hit a parse error.
- `C:/Users/barri/Developer/brehon-fork/.claude/rules/advisor-orchestrator.md` — the polling-loop discipline this skill supports. Read if you're unsure whether to surface something to the user vs. self-resolve.
- `C:/Users/barri/Developer/homeserver/CLAUDE.md` — system context including which advisor session model is current.
