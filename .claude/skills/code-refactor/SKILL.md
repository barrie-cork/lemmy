---
name: code-refactor
description: >
  Interactive code refactoring using 6 classic techniques — scan, analyse, propose, apply, verify.
  Language-agnostic: works on Python, Dart, TypeScript, PHP, Bash.
  DO use when: improving readability, reducing complexity, consolidating duplicates, clarifying intent,
  user says "refactor", "clean up this code", "simplify", "extract method", "rename", or targets a
  specific file/module for structural improvement.
  Do NOT use for: adding features (refactoring changes structure, not behaviour), dead code removal
  or audits (use code-review), visual/UI refactoring (use visual-ui-refactor), Django-specific
  structural changes on agent-grey (use the repo-local refactoring skill for Django patterns).
---

# Code Refactor

Interactive refactoring using six classic techniques. Analyses code, identifies opportunities, proposes changes with rationale, and applies them one at a time with verification after each step.

## Golden Rule

**Refactoring changes structure, never behaviour.** Every step must leave tests green. If tests do not exist for the code being refactored, flag the gap before proceeding.

## The 6 Techniques

| # | Technique | Signal | Impact |
|---|-----------|--------|--------|
| 1 | **Extract Method** | Block does one thing inside a larger function; comment explains what a block does | High — reduces function length, enables reuse |
| 2 | **Rename Variable/Method** | Name is generic (`data`, `tmp`, `process`), misleading, or abbreviation-only | Medium — improves readability at zero risk |
| 3 | **Simplify Conditional** | Nested if/else > 2 levels, repeated condition checks, boolean flag chains | High — reduces cyclomatic complexity |
| 4 | **Remove Duplicate Code** | Same logic in 2+ places (exact or near-match after normalising names) | High — single source of truth |
| 5 | **Guard Clauses** | Happy path buried inside nested conditionals; error handling scattered | Medium — linear reading flow |
| 6 | **Parameter Object** | Function takes 4+ related params that travel together across call sites | Medium — cleaner signatures, extensible |

## Phase 1: Scan

Read the target file(s) and build a structural map.

1. Read the file(s) specified by the user (or the module/directory if broadly scoped)
2. Read the repo's `CLAUDE.md` for test commands, lint commands, and project conventions
3. For each file, note:
   - Language (detect from extension: `.py`, `.dart`, `.ts`/`.tsx`/`.js`, `.php`, `.sh`)
   - Line count
   - Function/method count and average length
   - Nesting depth (max and average)
   - Import/dependency count

Output a structural summary:

```
| File | Lang | Lines | Functions | Avg length | Max depth |
|------|------|-------|-----------|------------|-----------|
```

## Phase 2: Analyse

Apply each technique as a lens to identify opportunities.

### 2a. Extract Method candidates
- Functions exceeding 30 lines (20 for Dart/TS)
- Blocks preceded by a comment explaining what they do (the comment IS the method name)
- Deeply nested blocks (depth > 3) performing a single logical operation
- Repeated inline logic that could become a helper

### 2b. Rename candidates
- Single-letter variables outside loop counters (`i`, `j`, `k` in `for` loops are fine)
- Generic names: `data`, `result`, `tmp`, `info`, `obj`, `val`, `item`, `thing`, `stuff`
- Abbreviations that are not domain-standard (`dto`, `api`, `url` are fine)
- Boolean variables not phrased as questions (`flag` vs `is_valid`, `has_permission`)
- Functions named `process`, `handle`, `do`, `run` without a domain qualifier

### 2c. Simplify Conditional candidates
- if/else chains > 3 branches (consider switch/match/when)
- Nested conditionals > 2 levels deep
- Negated conditions (`if not x and not y` — invert to positive)
- Repeated null/None/nil checks on the same variable
- Flag variables used solely to control later conditionals

### 2d. Duplicate Code candidates
- Identical or near-identical blocks (>5 lines) appearing 2+ times
- Copy-paste with only variable name differences
- Same API call pattern repeated with minor parameter variation
- Test setup code repeated across test methods

### 2e. Guard Clause candidates
- Functions where the happy path is indented 2+ levels
- Early validation scattered through the function body
- `if condition: <long block> else: return/throw` — invert to guard

### 2f. Parameter Object candidates
- Functions with 4+ parameters
- Parameter groups that appear together in multiple function signatures
- Config-like parameters (timeout, retries, verbose, dry_run) passed individually

## Phase 3: Prioritise

Rank findings using weighted scoring:

| Factor | Weight | Score range |
|--------|--------|-------------|
| Readability improvement | 0.35 | 1-5 (5 = major clarity gain) |
| Complexity reduction | 0.30 | 1-5 (5 = significant simplification) |
| Risk of behaviour change | 0.20 | 1-5 (5 = zero risk; 1 = logic restructure) |
| Blast radius | 0.15 | 1-5 (5 = single file; 1 = cross-module) |

**Weighted score = sum(factor x weight).** Present as a ranked table.

## Phase 4: Propose

Present top findings (max 10):

```
| # | Technique | Location | Description | Score | Risk |
|---|-----------|----------|-------------|-------|------|
| 1 | Extract Method | auth.py:45-78 | Extract token validation into validate_jwt_token() | 4.2 | Low |
| 2 | Guard Clauses | views.py:120-155 | Invert error checks, flatten happy path | 3.8 | Low |
| 3 | Rename | utils.py:12 | process_data() -> normalize_search_results() | 3.5 | None |
```

For each item show:
- **Before**: relevant code snippet (5-15 lines)
- **After**: what it would look like post-refactoring
- **Rationale**: one sentence explaining the structural improvement
- **Risk**: None / Low / Medium (never proceed with High risk without explicit user approval)

**STOP here and wait for user confirmation.** The user selects which items to apply (by number, "all", or "skip").

**Junior non-interactive mode:** When running as a Junior task (`claude -p`), auto-accept all None and Low risk proposals. Skip Medium risk unless the task description explicitly authorises them. Log all auto-decisions in the report.

## Phase 5: Apply

For each accepted refactoring, apply one at a time in priority order.

### 5a. Pre-flight
1. Check git state — if uncommitted changes exist, ask the user to commit first
2. Confirm test/lint commands from CLAUDE.md

### 5b. Apply single refactoring
1. Make the structural change
2. Update all references (imports, call sites, test files)
3. Preserve comments that carry domain knowledge (move them with the code)

### 5c. Language-specific notes

**Python:** Maintain `__init__.py` re-exports when extracting to new modules. Follow existing import style (absolute vs relative).

**Dart:** Maintain `part`/`part of` directives for generated files. Preserve Riverpod annotations when moving providers.

**TypeScript:** Update barrel files (`index.ts`). Preserve generic type parameters when extracting.

**Bash:** Place extracted functions before first call site. Use `local` for variables inside extracted functions. Preserve `set -euo pipefail` scope.

**PHP:** Respect WordPress coding standards (snake_case, prefix with theme/plugin name). Maintain hook registration when moving functions.

## Phase 6: Verify

After EACH individual refactoring (not after the batch):

### 6a. Run tests
Detect test command from repo CLAUDE.md. Fallback:

| Language | Default command |
|----------|-----------------|
| Python | `pytest` or `python manage.py test` |
| Dart | `flutter test` |
| TypeScript | `npm test` |
| Bash | `shellcheck <file>` + `bash -n <file>` |
| PHP | `php -l <file>` |

If no tests exist for the refactored code, note the gap and offer to write a basic smoke test before proceeding.

### 6b. Run linter
| Language | Default linter |
|----------|---------------|
| Python | `ruff check <file>` |
| Dart | `dart analyze` |
| TypeScript | `npx eslint <file>` or `npx tsc --noEmit` |
| Bash | `shellcheck <file>` |
| PHP | `php -l <file>` |

### 6c. Commit
After each verified refactoring:
```
refactor(<scope>): <technique> -- <short description>
```

### 6d. Rollback gate
If tests fail or linter raises new errors:
1. `git checkout -- .` to revert
2. Note the failure reason
3. Move to the next accepted refactoring
4. Report the failed item in the summary

## Phase 7: Report

```markdown
## Refactoring Report

**Target:** <file(s)>
**Date:** <YYYY-MM-DD>
**Techniques applied:** <count> of <proposed>

| # | Technique | Location | Description | Status | Lines changed |
|---|-----------|----------|-------------|--------|--------------|
| 1 | Extract Method | auth.py | validate_jwt_token() | Applied | +15 / -12 |
| 2 | Guard Clauses | views.py | flatten session_detail() | Applied | +8 / -14 |
| 3 | Rename | utils.py | process_data -> normalize_search_results | Failed (test) | reverted |

**Net complexity change:** <before metrics> -> <after metrics>
**Test status:** All passing / <N> failures (pre-existing)
**Gaps:** <any untested code flagged>
```

### Memory integration
If 3+ refactorings were applied to the same file/module, write a project memory:
```
memory_write(
  memory_type: "qa-result",
  title: "Refactored <file/module> -- <primary technique>",
  tags: "refactoring,<repo-name>",
  importance: 2,
  content: "<count> refactorings applied. Primary: <technique>. Net: <lines removed>. Tests: <status>."
)
```

## Completion Checklist

- [ ] Structural scan completed with metrics
- [ ] All 6 technique lenses applied
- [ ] Findings prioritised and presented
- [ ] User confirmed which items to apply
- [ ] Each refactoring applied individually with test+lint verification
- [ ] Each verified change committed separately
- [ ] Failed refactorings reverted cleanly
- [ ] Summary report produced
- [ ] Memory written (if 3+ changes applied)
