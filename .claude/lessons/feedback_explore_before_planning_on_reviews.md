---
name: Explore before planning on review-response work
description: Parallel Explore agents before finalising CodeRabbit review-response plans pay off — they catch over-scope, reuse misses, and latent-vs-urgent framing
type: feedback
originSessionId: 48b2e875-05b9-4267-92bc-7ff4c1e5d99c
---
On PR #7's Bucket 1 planning (5 Major findings), three parallel Explore agents surfaced material corrections that would have shipped wrong without them:

1. **Over-scope avoided**: CodeRabbit suggested changing `count_active_founders` to return IDs for reseed-of-active-founder detection. Explore showed that's a helper-API signature change distinct from the Finding 5 scope (CLI input validation). Became a Phase 5c deferral, not a Phase 5b patch.
2. **Reuse miss caught**: for the snapshot scope-precedence fix, Explore found `read_reputation_summary` at `crates/db_views/reputation/src/impls.rs:86-128` already uses the canonical `match community_id` two-query pattern with `.optional()?`. The fix mirrors that exactly rather than inventing a new shape.
3. **Urgency framing adjusted**: for the founder_seed filter, Explore confirmed that *only* `seed_founders` writes `EndorsementStrength + non-null expires_at` today. The fix became defence-in-depth against future drift rather than an acute bug — which changed how it was framed in the commit body and inline reply.

**Why:** CodeRabbit findings look urgent by default ("Major" badge, confident prose). Without the codebase verification pass, it's easy to either over-patch (accept every suggestion as-is) or under-patch (miss that a stated "fix" violates an existing invariant like decision-queue #16). Parallel agents give the evidence to frame each finding correctly.

**How to apply:** When responding to PR reviews with ≥3 medium-or-major findings, spawn up to 3 parallel Explore agents before finalising the plan — one per non-trivial finding, each asking: "does an existing pattern cover this? what's the full blast radius? what's urgent vs. latent?" Skip for ≤2 findings or purely mechanical doc nits.

**Pattern tie-in:** Complements `feedback_pr_review_triage_pattern.md` (4-bucket triage) and `feedback_subagent_model_and_effort.md` (use `model="opus"` + max-effort when spawning agents).
