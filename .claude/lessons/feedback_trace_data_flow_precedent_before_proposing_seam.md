# Trace the data-flow precedent before proposing how to supply missing data

**Class:** advisor session discipline (plan-gap detection + seam design)
**Origin:** m2-rooms-a T2 brief authoring, 2026-06-06 (DQ a3d0e9941441-055)

## The rule

When a plan step assumes a piece of data is available to a consumer, and you
discover it is NOT, your FIRST move is to trace the **data-flow precedent**:
how does this same consumer obtain its OTHER data today? Answer that before
proposing any mechanism to supply the missing data. The precedent both
(a) confirms the gap is real and (b) almost always dictates the *shape* of
the correct fix — so skipping it risks proposing a fix that fights the
established pattern.

## Why (the incident)

Authoring the m2-rooms-a T2 brief, I found that `CaseTransitionEvent` (the
payload the Matrix bridge receives) carries only `case_id`/status/
`community_id`/`target_type` — no juror identities — yet plan §13 T2 requires
the bridge to invite "the 5 assigned jurors as `Juror-<suffix>`". The bridge
has no DB access. Real gap.

My first instinct was to propose "augment `CaseTransitionEvent` with
`juror_pseudonyms`". Only after a user nudge ("have a look at this again")
did I trace the precedent: the M1 DM-relay (`services/bridge/src/relay.rs`)
has the **binary push** `brehon_sender`/`brehon_recipient` into the notify
payload — the bridge NEVER reaches back to the binary to resolve identities.

That precedent did two things at once:
1. **Confirmed the gap** — the bridge-resolves-identities-itself model the
   plan implied has no precedent; the binary-pushes-identities model is the
   established one.
2. **Dictated the fix shape** — "augment the event payload" is the
   precedent-consistent option (it's literally what M1 does), while "add a
   GET-back route for the bridge to fetch jurors" fights the established
   no-GET-back model.

Had I traced the precedent first, I'd have reached the grounded conclusion
without the round-trip. The cost was low here (one user nudge), but the same
miss in an autonomous run would have shipped either an unimplementable brief
or a silently precedent-violating seam.

## How to apply

- Plan step says "the consumer does X with data D" but D isn't in the
  consumer's inputs → STOP. Before proposing how D gets there, `grep`/`Read`
  for how the consumer gets its *current* inputs (the analogous data already
  flowing to it).
- The precedent answers "is this a real gap?" (if the consumer fetches
  everything via GET-back, maybe a new GET-back route IS the pattern; if it
  receives everything pushed, a push is the pattern).
- Cite the precedent file:line in the DQ/brief — it converts a guess into a
  grounded recommendation and gives the planner/reviewer the same anchor.

## Boundary / when this fires

Fires at **brief-authoring and plan-gap-detection time**, in the advisor
session — not during impl (impl follows the plan's already-decided seam).
Companion to:
- [[feedback_falsifiable_hypothesis_before_structural_fix]] — verify the
  named defect site contains the suspect operation before structural work.
- [[feedback_handover_assumptions_need_empirical_verification]] — verify
  named-but-untested system properties before patching.

This one is narrower and earlier: it's about the *data-flow* shape, applied
the moment you notice a consumer is missing an input the plan assumes it has.

<!-- verified: relay.rs BridgeNotifyPayload push-model + CaseTransitionEvent
     integer-only fields confirmed on phase-m2-rooms-a @ 56de6da77, 2026-06-06 -->
