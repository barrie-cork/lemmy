---
name: code-audit
description: >
  Static analysis + Claude deep review — produces a ranked audit report with refactoring opportunities.
  Language-agnostic: works on Python, Dart, TypeScript, PHP, Bash. Uses Serena MCP when available
  for precise symbol analysis and dead-code detection.
  DO use when: user says "audit", "code health", "tech debt scan", "what needs refactoring",
  before a planned refactoring sprint, or as a periodic Junior task for repo health tracking.
  Do NOT use for: actually applying refactorings (use code-refactor), security audits (use security
  repo agents), runtime/performance profiling, or dead code removal only (use repo-local code-review).
---

# Code Audit

Hybrid code quality analysis: cheap static tools first, then Claude deep-analyses the top flagged files. Produces a ranked report with ready-to-queue Junior task descriptions for `code-refactor`.

**This skill produces a report only — it does NOT modify code.**

## Phase 0: Tool Detection

**Do this FIRST, before any other phase.**

1. **Check for Serena MCP.** Read `.mcp.json` in the repo root. If it contains a `serena` entry, Serena is available — you MUST use it in Phase 2 Tier 2. Log: `"Serena MCP: available"` or `"Serena MCP: not configured"`.
2. **Check for ruff.** Run `ruff --version 2>/dev/null || pipx run ruff --version 2>/dev/null`. If bare `ruff` fails, use `pipx run ruff` for all subsequent ruff commands. If both fail, skip ruff and note in the report. Log which variant works.

## Phase 1: Scope

1. Determine target from user or task description:
   - Specific directory: `apps/search_strategy/`
   - Specific module: `lib/features/ingredients/`
   - Full repo (default if unspecified)
2. Read the repo's `CLAUDE.md` for conventions, language, test/lint commands
3. Detect primary language(s) from file extensions
4. Build file list, excluding:
   - Tests: `test/`, `tests/`, `*_test.dart`, `*_test.py`, `test_*.py`
   - Generated: `*.g.dart`, `*.freezed.dart`, `*.gen.ts`
   - Dependencies: `node_modules/`, `vendor/`, `.dart_tool/`
   - Build output: `build/`, `dist/`, `__pycache__/`
   - Tooling: `.junior/`, `.claude/`, `.serena/`, migrations
5. Count files per language

Output:
```
Scope: <target>
Files: <N> (<breakdown by language>)
```

## Phase 2: Cheap Metrics

Use the best available tool at each tier. Print `"Scanning file N/M..."` to stdout periodically to keep Junior's inactivity watchdog alive.

### Tier 1 — Shell commands (all files)

Run `wc -l` for line counts, then the appropriate linter. Use the ruff variant detected in Phase 0.

| Language | Command | Fallback | What it gives |
|----------|---------|----------|---------------|
| Python | `ruff check --output-format json .` | `pipx run ruff check --output-format json .` | Per-file violation counts + rule codes |
| Dart | `/snap/bin/dart analyze --format machine` | `dart analyze --format machine` | Errors, warnings, infos per file |
| Bash | `shellcheck --format json <files>` | | Per-file warning counts + codes |
| TypeScript | `npx tsc --noEmit 2>&1` | | Type errors per file |
| PHP | `find . -name "*.php" -exec php -l {} \;` | | Syntax errors |
| Rust | `cargo clippy --workspace -- -D warnings 2>&1` | | Compiler errors + lint violations; bucket `critical` on compile error, `major` on clippy deny |

> **Rust const-block caveat:** `longest-fn` metrics over-report for `const` blocks and macro expansions. Flag the finding as `medium` (not `critical`) when the longest function body is entirely inside a `const` block or macro.
> **Rust deferral guidance:** if `cargo clippy` fails to compile (dependency issue, missing feature flag), record as `validate: blocked` rather than a finding — do not attempt to fix compilation as part of the audit.

Collect: `{file, language, lines, linter_violations}` for every file.

### Tier 2 — Serena (MANDATORY when Phase 0 detected it)

If Phase 0 confirmed Serena is available, you MUST run these Serena tools before proceeding to Tier 3. This tier gives far more accurate structural data than manual file reading.

1. **`get_symbols_overview`** on each file in scope — returns function/class names with line ranges
   - Function count = number of function/method symbols
   - Longest function = max(end_line - start_line) across symbols
   - Class count = number of class symbols
2. **`find_referencing_symbols`** on exported functions/methods — zero references = dead code candidate
   - Flag any public function with 0 callers (excluding test files)
3. **`find_symbol`** for generic names (`process`, `handle`, `data`, `result`, `tmp`) — rename candidates

Serena is configured on: agent-grey, dog-shelter, my-food-system. Skip this tier ONLY if Phase 0 confirmed Serena is not in `.mcp.json`.

### Tier 3 — Claude reads files (fallback when no Serena)

For the top ~30 files by line count + linter violations, compute **exact** metrics (not approximations):

- **Function count:** exact number of `def`/`function`/method signatures. Use grep or Read to count precisely.
- **Longest function:** exact line count (end_line - start_line + 1). Identify boundaries by indent or brace matching.
- **Max nesting depth:** exact count of nested `if`/`for`/`while`/`try` at the deepest point.

**Do NOT use `~` approximations in the report.** If a count is uncertain, read the file again. The whole point of the cheap metrics pass is to give the ranking step concrete numbers.

Used on: food-producer, midleton-market, security, web-archive.

## Phase 3: Rank and Filter

Compute a composite score per file (0-10 scale):

| Factor | Weight | Calculation |
|--------|--------|-------------|
| Size | 0.20 | lines / 500, capped at 1.0 |
| Complexity | 0.25 | longest_function / 100, capped at 1.0 |
| Nesting | 0.20 | max_depth / 6, capped at 1.0 |
| Linter violations | 0.25 | violation_count / 20, capped at 1.0 |
| Function count | 0.10 | function_count / 30, capped at 1.0 |

**Score = sum(factor x weight) x 10**

Adjustments:
- Test files: multiply score by 0.7 (deprioritised, not excluded)
- Files with 5+ linter violations: auto-include regardless of composite score
- Files with dead code (Serena Tier 2): auto-include

Take the top 10-15 files (or all scoring above 5.0, whichever is fewer).

## Phase 4: Deep Analysis

For each file in the top list, read it fully and apply the 6 technique lenses from `code-refactor` Phase 2:

### 4a. Extract Method
- Functions exceeding 30 lines (20 for Dart/TS)
- Blocks preceded by explanatory comments
- Depth > 3 performing a single operation

### 4b. Rename
- Generic names: `data`, `result`, `tmp`, `info`, `obj`, `val`
- Booleans not phrased as questions (`flag` vs `is_valid`)
- Functions named `process`, `handle`, `do` without domain qualifier

### 4c. Simplify Conditional
- if/else > 3 branches
- Nesting > 2 levels
- Negated conditions, flag variables

### 4d. Remove Duplicates
- Near-identical blocks (>5 lines) in 2+ files
- Same API call pattern with minor variation

### 4e. Guard Clauses
- Happy path indented 2+ levels
- `if condition: <long block> else: return/throw`

### 4f. Parameter Object
- Functions with 4+ related parameters
- Same parameter group in multiple signatures

Score each finding with the weighted matrix:

| Factor | Weight | Score range |
|--------|--------|-------------|
| Readability improvement | 0.35 | 1-5 |
| Complexity reduction | 0.30 | 1-5 |
| Risk of behaviour change | 0.20 | 1-5 (5 = zero risk) |
| Blast radius | 0.15 | 1-5 (5 = single file) |

Assign risk tier: **None** / **Low** / **Medium**.

## Phase 5: Report

Write to `reports/code-audit-YYYY-MM-DD.md`:

```markdown
# Code Audit Report — <repo>

**Date:** YYYY-MM-DD
**Scope:** <directory or "full repo">
**Files scanned:** <N> (<breakdown by language>)
**Serena:** available / not configured
**Overall health:** Good / Fair / Needs Attention

## Cheap Metrics Summary

| # | File | Lang | Lines | Functions | Longest fn | Max depth | Linter issues | Dead code | Composite |
|---|------|------|-------|-----------|------------|-----------|---------------|-----------|-----------|

## Deep Analysis — Top N Files

### 1. <file> (composite: X.X)

| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|

**Linter highlights:** <top 3 violations>
**Cross-file duplicates:** <if any>

### 2. <file> (composite: X.X)
...

## Refactoring Task Queue

| Priority | Task description (<100 chars) | Target | Technique | Risk |
|----------|-------------------------------|--------|-----------|------|

## Summary

- **Total findings:** <N>
- **By risk:** None: <n>, Low: <n>, Medium: <n>
- **By technique:** Extract Method: <n>, Rename: <n>, Simplify: <n>, Duplicates: <n>, Guard: <n>, Param Object: <n>
- **Dead code flagged (Serena):** <n functions with 0 callers>
- **Systemic patterns:** <patterns spanning 3+ files>
```

Commit the report: `git add reports/ && git commit -m "audit(<scope>): code quality report YYYY-MM-DD"`

## Phase 6: Task Generation

Generate Junior task descriptions for each finding, ready to queue. Each row in the Task Queue table MUST include the full task description (not just a short label) so the user can copy-paste directly into `junior-add-task`.

**Requirements for every task description:**

1. **Under 100 chars** (per TH task queuing best practices — becomes the branch name)
2. **MUST reference the skill path verbatim:** `Follow .claude/skills/code-refactor/SKILL.md`
3. **MUST name the target file and technique concisely**

**Format template:**
```
Refactor <file>: <technique>. Follow .claude/skills/code-refactor/SKILL.md
```

**Examples:**
- `Refactor models.py: extract format_term to module-level. Follow .claude/skills/code-refactor/SKILL.md` (89 chars)
- `Refactor views.py: split _handle_returnable_state_transition. Follow .claude/skills/code-refactor/SKILL.md` (104 chars — too long, shorten)
- `Refactor views.py: split _handle_returnable_state. Follow .claude/skills/code-refactor/SKILL.md` (94 chars)

If the standard format exceeds 100 chars, abbreviate the technique description, NOT the skill path. The skill path is mandatory.

**Task Queue table must use this exact column layout** so descriptions can be copy-pasted:

```
| Priority | Task description | Target | Technique | Risk |
|----------|-----------------|--------|-----------|------|
| P1 | Refactor models.py: extract format_term to module-level. Follow .claude/skills/code-refactor/SKILL.md | models.py | Extract Method | Low |
```

Group rows by risk tier (None/Low first, Medium last).

**This skill does NOT auto-queue tasks.** The user reviews the report and selects which to proceed with.

## Memory Integration

After producing the report:

1. **Search first:** `memory_search_hybrid(query: "code audit findings dead code", tags: "<repo-name>")` for previous audit results (hybrid handles concept drift across audit scopes)
2. **If 5+ findings:** write a `qa-result` memory:
   ```
   memory_write(
     memory_type: "qa-result",
     title: "Code audit: <repo> <scope> — <N> findings",
     tags: "code-quality,<repo-name>",
     importance: 2,
     content: "<N> findings across <M> files. Top: <technique> (<count>). Health: <Good/Fair/Needs Attention>. Dead code: <N>."
   )
   ```
3. **If systemic pattern spans 3+ files:** write a `pattern` memory:
   ```
   memory_write(
     memory_type: "pattern",
     title: "<pattern description>",
     tags: "code-quality,<repo-name>",
     importance: 3,
     content: "Found in <N> files: <file list>. Pattern: <description>. Suggested fix: <technique>."
   )
   ```
4. **If previous audit exists:** note improvement or regression in the report summary

## Completion Checklist

- [ ] Scope determined and file list built
- [ ] Shell linters run (Tier 1)
- [ ] Serena analysis run (Tier 2, if available)
- [ ] Claude structural analysis run (Tier 3, if needed)
- [ ] Files ranked by composite score
- [ ] Top files deep-analysed with 6 technique lenses
- [ ] Report written to `reports/code-audit-YYYY-MM-DD.md`
- [ ] Task descriptions generated
- [ ] Report committed
- [ ] Memory written (if thresholds met)
