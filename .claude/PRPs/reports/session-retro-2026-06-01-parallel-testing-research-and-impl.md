# Session retro — 2026-06-01 — parallel-testing-research-and-impl

**Harness:** claude-code  
**Session window:** 2026-06-01T~10:00 → ~11:30 IST (~90 min)  
**Branch at start:** `703c917dc` (`governance-v0`)  
**Branch at end:** `16f995f61` (`governance-v0`)  
**Files touched:** 8 (across HEAD~2..HEAD diff)  
**Commits:** 2 (both explicit; 0 auto)

## TL;DR

The user asked for research into best practices for parallel Rust testing with a view to scheduling multi-lane e2e runs. This session did that research exhaustively — four parallel Explore agents, followed by direct file reads to close ambiguities — then implemented four concrete changes: nextest as default (wrappers), nextest in CI, `e2e_filter` DQ field, and a canonical filter-group lesson. The most load-bearing finding was that all four changes were **already committed in `16f995f61`** before the current impl conversation started, because a prior session (the v1-redaction-r1 handover) had bundled them. The "implement" step verified correctness rather than produced new commits. Top change proposal: add an "already-shipped check" to the pre-impl flow so the advisor doesn't re-derive changes that are already in HEAD.

---

## What surprised us

**Advisor (this session):**

- **The prior session had already shipped all four changes.** When the user said "implement 1 to 4," the changes were already at HEAD (`16f995f61`). The session's `git log --oneline` shown in the system-reminder at start explicitly listed that commit, but the impl flow proceeded as if the changes were pending. The `Edit` tool reported success for each edit (correct — the files matched the target state), and `git status` showed nothing staged (correct — files were already committed). Only `git hash-object` comparison confirmed the no-op nature. This was ~15 min of unnecessary work.

- **The "67 tests" figure in the lesson was stale.** `feedback_local_validation_cycle_2026_05_02.md` cited 67 tests; the current `e2e.rs` has 41. The lesson was authored against an older version of the file. Stale lesson metrics are a recurring class (cf. `feedback_runbook_audit_drift_post_event_check.md`) — this instance went undetected until a Bash `grep -c` cross-check.

- **Four parallel Explore agents produced high-quality, non-duplicating research.** The agents divided the problem space cleanly (nextest syntax, test count, schema isolation feasibility, Docker concurrency + test names) without overlap. The synthesis cost in the main context was low. Positive surprise — the "research until no ambiguities" framing drove genuine depth.

- **Per-test schema isolation is categorically infeasible** on this codebase without ~300–400 lines of new code — a much larger scope than implied by the original "maybe consider it" framing. The three specific blockers (hardcoded `public.*` in triggers DDL, `schema_sentinel_satisfied()` hardcoded check, `pg_dump --schema=public --schema=r` cap) were found in the actual source, not estimated. This clarity is worth the research time.

**Impl (no Junior tasks dispatched):**

- Not applicable — this was an advisor-only research-and-impl session.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | **Pre-impl HEAD check** — before executing any "implement X" request, run `git show HEAD --stat` and check whether the named changes are already present. If they are, confirm with the user before re-deriving. | Eliminates ~15 min of no-op edit work when a prior session has already shipped the changes. | minor (1 sentence at session-start or a pre-impl habit) | 1× this session; same class as `feedback_runbook_audit_drift_post_event_check.md` |
| 2 | **Stale-metric guard in lesson authoring** — when a lesson file cites a numeric count derived from a source file (e.g., "67 tests in e2e.rs"), add a maintenance note stating the count's source and date, and flag it for update whenever the source file changes. Pattern: `<!-- count from grep -c #\[tokio::test\] e2e.rs, 2026-05-03 — re-verify before citing -->`. | Prevents stale metrics from silently misleading future briefs that cite the lesson. | minor (template addition to lesson authoring habit) | 1× this session; 1× prior (26-min baseline cited for nextest in advisor-orchestrator.md per feedback_local_validation_cycle_2026_05_02.md §"lessons within the lesson" #1) → threshold met |
| 3 | **Filter-group lesson maintenance note** — `feedback_e2e_nextest_filter_groups.md` currently says "as of 2026-06-01, 41 tests" but has no guard instructing the next author to re-verify the count when e2e.rs changes. Add a maintenance line: "Re-run `grep -c '#\[tokio::test\]' crates/server/tests/e2e.rs` before citing the count in a brief." | Ensures the lesson stays accurate through future test additions without a full re-read of e2e.rs. | minor (edit the lesson) | 1× this session |
| 4 | **`threads-required` tuning for 64 GB machine** — the nextest.toml comment now correctly states RAM is not the constraint, but `threads-required=4` (→ 2 concurrent containers) is still the conservative default. A single scoped run (7 jury tests × 22s = ~2.5 min) doesn't stress Docker. Consider dropping `threads-required` to `2` (→ 4 concurrent) in a dry run to measure whether Docker named-pipe saturation appears. If it doesn't, make 2 the new default. | Cuts full e2e from ~12 min to ~6 min; cuts scoped runs proportionally. | minor (one nextest.toml edit + one ~12 min local measurement) | First measurement; not yet a recurrence |

## What to carry forward

**Advisor:**

- **Four parallel Explore agents with narrow scopes** outperform two broader agents for research that spans independent problem dimensions. This session used (1) nextest syntax + installation, (2) test count outside e2e.rs, (3) schema isolation feasibility, (4) Docker concurrency + test names. Each returned a clean, focused report. The synthesis was easy. Use this pattern for any "resolve ambiguities" research task with 3+ independent questions.

- **Bash `grep -c` cross-checks after Explore agents** caught the stale "67 tests" claim before it propagated into the lesson. Spot-checking numeric claims from subagents against the actual source with a cheap Bash call is worth the ~5s cost on every research session that produces a count, rate, or size figure.

- **The "already-shipped" commit pattern** (prior session bundles research output before the current session's impl conversation) is load-bearing for the handover workflow but creates a confusing impl UX. The handover mechanism is correct; the session-start habit of checking `git show HEAD --stat` against the task list is the mitigation.

- **Schema isolation is off the table for this hardware.** The three blockers are in code, not in estimates. The decision is documented in `feedback_local_validation_cycle_2026_05_02.md` and repeated here for clarity: option-3 (shared container, per-test schema) requires rewriting trigger DDL, the sentinel, and pg_dump scope — ~300–400 lines. The template-restore scaffolding is already in place if a future contributor wants to pursue it on slower hardware where cold migrations genuinely dominate pg_restore.

- **`e2e_filter` in validate-pending-laptop DQ** is the highest-leverage multi-lane improvement. A jury-mechanics task that writes `"e2e_filter": "test(~submit_jury_vote)"` runs 7 tests in ~2.5 min instead of 41 tests in ~12 min. This should become a mandatory field in impl-task brief §4 whenever the task touches a named governance area (not "unknown scope"). The lesson table makes the mapping mechanical.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| 4× parallel Explore agents (research phase) | ~60 | 0 | medium | Covered all four dimensions cleanly; no overlap; synthesis straightforward. Positive surprise: schema isolation feasibility was resolved definitively from source code, not estimates. |
| Direct Read + Bash cross-checks (ambiguity closure) | ~10 | 0 | low | Bash `grep -c` caught stale test count. `git hash-object` confirmed prior-session no-op. Both cheap and high-value. |
| Edit × 6 (impl phase) | 0 | ~15 | high | All 6 edits were no-ops — files already at target state from prior session commit. `Edit` reported success (correct), but the session treated them as real work. The `git show HEAD --stat` check at the start would have short-circuited the whole impl leg. |
| nextest.toml comment update (64 GB RAM) | ~5 | 0 | none | User's "64 GB RAM" message arrived mid-impl; integrated cleanly. Comment now accurately states RAM is not the bottleneck. |
| lesson authoring (`feedback_e2e_nextest_filter_groups.md`) | ~30 | 0 | none | 41-test inventory + 9 filter groups + multi-lane safety + maintenance note. Reusable across all future jury/reputation/grace brief dispatches. |

## Complexity scores (heavy tasks only)

No Junior tasks dispatched this session. The research phase used 4 Explore subagents; no impl-task complexity metric applies.

## Decisions to revisit

- **`threads-required=2` measurement** — drop concurrency cap from 4 → 2 (→ 4 concurrent containers) on the 64 GB laptop and measure whether Docker named-pipe saturation appears. If it doesn't, make 2 the permanent setting. This cuts full e2e wall-clock to ~6 min. Estimated risk: low (Docker Desktop on Windows allows 10+ concurrent containers in practice; the named-pipe cap is usually hardware-driver not memory). Trigger: next full local e2e run.

- **`e2e_filter` mandatory vs optional** — currently specified as optional in the DQ schema (`string | null`). Consider making it required for `validate-pending-laptop-e2e` entries when the task touches a single named governance area. Mandatory would enforce brief authors to pick a filter group, preventing accidental full-suite runs on scoped tasks. Risk: advisor must identify the correct group, which is mechanical (lesson table) but adds a step. Trigger: first `validate-pending-laptop-e2e` entry written under the new schema.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Stale-metric guard** (change #2): promote to `.claude/lessons/feedback_stale_metric_lesson_guard.md` — generalises beyond e2e.rs to any lesson that cites a count derived from a source file. Recurrence: 1× this session (67 tests) + 1× prior (26-min baseline in advisor-orchestrator.md, documented in feedback_local_validation_cycle_2026_05_02.md §"lessons within the lesson" #1) → threshold met.

- [ ] **Pre-impl HEAD check habit**: codify as a one-liner in `.claude/rules/advisor-orchestrator.md` §3.1 ("Before any impl execution, run `git show HEAD --stat` and verify the named changes are not already present at HEAD"). Cost: minor edit. Not promoting to a standalone lesson (single occurrence); inline rule addition is sufficient.

- [ ] **Update `feedback_e2e_nextest_filter_groups.md`**: add maintenance note per change #3. Minor edit; execute directly next session without a lesson.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
