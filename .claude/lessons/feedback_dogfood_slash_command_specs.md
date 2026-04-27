---
name: Dogfood new slash-command specs against a real input before commit
description: When authoring a new slash command (workflow spec, not a bash script), do one mental walk-through against an actual existing input file (brief, plan, log) BEFORE commit. Catches prose-vs-mechanics drift the same way bash -n + real-target runs catch script bugs.
type: feedback
---

When authoring a new slash command — particularly one with multi-step workflow prose like `/brehon-clarify` or `/brehon-verify` — do one mental walk-through against a real existing input file before commit.

**Why:** 2026-04-27 spec-kit pattern adoption. Authored `/brehon-clarify` and `/brehon-verify` with detailed Step 1..N prose. Did not dogfood either against existing artifacts (`jm-d-impl-2.md` brief, `v1-jury-mechanics-d.plan.md`). Generalises `feedback_test_scripts_against_real_targets.md` from bash → spec-as-prose. The risk is the same: prose can describe a workflow that doesn't actually match the input shape, but reading prose alone won't surface the gap.

**How to apply:**

- After writing a new slash command spec but before commit: `Glob` for an existing input the command would consume (a brief, a plan, a log). Open one. Walk through the spec's Step 1..N against that real input. Note any spot where the spec's assumption doesn't hold (missing field, ambiguous parser, undefined edge case).
- Either fix the spec or document the limitation explicitly in the spec's `<rationale>` block under a "Pre-commit dogfood" sub-section. The dogfood is a one-paragraph note: which input was tried, what worked, what didn't.
- If the command is a slash-command for a workflow-with-side-effects (writes DQ, commits files, opens PRs), the dogfood step is mandatory — the cost of fixing post-deploy is 10× the cost of fixing pre-commit.

**Specific dogfood targets in Brehon:**

- New planning-stage command → most-recent planning brief at `.claude/PRPs/briefs/<phase>-planning-N.md`.
- New impl-stage command → most-recent impl brief at `.claude/PRPs/briefs/<phase>-impl-N.md`.
- New verification command → most-recent shipped plan at `.claude/PRPs/plans/<phase>.plan.md`.
- New BM verb → most-recent runlog entry at `.claude/runlog/<phase>.md`.

**Generalises to:** any spec-as-prose authorship (rules, agent contracts, command files, lessons that reference workflow steps). Prose lints catch typos; dogfood catches semantics.

**Symptom to recognise in retrospect:** a slash command's first real-world run produces an unexpected error or empty output, and the fix is a one-line clarification in the spec. That fix was catchable in a 5-minute dogfood.

**Brehon-specific application:**

- Per `.claude/rules/advisor-orchestrator.md` "Dogfood gate", the advisor must include a "Pre-commit dogfood" sub-section under `<rationale>` in every new slash-command spec authored under `.claude/commands/`. The sub-section names the dogfood input + what worked + what didn't. A new command without a dogfood note is a process miss.
