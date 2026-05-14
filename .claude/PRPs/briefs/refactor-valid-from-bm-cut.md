---
phase: chore/refactor-valid-from
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.D.6 (rank 6 — PR-4)
---

# [role:bm-task] bm-cut chore/refactor-valid-from — see .claude/PRPs/briefs/refactor-valid-from-bm-cut.md

## §1 Role + dispatch

`[role:bm-task] bm-cut chore/refactor-valid-from — cut off governance-v0`

## §2 Scope

Cut `chore/refactor-valid-from` off `governance-v0`.

- **Phase 0:** arg `--chore refactor-valid-from` → `chore/refactor-valid-from` (type chore).
- **Phase 1:** trunk clean + synced.
- **Phase 2:** SKIP plan check (chore branch).
- **Phase 3:** `git checkout -b chore/refactor-valid-from governance-v0`. No push.
- **Phase 4:** create `.claude/runlog/chore-refactor-valid-from.md`.
- **Phase 5:** standard summary.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md`
- `.claude/rules/branch-manager.md`
- `.claude/rules/phase-branch.md`
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — execution context (PR-4 of 6, parallel-lane)
- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.D.6 (the finding — pin `valid_from` literals on 2 seed migrations)

## §4 Constraints

Same as PR-1 bm-cut brief; substitute runlog path. Branch local-only, no push.

## §5 Concurrency note

Lane-dedicated worktree: `C:/Users/barri/Developer/brehon-fork-refactor-valid-from`. Parallel with PRs 3/5/6 — verified zero file overlap (this PR is the only one touching `migrations/`).
