---
name: Four-role retros need per-role signals
description: Under the four-role model (advisor + planning + impl + BM), a single "what surprised us" lump captures only the loudest role and loses orchestration-specific signals. Sub-phase retros must reflect per-role.
type: feedback
---

Under the four-role model (advisor + planning + impl + BM), the canonical retro template's three H2 sections (`## What surprised us / ## What to change / ## What to carry forward` per `feedback_retro_not_report.md`) capture only the loudest role's signal — usually impl, since impl produces the most commits — and lose the orchestration-specific signals that the model exists to deliver.

**Why:** the canonical retro lesson was authored for the single-session model where one CC session owns planning, impl, and BM together; "what surprised us" naturally reflects on all three. With the roles split, the signals fragment:

- **Advisor sees:** brief sufficiency, DQ triage decisions, user-gate moments, polling-loop friction, four-role transitions
- **Planning sees:** brief-to-plan translation, design-doc gaps, watchpoint specificity outcomes, DoD smoke-test results
- **Impl-task sees:** MIRROR-ref accuracy, per-task validation gates, lessons-injection misses, test-substitution decisions
- **BM-task sees:** CR triage clarity, finding-bucket distribution, runlog completeness, attribution integrity

If the retro doesn't separate by role, retrospective questions degenerate to whichever role's signals dominated the artifact trail (commit volume, finding count). The orchestration-side learnings — was the brief sufficient, did dispatch-line conventions work, did mid-task DQ push surface entries promptly, did per-task isolation cause coordination friction — get lost.

**How to apply:** at every v1 sub-phase close (and any future v0 phase that runs under the four-role model), the retro file at `.claude/PRPs/retros/<phase-id>-retro.md` should structure each H2 section by role. Two acceptable shapes:

1. **Single-section, per-role bullets:** under `## What surprised us`, write four sub-bullets prefixed `**Advisor:**`, `**Planning:**`, `**Impl:**`, `**BM:**`. Repeat for the other two H2 sections. Cheapest format, works when most sections have signals.

2. **Per-role H3 subsections:** under each H2, create `### Advisor` / `### Planning` / `### Impl` / `### BM`. More verbose but easier to write when one role has multiple signals to record.

Either shape satisfies the discipline. "Nothing surprised the planning subagent" is a valid empty answer — the structure makes the absence visible. The retro author (advisor) uses both Junior commit-message bodies (per the `LESSON:` trailer convention in `feedback_junior_pmd_write_convention.md`) and DQ resolved entries as the raw input for the per-role signals; the runlog at `.claude/runlog/bm-runlog.md` is the BM-side input.

**Per-role retro questions** (suggested checklist, not mandatory wording):

- **Advisor:** Was the brief sufficient or did Junior have to guess? Did DQ entries arrive promptly via mid-task push? How many user-gate interruptions vs autonomous transitions? Did the polling cadence (~10 min default) match the work pace? Did any catch-fire procedures fire?
- **Planning:** Did the canonical plan template match the sub-phase shape? Were watchpoints actionable per `feedback_advisor_watchpoint_specificity`? Did MIRROR refs the plan named actually demonstrate the pattern when impl read them? Did the DoD smoke-test commands execute as written?
- **Impl-task:** Did MIRROR refs hold up under the change? Were per-task validation gates correct? Did `Glob .claude/lessons/` surface relevant prior patterns? Did any lesson get learned that belongs in PMD (write a `LESSON:` trailer per the convention)?
- **BM-task:** Was the four-bucket CR triage clean? Did the runlog capture state changes? Did any attribution integrity rule come close to firing?

**Meta-retro at sub-phase close (do once per lane, not per slice):** the orchestration model itself deserves a retro every few sub-phases — "did the four-role split actually deliver autonomy or did it just move the work around?" Cadence: at lane-close (e.g. when v1-JM-d ships and the JM lane is done), not at each slice-close.

**Out of scope for this lesson:** the retro file location and the three required H2 sections are still governed by `feedback_retro_not_report.md`. This lesson is the *content* discipline for sub-phases that ran under the four-role model. The retro gate in `/brehon-phase-transition` Step 0 still only blocks on the three H2 headers being present — per-role structure is advisory (the skill nudges the user but doesn't refuse to transition).
