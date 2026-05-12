---
phase: v1-SL-d
role: bm-task
task: bm-poll-cr
brief_n: 1
authored: 2026-05-10
---

# [role:bm-task] v1-SL-d bm-poll-cr — ingest CodeRabbit review on PR #123

## §1 Role + dispatch

`[role:bm-task] bm-poll-cr v1-SL-d PR 123`

## §2 Scope

Run `/bm-poll-cr` for PR #123 (`phase-v1-SL-d` → `governance-v0`). CodeRabbit has posted its review. Ingest all findings into `.claude/PRPs/reviews/pr-123-findings.yaml` following the schema at `.claude/PRPs/reviews/SCHEMA.md`.

**PR:** `barrie-cork/lemmy` #123
**Head:** `phase-v1-SL-d`
**Base:** `governance-v0`

## §3 Required reading

- `.claude/rules/branch-manager.md` — findings YAML schema discipline, autonomy bounds
- `.claude/PRPs/reviews/SCHEMA.md` — mandatory findings file schema
- `.claude/commands/bm/bm-poll-cr.md` — full bm-poll-cr procedure

## §4 Constraints

- `--repo barrie-cork/lemmy` on every `gh` command.
- Write findings to `.claude/PRPs/reviews/pr-123-findings.yaml`.
- Every finding needs stable `id`, `bucket`, `source`, `severity`.
- Do NOT touch `crates/**`, `migrations/**`, `tests/**`.
- After ingesting: write runlog entry + commit + push to `governance-v0`.
