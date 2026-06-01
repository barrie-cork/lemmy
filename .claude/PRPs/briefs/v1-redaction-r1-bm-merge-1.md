# [role:bm-task] bm-merge — v1-redaction-r1 PR #173

## 1. Role + dispatch

`[role:bm-task] bm-merge v1-redaction-r1 PR #173 — see .claude/PRPs/briefs/v1-redaction-r1-bm-merge-1.md`

## 2. Scope

Merge PR #173 (`phase-v1-redaction-r1` → `governance-v0`) on `barrie-cork/lemmy`.

- Use `--merge` (no squash — task-per-commit history is load-bearing for retros per `phase-branch.md`)
- `--repo barrie-cork/lemmy` mandatory on every `gh pr` command
- Do NOT delete the branch after merge (worktree cleanup is advisor's responsibility)

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — step-by-step merge verb script
- `.claude/rules/branch-manager.md` — autonomy bounds (merge requires user confirm — already granted)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command

## 4. Pre-merge state (advisor verified)

- PR #173: phase-v1-redaction-r1 → governance-v0
- Head: `phase-v1-redaction-r1` @ `f497afcc8`
- CR findings: 7 total — critical: 1 done (cr-6, CR self-rebutted 2nd pass), low: 1 wont-fix, nit: 5 wont-fix. Zero open. `final_recommendation: approve`
- Full e2e: 128 passed, 0 failed, 5 ignored (E2E_EXIT_0, validate-pending-laptop DQ `9f1d7e7ca817-001`)
- Workspace clippy: exit 0 (DQ `74ae9c10e50b-001` pass)
- `/brehon-verify`: 4/4 stories ✓ (report at `.claude/PRPs/reports/v1-redaction-r1-verify.md`)
- User gate 5 (merge confirm): granted — user said "Go for it"

## 5. Constraints

- NEVER merge into `main`
- NEVER squash — `--merge` only
- NEVER force-push
- Write a `bm:` runlog entry to `.claude/runlog/v1-redaction-r1-runlog.md` after merge completes
- If merge fails (state changed), raise `kind: "blocker"` DQ and stop
