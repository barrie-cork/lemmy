# Harness audit 2026-05-29 — Phase 1 inventory (RESUME ARTIFACT)

> **Status: Phase 1 complete, Phases 2–7 PENDING.** This file persists the Explore-subagent
> inventory so the post-compact session resumes the `harness-audit` skill from Phase 2 WITHOUT
> re-running the Phase 1 Explore subagent. Skill body: `.claude/skills/harness-audit/SKILL.md`.
>
> **Phase 0 env (captured pre-compact):** `claude-code | skillListingBudgetFraction=0.005
> (already-minimum, OUT of scope) | mcp=project-memory,junior-brehon,ref-context,tavily |
> pi=present (AGENTS.md exists → Pi-shared repo; `.pi/**` + Pi-shared rules OUT of scope) |
> no concurrent-session activity on gov-v0 last 60 min`.

## Ranked load table (by char count, descending)

| class | path | lines | chars | est_tokens (chars/4) |
|-------|------|-------|-------|----------------------|
| ALWAYS | .claude/rules/advisor-orchestrator.md | 375 | 47,413 | 11,853 |
| ALWAYS | .claude/rules/decision-queue.md | 492 | 30,596 | 7,649 |
| SCOPED | .claude/rules/governance-log-entry-kind-registry.md | 244 | 26,794 | 6,698 |
| ALWAYS | MEMORY.md (user-scope) | 186 | 25,375* | 6,344 |
| ALWAYS | .claude/rules/multi-lane-worktree.md | 348 | 17,731 | 4,433 |
| SCOPED | .claude/rules/pre-phase-harness-audit.md | 259 | 12,048 | 3,012 |
| ALWAYS | .claude/rules/branch-manager.md | 222 | 11,958 | 2,990 |
| ALWAYS | .claude/rules/pmd-invariants.md | 152 | 9,254 | 2,314 |
| ALWAYS | CLAUDE.md | 116 | 7,068 | 1,767 |
| SCOPED | .claude/rules/handover.md | 163 | 6,698 | 1,674 |
| SCOPED | .claude/rules/pm-plugin-hooks-stable.md | 124 | 5,874 | 1,468 |
| SCOPED | .claude/rules/view-crate-selectable-template.md | 137 | 5,373 | 1,343 |
| ALWAYS | .claude/rules/pmd-search-strategy.md | 52 | 4,243 | 1,061 |
| ALWAYS | .claude/rules/phase-branch.md | 72 | 3,934 | 984 |
| SCOPED | .claude/rules/no-cargo-output-paste.md | 76 | 3,155 | 789 |
| ALWAYS | .claude/rules/memory-injection.md | 27 | 3,096 | 774 |
| SCOPED | .claude/rules/cargo-output-capture.md | 75 | 2,830 | 708 |
| ALWAYS | .claude/rules/post-task-retro.md | 58 | 2,580 | 645 |
| SCOPED | .claude/rules/cross-repo-coordination.md | 38 | 2,124 | 531 |
| ALWAYS | .claude/rules/circuit-breaker.md | 43 | 2,056 | 514 |
| ALWAYS | .claude/rules/escalation.md | 47 | 1,977 | 494 |
| SCOPED | .claude/rules/evaluation-calibration.md | 36 | 1,793 | 448 |
| ALWAYS | .claude/rules/integrator.md | 30 | 1,703 | 426 |
| ALWAYS | .claude/rules/session-awareness.md | 23 | 1,020 | 255 |
| ALWAYS | .claude/rules/gh-pr-fork-target.md | 16 | 506 | 126 |
| ALWAYS | .claude/rules/no-destructive-defaults.md | 7 | 385 | 96 |

\* MEMORY.md char count drifted during this session's prune; subagent measured 26,067 bytes (UTF-8)
at one point = **107% of the 24,400 byte budget, OVER by ~1,667 bytes**. Re-measure at resume
(`wc -c` on the canonical path) — it was being actively edited. NOTE: the prune this session got it
to 184 lines but byte budget may still be marginally over; the rules corpus is the bigger lever.

## Summary

**ALWAYS (auto-load every session start):**
- 15 rule files = 138,452 chars / **34,613 tokens**
- + CLAUDE.md (1,767) + MEMORY.md (6,344) = **170,895 chars / ~42,724 tokens total ALWAYS-load**
- **≈ 21% of the 200K effective working window** (corrects the pre-Phase-1 ~59K/30% estimate, which
  wrongly counted all 24 rule files as always-loaded — 9 are SCOPED).

**SCOPED (auto-load only on Read of matching paths):** 9 files = 66,689 chars / 16,672 tokens. NOT
compression candidates for meta-work sessions (don't auto-load).

**Skills:** 16 SKILL.md files = 167,532 chars (skill-listing budget input; capped by
skillListingBudgetFraction=0.005).

**Largest ALWAYS file:** `advisor-orchestrator.md` (47,413 chars / 11,853 tokens) — the dominant
single compression target. `decision-queue.md` (30,596 / 7,649) is #2.

## Phase 4 OUT-OF-SCOPE (mandatory, Pi-shared / already-minimum — do NOT recommend cutting)

`.pi/**`, `AGENTS.md`, `.claude/rules/branch-manager.md`, `.claude/rules/no-cargo-output-paste.md`,
`.claude/rules/decision-queue.md` (schema — but its SIZE makes it a watch candidate; extract recipes
to refs/ only, never the schema/attribution/hard-refusals), `.claude/lessons/`, `.claude/skills/`,
`.claude/settings.json` (skillListingBudgetFraction already 0.005). Grep `.pi/` + `AGENTS.md` for any
file before recommending a cut.

## Resume instructions (post-compact)

1. Re-measure MEMORY.md `wc -c` (was mid-edit). 
2. Continue `harness-audit` Phase 2 (cross-reference grep counts for top-5 ALWAYS files) → Phase 3
   (score via `.claude/skills/harness-audit/helpers/scoring-matrix.md`) → Phase 4 (3 buckets +
   per-row Pi-impact grep) → Phase 5 (write final report `.claude/PRPs/reports/harness-audit-2026-05-29.md`)
   → Phase 6 (`memory_write_eval`) → Phase 7 (checklist).
3. Top compression hypotheses to score: `advisor-orchestrator.md` (11.9K — largest; already heavily
   refs/-externalized, check for further extract), `decision-queue.md` (7.6K — extract Recipes 1–3
   to refs/ per prior-trim precedent, keep schema/attribution inline), `multi-lane-worktree.md`
   (4.4K). Bias toward `paths:`-scoping or refs/ extraction, NOT deletion.
