# MCP write-time embedding — patch contract

## 1. Authority

This spec is authorised by `.claude/PRPs/plans/v1-rls-r1.plan.md`
§10.7 (commit `f61e09792`) + `docs/research/brehon-rls-pmd-review.md`
§4.2 ("No write-time embedding" finding). Source of truth for the
embedding gap: `.claude/lessons/feedback_pmd_backfill_after_write.md`
(source inspection of `MCPs/project-memory-mcp/dist/index.js`
confirmed zero `embed`/`ollama`/`memory_vectors`/`generateEmbedding`
references in the `memory_write_eval` handler — verified 2026-05-16).

This doc is the contract; the patch ships in a separate
`MCPs/project-memory-mcp` PR (per PRECON-2 — user decision
2026-05-20; v1-rls-r1 ships the spec only).

## 2. Patch insertion point

**Target file:** `MCPs/project-memory-mcp/dist/index.js`

**Target handler:** `memory_write_eval` (and sibling handlers
`memory_write`, `memory_promote_to_file`, `memory_update` — all
follow the same pattern; all currently omit the embed step).

**Insertion position:** post-FTS5 insert, pre-commit (per RLS-PMD
review §4.2 + `feedback_pmd_backfill_after_write.md` source
inspection — the FTS5 row must land first; both inserts in the same
transaction).

**Expected patch shape (JS pseudocode):**

```js
// After: db.prepare('INSERT INTO memories ...').run(row)
// Before: return success response
if (process.env.OLLAMA_URL) {
  try {
    const vec = await generateEmbedding(content);
    // generateEmbedding: POST OLLAMA_URL/api/embeddings
    //   { model: 'nomic-embed-text', prompt: content }
    //   → response.embedding (768-dim float array)
    db.prepare(
      'INSERT OR REPLACE INTO memory_vectors (memory_id, vector) VALUES (?, ?)'
    ).run(memoryId, JSON.stringify(vec));
  } catch (_err) {
    process.stderr.write(
      `MCP write-time-embed miss: memory_id=${memoryId} reason=ollama-unreachable\n`
    );
    // fall through to success return — do NOT throw
  }
}
```

## 3. Graceful-fallback contract

When the Ollama embedding endpoint is unreachable (any network error,
timeout, or `OLLAMA_URL` unset):

1. **Write the row.** The FTS5 insert MUST succeed and the row
   returned to the caller as normal. Write-time embedding failure
   MUST NOT cause the MCP tool call to fail.
2. **Log one stderr WARN.** Exact format (no variations):
   ```
   MCP write-time-embed miss: memory_id=<id> reason=ollama-unreachable
   ```
3. **Return success to the caller.** The MCP tool call response is
   `success: true` as if the embed had not been attempted.
4. **Weekly-review Step 1b catches.** The safety net in
   `.claude/skills/weekly-review/SKILL.md` Step 1b runs `backfill.js`
   on Sunday cadence to embed rows that hit the fallback path. Until
   the patch ships, Step 1b is the primary embed path; post-patch it
   becomes the safety net for legacy unembedded rows.

## 4. Verification recipe

After applying the patch, verify correct behaviour with this 3-write
probe:

```bash
# Step 1: capture baseline counts before the probe
python3 -c "
import sqlite3
db = sqlite3.connect('.project-memory/memory.db')
rows = db.execute('SELECT COUNT(*) FROM memories').fetchone()[0]
vecs = db.execute('SELECT COUNT(*) FROM memory_vectors').fetchone()[0]
print(f'Baseline: {rows} rows, {vecs} vectors')
"

# Step 2: write 3 test entries via the MCP tool (memory_write_eval x3)
# (invoke via Claude Code MCP tool calls — not shown here)

# Step 3: verify vectors match immediately (before any backfill.js run)
python3 -c "
import sqlite3
db = sqlite3.connect('.project-memory/memory.db')
rows = db.execute('SELECT COUNT(*) FROM memories').fetchone()[0]
vecs = db.execute('SELECT COUNT(*) FROM memory_vectors').fetchone()[0]
print(f'After 3 writes: {rows} rows, {vecs} vectors')
assert rows == vecs, f'FAIL: {rows} rows vs {vecs} vectors — write-time embed gap remains'
print('PASS: write-time embedding is atomic with FTS5 insert')
"
```

**Expected:** the assertion does NOT fire — `SELECT COUNT(*) FROM
memory_vectors` equals `SELECT COUNT(*) FROM memories` immediately
after write, without any `backfill.js` invocation in between.

## 5. Downstream-collapse claim

When the patch ships and the §4 verification recipe passes, the
following clean-up becomes valid. Do NOT apply until verified.

1. **Delete** `.claude/skills/session-retro/SKILL.md` Step 5.5
   inline-backfill instruction ("After Step 5's `memory_write_eval`
   … run the backfill inline"). Post-patch, write-time embedding
   makes this step redundant; leaving it causes confusion about
   whether the backfill is still required.

2. **Demote** `.claude/skills/weekly-review/SKILL.md` Step 1b from
   "mandatory backfill sweep" to aspirational: change the step
   description to "safety net for legacy unembedded rows (rows
   written before the write-time embedding patch landed)". The
   weekly backfill becomes belt-and-braces for historical rows,
   not the primary embed path.

3. **The Junior 7-day window vanishes.** The RLS-PMD review §5.3
   autonomy criterion "hybrid-recall gap ≤ 7 days for Junior writes"
   becomes near-real-time: every `memory_write_eval` from a Junior
   `post-task-retro` is semantically searchable immediately after
   write, without waiting for the next Sunday weekly-review cycle.

## 6. Verification deferred

The actual `dist/index.js` patch ships in a separate
`MCPs/project-memory-mcp` PR. This doc is the contract that PR
implements.

Verification of the §4 recipe and the §5 downstream-collapse claim
is deferred to the `MCPs/project-memory-mcp` PR's own Definition of
Done. v1-rls-r1 ships the spec only (per PRECON-2 — user decision
2026-05-20).

Until the patch ships:
- `.claude/lessons/feedback_pmd_backfill_after_write.md` mitigations
  remain load-bearing: session-retro Step 5.5 inline backfill
  (laptop-only) + weekly-review Step 1b safety net (Junior).
- Do NOT apply the §5 downstream-collapse clean-up items.
- A future advisor reading this doc should treat §4 and §5 as
  forward-looking contracts, not verified behaviour.
