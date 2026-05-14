---
purpose: Bootstrap prompt for the chore/refactor-toctou lane session
authored: 2026-05-14
audit_finding: 3.B.1 (rank 3, severity CRITICAL, effort M)
target_branch: chore/refactor-toctou
target_worktree: C:/Users/barri/Developer/brehon-fork-refactor-toctou
pr_position: PR-2 of 6 (serial with PR-1; can run independently)
---

# Bootstrap — chore/refactor-toctou lane session

## Initial instruction

```
You are the lane-dedicated session for chore/refactor-toctou (audit finding 3.B.1 CRITICAL, PR-2 of 6).

Your worktree: C:/Users/barri/Developer/brehon-fork-refactor-toctou
Your branch (after bm-cut): chore/refactor-toctou
Your effort estimate: M (half-day)
Severity: CRITICAL (TOCTOU race; closes the only CRIT-tier audit finding outside the e2e refactor)

Read these files in order:

1. .claude/PRPs/briefs/refactor-lane-session-bootstrap.template.md
2. .claude/PRPs/briefs/refactor-toctou-bm-cut.md
3. .claude/PRPs/briefs/refactor-toctou-impl.md

Then execute Phase 1 -> Phase 5.

Your refactor: wrap SELECT-then-UPDATE/INSERT in `crates/api/api_crud/src/governance/create_report.rs` (approx lines 130-232) in `run_transaction` per audit §3.B.1. Mirror the canonical pattern from `crates/api/api_crud/src/governance/create_endorsement.rs:130-145`.

CRITICAL pre-flight: read these 5 files BEFORE editing (your impl brief §2.4 enumerates them):
- create_endorsement.rs:100-160 (canonical pattern)
- revoke_endorsement.rs:70-100 (second canonical example)
- create_report.rs:60-260 (the WHOLE handler body — you need outer-scope context)
- governance_log::append signature (in db_schema)
- actor_pseudonym_helper::get_or_create signature

Refactor shape: extract a named `process_report` helper holding the SELECT + write logic; outer fn does pseudonym fetch BEFORE run_transaction; tx body uses `(&mut *conn).into()` coercion for governance_log::append. Pre-tx validation (reason_code checks at lines 75-81) stays where it is — audit §3.B.6 flagged it as a POSITIVE exemplar.

Pre-push gates: cargo-check AND `cargo test --no-run -p lemmy_server --test e2e` (test target compile per feedback_test_target_compile_validation.md). Both must exit 0.

When complete, surface 3-line summary and stop.

Concurrent-session discipline: PR-1 lane (e2e refactor) may be running in parallel — its scope is crates/server/tests/e2e.rs; yours is crates/api/api_crud/. Zero file overlap.
```

## Lane-specific substitutions

| Template placeholder | Lane value |
|---|---|
| `<area>` | `toctou` |
| `<lane-worktree-path>` | `C:/Users/barri/Developer/brehon-fork-refactor-toctou` |
| `chore/refactor-<area>` | `chore/refactor-toctou` |
| `refactor-<area>-precheck.log` | `refactor-toctou-precheck.log` |
| Lane bm-cut brief | `.claude/PRPs/briefs/refactor-toctou-bm-cut.md` |
| Lane impl brief | `.claude/PRPs/briefs/refactor-toctou-impl.md` |
| Audit finding section | §3.B.1 |

## Worktree creation

```bash
cd C:/Users/barri/Developer/brehon-fork
git fetch origin governance-v0
git pull --ff-only origin governance-v0
git worktree add ../brehon-fork-refactor-toctou governance-v0
```

## Cleanup (after PR merges)

```bash
cd C:/Users/barri/Developer/brehon-fork
git worktree remove ../brehon-fork-refactor-toctou
git branch -d chore/refactor-toctou
```
