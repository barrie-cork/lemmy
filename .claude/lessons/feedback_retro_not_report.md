---
name: Single-pass phases still need short-form retros
description: Completion reports are not retros — they lack retrospective framing (what surprised, what to change, what to carry forward). Rule 12 says "short form always" including zero-incident phases. Phase 3 missed this.
type: feedback
originSessionId: d1ae4829-750e-4b63-bcff-517e614b070b
---
Single-pass phases still need short-form retros — the completion report is not a retro, it lacks the retrospective framing (what surprised, what to change, what to carry forward). Rule 12 says "short form always" and that includes zero-incident phases.

**Why:** Phase 3 was single-pass with zero incidents. Both advisor and impl treated the completion report as the retro. The two real lessons (serde dep catch, clippy wrapper fix) were captured in the phase-complete file but not in retro format. The knowledge didn't get lost this time, but the discipline slipped — and without it, zero-incident phases silently erode the retro habit right when Phase 4 needs it most.

**How to apply:** At every sub-phase close and phase close, produce a short-form retro (3 questions: surprised / change / carry forward) as a distinct section — not folded into the completion report. Even "nothing surprised us, no changes needed" is a valid retro that confirms the process worked. The impl agent's completion report is the factual record; the advisor's retro interprets it.

**Harvest step (added 2026-04-26).** When authoring the retro, the advisor also runs the lesson harvest per `feedback_junior_pmd_write_convention.md` — scan the phase branch's commits for `LESSON:` trailers, cross-reference with DQ resolved entries, decide promote/skip/augment for each signal, and add a `## Lessons promoted this phase` section to the retro file listing what was promoted. This step closes the loop between Junior subagents (who can't write to PMD directly) and the durable PMD/`.claude/lessons/` knowledge layer.

**Per-role structure under the four-role model (added 2026-04-26).** When the completing phase ran under advisor + planning + impl + BM, structure each H2 section by role per `feedback_four_role_retro_signals.md` — otherwise the loudest role's signals dominate and orchestration learnings get lost.

**Per-task complexity score (added 2026-04-27).** Every impl-task sub-section in the retro must include a one-line complexity score — `<files>/<commits>/<runtime-min>/<max-log-silence-min>` — per `feedback_retro_task_complexity_score.md`. The metric surfaces planning-side bundling drift before it costs a watchdog kill. Aggregate scores across the sub-phase in §5 (Quantified outcomes); flag any task that scored >55min runtime, >40min log silence, or >8 files touched as a carry-forward signal for the next plan.
