---
name: MiniMax M-series prompting best practices
description: MiniMax M-series (M2.7/M3) official prompting best practices — apply when authoring any brief or preamble for a MiniMax-arm impl-task dispatch.
type: reference
---

MiniMax's official prompting best practices (source: `https://platform.minimax.io/docs/token-plan/prompting-best-practices`, fetched 2026-06-07). The page does **not** version-differentiate — one practice set covers all "Token Plan" models (M2.7, M3); there is no M2.7-vs-M3 selection guidance on it. Apply these whenever building a brief, runbook, or per-dispatch wrapper for a MiniMax arm (see [[minimax-trial-runbook]] §preamble).

**The eight load-bearing recommendations (verbatim where quoted):**

1. **Explain *why* a constraint matters.** "Explain why a constraint matters, the model can choose better tradeoffs." → This is the single most decision-relevant one for us: a constraint *listed without rationale* is one MiniMax may trade away. See [[cheap-model-arm-drops-adr-constraints]] — the m1-b task-4 arm dropped the ADR-015 gate exactly because the brief named it but didn't make it load-bearing.
2. **Put the task at the END of the prompt.** "Placing the task at the end of the prompt has the largest single impact on answer quality" (long-context behaviour). Context/refs/constraints first, the actual ask last.
3. **Labelled sections.** Use explicit headers — "Task / Context / Source / Constraints / Output format." Matches our brief §-shape already.
4. **The colleague test.** "Show your prompt to a colleague who has no context. If they would be confused, the model will be too."
5. **Examples beat abstract rules.** "A few well-crafted examples usually beat abstract style instructions." → cite a specific MIRROR ref / sibling implementation rather than describing the pattern in prose.
6. **Concrete output contracts.** Specify section names, table columns, hard bullet/length limits ("5 bullets maximum"), explicit scope boundaries.
7. **Explicit permission to refuse + source grounding.** "Provide explicit permission to refuse" and require citation/grounding — reduces the rationalise-it-away / hallucinate behaviour. For us: tell the arm to raise a DQ blocker rather than silently scope-out a constraint it can't satisfy.
8. **Tool/function-calling discipline.** Define each tool (name, purpose, inputs, return shape, failure behaviour). "Use parallel calls for independent read-only lookups." "Use tools only when they materially improve the answer" (avoid eagerness).

**Limits noted:** "long context windows for both input and output" (no hard token number stated); "keep system prompts concise — model may terminate early near capacity thresholds." Code-generation and hard-constraint-following are **not specifically addressed** on the page — which is itself the gap [[cheap-model-arm-drops-adr-constraints]] documents empirically.

**Model-version note:** our trial runbook upgraded the arm from M2.7 → M3 (2026-06-07, `MiniMax-M3` model ID) per user instruction; M2.7 remains a fallback override. First-party benchmarks (`.claude/PRPs/reports/image.png`) show near-Sonnet parity on impl-shaped SWE-bench at ~10× lower cost. Our AB data (n=2, [[cheap-model-arm-drops-adr-constraints]]) shows that parity holds on pure pattern-following (DTOs) but breaks on logic-with-embedded-ADR-constraint — which best-practice #1 directly targets.
