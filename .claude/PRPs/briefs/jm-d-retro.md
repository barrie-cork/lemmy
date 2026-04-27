---
role: advisor
artifact: retro
phase: v1-JM-d
created: 2026-04-27
status: pre-seeded — flesh out post-merge
---

# Brief — v1-JM-d retro (forward-seeded)

This file is pre-seeded with carry-forward items the advisor surfaced *during* execution, before the phase has shipped. At retro time (after `bm-merge`), the advisor authors the full retro per `feedback_retro_not_report.md` ("short-form always" — what surprised, what to change, what to carry forward) and incorporates the items below.

## Pre-seeded carry-forward items

### Architectural deferral — decomposer subagent (from session 2026-04-27)

The advisor authored each impl-task brief by hand mid-flight (read plan §13 → write brief → commit → queue). This is the single biggest cost on the autonomy path. Defer to JM-e or later: introduce a **decomposer subagent** (Sonnet 4.6, mechanical translation) that runs after plan approval and before bm-cut, emitting all `.claude/PRPs/briefs/<phase>-impl-{1..N}.md` + `<phase>-bm-{cut,pr,merge}.md` files in one task. Advisor's runtime job collapses from "author-and-queue" to "fetch-and-queue." Open question for the retro: planning-subagent extension vs. new fifth role; option (2) cleaner, option (1) smaller.

### Other items observed during execution (to expand at retro time)

- Phase-branch routing: `bm-cut` should `git checkout phase-<suffix>` on the daemon's main checkout (not just inside its worktree), `bm-merge` should reset to `governance-v0`. Currently undocumented in `.claude/commands/bm/bm-cut.md` Phase 6. See DQ #53 + addendum at `fe682fbc1` ancestor commits.
- Trunk-vs-phase divergence: every advisor commit on `governance-v0` (briefs, DQs, cheatsheet) diverges the phase branch. Manual rebase required. Should be a documented step in the orchestrator rule, or automated via post-commit hook.
- Settings.json drift: Claude Code worker performed a settings consistency-write at 2026-04-26 14:25 UTC (project `model: claude-opus-4-7` → user-scope `opus[1m]`). Survived as uncommitted drift across worktrees. See DQ #54 resolution; user-scope audit on EliteDesk deferred.

## Retro authoring at merge time

Per `feedback_retro_not_report.md`, the retro is distinct from the completion report. Three questions:

1. **What surprised us** — including the friction patterns above and any new ones from impl tasks 2–8 + CR cycle.
2. **What we'd change** — concrete edits to rule files, agent contracts, or skills.
3. **What we carry forward** — promote any retro-surfaced lesson to `.claude/lessons/feedback_*.md` and to PMD; cite by filename in the retro.

Per `feedback_four_role_retro_signals.md`, structure each `## What surprised us` and `## What we'd change` H2 with H3 sub-sections per role (advisor / planning / impl / BM) — otherwise the loudest role's signals dominate.
