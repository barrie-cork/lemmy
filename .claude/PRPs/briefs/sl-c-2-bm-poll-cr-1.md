# [role:bm-task] sl-c-2-bm-poll-cr-1 — poll CodeRabbit on PR #122

## 1. Role + dispatch

`[role:bm-task] sl-c-2-bm-poll-cr-1 — poll CodeRabbit findings on PR #122`

Run `/bm-poll-cr` per `.claude/commands/bm/bm-poll-cr.md`.

## 2. Scope

Ingest all CodeRabbit review comments on PR #122 (`phase-v1-SL-c-2 → governance-v0`).
Write findings to `.claude/PRPs/reviews/pr-122-findings.yaml` per the SCHEMA.

**Deliverable:** `.claude/PRPs/reviews/pr-122-findings.yaml` populated with all CR findings,
counters block updated, `last_poll_at` and `poll_count` set.

## 3. Required reading

- `.claude/commands/bm/bm-poll-cr.md` — full bm-poll-cr procedure
- `.claude/PRPs/reviews/SCHEMA.md` — findings YAML schema
- `.claude/rules/branch-manager.md` — file ownership + autonomy bounds

## 4. Constraints

- `--repo barrie-cork/lemmy` on every `gh` command
- PR number: **122**
- Write findings YAML only — do NOT post any PR comment
- Do NOT merge, do NOT send Telegram ping
- Append poll action to `.claude/runlog/bm-runlog.md`
