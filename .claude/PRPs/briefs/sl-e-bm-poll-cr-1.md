---
phase: v1-SL-e
role: bm-task
task: bm-poll-cr
brief_n: 6
authored: 2026-05-13
---

# [role:bm-task] SL-e bm-poll-cr — ingest CodeRabbit findings on PR #127 — see .claude/PRPs/briefs/sl-e-bm-poll-cr-1.md

## §1 Role + dispatch

`[role:bm-task] SL-e bm-poll-cr — ingest CodeRabbit findings on PR #127`

## §2 Scope

Run `bm-poll-cr` for PR #127 (phase-v1-SL-e → governance-v0).

CodeRabbit has posted its walkthrough comment + **9 inline review comments**. Ingest them into the findings YAML.

**PR:** `#127`
**Branch:** `phase-v1-SL-e`
**Repo:** `barrie-cork/lemmy`

Write `.claude/PRPs/reviews/pr-127-findings.yaml` per `.claude/PRPs/reviews/SCHEMA.md`.

## §3 Required reading

- `.claude/commands/bm/bm-poll-cr.md` — bm-poll-cr verb
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema
- `.claude/rules/branch-manager.md` — autonomy bounds
- `.claude/rules/gh-pr-fork-target.md` — `--repo barrie-cork/lemmy`

## §4 Constraints

- `--repo barrie-cork/lemmy` on all gh commands
- Read all 9 inline review comments via `gh api repos/barrie-cork/lemmy/pulls/127/comments`
- Read walkthrough comment via `gh pr view 127 --repo barrie-cork/lemmy --json comments`
- Write findings YAML with every finding's `id`, `bucket: ""` (left blank for triage), `source: "coderabbit"`, `severity` per CR's hint, `summary`, `file:line`, `recommendation`
- Update `counters` block from `findings[]`
- Set `last_poll_at` + `poll_count`
- Do NOT bucket findings (triage is a separate verb)
- Do NOT post a reply on the PR
