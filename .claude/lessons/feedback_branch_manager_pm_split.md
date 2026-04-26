---
name: Branch manager + PM role split for solo-dev v1
description: User prefers one CC session as branch-manager/PM (git, PRs, DQ, CR triage) and another as impl; how to handoff
type: feedback
originSessionId: 8e916c5a-be3d-4f41-baef-0a8d7736cdb8
---
User prefers a two-session model for v1 sub-phase work:
- **Impl session** — runs /prp-implement, writes code, owns task-per-commit rhythm on the phase branch
- **Branch-manager/PM session** (this role) — owns git topology, PR creation/merge, decision-queue writes and reads, CodeRabbit triage, chore(lint) follow-ups before PR

**Why:** for solo-dev on one thing at a time, the v1-AD-a "plan on one branch, phase on another" split created 3 branches + 2 PRs per sub-phase. User pushed back on complexity mid-AD-b. Two-session split preserves task granularity (important for CR review per-commit) without the solo-dev having to context-switch between writing Rust and managing git state.

**How to apply:**

1. At sub-phase start, branch-manager cuts `phase-v1-<AREA>-<letter>` off trunk and confirms the plan file is on trunk (previous plan PR merged). Tells impl session the start point.
2. During the sub-phase, branch-manager stays hands-off on code files (`crates/**`, `migrations/**`). Only writes to `.claude/decision-queue.json`, plan comments, and `chore(lint)`/`chore(pr-review)` commits the impl session explicitly defers.
3. Impl session commits per-task with `feat(scope): task N — ...`. Branch-manager DOES NOT commit on the phase branch during active impl unless the impl session hands off a specific task (e.g., "please run the chore(lint) cleanup now").
4. Branch-manager writes DQ entries when it spots a risk the impl session might miss (scope creep, branch topology concern, clippy debt). Impl session reads DQ per-iteration and either self-resolves or escalates.
5. At sub-phase close, branch-manager opens the PR (`gh pr create --repo barrie-cork/lemmy --base governance-v0 --head phase-v1-<AREA>-<letter>`), polls CodeRabbit, triages findings into fix-in-PR / rebut / carry-forward buckets per `feedback_pr_review_triage_pattern.md`, and drives the merge sequence.
6. Branch-manager NEVER pushes on behalf of impl if the impl session has staged but uncommitted edits — the race risk outweighs the coordination benefit.

**Concurrency discipline:**

- Single shared worktree. Both sessions see the same file state.
- File ownership by convention: impl owns Rust source + tests; branch-manager owns `.claude/decision-queue.json`, PR descriptions, `chore(*)` commits.
- Coordination happens via (a) user relay between sessions, (b) DQ entries, or (c) runlog-style comments in commit messages. Not via direct IPC.

**Reference:** established during v1-AD-b implementation (2026-04-20). User asked "How to simplify v1 planning + impl"; branch-manager proposed the two-session split; user adopted it. Carried through OQ-resolve PR #74, plan PR #75, phase PR #76, and DQ #41 (clippy-debt triage).
