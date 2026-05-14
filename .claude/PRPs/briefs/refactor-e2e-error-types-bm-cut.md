---
phase: chore/refactor-e2e-error-types
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-14
audit_source: .claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md
audit_findings: 3.E.1 + 3.E.2 + 3.E.3 + 3.E.4 (ranks 1, 2, 7, 8 — bundled per audit §4 PR-1)
---

# [role:bm-task] bm-cut chore/refactor-e2e-error-types — see .claude/PRPs/briefs/refactor-e2e-error-types-bm-cut.md

## §1 Role + dispatch

`[role:bm-task] bm-cut chore/refactor-e2e-error-types — cut off governance-v0`

## §2 Scope

Cut local-only branch `chore/refactor-e2e-error-types` off `governance-v0`. Per `.claude/commands/bm/bm-cut.md` Phase 0 → Phase 5 exactly, with `--chore refactor-e2e-error-types` argument shape.

**Phase 0 parse:** input arg `--chore refactor-e2e-error-types` → branch name `chore/refactor-e2e-error-types` → type `chore`.

**Phase 1 trunk state:** `git fetch origin`; expect `governance-v0` clean + synced with `origin/governance-v0`. Tip should be `51519fdc9` (execution plan) or a fast-forward descendant.

**Phase 2 plan-file check:** **SKIP** — chore branches do not require a plan file per `bm-cut.md` Phase 2 (the "plan required" decision tree applies to `phase-*` and `plan/*` patterns only). The driving artifact for this refactor is the audit report at `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md`, not a plan file.

**Phase 3 cut:** `git checkout -b chore/refactor-e2e-error-types governance-v0`. Do NOT push (branch sits local until impl-task makes the first commit).

**Phase 4 runlog:** create `.claude/runlog/chore-refactor-e2e-error-types.md` with bm-cut seed entry.

**Phase 5 output:** standard Phase 5 summary.

Do NOT push. Do NOT open a PR. Do NOT commit any code.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — operational script (all phases)
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch flow (chore branches follow same trunk discipline)
- `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md` — execution context (PR-1 of 6)
- `.claude/PRPs/reports/v1-code-quality-audit-2026-05-14.md` §3.E (the findings this refactor addresses)

## §4 Constraints

- File-ownership: BM may only write `.claude/runlog/chore-refactor-e2e-error-types.md`. No edits to `crates/**`, `migrations/**`, `tests/**`, plans, PRDs.
- Branch must be `chore/refactor-e2e-error-types` exactly. Pattern `^chore/[a-z0-9-]+$` matches.
- Branch stays local-only at this stage (no push).
- Attribution: if a DQ entry is needed, use `from: "bm"`, never `from: "advisor"`.

## §5 Concurrency note

This is PR-1 of 6 refactor PRs running concurrently. Per `.claude/PRPs/handovers/refactor-execution-plan-2026-05-14.md`, each refactor uses its own worktree (multi-lane discipline per PMD #302). The bm-cut session for this refactor runs in the lane-dedicated worktree `C:/Users/barri/Developer/brehon-fork-refactor-e2e` once cut. Until that worktree exists, this bm-cut runs in the canonical `brehon-fork` checkout.

Sibling refactor branches concurrently being cut: `chore/refactor-toctou`, `chore/refactor-eq-derive`, `chore/refactor-valid-from`, `chore/refactor-diesel-errors`, `chore/refactor-seed-tests`. Zero file overlap across all 6 verified per execution plan §"Dependency analysis".
