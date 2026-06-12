# Harness audit — 2026-06-12

**Branch:** `governance-v0`
**Run by:** `harness-audit` skill (project-scope, `.claude/skills/harness-audit/SKILL.md`)
**Scope:** Claude Code auto-load only. Pi-Coding excluded by design.

## Environment snapshot

`env: claude-code | settings.skillListingBudgetFraction=0.005 | mcp=project-memory,junior-brehon,ref-context,tavily | pi=present`

- Pi entry point (`AGENTS.md`): **present** (dual-harness repo; this audit covers Claude Code only)
- `.claude/settings.json::skillListingBudgetFraction`: **0.005** (already at minimum sane value)
- MCP servers configured: project-memory, junior-brehon, ref-context, tavily
- ⚠ Concurrent-session activity: **13 commits in last 60 min** on `governance-v0` (`610581d2a` "author m2-late-2 impl-task-1 + impl-task-3 briefs (Cohort 1)", `4ffb45453` "auto-phase handover refresh", …). All authored by `solo-dev` (single-author repo — author field cannot discriminate), but the pattern matches the in-flight m2-late-2 `/auto-phase` session recorded in MEMORY.md. Observation only; audit proceeded. Note: the inventory below is a snapshot against a tree that another session is actively growing.

## ⚠ Subagent-drift note (Phase 0 Step 6 working as designed)

The Phase-1 Explore subagent drifted on **3 classifications** versus the authoritative parent-side grep, while reporting "no class-mismatches detected":

| File | Subagent said | Grep authority |
|---|---|---|
| `pre-phase-harness-audit.md` | ALWAYS | **SCOPED** |
| `pmd-search-strategy.md` | SCOPED | **ALWAYS** |
| `phase-branch.md` | SCOPED | **ALWAYS** |

This is the 3rd consecutive audit with subagent class-drift, but the first where it had **zero effect**: the grep table won, and all counts were re-verified parent-side via `wc` (subagent's counts were exact; only labels drifted). The subagent's summary totals (162,428 ALWAYS chars) were also wrong as a consequence — superseded below.

## Auto-load inventory (Phase 1 — counts verified parent-side via `wc`)

| Class | Path | Lines | Chars | Est tokens (chars/4) |
|---|---|---|---|---|
| ALWAYS | `.claude/rules/advisor-orchestrator.md` | 310 | 46,112 | 11,528 |
| ALWAYS | `MEMORY.md` (user-scope) | 228 | 33,002 | 8,251 |
| ALWAYS | `.claude/rules/decision-queue.md` | 430 | 27,032 | 6,758 |
| ALWAYS | `.claude/rules/branch-manager.md` | 222 | 12,113 | 3,028 |
| ALWAYS | `.claude/rules/multi-lane-worktree.md` | 94 | 10,353 | 2,588 |
| ALWAYS | `CLAUDE.md` (root) | 118 | 7,943 | 1,986 |
| ALWAYS | `.claude/rules/pmd-search-strategy.md` | 62 | 5,065 | 1,266 |
| ALWAYS | `.claude/rules/pmd-invariants.md` | 54 | 4,528 | 1,132 |
| ALWAYS | `.claude/rules/universal-guards.md` | 93 | 4,288 | 1,072 |
| ALWAYS | `.claude/rules/phase-branch.md` | 72 | 3,934 | 984 |
| ALWAYS | `.claude/rules/memory-injection.md` | 27 | 3,096 | 774 |
| ALWAYS | `.claude/rules/session-awareness.md` | 23 | 1,020 | 255 |
| ALWAYS | `.claude/rules/gh-pr-fork-target.md` | 16 | 506 | 127 |
| ALWAYS | `.claude/rules/no-destructive-defaults.md` | 7 | 385 | 96 |
| SCOPED | `.claude/rules/governance-log-entry-kind-registry.md` | 282 | 31,234 | 7,809 |
| SCOPED | `.claude/rules/pre-phase-harness-audit.md` | 259 | 12,048 | 3,012 |
| SCOPED | `.claude/rules/handover.md` | 187 | 7,909 | 1,977 |
| SCOPED | `.claude/rules/pi-harness-constraints.md` | 181 | 6,842 | 1,711 |
| SCOPED | `.claude/rules/pm-plugin-hooks-stable.md` | 124 | 5,874 | 1,469 |
| SCOPED | `.claude/rules/view-crate-selectable-template.md` | 137 | 5,373 | 1,343 |
| SCOPED | `.claude/rules/no-cargo-output-paste.md` | 76 | 3,155 | 789 |
| SCOPED | `.claude/rules/cargo-output-capture.md` | 75 | 2,830 | 708 |
| SCOPED | `.claude/rules/cross-repo-coordination.md` | 38 | 2,124 | 531 |
| SCOPED | `.claude/rules/evaluation-calibration.md` | 36 | 1,793 | 448 |

**Totals:**
- ALWAYS-load: **14** files (12 rules + root CLAUDE.md + user-scope MEMORY.md), **159,377** chars, **~39,844** est. tokens (chars/4); **~48,296** tokens at the 3.3 chars/token empirical rate.
- ALWAYS rules-only subtotal (comparable to prior audits): **12** files, **118,432** chars.
- SCOPED files: **10** files, **79,182** chars (do NOT auto-load in meta-work sessions).
- `.claude/CLAUDE.md`: does not exist.
- User-scope MEMORY.md: **228 lines — OVER the ≤195 truncation threshold.** The harness warning is already firing ("only part of it was loaded"); content past the cut is silently dropped each session. Also over the memory-prune skill's ~24.4 KB byte budget (at 31.8 KB).
- Skill listing budget: **17** SKILL.md files, 194,437 chars total bodies (catalogue cost capped by `skillListingBudgetFraction=0.005`; bodies load on invocation only).

## Cross-reference quantification (Phase 2)

| File | External citations (files citing) | Redundancy clusters detected |
|---|---|---|
| `advisor-orchestrator.md` | 100+ (319+ occurrences; incl. archives/plans) | (a) §5.2 validate-pending-laptop handler ↔ `decision-queue.md` kind text; (b) §3.1 Linux-compile gate trigger list ↔ `decision-queue.md` `-linux` variant scope trigger |
| `decision-queue.md` | 100+ (291+ occurrences) | mirror of the two clusters above |
| `multi-lane-worktree.md` | 100 files (260 occurrences) | inline incident narratives (Hard refusals #1, #6, #7) ↔ `refs/multi-lane-mechanics.md` (its own extraction target) |
| `branch-manager.md` | 100+ (165+ occurrences) | — (Pi-shared, out of scope) |
| `phase-branch.md` | 252 files | `--repo barrie-cork/lemmy` rule ↔ `gh-pr-fork-target.md` (full duplicate) ↔ `branch-manager.md` |
| `gh-pr-fork-target.md` | 142 files | entire content duplicated in `phase-branch.md` + `branch-manager.md` |
| `pmd-invariants.md` | 66 files | HTTP-topology paragraph ↔ MEMORY.md index entry |
| `pmd-search-strategy.md` | 39 files | "brehon-fork PMD status (2026-05-16)" block ↔ `pmd-invariants.md` #1/#3 — **stale duplicate that now contradicts the canonical** (see P3) |
| `no-destructive-defaults.md` | 37 files | — |
| `memory-injection.md` | 22 files | §3–4 "PMD search before acting" ↔ `pmd-search-strategy.md` |
| `session-awareness.md` | 19 files | — (but see P7: mandated script does not exist) |
| `universal-guards.md` | 11 files | — (consolidation target, recently created) |
| `MEMORY.md` | n/a | closed-phase entries (✅ m2-late-1, ✅ test-dogfood, ✅ pilot spin-up, …) ↔ workflow_state files / retros |

Heading-anchor citations (`# governance-log-entry-kind-registry|decision-queue|advisor-orchestrator|branch-manager|auto-phase`): 11 occurrences across 5 files — section headings are a cross-harness contract; extractions must preserve them.

`.claude/refs/*.md` files: **12** (read-on-demand convention is well established).

## Composite scores (Phase 3 — `helpers/scoring-matrix.md` weights verbatim)

SCOPED files excluded by design (hard ceiling 6.0; none scored). Factors shown as weighted contributions.

| Path | Class | Always (.40) | Size (.25) | Redund (.20) | Cit-drag (.10) | Pi (.05) | Composite | Bucket |
|---|---|---|---|---|---|---|---|---|
| `MEMORY.md` | ALWAYS | 0.40 | 0.250 | 0.20 | 0.100 | safe 0.05 | **10.0** | High-confidence |
| `advisor-orchestrator.md` | ALWAYS | 0.40 | 0.250 | 0.20 | 0.000 | pointer-safe 0.05 | **9.0** | High-confidence |
| `decision-queue.md` | ALWAYS | 0.40 | 0.250 | 0.20 | 0.000 | **shared** 0.00 | 8.5 | **Out of scope** (Pi hard-load + mandatory schema entry) |
| `multi-lane-worktree.md` | ALWAYS | 0.40 | 0.250 | 0.10 | 0.000 | safe 0.05 | **8.0** | High-confidence |
| `pmd-search-strategy.md` | ALWAYS | 0.40 | 0.158 | 0.10 | 0.000 | safe 0.05 | **7.1** | High-confidence |
| `CLAUDE.md` | ALWAYS | 0.40 | 0.248 | 0.00 | 0.000 | pointer-safe 0.05 | 7.0 | Watch (documented discretion) |
| `pmd-invariants.md` | ALWAYS | 0.40 | 0.142 | 0.10 | 0.000 | pointer-safe 0.05 | **6.9** | High-confidence |
| `memory-injection.md` | ALWAYS | 0.40 | 0.097 | 0.10 | 0.000 | safe 0.05 | **6.5** | High-confidence |
| `universal-guards.md` | ALWAYS | 0.40 | 0.134 | 0.00 | 0.045 | safe 0.05 | 6.3 | Watch (documented discretion) |
| `phase-branch.md` | ALWAYS | 0.40 | 0.123 | 0.10 | 0.000 | **shared** 0.00 | 6.2 | **Out of scope** (Pi hard-load) |
| `gh-pr-fork-target.md` | ALWAYS | 0.40 | 0.016 | 0.20 | 0.000 | **shared** 0.00 | 6.2 | **Out of scope** (Pi hard-load) |
| `branch-manager.md` | ALWAYS | — | — | — | — | **shared** | — | **Out of scope** (mandatory) |
| `session-awareness.md` | ALWAYS | 0.40 | 0.032 | 0.00 | 0.005 | safe 0.05 | 4.9 | **High-confidence by evidence** (documented discretion — see P7) |
| `no-destructive-defaults.md` | ALWAYS | 0.40 | 0.012 | 0.00 | 0.000 | safe 0.05 | 4.6 | Out of scope (already-minimum) |

**Pi-shared determination (evidence-based):** `.pi/extensions/lemmy-hooks.ts:139–141` **hard-loads** `decision-queue.md`, `phase-branch.md`, `gh-pr-fork-target.md` into Pi sessions — genuinely shared, never compress. `advisor-orchestrator.md` and `pmd-invariants.md` are cited from `.pi/`/`AGENTS.md` as read-on-demand pointers by path + section name (not loads) — scored Pi-safe per the matrix's worked-example discretion, with the constraint that **section headings must be preserved** in any extraction.

## Recommendations (Phase 4)

### High-confidence wins (composite ≥ 6.0, plus one evidence-promoted)

| Priority | File | Recommendation | Est. tokens saved | Pi-impact (grep `.pi/` + `AGENTS.md`) |
|---|---|---|---|---|
| **P1** | `MEMORY.md` (user-scope) | **Run `/memory-prune`.** 228 lines > 195 truncation threshold — content is already being silently dropped每 session; 31.8 KB > 24.4 KB budget. Closed-phase entries (✅ m2-late-1, ✅ test-dogfood, ✅ pilot spin-up, executed plans) are the prune set; several are explicitly marked "delete at m2-late-2 ship". | ~2,600 | 0 — user-scope file, not in repo |
| **P2** | `.claude/rules/advisor-orchestrator.md` | **Extract inline incident narrative to `refs/`** per the file's own §3.6 narrative-to-refs discipline: §1 stash-check incident tail (~700 ch), surface-first condition-(d) grep procedure + recurrence narrative (~1,100 ch), §5.2 Shape-G-disabled fast-path narrative (keep rule statement + `workflow_run_id: 0` sentinel, move the 422/dogfood story, ~1,200 ch), §3.3 OQ-resolvability rationale (~900 ch), §3.1 finalize-merge look-order narrative (~700 ch). **Keep every section heading** (cross-harness citation contract). | ~1,400 | 16 bare-name hits — all path/section pointers, no loads; heading-preserving extract is Pi-safe |
| **P3** | `.claude/rules/pmd-search-strategy.md` | **Replace the stale "brehon-fork PMD status (2026-05-16)" block (~2,600 ch) with a 3-line pointer to `pmd-invariants.md` #1/#3.** This is a correctness fix as much as a trim: the block instructs laptop-local `backfill.js` re-backfill and laptop DB paths that the 2026-06-11 homeserver-HTTP cutover retired — `pmd-invariants.md` #3 now explicitly forbids re-adding inline `backfill.js`. An auto-loaded rule currently contradicts the canonical invariant every session. | ~700 | 1 bare-name hit, 0 path hits — safe |
| **P4** | `.claude/rules/multi-lane-worktree.md` | **Finish the narrative extraction to `refs/multi-lane-mechanics.md`:** Hard refusal #7's incident story (~1,200 ch beyond the rule statement) and #1's 2026-06-07 incident tail (~400 ch). #6 already has the refs pointer pattern to copy. Keep refusal statements + triggers inline. | ~500 | 0 hits — safe |
| **P5** | `.claude/rules/pmd-invariants.md` | **Move the IP-CORRECTION blockquote + pre-cutover archive-context parenthetical (~1,300 ch) to `refs/pmd-invariants-incidents.md`** (file already exists for exactly this). Keep the one-line "homeserver = 100.81.145.58; laptop = 100.104.171.26, do NOT use" warning inline. | ~400 | 6 bare-name hits, 0 path hits — pointers, safe |
| **P6** | `.claude/rules/memory-injection.md` | **Delete dead steps 1–2** (`docs/memory/PATTERNS.md` and `docs/memory/KNOWN_ISSUES.md` do not exist in this repo — verified this run) **and fold §"PMD search before acting" into a 1-line pointer to `pmd-search-strategy.md`** (whose content it duplicates). Keep the cross-cutting patterns list. Not a file deletion — no ledger entry needed. | ~350 | 1 bare-name hit — safe |
| **P7** | `.claude/rules/session-awareness.md` | **Delete the file** (documented discretion: composite only 4.9, but the rule is unenforceable dead weight — it mandates `bash scripts/agent-activity.sh`, which **does not exist in this repo**, references the tanglewood-hive environment, and its claimed SessionStart/SessionEnd auto-registration hooks are not wired in `.claude/settings.json`). Cross-session conflict detection is already covered by `multi-lane-worktree.md` + the surface-first ritual. **Delete-ledger requirement:** applying this MUST append `session-awareness.md` to `.claude/refs/harness-deleted-rules.txt` so `harness-regression-guard.sh` catches a silent re-add (the 2026-06-04 sync-tool regression vector). | ~310 | 2 bare-name hits, 0 path hits — safe |

### Watch items (composite 3.0–5.9, or ≥6.0 with documented discretion)

| File | Why deferred | Promotion trigger |
|---|---|---|
| `CLAUDE.md` (7.0) | Discretion: index function is the always-load contract — yaml-dense, already trimmed; its summaries are deliberate pointers, not redundancy | Grows past 9 KB OR gains a narrative block >3 lines |
| `universal-guards.md` (6.3) | Discretion: composite crosses 6.0 only via low-citation drag bonus; zero redundancy duplicates; it IS the consolidation target from 2026-05-31 — re-trimming it now churns a fresh artifact | Gains a canonical-source duplicate OR grows +50 lines; §4's stop-hook procedural detail (~800 ch) is the only extract shape if promoted |
| `decision-queue.md` non-schema prose | File is mandatory out-of-scope (schema body + Pi hard-load), but the `-linux` variant rationale (~1,400 ch) duplicates `advisor-orchestrator.md` §3.1's gate trigger | A third `validate-pending-laptop-*` variant lands → extract variant-rationale prose to `refs/dq-mechanics.md`, keep field contracts inline |
| **Growth-rate itself** | Surviving 12 ALWAYS rules grew **+8,442 chars in 5 days** (~+1,700/day), fully consuming the 2026-06-07 dedup win (see delta section). §3.6 narrative-to-refs at author time exists but mid-orchestration edits (other session, today's `af8ba7ccb`) keep adding inline narrative | If the next audit shows rules-only ALWAYS > 125 KB, recommend a PostToolUse hook hard-warn (the existing `rule-narrative-bloat-reminder.sh` is evidently advisory-only and being out-paced) |

### Out of scope

Mandatory entries (Pi-shared invariants):

- `.pi/**`
- `AGENTS.md`
- `.claude/rules/branch-manager.md` — Pi subagents READ this (`.pi/prompts/bm-*.md`, `.pi/skills/bm-task/SKILL.md`)
- `.claude/rules/no-cargo-output-paste.md` — `.pi/extensions/lemmy-hooks.ts` references by literal path (SCOPED anyway this run)
- `.claude/rules/decision-queue.md` (schema body) — Junior subagents use it as canonical contract; hard-loaded by `lemmy-hooks.ts:139`
- `.claude/lessons/` — Pi subagents read at startup
- `.claude/skills/` — Pi shares these

Other entries:

- `.claude/rules/phase-branch.md` + `.claude/rules/gh-pr-fork-target.md` — **hard-loaded by `lemmy-hooks.ts:140–141`**; never compress despite gh-pr-fork-target's full duplication elsewhere (it's 127 tokens; the duplication is load-bearing redundancy for Pi)
- `.claude/rules/no-destructive-defaults.md` — already-minimum (385 chars)
- `.claude/settings.json::skillListingBudgetFraction` — already at 0.005

## What changed since last audit

- Prior audit: `.claude/PRPs/reports/harness-audit-2026-06-07.md`
- **Rules-only ALWAYS chars: prior 118,673 (16 files) → current 118,432 (12 files). Net −241 chars (−0.2%).**
- **Decomposition of that flat line:** the prior audit's P1 was applied (`f0b7cffcd` deleted `circuit-breaker.md`, `escalation.md`, `integrator.md`, `post-task-retro.md`, −8,683 chars; ledger populated with all 4 basenames; `harness-regression-guard.sh` shipped and wired) — **but organic growth of +8,442 chars across the surviving 12 files in 5 days consumed the entire win.** The harness is running to stand still.
- Files deleted from `.claude/rules/`: the 4 above (tombstoned ✓, guard wired ✓ — verified this run).
- Files added to ALWAYS: none. Files moved ALWAYS → SCOPED: none.
- CLAUDE.md: 7,467 → 7,943 chars (+476).
- New in this run's scope: user-scope MEMORY.md (33,002 chars) counted into the ALWAYS budget and found over the truncation threshold — prior reports tracked line count only.
- Full ALWAYS total (rules + CLAUDE.md + MEMORY.md): **159,377 chars ≈ 48.3K tokens at 3.3 chars/token** — roughly 24% of a 200K window before the first user message.

## Recommendation summary

- **Top 3 wins by token impact:**
  1. `MEMORY.md` — run `/memory-prune` (over truncation threshold; content silently dropping) — ~2,600 tokens
  2. `.claude/rules/advisor-orchestrator.md` — narrative→refs extraction, headings preserved — ~1,400 tokens
  3. `.claude/rules/pmd-search-strategy.md` — replace stale 2026-05-16 PMD-status block (contradicts canonical topology) — ~700 tokens
- **Total estimated savings if all high-confidence wins apply: ~6,260 tokens** (~13% of the 48.3K-token full ALWAYS load).
- **Pi-Coding boundary check:** all 7 high-confidence targets Pi-safe (0 path-loads in `.pi/`/`AGENTS.md`; P2/P5 require heading preservation). 3 Pi hard-loaded rules identified and excluded.
- **Process notes:** (1) P3 is a correctness fix, not just a trim — an ALWAYS rule currently instructs a forbidden operation (laptop `backfill.js`) every session. (2) The growth-rate watch item is the structural finding: per-audit trims are being out-paced by mid-orchestration inline-narrative additions.

The user reviews this report and decides which (if any) trims to apply. This skill does not edit harness files.
