> **[CLOSED — shipped 2026-05-24, advisory-lock phase]** This brief is from the
> completed `v1-federation-inbound-e` sub-phase (pg_advisory_xact_lock eviction fix).

# Brief: bm-merge — v1-federation-inbound-e

## 1. Role + dispatch

`[role:bm-task] bm-merge v1-federation-inbound-e — merge PR #148 into governance-v0`

## 2. Scope

Merge PR #148 (`phase-v1-federation-inbound-e` → `governance-v0`) with `--repo barrie-cork/lemmy`.

**Pre-merge gate checks (read-only — confirm all pass before merging):**
- `mergeStateStatus == "CLEAN"` ✓ (confirmed 2026-05-23T09:30 UTC)
- `mergeable == "MERGEABLE"` ✓
- 0 open `severity: critical` findings in `fix-in-pr` bucket ✓
- `/brehon-verify` all 3 stories ✓ (report at `.claude/PRPs/reports/v1-federation-inbound-e-verify.md`)
- Conformance audit 0 Tier-1/2/3 findings ✓ (report at `.claude/PRPs/reports/conformance-audit-phase-diff-fed-in-e-2026-05-23.md`)
- Gate 5 user approval: given ✓

**Merge action:**
- `gh pr merge 148 --repo barrie-cork/lemmy --merge` (no squash — task-per-commit history is load-bearing)
- Do NOT squash, do NOT rebase

**Do NOT:** open new PRs, post comments, approve reviews, or do anything other than the merge.

## 3. Required reading

- `.claude/rules/branch-manager.md` — file-ownership, autonomy bounds, merge gate
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory
- `.claude/rules/phase-branch.md` — do not squash; task-per-commit history load-bearing

## 4. Constraints

- `--repo barrie-cork/lemmy` on every `gh pr` command
- `--merge` (not `--squash`, not `--rebase`)
- Never merge with open `severity: critical` findings in `fix-in-pr` bucket
- Write result (merge SHA + PR URL) to `.claude/runlog/bm-runlog.md`
