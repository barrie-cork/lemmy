---
name: In brainstorm mode, log notes — don't start writing
description: During idea-generation sessions (user feeding successive conceptual points, no concrete artifact yet requested), capture each point to a running notes file or inline summary and summarise back — do NOT begin writing/editing deliverable files per message. Batch the wiring into one consolidated pass once the user signals "now write". Use AskUserQuestion for clarity when a point is ambiguous instead of guessing.
type: feedback
---

When a session is clearly **brainstorming / ideation** — the user is feeding a
stream of successive conceptual points, refining a framing, with no concrete
artifact yet requested — **log each point as a note and summarise it back; do not
begin writing or editing deliverable files per message.** Batch the actual wiring
into ONE consolidated pass once the user explicitly signals "now write" / "wire
these in".

**Why:** In the 2026-06-27 comms-research session the user fed ~14 essence-
clarifications across the session. I treated the first several as "edit the files
now," which produced churn — re-editing the same files after each new point, and
mid-stream cross-file inconsistency. The user corrected the style explicitly mid-
session ("just record my points... then summarise them back. It's only then that
we decide to do the writing, because going back and forth and writing is probably
confusing for you"), and reinforced it in the retro ("when brainstorming, log
notes rather than begin writing tasks"). The later consolidated wiring pass
(themes 11–15 batched at the end) was clean and fast — that is the model. Writing
per-message during ideation burns context, multiplies commits, and forces
re-edits when the next point reshapes an earlier one.

**How to apply:**

1. **Detect the mode early.** Signals: the user is stating ideas/principles
   (not "do X"); points arrive successively; the deliverable is conceptual and
   still forming; no file/output was explicitly requested *yet*. At the first
   2–3 clarifications, **offer the mode proactively** — don't wait to be told:
   *"I'll log these as notes and summarise back as we go; say the word when you
   want me to write them into the files."*
2. **Capture + summarise back.** For each point: record it (running notes file or
   a tight inline summary), reflect it back in the user's own framing so they can
   confirm you got it, and flag any tension with earlier points. Keep a running
   "point bank."
3. **Use AskUserQuestion for clarity when a point is ambiguous** rather than
   guessing and writing. A wrong guess written into files is exactly the churn
   this lesson prevents.
4. **Defer writing; batch the wiring.** When the user signals "now write" (or
   ≥3 related conceptual edits are queued and stable), apply them in ONE
   consolidated pass at a natural checkpoint, then commit.
5. **Honesty carries through.** When a brainstormed point leans on a factual
   claim, record it as a *must-verify* note rather than settled fact (see
   `feedback_verify_automated_reviewer_claims_against_compiler.md` family) — but
   that's still a *note*, not a write.

**Boundary:** this is not "never write during a conversation." Once the user
requests a concrete artifact, or confirms the brainstorm is settled, normal
authoring resumes. The rule is specifically: during open-ended ideation, notes
first, writing batched and gated on an explicit go-signal.

**Generalises to:** any open-ended authoring/design session (messaging,
plan-shaping, naming, architecture sketching) where the user is thinking out loud
and the shape is still moving. Pairs with the up-front-AskUserQuestion scope-lock
pattern (`feedback_clarify_before_plan.md` is the planning-brief analogue).
