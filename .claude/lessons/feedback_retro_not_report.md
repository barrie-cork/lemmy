---
name: Single-pass phases still need short-form retros
description: Completion reports are not retros — they lack retrospective framing (what surprised, what to change, what to carry forward). Rule 12 says "short form always" including zero-incident phases. Phase 3 missed this.
type: feedback
originSessionId: d1ae4829-750e-4b63-bcff-517e614b070b
---
Single-pass phases still need short-form retros — the completion report is not a retro, it lacks the retrospective framing (what surprised, what to change, what to carry forward). Rule 12 says "short form always" and that includes zero-incident phases.

**Why:** Phase 3 was single-pass with zero incidents. Both advisor and impl treated the completion report as the retro. The two real lessons (serde dep catch, clippy wrapper fix) were captured in the phase-complete file but not in retro format. The knowledge didn't get lost this time, but the discipline slipped — and without it, zero-incident phases silently erode the retro habit right when Phase 4 needs it most.

**How to apply:** At every sub-phase close and phase close, produce a short-form retro (3 questions: surprised / change / carry forward) as a distinct section — not folded into the completion report. Even "nothing surprised us, no changes needed" is a valid retro that confirms the process worked. The impl agent's completion report is the factual record; the advisor's retro interprets it.
