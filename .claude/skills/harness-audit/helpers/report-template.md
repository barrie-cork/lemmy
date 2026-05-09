# Harness audit — `<UTC-date>`

**Branch:** `<current-branch>`
**Run by:** `harness-audit` skill (project-scope, `.claude/skills/harness-audit/SKILL.md`)
**Scope:** Claude Code auto-load only. Pi-Coding excluded by design.

## Environment snapshot

- Pi entry point (`AGENTS.md`): `<present|absent>`
- `.claude/settings.json::skillListingBudgetFraction`: `<value>` (minimum sane value: 0.005)
- MCP servers configured: `<server1, server2, …>`

## Auto-load inventory (Phase 1)

| Class | Path | Lines | Chars | Est tokens (chars/4) |
|---|---|---|---|---|
| `<ALWAYS\|SCOPED>` | `<path>` | `<N>` | `<N>` | `<N>` |
| … | … | … | … | … |

**Totals:**
- ALWAYS-load files: `<count>` files, `<total chars>` chars, `<total tokens>` est. tokens.
- SCOPED files: `<count>` files, `<total chars>` chars (do NOT auto-load in meta-work sessions).
- CLAUDE.md (root): `<lines>` / `<chars>`.
- `.claude/CLAUDE.md`: `<lines>` / `<chars>`.
- User-scope MEMORY.md: `<lines>` (truncation safe: `<lines ≤ 195>`).
- Skill listing budget: `<count>` SKILL.md files; total frontmatter description chars `<N>`.

## Cross-reference quantification (Phase 2)

| File | External citations | Redundancy clusters detected |
|---|---|---|
| `<path>` | `<N>` | `<list of canonical-source duplicates>` |
| … | … | … |

`.claude/refs/*.md` files: `<count>` (project's read-on-demand convention).

## Composite scores (Phase 3)

| Path | Class | Size | Redundancy | Citations | Pi | Composite | Bucket |
|---|---|---|---|---|---|---|---|
| `<path>` | `<class>` | `<0–1>` | `<0–1>` | `<0–1>` | `<safe\|shared>` | `<0–10>` | `<bucket>` |
| … | … | … | … | … | … | … | … |

## Recommendations (Phase 4)

### High-confidence wins (composite ≥ 6.0)

| Priority | File | Recommendation | Est. tokens saved | Pi-impact |
|---|---|---|---|---|
| `P1` | `<path>` | `<add paths: frontmatter \| extract to refs/ \| fold pattern>` | `~<N>` | `<grep hits in .pi/ + AGENTS.md>` |
| … | … | … | … | … |

### Watch items (composite 3.0–5.9)

| File | Why deferred | Promotion trigger |
|---|---|---|
| `<path>` | `<small / no-redundancy / cited heavily>` | `<file grows >X lines \| gains redundancy \| etc>` |
| … | … | … |

### Out of scope

Mandatory entries (Pi-shared invariants):

- `.pi/**`
- `AGENTS.md`
- `.claude/rules/branch-manager.md` — Pi subagents READ this
- `.claude/rules/no-cargo-output-paste.md` — `.pi/extensions/lemmy-hooks.ts` references by literal path
- `.claude/rules/decision-queue.md` (schema body) — Junior subagents use it as canonical contract; only non-schema sections may be touched
- `.claude/lessons/` — Pi subagents read at startup
- `.claude/skills/` — Pi symlinks via `.pi/settings.json`

Other entries:

- `.claude/settings.json::skillListingBudgetFraction` if already at `0.005` — no further compression possible without changing Claude Code minor-version behaviour.
- `<other already-minimum or ADR-protected items>`

## What changed since last audit

- Prior audit: `<.claude/PRPs/reports/harness-audit-<date>.md>` (or "no prior audit found")
- Auto-load total chars: prior `<N>` → current `<N>` (delta `<+/- N>`).
- Files added to ALWAYS: `<list>`.
- Files moved ALWAYS → SCOPED: `<list>`.
- Files deleted from `.claude/rules/`: `<list>`.

## Recommendation summary

- **Top 3 wins by token impact:**
  1. `<path>` — `<recommendation>` — `~<N>` tokens
  2. `<path>` — `<recommendation>` — `~<N>` tokens
  3. `<path>` — `<recommendation>` — `~<N>` tokens
- **Total estimated savings if all high-confidence wins apply:** `~<N>` tokens (`<N>%` of current auto-load).
- **Pi-Coding boundary check:** `<all clear | N entries surface for review>`.

The user reviews this report and decides which (if any) trims to apply. This skill does not edit harness files.
