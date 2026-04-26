# Evaluation Calibration — Anti-Inflation Guard

Complements post-task-retro and `/reflect` scored evals with calibration guidance to prevent score drift. Does not replace or override the scoring rubric — it adds boundary checks.

## Typical score range

Eval scores should cluster around 0.60-0.75. If your score is above 0.85, re-read the scoring rubric — you are likely inflating.

Reference scores:
- **0.60**: Task completed but had a retry loop, scope issue, or user correction
- **0.70**: Clean completion, clear goals, no blockers, no coverage gaps
- **0.80**: Clean completion with all outputs verified, no issues at all

## Anti-inflation rules

1. **Read-only sessions**: If there are 0 commits and no file changes, you cannot claim the full "verified" signal. Max score ~0.70.
2. **Trivial tasks**: Simple config changes, find-replace, or single-file edits should score 0.65-0.75 at most. A perfect trivial task is still trivial.
3. **Never score 1.0**: The rubric caps at 1.0 but realistic scores should never reach it. A score of 1.0 means literally nothing could have been better — this is never true.

## Improvement surfacing threshold

For tasks scoring below 0.75, you MUST include at least one concrete improvement suggestion — something that would prevent the same issue next time (a skill change, a guard, a checklist item, a new automation).

For tasks scoring 0.75+, improvement suggestions are encouraged but not mandatory.

## Enforcement

- Apply during post-task-retro Step 2 and `/reflect` session eval
- If your initial score exceeds 0.85, re-evaluate against these rules before writing the memory
- The feedback loop depends on honest scoring — inflated scores hide real problems
