---
id: setup-relay-protocol
from: advisor
to: impl
ts: 2026-04-24T00:10Z
relates_to: runlog/impl-relays
---

# Decision
Adopt file-based relay protocol for impl → advisor direction. Replaces ad-hoc console paste.

# Instructions
1. For every status report, question, or DQ draft you'd otherwise paste to the user: write it to `.claude/runlog/impl-relays/<id>.md` on this worktree (`brehon-fork-phase-v1-JM-a`) instead.
2. Follow the schema at `.claude/runlog/impl-relays/_README.md`. Required: YAML frontmatter (`id`, `from: impl`, `to: advisor`, `ts`, `relates_to`, `blocking`) + fixed-vocabulary sections (`# Context`, `# Ask`, `# Evidence`, `# Proposed`; omit any that don't apply).
3. Filename = `id` + `.md`. ID scopes: `task<N>-status`, `R<N.M>-ask`, `DQ<N>-draft`, `blocker-<slug>`, `adhoc-<slug>`.
4. After writing, tell the user one short line: `relay impl-relays/<id>.md`. Nothing else — no body paste, no summary. The file is the source of truth.
5. If not blocked on the ask, continue with other work (e.g. next task if independent). If fully blocked, stop and wait per `.claude/rules/decision-queue.md` §"After writing a question".
6. Advisor writes answers to `advisor-relays/<id>-answer.md` on the **primary** worktree (`brehon-fork`). Do not expect answer files to appear on this worktree. User relays the answer back to you.
7. Commit decision: treat relay files like the task-resume brief — your call whether to commit on the phase branch or leave untracked. Untracked is fine for working-tree scaffolding; committing gives durability if the worktree is torn down.

# What NOT to write in a relay

- Cargo output tails > 10 lines (use a file path in `# Evidence` instead)
- Speculation about advisor's likely answer (advisor decides from evidence)
- Decisions you can self-resolve from plan/rules/PRD (don't file unnecessary relays)
- Meta-commentary about the relay protocol itself

# Next
Proceed with Task 10 as you already are. Next time you'd normally paste a question to the user, file it as `impl-relays/<id>.md` and tell the user to relay it. No other behavior change.
