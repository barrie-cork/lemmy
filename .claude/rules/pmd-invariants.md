# PMD invariants

Five non-negotiable system invariants protecting the PMD substrate. Per `docs/research/brehon-rls-pmd-review.md` §4.8. Incident narratives + FP/FN taxonomy: `.claude/refs/pmd-invariants-incidents.md`.

## 1. Canonical PMD path (absolute, cross-lane)

Every worktree's `.mcp.json` `PROJECT_MEMORY_DB` MUST be
`C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db` — never relative, never
per-lane. Detection: `bash .claude/hooks/pmd-canonical-guard.sh`.

**Current topology (HTTP server, 2026-05-30+):** PMD MCP is an HTTP daemon (`http://localhost:11435/mcp`; Tailscale `http://100.104.171.26:11435/mcp`). `PROJECT_MEMORY_DB` env-var mechanism superseded — server manages DB server-side. Guard hook invocation is harmless no-op.

See also: `feedback_pmd_cross_lane_canonical_db.md`, `feedback_pmd_retro_check_http_store_split.md`, `multi-lane-worktree.md` §"PMD is cross-lane shared".

## 2. Two systems, one source of truth

System 1 (auto-loaded `.md` under `~/.claude/projects/.../memory/`) and System 2 (queryable SQLite-vec DB) are NOT interchangeable.
- Never write retro content directly to System 1
- Never assume System 2 is loaded at SessionStart
- When touching "project memory": check System 1 (`ls memory/*.md | wc -l`) AND System 2 (`memory_search_hybrid(query: "test", limit: 1)`) explicitly

See also: `feedback_pmd_two_memory_systems_distinction.md`

## 3. No write-time embedding (weekly backfill only)

`memory_write` / `memory_write_eval` writes FTS5 row only — NOT the vector. Under HTTP-daemon topology: interactive `backfill.js` is RETIRED. Weekly-review §1b sweep is the sole embedding pass. Do NOT re-add inline `backfill.js` to session-retro (targets wrong daemon-local store).

See also: `feedback_pmd_backfill_after_write.md`, `feedback_pmd_retro_check_http_store_split.md`, `.claude/PRPs/specs/mcp-write-time-embedding.md`

## 4. LESSON-trailer discipline

Every durable learning observation: `LESSON:` trailer in commit body OR `kind: "log"` DQ entry. Never both, never neither.

- Junior subagents: end commit body with `LESSON: <one-line observation>`
- Advisor harvests at retro time → promotes to PMD + `.claude/lessons/`

See also: `feedback_junior_pmd_write_convention.md`

## 5. SessionStart canonical-PMD guard (post-v1-rls-r1)

`pmd-canonical-guard.sh` runs at every SessionStart; emits loud WARN on lane drift. Wired via `SessionStart` in per-worktree `settings.local.json` — must be part of lane-bootstrap checklist; does not propagate automatically.

See also: `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`

## See also

- `.claude/rules/multi-lane-worktree.md` §"PMD is cross-lane shared, NOT per-lane isolated"
- `.claude/rules/pmd-search-strategy.md`
- `.claude/hooks/pmd-canonical-guard.sh`
- `docs/research/brehon-rls-pmd-review.md` §3 + §4 + §5.3
