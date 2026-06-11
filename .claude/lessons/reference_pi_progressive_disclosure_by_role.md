---
name: Pi progressive disclosure by role — mode-aware context slicing with auto-injected skills and filtered rule indexes
description: The solution pattern for the monolithic context dump defect: a MODE_CONTEXT map that specifies per-role PROJECT_CONTEXT.md sections, rule filters, skill paths, and authorized read paths. The before_agent_start handler uses this map to build role-specific context, auto-injects the matching skill file, and appends an authorized-paths notice overriding AGENTS.md's blanket .claude/ restriction. This pattern converts /brehon-mode from a cosmetic label change into a structural context reshape.
type: reference
---

# Pi progressive disclosure by role — mode-aware context slicing

## TL;DR

A `MODE_CONTEXT` map in `.pi/extensions/lemmy-hooks.ts` that specifies, for each Brehon mode, exactly which sections of PROJECT_CONTEXT.md to include, which `.claude/rules/` files to list, which `.pi/skills/` file to auto-inject, and which `.claude/` paths to authorize for reads. The `before_agent_start` handler reads this map and builds a mode-specific system prompt. A mode switch (`/brehon-mode planning` → `/brehon-mode impl-task`) now reshapes the entire injected context, not just a label.

## The MODE_CONTEXT pattern

### Data structure

```typescript
interface ModeContext {
  persona: string;                    // role-specific persona line (or "" for none)
  projectContextHeadings: string[];   // which ## headings from PROJECT_CONTEXT.md to include ([] = all)
  ruleFilter: string[];               // which rule filenames to show ([] = all)
  recommendedSkill: string | null;    // path to .pi/skills/<name>/SKILL.md, or null for none
  authorizedReadPaths: string[];      // .claude/ paths to authorize for this mode ([] = no override)
}
```

### Per-mode entries

Seven modes map to different slices. The key insight: each mode gets **only what it needs**, not the kitchen sink.

```typescript
const MODE_CONTEXT: Record<BrehonMode, ModeContext> = {
  planning: {
    persona: "You are the Planning agent for Brehon...",
    projectContextHeadings: [constraints, workflow, harness],  // 3 of 7 sections
    ruleFilter: ["handover.md", "pmd-invariants.md", "pmd-search-strategy.md", "session-awareness.md"],
    recommendedSkill: ".pi/skills/planning/SKILL.md",
    authorizedReadPaths: [".claude/commands/prp-core/prp-plan.md", ".claude/lessons/", ...],
  },
  "impl-task": {
    projectContextHeadings: [constraints, workflow, harness, rust],  // 4 of 7 — adds cargo wrappers
    ruleFilter: ["phase-branch.md", "no-cargo-output-paste.md", "cargo-output-capture.md", ...],
    recommendedSkill: ".pi/skills/impl-task/SKILL.md",
    authorizedReadPaths: [".claude/commands/prp-core/prp-implement.md", ".claude/lessons/", ...],
  },
  // ... 5 more modes
};
```

### Helper functions

Four pure functions consume the MODE_CONTEXT map:

1. **`sliceProjectContext(headings: string[])`** — splits PROJECT_CONTEXT.md on `\n(?=## )`, includes only sections whose heading text matches the requested list, always keeps the preamble (H1 + intro paragraph).

2. **`filteredRuleIndex(mode: BrehonMode)`** — if `ruleFilter` is empty, returns all rules (main-safe behavior). Otherwise, returns only rules whose filenames match a filter item.

3. **`autoInjectSkill(mode: BrehonMode)`** — reads the `recommendedSkill` path, strips YAML frontmatter (`---...---`), and returns a formatted "Active Skill" section. Falls back gracefully (file missing → null → no injection, no error).

4. **`authorizedPathsNotice(mode: BrehonMode)`** — generates a per-mode "Authorized .claude/ paths" notice with explicit override language: "(Override of AGENTS.md's blanket .claude/ read restriction. These paths are authorised because the current Brehon mode requires them.)"

### before_agent_start integration

The revised handler builds additions in order:

```
1. Sliced PROJECT_CONTEXT.md (via sliceProjectContext)
2. Mode instructions + persona + authorized paths notice
3. Pre-phase audit reminder (if applicable)
4. Coordination state (DQ/hopper summary)
5. Auto-injected skill (via autoInjectSkill)  ← NEW
6. Filtered rule index (via filteredRuleIndex) ← NEW
```

The order matters: skill content and rules appear after the mode instructions so the agent reads "I am in planning mode" before seeing the planning skill's instructions.

## When to apply

Use this pattern when:

- A pi extension has multiple modes/roles/contexts with different knowledge needs.
- The same extension injects content that's only relevant to a subset of modes.
- You find yourself adding `if (mode === "X")` conditionals inside `before_agent_start` — that's the signal to extract a MODE_CONTEXT map.
- A mode switch (`/brehon-mode X`) changes only a label but not the injected context.

## Extending with a new mode

Adding a new Brehon mode takes exactly three changes:

1. Add an entry to `BREHON_MODES` (label, description, instructions).
2. Add an entry to `MODE_CONTEXT` (headings, rules, skill, paths).
3. Optionally, add a skill file at `.pi/skills/<mode>/SKILL.md` and set `recommendedSkill`.

The `before_agent_start` handler, `pathPolicyDecision`, and all other extension logic remain unchanged — they consume the maps generically.

## Design decisions (do not re-litigate)

| Decision | Rationale |
|---|---|
| Empty `projectContextHeadings[]` = all sections | Backward-compatible: main-safe keeps full PROJECT_CONTEXT.md unchanged. |
| Empty `ruleFilter[]` = all rules | Same backward-compat principle. |
| `recommendedSkill: null` = no injection | Modes without a skill (review-readonly, ci-debug, harness-maintenance) don't get auto-injected content. |
| Skill frontmatter is stripped before injection | YAML frontmatter confuses LLMs; the body is the actionable content. |
| Graceful fallback on missing skill file | `fs.existsSync()` check; null returned; no error. A missing skill never breaks session start. |
| `persona: ""` = no persona line | Modes that don't need a role persona (main-safe) don't get one injected. |

## Ship evidence

- Plan: `.claude/PRPs/plans/pi-harness-context-injection.plan.md`
- Retro: `.claude/PRPs/reports/pi-harness-context-injection-retro.md`
- Implementation: `.pi/extensions/lemmy-hooks.ts` (MODE_CONTEXT + 4 helper functions + revised `before_agent_start`)
- Rewritten skills: `.pi/skills/{planning,impl-task,bm-task}/SKILL.md`
- Session retro: `.claude/PRPs/reports/session-retro-2026-06-10-pi-harness-context-injection.md`
