---
name: Plan §13 task stubs must mirror canonical sibling fixtures shape
description: When a plan §13 task stub adds code to a file that already contains a canonical-sibling fixtures module (v1-SL-*, v1-JM-*), the stub MUST mirror the sibling's return-type shape, error-handling pattern, and import set. Surfaced at v1-SL-c-2 Task 1 3-cycle catch-fire 2026-05-09.
type: feedback
---

When a Brehon plan's §13 task stub touches a file that already contains a canonical-sibling fixtures module (e.g. `mod v1_sl_b_fixtures` for sponsor-liability tests, `mod v1_jm_e_fixtures` for jury mechanics), the stub MUST inherit the sibling's:

- Test fn return-type signature (e.g. `LemmyResult<()>`, NOT `Result<(), Box<dyn Error>>`).
- Helper fn return-type signatures (uniform with the test fn — Case A or Case B per `feedback_lemmy_error_no_std_error.md`, never Case C mixed shapes).
- Import set (e.g. `use lemmy_utils::error::LemmyResult;`, NOT `use std::error::Error;`).
- `?` propagation pattern (bare `?` for Case A, annotated `.map_err` closures for Case B).

**Why:** sibling fixtures modules in the same file are a hand-off contract. v1-SL-b's plan §9.2 explicitly noted "the fixture-mod section is non-binding for c-1 since c-1 ships no test code; it remains as documentation for c-2" — that's a direct hand-off from c-1 to c-2. The c-2 plan §13 Task 1 stub ignored the hand-off and prescribed `Result<(), Box<dyn Error>>` outer + `Result<T, Box<dyn Error>>` helpers (Case C mixed shapes), introducing a 3-cycle impedance mismatch that no §G4 mechanical recipe could resolve. Replan cost: 3 ci-watcher cycles + 5 advisor session resumes + ~80 min cargo wall-clock for what should have been a single ~6-line change.

**How to apply:**

1. **Planner side (when authoring §13 task stubs):** before finalising any §13 stub that adds code to `crates/server/tests/e2e.rs` or any other file with a v1-* fixtures sibling, `Glob` + `Read` the canonical sibling's full module body. Quote 1-2 helper signatures + the test fn signature + the import block in §13's "Required reading" or §10 "Spec hand-off from prior phase" section. The stub's signatures MUST match.

2. **Advisor side (at plan-approval gate):** before approving any plan whose §13 touches a file containing existing v1-* fixtures modules, grep for `mod v1_<area>_<letter>_fixtures` in the file and cross-check the §13 stub against the matched module's signatures. If the stub introduces a new return-type shape (e.g. `Box<dyn Error>` outer when sibling uses `LemmyResult<()>`), file a planner-side DQ before approval.

3. **Symptom of a miss:** §G4 catch-fire on E0277 (or E0283 with abstract trait object outer) at the new test fn's first `?` propagation site. If 1+ cycles fail on the same site with shape-mismatch errors, the root cause is almost always a stub uniformity miss — re-plan, don't keep cycling §G4 mechanical fixes.

**Generalises to:** any plan §13 task that adds code to a file with existing canonical-sibling patterns — not just e2e fixtures modules. Migration files, handler modules, view modules, newtype modules. Any time a planner's stub introduces a shape that diverges from a sibling already shipped, the gate should fire.

**Symptom-recognised at v1-SL-c-2 cycles 1-3 (workflow runs `25582548670` / `25595869651` / `25603848858`)** — full RCA at `.claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md`. The 3-cycle catch-fire was directly traceable to the §13 stub mismatch with v1-SL-b's `mod v1_sl_b_fixtures` shape (e2e.rs:11001-11924).

## See also

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A/B/C enumeration the sibling-mirror picks from.
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — companion gate (watchpoints cite specific tables/files); this lesson extends to canonical-sibling shape.
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — generalisation to spec/template/rule authoring.
- `.claude/rules/advisor-orchestrator.md` §2.4 file-class lesson injection table — the canonical-sibling parenthetical points back to this lesson.
- `.claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md` — the surfacing RCA.
