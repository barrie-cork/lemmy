# PMD pheromone fixes — plan (A → C → B)

## 0. Status + priority

**PRIORITY.** Authored 2026-06-11 (Opus advisor session). Blocks the deferred
"port pheromone system to the Mac/canonical binary" work — do NOT port until
B is fixed, or the runaway-loop bug propagates to every stdio-mode repo
(phd-vault et al.).

Companion artifacts:
- `.claude/PRPs/specs/mcp-pmd-read-pheromone.md` — the original read-pheromone
  contract (the design these fixes amend).
- `.claude/PRPs/specs/pmd-pheromone-relevance-labeling.md` — companion spec.

## 1. Why this exists (the retro-miss incident, 2026-06-11)

The `retro-check.sh` Stop hook repeatedly fired "retro not found" against a
retro (#936) that **existed and was correct**. Root-caused to a four-link chain:

1. The retro existed (written 16:23) — always a false negative.
2. The hook used `memory_search("Task retro", limit 20)` — a **relevance**
   query — to answer a **recency** question. 521 qa-result rows match
   "Task retro"; limit 20 returns the top 4% by ranking, where recency is not
   a factor.
3. The FTS path orders `importance DESC, pheromone_boost DESC, updated_at DESC`.
   A fresh retro has `read_count=0 → boost=0`, so it sorts **last in its
   importance band**, behind every previously-read retro. Zero chance of being
   in the top 20.
4. Each `memory_search` call **deposits** pheromone on the 20 rows it returns,
   inflating their read_count (observed: 92–113, all `last_read=16:28` today
   from the debug runs), making them progressively more dominant and burying
   new retros deeper every run — the **runaway positive-feedback loop** the
   spec's `τ_max` cap is designed to prevent, but which is **hybrid-path-only**.

The hook fix (switch to `memory_get_recent` — recency-ordered, non-depositing)
is the proximate fix (issue A). C cleans the data damage; B closes the
structural gap so the loop can't recur on any FTS consumer.

## 2. Issue A — commit the hook fix (DONE in working tree, needs commit)

- **What:** `.claude/hooks/retro-check.sh` HTTP block switched from
  `memory_search` (relevance, depositing) to `memory_get_recent` (recency,
  non-depositing). Verified: hook exits 0 with the live retro present.
- **State:** edited in this worktree, **uncommitted**.
- **Action:** commit on `governance-v0` (meta-work, direct-commit per
  `phase-branch.md`). Subject: `fix(hooks): retro-check uses memory_get_recent
  (recency, not BM25-relevance)`.
- **Validation:** `bash .claude/hooks/retro-check.sh; echo $?` → `0`.

## 3. Issue C — clean the debug-run signal pollution

- **What:** ~21 memories were read by this session's hook-debugging FTS
  searches; ~20 old retros had read_count inflated to 92–113 with
  `last_read_at ≥ 2026-06-11 16:00`. This is corrupted pheromone signal —
  those rows look "hot" because they were queried during debugging, not from
  genuine relevance use.
- **Constraint:** mutates the **live** brehon-fork PMD
  (`/srv/brehon-fork/.project-memory/memory.db`, homeserver). **Back up first**
  (`pmd-snapshot.service` runs hourly, but take an explicit `.backup` before).
- **Action (proposed):** the precise per-row deposit count from this session is
  recoverable from `read_events` (the debug events all carry today's timestamps
  and `tool='fts'`). Decrement each affected memory's `read_count` by its count
  of session-debug `read_events`, and reset `last_read_at` to the most recent
  *non-debug* read (or NULL if none). Then delete the debug `read_events` rows.
  - Identify: `SELECT memory_id, COUNT(*) FROM read_events WHERE
    <session-debug window> GROUP BY memory_id`.
  - Simpler conservative alternative if per-row decrement is fiddly: accept the
    inflation as bounded noise (the 30-day half-life decays it; the hook no
    longer feeds it). Decide at execution time.
- **Validation:** after cleanup, `SELECT MAX(read_count) FROM memories` returns
  a value consistent with pre-session state; no row shows `last_read_at` inside
  the debug window unless it had a legitimate concurrent read.

## 4. Issue B — cap the FTS-path pheromone boost (the structural fix)

- **What:** the `τ_max` bound that prevents runaway convergence exists only on
  the **hybrid RRF path** (`PHEROMONE_WEIGHT = 0.5 / RRF_K` additive nudge).
  The **FTS path** (`memory_search`, `ftsFallback`, context lookups) uses
  `PHEROMONE_SQL_BOOST` as an **uncapped** secondary `ORDER BY` key, so it can
  fully reorder within an importance band and any repeated same-query consumer
  self-reinforces without bound.
- **Options (pick at spec-amend time):**
  1. **Cap the SQL boost** — wrap `PHEROMONE_SQL_BOOST` in a `MIN(boost, τ_max)`
     so a hot memory cannot outrank a cold one by more than a bounded margin
     within the band. Cleanest analog to the hybrid cap.
  2. **Demote pheromone below recency** — reorder the FTS `ORDER BY` to
     `importance DESC, updated_at DESC, pheromone DESC` (or insert `created_at
     DESC` ahead of pheromone) so freshness wins ties and pheromone only breaks
     true freshness-ties. Changes ranking semantics more.
  3. **Both** — cap *and* keep pheromone as the lowest tie-breaker.
- **Recommendation:** Option 1 (bounded cap) — minimal semantic change,
  directly mirrors the hybrid path's existing τ_max, satisfies spec invariant
  #1/#2 symmetry. Amend `mcp-pmd-read-pheromone.md` §8 invariants to state the
  cap applies to **both** paths.
- **Where:** homeserver `src/tools/search.ts` → `npm run build` → restart
  `project-memory-http`. Now version-controlled (commit `776e172`), so commit
  the change.
- **Validation (extend spec §9):**
  1. Relevance/recency-dominance proof: a cold fresh retro is NOT buried below
     N hot old retros of equal importance beyond the cap margin.
  2. Loop-bound proof: running the same FTS query K times does not monotonically
     increase the top-result's rank dominance without bound.
  3. Re-run the original retro-miss scenario against `memory_search` (not just
     get_recent) and confirm a fresh retro surfaces within a reasonable window.

## 5. Sequencing + scope guard

- **Now (cheap, stops + cleans the bleeding):** A, then C.
- **Deliberate (touches shipped design):** B — author as a §8 amendment to
  `mcp-pmd-read-pheromone.md` with its own validation, not a quick patch.
- **Deferred until B ships:** port the pheromone system to the Mac/canonical
  binary (phd-vault et al.). Porting before B propagates the runaway loop.

## 6. See also

- `.claude/PRPs/specs/mcp-pmd-read-pheromone.md` §2 (ACO failure modes), §7
  (the two ranking paths), §8 (invariants — to be amended by B), §9 (validation).
- `.claude/rules/pmd-invariants.md` — substrate invariants.
- `.claude/hooks/retro-check.sh` — the fixed consumer (issue A).
