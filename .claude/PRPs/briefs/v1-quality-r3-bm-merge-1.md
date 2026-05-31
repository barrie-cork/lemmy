# [role:bm-task] bm-merge — v1-quality-r3 PR #169

## 1. Role + dispatch

`[role:bm-task] bm-merge v1-quality-r3 PR #169 — see .claude/PRPs/briefs/v1-quality-r3-bm-merge-1.md`

## 2. Scope

Merge PR #169 (`phase-v1-quality-r3` → `governance-v0`) on `barrie-cork/lemmy`.

- Use `--merge` (no squash — task-per-commit history is load-bearing for retros per `phase-branch.md`)
- `--repo barrie-cork/lemmy` mandatory on every `gh pr` command
- Do NOT delete the branch after merge (worktree cleanup is advisor's responsibility)

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — step-by-step merge verb script
- `.claude/rules/branch-manager.md` — autonomy bounds (merge requires user confirm — already granted by advisor relaying user "I confirm")
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command

## 4. Pre-merge state (advisor verified)

- PR #169: `refactor(e2e): v1-quality-r3 — EnvVarGuard C4 sweep (23 raw set_var sites)`
- Base: `governance-v0`, Head: `phase-v1-quality-r3` @ `17366ec16`
- CR triage: 1 finding (cr-1, low) — marked `done` (PR body updated with template sections)
- Zero open critical/major findings — CR recommendation: `approve`
- Full e2e: 126 passed, 0 failed, 5 ignored (E2E_EXIT_0, 2716s, advisor-laptop gate 2026-05-31)
- `/brehon-verify` report: all stories ✓ (`.claude/PRPs/reports/v1-quality-r3-verify.md`)
- User gate 5 (merge confirm): granted — user said "I confirm"

## 5. Constraints

- NEVER merge into `main`
- NEVER squash — `--merge` only
- NEVER force-push
- Write a `bm:` runlog entry to `.claude/runlog/bm-runlog.md` after merge completes
- If merge fails (state changed, open critical finding), raise `kind: "blocker"` DQ and stop
