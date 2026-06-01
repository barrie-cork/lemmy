# [role:bm-task] bm-merge — v1-quality-r3c PR #172

## 1. Role + dispatch

`[role:bm-task] bm-merge v1-quality-r3c PR #172 — see .claude/PRPs/briefs/v1-quality-r3c-bm-merge-1.md`

## 2. Scope

Merge PR #172 (`phase-v1-quality-r3c` → `governance-v0`) on `barrie-cork/lemmy`.

- Use `--merge` (no squash — task-per-commit history is load-bearing for retros per `phase-branch.md`)
- `--repo barrie-cork/lemmy` mandatory on every `gh pr` command
- Do NOT delete the branch after merge (worktree cleanup is advisor's responsibility)

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — step-by-step merge verb script
- `.claude/rules/branch-manager.md` — autonomy bounds (merge requires user confirm — already granted)
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command
- `.claude/lessons/feedback_bm_false_success_advisor_post_condition_catch.md` — verify PR state after merge; NEVER retry on non-zero; NEVER force-push

## 4. Pre-merge state (advisor verified)

- PR #172: `mergeable: MERGEABLE`, `mergeStateStatus: CLEAN`
- Base: `governance-v0`, Head: `phase-v1-quality-r3c` @ `bef199feb`
- CR findings: 0 critical, 0 open major/medium/low in fix-in-pr (cr-2 addressed at `bef199feb`, all others wont-fix)
- Full e2e: 128 passed, 0 failed, 5 skipped (validate-pending-laptop DQ `a3d0e9941441-044` → result: pass)
- `/brehon-verify`: all 3 stories ✓ (report at `.claude/PRPs/reports/v1-quality-r3c-verify.md`)
- Merge-forward: governance-v0 merged into phase branch at `0216babc9` (conflict resolved)
- User gate 5 (merge confirm): granted — user said "Yes, go for it"

## 5. Constraints

- NEVER merge into `main`
- NEVER squash — `--merge` only
- NEVER force-push; NEVER retry `gh pr merge` on failure; NEVER hand-resolve conflicts
- If `gh pr merge` exits non-zero: capture verbatim stdout+stderr, write to task output, STOP
- Write a `bm:` runlog entry to `.claude/runlog/bm-runlog.md` AFTER merge completes (post-merge, not pre-merge)
- If merge fails (state changed), raise `kind: "blocker"` DQ and stop
