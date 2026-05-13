---
phase: v1-SL-e
role: bm-task
task: bm-pr
brief_n: 5
authored: 2026-05-13
---

# [role:bm-task] SL-e bm-pr — open PR for phase-v1-SL-e — see .claude/PRPs/briefs/sl-e-bm-pr-1.md

## §1 Role + dispatch

`[role:bm-task] SL-e bm-pr — open PR for phase-v1-SL-e into governance-v0`

## §2 Scope

Run `bm-pr` for phase-v1-SL-e. SL-e is the **lane-closer** for v1 sponsor-liability (SL-a → SL-b → SL-c → SL-d → SL-e). All 4 tasks shipped + validated.

**Phase branch:** `phase-v1-SL-e`
**Tip:** `e475436ed` (Task 4 retro finalize-merged)
**Base:** `governance-v0`
**Repo:** `barrie-cork/lemmy`

**Retro:** `.claude/PRPs/reports/v1-SL-e-retro.md` (already on phase branch, committed `259f5db5a`)

## §3 Required reading

- `.claude/commands/bm/bm-pr.md` — bm-pr verb
- `.claude/rules/branch-manager.md` — BM autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
- `.claude/rules/phase-branch.md` — phase-branch + PR flow
- `.claude/PRPs/reports/v1-SL-e-retro.md` — retro for PR body summary

## §4 Constraints

- **--repo barrie-cork/lemmy** on all gh commands
- Base: `governance-v0` (NOT `main`)
- Head: `phase-v1-SL-e`
- **Not draft** (CR skips drafts)
- **PR title:** `Phase v1-SL-e — lane-closer e2e suite (revocation + window-expiry + backfill)`
- **PR body** assembles from:
  - 1-line summary
  - Task table (T0-T4) with commits + DQ entries
  - Retro reference: `.claude/PRPs/reports/v1-SL-e-retro.md`
  - Plan reference: `.claude/PRPs/plans/v1-sponsor-liability-e.plan.md`
  - Test plan checklist
- Do NOT merge the PR (this verb opens only)
- Write `.claude/PRPs/reviews/pr-<N>-findings.yaml` shell for CR triage later
