---
name: PR review triage — four-bucket split
description: CodeRabbit reviews on large diffs need buckets (mechanical / rebuttal / in-phase / carry-forward) before patching
type: feedback
originSessionId: 07131c85-9c72-4cef-a469-5051bc6e326a
---
# Four-bucket PR review triage

For any CodeRabbit review with 15+ comments, split findings into four buckets before touching code:

1. **Bucket A — Mechanical fixes.** Markdown escapes, typos, dead code, duplicate JSON entries, dead comments, TODO-placeholder resolution, header-vs-implementation drift. Land as-is in one commit.
2. **Bucket B — Plan-accepted tradeoffs.** CodeRabbit flagging what the plan explicitly chose (e.g. `NotFound` error collapse, TOCTOU windows inside a transaction, deferred validation paths). Reply with `in_reply_to` pointing at the GOTCHA or plan section. Do NOT patch.
3. **Bucket C — Real in-phase defects.** Genuine bugs introduced by the current phase's code that can be fixed without design work. Patch as one follow-up commit.
4. **Bucket D — Carry-forward to next phase.** Real issues blocked on scope deferred to a later phase (e.g. a wrapper that's not written yet, an endpoint not yet shipped, hardening passes for later). Track in the phase completion report; don't fix now.

**Why:** Why the split matters:

- **Bucket B should NEVER be patched silently** — if a reviewer flags a plan-accepted tradeoff, leaving it unpatched without a reply looks like silent ignoring; posting a rebuttal demonstrates the plan was deliberate.
- **Bucket D should NEVER be forced into the current PR** — it's how 5a grows into 5a+5b+5c feature-creep. Document in the completion report and move on.
- **Mixing Bucket A with Bucket C bloats the review-response commit** — CodeRabbit sees mixed diff intent and re-reviews get noisier.

**How to apply:** For each comment, answer two questions:
- Is this a real defect? → Yes: C or D. No: B.
- Is the fix in-scope for the current phase? → Yes: A or C. No: D.

**Rebuttal format for Bucket B:** One aggregate top-level PR comment is simpler than per-comment `in_reply_to` replies. Structure:
- "Patched in `<sha>`: <A+C items>"
- "Plan-accepted — not patched: <B items with GOTCHA/plan links>"
- "Carry-forward to Phase N: <D items>"

**Existence proof:** Phase 5a PR #4 (2026-04-17) — 40 CodeRabbit comments split 11 A / 4 B / 8 C / 2 D = 25 legit + 15 trivial. Applied buckets A+C in one follow-up commit; posted one aggregate rebuttal for B; tracked D in `project_phase_5a_pr4_open.md` handover.
