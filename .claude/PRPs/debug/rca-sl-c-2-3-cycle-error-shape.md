# RCA — v1-SL-c-2 Task 1 e2e error-shape: 3-cycle catch-fire

**Date:** 2026-05-09
**Sub-phase:** v1-SL-c-2 (sponsor-liability — c-2 sibling of c-1)
**Site:** `crates/server/tests/e2e.rs:12041-12108`
**Resolution:** re-plan to uniform `LemmyResult<T>` throughout, mirroring v1-SL-b canonical sibling.

## Summary

Three §G4 mechanical-fix cycles attempted on the same compile-error class at the same e2e.rs site, each producing a fresh failure variant. The failure pattern is not "Junior worker can't follow a recipe" — it is "the §G4 mechanical recipe family was too narrow to converge on this code shape". The recipe in `feedback_lemmy_error_no_std_error.md` (pre-2026-05-09) presumed a single recipe across all error-shape combinations; the reality requires case enumeration.

## Cycle history

| # | Brief | Approach | Workflow | Compile error | Why it failed |
|---|---|---|---|---|---|
| 1 | `sl-c-2-impl-1.md` | Plan §13 stub: `Result<(), Box<dyn Error>>` outer + no bridges | 25582548670 | 1× E0277 — `LemmyError: std::error::Error` not satisfied at `e2e.rs:12049` | Brief did not inject `feedback_lemmy_error_no_std_error.md` (file-class table didn't yet require it for impl-task; only fix-impl). Plan §13 stub itself prescribed wrong outer (Box<dyn Error>) when v1-SL-b §9.2 hand-off documented `LemmyResult<()>`. |
| 2 | `sl-c-2-fix-impl-1.md` | Flip outer to `LemmyResult<()>` + **forbid** helper updates | 25595869651 | 11× E0277 — `Send/Sync/Sized` trait bounds cascading at `e2e.rs:12069-12122` | Brief inverted the §G4 canonical recipe (`feedback_lemmy_error_no_std_error.md` pre-amendment). Helpers stayed `Result<T, Box<dyn Error>>`; test fn flipped to `LemmyResult<()>`. Result: Case C (mixed shapes) — `?` propagation across the helper boundary cannot bridge `dyn Error` (no Send/Sync/Sized) into `LemmyError` (requires Send/Sync/'static). |
| 3 | `sl-c-2-fix-impl-2.md` | Revert outer to `Result<(), Box<dyn Error>>` + bare `.map_err(\|e\| format!("{e}").into())?` bridges at 5 Lemmy-native call sites | 25603848858 | 1× E0283 — `Into<_>` cannot infer target through abstract trait object at `e2e.rs:12049` | Brief copied §G4 row 4 verbatim (per the anti-paraphrase gate added at commit 558cef1a5). The recipe text was correct *for a concrete outer error type* but wrong *for an abstract trait object outer* (`Box<dyn Error>`). Compiler cannot resolve `_` in `Into<_>` because the target is a trait object. The recipe family was missing the case enumeration. |

## Root causes

### RC-1 — Planner-side: plan §13 stub ignored v1-SL-b §9.2 hand-off

**Evidence:** `.claude/PRPs/plans/v1-sponsor-liability-c-1.plan.md` §9.2 line 791 documents: "the fixture-mod section [for `mod v1_sl_b_fixtures`] is non-binding for c-1 since c-1 ships no test code; it remains as documentation for c-2." This was an explicit canonical-shape hand-off to v1-SL-c-2.

The v1-SL-c-2 plan §13 Task 1 stub prescribed `Result<(), Box<dyn Error>>` for the test fn AND `Result<T, Box<dyn Error>>` for the 4 helpers — directly contradicting the c-1 hand-off. The planner-side miss introduced a 3-cycle impedance mismatch that no mechanical recipe could resolve.

**Why missed:** the v1-SL-c-2 planning stage didn't surface the c-1 §9.2 hand-off. The advisor's plan-approval gate (DoD smoke + watchpoint specificity) does not include "stub-shape uniformity check across sibling fixtures modules" as a watchpoint.

### RC-2 — Lesson-side: recipe lacked case enumeration

**Evidence:** the pre-2026-05-09 body of `feedback_lemmy_error_no_std_error.md` prescribed exactly one recipe: `.map_err(|e| format!("{e}").into())` against `Result<(), Box<dyn Error>>` outer. The lesson did not enumerate:

- **Case A** — uniform `LemmyResult<T>` throughout (preferred, canonical-sibling shape). This is the v1-SL-b shape that already shipped.
- **Case B** — uniform `Box<dyn Error>` throughout, requiring **annotated** `.map_err` closures because abstract trait objects defeat type inference (E0283).
- **Case C** — mixed shapes (hard refusal — re-plan, do not bridge).

The §G4 classifier's row 4 cited the lesson but inherited the same single-recipe assumption. When Junior #160 applied the recipe verbatim per the anti-paraphrase gate, the recipe was structurally insufficient for the abstract-outer case.

### RC-3 — Advisor-side: brief authorship at fix-impl-1 inverted both halves of canonical recipe

**Evidence:** `sl-c-2-fix-impl-1.md` prescribed flipping test fn signature to `LemmyResult<()>` + forbidding `.map_err`. Both halves were the inverse of the §G4 row 4 recipe. The RCA at `.claude/PRPs/debug/rca-sl-c-2-fix-impl-1-wrong-recipe.md` (cycle 2's existing RCA) documented this as a brief-authorship lapse.

The anti-paraphrase gate (`.claude/rules/advisor-orchestrator.md` §5.3 "Mandatory verbatim §G4 row in fix-impl briefs", added at commit 558cef1a5) prevents this specific class of lapse. fix-impl-2 used the gate and copy-pasted the row text verbatim — but cycle 3 still failed because the row text itself was insufficient (RC-2).

The anti-paraphrase gate is correct as far as it goes: it makes "I read the row but prescribed something different" structurally impossible. It does NOT detect "the row is too narrow for this code shape" — that requires case enumeration in the lesson + row split in §G4.

### RC-4 — File-class table cited the lesson but the lesson body was wrong-shaped

**Evidence:** the file-class lesson injection table (`.claude/rules/advisor-orchestrator.md` §2.4) added an `crates/server/tests/e2e.rs` row that mandates `feedback_lemmy_error_no_std_error.md` injection. Cycle 3's brief correctly injected it. But the lesson body itself was wrong-shaped (RC-2), so injection alone wasn't enough.

This is a "garbage in, garbage out" failure: the file-class table gates the right action (read this lesson) but doesn't verify the lesson body answers the right question. The lesson amendment (case A/B/C enumeration) closes this by giving the lesson body discriminating power.

## What the cycles cost

- Wall-clock: ~17 minutes per cycle × 3 = ~51 min compile time, plus brief-authoring + ci-watcher dispatch + DQ raise/mutate per cycle.
- Push grants: 4 (impl-1 commit, fix-impl-1 commit, fix-impl-2 commit, this re-plan).
- Junior tasks dispatched: #154 (impl-1), #155+#156 (failed ci-watchers due to daemon ref-fetch bug), #157 (failed fix-impl-1 due to same), #158 (fix-impl-1 retry, failed cycle 2), #159 (ci-watcher-2), #160 (fix-impl-2), #161 (ci-watcher-3).
- Total context spend: ~4 advisor session resumes (compaction events at end of c-1 retro, end of c-2 plan-approval, mid-cycle 2, post-compact resume #4).
- Workflow runs: 3 cargo-validate-workspace (≥17m each = ~51 GH-Actions runner-minutes).

## Resolution

**Re-plan to uniform `LemmyResult<T>` throughout** (Case A), mirroring the v1-SL-b canonical sibling at `crates/server/tests/e2e.rs:11001-11924` verbatim.

Specifically:

1. **Hard-reset** the impl-1 worker branch (`origin/junior/role-impl-task-sl-c-2-impl-1-...-md-154`) to `c2761b284` (original Task 1 commit), wiping `4be7b7f95` (cycle 2 fix), `cb9b7fe49` (cycle 3 fix), and the 4 DQ raise/mutate commits on top. Force-push with-lease.
2. **Replan brief** at `.claude/PRPs/briefs/sl-c-2-impl-1-replan.md` prescribing the uniform `LemmyResult<T>` shape — test fn signature `LemmyResult<()>`, all 4 helpers `LemmyResult<T>`, no `.map_err` bridges, mirrors v1-SL-b imports + propagation pattern.
3. **Lesson amendment** at `.claude/lessons/feedback_lemmy_error_no_std_error.md` adding cases A/B/C with explicit recipes per case + the v1-SL-c-2 surfacing.
4. **§G4 row 4 split** into 4a (uniform LemmyResult flip), 4b (annotated .map_err for Box<dyn Error> outer), 4c (mixed shapes — refuse auto-fix; surface for re-plan).
5. **File-class table note** that when a v1-SL-* fixtures sibling exists in the same file, mirror its case verbatim (canonical-schema-first gate).
6. **Plan §17 amendment** (carry-forward to retro) noting Tasks 2-5 stubs already use `LemmyResult<()>` per c-1 hand-off; only Task 1 needed the re-plan.
7. **DQ supersession**: DQ #164, #165, #166 stay where they are with a `superseded_by_replan: true` flag; advisor commit subject `docs(decision-queue): supersede DQ #164 #165 #166 — replan v1-SL-c-2 Task 1 error shape`.

## Carry-forward to retro

For the v1-SL-c-2 sub-phase retro (deferred until end of c-2 ship):

- **Planner signal**: plan §13 stub uniformity check is missing from the watchpoint specificity gate. Add to `feedback_advisor_watchpoint_specificity.md` or new `feedback_plan_stub_uniformity_with_canonical_sibling.md`.
- **Advisor signal**: the file-class table fires the right lesson but doesn't verify the lesson body answers the question. Future amendment: file-class table rows should cite specific case enumerations, not just lesson paths.
- **Lesson signal**: case enumeration is the missing dimension; this RCA is the first time it's captured. Lesson body now contains it.
- **Per-task complexity score** (per `feedback_retro_task_complexity_score.md`): Task 1 effective LOC delta = ~30 lines code, but **3 cycles × 1 ci-watcher each = 3 fixed-cost wall-clock + brief-authoring overhead**. Real complexity score for retro = max-cycle-count × per-cycle wall-clock, not LOC.

## See also

- `.claude/PRPs/debug/rca-sl-c-2-fix-impl-1-wrong-recipe.md` — cycle 2 RCA (now superseded; this RCA covers all 3 cycles).
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — amended lesson with case enumeration.
- `.claude/PRPs/briefs/sl-c-2-impl-1-replan.md` — the re-plan brief.
- `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §17 — carry-forward note.
- v1-SL-b canonical reference: `crates/server/tests/e2e.rs:11001-11924`.
