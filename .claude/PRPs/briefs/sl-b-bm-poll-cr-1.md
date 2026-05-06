---
role: bm-task
verb: bm-poll-cr
args: "119"
phase: v1-SL-b
created: 2026-05-06
---

# Brief — `bm-poll-cr` for PR #119 (phase-v1-SL-b)

## 1. Role + dispatch line

`[role:bm-task] bm-poll-cr 119 — see .claude/PRPs/briefs/sl-b-bm-poll-cr-1.md`

You are the **bm-task** subagent. Execute the `bm-poll-cr` verb for PR #119.

PR context at brief-write time:

- **PR:** #119
- **URL:** https://github.com/barrie-cork/lemmy/pull/119
- **Title:** `v1-SL-b — revoke_endorsement handler + DTO + route + 9 tests`
- **Base ← Head:** `governance-v0 ← phase-v1-SL-b`
- **State:** OPEN, non-draft
- **Head SHA:** `8f8ef09a6c64fecf49f7aa9363f75106b3cc1160`

## 2. Scope

**Produce:**
- Poll all CodeRabbit surfaces for PR #119.
- Write/update `.claude/PRPs/reviews/pr-119-findings.yaml` according to `.claude/PRPs/reviews/SCHEMA.md`.
- Preserve stable IDs on re-poll if the file already exists.
- Append BM runlog entry required by `.claude/commands/bm/bm-poll-cr.md` / `.claude/rules/branch-manager.md`.
- Return the counters table and next suggested step.

**Do NOT:**
- Post any PR comments.
- Submit a GitHub review.
- Merge, close, or retitle the PR.
- Touch implementation files (`crates/**`, `migrations/**`, `tests/**`, `Cargo.toml`, `Cargo.lock`).
- Send Telegram. If the script suggests `bm-ping cr-posted`, skip/record per Junior no-AskUserQuestion discipline; advisor will queue separately if wanted.

## 3. Required reading

1. `.claude/commands/bm/bm-poll-cr.md` — operational script, Phase 1–7.
2. `.claude/rules/branch-manager.md` — file ownership, autonomy, findings YAML rules.
3. `.claude/PRPs/reviews/SCHEMA.md` — findings schema.
4. `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory.
5. `.claude/agents/bm-task.md` or `.pi/skills/bm-task/SKILL.md` if available — Junior-dispatched bm-task contract.
6. Lessons matching this verb:
   - `feedback_coderabbit_*`
   - `feedback_pr_review_triage_pattern.md`
   - `feedback_pipes_mask_exit_codes.md`
   - `feedback_gh_pr_fork_repo_flag.md`

## 4. Constraints

- Use `gh pr view 119 --repo barrie-cork/lemmy` for PR metadata.
- Use `gh pr diff 119 --repo barrie-cork/lemmy --name-only` before trusting any walkthrough claim.
- Poll all three CodeRabbit endpoints named in `bm-poll-cr.md`:
  - `repos/barrie-cork/lemmy/pulls/119/reviews`
  - `repos/barrie-cork/lemmy/pulls/119/comments`
  - `repos/barrie-cork/lemmy/issues/119/comments`
- Parse severity from CodeRabbit's header second token.
- For walkthrough/pre-merge-check rows, disambiguate stable identity by `(source, cr_url, check_name)`.
- If CodeRabbit has not posted yet, write the script's zero-finding/warning state; do not fabricate findings.
- Commit and push any tracked BM artifacts/runlog changes to `phase-v1-SL-b`.
- If an operation would require asking the user, file a pending DQ blocker and stop; Junior `-p` cannot ask interactively.

## 5. Expected output

Return the standard `/bm-poll-cr` completion summary with:

- PR number and head SHA polled.
- Counts by source and severity.
- Findings YAML path.
- Any warning if CodeRabbit has not posted yet.
- Next suggested step: `/bm-prp-review 119` if findings ingestion is complete, or re-poll later if CodeRabbit has not posted.
