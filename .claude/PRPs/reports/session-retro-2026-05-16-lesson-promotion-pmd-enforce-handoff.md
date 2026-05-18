# Session retro — 2026-05-16 — lesson-promotion-pmd-enforce-handoff

**Harness:** claude-code
**Session window:** 2026-05-16T21:40:00Z → 2026-05-16T22:10:00Z (~30 min) — the post-first-retro arc only
**Branch at start:** `6a7723a7e` (`governance-v0`) — the first-retro commit
**Branch at end:** `234d51ad8` (`governance-v0`)
**Files touched:** 11 (8 lessons/rules/skills + handover + runlog + the prior retro's checkbox update)
**Commits:** 3 explicit (`36ccee1ad`, `287ad47e9`, `234d51ad8`), 0 auto

> **Scope note:** this is the **second** retro of the 2026-05-16 session. The first
> (`session-retro-2026-05-16-v1-ad-e-gate-bmcut-finalize-recovery.md`) covered v1-AD-e
> gate-1 + the bm-cut finalize-merge recovery. This retro covers ONLY the work *after*
> that retro: executing its 5 approved promotions, the PMD-embed-gap fix, and the
> v1-AD-e resume handoff. It deliberately does not re-litigate the first arc.

## TL;DR

A short, mostly-mechanical curation arc: executed the first retro's 5 approved promotion candidates (2 new lessons + 1 lesson-augment + bm-cut.md + multi-lane-worktree.md), then closed a newly-surfaced gap (PMD has no write-time embedding) via skill-level enforcement rather than the classifier-blocked auto-exec hook, then wrote a cold-start resume handoff for v1-AD-e. The single highest-value finding is meta: **the first retro's "What to change" items were actioned in the same session, and that worked well** — the retro→promote loop has near-zero latency when the same session that wrote the retro also executes it. The notable friction: the auto-mode classifier correctly blocked an auto-exec PostToolUse hook, forcing a better (skill-enforced) design — a case where a guardrail improved the outcome.

---

## What surprised us

- **The auto-mode classifier blocked the PostToolUse hook write — and was right to.** I'd chosen (with user approval of the *approach*) a hook that spawns detached `backfill.js` after every PMD write. The classifier denied the `.claude/hooks/*.sh` write as "self-modification of agent-loaded config; ambiguous prior authorization." Surprising in the moment (the user had picked "PostToolUse hook"), but the denial forced a re-think that produced a *strictly better* design: skill-level enforcement at the 2 points that own PMD writes (session-retro Step 5.5 inline, weekly-review Step 1b for the Junior-side DB), which covers more cases (not just this repo's CC sessions) with less blast radius. The guardrail improved the outcome. Surprise: medium.
- **`post-task-retro` structurally cannot run an inline backfill.** Discovered while placing the enforcement: its `memory_write_eval` MUST be the absolute-final action before exit (the `junior/*` Stop hook matches a fresh retro within 30 min; anything after it risks the hook + the watchdog). So the Junior-side PMD DB can't be backfilled at write-time at all — it had to go to weekly-review's Sunday sweep. A non-obvious constraint that shaped the design. Surprise: low-medium.
- **PMD MCP server has zero write-time embedding — confirmed by source, not docs.** `dist/index.js` (all `memory_write*` handlers) has no `embed`/`ollama` reference; embedding is exclusively in the standalone `backfill.js`. The user's question ("will new entries be first-class searchable?") had a definitive *no* answer only after reading the server source — assuming from `pmd-search-strategy.md` alone would have been a weaker answer. Surprise: low (consistent with the lesson corpus, but the source-confirmation was load-bearing).
- **A concurrent session wrote a near-identically-named retro today** (`session-retro-2026-05-16-pmd-backfill-dq-hook-scope.md`). The PMD-backfill topic was apparently surfaced independently by another session. No collision (distinct slugs), but it confirms the embed-gap is being hit repo-wide, not just by this session — strengthens the recurrence case. Surprise: low.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | When a `/session-retro` produces approved §3 promotions, **execute them in the same session if the user approves inline** (as happened here: "Go for it" → all 5 shipped immediately). Add a one-line note to `session-retro/SKILL.md` Step 6 that same-session execution of approved promotions is the preferred path (near-zero latency; the context is hot). Currently the skill implies a *follow-up* session executes checked boxes. | Removes the promote-later drift where approved lessons sit unwritten across sessions (the exact gap that left #344–#348 unembedded). | trivial | 1× this session + the embed-gap recurrence it fixed (≥2 effective) |
| 2 | The `session-retro` skill's "When NOT to use" should add: *a second retro in the same session is valid when a distinct work-arc followed the first retro* (here: promotion-execution + handoff after a gate/recovery retro). Without this, the skill's "session too short / already retro'd" guidance could wrongly suppress a legitimate second retro. | Prevents under-retro of multi-arc sessions; makes the scope-note pattern (used in this file's header) the documented norm. | trivial | 1× this session (single-instance — noted, not promoted; see below) |
| 3 | `feedback_pmd_backfill_after_write.md` should cross-link from `pmd-search-strategy.md` (the rule already documents the manual backfill but doesn't yet point at the new enforcement lesson). One-line "See also" addition to the rule. | A session reaching the rule for the manual recipe also discovers the now-enforced Step 5.5 / Step 1b, closing the "knew the command, didn't know it's now automated" gap. | trivial | 1× this session (single-instance — noted, not promoted) |

## What to carry forward

- **Same-session retro→promote loop.** The first retro wrote 5 proposals; the user approved inline; all 5 shipped before session end. This is the ideal — the retro's value compounds immediately instead of waiting for a follow-up session that may never come (cf. the unembedded #344–#348 from sessions that didn't loop back). Make this the default reflex when the user is present and approves.
- **Schema-first gate held cleanly across 4 file authorings.** Every new/edited lesson + the bm-cut.md + multi-lane-worktree.md edit read a canonical sibling first (`feedback_junior_finalize_skips_*`, `feedback_pmd_two_memory_systems_*`, `inject-dq-state.sh`, the existing Hard refusals) and cited it. Zero schema drift. The gate is cheap (~2s read) and worked — keep applying it without exception.
- **Dogfooding the rule you just wrote.** The new `feedback_pmd_backfill_after_write.md` + Step 5.5 were exercised *in the same session* (sync → backfill → verify 261/261). Writing a process rule and immediately running it surfaces "does this command actually work as written" before it ships — caught nothing this time, but it's the right discipline (cf. `feedback_dogfood_slash_command_specs.md`).
- **Verify live state before authoring a handoff.** The handoff's state snapshot was built from fresh `git ls-remote` / DQ-read / daemon-query, not session memory — which caught that fed-inbound-a's Cohort A had gone fully terminal (the parking blocker had cleared) since the parked-state bookmark was written. A memory-derived handoff would have shipped a stale "still blocked" premise.
- **A blocked action is a design signal, not just an obstacle.** The classifier denial didn't just stop the hook — it indicated the hook was the wrong shape. Treating the denial as "find a better design" rather than "find a workaround" produced the superior skill-enforced solution. Carry this framing.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/session-retro` (first, prior arc) | — | — | — | not in scope of this retro; scored in its own file |
| Promotion execution (5 items: 2 lessons + augment + 2 rule/cmd edits) | 15 | 0 | none | Mechanical from the approved retro list; schema-first gate kept it clean. Saved vs re-deriving the lesson content from scratch in a future session. |
| `AskUserQuestion` ×2 (PMD-gap approach, hook-authz post-block) | 5 | 0 | medium | The 2nd one (post classifier-block) correctly re-surfaced the decision instead of working around the denial — produced the better design. |
| Auto-mode classifier (hook write denial) | 10 | 3 | medium | Net positive: 3 min "wasted" re-planning the approach, but it blocked a worse design and forced the skill-enforced one. Counting the block as a *save*. |
| `sync-lessons-to-pmd.sh` + `backfill.js` (×2 runs) | 4 | 0 | none | Idempotent, fast; dogfooded the new Step 5.5. 261/261 embedded verified. |
| Handoff authoring (`v1-AD-e-resume-bootstrap.md`) | 20 | 0 | low | A cold-start session resuming v1-AD-e without this would re-derive state (~20 min) and risk re-running the completed gate-1. The live-state-verify caught the cleared parking blocker. |

## Complexity scores (heavy tasks only)

Per `feedback_retro_task_complexity_score.md`. No impl-tasks / Junior tasks ran in this arc (pure curation + handoff authoring — laptop-side, advisor-only). The metric (designed for Junior impl-task watchdog-envelope tracking) does not apply. Recorded for completeness:

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Promotion-execution + PMD-enforce + handoff (whole arc) | 11 | 3 | ~30 | n/a (interactive, no Junior worker) |

No watchdog-envelope concern (no Junior worker involved). The metric's blind spot noted in the first retro (it measures the worker, not advisor-side curation) applies here too — this arc is invisible to the complexity metric by design.

## Decisions to revisit

- **`post-task-retro` Junior-side PMD writes only get embedded weekly.** The Sunday weekly-review sweep is the safety net, but a lesson/eval written by a Junior task on Tuesday is FTS5-only for up to 5 days. Acceptable for now (FTS5 still finds exact-token matches; semantic recall is the degraded part), but if a Junior-written lesson proves load-bearing mid-week and a paraphrased search misses it, revisit (options: a more frequent daemon-side backfill cron, or a lighter pre-Step-7 fire-and-forget). Track in the next weekly-review.
- **Two retros per session is becoming normal for long advisor sessions.** This is the 2nd this session; the first ship-1 session also produced multiple. If 3+ multi-arc sessions recur, the "When NOT to use" guidance (change #2) should be promoted from a single-instance note to an actual skill edit.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] #1 same-session retro→promote-loop note → add to `.claude/skills/session-retro/SKILL.md` Step 6 (preferred path = execute approved promotions inline when user approves; the latency win + the unembedded-#344–#348 evidence make this ≥2 effective)
- [ ] #3 cross-link `feedback_pmd_backfill_after_write.md` ↔ `.claude/rules/pmd-search-strategy.md` (one-line "See also" both directions — trivial, closes the "knew the manual command, didn't know it's now enforced" gap)
- ( ) #2 "second retro per session is valid for multi-arc sessions" → `session-retro/SKILL.md` "When NOT to use" — **single-instance this session; NOT promoted** (recorded in Decisions-to-revisit; promote only if 3+ multi-arc sessions recur, per the skill's own discipline against over-promotion)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase reliability section
omitted — no `/auto-phase` invocation in this arc; on-disk auto-state JSONs
are prior-session artifacts only (Step 0.5 trigger did not fire). Scope:
the post-first-retro curation + handoff arc only; the v1-AD-e gate-1 +
bm-cut recovery arc is retro'd separately in
`session-retro-2026-05-16-v1-ad-e-gate-bmcut-finalize-recovery.md`._
