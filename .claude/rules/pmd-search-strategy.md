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

Topology, live DB location, embedding/backfill mechanics: `.claude/rules/pmd-invariants.md`
invariants #1 + #3 are canonical (HTTP daemon on homeserver since 2026-06-11; the laptop
file-DB is a STALE copy; do NOT run laptop `backfill.js`). This section previously carried
2026-05-16 laptop-topology instructions — superseded, removed by harness-audit-2026-06-12 P3.

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
