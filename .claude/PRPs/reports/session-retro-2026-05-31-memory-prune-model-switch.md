# Session retro — 2026-05-31 — memory-prune + model switch

**Harness:** claude-code  
**Session window:** ~2026-05-31T20:00 IST → ~2026-05-31T20:30 IST (~30 min)  
**Branch at start:** `ff97b84cf` (`governance-v0`)  
**Branch at end:** `ff97b84cf` (`governance-v0`) — no repo commits  
**Files touched:** 1 (MEMORY.md only, user-scope PMD)  
**Commits:** 0 (MEMORY.md is outside the repo; no git commit needed)

## TL;DR

Short housekeeping session: model switched to Sonnet 4.6, then `/memory-prune` run. Prune trimmed MEMORY.md from 23,490 → 21,422 bytes (2,068 bytes / ~2 lines saved) by archiving one stale workflow-state entry, moving one completed-watch entry to Historical, and shortening 15 verbose lines. The byte-vs-lines distinction surfaced as the key insight: the file was 190 lines (under the 200-line soft ceiling) but within 910 bytes of the 24,400-byte hard truncation limit — a near-miss that wouldn't have been caught by a lines-only check. Top change proposal: the prune skill's Step 1 urgency gate should trigger **proactive prune** whenever bytes are within 10% of the 24,400 limit (~22,000+), not just when over. At 23,490 bytes this session was in that zone and needed to be caught proactively, not reactively.

---

## What surprised us

- **Advisor:** The byte headroom was only 910 bytes (3.7% of budget) despite the file being 10 lines under the soft ceiling. The two limits diverge in practice because verbose lines (some >400 bytes) mean a file can be under 200 lines yet over 24,400 bytes. The skill body already documents this phenomenon, but the urgency gate bands (Near budget: 22,000–24,400) correctly captured it — the near-miss was a test of those bands working as designed rather than a gap.

- **Advisor:** Line 14 (v1-RT-r5 workflow state) was 415 bytes — the longest single line in the file, and more than double the 200-byte verbose threshold. That one line was ~1.8% of the total file budget on its own. Active workflow state lines will always be the fastest-growing entries because they accumulate execution detail during a sub-phase; this argues for a structural rule: active workflow state entries should be capped at ~180 bytes in the index, with detail deferred to the linked workflow_state file.

- **Advisor:** The v1-quality-r3b entry (line 15) explicitly said "Delete when v1-quality-r3c ships" — and r3c was already active on line 16. The self-referential delete instruction worked exactly as intended: the prune scan caught it and the entry was archived. No friction.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | In `memory-prune/SKILL.md` Step 1 urgency gate: add a note that active workflow-state lines should be capped at ~180 bytes at write time (detail to linked `.md`), not at prune time. | Reduces the frequency of the "415-byte active state entry" class before it requires pruning. | minor | 1× this session; structural pattern visible in file |
| 2 | In `memory-prune/SKILL.md` Step 2 classification table: add `workflow-state-verbose` as a distinct sub-class of `Verbose` with the 180-byte cap and "detail to linked file" action. Makes the classification mechanical rather than judgment-based for the most common verbose pattern. | Faster prune classification for the highest-byte-cost entry class. | minor | 1× this session, recurrent across every active sub-phase |
| 3 | When authoring new workflow-state index entries in MEMORY.md, enforce the 180-byte guideline at write time: the entry should fit `**ACTIVE: [label](file.md)** — <3-5 word state summary>. Detail in workflow_state file.` Any entry longer than that defers detail to the linked `.md`. | Prevents accumulation of the verbose-active-state class between prune cycles. | minor | retroactively matches the pattern visible across multiple active-state entries |

## What to carry forward

- **Byte-vs-lines distinction is load-bearing.** The binding limit is bytes, not lines. Always run `wc -c` AND `wc -l`; report byte headroom as the headline. A file under 200 lines can still be within 3% of hard truncation.
- **Self-referential archive instructions work.** The "Delete when X ships" pattern in workflow-state entries is an effective self-cleaning mechanism. Use it more deliberately when authoring entries whose lifespan is bounded by a known future event.
- **Verbose-line identification is fast via `awk 'length > 200'`** — the prune skill's classification pass can be done mechanically in seconds, not by reading every line. This pattern is worth making part of the routine Step 2 tooling rather than ad-hoc grep.
- **Prune is near-budget-triggered, not calendar-triggered.** The 22,000-byte threshold (10% below hard limit) is the right trigger; don't wait for truncation warnings.

---

## Three-signal scoring

Per `.claude/lessons/feedback_four_role_retro_signals.md`.

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/model sonnet` | 0 | 0 | none | Routine switch; no friction |
| `/memory-prune` skill | 15 | 0 | low | Byte-vs-lines near-miss (910 bytes headroom) surfaced correctly by the skill's bands; no wasted loops; AskUserQuestion clean |

## Complexity scores (heavy tasks only)

No impl tasks, no Junior dispatch, no repo commits. N/A.

## Decisions to revisit

- The 180-byte cap for workflow-state index entries is a guideline, not a hard rule. Worth encoding in `feedback_memory_workflow_state_entry_length.md` as a lesson if a future session (or two) produces the same verbose-active-state entries after a prune. Single recurrence this session — watch, don't promote yet.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] `memory-prune/SKILL.md` Step 2: add `workflow-state-verbose` sub-class with 180-byte cap — update skill (minor; no promotion to lesson needed; skill update suffices)
- [ ] Author `feedback_memory_workflow_state_entry_length.md` if this recurs in one more session (watch threshold not yet met)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
