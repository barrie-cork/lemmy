---
name: disable-model-invocation on user-only /commands
description: Feedback — add disable-model-invocation:true to commands the user always invokes explicitly; shrinks model-visible skill listing
type: feedback
originSessionId: 54245922-a49f-4cd5-9d75-f35c34d1573c
---
Commands that are ALWAYS user-typed and should never be auto-selected by the model should carry `disable-model-invocation: true` in their YAML frontmatter. This removes them from the model's skill-listing injection while keeping them available via `/command-name` in the terminal.

**Why:** Every skill in the model-visible list costs tokens on every tool-result re-emission (the skill list injects on every turn). Removing 12 skills shaved ~250 tokens per tool result on this project (Skills bucket 1.7k → 1.5k) and reduced the auto-select surface the model has to mentally filter through. The bm/* dispatcher subagents in particular are always user-triggered (every /bm-* is a deliberate user keystroke), so auto-selecting one makes no sense.

**How to apply:** Add the flag to command frontmatter alongside `description` and `argument-hint`:

```yaml
---
description: BM — cut a new phase or plan branch off governance-v0 trunk
argument-hint: <branch-suffix>
disable-model-invocation: true
---
```

**Candidates for the flag** — user-invoked-only, no auto-select value:
- All dispatcher subagent-launching commands (Brehon's bm/* family)
- Git/PR lifecycle commands (prp-commit, prp-pr) — you know when you want to commit
- Rare manual interventions (prp-ralph-cancel)

**Do NOT flag:**
- Planning / analysis commands you'd want the model to reach for when relevant (prp-plan, prp-debug, prp-codebase-question, prp-review, etc.)
- Skills with `trigger-when` semantics in their description — those are designed for auto-select

Phase B added the flag to 9 bm/* + 3 prp-core commands on 2026-04-23 (commit `0ff419cfc`).
