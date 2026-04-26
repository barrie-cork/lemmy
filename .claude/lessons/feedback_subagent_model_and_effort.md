---
name: Sub-agent model + effort defaults
description: When spawning sub-agents via the Agent tool on this project, explicitly pass model="opus" and instruct the agent to run at max effort
type: feedback
originSessionId: 893350af-37c8-4d65-b9d5-420a0261dd29
---
When launching sub-agents via the Agent tool in this project, always:

- Set `model="opus"` on the Agent call (forces Opus 4.7 instead of inheriting Sonnet/Haiku)
- Include an explicit instruction in the prompt such as "Run at maximum effort — deepest reasoning, most thorough exploration" so the sub-agent's internal effort level matches the parent session.
- Prefer to run multiple agents in a single message (parallel) when the tasks are independent, rather than serialising them.

**Why:** User is running parent at effort=max with Opus 4.7. Sub-agents default to lower-capability models/effort which produces shallow results out of step with the parent's rigour. Reinforced 2026-04-16 during Phase 4a implementation.

**How to apply:** Any ad-hoc `Agent` tool invocation in this repo (code-reviewer, Explore, Plan, general-purpose, etc.) MUST set `model: "opus"` and include a max-effort directive in the prompt. Also applies to opus when the Agent tool is used for brainstorming or broad exploration.

**Exception — scripted dispatchers with per-verb tiering.** Slash-command families that dispatch to a subagent via a scripted contract may tier down by verb (Haiku/Sonnet/Opus) when the script encodes the judgment as bright-line rules/refusals and the cheaper tier is provably sufficient. Example: `.claude/commands/bm/bm-*.md` (2026-04-23, commit `af1f4260f`) — 4× haiku (mechanical), 3× sonnet (prose/gate), 2× opus (judgment: triage, prp-review). Each override carries an inline rationale in the dispatcher. This is not a reversal of the default — it's per-verb opt-out with evidence.
