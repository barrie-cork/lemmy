---
phase: v1-federation-inbound-a
role: bm-task
task: bm-triage
brief_n: 1
authored: 2026-05-18
canonical_ref: .claude/PRPs/briefs/sl-e-bm-triage-1.md
---

# [role:bm-task] fed-in-a bm-triage — draft four-bucket triage on PR #138 — see .claude/PRPs/briefs/federation-inbound-a-bm-triage.md

## §1 Role + dispatch

`[role:bm-task] fed-in-a bm-triage — draft four-bucket triage on PR #138 (29 findings: 1 critical + 1 major + 21 medium + 6 low; 23 CR + 6 Copilot)`

## §2 Scope

Run `bm-triage` for PR #138 (`phase-v1-federation-inbound-a` → `governance-v0`).

Findings YAML at `.claude/PRPs/reviews/pr-138-findings.yaml` has
**29 findings** (all `source: coderabbit`; `reviewer:` discriminator
distinguishes CR vs Copilot):

- **1 critical:** `cr-21` — `crates/api/api/src/governance/config.rs:1671` ("Potential issue / Critical / Quick win").
- **1 major:** `cr-9` — `.claude/PRPs/briefs/federation-inbound-a-impl-1.md:31` (a brief-file finding).
- **21 medium**, **6 low** — distributed across `crates/**`, migrations, briefs, runlog, decision-queue.
- `reviewer:` field: 6 findings have `reviewer: copilot`; the other 23 are CR (some may have `reviewer:` unset — treat unset as `reviewer: coderabbit`).

Draft the triage: assign `bucket` per `.claude/refs/pr-review-triage.md`
four-bucket pattern (`fix-in-pr` / `rebut` / `carry-forward` / `done` /
`wont-fix`) and write `rationale` for each finding.

Then draft `.claude/PRPs/reviews/pr-138-comment.md` summarising the
triage for user review **before** posting.

**Do NOT post the PR comment.** User must approve via USER-GATE-3 first.

## §3 Required reading

- `.claude/commands/bm/bm-triage.md` — bm-triage verb
- `.claude/refs/pr-review-triage.md` — four-bucket triage pattern
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (bucket / rationale / addressed_in invariants)
- `.claude/lessons/feedback_pr_review_triage_pattern.md` — bucketing heuristics
- `.claude/lessons/feedback_severity_labels_dont_imply_semantic.md` — severity label ≠ semantic impact (a "critical"-tagged CR finding may be low real-impact and vice versa; judge on actual effect, not the label)
- `.claude/lessons/feedback_verify_automated_reviewer_claims_against_compiler.md` — CR/Copilot trait/type/API claims are hypotheses; for any finding proposing a code change that contradicts a compiling state or a user-ratified decision, the rationale MUST note "compile-verify before fix-in-pr" (do NOT bucket as fix-in-pr a change that would break a currently-compiling callsite without flagging it)
- Each finding's `cr_url` — read the full review text via `gh api repos/barrie-cork/lemmy/pulls/comments/<id>` when the summary alone is insufficient to bucket

## §4 Constraints

- **--repo barrie-cork/lemmy** on all `gh` commands.
- Every finding gets exactly one bucket: `fix-in-pr` / `rebut` /
  `carry-forward` / `done` / `wont-fix`. Never empty, never invented.
- Write `rationale` for each finding (1-2 sentences: what bucket + why).
- **Brief-file findings** (findings whose `file` is under
  `.claude/PRPs/briefs/**` or `.claude/runlog/**` or
  `.claude/decision-queue.json`) — these are advisor meta-artifacts,
  NOT shipped code. Default bucket `wont-fix` or `carry-forward` with
  rationale "advisor meta-doc; CR prose-quality finding, not a code
  defect" UNLESS the finding identifies a factual error that would
  mislead a future reader (then `fix-in-pr`). Do NOT auto-`fix-in-pr`
  a markdown-lint or prose-style nit on a brief file.
- **The 1 critical (`cr-21`, config.rs:1671)** — this is real code.
  Read its full `cr_url` text. Bucket on actual impact per
  `feedback_severity_labels_dont_imply_semantic` +
  `feedback_verify_automated_reviewer_claims_against_compiler`: if it
  is a genuine logic/safety defect → `fix-in-pr`; if CR misread the
  code (the surrounding code compiles + the e2e suite is green at tip
  `122187ebd`) → `rebut` with a precise why. Do NOT reflexively
  `fix-in-pr` solely because the label is "critical".
- For `bucket: fix-in-pr`: leave `addressed_in: null` (impl fills on
  fix commit).
- For `bucket: rebut`: rationale must explain precisely why CR/Copilot
  was wrong (cite the compiling code / the user-ratified decision /
  the e2e-green evidence). User gate decides whether to post the
  rebuttal.
- For `bucket: done`: rationale cites the existing commit SHA that
  already addresses it.
- Group findings that share a root cause (note the grouping in each
  member's rationale; recommend a single shared fix commit).
- Regenerate the `counters` block from `findings[]` (open/done/
  rebutted/carry_forward/wont_fix per severity).
- Write the draft PR comment at `.claude/PRPs/reviews/pr-138-comment.md`
  (summary by bucket, plain Markdown, group by reviewer where useful).
- **Do NOT post the PR comment** (USER-GATE-3 decides).
- **Do NOT push** to `phase-v1-federation-inbound-a` until the triage
  YAML + comment draft are ready. NOTE: `pr-138-findings.yaml` +
  `pr-138-comment.md` are **gitignored** (runtime artifacts per
  branch-manager.md) — verify with `git check-ignore -q <path>; echo $?`
  (exit 0 = ignored). If gitignored, the "commit atomically" step is a
  no-op; instead leave both files in the worktree and report their
  paths + the per-bucket counts in the task output. (The advisor reads
  them from the daemon worktree for USER-GATE-3.)
- Do NOT edit `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership boundary).
- Report in task output: per-bucket counts (fix-in-pr / rebut /
  carry-forward / done / wont-fix), the critical finding's bucket +
  one-line rationale, and both artifact paths.

## §5 Out of scope

- Posting the PR comment (USER-GATE-3).
- Any fix-in-PR code commit (post-gate impl-task work).
- Merging the PR.
- Editing the findings' underlying code.
