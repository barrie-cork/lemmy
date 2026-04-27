# Memory Injection

At the START of every task, before writing any code:

1. If `docs/memory/PATTERNS.md` exists in this repo, read it
2. If `docs/memory/KNOWN_ISSUES.md` exists in this repo, read it

Apply any relevant patterns or known issues to your current task. These are confirmed failure modes (3+ occurrences) — not suggestions.

## Cross-cutting patterns (always applicable)

- **Commit before queuing Junior tasks** — worktrees are created from git HEAD; uncommitted changes don't propagate
- **Never use slash commands in task descriptions or CLAUDE.md enforcement** — Junior runs `claude -p` (non-interactive); slash commands aren't parsed. Use `.claude/rules/` for mandatory behaviour, reference skill files by path
- **Empty job queues may mean "never ran"** — check active=0 AND waiting=0 AND paused=0, then verify outputs end-to-end rather than trusting queue counters
- **Serena: don't list css/javascript/html/scss in project.yml** — use `typescript` (covers JS/TS) and valid Language enum values only
- **.mcp.json must be committed** — Junior worktrees without it have zero MCP tools
- **Docker test commands create root-owned files** — these block worktree cleanup; use `--user $(id -u):$(id -g)` or expect manual cleanup
- **Creating directories under /srv/ needs sudo** — `/srv/` is root-owned; use `sudo mkdir -p /srv/<dir> && sudo chown barrie:barrie /srv/<dir>`. Same applies to `/srv/webdata/`, `/srv/backups/`
- **Verify before mutating** — before `mv`/`cp`/`rm -rf` on files or directories: backup live DBs first, check for untracked/ignored content in git repos, generate a manifest for batch deletions (3+ items). When `rm -rf` partially fails, STOP and investigate — don't force-retry
- **When removing a class/function, grep the entire repo for its name first** — imports in `__init__.py`, test files, and downstream consumers will break silently if not updated

## PMD search before acting

3. Search PMD for the subsystem being modified — prefer `memory_search_hybrid` for multi-word queries (semantic + FTS5 via RRF, +62.7% recall, handles synonyms). Example: `memory_search_hybrid(query: "docker container OOM memory limit", tags: "infrastructure")`. Prior bugs, decisions, and patterns about that subsystem must inform the approach before proposing a plan or fix.
4. Also fetch recent lessons: `memory_search_hybrid(query: "recent lessons improvement", tags: "lesson", limit: 5)`. Read any results and apply relevant lessons to your current approach. These are improvement points from recent sessions — not yet confirmed as patterns but worth heeding.
   - Single-keyword FTS5 still works via `memory_search` (e.g. `memory_search(query: "worktree", tags: "junior")`), but the one-keyword-one-tag workaround is no longer required — hybrid handles multi-word queries correctly.
