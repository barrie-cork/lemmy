# [role:bm-task] bm-merge — m1-b PR #177

## 1. Role + dispatch

`[role:bm-task] bm-merge m1-b PR #177 — see .claude/PRPs/briefs/m1-b-bm-merge-1.md`

## 2. Scope

Merge PR #177 (`phase-m1-b` → `governance-v0`) on `barrie-cork/lemmy`.

- Use `--merge` (**NO squash** — task-per-commit history is load-bearing for retros per `phase-branch.md`)
- Use `--admin` to bypass the **advisory** failing check (see §4 — user-authorized).
- Use `--delete-branch` (merge + remote branch deletion in one step).
- `--repo barrie-cork/lemmy` mandatory on every `gh pr` command.

Merge command (exact):
```
gh pr merge 177 --repo barrie-cork/lemmy --merge --admin --delete-branch
```

## 3. Required reading

- `.claude/commands/bm/bm-merge.md` — step-by-step merge verb script (L14 runlog-AFTER-merge + L16 branch-deletion verify).
- `.claude/rules/branch-manager.md` — autonomy bounds (merge requires user confirm — **already granted**, gate 5).
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on every gh command.
- `.claude/lessons/feedback_bm_merge_unstable_admin_bypass.md` — the `--admin` bypass remedy for the known-unreliable adr-compliance advisory check.

## 4. Pre-merge state (advisor verified)

- **PR #177**: `phase-m1-b` → `governance-v0`. Head `phase-m1-b` @ `5affadb05`.
- **mergeable**: MERGEABLE.
- **CodeRabbit**: SUCCESS (re-reviewed clean after the F2 fix commit — no new actionable findings). The 3 original findings: F2 fixed (read_current tie-breaker), F1 + F3 rebutted (responses posted on PR).
- **⚠️ Red-flag diff scan (adr-compliance): FAILURE — ADVISORY ONLY.** It flags the two new governance routes (`admin_set_messaging_config` / `admin_get_messaging_config`) under ADR-010 §2 (advisory). These are **plan-sanctioned M1-b Tasks 4-5 additions** (post-v0 messaging extension, ADR-016 backplane) — NOT a v0-scope violation. **User acknowledged this advisory flag and authorized `--admin` bypass at gate 5 (merge confirm).** This is the documented remedy per `feedback_bm_merge_unstable_admin_bypass.md`.
- **Validation**: all green — `cargo check --workspace --features full` (0/0), `cargo test --no-run -p lemmy_server --test e2e` (0/0), 2 new e2e tests pass, Linux-compile gate (`cargo-linux.sh`, 0/0). All validate-pending DQ resolved.
- **/brehon-verify**: all 3 in-scope Tree-B stories (2, 3, 5) ✓, no phantoms (stories 1, 4, 6 are Tree A, out of scope). Report at `.claude/PRPs/reports/m1-b-verify.md`.
- **User gate 5 (merge confirm)**: GRANTED — user said "Confirm merge (advisory flag acknowledged)".

## 5. Constraints

- NEVER merge into `main`.
- NEVER squash — `--merge` only.
- NEVER force-push.
- `--admin` is authorized for THIS merge ONLY (to bypass the advisory adr-compliance red-flag). Do not use `--admin` to bypass any other failing check — if a NON-advisory check is also red, STOP and raise a `kind: "blocker"` DQ.
- **L14 (runlog AFTER merge):** AFTER `gh pr merge` succeeds → `git checkout governance-v0` + `git pull` → Edit `.claude/runlog/m1-b-runlog.md` with a `bm:` COMPLETE entry citing the real merge SHA → `git add` → `git commit -m "chore(bm): merge PR #177 complete"` → `git push origin governance-v0`. The runlog commit lands AFTER the merge, NOT before (per `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md`).
- If the merge fails (state changed since advisor verified), raise a `kind: "blocker"` DQ and stop — do NOT improvise an alternate merge strategy.
