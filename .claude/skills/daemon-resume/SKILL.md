---
name: daemon-resume
description: >
  Health check for repos resuming after being paused. Validates PM, Serena, skills, and MCP config.
  DO use when: re-enabling a paused Junior daemon, first task after a long gap (>7 days).
  Do NOT use for: active repos, routine tasks, weekly reviews.
---

# Daemon Resume Health Check

Run this skill as the first task when re-enabling a paused Junior daemon. It validates that all supporting systems are functional before queuing real work.

## Steps

### 1. Project Memory health

1. Call `memory_review` — record total count, type breakdown, prunable entries
2. Call `memory_prune` with `dry_run: true` — note what would be pruned
3. If prunable > 50% of total, run `memory_prune` (not dry run) to clean up
4. Write a test memory and search for it to confirm read/write works:
   ```
   memory_write(title="Resume smoke test", memory_type="deploy-note", importance=1, content="Daemon resume validation")
   ```
   Then `memory_search(query="Resume", tags="deploy-note")` — confirm it returns. (FTS5 is correct here: this is an exact-phrase roundtrip smoke test, not a recall query.)

### 2. Serena check

1. Check if `.serena/` directory exists
2. If it exists, list `.serena/memories/` — note file count and last-modified dates
3. If memories are empty or all older than 14 days:
   - Run Serena's `get_symbols_overview` on project root
   - Update `.serena/memories/` with current project state
4. If `.serena/` doesn't exist, note "Serena not configured" and skip

### 3. Shared skills verification

1. List `.claude/skills/` — confirm post-task-retro and weekly-review exist
2. List `.claude/rules/` — confirm post-task-retro.md rule exists
3. If any are missing, note: "Shared skills/rules out of sync — run sync-shared-skills.sh from Mac"

### 4. MCP config check

1. Read `.mcp.json` — confirm `project-memory` MCP is configured
2. Check that `PROJECT_MEMORY_DB` path is absolute (not relative)
3. If Serena exists in `.serena/`, confirm Serena is also in `.mcp.json`
4. Flag any issues found

### 5. Git state

1. `git status` — confirm clean working tree on `main` branch
2. `git log --oneline -5` — note last activity date
3. If not on `main`, flag: "Not on main branch — Junior merges into HEAD"

### 6. Report

Write a `deploy-note` memory summarising:
```
memory_write(
  title: "Daemon resume check: <repo>",
  memory_type: "deploy-note",
  tags: "daemon-resume,<repo>",
  importance: 2,
  content: |
    PM: <total memories>, <prunable> prunable, read/write: OK/FAIL
    Serena: <configured/not configured>, <fresh/stale/empty>
    Skills: <count> skills, retro rule: <present/missing>
    MCP: project-memory <OK/FAIL>, Serena <OK/FAIL/N/A>
    Git: <branch>, last commit <date>, working tree <clean/dirty>
    Issues: <list or "none">
)
```

Delete the smoke test memory from step 1.
