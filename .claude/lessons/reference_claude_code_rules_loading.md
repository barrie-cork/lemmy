---
name: Claude Code rules/ loading behavior
description: Empirically verified behavior of .claude/rules/ auto-loading and paths: frontmatter (Claude Code v2.1.118, 2026-04-23)
type: reference
originSessionId: 54245922-a49f-4cd5-9d75-f35c34d1573c
---
Auto-load behavior for `.claude/rules/*.md` files, measured empirically in fresh Claude Code sessions (v2.1.118, Opus 4.6) on 2026-04-23:

1. **Recursion is on.** Claude Code auto-loads every `.md` under `.claude/rules/`, INCLUDING subdirectories. An `archived/` subdirectory inside `rules/` still loads. The conventional escape is to move retired rules OUT of `.claude/rules/` entirely (e.g. `.claude/archived-rules/`). This is the opposite of what earlier Brehon advisor docs assumed — they claimed subdirectories weren't recursed.

2. **`paths:` frontmatter is include-only.** Multiple entries OR together. `!pattern` negation is silently ignored — the rule still loads. If you need to exclude a subtree, you cannot; use a narrower include list instead.

3. **`paths:`-scoped rules only load on Read events.** Grep, Glob, and Edit do NOT trigger rule load even when they hit a scoped-path match. A file-Read event is required. Implication: scoping a rule whose relevance is triggered by Grep-first or Edit-first workflows will silently drop the rule for exactly those sessions. Safe to scope only when the triggering work always begins with a Read (e.g. consulting an existing view crate as template, reading a registry file before editing it).

4. **Unscoped rules always load at session start.** They're in the Memory files bucket in /context, visible before any tool call.

5. **Subagents:** 2c not empirically tested this session; Perplexity Q5 had contested guidance. If subagent behavior matters, test in a fresh session.

Governance-v0 measured reductions from the Phase A + Phase B trim:
- Memory files: 34.4k → 15.3k (-55%)
- Two rules scoped on Read: `governance-log-entry-kind-registry.md` (governance code paths) and `view-crate-selectable-template.md` (db_views paths)
- Eight rules left unscoped to preserve guardrails in Grep/Edit-first sessions

Phase B plan: `C:\Users\barri\.claude\plans\phase-b-handover.md`. Commits: `0ff419cfc` (Phase B) + `dbc0fecad` (swap speculative env var for native `skillListingBudgetFraction: 0.005`) on governance-v0.

**Native knobs for per-turn skill listing budget** (Claude Code settings schema, v2.1.118):
- `skillListingBudgetFraction` (default 0.01): fraction of context window allocated to the per-turn skill listing. Setting 0.005 on this project shortens skill descriptions on every turn.
- `skillListingMaxDescChars` (default 1536): per-skill description cap; raise to opt-in to higher per-turn cost.

The handover's original `env.SLASH_COMMAND_TOOL_CHAR_BUDGET=3000` is NOT a documented settings key — it was a speculative env var. Use the native schema fields instead.
