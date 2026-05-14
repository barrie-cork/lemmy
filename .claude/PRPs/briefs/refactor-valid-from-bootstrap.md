---
purpose: Bootstrap prompt for the chore/refactor-valid-from lane session
authored: 2026-05-14
audit_finding: 3.D.6 (rank 6, severity MED, effort S)
target_branch: chore/refactor-valid-from
target_worktree: C:/Users/barri/Developer/brehon-fork-refactor-valid-from
pr_position: PR-4 of 6 (parallel with PR-3, PR-5, PR-6)
---

# Bootstrap — chore/refactor-valid-from lane session

## Initial instruction (paste this as your first message to the lane session)

```
You are the lane-dedicated session for chore/refactor-valid-from (audit finding 3.D.6, PR-4 of 6).

Your worktree: C:/Users/barri/Developer/brehon-fork-refactor-valid-from
Your branch (after bm-cut): chore/refactor-valid-from
Your effort estimate: S (30-120 min)

Read these files in order before any state-changing action:

1. .claude/PRPs/briefs/refactor-lane-session-bootstrap.template.md
2. .claude/PRPs/briefs/refactor-valid-from-bm-cut.md
3. .claude/PRPs/briefs/refactor-valid-from-impl.md

Then execute Phase 1 -> Phase 5 of the template against the briefs.

Your refactor: pin `valid_from` literal on 2 seed migrations per audit §3.D.6. Mirror canonical pattern from `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:34-62` (it pins `'2026-04-23T00:02:00Z'::timestamptz`). The two non-pinned migrations need:

- `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` -> pin to '2026-04-18T00:00:00Z'::timestamptz (matches dir name)
- `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql` -> pin to '2026-04-22T00:03:00Z'::timestamptz (matches dir name)

Audit timestamps MUST be unique across the two files. Update column list to include valid_from. Update header comments mirroring the v1-JM-a canonical wording. Check down.sql for parity (may need same literal).

Pre-push cargo-check. Push. Two workflows fire (workspace + migration); raise TWO validate-pending DQ entries per Recipe 1.

When complete, surface 3-line summary and stop.

Concurrent-session discipline: PR-3, PR-5, PR-6 lanes may be running in parallel in their own worktrees.
```

## Lane-specific substitutions

| Template placeholder | Lane value |
|---|---|
| `<area>` | `valid-from` |
| `<lane-worktree-path>` | `C:/Users/barri/Developer/brehon-fork-refactor-valid-from` |
| `chore/refactor-<area>` | `chore/refactor-valid-from` |
| `refactor-<area>-precheck.log` | `refactor-valid-from-precheck.log` |
| Lane bm-cut brief | `.claude/PRPs/briefs/refactor-valid-from-bm-cut.md` |
| Lane impl brief | `.claude/PRPs/briefs/refactor-valid-from-impl.md` |
| Audit finding section | §3.D.6 |

## Worktree creation

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
git pull --ff-only origin governance-v0
git worktree add ../brehon-fork-refactor-valid-from governance-v0
```

## Cleanup (after PR merges)

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-refactor-valid-from
git branch -d chore/refactor-valid-from
```
