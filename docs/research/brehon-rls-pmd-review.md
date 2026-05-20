# Brehon RLS/PMD Review — Harness, Retro Flow, and Autonomy Readiness

**Repo:** `barrie-cork/lemmy@governance-v0`
**Scope:** `.claude/` harness — RLS (Recursive Learning System) and PMD (Project Memory Database) pipeline
**Goal frame:** RLS is the trust-building substrate for the autonomy journey. Hardening = closing the gaps between what the system says it does and what it provably does, so that trust scales without supervision.

---

## §1 Layered architecture — the five retro tiers

```
┌──────────────────────────────────────────────────────────────────────────┐
│  Tier 5  HARVEST          retro-harvest skill (ad-hoc, read-only)         │
│          ────────         scans 52 retros → ~48 unchecked proposals       │
│                           e.g. cycle_count ≥ 3 catch-fire rule (missed)   │
│                                  ▲                                        │
│  Tier 4  WEEKLY           weekly-review skill (Sun 02:00 UTC)             │
│          ────────         Step 1b backfill safety net + lesson promotion  │
│                                  ▲                                        │
│  Tier 3  SUB-PHASE        phase-retro-gate (per Anchor close)             │
│          ─────────        consolidates session retros into phase signal   │
│                                  ▲                                        │
│  Tier 2  SESSION          session-retro skill (post-Stop)                 │
│          ────────         Step 0.5 auto-phase detect → template → eval    │
│                                  ▲                                        │
│  Tier 1  PER-TASK         post-task-retro skill (in-session)              │
│          ─────────        retro-check.sh Stop hook enforcement            │
│                                  ▲                                        │
│  Tier 0  OBSERVATION      LESSON: trailers · observation-capture.sh JSONL │
│          ──────────       user_gate_history.notes · auto-state mutations  │
└──────────────────────────────────────────────────────────────────────────┘
                                  ▼
        ┌─────────────────────────────────────────────────────────┐
        │  PMD — two-system invariant                              │
        │                                                          │
        │  System 1 (auto-loaded):  ~/.claude/projects/.../memory/ │
        │                           markdown files, read at start  │
        │  System 2 (query-only):   .project-memory/memory.db      │
        │                           SQLite + sqlite-vec, hybrid    │
        │                           FTS5 + embedding search        │
        │                                                          │
        │  Canonical path (cross-lane, absolute):                  │
        │  C:/Users/barri/Developer/brehon-fork/.project-memory/   │
        │  memory.db                                               │
        └─────────────────────────────────────────────────────────┘
                                  ▼
        ┌─────────────────────────────────────────────────────────┐
        │  .claude/lessons/ — promoted patterns (recurrence ≥ 2)   │
        │  PATTERNS.md / Learned Patterns — canonical (≥ 3)        │
        └─────────────────────────────────────────────────────────┘
```

Five tiers, one substrate (PMD), one promotion ladder (note → lesson → canonical pattern). The architecture is sound. The gaps are all at the **edges** — between tiers, between systems, between spec and enforcement.

---

## §2 Retro flow trace — observation to canonical pattern

A single learning travels through eight stages. Each arrow below is a real handoff in your harness; the **bold** arrows are where the pipeline is currently load-bearing on agent honesty rather than enforcement.

```
1. OBSERVATION
   ├─ LESSON: trailer (in-message, optional, agent discipline)
   ├─ observation-capture.sh → ~/.cache/tw-observations/*.jsonl (shadow, no consumer)
   ├─ user_gate_history.notes (auto-state mutation)
   └─ four-role-signals (auto-detected from transcript)
            │
            ▼  ← retro-check.sh Stop hook (branch-scoped on junior/*, 30 min;
                 time-window 60 min on others; fail-open after 3 attempts)
2. EPHEMERAL RETRO FILE
   .claude/retros/<session-id>.md
   ├─ H2 headers: "What surprised us / What to change / What to carry forward"
   ├─ Step 0.5 auto-phase detector (template-optional)
   ├─ Step 5 evals.json scoring (typical 0.60-0.75, read-only max 0.70)
   └─ Step 5.5 memory_write_eval ← REQUIRED final action
            │
            ▼
3. PMD WRITE (System 2)
   memory_write_eval → memories table
   ├─ FTS5 index: immediate
   └─ embedding: ⚠ NOT WRITTEN AT WRITE-TIME (see §3 and §4.2)
            │
            ▼  ← backfill.js (manual or weekly safety net)
4. EMBEDDING BACKFILL
   Verify-missing query → embed via nomic-embed-text → memory_vectors table
            │
            ▼
5. CROSS-SESSION SEARCH
   memory_search_hybrid (FTS5 + vector, post-2026-05-16 working)
   pmd-search-strategy rule governs query construction
            │
            ▼
6. PROMOTION (recurrence-driven)
   ├─ 1 occurrence: session note only
   ├─ 2 occurrences: .claude/lessons/<name>.md
   └─ 3+ occurrences: PATTERNS.md / Learned Patterns
            │
            ▼
7. SYNC TO PMD (System 1 surface)
   scripts/sync-lessons-to-pmd.sh --db "$CANON_PMD" --strict
   ⚠ manual invocation; --strict closes the lane-local stranding risk
            │
            ▼
8. NEXT SESSION INJECTION
   SessionStart auto-load (System 1 markdown) +
   inject-dq-state.sh UserPromptSubmit hook (DQ/hopper context)
```

**Load-bearing on honesty (not enforcement):**
- Step 1 → 2: agents must write LESSON: trailers and honest auto-state notes
- Step 2 → 3: Stop hook enforces *that* a retro is written, not that it's calibrated honestly
- Step 3 → 4: write-time embedding gap means FTS5-only search for the window between write and next backfill (worst case 7 days for Junior)
- Step 6 → 7: sync to PMD is a script someone has to remember to run
- Step 7 → 8: SessionStart canonical-PMD guard is **spec'd but not implemented** (the single biggest gap)

---

## §3 PMD write/embed/search pipeline — the two-system distinction

### Two systems, one source of truth

| | System 1 (auto-loaded markdown) | System 2 (queryable database) |
|---|---|---|
| **Path** | `~/.claude/projects/C--Users-barri-Developer-brehon-fork/memory/` | `.project-memory/memory.db` (canonical: `C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db`) |
| **Format** | Markdown files | SQLite + `sqlite-vec` extension |
| **Loaded** | Automatically at SessionStart | Queried on demand via MCP tools |
| **Write path** | `sync-lessons-to-pmd.sh` (manual) | `memory_write_eval` (MCP, every retro) |
| **Read path** | Cat into context at start | `memory_search_hybrid` (FTS5 + vector) |
| **Cross-lane?** | Per-Claude-project, isolated | **Shared via canonical absolute path** |

The two-systems lesson is foundational. Every PMD-meta lesson on disk ultimately reduces to: *don't conflate auto-loaded markdown with the queryable DB, and don't let any lane resolve `.project-memory/memory.db` relative to its own worktree.*

### Canonical path invariant

The single most important invariant in your harness:

```
PROJECT_MEMORY_DB = C:/Users/barri/Developer/brehon-fork/.project-memory/memory.db
```

Absolute. Cross-lane. Never relative. The 2026-05-18 v1-ship-1 incident (27+ false Stop-hook blocks, 21 stranded retros) was a single-symptom failure of this invariant. Every guard in your harness — `retro-check.sh`, `sync-lessons-to-pmd.sh --strict`, the SessionStart-guard *spec* — exists because of this one incident.

### No write-time embedding

Confirmed by source inspection of the MCP server `dist/index.js`: `memory_write_eval` writes the row and the FTS5 index. **It does not embed.** Until `backfill.js` runs, the row is invisible to vector search.

This costs you the +62.7% Recall@10 advantage of hybrid search for the window between write and next backfill. The current mitigations are:

1. session-retro Step 5.5 instructs the retro author to invoke backfill inline (laptop path)
2. weekly-review Step 1b is the safety net (Sunday 02:00 UTC) — worst case 7-day window for Junior writes which cannot inline-backfill (Stop hook constraint)

If MCP is patched to embed at write time, you can delete Step 5.5, Step 1b can become aspirational, and the entire embedding-coverage class of lessons becomes obsolete. **Highest single point of leverage in the whole pipeline.**

### Branch-scoped enforcement

`retro-check.sh` resolves the DB via `git rev-parse --git-common-dir` → canonical, then queries:

```sql
-- Junior lanes (branch starts with junior/)
SELECT … WHERE source_ref = ? AND timestamp > now - 30min

-- Other lanes
SELECT … WHERE timestamp > now - 60min  -- recently bumped from 15
```

Junior is the strict regime because Junior is the autonomy frontier; the other lanes are calibrated for a human-in-the-loop reality. Fail-open after 3 attempts is the pressure-valve for genuine loops. It is also (see §4.7) the single largest enforcement bypass surface.

---

## §4 Hardening opportunities

Ten items, in priority order. Each is grounded in a specific lesson, hook, or skill on disk.

### 4.1 SessionStart canonical-PMD guard — spec'd, not implemented

Status: **PENDING per `feedback_mcp_canonical_pmd_path_enforce_at_session_start.md`**.

The lesson tracks `.claude/hooks/pmd-canonical-guard.sh` + per-worktree `.claude/settings.local.json` wiring via the lane-bootstrap checklist. WARN-not-FAIL semantics. The lesson's own `fix status: PENDING` is, per the lesson-must-pair-with-fix discipline, the system's confession that this gap is still live.

This is the #1 hardening because: (a) `settings.json` SessionStart currently runs `pre-phase-audit.sh` only, (b) the 2026-05-18 incident is the historical proof that path resolution at session start is the failure mode you cannot afford, and (c) every other guard downstream depends on the canonical path being correct from turn 1.

**Fix:** implement `pmd-canonical-guard.sh` as drafted, wire it into `settings.json` SessionStart array *before* `pre-phase-audit.sh`, and bake it into the lane-bootstrap checklist so new worktrees inherit the wiring via `.claude/settings.local.json`.

### 4.2 No write-time embedding in MCP `dist/index.js`

Verified absent in source. Until patched, every retro write leaves a vector-shaped hole until `backfill.js` runs.

**Fix:** patch MCP server. Insert embedding call in `memory_write_eval` handler after the FTS5 insert, before commit. Fall back gracefully if Ollama unreachable (write the memory, log the miss, let weekly-review catch it). This collapses two lessons (Step 5.5, Step 1b) and removes the Junior 7-day window entirely.

### 4.3 `.mcp.json` documentary guard never re-read

The `_comment_pmd_cross_lane` marker in `.mcp.json.example` is bootstrap-and-forget. Same anti-pattern as 4.1: a documentary check that nothing verifies after the fact. Because `.mcp.json` is gitignored and per-worktree, drift between worktrees is a real risk.

**Fix:** SessionStart hook (or extension of the canonical-PMD guard from 4.1) that reads `.mcp.json`, confirms the comment marker is intact, and confirms the canonical path matches. WARN level — this is hygiene, not a stop condition.

### 4.4 Junior post-task-retro cannot inline-backfill

The Stop hook contract requires `memory_write_eval` as the final action. Junior, running as a daemon, cannot then shell out to `backfill.js`. Weekly-review (Sunday 02:00 UTC) is the only embedding enforcement point for Junior writes.

Worst case: a Monday Junior retro is FTS5-only for ~6.5 days. During that window, hybrid search regresses to keyword-only on the most recent (and most predictive) memories.

**Fix:** preferred is 4.2 (write-time embedding) — collapses this gap entirely. Failing that: a PostToolUse hook on `memory_write_eval` that fires `backfill.js` non-blockingly. Junior's watchdog (~6 min no-stdout = SIGKILL) means it must be background-detached.

### 4.5 `sync-lessons-to-pmd.sh` to PMD is manual

The `--strict` flag (exit 3 on lane-local resolution) closed the stranding risk, but only when someone remembers to invoke it. The lesson promotion ladder (note → lesson → canonical) breaks at step 7 if no one runs the sync.

**Fix:** add to weekly-review skill as a mandatory Step 2 (after backfill safety net). Lesser fix: PostToolUse hook on any `Write|Edit` to `.claude/lessons/*.md` that queues a sync for the next Stop event. Best fix: both — weekly is the safety net, post-edit is the fast path.

### 4.6 retro-harvest is read-only, ad-hoc

The 2026-05-16 analysis found ~48 unchecked proposals across 52 retro files. Among them, a `cycle_count ≥ 3` catch-fire rule was proposed 7 days before it would have prevented a ~123-min loss. This is the harvest tier's job *not* getting done.

**Fix:** fold retro-harvest into weekly-review as a mandatory Step 3. Every Sunday 02:00 UTC: backfill safety net → sync lessons → harvest proposals. Output is a single weekly artifact: `.claude/harvest/<iso-week>.md` with proposed lessons and rule promotions. Manual review thereafter, but the surfacing is automatic.

### 4.7 retro-check.sh fail-open after 3 attempts

By design for true loops. The bypass surface: a determined-to-bypass agent gets a free third pass with no retro written. This is acceptable for a human-in-the-loop regime where the human catches it. For autonomy, it's a known hole.

**Fix:** don't change the fail-open behaviour (loops are real). Add: every fail-open emits a `governance-log-entry` of kind `retro_bypass` with the session id, attempt count, and last assistant message hash. Junior cannot bypass without leaving a trail. Weekly-review Step 4 reads these and flags.

### 4.8 Five PMD-meta lessons should promote to a top-level rule

The invariants — canonical path, two-systems, backfill, LESSON-trailer, SessionStart guard — are not principle-style guidance. They are hard system invariants. Scattering them as lesson files mixes two different categories of memory (heuristics vs. invariants).

**Fix:** create `.claude/rules/pmd-invariants.md` consolidating the five as numbered, non-negotiable statements. Keep the lesson files for forensics (they are the *evidence trail*), but the rule file is the authoritative reference. This aligns with the principles-not-rules lesson's spirit: *some things are rules, and they should be named as such.*

### 4.9 Auto-phase 10-category reliability section is template-optional

session-retro Step 0.5 detects auto-phase via transcript or auto-state mutation. The detector relies on the retro author's honesty about whether the session was auto-phase.

**Fix:** check auto-state file mtime against session window. If mutated within the session window, the auto-phase section is **mandatory**, not optional. Step 0.5 becomes a hard gate, not a heuristic.

### 4.10 observation-capture.sh JSONL has zero consumers

Shadow mode, 7-day precision measurement (PMD #918 Phase 4). Four detectors: deploy-start, service-restart, server-shell, server-file-write. Currently 0% downstream surface.

**Fix:** add a Step 4 to weekly-review: consume the last 7 days of JSONL, compute precision against retros, decide promote-to-active or extend-shadow. Until something reads them, the events are write-only — and a system that writes signal nothing reads is just noise generation.

---

## §5 Autonomy-readiness criteria

What must hold for trust to scale to full autonomy? Five conditions, derived from the gaps above. Treat these as the calibration matrix you measure the harness against quarterly.

### 5.1 Calibration honesty
Evaluation scores stay in the 0.60-0.75 band; read-only sessions cap at 0.70; no 1.0s. The evaluation-calibration rule is the anti-inflation guard. **Autonomy depends on Junior reporting its own failures accurately.** Drift here is undetectable by the harness itself — it requires either retro-harvest catching the pattern or a human noticing the band shift. Add a weekly-review check: histogram of last 50 scores; alert if median > 0.80.

### 5.2 No-bypass enforcement
retro-check.sh's fail-open is bounded (item 4.7). Every bypass leaves a governance log entry. The bypass rate is a tracked metric. Autonomy = the bypass rate is monotonically decreasing.

### 5.3 Embedding coverage
The verify-missing query returns 0 within 24 hours of any write. Right now this depends on weekly-review (worst case 7 days). With 4.2 implemented, it's near-real-time. **Embedding coverage is the proxy for "the system can find what it knows".** Below 100% rolling-24h, hybrid search degrades silently.

### 5.4 Harvest cadence
The retro-harvest tier runs. Currently ad-hoc. Folded into weekly-review (4.6), it becomes guaranteed. Unchecked proposals do not accumulate beyond one week. The 7-days-before-loss anti-pattern (the catch-fire rule) does not recur.

### 5.5 Retro-not-report discipline
The three canonical H2 headers stay: "What surprised us / What to change / What to carry forward." The retro-not-report lesson is foundational — reports describe what happened, retros change what happens next. Autonomy is the second mode at scale. A retro that reads like a report is a tier-2 failure that nothing downstream will fix.

---

## §6 Top 5 highest-leverage hardenings

Ranked by leverage = (downstream lessons collapsed) × (autonomy-readiness criterion unblocked) ÷ (implementation cost).

### 1. Patch MCP `dist/index.js` to embed at write time (4.2)
Single change. Collapses Step 5.5 in session-retro, Step 1b in weekly-review, the Junior 7-day window (4.4), and the entire embedding-coverage autonomy criterion (5.3) becomes free. **The highest-leverage change in the system, full stop.** Estimated cost: one MCP patch, one test against the verify-missing query.

### 2. Implement the SessionStart canonical-PMD guard (4.1)
Spec'd, drafted, status PENDING. The single most important invariant (canonical path) is currently guarded *after* it can fail. Implementing this lifts the guard upstream of every other check. Closes the historical incident class (v1-ship-1). Estimated cost: one shell script (drafted), one settings.json edit per worktree, one bootstrap-checklist update.

### 3. Promote the five PMD-meta invariants to `.claude/rules/pmd-invariants.md` (4.8)
Reclassifies the most-violated invariants from "lesson" (heuristic, recurrence-based) to "rule" (non-negotiable). Removes the category confusion that makes the lessons feel optional. Estimated cost: one file, ~200 lines. The lessons stay as the evidence trail; the rule is the authoritative summary.

### 4. Fold retro-harvest into weekly-review as mandatory Step 3 (4.6)
The 48 unchecked proposals are the evidence that the harvest tier is not running. Making it part of the weekly cadence guarantees it runs at the same rhythm as backfill and sync. Closes autonomy criterion 5.4. Estimated cost: one skill edit, one harvest output directory.

### 5. Governance-log every retro-check.sh fail-open (4.7)
Doesn't change the fail-open behaviour (loops are still real). Adds a tracked trail so the bypass rate is measurable. Closes autonomy criterion 5.2 — you cannot improve what you cannot count. Estimated cost: one shell function in retro-check.sh, one governance-log-entry kind registered.

---

## Appendix — what's already working well

For balance. The harness is not in trouble; it has specific, identifiable gaps. The following are evidence of mature design:

- **Two-systems lesson is fully internalised** across rules, skills, and hooks. Every tool that touches PMD knows which system it's talking to.
- **retro-check.sh branch-scoped logic** is precisely right: strict on Junior (autonomy frontier), permissive elsewhere (human-in-loop).
- **Hybrid search post-2026-05-16** measurably outperforms FTS5-only by 62.7% Recall@10. The pmd-search-strategy rule reflects this.
- **Stop hook order** (prp-ralph-stop.sh THEN retro-check.sh) is correct: Ralph loop continuation must check before retro enforcement, otherwise Ralph blocks itself.
- **Calibration anti-inflation** (0.60-0.75 band, no 1.0s, read-only cap at 0.70) is the cultural foundation of the whole RLS. Without this, scores would inflate, trust would erode, autonomy would be impossible.
- **The `--strict` flag on sync-lessons-to-pmd.sh** closed the lane-local stranding class. Demonstrates the harness can absorb and fix its own incidents.
- **observation-capture.sh in shadow mode** is the right way to introduce new detectors — measure first, promote later. The only gap is that nothing measures (4.10).

The architecture is sound. The execution gaps are concentrated at the edges where enforcement gives way to discipline. Closing those edges is what turns this from a careful harness into an autonomy substrate.
