---
name: Merge-forward e2e.rs conflict — default to governance-v0 side
description: When a merge-forward conflicts on crates/server/tests/e2e.rs and the phase branch's only changes are appended comments, #[ignore] attributes, or DISABLE_* markers (no test-logic changes), take the governance-v0 side — it reflects the latest correctly-integrated test configuration.
type: feedback
---

# Merge-forward e2e.rs conflict: default to governance-v0 side

When a merge-forward produces a conflict on
`crates/server/tests/e2e.rs` AND the phase branch's only changes to
that file are appended comments, `#[ignore]` attributes, or
`DISABLE_*` markers (no test-logic changes), **take the governance-v0
side of the conflict**.

**Why:** governance-v0 reflects the latest correctly-integrated test
configuration. Taking the phase side risks carrying stale DISABLE
placements or conflict artifacts into the branch. v1-redaction-r1
example: pre-bm-pr merge-forward (RT-r5 test updates) produced a
conflict on e2e.rs; the advisor took the phase side, which kept the
wrong DISABLE_* test placement from quality-r3c. This caused the
phase-tip e2e run to fail (`1b88b91ce`); a manual patch at `9bf49615f`
restored the correct placement. Cost: ~1 hour (e2e run + diagnosis +
fix + re-run).

**Decision rule at merge-forward conflict on e2e.rs:**
1. Check: did the phase branch make any test-logic changes to e2e.rs
   (new test fns, changed assertions, changed setup/teardown)?
   - Yes → resolve manually, take the superset of both sides.
   - No (only comments / `#[ignore]` / `DISABLE_*` appended) →
     **take governance-v0 side**. The phase-branch additions are
     disposable (they were advisory; the real test state lives on
     governance-v0).
2. After resolving, commit with "merge-forward: took governance-v0
   side for e2e.rs (phase branch made no test-logic changes)" so
   the next session understands the resolution rationale.

**Source:** `fd01f6ef9` LESSON trailer, v1-redaction-r1 merge-forward
(2026-06-01).
