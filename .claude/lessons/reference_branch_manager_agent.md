---
name: Branch-manager (BM) agent — files, commands, activation
description: Standalone CC-session agent that owns git topology, PR lifecycle, CR ingestion, /prp-review, Telegram pings; activated 2026-04-23
type: reference
originSessionId: 836d315e-b9ee-46ba-9ba4-4372e79e69c8
---
Branch-manager (BM) operationalises `feedback_branch_manager_pm_split.md` as a separate Claude Code session driven by 9 `/bm-*` slash commands.

## Activation
Open a second CC session in `C:\Users\barri\Developer\brehon-fork\`. First run `/bm-status` — it reads DQ + runlog + current PR state and prints a snapshot with a suggested-next action.

## Files (all under .claude/)

```
rules/branch-manager.md                    # operating rules + autonomy table
commands/bm/{bm-cut, bm-status, bm-push, bm-pr, bm-poll-cr,
            bm-prp-review, bm-triage, bm-ping, bm-merge}.md
PRPs/reviews/SCHEMA.md                     # YAML findings-file schema v1 (TRACKED)
PRPs/reviews/pr-<N>-findings.yaml          # runtime per-PR (gitignored)
PRPs/reviews/pr-<N>-comment.md             # draft digest (gitignored)
PRPs/reviews/pr-<N>-review.md              # canonical /prp-review MD (TRACKED)
```

`.gitignore` patches: `pr-*-findings.yaml` + `pr-*-comment.md` ignored; `SCHEMA.md` + `pr-<N>-review.md` tracked.

## Autonomy table (locked by user 2026-04-23)
- **AUTO (no prompt):** branch cut, push (`phase-*`/`plan/*`), PR open, edit own PR body, `/prp-review` cargo runs, write findings YAML, write runlog
- **ASKS first:** `gh pr comment`, `gh pr review`, `gh pr merge`, Telegram pings (any of 5 events), force-with-lease, branch delete

## Findings YAML schema (v1)
Top-level: `pr, title, head, base, opened_at, last_poll_at, poll_count, last_claude_run_at, claude_run_count, recommendation, findings[], counters{}, merged_at, merge_commit, final_recommendation`.
Finding row: `id, source, severity, posted_at, file, line, summary, cr_url|adr, bucket, addressed_in, rationale, notes`.
Source ∈ {coderabbit, claude, user}. Severity ∈ {critical, major, medium, low, nit}. Bucket ∈ {fix-in-pr, rebut, carry-forward, done, wont-fix}. Stable IDs (`cr-<N>`, `claude-<N>`, `user-<N>`) — never renumber.

Other sessions read with `yq '.findings[] | select(.bucket=="fix-in-pr" and .severity=="critical")' .claude/PRPs/reviews/pr-<N>-findings.yaml` or `python yaml.safe_load(open(..., encoding="utf-8"))`.

## Hard refusals
- File ownership: BM NEVER touches `crates/**`, `migrations/**`, `tests/**`, `docs/brehon-law-inspired-network/**`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `Cargo.{toml,lock}`, `rust-toolchain.toml`.
- Critical ADR violation rebutted without superseding-ADR DQ entry → STOP.
- Cargo failure bucket-changed away from `fix-in-pr` → STOP.
- Merge with critical fix-in-pr open → STOP (per `feedback_coderabbit_block_merge_critical.md`).
- `--squash` at merge → never (per `phase-branch.md`).
- Telegram body containing diff hunks / file content >5 lines / secrets / DQ-answer text → STOP without ASK.
- Telegram event outside the 5-event allowlist (`cr-posted`, `pr-ready`, `dq-blocking`, `merge-ready`, `cargo-done`) → STOP.

## Coordination model
Single shared worktree with impl session. Channels: `.claude/decision-queue.json` (BM writes risks impl might miss; `answered_by: "bm-self-resolved"` only — never `"advisor"` or `"user"`), `.claude/runlog/<phase>-runlog.md` (BM appends `bm:`-prefixed lines), user relay (handoffs are user-typed). Session-start ritual: fetch + status + read DQ + read most-recent runlog + list open PRs.

## Why
Solo-dev v1-AD-a→b created 3 branches + 2 PRs per sub-phase, mid-AD-b user pushed back. Two-session split adopted (`feedback_branch_manager_pm_split.md`); BM agent formalises the PM half so both sessions are scripted. Built 2026-04-23 in opus 4.7 session, activated for v1-AD-d onwards.
