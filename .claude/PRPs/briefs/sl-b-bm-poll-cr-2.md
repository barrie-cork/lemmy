---
role: bm-task
verb: bm-poll-cr
args: "119"
phase: v1-SL-b
created: 2026-05-07
---

# Brief — `bm-poll-cr` re-poll for PR #119 (phase-v1-SL-b) after CR fixes

## 1. Role + dispatch line

`[role:bm-task] bm-poll-cr 119 — see .claude/PRPs/briefs/sl-b-bm-poll-cr-2.md`

You are the **bm-task** subagent. Execute the `bm-poll-cr` verb for PR #119.
This is a **re-poll** after the CR fix-in-PR commits landed. The PR head has
advanced since the previous poll-cr run. CodeRabbit should have re-reviewed.

PR context at brief-write time:

- **PR:** #119
- **URL:** https://github.com/barrie-cork/lemmy/pull/119
- **Title:** `v1-SL-b — revoke_endorsement handler + DTO + route + 9 tests`
- **Base ← Head:** `governance-v0 ← phase-v1-SL-b`
- **State:** OPEN, non-draft
- **Head SHA at brief write:** `68de0e951` (latest: DQ mutation commits + advisor fix)
- **Prior poll head SHA:** `8f8ef09a6c64fecf49f7aa9363f75106b3cc1160`

Fix commits landed since last poll-cr:
- `a639d5d0d` Junior #131 impl (CR fixes: cr-1 through cr-4)
- `a9719fa82` advisor fix: unused_assignments (clippy §G4 hand-fix)

## 2. Scope

**Produce:**
- Re-poll all CodeRabbit surfaces for PR #119.
- Update `.claude/PRPs/reviews/pr-119-findings.yaml` per `.claude/PRPs/reviews/SCHEMA.md`.
  - Preserve stable IDs on existing findings.
  - Update `bucket` for findings that were `fix-in-pr` and are now addressed:
    set `bucket: done` and `addressed_in: <commit-sha>` for each confirmed fix.
  - Add any new findings CodeRabbit raised on the updated diff.
  - Update `last_poll_at`, `poll_count`, `head_sha` on the file header.
- Append BM runlog entry per `.claude/commands/bm/bm-poll-cr.md`.
- Return the four-bucket counters and next suggested step.

**Do NOT:**
- Post any PR comments.
- Submit a GitHub review.
- Merge, close, or retitle the PR.
- Touch implementation files (`crates/**`, `migrations/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`).
- Send Telegram pings (Junior `-p` cannot ask interactively; skip ping steps).

## 3. Required reading

1. `.claude/commands/bm/bm-poll-cr.md` — operational script, Phase 1–7.
2. `.claude/rules/branch-manager.md` — file ownership, autonomy, findings YAML rules.
3. `.claude/PRPs/reviews/SCHEMA.md` — findings schema.
4. `.claude/PRPs/reviews/pr-119-findings.yaml` — existing findings file; read before polling so you can diff old vs new.
5. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
6. Lessons:
   - `.claude/lessons/feedback_coderabbit_triage_four_buckets_confirmed.md`
   - `.claude/lessons/feedback_pr_review_triage_pattern.md`
   - `.claude/lessons/feedback_pipes_mask_exit_codes.md`
   - `.claude/lessons/feedback_gh_pr_fork_repo_flag.md`

## 4. Constraints

- Use `gh pr view 119 --repo barrie-cork/lemmy` for PR metadata; confirm head SHA matches.
- Poll all three CodeRabbit endpoints:
  - `repos/barrie-cork/lemmy/pulls/119/reviews`
  - `repos/barrie-cork/lemmy/pulls/119/comments`
  - `repos/barrie-cork/lemmy/issues/119/comments`
- Parse severity from CodeRabbit's header second token.
- If CodeRabbit has not yet re-reviewed the updated head (i.e. most recent CR comment
  still references the old head SHA), write a warning; do not fabricate updated findings.
- Commit and push any tracked BM artifacts / runlog changes to `phase-v1-SL-b`.
- If an operation requires user input, file a pending DQ blocker and stop.

## 5. Expected output

Return the standard `/bm-poll-cr` completion summary with:

- PR number and head SHA polled.
- Counts by bucket: `fix-in-pr | rebut | carry-forward | done | wont-fix`.
- Any remaining open `critical` findings in `fix-in-pr`.
- Whether CodeRabbit has reviewed the updated head yet.
- Next suggested step: bm-triage if new findings, or bm-merge readiness check if
  all critical findings are `done`/`rebut`/`wont-fix`.
