# [role:bm-task] bm-merge — v1-quality-r3b PR #170

## 1. Role + dispatch

`[role:bm-task] bm-merge v1-quality-r3b PR #170 — see .claude/PRPs/briefs/v1-quality-r3b-bm-merge-1.md`

## 2. Scope

Merge PR #170 (`phase-v1-quality-r3b` → `governance-v0`) on `barrie-cork/lemmy`.

- Use `--merge` (no squash — task-per-commit history is load-bearing for retros per `phase-branch.md`)
- `--repo barrie-cork/lemmy` mandatory on every `gh pr` command
- Do NOT delete the branch after merge (worktree cleanup is advisor's responsibility)

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — step-by-step merge verb script
- `.claude/rules/branch-manager.md` — autonomy bounds (merge requires user confirm — already granted)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command

## 4. Pre-merge state (advisor verified)

- PR #170: `mergeable: MERGEABLE`, `mergeStateStatus: UNSTABLE` (CI pending, non-blocking)
- Base: `governance-v0`, Head: `phase-v1-quality-r3b` @ `d1fb230f1`
- CR: zero actionable findings (CodeRabbit posted "No actionable comments were generated")
- Full e2e: 126 passed, 0 failed, 5 ignored (E2E_EXIT_0, validate-pending-laptop `be6ddc108436-001`)
- `/brehon-verify`: all stories ✓ (report at `.claude/PRPs/reports/v1-quality-r3b-verify.md`)
- Merge-forward: governance-v0 merged into phase branch at `d1fb230f1` (conflict resolved)
- User gate 5 (merge confirm): granted — user said "Yes"

## 5. Constraints

- NEVER merge into `main`
- NEVER squash — `--merge` only
- NEVER force-push
- Write a `bm:` runlog entry to `.claude/runlog/v1-quality-r3b-runlog.md` after merge completes
- If merge fails (state changed), raise `kind: "blocker"` DQ and stop
