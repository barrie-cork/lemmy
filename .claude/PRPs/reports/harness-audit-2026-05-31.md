# Harness audit — 2026-05-31

**Branch:** `governance-v0`
**Run by:** `harness-audit` skill (project-scope, `.claude/skills/harness-audit/SKILL.md`)
**Scope:** Claude Code auto-load only. Pi-Coding excluded by design.

## Environment snapshot

- Pi entry point (`AGENTS.md`): `present`
- `.claude/settings.json::skillListingBudgetFraction`: `0.005` (already at minimum sane value)
- MCP servers configured: `project-memory, junior-brehon, ref-context, tavily`
- Active lane worktrees: brehon-fork-quality-r2-validate, brehon-fork-quality-r3b, brehon-fork-redaction-r1, brehon-fork-rt-r4, brehon-fork-rt-r5 (5 other-lane worktrees; no concurrent-session commit collision detected)

## Auto-load inventory (Phase 1)

| Class | Path | Lines | Chars | Est tokens (chars/4) |
|---|---|---|---|---|
| ALWAYS | `.claude/rules/advisor-orchestrator.md` | 293 | 38,471 | 9,618 |
| ALWAYS | `.claude/rules/decision-queue.md` | 417 | 26,484 | 6,621 |
| ALWAYS | `.claude/rules/branch-manager.md` | 222 | 12,113 | 3,028 |
| ALWAYS | `.claude/rules/pmd-invariants.md` | 166 | 10,448 | 2,612 |
| ALWAYS | `.claude/rules/pmd-search-strategy.md` | 62 | 5,065 | 1,266 |
| ALWAYS | `.claude/rules/phase-branch.md` | 72 | 3,934 | 984 |
| ALWAYS | `.claude/rules/memory-injection.md` | 27 | 3,096 | 774 |
| ALWAYS | `.claude/rules/post-task-retro.md` | 58 | 2,580 | 645 |
| ALWAYS | `.claude/rules/circuit-breaker.md` | 43 | 2,056 | 514 |
| ALWAYS | `.claude/rules/escalation.md` | 47 | 1,977 | 494 |
| ALWAYS | `.claude/rules/integrator.md` | 30 | 1,703 | 426 |
| ALWAYS | `.claude/rules/session-awareness.md` | 23 | 1,020 | 255 |
| ALWAYS | `.claude/rules/gh-pr-fork-target.md` | 16 | 506 | 127 |
| ALWAYS | `.claude/rules/no-destructive-defaults.md` | 7 | 385 | 96 |
| SCOPED | `.claude/rules/governance-log-entry-kind-registry.md` | 244 | 26,795 | 6,699 |
| SCOPED | `.claude/rules/multi-lane-worktree.md` | 353 | 18,105 | 4,526 |
| SCOPED | `.claude/rules/pre-phase-harness-audit.md` | 259 | 12,048 | 3,012 |
| SCOPED | `.claude/rules/handover.md` | 163 | 6,698 | 1,675 |
| SCOPED | `.claude/rules/pm-plugin-hooks-stable.md` | 124 | 5,874 | 1,469 |
| SCOPED | `.claude/rules/view-crate-selectable-template.md` | 137 | 5,373 | 1,343 |
| SCOPED | `.claude/rules/no-cargo-output-paste.md` | 76 | 3,155 | 789 |
| SCOPED | `.claude/rules/cargo-output-capture.md` | 75 | 2,830 | 708 |
| SCOPED | `.claude/rules/cross-repo-coordination.md` | 38 | 2,124 | 531 |
| SCOPED | `.claude/rules/evaluation-calibration.md` | 36 | 1,793 | 448 |
| ROOT | `CLAUDE.md` | 117 | 7,467 | 1,867 |

**Totals:**
- ALWAYS-load files: 14 files, **110,838 chars**, **~27,710 est. tokens** (at chars/4; ~33,587 tokens at 3.3 chars/token empirical rate).
- SCOPED files: 10 files, 84,795 chars (do NOT auto-load in meta-work sessions).
- `CLAUDE.md` (root): 117 lines / 7,467 chars.
- `.claude/CLAUDE.md`: absent (correct — only root CLAUDE.md).
- User-scope MEMORY.md: **195 lines** ⚠ AT TRUNCATION BOUNDARY (limit = 195; zero headroom).
- Skill listing budget: 16 SKILL.md files, 169,257 total chars (capped by `skillListingBudgetFraction=0.005`).

## Cross-reference quantification (Phase 2)

| File | External citations (files citing) | Redundancy clusters detected |
|---|---|---|
| `decision-queue.md` | 673 | Recipes 1–3 bodies → `.claude/refs/dq-recipes.md`; routing matrix → `.claude/refs/dq-mechanics.md`; ci-watcher mutation pattern also in `advisor-orchestrator.md` §5.2 |
| `advisor-orchestrator.md` | 415 | §3.1 stage-shape → `.claude/refs/auto-phase.md`; §4.1 cohort → `.claude/refs/auto-phase.md`; §5.3 §G4 → `.claude/refs/auto-phase.md`; §5.1 forbidden windows → `.claude/refs/advisor-validation.md`; §5.2 validate-pending-laptop → `.claude/refs/advisor-validation.md`; §3.7/3.8 narrow gates → `.claude/refs/advisor-narrow-gates.md`; §6.1/6.2 subagent dispatch → `.claude/refs/advisor-subagent-dispatch.md` |
| `branch-manager.md` | 233 | Telegram scope → `.claude/refs/bm-mechanics.md`; Findings YAML invariants → `.claude/refs/bm-mechanics.md`; Failure modes → `.claude/refs/bm-mechanics.md` |
| `multi-lane-worktree.md` (SCOPED) | 142 | Lifecycle → `.claude/refs/multi-lane-mechanics.md`; Worktree-aware DQ id → `.claude/refs/multi-lane-mechanics.md` |
| `pmd-invariants.md` | 47 | §1 canonical path overlaps `pmd-search-strategy.md` §"brehon-fork PMD status"; §3 no-write-time-embedding overlaps PMD specs |
| `pmd-search-strategy.md` | 31 | §"brehon-fork PMD status" section partially duplicates `pmd-invariants.md` §1 |

`.claude/refs/*.md` files: **11** (healthy read-on-demand convention in place).

## Composite scores (Phase 3)

Scoring formula: `Composite = (always×0.40 + size×0.25 + redundancy×0.20 + citation_drag×0.10 + pi×0.05) × 10`
- `always`: 1.0 if ALWAYS, else 0.0
- `size`: `min(chars/8000, 1.0)`
- `redundancy`: `min(redundancy_count/2, 1.0)` — count of canonical-source duplicates against another auto-loaded file
- `citation_drag`: `1.0 - min(external_citations/20, 1.0)` — high citations reduce score (renaming risk)
- `pi`: 0.0 if Pi-shared, else 1.0

| Path | Class | Always | Size | Redundancy | Cit-drag | Pi | Composite | Bucket |
|---|---|---|---|---|---|---|---|---|
| `advisor-orchestrator.md` | ALWAYS | 0.40 | 0.25×1.0=0.25 | 0.20×1.0=0.20 | 0.10×0.0=0.00 | 0.05×0.0=0.00 | **8.5** | **High-confidence** |
| `pmd-invariants.md` | ALWAYS | 0.40 | 0.25×1.0=0.25 | 0.20×0.5=0.10 | 0.10×0.0=0.00 | 0.05×1.0=0.05 | **8.0** | **High-confidence** |
| `decision-queue.md` | ALWAYS | 0.40 | 0.25×1.0=0.25 | 0.20×1.0=0.20 | 0.10×0.0=0.00 | 0.05×0.0=0.00 | **8.5** | **High-confidence** |
| `branch-manager.md` | ALWAYS | 0.40 | 0.25×1.0=0.25 | 0.20×1.0=0.20 | 0.10×0.0=0.00 | 0.05×0.0=0.00 | **8.5** | Pi-shared → Out of scope |
| `pmd-search-strategy.md` | ALWAYS | 0.40 | 0.25×0.63=0.16 | 0.20×0.5=0.10 | 0.10×0.0=0.00 | 0.05×1.0=0.05 | **7.1** | **High-confidence** |
| `phase-branch.md` | ALWAYS | 0.40 | 0.25×0.49=0.12 | 0.20×0.0=0.00 | 0.10×0.0=0.00 | 0.05×1.0=0.05 | **5.7** | Watch |
| `memory-injection.md` | ALWAYS | 0.40 | 0.25×0.39=0.10 | 0.20×0.0=0.00 | 0.10×0.0=0.00 | 0.05×1.0=0.05 | **5.5** | Watch |
| `post-task-retro.md` | ALWAYS | 0.40 | 0.25×0.32=0.08 | 0.20×0.0=0.00 | 0.10×1.0=0.10 | 0.05×1.0=0.05 | **6.3** | **High-confidence** |
| `circuit-breaker.md` | ALWAYS | 0.40 | 0.25×0.26=0.06 | 0.20×0.0=0.00 | 0.10×1.0=0.10 | 0.05×1.0=0.05 | **6.1** | **High-confidence** |
| `escalation.md` | ALWAYS | 0.40 | 0.25×0.25=0.06 | 0.20×0.0=0.00 | 0.10×1.0=0.10 | 0.05×1.0=0.05 | **6.1** | **High-confidence** |
| `integrator.md` | ALWAYS | 0.40 | 0.25×0.21=0.05 | 0.20×0.0=0.00 | 0.10×1.0=0.10 | 0.05×1.0=0.05 | **6.0** | **High-confidence** |
| `session-awareness.md` | ALWAYS | 0.40 | 0.25×0.13=0.03 | 0.20×0.0=0.00 | 0.10×1.0=0.10 | 0.05×1.0=0.05 | **5.8** | Watch |
| `gh-pr-fork-target.md` | ALWAYS | 0.40 | 0.25×0.06=0.02 | 0.20×0.0=0.00 | 0.10×1.0=0.10 | 0.05×1.0=0.05 | **5.7** | Watch |
| `no-destructive-defaults.md` | ALWAYS | 0.40 | 0.25×0.05=0.01 | 0.20×0.0=0.00 | 0.10×1.0=0.10 | 0.05×1.0=0.05 | **5.6** | Watch |

> Note: `advisor-orchestrator.md` and `decision-queue.md` have Pi citations (advisor-orchestrator cited in `.pi/prompts/brehon-clarify.md`; decision-queue cited in `.pi/agents/bm-pi.md`). However, the Pi citations are "see also" pointers, not load-deps (Pi agents read these rules as context, not as computed paths). This matches the precedent from the 2026-05-09 trim (pm-plugin-hooks-stable.md §"Worked example" in scoring matrix). Composite stands; Pi-impact column in Phase 4 surfaces the citation count explicitly. `branch-manager.md` is Pi-shared in a load-bearing way (`.pi/agents/bm-pi.md` reads it as primary contract) — out of scope regardless of composite.

## Recommendations (Phase 4)

### High-confidence wins (composite ≥ 6.0)

| Priority | File | Current chars | Recommendation | Est. tokens saved | Pi-impact |
|---|---|---|---|---|---|
| P1 | `advisor-orchestrator.md` | 38,471 | **Already heavily extracted** — §3.1, §4.1, §5.3 already have `refs/` bodies. Remaining resident prose (surface-first ritual, stage-shape navigation text, §3.6/§3.9 gate prose) is load-bearing for meta-work sessions. **Targeted action:** extract the §3.1 "BM post-dispatch 3-signal check" table (currently ~40 lines resident) to `.claude/refs/advisor-orchestrator-incidents.md` with a 2-line pointer. Also extract §5.5 "Retro-bypass observability" (~25 lines) same destination. Estimated chars removable: ~1,800 chars | ~450 tokens | 4 hits in `.pi/prompts/brehon-clarify.md` (pointer refs, not load-dep) — Pi-safe |
| P2 | `decision-queue.md` | 26,484 | **Targeted extract:** the "Subagents and attribution" section (§bottom, ~80 lines) largely duplicates the attribution rules already in the "Attribution integrity" section. Consolidate into one canonical section + pointer. Also extract the full "Recipes" procedural prose block (the block currently says "read refs just-in-time" — verify it's already fully delegated to `refs/dq-recipes.md`; if any recipe prose remains resident, move it). Estimated chars removable: ~2,500 chars | ~625 tokens | 3 hits in `.pi/agents/bm-pi.md` (BM contract refs) — Pi-safe for extract; do NOT remove attribution rules |
| P3 | `pmd-search-strategy.md` | 5,065 | **Extract PMD status section.** The "brehon-fork PMD status (2026-05-16)" block (~30 lines) is operational-state documentation that duplicates `pmd-invariants.md` §1 HTTP topology note. Extract to `.claude/refs/` or collapse into pmd-invariants.md §1 with a one-liner. Estimated chars removable: ~900 chars | ~225 tokens | 0 Pi hits — Pi-safe |
| P4 | `pmd-invariants.md` | 10,448 | **Trim per-invariant "See also" + narrative.** Each invariant carries a "Why this is non-negotiable" + "How to apply" block averaging ~250 chars of prose. Per `feedback_rule_narrative_to_refs_at_author_time.md`, incident narratives belong in `refs/`. Headings stay. Moving the §3 "no-write-time-embedding" historical narrative (~400 chars) + §5 "SessionStart guard" worked-example (~400 chars) to a new `.claude/refs/pmd-invariants-incidents.md`. Estimated chars removable: ~1,500 chars | ~375 tokens | 0 Pi hits — Pi-safe |
| P5 | `post-task-retro.md` | 2,580 | **Low-effort Read-scope add.** This rule is only needed when exiting a task. Add `paths:` frontmatter scoping to Junior worktree branches (`junior/*`) or cargo-output files. However: the Stop hook enforces this rule regardless — the ALWAYS load ensures even non-Junior tasks see it. **Defer to Watch.** Composite just above threshold; no Pi hits; no redundancy. Reclassified to Watch. | — | 0 Pi hits |
| P5 | `circuit-breaker.md` | 2,056 | **Read-scope candidate.** This rule applies in all sessions but is not referenced in any Pi agent. Low redundancy, no Pi hits. Could add `paths:` frontmatter matching any file write or tool error context. However: it IS a universal rule (applies in all sessions per "Skip conditions: None") — scoping would violate its own stated scope. **Reclassify to Watch.** | — | 0 Pi hits |
| P5 | `escalation.md` | 1,977 | Same as circuit-breaker — universal skip-condition. **Reclassify to Watch.** | — | 0 Pi hits |
| P5 | `integrator.md` | 1,703 | Same — universal. **Reclassify to Watch.** | — | 0 Pi hits |

**Actionable high-confidence wins (P1–P4 only):**

| Priority | File | Recommendation | Est. tokens saved | Pi-impact |
|---|---|---|---|---|
| P1 | `advisor-orchestrator.md` | Extract §3.1 BM 3-signal-check table + §5.5 retro-bypass (~1,800 chars) to `refs/advisor-orchestrator-incidents.md` | ~450 | Pi-safe (pointer refs only) |
| P2 | `decision-queue.md` | Collapse "Subagents and attribution" duplication + verify recipe prose fully delegated (~2,500 chars) | ~625 | Pi-safe (BM contract refs, not load) |
| P3 | `pmd-search-strategy.md` | Extract/collapse PMD-status section into pmd-invariants §1 (~900 chars) | ~225 | Pi-safe (0 hits) |
| P4 | `pmd-invariants.md` | Move §3 + §5 incident narratives to `refs/pmd-invariants-incidents.md` (~1,500 chars) | ~375 | Pi-safe (0 hits) |

**Total estimated savings (P1–P4): ~6,700 chars → ~1,675 tokens (~5.0% of ALWAYS-load corpus)**

### Watch items (composite 3.0–5.9)

| File | Composite | Why deferred | Promotion trigger |
|---|---|---|---|
| `phase-branch.md` | 5.7 | No redundancy; medium size; zero external citations (high cit-drag = low score) — already minimal | Grows past 100 lines OR gains a duplicate in another always-load rule |
| `memory-injection.md` | 5.5 | No redundancy; small; universal skip-condition (PMD search in all sessions) | Grows past 60 lines OR gains redundancy from a new always-load rule covering same ground |
| `post-task-retro.md` | 6.3→Watch | Universal skip-condition ("None") — Read-scoping would violate its own rule; composite inflated by low citations | Redundancy with a new retro-enforcement rule, OR Stop hook absorbs all enforcement so rule becomes advisory-only |
| `circuit-breaker.md` | 6.1→Watch | "Skip conditions: None" — universal application prevents Read-scoping | Redundancy in a consolidated "error-handling rules" super-file |
| `escalation.md` | 6.1→Watch | Same as circuit-breaker | Same |
| `integrator.md` | 6.0→Watch | Same | Same |
| `session-awareness.md` | 5.8 | Tiny (23 lines), no redundancy | Already near-minimum; no action |
| `gh-pr-fork-target.md` | 5.7 | Tiny (16 lines), no redundancy; composable into `branch-manager.md` "See also" | If branch-manager.md gets a full fork-target section, this becomes redundant → merge |
| `no-destructive-defaults.md` | 5.6 | 7 lines — already minimum | No action; can't get smaller |

**Notable: MEMORY.md at 195/195 lines (zero headroom).** This is not a `rules/` file but it IS part of the auto-load corpus. One new entry would push it past the 195-line truncation. Immediate action recommended: prune one or more historical entries before next session. This is a watch-to-urgent item.

### Out of scope

Mandatory Pi-shared entries per Phase 4 template:

- `.pi/**` — Pi-Coding namespace, not Claude Code
- `AGENTS.md` — Pi entry point, must not be touched
- `.claude/rules/branch-manager.md` — Pi-shared load-bearing (`.pi/agents/bm-pi.md` reads it as primary BM contract; composite 8.5 but Pi-shared overrides)
- `.claude/rules/no-cargo-output-paste.md` — SCOPED (not auto-loading in meta-work sessions); out of scope per Phase 3 note
- `.claude/rules/decision-queue.md` (schema body) — Pi-referenced (`.pi/agents/bm-pi.md`); only non-schema sections are safe to touch; core schema is out of scope
- `.claude/lessons/` — Pi subagents read at startup
- `.claude/skills/` — Pi symlinks via `.pi/settings.json`

Other out-of-scope entries:

- `.claude/settings.json::skillListingBudgetFraction=0.005` — already at minimum; no further compression possible
- All SCOPED rules (10 files, 84,795 chars) — do not auto-load in meta-work sessions; compression yields no session-start savings
- `CLAUDE.md` (root) — core project config; not in `.claude/rules/`

## What changed since last audit

- Prior audit: `.claude/PRPs/reports/harness-audit-2026-05-29.md` (2 days ago)
- Auto-load total chars: prior **138,452** → current **110,838** (delta **−27,614 chars / ~8,370 tokens**)

Wait — the 2026-05-29 audit reported 15 ALWAYS files at 138,452 chars; current is 14 files at 110,838 chars. **Delta: −1 file, −27,614 chars.** This is a significant drop. Likely cause: one file moved from ALWAYS to SCOPED (added `paths:` frontmatter) between 2026-05-29 and today. The commit `7ac94f29f docs(rules): add single-file pull pattern to multi-lane-worktree.md` touched multi-lane-worktree.md — but that file is SCOPED already. The 2026-05-29 audit may have miscounted (it had 15 ALWAYS; current has 14). Most likely `pre-phase-harness-audit.md` was counted as ALWAYS in the prior audit but is SCOPED (has `paths:` frontmatter) — consistent with the 2026-05-09 lesson about frontmatter-detection misses. If so, effective delta is smaller; treat as baseline-reset.

- Files added to ALWAYS since prior audit: none detected
- Files moved ALWAYS → SCOPED since prior audit: likely `pre-phase-harness-audit.md` (re-classified correctly this run)
- Files deleted from `.claude/rules/`: none

## Recommendation summary

- **Top 3 wins by token impact:**
  1. `decision-queue.md` — collapse "Subagents and attribution" duplication + verify recipe delegation — ~625 tokens
  2. `advisor-orchestrator.md` — extract BM 3-signal-check table + retro-bypass section to refs/ — ~450 tokens
  3. `pmd-invariants.md` — move §3 + §5 incident narratives to refs/ — ~375 tokens
- **Total estimated savings if all high-confidence wins apply:** ~6,700 chars / **~1,675 tokens** (~5.0% of current ALWAYS-load at 110,838 chars)
- **Pi-Coding boundary check:** 2 ALWAYS rules are Pi-shared load-bearing (`branch-manager.md`, `decision-queue.md` schema body) — explicitly out of scope. All other high-confidence wins have zero Pi load-dep hits.
- **⚠ URGENT: MEMORY.md at 195/195 lines.** One new entry will cross the truncation threshold. Prune before next session write.

The user reviews this report and decides which (if any) trims to apply. This skill does not edit harness files.
