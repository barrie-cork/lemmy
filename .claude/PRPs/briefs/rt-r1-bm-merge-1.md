---
phase: v1-RT-r1
role: bm-task
task: bm-merge
brief_n: 12
authored: 2026-05-12
---

# [role:bm-task] RT-r1 bm-merge — merge PR #126 phase-v1-RT-r1 into governance-v0 — see .claude/PRPs/briefs/rt-r1-bm-merge-1.md

## §1 Role + dispatch

`[role:bm-task] RT-r1 bm-merge — merge PR #126 phase-v1-RT-r1 into governance-v0`

## §2 Scope

Run `bm-merge` for PR #126 (`phase-v1-RT-r1` → `governance-v0`).

**User has explicitly confirmed merge** (user gate 5 satisfied in advisor session).

- PR #126: "Phase v1-RT-r1 — reputation-tuning r1: config consts, ReputationEventSourceType, ENTRY_KIND shims, schema extensions, e2e round-trip probe"
- Base: `governance-v0`
- Head: `phase-v1-RT-r1`
- Repo: `barrie-cork/lemmy`
- 76 commits; all CR fix-in-pr findings resolved; CI green

Merge with `--merge` (no squash, no rebase — task-per-commit history is load-bearing for retros per `phase-branch.md`).

After merge: write runlog entry, delete `phase-v1-RT-r1` remote branch.

## §3 Required reading

- `.claude/commands/bm/bm-merge.md` — full merge gate + execute script
- `.claude/rules/branch-manager.md` — autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/runlog/bm-runlog.md` — append merge entry

## §4 Constraints

- `gh pr merge 126 --repo barrie-cork/lemmy --merge` — no squash, no rebase
- Do NOT merge into `main`
- Write `chore(bm): merge phase-v1-RT-r1 PR #126 into governance-v0` runlog entry
- Delete remote branch `phase-v1-RT-r1` after successful merge
- User gate 5 already cleared — do NOT re-ask for merge confirmation
