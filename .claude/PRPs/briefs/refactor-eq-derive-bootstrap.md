---
purpose: Bootstrap prompt for the chore/refactor-eq-derive lane session
authored: 2026-05-14
audit_finding: 3.C.1 (rank 11, severity MAJ, effort XS)
target_branch: chore/refactor-eq-derive
target_worktree: C:/Users/barri/Developer/brehon-fork-refactor-eq-derive
pr_position: PR-3 of 6 (parallel with PR-4, PR-5, PR-6)
---

# Bootstrap — chore/refactor-eq-derive lane session

## Initial instruction (paste this as your first message to the lane session)

```
You are the lane-dedicated session for chore/refactor-eq-derive (audit finding 3.C.1, PR-3 of 6).

Your worktree: C:/Users/barri/Developer/brehon-fork-refactor-eq-derive
Your branch (after bm-cut): chore/refactor-eq-derive
Your effort estimate: XS (<30 min)

Read these two files in order before any state-changing action:

1. .claude/PRPs/briefs/refactor-lane-session-bootstrap.template.md
   (Shared template — pre-flight checks, hard refusals, execution sequence)

2. .claude/PRPs/briefs/refactor-eq-derive-bm-cut.md
   (Your bm-cut brief — Phase 1)

3. .claude/PRPs/briefs/refactor-eq-derive-impl.md
   (Your impl-task brief — Phase 2)

Then execute Phase 1 -> Phase 5 of the template against the briefs.

Your refactor: change line 10 of crates/db_schema/src/source/governance/governance_config.rs from
  #[derive(PartialEq, Serialize, Deserialize, Debug, Clone)]
to
  #[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]

One line. Pre-push cargo-check. Push. Raise validate-pending DQ. Go idle.

When complete, surface 3-line summary per template Phase 5 and stop. The canonical advisor session will pick up the validate-pending DQ on its next poll.

Concurrent-session discipline: PR-4, PR-5, PR-6 lanes may be running in parallel in their own worktrees. The canonical brehon-fork session is read-only. You write only to your worktree.
```

## Lane-specific substitutions (the lane session will pick these up from this file via the prompt above)

| Template placeholder | Lane value |
|---|---|
| `<area>` | `eq-derive` |
| `<lane-worktree-path>` | `C:/Users/barri/Developer/brehon-fork-refactor-eq-derive` |
| `chore/refactor-<area>` | `chore/refactor-eq-derive` |
| `refactor-<area>-precheck.log` | `refactor-eq-derive-precheck.log` |
| Lane bm-cut brief | `.claude/PRPs/briefs/refactor-eq-derive-bm-cut.md` |
| Lane impl brief | `.claude/PRPs/briefs/refactor-eq-derive-impl.md` |
| Audit finding section | §3.C.1 |

## Worktree creation (run once from canonical brehon-fork checkout before opening this lane's session)

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
git pull --ff-only origin governance-v0
git worktree add ../brehon-fork-refactor-eq-derive governance-v0
```

The worktree starts on `governance-v0`. The lane session's bm-cut Phase 3 will create the chore branch.

## Cleanup (after PR merges to governance-v0)

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-refactor-eq-derive
git branch -d chore/refactor-eq-derive
```
