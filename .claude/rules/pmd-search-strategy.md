# PMD Search Strategy

Project memory exposes two search tools. Pick by query shape.

> **Output-size footgun (2026-05-29):** a broad `memory_search_hybrid` query can return an
> 85K–105K-char result (hundreds of lines) that is auto-spilled to a tool-results file with a
> "read in chunks" instruction — a single recall query then costs more context than the thing
> you were investigating. This bites hardest in context-budget-sensitive sessions. Mitigations:
> (a) keep queries narrow and distinctive (specific subsystem/error terms, not generic phrases
> like "recent lessons improvement"); (b) for a pure *coverage check* ("do we already have a
> lesson on X?"), `grep`/`ls` the `.claude/lessons/` dir directly — it's faster and bounded;
> (c) if a broad semantic query is genuinely needed, run it inside a subagent so the dump stays
> out of the main context (per the spill-file's own guidance).

## brehon-fork PMD status (2026-05-16)

- **DB path:** `.project-memory/memory.db` (relative to repo root). The MCP server's env wires `PROJECT_MEMORY_DB` here.
- **Indexed corpora:** evals/qa-results (auto-written by retros), decisions, bugs, **all `.claude/lessons/feedback_*.md` files** (imported via `scripts/sync-lessons-to-pmd.sh` as `memory_type: "pattern"`, `tags: "lesson,feedback"`).
- **Embedding mode:** **hybrid (semantic + FTS5).** Fixed 2026-05-16: `.mcp.json`'s `project-memory` env now sets `OLLAMA_URL=http://homeserver:11434` (Tailscale-routed Ollama on the EliteDesk, model `nomic-embed-text`, 768-dim). The MCP server's hardcoded default was the stale LAN IP `192.168.1.157:11434` — unreachable from the laptop, which is why `memory_vectors` sat empty and `memory_search_hybrid` silently FTS5-degraded for ~7 days. All memories backfilled via `dist/scripts/backfill.js`. Query-time embedding works after MCP-server restart (reload Claude Code session to pick up `.mcp.json`).
- **Re-backfill after bulk lesson import:** `scripts/sync-lessons-to-pmd.sh` adds rows without vectors. Re-run the backfill to embed them: `OLLAMA_URL=http://homeserver:11434 PROJECT_MEMORY_DB=.project-memory/memory.db PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js --verbose`. Idempotent on `(memory_id, model)`.
- **If embeddings stop working again:** check `curl -s -m5 http://homeserver:11434/api/tags` (Tailscale up? Ollama up?), then `SELECT COUNT(*) FROM memory_vectors` (0 = backfill needed). The tool degrades silently to FTS5 with one stderr line — absence of vector hits is the symptom.

## Default: `memory_search_hybrid`

For multi-word or natural-language queries, use `memory_search_hybrid`. It combines semantic embeddings with FTS5 via Reciprocal Rank Fusion (k=60, top-50 + top-50), handles synonyms and concept drift, and delivers +62.7% Recall@10 over FTS5 alone in repos where embeddings are populated. Tags are still recommended for scoping.

**On brehon-fork (laptop, Ollama via Tailscale — operational since 2026-05-16):** semantic embeddings are live. Multi-word natural-language queries now resolve through RRF — the `+62.7% Recall@10` advantage applies. Synonym/concept-drift queries that previously missed in FTS5-only mode now hit:

- `query: "LemmyError doesn't implement std::error::Error in tests"` → hits `feedback_lemmy_error_no_std_error.md` via embedding similarity.
- `query: "postgres jsonb canonical text rendering"` → hits `feedback_postgres_jsonb_canonicalization.md` (embedding bridges `canonical`↔`canonicalization`; FTS5 alone could not).

Single distinctive keywords still work too (FTS5 leg of the fusion):

- `query: "LemmyError"` → `feedback_lemmy_error_no_std_error.md` + adjacent `feedback_async_pool_test_pattern.md`.
- `query: "jsonb"` → `feedback_postgres_jsonb_canonicalization.md`.

```
memory_search_hybrid(query: "docker container OOM memory limit", tags: "infrastructure")
memory_search_hybrid(query: "junior worktree stale branch cleanup", tags: "junior")
memory_search_hybrid(query: "hook registration settings.json not updated", tags: "junior")
```

Graceful fallback: if the embedding service (ollama) is unreachable, the tool automatically returns FTS5 results with a single stderr warning — no caller changes needed.

## Fallback: `memory_search` (FTS5, exact-token)

Use `memory_search` only when you need strict token matching — workflow IDs, exact error strings, specific tags. Capped at 2 words per query (enforced by `hooks/shared/validate-memory-search.sh`); `tags` required unless `memory_type` is set.

```
# Good — single token + tag
memory_search(query: "worktree", tags: "junior")

# Good — type filter (word limit relaxed to 3)
memory_search(query: "retro", memory_type: "pattern")

# Bad — multi-word query; use memory_search_hybrid instead
memory_search(query: "docker deployment bug fix worktree issue")
```

## Before writing

Search first (`memory_search_hybrid` for recall) to avoid duplicates. If a similar memory exists, use `supersedes: <old_id>` on the new write.
