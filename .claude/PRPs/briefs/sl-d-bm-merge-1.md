---
phase: v1-SL-d
role: bm-task
task: bm-merge
brief_n: 1
authored: 2026-05-11
---

# [role:bm-task] v1-SL-d bm-merge — merge PR #123 into governance-v0

## §1 Role + dispatch

`[role:bm-task] bm-merge v1-SL-d PR 123 — see .claude/PRPs/briefs/sl-d-bm-merge-1.md`

## §2 Scope

Execute-side only (Phases 5-9 from `.claude/commands/bm/bm-merge.md`). Gate-side checks already passed in the advisor session:

- PR #123: OPEN, CLEAN, MERGEABLE
- adr-compliance: SUCCESS
- CodeRabbit: SUCCESS
- DQ pending: 0
- Findings: 0 fix-in-pr, 0 critical open (all triaged: rebut cr-1/cr-2/cr-5, carry-forward cr-3/cr-4)
- `recommendation: approve` in `.claude/PRPs/reviews/pr-123-findings.yaml`

**PR:** `barrie-cork/lemmy` #123  
**Head:** `phase-v1-SL-d`  
**Base:** `governance-v0`

## §3 Required reading

- `.claude/rules/branch-manager.md` — file ownership, autonomy bounds
- `.claude/rules/phase-branch.md` — `--merge` not `--squash`
- `.claude/commands/bm/bm-merge.md` — Phases 5-9 procedure
- `.claude/PRPs/reviews/pr-123-findings.yaml` — final findings state

## §4 Constraints

- `--repo barrie-cork/lemmy` on every `gh` command.
- `--merge` not `--squash` (task-per-commit history is load-bearing for retros).
- `--delete-branch` to remove `origin/phase-v1-SL-d` after merge.
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`.

### Explicit git sequence (L14 fix — mandatory order):

1. Edit `.claude/runlog/bm-runlog.md` — append bm: merge entry per Phase 8 template.
2. `git add .claude/runlog/bm-runlog.md`
3. `git commit -m "chore(bm): merge PR #123 — runlog entry"`
4. `git push origin governance-v0`
5. THEN: `gh pr merge 123 --repo barrie-cork/lemmy --merge --delete-branch`
6. After merge: `git fetch origin && git checkout governance-v0 && git pull --ff-only origin governance-v0`
7. Verify branch deleted: `git ls-remote origin refs/heads/phase-v1-SL-d`. If still present → `gh api -X DELETE -H "Accept: application/vnd.github+json" /repos/barrie-cork/lemmy/git/refs/heads/phase-v1-SL-d`.
8. Update `.claude/PRPs/reviews/pr-123-findings.yaml` with `merged_at` + `merge_commit` + `final_recommendation: approve`.
