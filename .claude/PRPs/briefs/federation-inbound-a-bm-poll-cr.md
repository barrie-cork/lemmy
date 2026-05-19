---
phase: v1-federation-inbound-a
role: bm-task
task: bm-poll-cr
brief_n: 1
authored: 2026-05-18
canonical_ref: .claude/PRPs/briefs/sl-e-bm-poll-cr-1.md
---

# [role:bm-task] fed-in-a bm-poll-cr — ingest CodeRabbit + Copilot findings on PR #138 — see .claude/PRPs/briefs/federation-inbound-a-bm-poll-cr.md

## §1 Role + dispatch

`[role:bm-task] fed-in-a bm-poll-cr — ingest CR + Copilot findings on PR #138`

## §2 Scope

Run `bm-poll-cr` for PR #138 (`phase-v1-federation-inbound-a` → `governance-v0`).

CodeRabbit has posted its walkthrough comment + a `COMMENTED` review +
**inline review comments**. Copilot has also posted a `COMMENTED` review +
inline comments. Ingest **all** of them into the findings YAML.

**PR:** `#138`
**Branch:** `phase-v1-federation-inbound-a`
**Repo:** `barrie-cork/lemmy`
**Observed at brief-author time:** 29 inline review comments total —
**23 from `coderabbitai[bot]`** + **6 from `Copilot`** (via
`gh api repos/barrie-cork/lemmy/pulls/138/comments`). Plus the CR
summary/walkthrough comment + a CR review body (~17 KB) + a Copilot
review body (~6 KB). Re-fetch live counts at task time — they may have
grown.

Write `.claude/PRPs/reviews/pr-138-findings.yaml` per
`.claude/PRPs/reviews/SCHEMA.md`.

## §3 Required reading

- `.claude/commands/bm/bm-poll-cr.md` — bm-poll-cr verb
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (id / bucket / source / severity / addressed_in / counters invariants)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + findings-YAML schema discipline
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` mandatory

## §4 Constraints

- `--repo barrie-cork/lemmy` on all `gh` commands.
- Read **all** inline review comments via
  `gh api repos/barrie-cork/lemmy/pulls/138/comments` (paginate if
  >30 — use `--paginate`). These carry both `coderabbitai[bot]` and
  `Copilot` authored comments.
- Read the CR walkthrough/summary comment via
  `gh pr view 138 --repo barrie-cork/lemmy --json comments` (filter
  `author.login == "coderabbitai"`).
- Read the two `COMMENTED` reviews via
  `gh pr view 138 --repo barrie-cork/lemmy --json reviews` (CR review
  body + Copilot review body — these may contain top-level findings
  not attached to a line).
- Write findings YAML with every finding's:
  - stable `id` — `cr-1`, `cr-2`, … for `coderabbitai[bot]`;
    `copilot-1`, `copilot-2`, … for `Copilot` (distinct prefixes per
    source so triage can filter).
  - `bucket: ""` (left blank — triage is a separate verb).
  - `source:` + `reviewer:` — **ADVISOR DECISION (do NOT raise a
    blocker on this; it is resolved here):** SCHEMA.md's `source`
    enum is strictly `coderabbit | claude | user` and has no
    `copilot` value. Both CodeRabbit and Copilot are automated code
    reviewers, so:
      - CR findings: `source: coderabbit`, `reviewer: coderabbit`.
      - Copilot findings: `source: coderabbit`, `reviewer: copilot`.
    The `reviewer:` sub-field is an additive discriminator (SCHEMA.md
    permits additive fields per its "forward-only" note). `cr_url` is
    still required for every `source: coderabbit` finding — use the
    GitHub discussion/comment `html_url` from the API for both CR and
    Copilot comments (Copilot inline comments do carry an `html_url`).
    This keeps the schema valid AND lets `bm-triage` filter Copilot
    vs CR via `reviewer:`. (Advisor logged this as a `kind: log` DQ
    for a SCHEMA.md `reviewer:` documentation amendment at retro —
    you do not need to file anything.)
  - `severity` per the reviewer's hint (critical / major / medium /
    low / nit — CR tags severity; Copilot is usually advisory →
    default `low` unless it flags a bug).
  - `summary` (one line), `file` + `line`, `recommendation`
    (the reviewer's suggested fix verbatim or tightly paraphrased).
- Regenerate the `counters` block from `findings[]`.
- Set `last_poll_at` (ISO 8601 UTC) + `poll_count` (1 for first poll).
- Do **NOT** bucket findings (triage is the separate `bm-triage` verb).
- Do **NOT** post any reply/comment on the PR.
- Do **NOT** edit `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership boundary).
- Commit the findings YAML to the **lane phase branch**
  `phase-v1-federation-inbound-a` (subject
  `chore(pr-review): ingest PR #138 CR+Copilot findings`) and push.
  (`.claude/PRPs/reviews/pr-138-findings.yaml` is gitignored as a
  runtime artifact per branch-manager.md — if gitignored, the commit
  is a no-op; in that case just leave the file in the worktree and
  report its path + the finding counts in the task output. Verify
  `.gitignore` status first with `git check-ignore -q
  .claude/PRPs/reviews/pr-138-findings.yaml; echo $?`.)
- Report in the task output: total findings, count by source
  (coderabbit / copilot), count by severity, and the findings-YAML
  path.

## §5 Out of scope

- Triage / bucketing (separate `bm-triage` verb → USER-GATE-3).
- Any fix-in-PR commit (post-triage impl-task work).
- Merging the PR.
- Posting a PR comment or review reply.
