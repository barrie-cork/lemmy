# Session retro — 2026-05-09 — context-injection-optimisation

**Harness:** claude-code
**Session window:** ~2026-05-09T19:00 → ~2026-05-09T21:30 UTC (~150 min)
**Branch at start:** `6bec179b9` (`governance-v0`)
**Branch at end:** `e58f77503` (`governance-v0`)
**Files touched:** 10 (6 rules, CLAUDE.md, .claude/CLAUDE.md, MEMORY.md, plan file)
**Commits:** 4 explicit (chore(advisor): path-scope rules, YAML sections, remove phase state, delete .claude/CLAUDE.md)

## TL;DR

Session audited what loads eagerly at every Claude Code session start and eliminated ~9,000 tokens of unnecessary baseline context. Six rules were path-scoped so they only load when relevant files are opened; CLAUDE.md was refactored from prose to YAML for machine-parseable key-value fields and stripped of phase-specific state that becomes stale within days; `.claude/CLAUDE.md` was deleted entirely as a duplicate of `rules/decision-queue.md`. Most load-bearing finding: CLAUDE.md was mixing **invariants** (project identity, constraints, role model) with **state** (active sub-phase SHA, phase branch name, last-shipped PR) — the invariants belong in CLAUDE.md, the state belongs in git.

---

## What surprised us

- **`@`-imports do NOT save tokens** — the Claude Code guide confirmed that `@filename` inside CLAUDE.md expands the file at load time. Many people assume it defers loading; it doesn't. Real savings only come from path-scoped rules and skills.
- **`.claude/CLAUDE.md` was purely redundant.** It loaded eagerly alongside root CLAUDE.md, and its entire content (DQ entry shape, attribution table, mid-task push snippet, commit-subject regex) was already in `rules/decision-queue.md` which also loads eagerly. Two copies of the same 60-line cheatsheet adding zero extra information per session.
- **The empirical lesson `reference_claude_code_rules_loading.md` was the most useful single source** — it documented that `paths:`-scoped rules only trigger on Read events (not Grep/Glob/Edit), which is a significant constraint: a rule whose workflow starts with Grep is silently dropped if scoped. This shaped which 6 rules were safe to scope vs which must stay unscoped.
- **User framing "machine readable" clarified the actual goal.** The initial YAML conversion was purely structural; the user's follow-up ("these will be read by Claude, not by me") sharpened the criterion: structured data where Claude needs to extract a named value, prose where Claude needs to apply conditional logic. This split cleanly once articulated.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Add `paths:` to `post-task-retro.md` rule (if it exists as a rule file) — only needed when writing retro files | ~30 tokens/session | minor | 0× prior — new candidate from this session |
| 2 | Audit remaining unscoped rules annually for stale-scope: rules written for a pattern that's now always path-scoped should be promoted | Keeps baseline lean as codebase evolves | minor | 1× this session — schedule at each lane-close |
| 3 | Add a comment to `rules/decision-queue.md` top: `# Quick reference — entry shape and attribution at top; full rules below` so it reads well as the sole DQ doc | Makes the deletion of `.claude/CLAUDE.md` transparent to future readers | minor | 1× this session |
| 4 | CLAUDE.md invariant/state discipline: when updating active sub-phase data, check CLAUDE.md and remove any phase-specific fields rather than updating them in place | Prevents CLAUDE.md re-accumulating stale state | minor | 1× this session — should become habit |

## What to carry forward

- **Invariants in CLAUDE.md, state in git.** Any field in CLAUDE.md that changes more than once per quarter is state, not an invariant. Remove it; let Claude derive it via `git log -1` or `/start-brehon`.
- **YAML for named-key lookups, prose for conditional logic.** If Claude needs to extract a specific value (path, model name, regex pattern), YAML with a meaningful key name is unambiguous. If Claude needs to apply a rule ("only when all hold: X, Y, Z"), a tight bullet list is better than YAML.
- **`paths:` scoping only safe for Read-first workflows.** Before scoping any rule, ask: "does the workflow that needs this rule always begin with a Read event?" If it starts with Grep or Edit, the rule will be silently absent. The six rules scoped this session all satisfy this (cargo work always involves opening a source file first).
- **Explore + claude-code-guide agents in parallel for research.** Running codebase exploration and official docs lookup simultaneously saved ~30 min vs serial. The subagent returned a self-contained synthesis rather than dumping raw probe output into parent context — correct pattern per the lazy-load discipline.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| Explore subagent (codebase inventory) | 25 | 0 | low | Clean parallel execution; returned compact summary, no raw dump |
| claude-code-guide subagent (@ import behaviour) | 15 | 0 | medium | Confirmed @ expands at load — most practitioners assume deferred |
| Explore subagent (lesson file reads) | 15 | 0 | none | Standard lookup; useful synthesis |
| Plan mode (plan file) | 10 | 0 | none | Good forcing function for organising the work before touching files |
| Manual CLAUDE.md edits | 0 | 5 | low | Two rounds needed (first pass prose→pointers, then prose→YAML, then YAML cleanup after user clarification on phase state) — could have been one pass with clearer upfront framing |

## Complexity scores (heavy tasks only)

No impl-tasks, no cargo runs. Skipped — session was pure meta/config work.

## Decisions to revisit

- `reference_claude_code_rules_loading.md` was empirically verified on CC v2.1.118 (2026-04-23). `paths:` scoping should be re-verified after any major CC version upgrade — the Read-only trigger behaviour in particular may change.
- The six path-scoped rules should be reviewed at v1-SL-c retro to confirm they triggered correctly (i.e. were present in context when cargo/scripts files were opened) and didn't silently drop.

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

- [ ] **Invariants vs state in CLAUDE.md**: promote to `feedback_claude_md_invariants_vs_state.md` — the distinction (invariant = doesn't change phase-to-phase; state = derivable from git) is non-obvious and was the root cause of CLAUDE.md accumulating stale fields across multiple prior phases. 1× this session; likely recurred silently in prior sessions given the stale fields found.
- [ ] **`@`-imports expand at load (no deferred loading)**: add a one-liner to `reference_claude_code_rules_loading.md` to prevent future sessions from reaching for `@` as a token-saving mechanism. 1× this session.

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
