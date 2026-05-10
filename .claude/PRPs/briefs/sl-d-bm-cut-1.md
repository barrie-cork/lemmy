---
phase: v1-SL-d
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-10
---

# [role:bm-task] bm-cut v1-SL-d — see .claude/PRPs/briefs/sl-d-bm-cut-1.md

## §1 Role + dispatch

`[role:bm-task] bm-cut v1-SL-d — cut phase-v1-SL-d off governance-v0`

## §2 Scope

Cut branch `phase-v1-SL-d` off `governance-v0`. Follow `.claude/commands/bm/bm-cut.md` Phase 0 → Phase 5 exactly:

- Parse argument: `v1-SL-d` → branch name `phase-v1-SL-d` (phase type)
- Verify trunk state (fetch, status, sync check)
- Verify plan file: `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` exists on `governance-v0`
- Cut branch locally: `git checkout -b phase-v1-SL-d governance-v0`
- Append to `.claude/runlog/v1-SL-d-runlog.md` (create if not yet existing)
- Output Phase 5 summary

Do NOT push the branch. Do NOT open a PR. Do NOT commit any code.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — operational script (all phases)
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch flow

## §4 Constraints

- File-ownership: BM may only write `.claude/runlog/v1-SL-d-runlog.md`. No edits to `crates/**`, `migrations/**`, `tests/**`, plans, PRDs.
- Branch must be `phase-v1-SL-d` (pattern `^phase-v\d+-[A-Z]+-[a-z]$` matches).
- Branch stays local-only at this stage (no push).
- Attribution: if a DQ entry is needed, use `from: "bm"`, never `from: "advisor"`.
