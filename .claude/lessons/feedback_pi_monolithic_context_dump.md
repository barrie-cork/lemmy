---
name: Pi monolithic context dump — single before_agent_start blob for all Brehon modes wastes 40-60% of context window
description: When pi's before_agent_start handler injects the same full PROJECT_CONTEXT.md, all 25 rule files, and the same mode instructions regardless of Brehon role, planning sessions get cargo-wrapper tables they'll never use, BM sessions get Rust error conventions they don't need, and every session carries ~3KB of irrelevant instruction overhead. Progressive disclosure by role eliminates this waste.
type: feedback
originSessionId: pi-harness-context-injection-2026-06-10
---

# Pi monolithic context dump — single `before_agent_start` blob wastes 40-60% of context window

## TL;DR

The `before_agent_start` handler in `.pi/extensions/lemmy-hooks.ts` injected the **same full context** (entire PROJECT_CONTEXT.md, all 25 `.claude/rules/` references, identical mode instructions) regardless of the active Brehon role. A planning session got cargo wrapper tables and Rust error conventions it would never use. A BM session got pi-specific Rust tactics and PMD topology detail it didn't need. Every Brehon role wasted at minimum 40% of its injection context on irrelevant instructions, and those instructions survived compaction into session summaries — permanently occupying context-budget slots across turns.

## The defect class

Three distinct defects in one handler:

### Defect 1 — Monolithic PROJECT_CONTEXT.md

The handler appended the **entire** PROJECT_CONTEXT.md (~3 KB, 7 sections) for every mode. But:

- **Planning mode** only needs the first 3 sections (Brehon constraints, coding workflow, dual-harness boundary) — the cargo wrapper table, subagent install instructions, and setup decisions log are irrelevant.
- **BM mode** needs Brehon constraints + subagents + setup decisions — not the cargo wrapper table or Rust error conventions.
- **Impl-task mode** needs everything *except* the RLS parity section and subagent install instructions.

A 90-line Rust quick-reference section loaded into every planning session is pure noise.

### Defect 2 — All 25 rules listed for every mode

The rule index listed every `.claude/rules/*.md` file regardless of relevance. BM only needs ~5-6 of these; planning needs ~4. The other 20+ rules are noise that the agent may still read (they're listed as "available"), wasting read tokens on irrelevant governance.

### Defect 3 — Mode-switch instructions don't change what's injected

Switching to `/brehon-mode planning` changed a label and added 3 bullet points — but the underlying context dump stayed identical. The agent saw the same monolithic PROJECT_CONTEXT.md with the same cargo wrappers it wouldn't use. The mode system was cosmetic rather than structural.

## Symptom to recognise

- After `/brehon-mode planning`, the system prompt still shows cargo wrapper tables, `unwrap()` rules, and subagent install instructions.
- After `/brehon-mode bm`, the system prompt still shows pi-specific Rust tactics and PMD topology.
- The `## Available Project Rules` section always lists exactly 25 files regardless of mode.
- Mode-switch instructions are 3 lines appended to a ~3KB blob that never changes shape.

## The fix (shipped 2026-06-10)

Progressive disclosure by role, implemented in `.pi/extensions/lemmy-hooks.ts`:

1. **MODE_CONTEXT map** — per-mode specification of which PROJECT_CONTEXT.md sections to include, which rules to list, which skill to auto-inject, and which `.claude/` paths to authorize.
2. **Section slicing** — `sliceProjectContext()` splits PROJECT_CONTEXT.md on `## ` headings and includes only the requested sections.
3. **Rule filtering** — `filteredRuleIndex()` reduces 25 rules to 4-6 per mode.
4. **Skill auto-injection** — `autoInjectSkill()` reads and injects the role's SKILL.md without requiring a manual `/skill:` step.
5. **Authorized paths** — `authorizedPathsNotice()` generates a per-mode override of AGENTS.md's blanket `.claude/` read restriction.

See `.claude/PRPs/plans/pi-harness-context-injection.plan.md` for the full design and `.claude/PRPs/reports/pi-harness-context-injection-retro.md` for execution evidence.

## Context savings (empirical)

| Mode | PROJECT_CONTEXT.md sections | Rules listed | Approximate token savings |
|------|---------------------------|-------------|--------------------------|
| planning | 7 → 3 | 25 → 4 | ~55% |
| impl-task | 7 → 4 | 25 → 5 | ~35% |
| bm | 7 → 5 | 25 → 6 | ~40% |
| ci-debug | 7 → 5 | 25 → 2 | ~50% |
| harness-maintenance | 7 → 6 | 25 → 3 | ~25% |
| main-safe | 7 (unchanged) | 25 (unchanged) | 0% (backward-compatible) |

## Generalises to

Any pi extension that injects context via `before_agent_start` where different modes/roles/contexts need different slices of the same knowledge base. The pattern (MODE_CONTEXT map + section slicing + auto-injection) is reusable for any multi-mode pi extension.
