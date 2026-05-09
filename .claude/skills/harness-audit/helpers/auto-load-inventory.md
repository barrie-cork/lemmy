# Auto-load inventory — reference

Helper for `harness-audit` Phase 1. Captures **what Claude Code auto-loads at session start** in this repo, and how the audit detects each class. Read just-in-time when Phase 1 needs a reminder of the load model.

Source of truth for empirical behaviour: `.claude/lessons/reference_claude_code_rules_loading.md` (last verified Claude Code v2.1.118, Opus 4.6, 2026-04-23). Re-verify when Claude Code's minor version changes.

## Auto-load classes

| Class | Triggered when | How to detect |
|---|---|---|
| **Project CLAUDE.md** | Every session start | File `CLAUDE.md` at repo root |
| **Nested CLAUDE.md** | Every session start | File `.claude/CLAUDE.md` |
| **Always-load rules** | Every session start | `.claude/rules/*.md` whose line 1 is NOT `---` |
| **Read-scoped rules** | Only when session Reads a file matching `paths:` glob | `.claude/rules/*.md` with `---` frontmatter containing `paths:` block |
| **User-scope MEMORY.md** | Every session start (if file exists) | `~/.claude/projects/<project-slug>/memory/MEMORY.md` |
| **Lessons via PMD** | On-demand via `memory_search_hybrid` | `.claude/lessons/*.md` are NOT auto-loaded; indexed in PMD only |
| **Skills (frontmatter description only)** | Per-turn skill listing | `.claude/skills/*/SKILL.md` frontmatter `description:` field counts toward `skillListingBudgetFraction` |
| **MCP server descriptions** | Every session start | Loaded from configured `.mcp.json` |
| **Tool schemas** | Lazy-loaded via `ToolSearch` after disconnect or on-demand | Not auto-loaded for deferred tools |

## Frontmatter scoping (load-bearing for Phase 1)

A `.claude/rules/*.md` file is **SCOPED** if:

1. Line 1 is `---`.
2. Lines 2–N contain a `paths:` YAML block listing glob patterns.
3. Line N+1 is `---` (frontmatter terminator).

Example (verified scoped):

```yaml
---
paths:
  - "crates/db_schema/src/source/governance/governance_log.rs"
  - "crates/api/api/src/governance/**/*.rs"
---
```

Per `reference_claude_code_rules_loading.md`:

- **Recursion is on.** Subdirectories under `.claude/rules/` also auto-load. The escape is to move retired rules OUT of `.claude/rules/` (e.g. `.claude/archived-rules/`).
- **`paths:` is include-only.** `!pattern` negation is silently ignored.
- **Read-trigger only.** Grep, Glob, Edit do NOT trigger SCOPED rule load. Safe to scope only when the triggering work always begins with a Read.
- **Unscoped = always.** No `paths:` frontmatter means the rule is in the Memory bucket from session start.

## Current ALWAYS-load files in brehon-fork (snapshot 2026-05-09 post-trim)

For audit context. Phase 1's subagent re-derives this fresh — these numbers are not authoritative for the next audit.

```
ALWAYS  decision-queue.md             530 lines (post recipe-extract)
ALWAYS  advisor-orchestrator.md       378 lines (post §5.2 collapse)
ALWAYS  auto-phase.md                 376
ALWAYS  branch-manager.md             264
ALWAYS  pre-phase-harness-audit.md    252
ALWAYS  handover.md                   158
ALWAYS  pm-plugin-hooks-stable.md       0  (now SCOPED — paths:frontmatter added 2026-05-09)
ALWAYS  phase-branch.md                72
ALWAYS  no-cargo-output-paste.md       69
ALWAYS  cargo-output-capture.md        68
ALWAYS  post-task-retro.md             58
ALWAYS  pmd-search-strategy.md         52
ALWAYS  escalation.md                  47
ALWAYS  circuit-breaker.md             43
ALWAYS  cross-repo-coordination.md     32
ALWAYS  integrator.md                  30
ALWAYS  evaluation-calibration.md      30
ALWAYS  memory-injection.md            27
ALWAYS  session-awareness.md           23
ALWAYS  gh-pr-fork-target.md           16
ALWAYS  no-destructive-defaults.md      7

SCOPED  governance-log-entry-kind-registry.md  208
SCOPED  view-crate-selectable-template.md      137
SCOPED  pm-plugin-hooks-stable.md              116
```

Phase 1's subagent confirms these classifications by reading the first 10 lines of each file. Any file whose line 1 is `---` is SCOPED regardless of historical classification — the rules file is the source of truth, not this snapshot.

## What is NOT auto-loaded (out of audit scope)

- `.claude/lessons/*.md` — indexed in PMD; loaded only via `memory_search*` tools.
- `.claude/PRPs/**` — briefs, plans, reports; loaded only via Read.
- `.claude/refs/*.md` — read-on-demand convention (the `helpers/dq-recipes.md` extract precedent).
- `.claude/skills/*/SKILL.md` body — only the frontmatter description loads at session start. Body loads on `/skill-name` invocation.
- `.claude/commands/*.md` body — same as skills: argument-parse + body load on `/command-name` invocation.
- `.claude/decision-queue.json` — UserPromptSubmit hook injects pending count; full body loads via Read on demand.
