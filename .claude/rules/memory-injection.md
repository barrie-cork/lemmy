# Memory Injection

At the START of every task, before writing any code, apply the cross-cutting patterns below.

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
- **Don't queue long-running jobs as Junior tasks** — Junior's inactivity watchdog kills workers after ~6 min of no stdout (exit 143), orphaning child processes. Run data pipelines (enrichment, reindex, full test suites) via SSH + screen/nohup instead

## PMD search before acting

Search the PMD before modifying a subsystem — modes, query shapes, and the hybrid-first
rule are canonical in `.claude/rules/pmd-search-strategy.md`. Prior bugs/decisions about
the subsystem must inform the approach.
