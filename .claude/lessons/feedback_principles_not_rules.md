---
name: Frame guidance as principles, not rules
description: When adding judgment-call guidance to slash commands or rules files (especially around skill/agent delegation, scope decisions, etc.), write principles that explain the underlying reasoning — not bright-line rules that enumerate conditions.
type: feedback
originSessionId: 9690351b-28f8-4817-b247-7ccf4a42b2b2
---
**Rule:** When adding guidance to slash commands (`.claude/commands/**`), rules (`.claude/rules/**`), or skills about *judgment calls* — when to delegate to a skill/agent, when to use inline, when to escalate — write **principles** that name the underlying trade-off, not **rules** that enumerate "do X when Y, skip when Z".

**Why:** User explicitly asked for this 2026-04-25 during prp-plan / prp-implement edits to add `/cargo-validate`, `/test-write`, `/edit-mechanical` triggers. Rule-style framing ("must invoke X when Y AND not Z") drifts from the skill's own skip-conditions, encodes brittle compliance language, and removes the implementing agent's ability to reason about edge cases. Principle-style framing ("the skill exists to enforce <discipline>; use it when that discipline is the gating concern") gives the agent the reasoning to handle cases the principle-author didn't anticipate.

The skip-condition list belongs in the SKILL.md `## When to invoke` / `## Skip when` blocks (one canonical source). The prp-* command file says *why* the skill exists and *what concern it addresses* — then defers to the skill itself for the conditions.

**How to apply:**
- For skill-delegation triggers in slash commands: lead with the principle (one line — what does the skill actually buy you?). Defer specific skip-conditions to the SKILL.md by reference, not duplication.
- For agent-selection guidance: same — one line on what the agent's perspective adds; defer "skip when" to the agent's description or the calling judgment.
- Avoid bright-line rules unless the rule is genuinely binary (e.g. "PRs into main fail CodeRabbit; use governance-v0" is a rule because it's a hard constraint, not a judgment).
- Match the language register: "prefer X when…" / "X is the right shape when…" beats "must use X when…" for judgment calls.

**Difference between rules and principles in this codebase:**
- **Rules** (`.claude/rules/`): hard constraints — file ownership boundaries, security invariants, ADR enforcement, exit-code-capture mandates. These ARE binary; violation = process breach.
- **Principles** (in slash commands, plan §10/§19, retro §3): judgment-call framing — when to use a skill, when to defer, when to bundle vs split. These admit edge cases the author didn't see.

Source: 2026-04-25 prp-plan/implement skill-trigger edit; user pushed back on rule-shaped first draft of the trigger blocks. Applies to all future slash-command edits + rule additions.
