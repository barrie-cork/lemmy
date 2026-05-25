# Brief: v1-deps-r1 BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] v1-deps-r1-bm-poll-cr — see .claude/PRPs/briefs/v1-deps-r1-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit findings on PR #153 (`phase-v1-deps-r1 → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-153-findings.yaml` (create or update)
- Runlog entry in `.claude/runlog/v1-deps-r1-runlog.md`
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`

## 3. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema

## 4. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: 153
- Record poll results in `.claude/runlog/v1-deps-r1-runlog.md`
- Do not post comments or submit reviews
