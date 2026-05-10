---
phase: v1-RT-r1
role: bm-task
task: bm-cut
brief_n: 1
authored: 2026-05-10
---

# [role:bm-task] bm-cut v1-RT-r1 — see .claude/PRPs/briefs/rt-r1-bm-cut-1.md

## §1 Role + dispatch

`[role:bm-task] bm-cut v1-RT-r1 — cut phase-v1-RT-r1 off governance-v0`

## §2 Scope

Cut branch `phase-v1-RT-r1` off `governance-v0`. Follow `.claude/commands/bm/bm-cut.md` Phase 0 → Phase 5 exactly:

- Parse argument: `v1-RT-r1` → branch name `phase-v1-RT-r1` (phase type)
- Verify trunk state (fetch, status, sync check). Expected trunk tip at dispatch time: `8885cabc8` (`chore(advisor): rt-r1 plan amendment §10.8 + §1 + §19 — dual-const audit surface per DQ #188`) or a fast-forward descendant if the shutter session has pushed additional SL-d artifacts.
- Verify plan file: `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` exists on `governance-v0` (it does, per `5ab47690f`).
- Cut branch locally: `git checkout -b phase-v1-RT-r1 governance-v0`
- Append to `.claude/runlog/v1-RT-r1-runlog.md` (create if not yet existing)
- Output Phase 5 summary

Do NOT push the branch. Do NOT open a PR. Do NOT commit any code.

## §3 Required reading

- `.claude/commands/bm/bm-cut.md` — operational script (all phases)
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds
- `.claude/rules/phase-branch.md` — phase-branch flow

## §4 Constraints

- File-ownership: BM may only write `.claude/runlog/v1-RT-r1-runlog.md`. No edits to `crates/**`, `migrations/**`, `tests/**`, plans, PRDs.
- Branch must be `phase-v1-RT-r1` (pattern `^phase-v\d+-[A-Z]+-[a-z0-9-]+$` matches; sub-phase letter `r1` is the lane-internal numbering used by the reputation-tuning lane per PRD §15).
- Branch stays local-only at this stage (no push).
- Attribution: if a DQ entry is needed, use `from: "bm"`, never `from: "advisor"`.

## §5 Concurrency note

Sibling phase `phase-v1-SL-d` is concurrently active under shutter session ownership. RT-r1 and SL-d branches share zero handler files (per RT-r1 plan §6 + SL-d brief). bm-cut for RT-r1 does not touch SL-d's lane.
