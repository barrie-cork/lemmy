# [role:bm-task] bm-merge — m2-late-2 PR #196

## 1. Role + dispatch

`[role:bm-task] bm-merge m2-late-2 PR #196 — see .claude/PRPs/briefs/m2-late-2-bm-merge-1.md`

## 2. Scope

Merge PR #196 (`phase-m2-late-2` → `governance-v0`) on `barrie-cork/lemmy`.

- Use `--merge` (**NO squash** — task-per-commit history is load-bearing for retros per `phase-branch.md`)
- Use `--admin` to bypass the **advisory** failing check (see §4 — user-authorized).
- Use `--delete-branch` (merge + remote branch deletion in one step).
- `--repo barrie-cork/lemmy` mandatory on every `gh pr` command.

Merge command (exact):
```
gh pr merge 196 --repo barrie-cork/lemmy --merge --admin --delete-branch
```

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — step-by-step merge verb script (L14 runlog-AFTER-merge + L16 branch-deletion verify).
- `.claude/rules/branch-manager.md` — autonomy bounds (merge requires user confirm — **already granted**, gate 5).
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command.
- `.claude/lessons/feedback_bm_merge_unstable_admin_bypass.md` — the `--admin` bypass remedy for the known-unreliable adr-compliance advisory check.

## 4. Pre-merge state (advisor verified 2026-06-12T22:05Z)

- **PR #196**: `phase-m2-late-2` → `governance-v0`. Head `phase-m2-late-2` @ `0458d5b08`.
- **mergeable**: MERGEABLE (verified).
- **mergeStateStatus**: UNSTABLE — CodeRabbit advisory check only (same pattern as m2-rooms-a PR #191 and m1-b PR #177; documented remedy: `--admin`). Red-flag diff scan = SUCCESS.
- **open_critical**: 0 — pr-196-findings.yaml: fix-in-pr=0, rebut=3, done=1. No blocking findings.
- **CR fixes**: cr-2 done in commit `0458d5b08` (T1 DQ commands[] audit-trail correction). cr-1/3/4 rebutted.
- **Merge-forward**: complete — governance-v0 merged into phase-m2-late-2 by bm-pr task #668 at `4f486e25d`.
- **User gate 5 (merge confirm)**: GRANTED — user said "Yes" at 2026-06-12T22:05Z.

## 5. Constraints

- NEVER merge into `main`.
- NEVER squash — `--merge` only.
- NEVER force-push.
- `--admin` is authorized for THIS merge ONLY (to bypass the advisory CodeRabbit UNSTABLE flag). Do not use `--admin` to bypass any other failing check — if a NON-advisory check is also red, STOP and raise a `kind: "blocker"` DQ.
- **L14 (runlog AFTER merge):** AFTER `gh pr merge` succeeds → `git checkout governance-v0` + `git pull` → Edit `.claude/runlog/m2-late-2-runlog.md` with a `bm:` COMPLETE entry citing the real merge SHA → `git add` → `git commit -m "chore(bm): merge PR #196 complete — m2-late-2"` → `git push origin governance-v0`. The runlog commit lands AFTER the merge, NOT before.
- If the merge fails (state changed since advisor verified), raise a `kind: "blocker"` DQ and stop — do NOT improvise an alternate merge strategy.
- **File-ownership:** do NOT touch `crates/**`, `migrations/**`, `tests/**`, `services/bridge/src/**`. BM owns only the merge action + runlog entry.
- Create `.claude/runlog/m2-late-2-runlog.md` if it does not already exist (phase m2-late-2 runlog).
