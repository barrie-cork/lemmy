# Session retro — 2026-05-31 — pmd-validation-pheromone-spec

**Harness:** claude-code
**Session window:** 2026-05-30 ~23:00Z → 2026-05-31 ~00:50Z (~110 min, spanned one `/compact`)
**Branch at start:** `0a117e30e` (`governance-v0`)
**Branch at end:** `f392a5606` (`governance-v0`)
**Files touched:** 4 (`retro-check.sh` v4 + lesson + `pmd-invariants.md` via task-533 merge; `mcp-pmd-read-pheromone.md` new; MEMORY.md pointer — auto-memory, outside git)
**Commits:** 3 explicit (`091a367ac` merge task-533, `a6643c689` lesson-sync hook, `f392a5606` pheromone spec) — 0 auto

## TL;DR

Post-compact continuation that closed the loop on the PMD retro-hook fix (Junior #533, dispatched pre-compact) and then ran a full PMD end-to-end validation, all green. The session's durable output is a new spec, `mcp-pmd-read-pheromone.md` — an ant-stigmergy-inspired recency/frequency ranking boost for the PMD, designed as a deliberately *minimal* change (two columns + one UPDATE-on-read + one ORDER BY term, no background cron) after reading the actual MCP source. The top carry-forward: **reading the real code before speccing turned an "interesting idea" into a 320-line minimal-surface contract** — the design's whole shape (pheromone rides on top of RRF, never fights it) fell out of seeing that `ORDER BY importance, updated_at` is static and reads leave zero trace today. The recurring friction the user named explicitly: the advisor doesn't reliably know **where to look first** for a finalize-merge result (daemon-local ref vs origin vs local), causing a multi-step hunt on nearly every task close.

---

## What surprised us

- **The PMD "pheromone" system isn't weak today — it's entirely absent.** Expectation going in was "importance exists, we just need to decay it." Reading `src/tools/search.ts` showed all three search handlers end with `ORDER BY m.importance DESC, m.updated_at DESC`, both static fields; reads leave *no trace at all*. There is no deposit step to tune — the whole mechanism is missing. That reframed the spec from "add decay to existing signal" to "add the deposit *and* the consume, but keep them subordinate to the RRF relevance engine that already works."
- **Task #533's finalize-merge landed on the daemon-local `governance-v0` but was never pushed to origin.** The merge commit `a47a85e16` existed on the daemon, the worker branch `d5df9df4b` existed on origin, but `origin/governance-v0` was still at the pre-merge brief commit. Took four probes (local log → daemon log → origin fetch → daemon push) to reconcile. The user flagged this directly mid-session: *"every task advisory does not know when first to look."* This is real recurring friction, not a one-off.
- **Two research subagents both died on API ConnectionRefused / socket-closed** (~600s and ~100s in) returning zero tokens. The fallback — reasoning from first principles + the actual code — produced a *better* spec than the research would have, because the binding constraint was the existing code shape, not the algorithm literature. Surprising that the agent failure was net-neutral-to-positive.
- **The user's redirect ("identify the minimal change that will make real impact") arrived at exactly the right moment** — just as the research agents failed. It cut off a potential over-research spiral and forced the spec toward minimal-surface. Good instinct; worth internalising as a default posture for speculative enhancements.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Author `feedback_finalize_merge_where_to_look_first.md` lesson encoding the **canonical lookup order** for a finalize-merge result: (1) `ssh homeserver "git log <trunk> -1"` (daemon-local — the merge lands here first), (2) `git ls-remote origin <trunk>` (did the daemon push?), (3) local fetch. The merge is daemon-local-first, push-second; checking origin first always shows stale. | Eliminates the 3-4-probe hunt on every `governance-v0`-based task close. Advisor goes straight to the daemon ref. | minor | **2×+** — this session (task #533) + the user explicitly said "this is always happening" |
| 2 | Add a one-line post-finalize check to the relevant flow: after a `base_branch=governance-v0` Junior task reports `done`, the advisor's first action is `ssh homeserver "cd /srv/brehon-fork && git log governance-v0 -1 --oneline"` to see if the daemon merged + whether it pushed (compare to `git ls-remote origin governance-v0`). Codify in `advisor-orchestrator.md` §3.1 stage-shape ("impl-task complete on governance-v0 → check daemon-local trunk tip FIRST, then origin"). | Turns the ad-hoc hunt into a single deterministic command. Pairs with #1. | minor | 2×+ (same basis as #1) |
| 3 | When dispatching speculative/enhancement specs, default to **read the target source before speccing** — make it an explicit first step, not an afterthought. This session it was the difference between a back-of-envelope formula and a minimal-surface contract. | Specs land minimal + grounded; avoids gold-plating + avoids speccing against an imagined code shape. | minor | 1× strong this session + aligns with existing `feedback_read_canonical_before_writing_spec.md` (extend that lesson rather than new file) |

## What to carry forward

- **Read the real code before speccing a code change.** The pheromone spec's entire safety story (pheromone is a *bounded tertiary term within an importance band*, RRF stays dominant, evaporation-floor = boost≥0) is a direct consequence of having read that RRF already does relevance ranking. Speccing blind would have produced a formula that fights the relevance engine. Used once, decisively, this session.
- **Subagent-failure graceful degradation to first-principles + ground-truth.** When both research agents died, falling back to "reason from what we know + read the authoritative source" lost nothing. Don't treat a failed research dispatch as a blocker — the code is usually the better authority anyway for a code-change spec.
- **Decay-on-read, not background cron, as the default for any "freshness" mechanic on the solo-dev SQLite store.** Captured in the spec §4 but worth holding as a general principle: a background sweep adds an ops surface that can silently die (`feedback_pmd_backfill_after_write.md` failure class). Lazy compute-at-query-time is almost always the right primitive here.
- **Park speculative work with an explicit co-ship linkage.** The pheromone spec + write-time-embedding spec touch the same two files; recorded the "co-ship these two" pointer in MEMORY.md so the linkage surfaces when either is next scheduled. Cheap insurance against re-deriving the bundling.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `Explore` subagent (task-533 log extraction, 506KB) | 12 | 0 | none | Kept the 506KB log out of main context; returned verbatim probe table + commit SHA on first read. Clean offload. |
| 2× `general-purpose` research subagents (ACO + cache-decay) | 0 | 8 | high | Both died on API ConnectionRefused / socket-closed, 0 tokens. Net wasted ~8 min wall-clock, but fallback to first-principles was better — see "What surprised us". |
| task-533 verify + merge-forward (manual git) | — | 10 | medium | The 4-probe hunt to find where the finalize-merge landed. The 10 "wasted" min is the friction #1/#2 above target. |
| PMD end-to-end validation (write/search/recent + daemon sqlite) | 8 | 0 | low | `memory_write_eval` → `memory_search_hybrid` round-trip confirmed in-session; daemon sqlite 577→579 surfaced the legacy-parallel-store fact cleanly. |
| Spec authoring (`mcp-pmd-read-pheromone.md`) | — | 0 | none | ~320 lines, grounded in source read. The "heavy" deliverable; no wasted effort. |
| MEMORY.md co-ship pointer | 2 | 0 | none | Anchored the spec-bundling linkage. |

## Complexity scores (heavy tasks only)

No Junior tasks ran *within* this session — task #533 executed pre-compact (its complexity belongs to that session's record). The merge-forward + spec authoring were advisor-inline, not watchdog-bounded. No task crossed the >55min/>40min-silence/>8-file flags.

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| (none — no in-session Junior dispatch) | — | — | — | — |

## Decisions to revisit

- The daemon-local sqlite (`/srv/brehon-fork/.project-memory/memory.db`, now 579 rows) is populated by *some* path not connected to the live HTTP server — provenance still unclear (noted in task-533's lesson as "future cleanup may be warranted"). Worth a focused 15-min investigation in a future PMD-maintenance session: is it the daemon's own retro writes via a stale config, or a leftover backfill target? If the former, it's a second silent store that could mislead a future hook.
- Pheromone spec is speculative until the corpus is large enough that stale lessons clutter search. The honest trigger ("when search surfaces stale lessons ahead of the one you needed") is recorded in MEMORY.md. No action until then.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 (finalize-merge lookup order): promote to new `.claude/lessons/feedback_finalize_merge_where_to_look_first.md` — **meets 2× threshold** (this session + user's explicit "always happening"). Cross-references existing `feedback_daemon_local_trunk_stale_multi_lane.md` (same daemon-local-first principle, opposite direction — that's about *reading* the stale ref, this is about *where the merge result lands*).
- [ ] Change #2 (advisor §3.1 post-finalize check): update `.claude/rules/advisor-orchestrator.md` §3.1 stage-shape — "impl-task complete on governance-v0 → check daemon-local trunk tip FIRST." Pairs with #1.
- [ ] Change #3 (read source before speccing): extend existing `.claude/lessons/feedback_read_canonical_before_writing_spec.md` with a "for code-change specs, read the target source first" clause rather than a new file (avoid lesson-corpus duplication).

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`. Auto-phase section omitted — no
`/auto-phase` invocation and no auto-state mutation this session (leftover
`v1-RT-r3.json` is from a prior session per Step 0.5 case 4)._
