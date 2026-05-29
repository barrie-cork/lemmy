# Brief: v1-quality-r2a BM-POLL-CR

## 1. Role + dispatch

`[role:bm-task] v1-quality-r2a-bm-poll-cr — see .claude/PRPs/briefs/v1-quality-r2a-bm-poll-cr-1.md`

## 2. Scope

Poll CodeRabbit findings on PR #161 (`phase-v1-quality-r2 → governance-v0`) on `barrie-cork/lemmy`.

**Produce:**
- Findings YAML at `.claude/PRPs/reviews/pr-161-findings.yaml` (create new — first poll on this PR)
- Runlog entry in `.claude/runlog/v1-quality-r2a-runlog.md`
- Summary of finding counts by bucket + severity in task output

**Do NOT:**
- Merge the PR
- Post any PR comments or submit reviews
- Touch any file in `crates/`, `Cargo.toml`, `Cargo.lock`, `.claude/PRPs/plans/`, `docs/`, `tests/`, `migrations/`

## 3. Context

CR already posted 4 actionable comments at PR-open time. Copilot also reviewed. Both reviews are present BEFORE this bm-poll-cr runs — no wait required; the worker can proceed directly to YAML authorship.

PR scope reminder: 2 bash scripts (`scripts/brehon/dq-lint-durations.sh` + `scripts/brehon/precheck.sh`), 3 DQ sweep edits to back-dated entries, 1 new `kind: log` DQ entry, 1 debug fragment by-product. Zero Rust. The CR findings should focus on bash quality (shellcheck patterns, error handling) + JSON shape.

## 4. Required reading

- `.claude/rules/branch-manager.md` — file-ownership boundaries, autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema

## 5. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- PR number: 161
- Record poll results in `.claude/runlog/v1-quality-r2a-runlog.md`
- Do not post comments or submit reviews
- Initial bucket for all findings: default to `fix-in-pr` for CRITICAL/MAJOR, `carry-forward` candidates flagged for advisor triage at gate-3
