---
purpose: Bootstrap prompt for the chore/refactor-seed-tests lane session
authored: 2026-05-14
audit_finding: 3.E.20 (rank 17, severity MED, effort S)
target_branch: chore/refactor-seed-tests
target_worktree: C:/Users/barri/Developer/brehon-fork-refactor-seed-tests
pr_position: PR-6 of 6 (parallel with PR-3, PR-4, PR-5)
---

# Bootstrap — chore/refactor-seed-tests lane session

## Initial instruction

```
You are the lane-dedicated session for chore/refactor-seed-tests (audit finding 3.E.20, PR-6 of 6).

Your worktree: C:/Users/barri/Developer/brehon-fork-refactor-seed-tests
Your branch (after bm-cut): chore/refactor-seed-tests
Your effort estimate: S (30-120 min)

Read these files in order:

1. .claude/PRPs/briefs/refactor-lane-session-bootstrap.template.md
2. .claude/PRPs/briefs/refactor-seed-tests-bm-cut.md
3. .claude/PRPs/briefs/refactor-seed-tests-impl.md

Then execute Phase 1 -> Phase 5.

Your refactor: add `#[cfg(test)] mod tests { ... }` block at end of `crates/tools/seed_founders/src/main.rs` with exactly 5 unit tests for `parse_founder_spec`:

- parse_founder_spec_valid (happy path)
- parse_founder_spec_negative_delta (jury_reliability=0 + reporting_accuracy=-3)
- parse_founder_spec_exceeds_max (value > max_seed_delta)
- parse_founder_spec_wrong_segment_count (3 + 5 segments both reject)
- parse_founder_spec_non_numeric (parse error on person-id and jury_reliability)

Test signatures use #[test] (not #[tokio::test]) — parse_founder_spec is sync. Test assertions follow feedback_clippy_test_style.md (LemmyResult, .expect for happy path, .unwrap_err for error path with .to_string() substring check).

Pre-push gates: cargo-check AND cargo-test --tests -p seed_founders. Both must exit 0 BEFORE push. Tests should run in <1 second (pure parse function, no DB).

When complete, surface 3-line summary and stop.

Concurrent-session discipline: PR-3, PR-4, PR-5 lanes may be running in parallel.
```

## Lane-specific substitutions

| Template placeholder | Lane value |
|---|---|
| `<area>` | `seed-tests` |
| `<lane-worktree-path>` | `C:/Users/barri/Developer/brehon-fork-refactor-seed-tests` |
| `chore/refactor-<area>` | `chore/refactor-seed-tests` |
| `refactor-<area>-precheck.log` | `refactor-seed-tests-precheck.log` |
| Lane bm-cut brief | `.claude/PRPs/briefs/refactor-seed-tests-bm-cut.md` |
| Lane impl brief | `.claude/PRPs/briefs/refactor-seed-tests-impl.md` |
| Audit finding section | §3.E.20 |

## Worktree creation

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
git pull --ff-only origin governance-v0
git worktree add ../brehon-fork-refactor-seed-tests governance-v0
```

## Cleanup (after PR merges)

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-refactor-seed-tests
git branch -d chore/refactor-seed-tests
```
