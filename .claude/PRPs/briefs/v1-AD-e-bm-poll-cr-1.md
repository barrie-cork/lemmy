---
phase: v1-AD-e
role: bm-task
task: bm-poll-cr
brief_n: 1
authored: 2026-05-17
---

# [role:bm-task] AD-e bm-poll-cr — ingest CodeRabbit findings on PR #133 — see .claude/PRPs/briefs/v1-AD-e-bm-poll-cr-1.md

## §1 Role + dispatch

`[role:bm-task] AD-e bm-poll-cr — ingest CodeRabbit findings on PR #133`

## §2 Scope

Run `bm-poll-cr` for PR #133 (`phase-v1-AD-e` → `governance-v0`).

CodeRabbit has posted its review with **6 actionable findings** (the
`coderabbitai` PR-level review submitted 2026-05-17T08:18:47Z, body opens
`**Actionable comments posted: 6**`). Ingest all 6 into the findings YAML
plus any inline review comments and the walkthrough/issue comment.

**PR:** `#133`
**Branch:** `phase-v1-AD-e`
**Repo:** `barrie-cork/lemmy`

Write `.claude/PRPs/reviews/pr-133-findings.yaml` per
`.claude/PRPs/reviews/SCHEMA.md`. Force-add + commit + push it (gitignored
by default — MUST be committed so the advisor can retrieve it without
relying on worktree persistence).

### §2a Source landscape (read before pulling — three review events on this PR)

`gh pr view 133` shows **3** review/comment events. Disambiguate:

1. **`coderabbitai` issue comment @ 08:15:37Z** — `No actionable comments
   were generated in the recent review. 🎉` — this was CR's pass against
   the **original tip** (before the advisor's governance-v0 merge). It is
   a **superseded/stale** "no findings" notice — NOT the authoritative
   review. Do not let it zero out the ingest.
2. **`copilot-pull-request-reviewer` review @ 08:17:19Z** (state
   COMMENTED) — Copilot's PR overview. **OUT OF SCOPE for bm-poll-cr**
   (this verb ingests CodeRabbit only, per the canonical command). Do NOT
   ingest Copilot comments as `source: coderabbit`. Note in the runlog
   that a Copilot review exists so the later `bm-triage` / `bm-prp-review`
   step can fold it in as `source: claude` if it has actionable content.
3. **`coderabbitai` review @ 08:18:47Z** (state COMMENTED) —
   `**Actionable comments posted: 6**` — **THIS is the authoritative CR
   review.** Its 6 findings + any inline `pulls/133/comments` rows are
   what to ingest.

### §2b Head-SHA-vs-finding-timestamp skew (do NOT treat as a problem)

CR's 6-findings review (08:18:47Z) was posted against a tip **earlier**
than the current PR head `856d4f444` (the advisor merged governance-v0
into phase-v1-AD-e at ~09:18Z to clear a CONFLICTING state). **The merge
added ZERO code changes** (verified: `git diff --stat <pre-merge> HEAD --
crates/ migrations/ tests/` is empty — it only absorbed governance-v0
meta-work: retros/lessons/rule updates). Therefore every CR finding still
applies verbatim to the same code at the same `file:line`. Phase 5.1's
skip-the-write short-circuit will correctly PROCEED (head SHA changed
since there is no prior YAML — this is poll #1). Do not annotate findings
as outside-diff merely because their `posted_at` predates `856d4f444`;
ground-truth `file:` against `gh pr diff 133 --name-only` per Phase 1.5
as normal.

## §3 Required reading

- `.claude/commands/bm/bm-poll-cr.md` — bm-poll-cr operational script
  (follow phases 1→8 verbatim; note Phase 1.5 diff ground-truth, Phase 3
  severity-token parse, Phase 4 stable `cr-<seq>` IDs, Phase 5.1
  skip-write short-circuit, Phase 6 force-add+commit+push)
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema (stable `id`,
  `bucket: ""` left blank for triage, `source`, `severity`, `summary`,
  `file:line`, `recommendation`, regenerated `counters`)
- `.claude/rules/branch-manager.md` — BM autonomy bounds + file-ownership
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy` on
  every gh command
- `.claude/lessons/feedback_python_utf8_encoding_windows.md` — YAML write
  MUST use `encoding="utf-8"` + `allow_unicode=True` (CR severity emoji)

## §4 Constraints

- **`--repo barrie-cork/lemmy`** on every `gh` command.
- Pull all 3 CR sources per command Phase 2 (one query each):
  - `repos/barrie-cork/lemmy/pulls/133/reviews` (PR-level — the 6-findings
    review lives here)
  - `repos/barrie-cork/lemmy/pulls/133/comments` (inline per-file/line)
  - `repos/barrie-cork/lemmy/issues/133/comments` (walkthrough / pre-merge
    check)
  - **Filter `select(.user.login == "coderabbitai[bot]")`** — this is what
    excludes the Copilot review automatically. Do NOT widen the filter.
- Parse severity from CR's second header token per the Phase 3 table
  (`Critical|Major|Medium|Minor|Nitpick` → `critical|major|medium|low|nit`).
  The emoji is decoration; the token is authoritative.
- Assign stable `cr-<seq>` IDs in chronological order across all CR
  sources. This is poll #1 so all are new (`cr-1`..`cr-N`).
- **`bucket: ""`** left blank on every finding — **do NOT bucket**
  (triage is the next, separate verb; bucketing here is a process breach).
- Regenerate `counters` from `findings[]`; set `last_poll_at`,
  `poll_count: 1`, `last_polled_head_sha: 856d4f444`,
  `last_polled_fingerprint`.
- Write `.claude/PRPs/reviews/pr-133-findings.yaml` with
  `encoding="utf-8"`, `allow_unicode=True`, `sort_keys=False`.
- **Force-add + commit + push** the YAML (per command Phase 6):
  `git add -f .claude/PRPs/reviews/pr-133-findings.yaml` →
  `git commit -m "chore(bm): poll-cr #133 — ingest CR findings (poll #1)"`
  → `git push origin phase-v1-AD-e`. (The advisor reads it off the phase
  branch for the triage gate.)
- Append the Phase 7 runlog entry to `.claude/runlog/bm-runlog.md`
  (`## bm: poll-cr — <ISO>` with counters + the §2a Copilot-review note +
  the §2b zero-code-merge note).
- Do **NOT** post any reply/comment on the PR (triage is separate;
  posting needs user confirmation per autonomy bounds).
- Do **NOT** send a Telegram ping (advisor handles outbound; no ping
  unless the user asks).
- Do **NOT** touch `crates/**`, `migrations/**`, `tests/**`,
  `docs/brehon-law-inspired-network/**` (BM file-ownership HARD boundary).
- If the parser finds **fewer than 6** actionable CR findings: do NOT
  silently proceed — emit a top-level `notes:` on the YAML + a runlog
  warning ("CR review header said 6 actionable; parser ingested N") so
  the advisor can investigate before triage. CR's own count is the
  ground truth to reconcile against.
