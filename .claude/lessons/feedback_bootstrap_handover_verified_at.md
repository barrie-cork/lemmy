---
name: Bootstrap handover VERIFIED_AT annotation
description: Bootstrap handover RESUME blocks must carry a VERIFIED_AT line citing the git SHA that confirms the "next action" is still outstanding; sessions that complete work must update it before exiting.
metadata:
  type: feedback
---

Bootstrap handover RESUME blocks must carry a `VERIFIED_AT: <SHA>` annotation on every "next action" or "current stage" claim. Without it, the handover is undated relative to the git log and may describe completed work as pending.

**Why:** In the 2026-06-01 state-reconciliation session, `v1-quality-r3b-bootstrap.md` described the phase as "not yet started" with a `NEXT concrete action: author planning brief`. The fix (`LemmyContext::database_url()`) was already live in the code and PR #170 had merged. The bootstrap was authored before the Docker spin-up session ran r3b to completion; that session treated the canonical checkout as read-only and never updated the handover. The pre-impl HEAD check (§3.1 advisor-orchestrator) was the only catch — it saved ~45 min of no-op authorship. `VERIFIED_AT` makes the staleness detectable without reading the code.

**How to apply:**

1. **When authoring a bootstrap handover**, every `NEXT concrete action` claim in the RESUME block must carry:

   ```markdown
   VERIFIED_AT: <short-SHA>  <!-- confirmed outstanding at this git log position -->
   ```

   The SHA is the HEAD at the time of writing — it proves the claim was checked against the actual code state, not just copied from a prior session's notes.

2. **When a parallel session (Docker spin-up, lane-B, etc.) completes work described in an existing bootstrap**, it MUST update the `VERIFIED_AT` line in that handover before exiting — or append `STATUS: SHIPPED <PR#>` to the RESUME block. A session that treats the canonical checkout as read-only has an obligation to flag the handover as stale via a minimal commit even if it commits nothing else.

3. **At session start, when reading a bootstrap handover**, treat the `NEXT concrete action` as a **hypothesis** (not a fact) and run the pre-impl HEAD check (`git show HEAD --stat` on the named files) before proceeding. This is already required by `advisor-orchestrator.md §3.1` — the `VERIFIED_AT` line helps you quickly calibrate whether the hypothesis is fresh (SHA close to current HEAD) or stale (SHA many commits behind).

**Companion:** `feedback_handover_assumptions_need_empirical_verification.md` covers the broader "handover claims are hypotheses" discipline; this lesson is the specific `VERIFIED_AT` annotation mechanic.

**bm-merge integration:** Phase 8.5 of `bm-merge.md` appends a tombstone to the bootstrap handover at merge time. This is the structural backstop — VERIFIED_AT is the author-time guard; the tombstone is the post-merge guard.
