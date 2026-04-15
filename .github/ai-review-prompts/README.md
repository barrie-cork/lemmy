# AI review prompts

Source of truth for the ADR rubric and per-phase rubrics consumed by
`.github/workflows/governance-ai-review.yml` (and the dormant
`claude-code-action.yml.disabled`). Every file cites ADR IDs from
`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` §2 and
section numbers from `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md`.

Keep each rubric under 200 lines so the 8K-input-token window on the free
GitHub Models tier is never exhausted. The workflow concatenates `adr-rubric.md`
with one detected-phase rubric, so the combined prompt plus the diff must fit.

Humans reviewing PRs should reference the same files — the goal is one source
of truth, not two copies drifting apart.
