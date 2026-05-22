# Branch-manager mechanics

Externalised from `.claude/rules/branch-manager.md` on 2026-05-22
(rule-trim pass). The rule file kept the role boundary, file-ownership
HARD list, autonomy bounds table, what-BM-refuses, session-start
ritual, and DQ-side-use. This file holds operational detail that fires
infrequently (Telegram pings, YAML schema specifics, push/CR failure
modes).

Cite this file as `bm-mechanics.md §"Telegram scope"` /
`bm-mechanics.md §"Findings YAML"` / `bm-mechanics.md §"Failure modes"`.

> **Loading note:** this file is NOT auto-loaded at session start.
> Read on-demand when (a) the BM session is about to fire a Telegram
> ping, (b) authoring a findings-YAML edit script, or (c) recovering
> from a push/PR/CR failure.

## Telegram scope (notification carrier only)

Per `feedback_telegram_scope_notification_only.md`, the BM session
fires Telegram pings ONLY for these events, and ALWAYS asks before
sending:

| Event | Trigger | Ping body shape |
|---|---|---|
| `cr-posted` | `/bm-poll-cr` finds new findings | "CR posted N findings on PR #X (Y critical, Z major)" |
| `pr-ready` | `/bm-pr` succeeds | "PR #X opened: {title} → {url}" |
| `dq-blocking` | BM writes a DQ entry it can't self-resolve | "BM filed DQ #N (blocking): {one-line}" |
| `merge-ready` | `/bm-merge` pre-checks all green | "PR #X ready to merge — awaiting confirmation" |
| `cargo-done` | Long cargo run from `/bm-prp-review` finishes | "cargo test --test e2e finished, exit {N} in {time}" |

Never:

- Diff content
- CR review responses or composed answers
- DQ answers (attribution rules in `decision-queue.md` forbid auto-edit
  from Telegram content)
- Secrets, tokens, `.env` lines, paths likely to leak deployment info
- File-tail dumps (cargo logs, error backtraces)
- Anything authored on phone — those are user-only via terminal

If the Telegram MCP server is disconnected, BM **silently skips** the
ping (logs to runlog) instead of failing the command. Pings are
notifications, not gating signals.

## Findings YAML — schema discipline

Every PR ingestion writes `.claude/PRPs/reviews/pr-<N>-findings.yaml`
in the schema documented at `.claude/PRPs/reviews/SCHEMA.md`. Hard
invariants:

- Every finding has a stable `id` (e.g. `cr-1`, `claude-1`, `user-1`)
  that does not change once written.
- `bucket` is one of: `fix-in-pr`, `rebut`, `carry-forward`, `done`,
  `wont-fix`. Never empty, never invented.
- `addressed_in` is a commit SHA when `bucket=done`, else `null`.
- `source` is one of: `coderabbit`, `claude`, `user`. Never empty.
- `severity` is one of: `critical`, `major`, `medium`, `low`, `nit`.
- `counters` block is regenerated from `findings[]` on every write.
- `last_poll_at` and `poll_count` advance every `/bm-poll-cr` run.

The findings file is the BM's only structured handoff to the impl
session. Impl reads with `yq` (e.g.
`yq '.findings[] | select(.bucket=="fix-in-pr" and .severity=="critical")'`).

## Failure modes the BM is responsible for catching

- Push fails because impl is mid-commit → retry on next `/bm-push`,
  notify user.
- `gh pr create` fails because PR already exists → fall through to
  `gh pr edit` to update body if needed.
- CR parser finds zero findings on a PR that was open >30 min → log
  warning to runlog, ask user (CR may have failed silently).
- `/prp-review` cargo step fails → write the failure into the findings
  YAML as `source: claude`, `severity: critical`, `bucket: fix-in-pr`.
- A finding's commit-SHA ref disappears (force-push removed it) →
  mark `addressed_in: null` and bucket back to `fix-in-pr`, log to
  runlog.

## See also

- `.claude/rules/branch-manager.md` — role boundary, file-ownership,
  autonomy bounds, what-BM-refuses, session-start ritual.
- `.claude/PRPs/reviews/SCHEMA.md` — full findings YAML schema.
- `feedback_telegram_scope_notification_only.md` — Telegram scope source.
- `feedback_branch_manager_pm_split.md` — the role split this
  agent operationalises.
