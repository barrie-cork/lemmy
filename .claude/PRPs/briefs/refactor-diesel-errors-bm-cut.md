---
phase: chore/refactor-diesel-errors
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.A.5 (rank 13 — PR-5)
---

# [role:bm-task] bm-cut chore/refactor-diesel-errors — see .claude/PRPs/briefs/refactor-diesel-errors-bm-cut.md

## §1 Role + dispatch

`[role:bm-task] bm-cut chore/refactor-diesel-errors — cut off governance-v0`

## §2 Scope

Cut `chore/refactor-diesel-errors` off `governance-v0`.

- **Phase 0:** arg `--chore refactor-diesel-errors` → `chore/refactor-diesel-errors` (type chore).
- **Phase 1:** trunk clean + synced.
- **Phase 2:** SKIP plan check (chore branch).
- **Phase 3:** `git checkout -b chore/refactor-diesel-errors governance-v0`. No push.
- **Phase 4:** create `.claude/runlog/chore-refactor-diesel-errors.md`.
- **Phase 5:** standard summary.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md`
- `.claude/rules/branch-manager.md`
- `.claude/rules/phase-branch.md`
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — execution context (PR-5 of 6, parallel-lane)
- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.A.5 (the finding — replace `.optional().ok().flatten()` chain in `admin_audit_stream.rs`)

## §4 Constraints

Same as PR-1 bm-cut brief; substitute runlog path. Branch local-only, no push.

## §5 Concurrency note

Lane-dedicated worktree: `C:/Users/barri/Developer/brehon-fork-refactor-diesel-errors`. Parallel with PRs 3/4/6 — verified zero file overlap (this PR touches only `admin_audit_stream.rs`).
