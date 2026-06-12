---
name: update-state
description: "Sync Brehon MEMORY.md state files to ground truth. Reads git HEAD, Junior task statuses, and open DQ entries, then patches stale workflow_state_*.md and project_*.md files in the brehon-fork PMD System 1 directory. Use when the user says 'update state files', 'sync state', 'my state files are stale', 'update memory', or asks to reconcile MEMORY.md after session gaps. Also invoke automatically when /start-brehon or /check-dq surfaces a stale SHA or stale task status in an MEMORY.md entry. Invoke with /update-state."
---

# Update Brehon State Files

Reconciles the System 1 memory files (`~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/*.md`) against live ground truth (git HEAD, Junior task DB, DQ pending). Writes targeted patches to stale entries only — never rewrites a whole file from scratch.

## What this skill does NOT do

- Does not read plan bodies, PRD sections, or long documents (only file names + dates)
- Does not answer DQ entries — that's the advisor's job per `decision-queue.md`
- Does not queue Junior tasks
- Does not commit or push anything to git (memory files are gitignored System 1 artefacts)
- Does not touch System 2 (PMD SQLite-vec DB) — that requires separate `memory_update` MCP calls

## Inputs

`$ARGUMENTS` (optional):
- `--file <name>` — update only the named memory file (e.g. `--file workflow_state_m2_late_2.md`). Skips the full sweep.
- `--dry-run` — print what would change but do not write any files.

## Steps

### Phase 1 — Parallel ground-truth probes

Run all probes in a single message (parallel tool calls). The whole sweep should cost ≤ 5 KB of context.

#### A. Git HEAD on governance-v0

```bash
git -C "C:/Users/barri/Developer/brehon-fork" rev-parse --short HEAD
git -C "C:/Users/barri/Developer/brehon-fork" log --oneline -5
```

Captures: current HEAD SHA + last 5 commit subjects + dates. Used to detect "stale SHA" mentions in state files.

#### B. All active worktrees

```bash
git -C "C:/Users/barri/Developer/brehon-fork" worktree list
```

Captures: which lane worktrees exist, what branch they're on, what their tip SHA is. Used to update workflow_state entries referencing phase branches.

#### C. Open PRs on the fork

```bash
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,baseRefName,state,mergeStateStatus --limit 10
```

Used to detect stale "PR open" vs "PR merged" mentions.

#### D. Junior task statuses (recent + running)

Use `mcp__junior-brehon__list_tasks` with no status filter (or two calls: `status: "running"` and `status: "done"` limit 10 each).

Focus on: any task referenced by id in MEMORY.md (look for `#<N>` patterns with `[role:smoke]`, `[role:planning]`, `[role:impl-task]`, `[role:bm-task]`). The id list comes from a quick grep of the memory directory in Phase 2.

#### E. DQ pending count

```python
import io, json
dq = json.load(io.open(r'C:/Users/barri/Developer/brehon-fork/.claude/decision-queue.json', encoding='utf-8'))
pending = dq.get('pending', [])
print(f'pending={len(pending)}')
for e in pending:
    print(f'  id={e["id"]} from={e.get("from")} kind={e.get("kind","?")} q={e["question"][:70]}')
```

#### F. EliteDesk daemon gov-v0 tip (optional, only if probe A shows local is behind)

```bash
ssh -o ConnectTimeout=5 homeserver 'cd /srv/brehon-fork && git rev-parse --short governance-v0'
```

Skip if SSH is slow or unavailable — not gating.

### Phase 2 — Identify stale files

Read the MEMORY.md index to find which files are marked ACTIVE or ⏳ (pending):

```python
import re
mem = open(r'C:/Users/barri/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/MEMORY.md', encoding='utf-8').read()
# Find lines referencing .md files that are marked ACTIVE, ⏳, or have a HEAD SHA
active_files = re.findall(r'\[([^\]]+)\]\(([^)]+\.md)\)', mem)
# Find SHA mentions that might be stale
sha_mentions = re.findall(r'HEAD [`\'"]?([0-9a-f]{9,12})[`\'"]?', mem)
# Find task id mentions
task_ids = re.findall(r'(?:task |#)(\d{3,})', mem)
```

Then read each referenced `.md` file from the memory directory and scan for:

1. **Stale HEAD SHA** — any 9–12 hex char string that doesn't match probe A's HEAD or recent commits.
2. **Stale task status** — any `⏳` marker next to a task id that probe D shows as `done` or `failed`.
3. **Date-sensitive claims** — any absolute date older than 3 days paired with a "not yet started" or "pending" qualifier that ground truth contradicts.

Only list files that have at least one stale item. If `--file` is set, skip files not matching.

### Phase 3 — Draft patches

For each stale file, produce a minimal patch — the smallest edit that makes the file accurate:

**Rules for patch authoring:**

1. **Never delete existing structure** — only update the content of named sections (e.g. update the status line inside "## m2-late-2 plan status", not the section heading).
2. **Preserve frontmatter** — do not alter `name`, `description`, `metadata`, or any other frontmatter field.
3. **Absolute dates only** — use `YYYY-MM-DD` format. Never "yesterday" or "3 days ago".
4. **Mark transitions explicitly** — if a `⏳` item becomes done, change it to `✅` and add the completion date + outcome in parentheses.
5. **Session handoff blocks** — only APPEND to this section, never edit existing blocks. A new checkpoint block goes at the bottom.
6. **Keep patches minimal** — if a line just needs a SHA updated, change only that line. Don't reformat surrounding text.

If `--dry-run` is set, print the diff-style view of what would change to stdout, then stop.

### Phase 4 — Apply patches

Use the `Edit` tool with `old_string` / `new_string` for each targeted line change. For `workflow_state_*.md` files:

- Update the "plan status" section's date line if HEAD changed.
- Change `⏳` to `✅ <YYYY-MM-DD>` for any completed task reference.
- Append a new session handoff block under "## Session handoff blocks" if the HEAD SHA changed since the last block was written.

For `project_*.md` files:

- If the file tracks a task outcome (e.g. `project_auto_phase_test_dogfood_active.md`), update only the specific `⏳` task line.
- If the file has a "readiness path" section with steps, update the step statuses.

**Do not add new sections** — only update existing content.

### Phase 5 — Report

After all patches are applied, emit a compact report:

```
# update-state @ <YYYY-MM-DD HH:MM UTC>

## Ground truth
- governance-v0 HEAD: <sha> (<commit subject>)
- Junior running tasks: <count>
- DQ pending: <count>

## Patches applied
- <file>: <one-line description of what changed>
- ...

## No changes needed
- <file>: <reason still current>
- ...

## Skipped (could not verify)
- <file>: <reason — SSH unavailable, task id not found in DB, etc.>

Done. State files reflect <sha>.
```

If no patches were needed, emit a single line: `All active state files current as of <sha>. No changes.`

## Common stale patterns (save time — check these first)

| Pattern | How to detect | Typical fix |
|---|---|---|
| `HEAD \`<old-sha>\`` in status line | Old SHA ≠ probe A HEAD | Replace SHA + commit subject |
| `⏳ task #<N>` | Task N status = done in DB | Replace `⏳` with `✅ YYYY-MM-DD (done/<outcome>)` |
| "not yet started" + a branch now exists | Probe B shows worktree on this branch | Update status line to "in progress" + current SHA |
| "PR #<N> open" | Probe C shows PR N merged/closed | Update to "PR #N merged @ <sha>" |
| "BLOCKER: <thing>" already resolved per ground truth | Probe shows condition cleared | Append resolution + date to blocker line |

## Hard constraints

- **Never write `"answered_by": "advisor"` or any DQ entry** — this skill only touches MEMORY.md System 1 files, never `decision-queue.json`.
- **Never commit or push** — MEMORY.md files are gitignored; they don't go to origin. Writing them with the `Edit` or `Write` tool is the full action.
- **Never read plan bodies** — file names and dates only. Plans are 200–400 line files; reading them all would consume the context budget.
- **Stop if ambiguous** — if a state file's "⏳" marker cannot be resolved from the five probes (e.g. task id not found, branch not in worktree list), leave the marker intact and note it in the "Skipped" section. Do NOT guess.
- **MEMORY.md index is read-only** — only update the pointed-to `.md` files, never edit the MEMORY.md index itself (except to update the single-line pointer if a file was renamed — but that's rare and should surface to the user first).

## Reference

- Ground-truth sources: `git log`, `mcp__junior-brehon__list_tasks`, `decision-queue.json`, `gh pr list`
- Files this skill writes to: `~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/workflow_state_*.md`, `project_*.md`
- Files this skill never touches: `MEMORY.md` (index), `*.md` outside the memory directory, anything in `crates/`, `migrations/`, `docs/`, `.claude/`
- Companion skill: `/start-brehon` — read-only synthesis. Use `/start-brehon` when you want a status report; use `/update-state` when you want to persist the ground truth so the next session starts accurate.
- System 2 updates: if a state file's PMD memory entry (in the SQLite-vec DB) also needs updating, note it in the report under "System 2 pending" — but do not call `memory_update` here. That's a separate step the advisor does after reviewing the report.
