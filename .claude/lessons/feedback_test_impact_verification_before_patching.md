---
name: Verify test-impact of semantic fixes before patching assertions
description: When a fix changes semantics that might drift existing test assertions, trace the test fixtures FIRST — don't pre-patch assertions or assume drift
type: feedback
originSessionId: 48b2e875-05b9-4267-92bc-7ff4c1e5d99c
---
On PR #7 Bucket 1 Fix 3, the snapshot-read change from `OR`-union to community-first/instance-fallback precedence COULD have shifted which rows the three `sponsor_liability_with_founder_multiplier` test branches read against. Before applying the fix, I read the test's `seed_snapshot` helper at `crates/server/tests/e2e.rs:1594-1614` and confirmed every branch writes `community_id: None` (instance-scoped only). Under the new precedence (community-first, instance-fallback), each branch's `Some(cid)` case misses the community lookup and falls through to the identical instance row — so existing assertions on `-50`, `-75`, `-100`, and post-recompute `0` stay valid. No test adjustment needed.

**Why:** If I'd either (a) patched assertions pre-emptively in anticipation of drift or (b) just shipped the code change and treated test failures as "expected drift, update the numbers" — both paths risk hiding real semantic regressions under cosmetic assertion churn. The correct path is: predict test behaviour by tracing fixtures, ship the code change, and ONLY touch assertions if the prediction was wrong AND investigation reveals the drift reflects intentional semantic change (not a regression).

**How to apply:** For any fix that changes read/write semantics touched by existing e2e tests:
1. Before editing code, grep for tests that exercise the affected path.
2. Read the test fixture helpers (seed_*, setup_*) to understand what the test actually writes.
3. Walk the test scenarios manually through the new code to predict pass/fail/value-shift.
4. Ship the code change; run the tests.
5. If prediction matches reality → done. If mismatch → investigate the mismatch BEFORE patching assertions. Test adjustment is sometimes correct (semantic change is intentional) but test regression is also a possibility (semantic change was wrong); don't conflate them.

**Pattern tie-in:** Advisor's explicit Order-of-Operations guidance on PR #7 Fix 3: "test adjustment ≠ test regression. Investigate carefully before patching."
