# [role:bm-task] sl-c-2-bm-merge-1 — merge PR #122 (phase-v1-SL-c-2)

## 1. Role + dispatch

`[role:bm-task] sl-c-2-bm-merge-1 — merge PR #122 phase-v1-SL-c-2 into governance-v0`

Run `/bm-merge` per `.claude/commands/bm/bm-merge.md`.

## 2. Scope

Merge PR #122 (`phase-v1-SL-c-2 → governance-v0`) on `barrie-cork/lemmy`.

**Pre-checks (confirm before merging):**
- PR #122 state = OPEN ✓
- Base = governance-v0 ✓
- 0 critical findings open in `.claude/PRPs/reviews/pr-122-findings.yaml` ✓
- recommendation = approved ✓
- CodeRabbit status = SUCCESS ✓

**Merge command:**
```bash
gh pr merge 122 --repo barrie-cork/lemmy --merge --delete-branch
```

Do NOT squash — task-per-commit history is load-bearing for retros.

**After merge:**
1. Append merge entry to `.claude/runlog/bm-runlog.md`:
   `## <timestamp> bm-merge: merged PR #122 phase-v1-SL-c-2 into governance-v0`
2. Commit runlog: `git add .claude/runlog/bm-runlog.md && git commit -m "chore(bm): merged PR #122 phase-v1-SL-c-2 into governance-v0" && git push origin governance-v0`
3. Verify branch deletion: `git ls-remote origin refs/heads/phase-v1-SL-c-2`

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — full bm-merge procedure
- `.claude/rules/branch-manager.md` — autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory

## 4. Constraints

- `--repo barrie-cork/lemmy` on every `gh` command
- `--merge` (no squash, no rebase) — preserve per-task commit history
- `--delete-branch` — clean up phase branch after merge
- Do NOT send Telegram ping
- Commit runlog BEFORE `gh pr merge` per L14 discipline
