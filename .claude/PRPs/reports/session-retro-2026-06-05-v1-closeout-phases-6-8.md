# Session retro — 2026-06-05 — v1-closeout-phases-6-8

**Harness:** claude-code
**Session window:** 2026-06-04 (evening) → 2026-06-05 (~120 min across two context windows)
**Branch at start:** `phase-v1-closeout` (prior compacted session had completed Phase 7 type-state scaffold)
**Branch at end:** `9a0b3095a` merged to `origin/governance-v0` (PR #181)
**Files touched:** ~30 (Phase 8 comment-only: 9 files; retro file: 1; plan update: 1; merge)
**Commits:** 4 this context window (merge-forward, retro, Phase 8, plan update); full session ~42 commits

## TL;DR

Session completed the v1 close-out arc: Phase 7 e2e validation confirmed GOLDEN_INVARIANT (`130 passed / 5 skipped`) with one pre-existing timing flake; Phase 8 governed 12 TODO markers via workflow fan-out + grep verification; capstone retro authored; PR #181 opened and merged to `governance-v0`. Key finding: ADR red-flag scanner produced false positives on Phase 6 move-not-delete diffs — PR comment acknowledged the false positives rather than attempting to satisfy an incorrect scanner signal. Most impactful change candidate: always run `--no-fail-fast` for GOLDEN_INVARIANT assertions to prevent undercounting.

---

## What surprised us

- **ADR scanner false positives on move-not-delete (Phase 6 e2e decomposition):** The diff-based ADR scanner flagged `EmergencyRemove` "removal" and `governance_log` "destructive change" — both were Phase 6 file moves, not deletions. The `−` lines in `e2e.rs` had matching `+` lines in the new module files, but the scanner only saw the diff of `e2e.rs` itself. Surfacing this in the PR comment rather than trying to satisfy the scanner was the right call, but the false-positive rate is high enough that this is a known limitation worth documenting.

- **e2e `--no-fail-fast` discovery:** Initial Phase 7 validation counted only 117/130 tests because short-circuit exit on the timing flake stopped the run. Rerun with `--no-fail-fast` confirmed 130 passed / 5 skipped. This isn't new, but it surfaced again: GOLDEN_INVARIANT verification must always use `--no-fail-fast` or the count is unreliable.

- **PR #181 merged without explicit `bm-merge`:** The PR was auto-merged (admin bypass per the same pattern as m1-b PR #177). The advisor recognised the pattern (`feedback_bm_merge_unstable_admin_bypass.md`) and didn't treat it as an error.

- **Phase 8 workflow fan-out: `v0-polish-ignore` false alarm:** Workflow suggested updating `#[ignore]` file paths. On inspection, the tests were already in the correct post-Phase-6 location (`crates/server/tests/e2e/governance.rs`). Workflow classification was stale relative to the Phase 6 refactor — required a manual check before acting.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add `--no-fail-fast` to every GOLDEN_INVARIANT verification step in plan §15 DoD commands and in the Phase 7 plan template | Prevents undercounting when a flake short-circuits early; makes the true pass count visible before declaring invariant preserved | minor (edit plan template + brehon-reference.md note) | 2× (Phase 7 here + prior phase 6 run had same issue per retro §3) |
| 2 | Add a "move-not-delete" note to `.coderabbit.yaml`'s ADR scanner section (or its docs) clarifying that file moves generate false-positive `-` lines in per-file diff context | Reduces noise in future phase-based refactors; saves ~5 min per PR that touches a file-decomposition | minor (edit `.coderabbit.yaml` comment or add a note to the ADR scanner job's summary template) | 1× (Phase 6 decomposition; likely to recur on any future module split) |
| 3 | Before acting on any workflow fan-out "stale/delete" verdict, the advisor must grep-verify the specific claim before editing | Prevents false-positives like the `v0-polish-ignore` stale path claim — workflow saw old context, file was already correct | zero-cost (process discipline; document in `feedback_workflow_fanout_grep_verify_before_act.md`) | 2× (Phase 8 `v0-polish-ignore` + Phase 8 federation cron TODO — both required grep confirmation before acting) |

## What to carry forward

- **Grep-before-act on workflow fan-out verdicts.** Workflow agents can return stale classifications if the codebase changed since the state they reasoned about. For any "stale/delete" verdict touching live code paths, always grep to confirm before editing. Phase 8 applied this correctly for the federation cron TODO (confirmed wired at `scheduled_tasks.rs:405-465`) and the `v2-cleanup` protocol stubs (confirmed typed structs exist at `aa159a0f4`). Maintain this as an invariant, not a best-effort check.

- **ADR scanner false positives are PR-comment-addressable, not block-worthy.** When the scanner fires on a move-not-delete diff, the correct response is a maintainer comment explaining the false positive — not reverting the refactor or restructuring the diff to satisfy the scanner. The scanner is a heuristic; the reviewer reading the comment is the gate.

- **Phase 8 comment-hygiene taxonomy.** The five categories (`stale-can-delete`, `v2-cleanup`, `v2-deferred`, `defer-to-v2`, `correct-as-is`) provide a reusable classification framework for future TODO sweeps in governance crates. Worth preserving in the lesson corpus if a future session runs a similar sweep.

- **SL-b timing flake: documented, not fixed.** `calculated_at >= test_start` races PG↔Rust clocks at sub-ms resolution. The flake is pre-existing, rare, and the logic is correct. The clarifying comment added in Phase 8 (`sponsor_liability.rs:263`) is the right mitigation level for v1 — optional v2 hardening via `test_start - 1s` lower bound or small tolerance noted as carry-forward.

---

## Three-signal scoring

Per `feedback_four_role_retro_signals.md`. No Junior subagents dispatched this session (NO-ELITEDESK; all work was direct advisor).

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Phase 8 workflow fan-out (6-cluster parallel TODO audit) | 25 | 5 | low | Returned structured ledger in one pass; `v0-polish-ignore` verdict required manual grep correction (+5 min wasted) |
| Phase 7 e2e rerun with `--no-fail-fast` | 0 | 15 | medium | First run short-circuited at timing flake; second run confirmed GOLDEN_INVARIANT; `--no-fail-fast` should be default for invariant checks |
| ADR scanner PR comment (false-positive triage) | 10 | 5 | medium | Recognised move-not-delete false positive quickly; PR comment was sufficient; scanner noise is structural |
| Post-task retro `memory_write_eval` | 2 | 0 | none | Clean; single call, correct fields |
| Capstone retro authorship (Phase 0–8 summary) | 0 | 0 | none | Mechanical given the session's thorough in-line documentation |

## Complexity scores (heavy tasks only)

This session was advisor-only (no Junior workers). The "tasks" are direct advisor commits:

| Task | Files | Commits | Runtime (approx min) | Notes |
|---|---:|---:|---:|---|
| Phase 8 TODO sweep (comment-only) | 9 | 1 | ~30 | Low risk; workflow audit + grep verify before each edit |
| Phase 7 e2e validation | 0 | 1 (plan update) | ~40 | Two runs required; second confirmed GOLDEN_INVARIANT |
| PR #181 open + merge | 0 | 0 (PR lifecycle) | ~15 | ADR false-positive triage added ~5 min |

## Decisions to revisit

- **SL-b timing flake tolerance:** `test_start - 1s` lower bound or small clock tolerance. Noted as optional v2 hardening in the retro carry-forwards. Not urgent for v1; reconsider when starting v2-SL sub-phase.
- **ADR scanner diff-context limitation:** the scanner operates on per-file diffs and cannot see the matching `+` lines in sibling new files. A cross-file diff context (or a `--diff-filter=D` restriction to actual deletions) would eliminate the false-positive class. Low priority but worth raising as a config option if the `.coderabbit.yaml` ADR scanner supports it.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **`--no-fail-fast` for GOLDEN_INVARIANT assertions:** update plan template at `.claude/PRPs/templates/plan.template.md` §15 to note that e2e invariant checks require `--no-fail-fast`. Also add to `feedback_e2e_nextest_filter_groups.md` as a reminder. (2× recurrence: Phase 6 + Phase 7)
- [ ] **Workflow fan-out grep-verify discipline:** promote to `.claude/lessons/feedback_workflow_fanout_grep_verify_before_act.md`. Concrete rule: for any workflow agent verdict classifying a code path as "stale/delete/unused", run `grep` against the codebase before acting. (2× recurrence this session: `v0-polish-ignore` + federation cron TODO)

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
