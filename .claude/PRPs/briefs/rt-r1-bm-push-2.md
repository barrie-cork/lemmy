---
phase: v1-RT-r1
role: bm-task
task: bm-push-2
brief_n: 11
authored: 2026-05-12
---

# [role:bm-task] RT-r1 bm-push — push phase-v1-RT-r1 with fix-impl-4/5/6 commits — see .claude/PRPs/briefs/rt-r1-bm-push-2.md

## §1 Role + dispatch

`[role:bm-task] RT-r1 bm-push-2 — push phase-v1-RT-r1 with fix-impl-4/5/6 commits`

## §2 Scope

Run `bm-push` for phase-v1-RT-r1. The phase branch already has all fix-impl commits and is at `44b897556` on origin. PR #126 is open into `governance-v0`.

Push phase-v1-RT-r1 to origin (already up-to-date) and confirm PR #126 reflects the latest commits.

**Phase branch:** `phase-v1-RT-r1`
**PR:** `#126` into `governance-v0`
**Repo:** `barrie-cork/lemmy`

## §3 Required reading

- `.claude/commands/bm/bm-push.md` — bm-push verb
- `.claude/rules/branch-manager.md` — BM autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`

## §4 Constraints

- `--repo barrie-cork/lemmy` on all gh commands
- Do NOT merge the PR
- Do NOT force-push
- Confirm PR #126 is open and reflects the fix-impl-4/5/6 commits
