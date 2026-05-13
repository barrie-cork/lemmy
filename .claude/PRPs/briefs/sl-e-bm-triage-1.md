---
phase: v1-SL-e
role: bm-task
task: bm-triage
brief_n: 7
authored: 2026-05-13
---

# [role:bm-task] SL-e bm-triage — draft four-bucket triage on PR #127 — see .claude/PRPs/briefs/sl-e-bm-triage-1.md

## §1 Role + dispatch

`[role:bm-task] SL-e bm-triage — draft four-bucket triage on PR #127 (9 findings: 4 CR major + 1 CR nit + 4 Copilot medium)`

## §2 Scope

Run `bm-triage` for PR #127.

Findings YAML at `.claude/PRPs/reviews/pr-127-findings.yaml` has **9 findings**:
- **cr-1, cr-2, cr-3** (major, source=coderabbit) — DQ ID immutability + scope-decision surfacing + impossible chronology in `.claude/decision-queue.json`
- **cr-4** (nit, source=coderabbit) — MD022 blank-line in runlog
- **cr-5** (major, source=coderabbit) — `BREHON_DISABLE_GRACE_CHECK_JOB` env var restore on failure paths in e2e.rs
- **copilot-1** (medium, source=copilot) — env var RAII guard (same root issue as cr-5)
- **copilot-2** (medium, source=copilot) — `sponsor1`/`sponsors[1]` naming confusion
- **copilot-3** (medium, source=copilot) — 5s grace tolerance flake risk
- **copilot-4** (medium, source=copilot) — `revoked_at` "recent" check bounds

Draft the triage: assign `bucket` per `.claude/refs/pr-review-triage.md` four-bucket pattern (fix-in-pr / rebut / carry-forward / done / wont-fix) and write `rationale` for each.

Then draft `.claude/PRPs/reviews/pr-127-comment.md` summarising the triage for user review before posting.

**Do NOT post the PR comment.** User must approve via user gate 3 first.

## §3 Required reading

- `.claude/commands/bm/bm-triage.md` — bm-triage verb
- `.claude/refs/pr-review-triage.md` — four-bucket triage pattern
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema
- `.claude/lessons/feedback_pr_review_triage_pattern.md` — bucketing heuristics
- `.claude/PRPs/reports/v1-SL-e-retro.md` — context for "intentional choices" vs "drift" classification
- Each CR/Copilot finding's `cr_url` — read the full review text via `gh api repos/barrie-cork/lemmy/pulls/comments/<id>` if needed

## §4 Constraints

- **--repo barrie-cork/lemmy** on all gh commands
- Buckets: every finding gets one of `fix-in-pr` / `rebut` / `carry-forward` / `done` / `wont-fix`
- Write `rationale` for each finding (1-2 sentences; what bucket + why)
- For `bucket: fix-in-pr` findings: leave `addressed_in: null` (impl will fill on fix commit)
- For `bucket: rebut` findings: rationale must explain why CR/Copilot was wrong; user gate decides whether to post rebuttal comment
- For `bucket: done` findings: rationale cites the existing commit SHA that addresses it
- Note: cr-5 + copilot-1 are about the **same root issue** (env var restore on failure paths). Recommend grouping them — either both `fix-in-pr` with shared fix commit, or one `done` referencing the other.
- Update `counters` block per `findings[]`
- Write draft PR comment at `.claude/PRPs/reviews/pr-127-comment.md` (summary by bucket, plain Markdown)
- Do NOT post the PR comment
- Do NOT push to phase-v1-SL-e until triage YAML + comment draft are committed atomically
