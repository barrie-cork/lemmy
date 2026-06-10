---
name: pmd
description: Query and write the Project Memory Database from pi via CLI wrappers.
---

# PMD — Project Memory Database access

The Project Memory DB (PMD) stores lessons, patterns, and retro findings from this codebase.
Pi has no native MCP, so use these CLI wrappers via the `bash` tool.

## Query (search for relevant lessons)

```bash
bash /srv/brehon-fork/scripts/brehon/pmd-query.sh "your query here" --limit 5
```

**Options:**
- `--limit N` — number of results (default 5)
- `--tags TAG1,TAG2` — filter by tags (e.g. `lesson,feedback` or `junior`)

**Always search PMD before starting implementation work.** Use natural-language queries:

```bash
# Find lessons about a specific error or pattern
bash /srv/brehon-fork/scripts/brehon/pmd-query.sh "LemmyError std error trait" --limit 3

# Find lessons relevant to a file class
bash /srv/brehon-fork/scripts/brehon/pmd-query.sh "e2e test async pool postgres" --tags "lesson"

# Find recent lessons
bash /srv/brehon-fork/scripts/brehon/pmd-query.sh "recent lessons improvement" --tags "lesson" --limit 5
```

Output is a JSON array. Read the `title` and `content` fields of each result.

## Write (record a lesson after a task)

```bash
bash /srv/brehon-fork/scripts/brehon/pmd-write.sh \
  --title "Short descriptive title" \
  --content "Detailed lesson content" \
  --type "lesson" \
  --tags "lesson,feedback,brehon-fork"
```

**Types:** `pattern`, `decision`, `lesson`, `reference`, `user`, `feedback`, `project`

## Health check

```bash
bash /srv/brehon-fork/scripts/brehon/pmd-doctor.sh
```

Use `pmd-doctor.sh` when Pi reports PMD startup warnings, query/write wrappers fail, or a lesson sync says it could not obtain an MCP session ID. It verifies endpoint discovery, token presence, authenticated MCP `initialize`, and `tools/list` without printing secrets.

## Auth and topology

`PMD_HTTP_TOKEN` is read from `PMD_ENV_FILE` when set, then `/srv/brehon-fork/.env`, then repo-local `.env` (Mac/P50 pi sessions). The token is the same Bearer token configured in the laptop's `pmd-http-mcp` Windows service (NSSM `AppEnvironmentExtra`).

Topology is repo-specific:

- **Lemmy/Brehon:** PMD is an HTTP MCP service on the P50/Windows laptop. From EliteDesk/Mac use `http://100.104.171.26:11435/mcp` over Tailscale; on the PMD host use `http://localhost:11435/mcp`. Do not assume a per-worktree SQLite PMD is live.
- **phd-vault:** PMD topology differs and may be local to that repo/machine. Do not copy Lemmy's `/srv/brehon-fork/.env`, Tailscale URL, or service-host assumptions into phd-vault; inspect that repo's PMD docs/config first.
