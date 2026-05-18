---
name: PMD has no write-time embedding — backfill.js must run after every memory write
description: The project-memory MCP server inserts text-only rows; nothing embeds at write-time (verified by source inspection of dist/index.js — zero embed/ollama refs). memory_search_hybrid silently FTS5-degrades for unembedded rows. backfill.js is the only embed path and must be run separately. Enforced via session-retro Step 5.5 + weekly-review Step 1b.
type: feedback
---

The brehon-fork PMD MCP server (`C:/Users/barri/Developer/MCPs/project-memory-mcp/`) has **NO write-time embedding**. Verified by source inspection 2026-05-16: `dist/index.js` (the MCP tool handlers — `memory_write`, `memory_write_eval`, `memory_promote_to_file`, `memory_update`) contains **zero** `embed` / `ollama` / `memory_vectors` / `generateEmbedding` references. `dist/db.js` has only the `memory_vectors` table DDL. Embedding happens **exclusively** in the standalone `dist/scripts/backfill.js`, which must be invoked separately.

**Consequence:** every `memory_write*` MCP call AND every `scripts/sync-lessons-to-pmd.sh` run inserts **text-only rows** into `memories`. They are immediately FTS5-searchable (`memory_search`, the FTS5 leg of `memory_search_hybrid`) but **NOT semantically searchable** — `memory_search_hybrid` silently degrades to FTS5-only for unembedded rows (one stderr line, no error). The +62.7% Recall@10 semantic advantage is absent for those rows until `backfill.js` embeds them. Synonym/concept-drift queries miss unembedded entries entirely.

**Confirmed recurrence (2026-05-16):** 8 PMD memories were unembedded at one point — #349–#351 (this session's eval + 2 promoted lessons) AND #344–#348 (5 entries written by *other concurrent sessions* that did not run the backfill after their `memory_write` calls). Five of eight were from sessions that simply didn't know the backfill step existed — the gap is silent and easy to forget, which is why it needs skill-level enforcement, not a manual habit.

**Why no auto-embed hook:** an auto-executing PostToolUse hook that spawns `backfill.js` after each PMD write was considered and rejected (2026-05-16) — the auto-mode classifier correctly flags an auto-exec-background-on-tool-use hook as high-blast-radius self-modification, it only covers one repo's CC sessions (not MCP-server-side writes or other harnesses), and a detached `node` per write is heavier than the problem. The chosen fix is **skill-level enforcement at the two points that write PMD**:

- **`session-retro` (laptop, interactive) — Step 5.5 (MANDATORY).** After Step 5's `memory_write_eval` OR after a §3 promotion synced a new lesson, run the backfill inline (laptop Ollama via Tailscale `http://homeserver:11434`). The skill body has the exact command + precondition check + verify. This is safe inline because session-retro has no Stop-hook-final-action constraint.
- **`post-task-retro` (Junior, daemon) — CANNOT run inline.** Its `memory_write_eval` MUST be the absolute final action before exit (the `junior/*` Stop hook matches a fresh retro `source_ref` within 30 min; a post-eval backfill would violate "exit immediately" AND risk the Junior watchdog). So the Junior-side DB (`/srv/brehon-fork/.project-memory/memory.db`) is covered by the **weekly-review safety net** instead.
- **`weekly-review` (Junior, Sunday 02:00 UTC) — Step 1b (safety net).** Backfills the daemon-side PMD DB after the prune step. Locates `backfill.js` dynamically (the daemon MCP install path differs from the laptop's), uses localhost Ollama, non-fatal on failure (logs + continues; surfaces "backfill.js not found" / "Ollama down" as a week-over-week-degradation finding).

**How to apply:**

- **After ANY `memory_write_eval` / `memory_write` in an interactive laptop session** (not just via session-retro): if you wrote PMD, run the backfill before considering the work done:
  ```bash
  OLLAMA_URL=http://homeserver:11434 PROJECT_MEMORY_DB=.project-memory/memory.db \
  PROJECT_ROOT=C:/Users/barri/Developer/brehon-fork \
    node C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/backfill.js --verbose
  ```
  Idempotent on `(memory_id, model)` — only unembedded rows processed; the 250+ already-embedded skip. ~0.4 rows/s over Tailscale.
- **After `scripts/sync-lessons-to-pmd.sh`** (bulk lesson import): the sync adds text-only rows; the same backfill embeds them. This pair (sync → backfill) is one operation, never just the sync. (`pmd-search-strategy.md` already documents this; this lesson is the why + the enforcement points.)
- **Precondition check first** (the silent-degrade trap): `curl -s -m5 http://homeserver:11434/api/tags` (laptop) or `http://localhost:11434/api/tags` (daemon). Unreachable → rows stay FTS5-only; note it so the user knows semantic search is degraded until the next successful backfill.
- **Verify 0 missing after:** `python -c "import sqlite3;c=sqlite3.connect('.project-memory/memory.db');print('MISSING:',c.execute('SELECT COUNT(*) FROM memories m WHERE NOT EXISTS (SELECT 1 FROM memory_vectors v WHERE v.memory_id=m.id)').fetchone()[0])"`.

**Symptom to recognise:** `memory_search_hybrid` returns only exact-keyword matches for a topic you know was recently written (a synonym query misses it); a `SELECT COUNT(*)` shows `memories > memory_vectors`. The gap is silent — absence of semantic hits is the only signal. If a recently-written lesson/eval isn't surfacing for a paraphrased query, suspect unembedded rows before assuming the content is wrong.

**Generalises to:** any retrieval system where the write path and the index/embed path are decoupled and the embed step is a separate manual/scheduled job. The mitigation pattern — enforce the embed step at the skills/automation that own the write, plus a periodic safety-net sweep for writes that bypass those skills — applies whenever write-time indexing isn't atomic.

**When the structural fix lands:** if the MCP server is ever patched to embed at write-time (in `dist/index.js`'s `memory_write*` handlers), this lesson + the Step 5.5 / Step 1b enforcement become redundant — delete them then. Until then they are load-bearing.

**Companion lessons / docs:**
- `.claude/rules/pmd-search-strategy.md` — the "re-run backfill after bulk lesson import" procedure + the homeserver:11434 Ollama wiring + the silent-FTS5-degrade troubleshooting.
- `feedback_pmd_two_memory_systems_distinction.md` — the broader ".md auto-loaded vs SQLite query-only" distinction (this lesson is specifically about the SQLite side's embed gap).
- `feedback_junior_pmd_write_convention.md` — the Junior-side `LESSON:` trailer / `memory_write` convention whose rows this backfill embeds.

**Where this bit:** `.claude/PRPs/reports/session-retro-2026-05-16-v1-ad-e-gate-bmcut-finalize-recovery.md` (noted the 5-from-other-sessions unembedded gap) + the manual catch-up backfill run during that session.
