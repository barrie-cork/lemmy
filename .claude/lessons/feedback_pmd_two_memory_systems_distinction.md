---
name: PMD has two distinct memory systems — .md files (auto-loaded) and SQLite DB (query-only)
description: The project memory directory contains two separate systems that look like one: .md files auto-loaded into context, and a SQLite DB queried by memory_search_hybrid. Runbooks/audits that treat them as one system miss half the setup.
type: feedback
---

The brehon-fork PMD at `~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/` contains **two distinct memory systems** that serve different purposes and require separate setup steps:

**System 1 — `.md` files (auto-loaded into context):**
- Files in the `memory/` directory (e.g. `MEMORY.md`, `project_*.md`, `feedback_*.md`) are auto-loaded by the Claude Code harness at session start.
- These are read directly as file content — no query needed.
- Seeding means: copy or write the `.md` files into the `memory/` directory.
- An `MEMORY.md` index controls which files surface in the CLAUDE.md system-reminder block (files listed there are auto-injected; others load on demand).

**System 2 — SQLite DB (query-only via MCP):**
- `.project-memory/memory.db` is a separate SQLite database managed by the `project-memory` MCP server.
- Entries are written via `mcp__project-memory__memory_write` / `memory_write_eval` tool calls.
- Entries are queried via `mcp__project-memory__memory_search_hybrid` / `memory_search`.
- The DB is NOT populated by placing `.md` files in the directory. The two systems do not auto-sync.
- Seeding the DB requires explicit `memory_write` / `memory_promote_to_file` tool calls, or bulk import via the MCP server's import tooling.

**Why this matters:** The advisor-CWD migration (2026-04-30) had a Step 4 that said "migrate 200 memory files" — this correctly moved the `.md` files (System 1). But the migration had no step for the SQLite DB (System 2). The DB seeding had to be added under live smoke pressure when `memory_search_hybrid` queries returned no results despite the `.md` files being present. The two systems look like one because they share the same conceptual namespace ("project memory") and the same MCP server exposes both — but they are physically separate and independently populated.

**How to apply:**

- Any migration, setup, or audit that touches "project memory" must address BOTH systems explicitly:
  - **System 1 check:** `ls ~/.claude/projects/<dir>/memory/*.md | wc -l` — count `.md` files; verify `MEMORY.md` index is present
  - **System 2 check:** run `memory_search_hybrid(query: "test", limit: 1)` and confirm a result returns — if empty, the DB is not seeded even if `.md` files are present
- When writing a migration runbook, name the two systems explicitly: "Step N: migrate .md files (auto-load system)" and "Step M: seed SQLite DB (query system)".
- When diagnosing "memory_search returns nothing", check System 2 first (empty DB), not System 1 (missing .md files) — they have different failure modes that produce the same symptom.

**Symptom to recognise:** `memory_search_hybrid` returns 0 results but you know there are many memory files in the `memory/` directory. This is the System 2 DB-not-seeded pattern, not a search query problem.

**Generalises to:** Any toolchain where a human-readable artifact (markdown files, YAML configs) and a query-able store (SQLite, Elasticsearch, vector DB) share a namespace but are populated independently. "I put the files there" is not the same as "the DB knows about them."

**Related lessons:**
- `feedback_runbook_audit_drift_post_event_check.md` — the companion lesson from the same migration event; verify runbook claims against live state before executing
- `feedback_pmd_split_brehon_vs_homeserver.md` (laptop memory dir only, not in `.claude/lessons/`) — the overall PMD split strategy
