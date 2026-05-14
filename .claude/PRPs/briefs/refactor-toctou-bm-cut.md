---
phase: chore/refactor-toctou
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.B.1 (rank 3 — PR-2)
---

# [role:bm-task] bm-cut chore/refactor-toctou — see .claude/PRPs/briefs/refactor-toctou-bm-cut.md

## §1 Role + dispatch

`[role:bm-task] bm-cut chore/refactor-toctou — cut off governance-v0`

## §2 Scope

Cut `chore/refactor-toctou` off `governance-v0`. Same shape as the e2e-error-types bm-cut. Per `.claude/commands/bm/bm-cut.md` Phase 0 → Phase 5.

- **Phase 0:** arg `--chore refactor-toctou` → `chore/refactor-toctou` (type chore).
- **Phase 1:** trunk clean + synced.
- **Phase 2:** SKIP plan check (chore branch).
- **Phase 3:** `git checkout -b chore/refactor-toctou governance-v0`. No push.
- **Phase 4:** create `.claude/runlog/chore-refactor-toctou.md`.
- **Phase 5:** standard summary.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md`
- `.claude/rules/branch-manager.md`
- `.claude/rules/phase-branch.md`
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — execution context (PR-2 of 6)
- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.B.1 (the finding)

## §4 Constraints

Same as PR-1 bm-cut brief; substitute runlog path. Branch local-only, no push.

## §5 Concurrency note

Lane-dedicated worktree: `C:/Users/barri/Developer/brehon-fork-refactor-toctou` (cut after bm-cut). Sibling refactor branches: see execution plan §"Topology — per-worktree-per-lane".
