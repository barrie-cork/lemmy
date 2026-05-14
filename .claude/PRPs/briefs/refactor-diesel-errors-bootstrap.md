---
purpose: Bootstrap prompt for the chore/refactor-diesel-errors lane session
authored: 2026-05-14
audit_finding: 3.A.5 (rank 13, severity MAJ, effort S)
target_branch: chore/refactor-diesel-errors
target_worktree: C:/Users/barri/Developer/brehon-fork-refactor-diesel-errors
pr_position: PR-5 of 6 (parallel with PR-3, PR-4, PR-6)
---

# Bootstrap — chore/refactor-diesel-errors lane session

## Initial instruction

```
You are the lane-dedicated session for chore/refactor-diesel-errors (audit finding 3.A.5, PR-5 of 6).

Your worktree: C:/Users/barri/Developer/brehon-fork-refactor-diesel-errors
Your branch (after bm-cut): chore/refactor-diesel-errors
Your effort estimate: S (30-120 min)

Read these files in order:

1. .claude/PRPs/briefs/refactor-lane-session-bootstrap.template.md
2. .claude/PRPs/briefs/refactor-diesel-errors-bm-cut.md
3. .claude/PRPs/briefs/refactor-diesel-errors-impl.md

Then execute Phase 1 -> Phase 5.

Your refactor: replace `.optional().ok().flatten()` chain at 2 sites in `crates/api/api/src/governance/admin_audit_stream.rs` (around lines 227 and 234) with proper error propagation per audit §3.A.5.

CRITICAL pre-flight step: read crates/api/api/src/governance/admin_audit_stream.rs lines 200-250 to determine the enclosing function shape BEFORE editing. If the enclosing scope returns LemmyResult, use `.optional()?`. If it's a stream/spawn closure without Result outer, use the `.map_err(|e| { tracing::warn!(...); e }).ok().flatten()` fallback variant that at least logs the error. The brief documents both variants in §2.3.

Apply the SAME variant at both sites. Pre-push cargo-check. Push. Raise validate-pending DQ.

When complete, surface 3-line summary and stop.

Concurrent-session discipline: PR-3, PR-4, PR-6 lanes may be running in parallel.
```

## Lane-specific substitutions

| Template placeholder | Lane value |
|---|---|
| `<area>` | `diesel-errors` |
| `<lane-worktree-path>` | `C:/Users/barri/Developer/brehon-fork-refactor-diesel-errors` |
| `chore/refactor-<area>` | `chore/refactor-diesel-errors` |
| `refactor-<area>-precheck.log` | `refactor-diesel-errors-precheck.log` |
| Lane bm-cut brief | `.claude/PRPs/briefs/refactor-diesel-errors-bm-cut.md` |
| Lane impl brief | `.claude/PRPs/briefs/refactor-diesel-errors-impl.md` |
| Audit finding section | §3.A.5 |

## Worktree creation

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
git pull --ff-only origin governance-v0
git worktree add ../brehon-fork-refactor-diesel-errors governance-v0
```

## Cleanup (after PR merges)

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-refactor-diesel-errors
git branch -d chore/refactor-diesel-errors
```
