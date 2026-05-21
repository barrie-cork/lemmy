---
name: retro-harvest folded into weekly-review Step 2c (Sunday cadence)
description: Pre-v1-rls-r1 the .claude/skills/retro-harvest/SKILL.md skill ran ad-hoc and ~48 proposals accumulated unread across 52 retros. v1-rls-r1 added Step 2c to weekly-review/SKILL.md so retro-harvest fires on the same Sunday 02:00 UTC cadence as backfill and lesson promotion. One weekly artifact at .claude/harvest/<iso-week>.md (gitignored). SURFACING not auto-promoting.
type: feedback
---

# retro-harvest weekly cadence

## Pre-v1-rls-r1 state

Per RLS-PMD review §4.6 evidence: ~48 retro proposals accumulated unread across 52 retros (≈92% surfaced-but-unconsumed rate). The `.claude/skills/retro-harvest/SKILL.md` skill existed and worked, but ran only when the user explicitly invoked it. Without a cadence the corpus of "things a future advisor on related code would have wanted to know" sat invisible. The cycle_count ≥ 3 catch-fire proposal in `feedback_plan_stub_uniformity_with_canonical_sibling.md` sat 7 days before the v1-SL-c-2 cycle-3 incident (~123 min loss); had Step 2c existed, the proposal would have surfaced in that week's harvest and the §G4 hard-refusal would have shipped before the incident.

## Post-v1-rls-r1 state

Track B Task 4 inserted Step 2c between existing Step 2b and Step 3 of `.claude/skills/weekly-review/SKILL.md` — zero downstream renumber per DQ #296. Step 2c folds the existing retro-harvest skill body into the weekly cadence: Glob `.claude/PRPs/reports/*.md` for the prior 7 days, extract proposals not yet promoted to `.claude/lessons/`, write one artifact at `.claude/harvest/<iso-week>.md` (gitignored per Task 5; runtime-journal semantics). The step body emphasises "SURFACING, not auto-promoting" — promotion to `.claude/lessons/` or `CLAUDE.md` is a human decision after reading the harvest. Sunday 02:00 UTC cadence aligns with backfill + lesson promotion + scheduled audits.

## How to apply

- **Weekly cadence:** Step 2c fires automatically as part of the weekly-review run. No manual invocation needed.
- **Manual review thereafter:** the human reads the weekly harvest artifact and decides promotions per the RLS-PMD review §4.6 contract. SURFACING does not bypass human judgment — it makes that judgment cheap by collecting the candidate set in one place.
- **Quarterly trend:** count of surfaced-but-unpromoted proposals week-over-week — rising count is a calibration-honesty signal worth investigating (either proposals are increasingly speculative, or promotion discipline is slipping).

## See also

- `.claude/skills/retro-harvest/SKILL.md` — the consumed skill
- `.claude/skills/weekly-review/SKILL.md` Step 2c (Task 4) — the cadence call point
- `docs/research/brehon-rls-pmd-review.md` §4.6 — originating recommendation
- `.claude/PRPs/plans/v1-rls-r1.plan.md` — this plan
