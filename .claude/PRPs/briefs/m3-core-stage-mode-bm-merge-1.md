# Brief: m3-core-stage-mode BM-MERGE

## 1. Role + dispatch

`[role:bm-task] m3-core-stage-mode bm-merge — see .claude/PRPs/briefs/m3-core-stage-mode-bm-merge-1.md`

## 2. Scope

Merge PR #202 (`phase-m3-core-stage-mode → governance-v0`) on `barrie-cork/lemmy`.

**Pre-conditions (all verified by advisor before dispatch):**
- All CR findings addressed: cr-4 (MAJOR single-presenter) FIXED + VALIDATED @ phase `8b8c877a7`; cr-1/cr-2 (runlog doc-lint) FIXED; cr-3 (chair `_token` scaffold) REBUTTED (intentional Phase-6, no code change).
- Bridge Linux compile validated: check rc=0, clippy `-D warnings` rc=0, test stage 10/10 (DQ `8c61e18ffcea-001` resolved pass).
- `/brehon-verify` re-run: all 5 §16a stories ✓ (report `e9ab46a8e` on gov-v0).
- No critical findings open in `fix-in-pr` bucket with `addressed_in: null`.
- `mergeable: MERGEABLE`. `mergeStateStatus: UNSTABLE` is the CR non-blocking re-review marker only (Red-flag diff scan = SUCCESS; no required status checks gate this PR under Shape-G-residual).
- User gate 5 (merge confirm): **APPROVED 2026-06-19** ("Merge now (--merge)").

**Produce:**
- Merged PR #202 into `governance-v0` via `gh pr merge 202 --repo barrie-cork/lemmy --merge`
- Runlog entry in `.claude/runlog/bm-runlog.md` with the merged SHA
- Summary of merge outcome in task output (merged-into SHA + commit count)

**Do NOT:**
- Use `--squash` — task-per-commit history is load-bearing for retros. Use `--merge`.
- Merge into `main` — base is `governance-v0`.
- Touch `crates/`, `migrations/`, `tests/`, `services/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`.
- Open any new PRs.
- Delete the phase branch (advisor handles worktree/branch cleanup at phase-transition).

## 3. Required reading

- `.claude/rules/branch-manager.md` — autonomy bounds; merge requires no open critical fix-in-pr; merge is confirm-gated (already confirmed).
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
- `.claude/PRPs/reviews/pr-202-findings.yaml` — verify counters before merge (cr-4 addressed, cr-1/cr-2 addressed, cr-3 rebut).

## 4. Constraints

- PR number: **202**
- `--repo barrie-cork/lemmy` on every `gh pr` command.
- Merge strategy: **`--merge`** (NOT `--squash`).
- Base MUST be `governance-v0`.
- After merge: record in `.claude/runlog/bm-runlog.md` with the merged SHA.
- `answered_by` write rules: BM may only write `"bm-self-resolved"` — NEVER `"advisor"` or `"user"`.
