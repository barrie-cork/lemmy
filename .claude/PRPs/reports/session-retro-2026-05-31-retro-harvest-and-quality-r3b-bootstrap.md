# Session retro — 2026-05-31 — retro-harvest and quality-r3b bootstrap

**Harness:** claude-code
**Session window:** 2026-05-31 (morning) → 2026-05-31 (evening) (~full day session)
**Branch at start:** `f40dca07f` (`governance-v0`)
**Branch at end:** `d1fdd4775` (`governance-v0`)
**Files touched:** ~15 (lessons, hooks, decision-queue, briefs, roadmap, retro reports)
**Commits:** 14 (0 auto, 14 explicit)

## TL;DR

Two distinct work threads in one long session. First: closed v1-quality-r3 (bm-merge PR #169 → `a23ac216f`), repaired PMD HTTP-topology silent-degradation (DNS/hosts fix + lesson promotions + lesson-pmd-sync hook), authored type-state handler lesson, then bootstrapped v1-quality-r3b planning brief + handled a planner DQ mid-planning. Second: ran the first `/retro-harvest` sweep of the 21-day corpus — 112 open proposals triaged across 67 retros, yielding 7 Tier-1 actionable items (highest leverage: PreToolUse `git add` defence hook, missing model-effort lesson, harness-audit 3.3 ratio fix). The retro-harvest skill ran cleanly as a read-only sweep with Explore subagents — delegated extraction and 4 parallel triage batches effectively kept parent context small. Main carry-forward: retro-harvest is worth a cadence slot (fold into weekly-review or run every 2 weeks); the Tier-1 items need a dedicated short session before v1-quality-r3b impl starts.

---

## What surprised us

- **PMD HTTP-topology degradation was silent for ≥1 day** — DNS failure (homeserver resolved to wrong IP after network change) caused the PMD HTTP MCP to return timeouts, and the lesson-pmd-sync hook was silently swallowing errors without surfacing them. The failure mode looks like "PMD writes succeed" but nothing is actually stored. Fix was a hosts-entry + Restart-Service; the lesson (`feedback_pmd_retro_check_http_store_split.md` + corrected `feedback_finalize_merge_where_to_look_first.md`) was already on disk but silently wrong because it described the old SQLite path as the canonical store.

- **Planner raised a DQ mid-planning on v1-quality-r3b** — the brief stated option-a (SETTINGS-internal capture, zero callsite changes) as the fix shape, but the brief's own clarify-DQ from `/brehon-clarify` had already answered option-a. The planner apparently saw the brief's constraint section and the clarify-DQ as conflicting (brief said "zero callsite changes" but the DQ was written before the option-a/option-b split was codified). Resolution was trivial (DQ answered citing the clarify answer), but the friction point is real: when the same constraint appears both in a clarify-DQ answer AND in the brief §3 constraints, the planner may perceive a mismatch even when they say the same thing. First occurrence; note for next clarify-DQ cycle.

- **Retro-harvest `git log` date-oracle caveat** — all 17 phase-retros had distinct git-authorship dates (no clone-artifact mtime clash). The feared worst-case (all files same mtime from `git worktree add`) did not materialise, but the discipline of checking was confirmed worth the 30 seconds.

- **4 Tier-3 STALE items from a single retro (`session-retro-2026-05-26-pr-155-cr-triage-junior-479-fail.md`)** — all four boxes were already addressed in advisor-orchestrator.md within days of being written, yet the boxes remain unchecked. Confirms the "proposals written but never followed up" problem the harvest skill was built to close.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When the same constraint appears in both a clarify-DQ answer AND the brief §3/§4, add a one-line cross-ref: "per DQ <id>, option-a selected" in the brief constraints section | Prevents planner from reading brief+DQ as conflicting when they're consistent | minor | 1× this session; watch for 2nd in quality-r3b planning |
| 2 | Add a PMD HTTP reachability check at session start (curl `http://localhost:11435/mcp/health` or similar) to the advisor §1 polling-loop ritual | Surfaces silent PMD degradation within seconds of session-start, not after a full phase | minor | 3× cumulative (DNS SPOF, daemon-local store split, HTTP-topology repair) |
| 3 | Run retro-harvest on a 2-week cadence (every other weekly-review) — fold into weekly-review Step 6 as an optional sweep | Prevents the "112 open proposals accumulating while retro habit is healthy but harvest never runs" failure mode | minor | 1× this session (first harvest run) |
| 4 | After running retro-harvest, schedule a dedicated "Tier-1 action session" within 24 hours (or note it in the workflow-state file) | The harvest is only useful if the Tier-1 items are acted on; without explicit scheduling they drift to Tier-3 | minor | 1× this session; structural gap obvious |
| 5 | When authoring a bm-merge brief, verify that the planning brief for the NEXT sub-phase is already on trunk (or drafted) before closing — prevents the "bm-merge + bootstrap + new brief" being three separate context loads when it could be one | Reduces session-boundary overhead when two sub-phases are closely coupled | minor | 1× this session (quality-r3 close → quality-r3b bootstrap were a natural pair) |

## What to carry forward

- **Retro-harvest as a cadence item** — the sweep of 67 retros + 112 proposals took ~40 minutes of wall clock (Phase 1 Explore subagent + 4 parallel Phase 2 triage batches). The delegation to subagents kept parent context from blowing up on a 50-file read. Pattern: one extraction subagent → 4 parallel triage batches → synthesis in parent. Works cleanly; repeat.

- **Tier-3 box-ticking is cheap** — 23 STALE items identified; the action is just opening the retro file and ticking 23 checkboxes. Cost is <5 minutes. The psychological barrier was "I don't know which retros have stale boxes" — harvest removes that barrier. Do the box-ticking immediately after harvest, not as a separate scheduled task.

- **PMD HTTP health check on every lesson-write** — the lesson-pmd-sync PostToolUse hook now calls `memory_write` against `http://localhost:11435/mcp`, but it doesn't verify the write succeeded. For high-value lessons (type-state handler, finalize-merge order), a follow-up `memory_search_hybrid` spot-check costs 5 seconds and confirms the lesson is actually reachable.

- **DQ answer + brief constraint cross-reference** — when answering a clarify-DQ that resolves an option (A vs B), add "option-a selected: <brief §4 constraint X>" in the DQ answer text. This creates a forward pointer from the DQ to the brief, so the planner reading both sees they say the same thing.

- **Read-source-before-speccing discipline now in lessons** — `feedback_read_source_before_speccing.md` promoted from session-retro-2026-05-31-pmd-validation-pheromone-spec.md via `8a611db03`. Apply before any spec that claims "the handler does X" — verify the handler code actually does X first.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `bm-merge` (Junior dispatch) | 20 | 5 | low | Merge gate ran cleanly; 5 min for adr-compliance acknowledge bypass re-trigger |
| PMD HTTP topology repair | 0 | 35 | high | DNS silent failure — no signal, had to infer from MCP timeout pattern; now fixed with hosts entry |
| Lesson promotions (3 lessons from pmd-validation retro) | 15 | 0 | none | `8a611db03` batch: finalize-merge order + read-source-before-spec; clean |
| Type-state handler lesson authorship (`4e87daaa7`) | 10 | 0 | none | Straightforward; lesson now indexed in MEMORY.md |
| v1-quality-r3b planning brief authorship | 15 | 10 | low | Brief→clarify→DQ cycle added 10 min; DQ itself was trivial to answer but introduced a context-switch |
| DQ answer for planner blocker | 5 | 0 | low | DQ resolution was clear; the surprise was that a well-written brief+clarify-DQ pair still produced a "conflict" perception |
| `/retro-harvest` (full sweep) | 45 | 10 | medium | First run; 10 min overhead on Phase 0 date-oracle verification + batch synthesis. Currency triage depth was high-quality — very few unverified STALE claims. Medium surprise: 7 actionable Tier-1 items found, more than expected |
| Phase 2 parallel triage (4 Explore subagents) | 30 | 5 | low | Parallel dispatch worked well; 5 min to reconcile one LIVE vs STALE disagreement (cargo-test.bat exit propagation — moved to unverified) |
| Roadmap update (`32aa45a60`) | 3 | 0 | none | Mechanical |

## Complexity scores (heavy tasks only)

No Junior impl-tasks ran this session. Session was advisor + harvest + bootstrap work.

| Task | Files | Commits | Runtime (min) | Notes |
|---|---:|---:|---:|---|
| Retro-harvest (full sweep) | 1 (report) | 1 (report) | ~40 | Read-only sweep; 4 parallel triage subagents |
| v1-quality-r3b planning brief + DQ | 3 | 4 | ~30 | Brief + DQ + answer + merge |
| PMD topology repair | 4 | 5 | ~35 | Diagnose + fix + lessons + hook + weekly-review fix |

## Decisions to revisit

- **Tier-1 retro-harvest items before v1-quality-r3b impl**: specifically the missing `feedback_brehon_subagent_model_effort_assignments.md` lesson (Tier-1 #3) — if impl-task briefs reference the wrong model versions, planning brief quality degrades. Address before the next planning or impl dispatch.
- **2-week retro-harvest cadence**: fold into weekly-review skill Step 6 as optional sweep trigger, or add to `.claude/commands/weekly-review.md` as a "if last harvest >14d ago" conditional.

---

## Promotion candidates

- [ ] **PMD HTTP reachability check at session start** (Change #2): add to advisor-orchestrator.md §1 ritual OR extend `pmd-canonical-guard.sh` to curl `http://localhost:11435/mcp` — recurrence ≥3 (DNS SPOF + store-split + topology repair)
- [ ] **DQ answer + brief constraint cross-reference discipline** (carry-forward): add one sentence to advisor-orchestrator.md §3.3 clarify gate OR to the bm-task-brief template §2 "when answering a clarify-DQ, forward-reference the brief constraint by number"
- [ ] **Retro-harvest cadence slot** (Change #3): edit `.claude/skills/weekly-review/SKILL.md` to add Step 6 conditional: "if last retro-harvest report > 14 days ago, run `/retro-harvest`"

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
