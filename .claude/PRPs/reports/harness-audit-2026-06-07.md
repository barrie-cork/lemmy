# Harness audit — 2026-06-07

**Branch:** `governance-v0`
**Run by:** `harness-audit` skill (project-scope, `.claude/skills/harness-audit/SKILL.md`)
**Scope:** Claude Code auto-load only. Pi-Coding excluded by design.

## Environment snapshot

`env: claude-code | settings.skillListingBudgetFraction=0.005 | mcp=project-memory,junior-brehon,ref-context,tavily | pi=present`

- Pi entry point (`AGENTS.md`): **present** (Pi-Coding entry point; this audit covers Claude Code only)
- `.claude/settings.json::skillListingBudgetFraction`: **0.005** (already at minimum sane value)
- MCP servers configured: project-memory, junior-brehon, ref-context, tavily
- Concurrent-session activity: **none** (1 commit in last 60 min, by current user `solo-dev`)

## ⚠ Headline finding — a shipped consolidation was silently regressed

On **2026-05-31**, commit `7f3056c9a` ("harness strip pass — 4-file merge to universal-guards")
**deleted** `circuit-breaker.md`, `escalation.md`, `integrator.md`, and `post-task-retro.md`
and created `universal-guards.md` as their verbatim consolidation — collapsing 4 always-load
file instances into 1 (~7,300 chars consolidated, net ~17% ALWAYS-load reduction claimed).

On **2026-06-04**, commit `999317082` — subject `chore: update skill sync manifest`, authored
by **BM Task** (`bm-task@brehon`) — **re-added all four deleted files verbatim** (183 insertions).
The misleading "skill sync manifest" subject masked a rules-corpus mutation that a BM-task session
is not supposed to make (`.claude/rules/**` is impl/advisor territory, not a BM sync artifact).

**Net effect today:** `universal-guards.md` **and** all four standalone files auto-load
simultaneously. The four rules now load **twice** — **8,683 chars / ~2,630 tokens of pure
duplication** that a prior shipped pass had eliminated. The 2026-05-31 audit even *predicted*
this trigger ("Redundancy in a consolidated 'error-handling rules' super-file") as the promotion
condition for circuit-breaker/escalation — that condition is now met because the super-file
exists and the originals came back.

This is the single highest-value, fully-actionable finding in this run.

## Auto-load inventory (Phase 1)

Counts re-derived deterministically parent-side (`wc -c`/`wc -l`); the Phase-1 subagent's summary
row had arithmetic errors (reported 204,213 ALWAYS chars — false) and is superseded here. All
per-file class assignments match the Phase-0 Step-6 grep authority (no `⚠ class-mismatch` rows).

| Class | Path | Lines | Chars | Est tokens (chars/4) |
|---|---|---|---|---|
| ALWAYS | .claude/rules/advisor-orchestrator.md | 295 | 41539 | 10385 |
| ALWAYS | .claude/rules/decision-queue.md | 430 | 27032 | 6758 |
| ALWAYS | .claude/rules/branch-manager.md | 222 | 12113 | 3028 |
| ALWAYS | .claude/rules/multi-lane-worktree.md | 93 | 8156 | 2039 |
| ALWAYS | .claude/rules/pmd-search-strategy.md | 62 | 5065 | 1266 |
| ALWAYS | .claude/rules/universal-guards.md | 93 | 4288 | 1072 |
| ALWAYS | .claude/rules/phase-branch.md | 72 | 3934 | 984 |
| ALWAYS | .claude/rules/memory-injection.md | 27 | 3096 | 774 |
| ALWAYS | .claude/rules/post-task-retro.md | 59 | 2947 | 737 |
| ALWAYS | .claude/rules/pmd-invariants.md | 50 | 2856 | 714 |
| ALWAYS | .claude/rules/circuit-breaker.md | 43 | 2056 | 514 |
| ALWAYS | .claude/rules/escalation.md | 47 | 1977 | 494 |
| ALWAYS | .claude/rules/integrator.md | 30 | 1703 | 426 |
| ALWAYS | .claude/rules/session-awareness.md | 23 | 1020 | 255 |
| ALWAYS | .claude/rules/gh-pr-fork-target.md | 16 | 506 | 127 |
| ALWAYS | .claude/rules/no-destructive-defaults.md | 7 | 385 | 96 |
| SCOPED | .claude/rules/governance-log-entry-kind-registry.md | 211 | 29755 | 7439 |
| SCOPED | .claude/rules/pm-plugin-hooks-stable.md | 95 | 5874 | 1469 |
| SCOPED | .claude/rules/view-crate-selectable-template.md | 109 | 5373 | 1343 |
| SCOPED | .claude/rules/no-cargo-output-paste.md | 57 | 3155 | 789 |
| SCOPED | .claude/rules/cargo-output-capture.md | 60 | 2830 | 708 |
| SCOPED | .claude/rules/cross-repo-coordination.md | 27 | 2124 | 531 |
| SCOPED | .claude/rules/evaluation-calibration.md | 24 | 1793 | 448 |
| SCOPED | .claude/rules/handover.md | 129 | 6698 | 1675 |
| SCOPED | .claude/rules/pre-phase-harness-audit.md | 211 | 12048 | 3012 |

**Totals:**
- ALWAYS-load files: **16** files, **118,673** chars, **~29,668** est. tokens (chars/4); **~35,961** tokens at the 3.3 chars/token empirical rate.
- SCOPED files: **9** files, **69,650** chars (do NOT auto-load in meta-work sessions).
- CLAUDE.md (root): 117 lines / 7,467 chars (~1,867 tokens, always-load — add to ALWAYS budget).
- `.claude/CLAUDE.md`: NOT FOUND.
- User-scope MEMORY.md: 186 lines (truncation safe: ≤195). Note: SessionStart warning already firing at 26.2 KB — route to `memory-prune`, not this skill.
- Skill listing budget: 19 SKILL.md files (capped by `skillListingBudgetFraction=0.005`; not a compression target here).

## Cross-reference quantification (Phase 2)

Citation counts below are **active-file only** (rules/refs/skills/commands/agents/templates) —
archives, reports, and historical handovers excluded, since only active files create rename risk.

| File | Active citations | Redundancy clusters detected |
|---|---|---|
| advisor-orchestrator.md | 33 | §5.2 validate-pending flow + §5.4 DQ-triage routing partially mirror decision-queue.md (1 cluster) |
| decision-queue.md | 31 | mirror of above (1 cluster) |
| branch-manager.md | 28 | none |
| multi-lane-worktree.md | 17 | none |
| universal-guards.md | 2 | **§§1–4 duplicate circuit-breaker/escalation/integrator/post-task-retro verbatim (1 cluster, 4 files)** |
| pmd-search-strategy.md | 3 | "brehon-fork PMD status" overlaps pmd-invariants.md §1 HTTP-topology note (partial) |
| post-task-retro.md | 2 | duplicated inside universal-guards.md §4 |
| pmd-invariants.md | 4 | none (narrative already extracted to refs/ on 2026-05-31) |
| circuit-breaker.md | 1 | duplicated inside universal-guards.md §1 |
| escalation.md | 1 | duplicated inside universal-guards.md §2 |
| integrator.md | 1 | duplicated inside universal-guards.md §3 |

Heading-anchor §-citations (active): `decision-queue` 17 · `advisor-orchestrator` 17 · `auto-phase` 11 · `branch-manager` 3 · `governance-log-entry-kind-registry` 0.

`.claude/refs/*.md` files: **12** (the read-on-demand convention is well-established).

## Composite scores (Phase 3)

Scored only ALWAYS files (SCOPED ceiling is 6.0 by design; they don't auto-load in meta-work).

| Path | Class | Size | Redundancy | CitDrag | Pi | Composite | Bucket |
|---|---|---|---|---|---|---|---|
| advisor-orchestrator.md | ALWAYS | 1.00 | 0.5 | 0.0 | safe | **8.0** | High-confidence |
| decision-queue.md | ALWAYS | 1.00 | 0.5 | 0.0 | safe | **8.0** | High-confidence |
| universal-guards.md | ALWAYS | 0.54 | 0.5 | 0.9 | safe | **7.7** | High-confidence |
| post-task-retro.md | ALWAYS | 0.37 | 0.5 | 0.9 | safe | **7.3** | High-confidence |
| multi-lane-worktree.md | ALWAYS | 1.00 | 0.0 | 0.15 | safe | **7.2** | High-confidence |
| circuit-breaker.md | ALWAYS | 0.26 | 0.5 | 0.95 | safe | **7.1** | High-confidence |
| escalation.md | ALWAYS | 0.25 | 0.5 | 0.95 | safe | **7.1** | High-confidence |
| integrator.md | ALWAYS | 0.21 | 0.5 | 0.95 | safe | **7.0** | High-confidence |
| branch-manager.md | ALWAYS | 1.00 | 0.0 | 0.0 | safe | **7.0** | High-confidence |
| pmd-search-strategy.md | ALWAYS | 0.63 | 0.0 | 0.85 | safe | **6.9** | High-confidence |
| phase-branch.md | ALWAYS | 0.49 | 0.0 | 0.7 | safe | **6.4** | High-confidence |
| memory-injection.md | ALWAYS | 0.39 | 0.0 | 0.9 | safe | **6.4** | High-confidence |
| pmd-invariants.md | ALWAYS | 0.36 | 0.0 | 0.8 | safe | **6.2** | High-confidence |
| session-awareness.md | ALWAYS | 0.13 | 0.0 | 0.95 | safe | **5.8** | Watch |
| no-destructive-defaults.md | ALWAYS | 0.05 | 0.0 | 0.9 | safe | **5.5** | Watch |
| gh-pr-fork-target.md | ALWAYS | 0.06 | 0.0 | 0.7 | safe | **5.4** | Watch |

> **Matrix-saturation note:** the 0.40 always-load weight floors every ALWAYS file near 4.0
> before any other factor, so the "high-confidence" bucket is crowded with files whose *only*
> high signal is "it always loads." The genuinely actionable signal in this table is the
> **redundancy=0.5** rows (the duplication cluster) — everything else is the matrix doing what
> it's calibrated to do, not a real trim opportunity. Read Phase 4 with that filter.

## Recommendations (Phase 4)

### High-confidence wins (composite ≥ 6.0)

| Priority | File(s) | Recommendation | Est. tokens saved | Pi-impact |
|---|---|---|---|---|
| **P1** | circuit-breaker.md, escalation.md, integrator.md, post-task-retro.md | **DELETE all four** — they were consolidated into `universal-guards.md` on 2026-05-31 and silently re-added by BM-task commit `999317082` on 2026-06-04. Content is verbatim-duplicated; `universal-guards.md` already carries §§1–4. This is a **regression revert**, not a new trim. | **~2,630** | circuit-breaker 0, integrator 0, post-task-retro 0, escalation 0 (the 1 "escalation" hit is the prose word in `.pi/prompts/prp-plan.md`, not a `.md` ref) — **all Pi-safe** |
| P2 | advisor-orchestrator.md / decision-queue.md | Collapse the remaining validate-pending + DQ-triage cross-duplication into one canonical section + pointer (per 2026-05-31 P2; re-verify recipe prose is fully delegated to `refs/dq-mechanics.md`). | ~625 | Pi-safe for extract; do NOT remove attribution rules or DQ schema (Junior contract) |
| P3 | pmd-search-strategy.md | Extract the "brehon-fork PMD status (2026-05-16)" operational-state block (~30 lines) to `refs/` or fold into `pmd-invariants.md` §1. Carried from 2026-05-31 P3 (not yet applied). | ~225 | 0 Pi hits |

### Watch items (composite 3.0–5.9)

| File | Why deferred | Promotion trigger |
|---|---|---|
| session-awareness.md | Small (1,020 chars); applies in all sessions; no redundancy | Gains redundancy with another always-load rule, OR grows past +50 lines |
| no-destructive-defaults.md | 385 chars — already near-minimal; universal skip-condition | Grows past 60 lines OR becomes Read-scopable without violating "applies always" |
| gh-pr-fork-target.md | 506 chars; cited 6× as a hard contract; minimal | Folded into a "git/PR conventions" super-file if one is ever created |
| pmd-invariants.md, phase-branch.md, memory-injection.md | Composite 6.2–6.4 only from the always-load floor; no real redundancy after 2026-05-31 narrative-extraction | Any gains a canonical-source duplicate in another always-load rule |

### Out of scope

Mandatory entries (Pi-shared invariants — do NOT trim):
- `.pi/**`, `AGENTS.md`
- `.claude/rules/branch-manager.md` — Pi subagents READ this (28 active citations; ALSO scored high but protected)
- `.claude/rules/no-cargo-output-paste.md` — SCOPED; Pi references by literal path
- `.claude/rules/decision-queue.md` (schema body) — Junior subagent canonical contract; only the non-schema validate-pending/triage *cross-duplication* (P2) is touchable
- `.claude/lessons/`, `.claude/skills/` — Pi reads/symlinks these

Other:
- `.claude/settings.json::skillListingBudgetFraction` — already at `0.005`; no further compression.
- `universal-guards.md` itself — it is the *intended* consolidation target; it stays. Only its duplicate-source files (P1) get deleted.

## What changed since last audit

- Prior audit: `.claude/PRPs/reports/harness-audit-2026-05-31.md`.
- The 2026-05-31 audit reported **14 ALWAYS files / 110,838 chars**. This run finds **16 ALWAYS files / 118,673 chars** — a **+7,835-char (+2 file) regression**, fully explained by the BM-task re-add (`999317082`).
- Files added back to ALWAYS (regression): `circuit-breaker.md`, `escalation.md`, `integrator.md`, `post-task-retro.md` (post-task-retro existed pre-merge too; net +4 standalone re-adds vs the post-merge baseline of 14, but only +2 vs the prior *audit count* because the prior audit was taken at the post-merge tip where these were absent — see headline).
- Files this run sees that prior audit did not list: `multi-lane-worktree.md` (ALWAYS, 8,156 chars — the prior audit's 14-file table omitted it; it was the same blind spot the 2026-05-31 dogfood block warned about — note for the next run).
- Net of P1 (the regression revert), ALWAYS-load would return to **~12 files / ~109,990 chars**, slightly below the 2026-05-31 post-merge baseline.

## Recommendation summary

- **Top 3 wins by token impact:**
  1. **DELETE circuit-breaker / escalation / integrator / post-task-retro** (regression revert of `999317082`) — **~2,630 tokens**
  2. advisor-orchestrator/decision-queue validate-pending + triage cross-duplication collapse — ~625 tokens
  3. pmd-search-strategy.md PMD-status block extract — ~225 tokens
- **Total estimated savings if all high-confidence wins apply:** **~3,480 tokens** (~12% of current 29,668-token ALWAYS-load).
- **Pi-Coding boundary check:** all P1–P3 targets are Pi-safe (0 real `.md` references in `.pi/`/`AGENTS.md`).
- **Process note (surface to user / next retro):** a BM-task session mutated `.claude/rules/**` under a `chore: update skill sync manifest` subject. Per `.claude/rules/branch-manager.md` "File ownership boundaries (HARD)", BM never touches rules. The sync-manifest tooling appears to restore deleted rule files as a side effect — that mechanism should be investigated so the consolidation doesn't get re-reverted after P1 is applied.

The user reviews this report and decides which (if any) trims to apply. This skill does not edit harness files.
