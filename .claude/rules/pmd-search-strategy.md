# PMD Search Strategy

Project memory exposes two search tools. Pick by query shape.

## Default: `memory_search_hybrid`

For multi-word or natural-language queries, use `memory_search_hybrid`. It combines semantic embeddings with FTS5 via Reciprocal Rank Fusion (k=60, top-50 + top-50), handles synonyms and concept drift, and delivers +62.7% Recall@10 over FTS5 alone. Tags are still recommended for scoping.

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
