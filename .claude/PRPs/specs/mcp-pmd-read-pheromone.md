# MCP PMD read-pheromone (stigmergic recency/frequency boost) — patch contract

## 1. Authority + provenance

User-initiated 2026-05-31 (Opus advisor session). The design question:
*"Could lessons from nature such as how ants communicate via pheromones be
used to enhance the PM system, so memories recently/often read during a
period are elevated and those not read lately descend from the surface?"*

The answer is yes, and the minimal form of it is small. This doc is the
contract; the patch ships in a separate `MCPs/project-memory-mcp` PR (same
deferral model as its companion `mcp-write-time-embedding.md` — spec first,
patch separately).

**Companion spec:** `.claude/PRPs/specs/mcp-write-time-embedding.md` — the
two patches touch the same crate and the same two files (`src/db.ts`,
`src/tools/search.ts` → compiled `dist/`). Ship them in one PR if convenient;
they do not conflict (one adds the embed step on write, this adds the
pheromone step on read).

**Ground-truth source read (2026-05-31):** `src/db.ts` (schema) +
`src/tools/search.ts` (all three search handlers). Findings that make this a
*minimal* change rather than a rewrite are in §3.

## 2. The biological analogy, stated precisely (and where it stops)

Ant foraging is **stigmergy**: ants deposit pheromone on paths they walk;
pheromone **evaporates** over time; a stronger trail attracts more ants
(positive feedback); evaporation + a diversity floor stop the colony from
permanently locking onto the first path found. Relevance self-organises from
use, with no central curator.

The mapping onto PMD:

| Ant colony | PMD |
|---|---|
| Pheromone deposit (walking a path) | A search **returns** a memory row → increment its read-count, stamp `last_read_at` |
| Evaporation over time | Exponential decay of the read-boost as `last_read_at` ages |
| Stronger trail → more ants (positive feedback) | A frequently-read memory ranks higher → surfaces more → read more |
| Evaporation floor / τ_min (anti-stagnation) | The boost decays toward **0**, never negative — a memory never sinks *below* its static `importance` tier; a stale memory returns to prominence the moment it is read again |
| Cold-start path at τ_min | A never-read memory has read_count=0 → boost=0 → ranks purely on RRF relevance + static importance (today's behaviour, unchanged) |

**Where the analogy stops — and why that matters for safety.** Two ACO
failure modes must NOT be imported:

1. **Runaway positive feedback / premature convergence.** In ACO this is
   contained by `τ_max` (MAX-MIN Ant System, Stützle & Hoos) and by
   evaporation. Here it is contained structurally: the pheromone boost is a
   **bounded tertiary term**, applied *within* an importance band, *after*
   RRF has already selected the relevant candidate pool. It cannot pull an
   irrelevant memory into the results — it can only reorder memories the
   search already judged relevant. The relevance gate (RRF) is upstream and
   untouched.
2. **Trail starvation of good-but-unwalked paths.** A genuinely useful lesson
   that nobody *searches for* never gets deposited on — pheromone rewards
   "what was read", not "what was useful". This is real and unsolved by
   decay alone. The existing mitigation is the **mandatory pre-queue
   `memory_search_hybrid` call** (`.claude/rules/advisor-orchestrator.md`
   §2.3) — the equivalent of forcing foragers onto all candidate paths each
   cycle. The pheromone spec does **not** weaken that discipline; importance
   stays the human-curated prior, pheromone only adjusts ordering of what
   gets surfaced. Stated plainly in §7 so no future reader mistakes
   pheromone for a replacement for curation.

## 3. Why this is a minimal change (ground-truth from the code)

Three facts from reading `src/tools/search.ts` + `src/db.ts`:

1. **Reads leave zero trace today.** All three handlers (`memory_search`,
   `memory_search_hybrid`, `memory_get_recent`) end with
   `ORDER BY m.importance DESC, m.updated_at DESC` (recent path:
   `created_at DESC`). `importance` is static (set at write, `CHECK BETWEEN
   1 AND 5`, never mutated). `updated_at` only advances on an explicit
   `memory_update`. **There is no deposit step at all** — the pheromone
   system is currently entirely absent, not merely weak.

2. **RRF already does the relevance ranking.** In `memory_search_hybrid` the
   final order is the Reciprocal-Rank-Fusion of the semantic + FTS pools
   (`addRrf`, `RRF_K=60`). The `importance DESC, updated_at DESC` ORDER BY is
   only used to seed the *candidate pools* (`POOL_SIZE=50`) and as the
   pure-FTS fallback order. **The pheromone term must not touch the RRF
   math** — it enriches the deposit (free, on read) and adds one term to the
   final-fetch ORDER BY. The relevance engine is left alone.

3. **There is exactly one place per handler where final rows are returned.**
   The deposit `UPDATE` runs once, on the already-computed result IDs, right
   before the rows are serialised. One statement. No new query, no new scan.

So the change is: **two columns + one UPDATE-on-read + one ORDER BY term.**
No background job, no cron, no table scan, no change to RRF, no change to the
write path, no change to embeddings.

## 4. Decay-on-read, NOT background evaporation (the load-bearing decision)

A literal evaporation model would run a job that periodically multiplies every
row's pheromone by `(1-ρ)`. **Reject this.** On a solo-dev SQLite store of
~600 rows with no scheduler, a background sweep is the wrong primitive — it
adds an ops surface (a cron that can silently die, exactly the failure class
of `feedback_pmd_backfill_after_write.md`) for no benefit.

Use **lazy decay-on-read** (the LRFU / Caffeine-cache "aging on access"
pattern): store the raw signals (`read_count`, `last_read_at`), and compute
the decayed boost **only at query time**, inside the ORDER BY. Evaporation is
implicit in the formula — a memory whose `last_read_at` is old yields a small
boost *the next time anyone queries*, without anything having run in between.
No state needs to be rewritten for time to pass.

This mirrors how HN/Reddit "hot" ranking works (the score is recomputed from
`age` at read time, nothing decays the stored votes) and how FSRS computes
retrievability `R = exp(-t/S)` on demand rather than ticking every card down.

## 5. Schema change (`src/db.ts`)

Add two columns to `memories` (both nullable / defaulted so the migration is a
no-op for existing rows — additive, forward-only, matches the DQ-v3 migration
discipline):

```sql
ALTER TABLE memories ADD COLUMN read_count INTEGER NOT NULL DEFAULT 0;
ALTER TABLE memories ADD COLUMN last_read_at TEXT;   -- NULL = never read
```

`getDb()` runs `SCHEMA_SQL` with `CREATE TABLE IF NOT EXISTS`, so for a
*fresh* DB add the columns to the `CREATE TABLE memories (...)` block directly.
For the *existing* 579-row DB, the two `ALTER TABLE` statements must run once —
guard them so they are idempotent (SQLite has no `ADD COLUMN IF NOT EXISTS`
pre-3.35; check `PRAGMA table_info(memories)` for the column name first, or
wrap each ALTER in a try/catch that swallows the "duplicate column name"
error). Put this in a small `runMigrations(db)` called from `getDb()` after
`db.exec(SCHEMA_SQL)`.

No new index is required for the read path (the UPDATE is by primary key). An
optional `idx_memories_last_read ON memories(last_read_at)` helps only if a
future "show me cold memories" audit query is added — defer until that query
exists (YAGNI).

## 6. Deposit step (the pheromone drop) — `src/tools/search.ts`

In **each** of `memory_search` and `memory_search_hybrid`, immediately before
returning the final `jsonResult(rows)`, fire one batched UPDATE over the IDs
already selected:

```ts
function depositReadPheromone(db: Database.Database, ids: number[]): void {
  if (ids.length === 0) return;
  const placeholders = ids.map(() => "?").join(",");
  // ids are integers sourced from prior DB queries — safe to bind
  db.prepare(
    `UPDATE memories
       SET read_count = read_count + 1,
           last_read_at = datetime('now')
     WHERE id IN (${placeholders})`
  ).run(...ids);
}
```

- In `memory_search_hybrid`: call `depositReadPheromone(db, ranked)` after the
  `ranked` array is computed (it is already the exact set of returned IDs).
- In `memory_search` (FTS) and `ftsFallback`: the handler does
  `SELECT m.* ... LIMIT ?` — capture the returned rows' `id`s and deposit on
  those.
- **`memory_get_recent`: do NOT deposit.** That tool is a chronological
  firehose (used by retro-harvest sweeps and audits), not a relevance query;
  depositing on it would let a single "recent dump" inflate the read-count of
  whatever happens to be newest, polluting the signal. Pheromone is deposited
  only by *relevance* reads (the two search tools). State this exclusion in
  the patch comment.

**Crucially — `updated_at` must NOT change on read.** Deposit touches only
`read_count` + `last_read_at`. Bumping `updated_at` on read would (a) corrupt
the "when was this memory last *edited*" semantic other code relies on and (b)
fire the `memories_au` FTS-sync trigger on every search — a needless
re-index storm. The two new columns are deliberately *outside* the FTS
trigger's column set, so updating them is trigger-free.

## 7. Consume step (the ranking boost) — the ORDER BY term

The decayed boost, computed in SQL at query time:

```
boost = AMP_CAP_min( ln(1 + read_count) )  ×  exp( -LAMBDA × days_since_last_read )
```

Two factors, matching the ant model: **frequency** (`ln(1+read_count)`,
log-damped so a runaway-read memory can't dominate — same shape as TinyLFU's
frequency damping) × **recency** (`exp(-λ·Δdays)`, the Ebbinghaus / FSRS
forgetting curve, the evaporation term).

Parameters (tune in the patch PR; these are the recommended starting values,
justified below):

- `LAMBDA = 0.0231` → half-life ≈ 30 days (`ln2 / 0.0231 ≈ 30`). Rationale: a
  Brehon sub-phase is ~3–7 days; 30-day half-life means a lesson stays
  "warm" across the ~4–6 sub-phases where it is plausibly still relevant,
  then fades. Matches the existing 30-day default `expires_at` window for
  qa-result rows — pheromone and expiry age on the same clock.
- Frequency damping is `ln(1+n)` (no separate cap needed at these scales; at
  read_count=20 the term is ~3.0, at 100 it is ~4.6 — naturally bounded).

**Where the boost is injected** differs by path, because the two paths rank
differently:

### 7a. `memory_search_hybrid` (the primary, RRF-ranked path)

The RRF score is the relevance signal and **must stay dominant**. Inject
pheromone as a small additive nudge to the RRF score *before* the final sort,
scaled so it can only reorder near-ties (memories the fusion already scored
within ~one rank of each other), never override a clear relevance winner:

```ts
// after addRrf(ftsIds); addRrf(semIds);
// fetch read_count + last_read_at for the candidate ids in `scores`
const PHEROMONE_WEIGHT = 0.5 / RRF_K; // ≈ one-rank's worth of RRF mass, max
for (const [id, rrf] of scores) {
  const b = pheromoneBoost(readCount[id], lastReadAt[id]); // in [0, ~1]
  scores.set(id, rrf + PHEROMONE_WEIGHT * b);
}
```

`pheromoneBoost` is normalised to roughly `[0,1]` (e.g. divide the raw boost
by an expected-max so a hot memory contributes ≈ `PHEROMONE_WEIGHT`, a cold
one ≈ 0). The cap (`0.5/RRF_K`) is the `τ_max` analog: it bounds the maximum
influence so pheromone tunes order *within* relevance, never against it.

### 7b. `memory_search` (FTS) and `ftsFallback`

These order by `importance DESC, updated_at DESC`. Add pheromone as the
tertiary tie-breaker **inside the importance band** (so it never lifts a
memory out of its static-importance tier — the evaporation-floor guarantee):

```sql
ORDER BY
  m.importance DESC,
  ( ln(1 + m.read_count) * exp( -:LAMBDA *
      ( julianday('now') - julianday(COALESCE(m.last_read_at, m.created_at)) )
  ) ) DESC,
  m.updated_at DESC
```

SQLite has `ln`, `exp`, `julianday` as built-ins (math functions since 3.35,
shipped with `better-sqlite3`). `COALESCE(last_read_at, created_at)` gives a
never-read memory a recency anchor at its creation date — so brand-new
memories aren't penalised as "infinitely stale", they decay from birth like a
fresh ant trail.

## 8. Invariants the patch MUST preserve

1. **RRF relevance stays the dominant signal in hybrid search.** Pheromone is
   capped at ≤ one rank's worth of RRF mass. A high-relevance cold memory
   still outranks a low-relevance hot one.
2. **A memory never sinks below its static `importance` tier** in the FTS
   paths. Pheromone is a within-band tie-breaker, not a cross-band term.
   (The evaporation-floor / τ_min guarantee — stale ≠ buried-forever.)
3. **Deposit never bumps `updated_at`** and never fires the FTS sync trigger.
4. **`memory_get_recent` deposits nothing** (chronological, not relevance).
5. **Cold-start unchanged:** read_count=0 ⇒ boost=0 ⇒ today's ranking exactly.
   The patch is a strict superset; with all memories cold it is a no-op.
6. **Graceful under clock skew / NULL:** `COALESCE(last_read_at, created_at)`
   and `DEFAULT 0` mean no row ever produces NaN/NULL in the ORDER BY.

## 9. Validation (post-patch, before merge)

1. **Migration no-op proof:** on a copy of the 579-row DB, run the patched
   `getDb()`; assert row count unchanged, two new columns present, all
   existing rows `read_count=0, last_read_at=NULL`.
2. **Deposit proof:** `memory_search_hybrid("LemmyError")` twice; assert the
   returned rows' `read_count` went 0→1→2 and `last_read_at` advanced.
3. **No-FTS-storm proof:** confirm deposit does NOT change `updated_at` and
   the `memories_au` trigger did not fire (FTS row unchanged) — read the
   `updated_at` before/after.
4. **Recency-reorder proof:** two memories with equal `importance`, both
   matching an FTS query; read one of them 5×; assert it now ranks above the
   other; then `LAMBDA`-age it (set `last_read_at` 90 days back via a test
   fixture) and assert it sinks back below — evaporation works.
5. **Relevance-dominance proof:** a clearly-more-relevant cold memory still
   outranks a barely-relevant hot one in hybrid search (pheromone did not
   override RRF).
6. **`get_recent` exclusion proof:** call `memory_get_recent(20)`; assert no
   `read_count` changed.

## 10. Deploy

`src/` edit → `npm run build` (`tsc` → `dist/`) → restart the HTTP PMD server
(currently PID 5900 on the laptop, `node dist/index.js` on `localhost:11435`).
The daemon reaches it over Tailscale unchanged. No `.mcp.json` change. The
one-time `ALTER TABLE` runs automatically on first `getDb()` after restart
(§5 idempotent migration).

## 11. Explicitly out of scope (do NOT gold-plate)

- No background evaporation job (§4 — lazy decay-on-read is the whole point).
- No new "show cold memories" audit tool / index (YAGNI until a consumer
  exists).
- No change to write-time behaviour, embeddings, or the RRF fusion math.
- No per-memory-type pheromone weighting, no decay-rate-by-importance, no
  user-tunable λ surface. One global λ, one global frequency damp. Add
  differentiation only if a retro shows a specific tier mis-ranking.
- No change to `expires_at` / pruning. Pheromone reorders the *living*
  corpus; the prune/expiry path is a separate concern (a future enhancement
  could let high read_count *defer* expiry — noted, not specced here).

## 12. See also

- `.claude/PRPs/specs/mcp-write-time-embedding.md` — companion patch, same
  crate, same files; co-ship if convenient.
- `.claude/rules/pmd-invariants.md` §3 (no write-time embedding) + §1 (HTTP
  topology) — substrate this patch sits on.
- `.claude/rules/advisor-orchestrator.md` §2.3 — the mandatory pre-queue
  `memory_search_hybrid` call that is the anti-starvation forager-discipline
  pheromone does NOT replace (§2 caveat 2).
- `.claude/lessons/feedback_pmd_backfill_after_write.md` — the cron-death
  failure class §4 deliberately avoids by choosing lazy decay-on-read.
- LRFU (Lee et al., "LRFU: A Spectrum of Policies that Subsumes the LRU and
  LFU Policies", IEEE ToC 2001) — the CRF combined recency/frequency metric
  this design is the SQL-native form of. W-TinyLFU (Caffeine) — the
  frequency-damping + aging-on-access pattern. FSRS / Ebbinghaus — the
  `exp(-t/S)` forgetting curve used as the evaporation term. MAX-MIN Ant
  System (Stützle & Hoos 2000) — the τ_min/τ_max bounding that §2/§8
  translate into "boost ≥ 0, capped at one rank of RRF".
```
