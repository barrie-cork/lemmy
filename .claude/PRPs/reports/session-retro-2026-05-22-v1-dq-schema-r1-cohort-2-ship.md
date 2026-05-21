# Session retro — 2026-05-22 — v1-dq-schema-r1 cohort-2 ship

**Harness:** claude-code
**Session window:** 2026-05-22 (post-`/compact` resume) → 2026-05-22 (~30 min wall-clock)
**Branch at start:** `396b43ce2` (`governance-v0`)
**Branch at end:** `c858aa7ab` (`governance-v0`)
**Files touched:** 2 advisor-authored (`scripts/brehon/resolve-dq-canonical.sh`, `.claude/PRPs/reports/v1-dq-schema-r1-retro.md`) + 3 pre-staged from concurrent session (`.claude/hooks/session-start-multi-lane-check.sh`, `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`, `.claude/rules/advisor-orchestrator.md`)
**Commits:** 2 (both `chore(advisor):`)

## TL;DR

Post-compact resume executed cleanly via the pre-compact handover file at `.claude/PRPs/handovers/v1-dq-schema-r1-cohort-2-handover-2026-05-22.md` — zero context recovery work needed. Task 3 (advisor-direct, Option B) was a 2-line + ~6-line-comment patch to `resolve-dq-canonical.sh` that turned out to be load-bearing: empirical pre-merge validation against the live 134-entry DQ surfaced a `TypeError` the old sort key would have thrown on every post-v3-migration invocation. Top friction: the Task 4 retro commit picked up 3 pre-staged files from a concurrent advisor session because I used `git add <single-file>` without checking `git status` first — the result is a mixed-scope commit that's coherent but not what the commit body advertises. Top change: read `git status --short` BEFORE every `git add` to know what's already staged.

---

## What surprised us

- **Empirical pre-merge validation caught a hidden TypeError that the plan's empty-corpus example would not have surfaced.** The OLD sort key `key=lambda e: e['id']` throws `TypeError: '<' not supported between instances of 'str' and 'int'` on mixed-id corpora (133 int + 1 str id post-Task-1-migration). Without Task 3, every post-migration `resolve-dq-canonical.sh` invocation would crash. The discipline of running new logic against live state (not just the plan's example) made the difference between a working ship and a broken resolver.

- **The Task 4 retro commit silently picked up 3 pre-staged files from a concurrent advisor session.** I ran `git add .claude/PRPs/reports/v1-dq-schema-r1-retro.md` followed by `git commit -m ...`, and the commit included 4 files. The other 3 (`session-start-multi-lane-check.sh`, advisor-orchestrator addendum, lesson update) were already in the index from the concurrent fed-in-c-driving session that had been working in this same checkout. The commit body advertised "four-role retro" but actually contained four file changes. Coherent content (multi-lane discipline reinforcement) but a process miss — the commit-history reader can't tell from the subject what's inside.

- **Resume-after-`/compact` worked first-pass.** The handover file (`.claude/PRPs/handovers/v1-dq-schema-r1-cohort-2-handover-2026-05-22.md`) was self-contained enough that I picked up Task 3 in 3 tool calls (Read handover → fetch + log → list-tasks). This validated the handover discipline as load-bearing for cross-compact continuity.

- **Composite-id ship cycle and concurrent-session id collision happened in the same window.** Cohort 1 (Tasks 1+2 [P]) had two workers race for int id 339, exact bug class v3 eliminates. The bug fired DURING the ship that eliminates it — a perfectly emphatic confirmation of need. Validated empirically that the structural fix (Task 1's `dq-v3-new-entry.sh`) was necessary, not optional.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Read `git status --short` immediately before every `git add` (especially in a checkout that may be shared with another session) | Catches pre-staged files belonging to a concurrent session before they get folded into an unrelated commit | trivial (5 sec) | 1× this session (Task 4 retro mixed-scope); ≥2× prior (file-staging surprises in multi-lane sessions per `feedback_multi_lane_worktree_discipline.md`) |
| 2 | When the `git add` would commit on `governance-v0` AND `git worktree list` shows another worktree active on `phase-v1-*`, prefer `git diff --cached` review before commit | Same as #1 but explicit — confirms what's staged matches the planned commit body | trivial (10 sec) | 1× this session |
| 3 | Empirical pre-merge validation against live state should be in the impl-task subagent's task-0 checklist (currently only enforced via plan §15 DoD which uses the plan's example, not live state) | Catches TypeErrors / silent-degradation traps that the plan's representative example doesn't surface | minor (one-line addition to `.claude/agents/impl-task.md` checklist) | 1× this session (Task 3 found load-bearing TypeError); see `feedback_test_against_reality_not_syntax.md` (already a promoted pattern) |

## What to carry forward

- **Pre-compact handover files are load-bearing for cross-compact continuity.** Author them BEFORE `/compact`, not after. They survive context truncation; conversation context does not. The handover file pattern at `.claude/PRPs/handovers/<phase>-cohort-<N>-handover-<date>.md` worked first-pass this session and should be reused for any session that's likely to span a compact boundary or session-end.
- **Option B (advisor-direct authorship) is legitimate when (a) the change is mechanical, (b) the plan §13 enumerates exact edits, (c) Junior dispatch is unreliable due to a known daemon-substrate bug, AND (d) user explicitly approves.** All four conditions must hold; this is not a license to bypass the four-role model casually. The Task 3 commit body documented the exception verbatim — that level of provenance is the bar.
- **Empirical validation against live state, not just plan example, before commit.** This is already a promoted pattern (`pattern_test_against_reality_not_syntax`) but bears explicit reinforcement: any logic change touching state-dependent code (sort keys, dedupe rules, format converters) should be run against the live target before commit. The 2-min Python REPL check pays for itself the first time it catches a hidden TypeError.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `AskUserQuestion` (Task 3 Option A vs B gate) | 15 | 0 | none | Clean fork; user picked Option B; saved Junior dispatch + daemon-bug-recovery cycle |
| Pre-compact handover file (read at resume) | 20 | 0 | low | Self-contained enough to resume in 3 tool calls — validated the handover discipline pattern |
| Empirical pre-merge Python REPL validation | 10 | 0 | high | Surfaced TypeError on mixed-id sort that plan example would not have caught |
| `git add <single-file>` without `git status` check | 0 | 5 | medium | Caused mixed-scope retro commit; recoverable but a process miss — see "What to change" #1 |
| Task 3 advisor-direct edit (Option B) | 25 | 0 | none | Skipped Junior cycle entirely; ~5 min wall-clock vs ~20-30 min with manual-finalize-fallback risk |
| Task 4 retro authoring | 0 | 0 | none | Routine; all 9 DoD checks passed on first write |
| `memory_write_eval` (post-task retro) | 0 | 0 | none | Routine; recorded eval 464, score 0.82 |

## Complexity scores (heavy tasks only)

None this session — both Task 3 (2 lines + comment) and Task 4 (retro authoring) are below the heavy-task threshold (>55 min runtime, >40 min log silence, or >8 files).

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| Task 3 (advisor-direct edit) | 1 | 1 | ~5 | n/a — advisor session, no log silence concept |
| Task 4 (retro author + commit) | 1 advisor + 3 pre-staged | 1 | ~15 | n/a |

## Decisions to revisit

- **Should the canonical `brehon-fork` checkout enforce a working-tree-clean check before commit when `git worktree list` shows ≥2 active worktrees?** The mixed-scope retro commit could have been prevented by a pre-commit script that aborts when staged files don't match the file-list in the commit message subject. But that's a heavy mechanism for a class of friction that may not recur often. Worth a clarify pass before promoting to a hook.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] Change #1 ("read `git status --short` before every `git add` in a shared checkout"): recurrence is 1× this session + ≥2× prior (per `feedback_multi_lane_worktree_discipline.md`). Promotion candidate: extend `feedback_multi_lane_worktree_discipline.md` with a "Pre-commit hygiene in canonical checkout" sub-section explicitly naming the `git status` check. NOT a new lesson file — the canonical-checkout discipline already exists; this is a one-line reinforcement.
- [ ] Change #3 (empirical validation against live state in impl-task checklist): recurrence is 1× this session + multiple prior (per `pattern_test_against_reality_not_syntax`). Could promote to `.claude/agents/impl-task.md` task-0 checklist as a one-line addition: "For state-dependent logic changes (sort keys, dedupe, format), run against live target state before commit; plan example may have empty-corpus blind spot."
- [ ] PMD eval write: ALREADY DONE in this session via `mcp__project-memory__memory_write_eval` for the post-task-retro discipline (memory id 464). The session-retro eval is separately scored below.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
