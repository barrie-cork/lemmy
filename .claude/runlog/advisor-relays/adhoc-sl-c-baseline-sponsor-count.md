---
id: adhoc-sl-c-baseline-sponsor-count
from: advisor
to: impl
ts: 2026-05-07T12:30Z
relates_to: v1-SL-c plan / brehon-clarify gate; PR #119 cr-6
decision: surface as clarify-DQ
---

# Decision

The `baseline_sponsor_count` mechanic for `escape_rule = "majority_revocation"` belongs in v1-SL-c's grace-check evaluator, not v1-SL-b's synchronous handler. SL-b ships a narrow handler-only fix (correct the obviously-wrong `pre_count = active_sponsor_count + 1` derivation) plus a 10th e2e for the synchronous escape path; SL-c's plan-author resolves the authoritative persistence shape via the clarify gate.

# Instructions

1. Add a clarify-DQ entry on the SL-c brief asking: "When a case transitions `Decided → SponsorLiabilityPending`, where does the count of sponsors-at-decision-time live so the grace-check evaluator can apply `majority_revocation` correctly?" with three concrete options:
   - **(A) New column on `moderation_case`**: add `baseline_sponsor_count INTEGER NOT NULL DEFAULT 0` in the SL-c migration; set during the `Decided → SponsorLiabilityPending` transition (already authored in `submit_jury_vote.rs::process_vote` per registry SL-a entry for `_PENDING`).
   - **(B) Derive at evaluation time** from `surety` rows existing at `moderation_case.decided_at`: query active sureties with `endorsement_started_at <= decided_at`. No migration needed; more complex query.
   - **(C) Defer to v1-SL-d or later** if the grace-check evaluator can correctly evaluate without it (e.g. by relying on `any_revocation` as default escape rule and treating `majority_revocation` as a v2 capability).
2. The clarify-DQ should cite this relay (`relates_to: adhoc-sl-c-baseline-sponsor-count`) so the resolution is traceable.
3. Whichever option is chosen by SL-c clarify resolution, update PR #119's cr-6 finding `notes` field with the cross-reference once SL-c lands.
4. SL-b cr-6 fix-impl-task does NOT block on this — SL-b's handler comment will explicitly defer the authoritative mechanic to SL-c with a `// TODO(v1-SL-c)` marker citing this relay.

# Retro carry (v1-SL-b retro)

- Pattern: cross-sub-phase coordination via advisor-relay → clarify-DQ. First case where SL-b CR finding directly informs SL-c plan scope.
- Watch for: similar coordination patterns when CR findings on phase-N reveal scope properly belonging to phase-N+1.

# Next

SL-c clarify-DQ author absorbs (A)/(B)/(C) options into next clarify pass; resolution lands in SL-c plan §5/§13 before any handler edit on the Mac side.
