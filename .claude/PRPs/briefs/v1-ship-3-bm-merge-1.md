# [role:bm-task] bm-merge — v1-ship-3 PR #149

## 1. Role + dispatch

`[role:bm-task] bm-merge v1-ship-3 PR #149 — see .claude/PRPs/briefs/v1-ship-3-bm-merge-1.md`

## 2. Scope

Merge PR #149 (`phase-v1-ship-3` → `governance-v0`) on `barrie-cork/lemmy`.

- Use `--merge` (no squash — task-per-commit history is load-bearing for retros per `phase-branch.md`)
- `--repo barrie-cork/lemmy` mandatory on every `gh pr` command
- Do NOT delete the branch after merge (worktree cleanup is advisor's responsibility)

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — step-by-step merge verb script
- `.claude/rules/branch-manager.md` — autonomy bounds (merge requires user confirm — already granted by advisor relaying user "proceed")
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command

## 4. Pre-merge state (advisor verified)

- PR #149: `mergeStateStatus: CLEAN`, `mergeable: MERGEABLE`, `state: OPEN`
- Base: `governance-v0`, Head: `phase-v1-ship-3`
- CR triage: 1 fix-in-pr done (cr-4 committed `b605a0156`), 4 rebuts — no open critical findings
- Full e2e: 109 passed, 0 failed (E2E_EXIT_0, 2026-05-24)
- Retro: signed off (`f2ed219e9`)
- User gate 5 (merge confirm): granted — user said "proceed"

## 4. Constraints

- NEVER merge into `main`
- NEVER squash — `--merge` only
- NEVER force-push
- Write a `bm:` runlog entry to `.claude/runlog/v1-ship-3-runlog.md` after merge completes
- If merge fails (state changed), raise `kind: "blocker"` DQ and stop
