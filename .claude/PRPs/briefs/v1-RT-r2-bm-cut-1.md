# Brief: bm-cut — v1-RT-r2

## 1. Role + dispatch

`[role:bm-task] v1-RT-r2 bm-cut — cut phase-v1-RT-r2 from governance-v0 — see .claude/PRPs/briefs/v1-RT-r2-bm-cut-1.md`

## 2. Scope

Cut the phase branch `phase-v1-RT-r2` from `governance-v0` HEAD (`a149912c0`).

Actions:
1. Verify trunk `governance-v0` is clean and up-to-date with `origin/governance-v0`.
2. Run `git checkout -b phase-v1-RT-r2` from `governance-v0`.
3. Push `phase-v1-RT-r2` to origin with `git push -u origin phase-v1-RT-r2`.
4. Confirm the branch exists on origin (`gh api repos/barrie-cork/lemmy/branches/phase-v1-RT-r2`).
5. Write a `bm:` runlog entry to `.claude/runlog/bm-runlog.md` noting the cut SHA and timestamp.

**Do NOT open a PR yet** — PR opens after all impl tasks complete.

## 3. Required reading

- `.claude/rules/branch-manager.md` — BM autonomy bounds + phase-branch discipline
- `.claude/rules/phase-branch.md` — branch name must match `phase-v1-<area>-<letter>`
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every `gh` command

## 4. Constraints

- Branch name MUST be `phase-v1-RT-r2` exactly.
- Base MUST be `governance-v0` (never `main`).
- Do NOT commit any code changes — bm-cut is topology only.
- Do NOT open a PR in this task.
