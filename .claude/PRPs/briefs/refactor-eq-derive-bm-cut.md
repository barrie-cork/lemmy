---
phase: chore/refactor-eq-derive
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.C.1 (rank 11 — PR-3)
---

# [role:bm-task] bm-cut chore/refactor-eq-derive — see .claude/PRPs/briefs/refactor-eq-derive-bm-cut.md

## §1 Role + dispatch

`[role:bm-task] bm-cut chore/refactor-eq-derive — cut off governance-v0`

## §2 Scope

Cut `chore/refactor-eq-derive` off `governance-v0`.

- **Phase 0:** arg `--chore refactor-eq-derive` → `chore/refactor-eq-derive` (type chore).
- **Phase 1:** trunk clean + synced.
- **Phase 2:** SKIP plan check (chore branch).
- **Phase 3:** `git checkout -b chore/refactor-eq-derive governance-v0`. No push.
- **Phase 4:** create `.claude/runlog/chore-refactor-eq-derive.md`.
- **Phase 5:** standard summary.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md`
- `.claude/rules/branch-manager.md`
- `.claude/rules/phase-branch.md`
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — execution context (PR-3 of 6, parallel-lane)
- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.C.1 (the finding — one-line `Eq` derive add)

## §4 Constraints

Same as PR-1 bm-cut brief; substitute runlog path. Branch local-only, no push.

## §5 Concurrency note

Lane-dedicated worktree: `C:/Users/barri/Developer/brehon-fork-refactor-eq-derive`. Parallel with PRs 4/5/6 — verified zero file overlap.
