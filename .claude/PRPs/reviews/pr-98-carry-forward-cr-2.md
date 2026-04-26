## carry-forward from PR #98: submit_jury_vote lock-ordering deadlock fix (JM-d-candidate)

**Source:** CodeRabbit cr-2 on PR #98 (severity: critical)
**CR comment:** https://github.com/barrie-cork/lemmy/pull/98#discussion_r3142613525
**Findings YAML row:** `.claude/PRPs/reviews/pr-98-findings.yaml` → `cr-2`
**Resolved DQ ref:** decision-queue.json id=50 (originally filed as id=49; relocated to `resolved[]` as id=50 in `e9fa1e01a` per cr-1 fix)

### Problem

`crates/api/api/src/governance/submit_jury_vote.rs::process_vote` has a
lock-ordering hazard between step 2 and step 6:

- **Step 2** inserts into `jury_vote`. The FK from `jury_vote.case_id` →
  `moderation_case.id` causes Postgres to acquire an FK SHARE lock on
  the parent `moderation_case` row.
- **Step 6** acquires `SELECT ... FOR UPDATE` on the same `moderation_case`
  row.

Two concurrent transactions both holding FK SHARE on the same parent row
then both attempting upgrade to FOR UPDATE produce a deterministic
Postgres deadlock (`deadlock detected` error). This is reproducible 100%
of the time in the existing `#[ignore]`-marked test
`submit_jury_vote_concurrent_votes_decide_exactly_once` (PR #98,
`crates/server/tests/e2e.rs`).

### Why deferred from JM-c

JM-c was scoped to the `submit_jury_vote` 9-step rewrite for snapshot-aware
threshold + deadlock + appeal_window writes. The handler structure
(vote INSERT at step 2, FOR UPDATE at step 6) is pre-existing v0/JM-b-era
code. Refactoring the lock acquisition order is out of JM-c's surgical
scope per resolved DQ #50 (impl-self-resolved with option (a); originally
filed as #49, relocated as part of cr-1 fix in `e9fa1e01a`).

The `#[ignore]` test serves as the deferred regression test.

### Fix path (JM-d)

Refactor `submit_jury_vote::process_vote` to acquire `SELECT ... FOR UPDATE`
on `moderation_case` *before* inserting into `jury_vote` (DQ #50
option (c)). Then remove `#[ignore]` from the existing test to lock in
the regression coverage.

JM-d already touches the post-decision block for appeal-panel work, so
the lock-ordering refactor can land in a coherent context.

### Acceptance criteria

- [ ] Reorder steps so `SELECT ... FOR UPDATE` on `moderation_case` runs
      before the `jury_vote` INSERT.
- [ ] Remove `#[ignore]` from `submit_jury_vote_concurrent_votes_decide_exactly_once`
      and confirm it passes.
- [ ] No regression in the other JM-c e2e tests
      (`submit_jury_vote_writes_appeal_window_default` etc.).
- [ ] No new findings on `submit_jury_vote.rs` from CR re-review.

### Cross-references

- Plan: `.claude/PRPs/plans/v1-jury-mechanics-c.plan.md` §10.8 GOTCHA
- DQ #50 (resolved, impl-self-resolved with option (a); originally filed as #49)
- Plan: `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` (when written —
  this issue is the JM-d-candidate marker)
- Memory: `feedback_idempotency_guard_above_quorum_gate.md` — adjacent
  pattern for post-quorum re-runs

**Labels:** carry-forward, source-coderabbit, JM-d-candidate, deadlock
