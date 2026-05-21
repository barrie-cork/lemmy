---
phase: brehon-conformance-audit
role: bm-task
task: bm-poll-cr
brief_n: 1
authored: 2026-05-21
canonical_ref: .claude/PRPs/briefs/federation-inbound-a-bm-poll-cr.md
---

# [role:bm-task] brehon-conformance-audit bm-poll-cr — ingest CodeRabbit findings on PR #141 — see .claude/PRPs/briefs/brehon-conformance-audit-bm-poll-cr.md

## §1 Role + dispatch

`[role:bm-task] brehon-conformance-audit bm-poll-cr — ingest CR findings on PR #141`

## §2 Scope

Run `bm-poll-cr` for PR #141 (`phase-brehon-conformance-audit` → `governance-v0`).

CodeRabbit has posted its walkthrough/summary comment + **22 inline review
comments** + likely a `COMMENTED` review body. Ingest all of them into the
findings YAML.

**PR:** `#141`
**Branch:** `phase-brehon-conformance-audit`
**Repo:** `barrie-cork/lemmy`
**Observed at brief-author time:** **22 inline `coderabbitai[bot]` comments**
on `repos/barrie-cork/lemmy/pulls/141/comments` + 1 walkthrough/summary
comment on `repos/barrie-cork/lemmy/issues/141/comments` (created
2026-05-21T11:58:41Z, ~30s after PR open). Re-fetch live counts at task
time — they may have grown (CR sometimes drips additional findings in a
second pass).

This PR delivers the **brehon-conformance-audit skill + Clippy structural
enforcement gate**. Diff scope (per `gh pr diff --name-only`):
- `.claude/skills/brehon-conformance-audit/**` — new skill bundle (markdown
  + bash + Python under `axes/`, `scripts/`, `METRICS.md`, `SKILL.md`).
- `.claude/rules/advisor-orchestrator.md` — Cohort 5 Task 11 wiring.
- `.claude/lessons/feedback_*.md` — two paired new lessons (Cohort 5
  Task 12) + one retro-promoted lesson (`feedback_clippy_per_module_
  deny_requires_workspace_allow.md`).
- `clippy.toml` — `disallowed-methods` entries (Task 8 v2).
- Root `Cargo.toml` — workspace-allow override for `disallowed_methods`
  per rustc lint-precedence rule 4 (Task 8 v2 mechanism revision).
- 3 federation governance `mod.rs` files: `crates/apub/activities/src/
  governance/mod.rs`, `crates/api/api/src/governance/mod.rs`, `crates/
  db_schema/src/source/governance/mod.rs` — per-module `#![deny(
  clippy::disallowed_methods)]` (Task 9).
- `crates/utils/src/**` (fix-impl-1 — 6-site `unwrap_or_default` →
  explicit `unwrap_or_else`) + `crates/diesel_utils/src/**` (fix-impl-3
  — 4-site equivalent). Pre-mechanism-revision residue retained as
  net-positive cleanup per retro §2.
- `.mcp.json.example` — rust-analyzer-mcp server entry (Task 10).
- `.claude/PRPs/plans/brehon-conformance-audit.plan.md` — plan file
  (Task 1).
- `.claude/PRPs/reports/brehon-conformance-audit-retro.md` — Task 13
  retro (advisor-authored at User Gate 6 sign-off 2026-05-21).
- `.claude/PRPs/audit-metrics/v1-federation-inbound-b.json` — dogfood
  metrics file (Task 7).
- `.claude/PRPs/reports/conformance-audit-v1-federation-inbound-b-
  dogfood-2026-05-20.md` — dogfood report (Task 7).

89 commits ahead of `governance-v0`.

Write `.claude/PRPs/reviews/pr-141-findings.yaml` per
`.claude/PRPs/reviews/SCHEMA.md`.

## §3 Required reading

- `.claude/commands/bm/bm-poll-cr.md` — bm-poll-cr verb (Phase 1–8
  operational script)
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (id /
  bucket / source / severity / addressed_in / counters invariants)
- `.claude/rules/branch-manager.md` — BM autonomy bounds +
  findings-YAML schema discipline
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`
  mandatory
- `.claude/lessons/feedback_pr_review_triage_pattern.md` — four-bucket
  pattern (ingest-only here; triage is the separate `bm-triage` verb)

## §4 Constraints

- `--repo barrie-cork/lemmy` on every `gh` command (fork default is
  upstream LemmyNet/lemmy).
- Pull comments from all three CR API endpoints per bm-poll-cr.md
  Phase 2 (paginate if needed via `--paginate`):
  - `repos/barrie-cork/lemmy/pulls/141/reviews` — PR-level review
    summaries (`COMMENTED` body — may carry top-level findings not
    attached to a line).
  - `repos/barrie-cork/lemmy/pulls/141/comments` — **22 inline
    review comments** (per-file/per-line).
  - `repos/barrie-cork/lemmy/issues/141/comments` — issue-level PR
    comments (1 CR walkthrough/summary).
- Parse severity from CR header **second token** per bm-poll-cr.md
  Phase 3 (Critical / Major / Medium / Minor → `nit`).
- Assign stable `cr-<seq>` IDs in chronological order across all
  three sources (review + inline + issue).
- Write findings YAML with every finding's:
  - stable `id` — `cr-1`, `cr-2`, … for `coderabbitai[bot]`.
  - `bucket: ""` (left blank — triage is a separate verb).
  - `source: coderabbit`, `reviewer: coderabbit` (per the
    fed-in-a SCHEMA.md advisor decision recap; in this PR there is
    no Copilot review observed at brief-author time — only CR — so
    no `copilot-N` IDs needed).
  - `severity` per the reviewer's CR-header second token.
  - `summary` (~120 chars max), `file` + `line`, `cr_url`
    (the `html_url` from the GitHub API).
  - `recommendation` — the reviewer's suggested fix verbatim or
    tightly paraphrased; fenced code blocks captured verbatim into
    `notes:` without reformatting.
  - `outside-diff: true` in `notes:` if `file:` is not in
    `.cr-cache/pr-141-diff-files.txt` (ground-truth diff list per
    Phase 1.5).
- Regenerate the `counters` block from `findings[]` after merging
  per Phase 4 rules.
- Set `last_poll_at` (ISO 8601 UTC) + `poll_count: 1` (first poll).
- Compute + store `last_polled_head_sha` (from Phase 1 `headRefOid`)
  + `last_polled_fingerprint` (per Phase 5.1 `sha256` of
  `id:base64(body)` over all CR comments).
- Do **NOT** bucket findings — triage is the separate `bm-triage`
  verb at User Gate 3.
- Do **NOT** post any reply/comment on the PR (autonomy boundary —
  visible-to-others actions need confirmation).
- Do **NOT** edit `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership boundary
  per `.claude/rules/branch-manager.md`).
- Force-add the findings YAML to git so it survives worktree cleanup
  (the `.gitignore` covers `.claude/PRPs/reviews/pr-*-findings.yaml`
  by default; `git add -f` is required). Commit + push to
  `phase-brehon-conformance-audit`:
  ```
  git add -f .claude/PRPs/reviews/pr-141-findings.yaml
  git commit -m "chore(bm): poll-cr #141 — ingest CR findings (poll #1)"
  git push origin phase-brehon-conformance-audit
  ```
- Append a Phase 7 entry to `.claude/runlog/bm-runlog.md` on the
  phase branch (NOT trunk per L14 — runlog COMPLETE writes happen
  POST-merge only; this `bm-poll-cr` entry is a phase-branch
  Phase-7 entry per the verb's operational script).
- Report in task output: total findings, breakdown by severity
  (critical / major / medium / low / nit), count of `outside-diff`
  flags, findings-YAML path, and runlog path.

## §5 Out of scope

- Triage / bucketing (separate `bm-triage` verb → USER-GATE-3).
- Any fix-in-PR commit (post-triage impl-task work).
- Merging the PR (User Gate 5 + `bm-merge`).
- Posting a PR comment or review reply.
- Running `/bm-prp-review` cargo validation (not part of bm-poll-cr;
  separate verb if needed).

## §6 Commit subject

The BM Junior's own commit uses `chore(bm): poll-cr #141 — ingest CR
findings (poll #1)` per the verb's Phase 6 template (matches
`.claude/rules/branch-manager.md` autonomy rules).
