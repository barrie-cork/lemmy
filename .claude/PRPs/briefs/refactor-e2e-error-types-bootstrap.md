---
purpose: Bootstrap prompt for the chore/refactor-e2e-error-types lane session
authored: 2026-05-14
audit_findings: 3.E.1 (CRIT) + 3.E.2 (CRIT) + 3.E.3 (MAJ) + 3.E.4 (MAJ)
audit_ranks: 1, 2, 7, 8
target_branch: chore/refactor-e2e-error-types
target_worktree: C:/Users/barri/Developer/brehon-fork-refactor-e2e
pr_position: PR-1 of 6 (largest; can run parallel with all others but recommend serial start)
---

# Bootstrap — chore/refactor-e2e-error-types lane session

## Initial instruction

```
You are the lane-dedicated session for chore/refactor-e2e-error-types (audit findings 3.E.1+2+3+4, PR-1 of 6).

Your worktree: C:/Users/barri/Developer/brehon-fork-refactor-e2e
Your branch (after bm-cut): chore/refactor-e2e-error-types
Your effort estimate: L (full day)
Severity: 2 CRITICAL + 2 MAJOR audit findings, bundled into one structural refactor per execution plan §"PR-to-finding map"

This is the largest refactor in the fix-before-next-phase tier. The bundled audit findings cover:
- Case C error-type mixing across 20+ test fns in crates/server/tests/e2e.rs (currently Box<dyn Error>, should be LemmyResult)
- 4 phase-specific fixtures modules with ~70% duplication (extract shared governance_test_helpers)
- phase1_migrations_round_trip is 447 lines mixing forward/revert/reapply — split into 3 test fns
- PHASE_1_MIGRATION_COUNT is "bookkeeping fiction" — replace with named-migration list

Read these files in order before any state-changing action:

1. .claude/PRPs/briefs/refactor-lane-session-bootstrap.template.md
2. .claude/PRPs/briefs/refactor-e2e-error-types-bm-cut.md
3. .claude/PRPs/briefs/refactor-e2e-error-types-impl.md
4. .claude/lessons/feedback_junior_worker_e2e_edit_hang.md (LOAD-BEARING for edit discipline on the 8945-line file)
5. .claude/lessons/feedback_lemmy_error_no_std_error.md (Case A target shape)

The impl brief is large (274 lines). Read it slowly. The 4-pass approach in §2.4 is mandatory ordering — DO NOT skip pass 1 (error-type unification) to do passes 2-4 first. Each pass commits locally; final ship is one squashed commit per §6.

Anchor-Edit discipline is MANDATORY: NEVER `Read` the full e2e.rs file. Use `offset` + `limit` for every Read. Use surgical `old_string` (3-5 line context) for every Edit. Per-pass batching: ~10 edits at a time, then local commit + cargo check.

Pre-push gates (3 of them):
- cargo-check --workspace --features full
- cargo-clippy --workspace --features full --tests --no-deps -- -D warnings
- cargo-test --workspace --features full --test e2e (FULL e2e run, ~26 min local — LOAD-BEARING)

The full local e2e run is the load-bearing validation signal — workspace check alone (cargo test --no-run) is insufficient for a test-refactor. Per feedback_phase_2_e2e_gate_enforcement.md.

When all 3 gates pass, push. Raise validate-pending DQ for the workspace workflow AND a `kind: "validate-pending-laptop-e2e"` DQ for the local e2e (per advisor-orchestrator.md §5.2 validate-pending-laptop handler — your local e2e log path becomes the DQ's `local_log_path` field).

When complete, surface 5-line summary (extends the template Phase 5 by 2 lines because of the dual DQ raises) and stop.

Concurrent-session discipline: PR-2, PR-3, PR-4, PR-5, PR-6 lanes may be running in parallel in their own worktrees. Zero file overlap verified per execution plan.

This is critical-tier work. If you encounter ANYTHING unexpected — file shape drift, a test that breaks and can't be cleanly fixed, the canonical mirror (v1-SL-b fixtures at ~line 11139) has changed — STOP and file a DQ blocker. Do not improvise. The audit is the contract.
```

## Lane-specific substitutions

| Template placeholder | Lane value |
|---|---|
| `<area>` | `e2e-error-types` (note: worktree dir is shorter, `refactor-e2e`) |
| `<lane-worktree-path>` | `C:/Users/barri/Developer/brehon-fork-refactor-e2e` |
| `chore/refactor-<area>` | `chore/refactor-e2e-error-types` |
| `refactor-<area>-precheck.log` | `refactor-e2e-precheck.log` |
| Lane bm-cut brief | `.claude/PRPs/briefs/refactor-e2e-error-types-bm-cut.md` |
| Lane impl brief | `.claude/PRPs/briefs/refactor-e2e-error-types-impl.md` |
| Audit finding section | §§3.E.1, 3.E.2, 3.E.3, 3.E.4 |

## Worktree creation

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
git pull --ff-only origin governance-v0
git worktree add ../brehon-fork-refactor-e2e governance-v0
```

NB: the worktree dir is `brehon-fork-refactor-e2e` (shorter than the full branch name) — keeps the path manageable for daily use.

## Cleanup (after PR merges)

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-refactor-e2e
git branch -d chore/refactor-e2e-error-types
```

## Risk callout

This is the highest-blast-radius refactor in the tier. Mitigations:
- Local intermediate commits during each pass (squash for final ship)
- ~10 edits between local cargo-check runs
- Anchor-Edit discipline mandatory
- Full local e2e run before push (~26 min) — workspace check alone is NOT sufficient validation

If the local e2e suite shows a regression that can't be explained by the refactor's mechanical scope, STOP. File a DQ blocker. The canonical advisor session will triage. Do not attempt to fix the regression in the same PR (scope creep).
