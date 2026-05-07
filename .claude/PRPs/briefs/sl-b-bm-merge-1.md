---
role: bm-task
phase: v1-SL-b
created: 2026-05-07
---

# Brief — `sl-b-bm-merge-1` — merge PR #119 into governance-v0

## 1. Role + dispatch line

`[role:bm-task] sl-b-bm-merge-1 — see .claude/PRPs/briefs/sl-b-bm-merge-1.md`

## 2. Scope

Run `bm-merge` for PR #119 (`phase-v1-SL-b` → `governance-v0`).

**State at brief time:**
- Recommendation: `approve` (0 fix-in-pr open, 0 critical open)
- Last poll: poll #2, head SHA `3f119ce82`
- All validation green: DQ #155 (workspace-check cr-7), DQ #157 (workspace-check fix-impl-4), DQ #158 (e2e tip c87179af4) — all `result: "pass"` in `resolved[]`
- DQ #156: historical fail record in `pending[]` — this is not a blocking entry; it was the e2e run on tip `620861f08` (before fix-impl-4 landed) and is intentionally left in `pending[]` per option-2 rules. Do NOT treat it as a blocker for merge.
- Triage digest comment posted: https://github.com/barrie-cork/lemmy/pull/119#issuecomment-4400616726

**Merge conflict:** `decision-queue.json` and `PI_AUDIT_REPORT.md` differ between
`phase-v1-SL-b` (154+ entries, in-flight state) and `governance-v0` (empty pending).
Resolution: keep the **phase-branch versions** of both files (they contain the full
v1-SL-b DQ history and the current PI audit state). Do NOT keep the governance-v0
versions.

**Do NOT:**
- Squash (use `--merge` — task-per-commit history is load-bearing for retros)
- Merge without asking user first (branch-manager.md autonomy: merge requires confirm)
- Re-run bm-poll-cr (CR was polled at poll #2 on the current HEAD `3f119ce82`; no new
  commits since then)

**Produce:**
- Merged PR #119 into governance-v0 (after user confirm)
- Local governance-v0 fast-forwarded to merge commit
- Findings YAML archived with `merged_at` + `merge_commit` fields
- Runlog entry appended

## 3. Required reading

1. `.claude/commands/bm/bm-merge.md` — follow all phases
2. `.claude/rules/branch-manager.md` — autonomy bounds, file ownership
3. `.claude/rules/phase-branch.md` — `--merge` not `--squash`
4. `.claude/rules/gh-pr-fork-target.md` — always `--repo barrie-cork/lemmy`
5. `.claude/PRPs/reviews/pr-119-findings.yaml` — verify counters before merge
6. `.claude/decision-queue.json` — note DQ #156 is non-blocking (historical fail)

## 4. Constraints

- ASK user before `gh pr merge` (mandatory per branch-manager.md autonomy table)
- Use `--merge` not `--squash`
- Always pass `--repo barrie-cork/lemmy` to all `gh` commands
- Merge conflict resolution: keep phase-branch versions of `decision-queue.json`
  and `PI_AUDIT_REPORT.md`
- Do not touch `crates/`, `migrations/`, `tests/`, `docs/brehon-law-inspired-network/`
- Do not create PRs into `main`
