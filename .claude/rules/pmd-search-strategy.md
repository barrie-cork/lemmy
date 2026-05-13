# PMD Search Strategy

Project memory exposes two search tools. Pick by query shape.

## brehon-fork PMD status (2026-05-09)

- **DB path:** `.project-memory/memory.db` (relative to repo root). The MCP server's env wires `PROJECT_MEMORY_DB` here.
- **Indexed corpora:** evals/qa-results (auto-written by retros), decisions, bugs, **all 98 `.claude/lessons/feedback_*.md` files** (imported via `scripts/sync-lessons-to-pmd.sh` as `memory_type: "pattern"`, `tags: "lesson,feedback"`).
- **Embedding mode:** **FTS5-only.** The `memory_vectors` table is provisioned but empty — the laptop's MCP server cannot reach Ollama at the EliteDesk (Tailscale alias `homeserver`, port 11434). `memory_search_hybrid` auto-falls-back to FTS5; cross-tool API stays the same.
- **To wire embeddings later:** point the project-memory MCP env at `http://homeserver:11434` (Tailscale-routed Ollama), restart MCP server, run a backfill against rows where there is no `memory_vectors` entry. Future scope.

## Default: `memory_search_hybrid`

For multi-word or natural-language queries, use `memory_search_hybrid`. It combines semantic embeddings with FTS5 via Reciprocal Rank Fusion (k=60, top-50 + top-50), handles synonyms and concept drift, and delivers +62.7% Recall@10 over FTS5 alone in repos where embeddings are populated. Tags are still recommended for scoping.

**On brehon-fork specifically (laptop, no Ollama):** the call still works but runs FTS5-only. The `+62.7% Recall@10` advantage doesn't apply until embeddings are wired. Practical implication: **prefer single distinctive keywords over multi-word natural-language queries** for now. Examples that return useful hits:

- `query: "LemmyError"` → returns `feedback_lemmy_error_no_std_error.md` directly + adjacent `feedback_async_pool_test_pattern.md` (which references the lesson in its body).
- `query: "jsonb"` → returns `feedback_postgres_jsonb_canonicalization.md`.
- `query: "diesel migration"` → returns `feedback_lemmy_migration_runner.md` + adjacent migration-class lessons.

Multi-word queries that miss in FTS5-only mode (will hit when embeddings are wired):

- `query: "LemmyError doesn't implement std::error::Error in tests"` → currently returns nothing (common terms dilute the signal).
- `query: "postgres jsonb canonical text rendering"` → currently returns nothing (`canonical` isn't in the title; FTS5 doesn't bridge the synonym).

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
