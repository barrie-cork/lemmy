# Lessons

Stable, reusable lessons promoted from PMD (the laptop's project-memory database).

These files capture **immutable** technical and process patterns — failure modes, syntax footguns, attribution rules, retro discipline, advisor watchpoints. They travel with the repo so any session (advisor on the laptop, Junior subagents on the EliteDesk, future contributors) can read them without an MCP connection.

## Read order

There is no required order. Each file is a single self-contained lesson. Glob by filename keywords matching your task:

- `feedback_clippy_*` — Rust lint discipline
- `feedback_cargo_*`, `feedback_pq_sys_*`, `feedback_features_*` — Rust build invariants
- `feedback_pre_phase_*`, `feedback_plan_*` — pre-phase gates and plan invariants
- `feedback_advisor_*`, `feedback_dq_*`, `feedback_decision_queue_*` — advisor↔impl protocol
- `feedback_retro_*` — retro discipline
- `feedback_pr_*`, `feedback_coderabbit_*` — PR / CR cycle
- `feedback_telegram_*` — notification scope (rarely applies during an impl task)
- `reference_*` — pointers to skills, commands, external systems

## What's NOT here

- `project_*.md` files — current state, in-flight session handoffs, recent activity. These mutate often and live at user-scope on the laptop, not in the repo.
- The laptop-side `MEMORY.md` index — points to local files, not portable.

## Why "in the repo"

The advisor session (laptop) and Junior subagents (EliteDesk worktrees) are on different machines. Without these files in the repo, subagents have no access to the lessons cited by rules like `pre-phase-harness-audit.md`, `decision-queue.md`, `branch-manager.md`. One-system principle: same source of truth on both sides.

When a retro promotes a new lesson to PMD, copy the file here in the same retro commit.
