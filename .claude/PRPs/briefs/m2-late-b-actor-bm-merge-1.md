# Brief: m2-late-b-actor BM-MERGE

## 1. Role + dispatch

`[role:bm-task] m2-late-b-actor bm-merge — see .claude/PRPs/briefs/m2-late-b-actor-bm-merge-1.md`

## 2. Scope

Merge PR #197 (`phase-m2-late-b-actor → governance-v0`) on `barrie-cork/lemmy`.

**Pre-conditions (all verified by advisor before dispatch):**
- All fix-in-pr findings addressed: cr-6, cr-8, cr-9 at `dcb5d3723`, confirmed by CR ("✅ Addressed in commits dcb5d37 to 210ff13")
- DQ pending = 0 on phase branch
- Bridge Linux compile validated: exit 0 (`32a44ffbe8e7-001` resolved pass)
- No critical findings open in `fix-in-pr` bucket
- `mergeStateStatus: UNSTABLE` is ADR advisory check only (non-blocking)
- User gate 5 (merge confirm): APPROVED

**Produce:**
- Merged PR #197 into `governance-v0`
- Runlog entry in `.claude/runlog/bm-runlog.md`
- Summary of merge outcome in task output

**Do NOT:**
- Merge if any critical finding has `bucket: fix-in-pr` and `addressed_in: null`
- Touch `crates/`, `migrations/`, `tests/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`
- Open any new PRs
- Push to `main`

## 3. Required reading

- `.claude/rules/branch-manager.md` — autonomy bounds; merge requires no open critical fix-in-pr
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/pr-197-findings.yaml` — verify counters before merge

## 4. Constraints

- PR number: **197**
- `--repo barrie-cork/lemmy` on every `gh pr` command
- Merge strategy: squash is NOT allowed (task-per-commit history is load-bearing for retros); use `--merge` or `--rebase`
- After merge: record in `.claude/runlog/bm-runlog.md` with merged SHA
- `answered_by` write rules: BM may only write `"bm-self-resolved"` — NEVER `"advisor"` or `"user"`
