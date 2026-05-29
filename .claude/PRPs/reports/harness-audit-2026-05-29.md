# Harness audit — `2026-05-29`

**Branch:** `governance-v0`
**Run by:** `harness-audit` skill (project-scope, `.claude/skills/harness-audit/SKILL.md`)
**Scope:** Claude Code auto-load only. Pi-Coding excluded by design.

> **Headline:** the rules corpus is **already well-trimmed** and **dual-harness shared**. ALWAYS-load
> is ~42.7K tokens ≈ **21% of the 200K effective working window** (not the ~30% the pre-audit framing
> assumed — 9 of 24 rule files are SCOPED and don't auto-load). The three largest files
> (`advisor-orchestrator.md`, `decision-queue.md`, `multi-lane-worktree.md`) score high-confidence on
> *size* but are **referenced 68× across 18 `.pi/` files by named section anchor** — so the safe action
> is **narrow narrative-extraction that preserves every Pi-cited heading**, NOT bulk extraction or
> rename. Total realistic win ≈ **3–4K tokens (~1.5–2% of budget)**. The corpus is healthier than the
> "rules are the lever" framing suggested; the biggest single lever is keeping it from growing, not
> cutting it now.

## Environment snapshot

- Pi entry point (`AGENTS.md`): **present** — repo is Pi-shared; this audit covers Claude Code only.
- `.claude/settings.json::skillListingBudgetFraction`: **0.005** (already at minimum sane value — OUT of scope).
- MCP servers configured: `project-memory, junior-brehon, ref-context, tavily`.
- Concurrent-session check: no non-self commits on `governance-v0` in the last 60 min.

`env: claude-code | settings.skillListingBudgetFraction=0.005 | mcp=project-memory,junior-brehon,ref-context,tavily | pi=present`

## Auto-load inventory (Phase 1)

(From the persisted Phase-1 Explore-subagent inventory: `.claude/PRPs/reports/harness-audit-2026-05-29-phase1-inventory.md`.)

| Class | Path | Lines | Chars | Est tokens (chars/4) |
|---|---|---|---|---|
| ALWAYS | `.claude/rules/advisor-orchestrator.md` | 375 | 47,413 | 11,853 |
| ALWAYS | `.claude/rules/decision-queue.md` | 492 | 30,596 | 7,649 |
| SCOPED | `.claude/rules/governance-log-entry-kind-registry.md` | 244 | 26,794 | 6,698 |
| ALWAYS | MEMORY.md (user-scope) | 186 | 25,375 | 6,344 |
| ALWAYS | `.claude/rules/multi-lane-worktree.md` | 348 | 17,731 | 4,433 |
| SCOPED | `.claude/rules/pre-phase-harness-audit.md` | 259 | 12,048 | 3,012 |
| ALWAYS | `.claude/rules/branch-manager.md` | 222 | 11,958 | 2,990 |
| ALWAYS | `.claude/rules/pmd-invariants.md` | 152 | 9,254 | 2,314 |
| ALWAYS | CLAUDE.md (root) | 116 | 7,068 | 1,767 |
| SCOPED | `.claude/rules/handover.md` | 163 | 6,698 | 1,674 |
| SCOPED | `.claude/rules/pm-plugin-hooks-stable.md` | 124 | 5,874 | 1,468 |
| SCOPED | `.claude/rules/view-crate-selectable-template.md` | 137 | 5,373 | 1,343 |
| ALWAYS | `.claude/rules/pmd-search-strategy.md` | 52 | 4,243 | 1,061 |
| ALWAYS | `.claude/rules/phase-branch.md` | 72 | 3,934 | 984 |
| SCOPED | `.claude/rules/no-cargo-output-paste.md` | 76 | 3,155 | 789 |
| ALWAYS | `.claude/rules/memory-injection.md` | 27 | 3,096 | 774 |
| SCOPED | `.claude/rules/cargo-output-capture.md` | 75 | 2,830 | 708 |
| ALWAYS | `.claude/rules/post-task-retro.md` | 58 | 2,580 | 645 |
| SCOPED | `.claude/rules/cross-repo-coordination.md` | 38 | 2,124 | 531 |
| ALWAYS | `.claude/rules/circuit-breaker.md` | 43 | 2,056 | 514 |
| ALWAYS | `.claude/rules/escalation.md` | 47 | 1,977 | 494 |
| SCOPED | `.claude/rules/evaluation-calibration.md` | 36 | 1,793 | 448 |
| ALWAYS | `.claude/rules/integrator.md` | 30 | 1,703 | 426 |
| ALWAYS | `.claude/rules/session-awareness.md` | 23 | 1,020 | 255 |
| ALWAYS | `.claude/rules/gh-pr-fork-target.md` | 16 | 506 | 126 |
| ALWAYS | `.claude/rules/no-destructive-defaults.md` | 7 | 385 | 96 |

**Totals:**
- ALWAYS-load files: **15 rule files**, 138,452 chars, **34,613 est. tokens**.
- + CLAUDE.md (1,767) + MEMORY.md (6,344) = **170,895 chars / ~42,724 est. tokens total ALWAYS-load ≈ 21% of 200K**.
- SCOPED files: **9 files**, 66,689 chars / 16,672 tokens — do NOT auto-load in meta-work sessions.
- User-scope MEMORY.md: **186 lines** (line-truncation safe at ≤195) BUT **25,375 bytes = 104% of the 24,400-byte budget — OVER by ~975 bytes**. Bytes bind first; see Watch items.
- Skill listing budget: **16 SKILL.md files** (167,532 chars total); capped by `skillListingBudgetFraction=0.005`.

## Cross-reference quantification (Phase 2)

| File | External citations (non-`rules/`) | Redundancy clusters detected |
|---|---|---|
| `advisor-orchestrator.md` | 250+ files | none — `ci-watcher mutation pattern` / `§G4 classifier` are canonical-in-`decision-queue`-or-self, cross-referenced (2026-05-09 collapse already deduped) |
| `decision-queue.md` | 250+ files | none — per-kind routing canonical here, pointer in advisor-orchestrator §5.4 |
| `multi-lane-worktree.md` | ~120 files | none |
| `branch-manager.md` | ~200 files | SessionStart-ritual prose shared (descriptive, not mechanism) |
| `pmd-invariants.md` | ~34 files | SessionStart-ritual prose duplicated across branch-manager + multi-lane + session-awareness (1 cluster) |

`.claude/refs/*.md` files: **9** (`dq-recipes`, `auto-roadmap`, `advisor-narrow-gates`, `advisor-subagent-dispatch`, `auto-phase`, `dq-mechanics`, `bm-mechanics`, `multi-lane-mechanics`, `advisor-validation`) — the read-on-demand convention is **already heavily used**; the top-2 ALWAYS files carry 6–7 refs pointers each.

### Pi-boundary finding (load-bearing — changes the recommendations)

`.pi/` references the top rule files **68× across 18 files**. Critically, the references are predominantly
to **named section anchors by prose**, not just filenames:

- `advisor-orchestrator.md "Stage-shape orchestration"` — `.pi/prompts/brehon-clarify.md` (×2), `.pi/prompts/brehon-verify.md`
- `advisor-orchestrator.md "Catch-fire procedures"` — `.pi/prompts/brehon-verify.md` (×3)
- `decision-queue.md Attribution integrity` — `.pi/agents/bm-pi.md`
- `decision-queue.md Mid-task visibility` — `.pi/skills/bm-task/SKILL.md`
- `.pi/skills/ci-watcher/SKILL.md` cites the `validate-pending` mutation contract (×11)

**Consequence:** every section *heading* Pi cites by name must remain a heading in its current file.
Extraction is admissible **only for explanatory sub-prose inside a Pi-cited section** (incident
narratives, false-positive/false-negative taxonomies) — keeping the heading + adding a refs pointer.
Bulk extraction, section rename, or moving a Pi-cited heading would break Pi-Coding **silently**.

## Composite scores (Phase 3)

Per `helpers/scoring-matrix.md` (weights: always 0.40, size 0.25, redundancy 0.20, citation-drag 0.10, pi 0.05).

| Path | Class | Size | Redund | Cite-drag | Pi | Composite | Bucket |
|---|---|---|---|---|---|---|---|
| `pmd-invariants.md` | ALWAYS | 1.0 | 0.5 | 0.0 | safe | **8.0** | High-confidence |
| `advisor-orchestrator.md` | ALWAYS | 1.0 | 0.0 | 0.0 | safe | **7.0** | High-confidence |
| `decision-queue.md` | ALWAYS | 1.0 | 0.0 | 0.0 | safe | **7.0** | High-confidence |
| `multi-lane-worktree.md` | ALWAYS | 1.0 | 0.0 | 0.0 | safe | **7.0** | High-confidence |
| `branch-manager.md` | ALWAYS | 1.0 | 0.0 | 0.0 | **shared** | 6.5 | High-conf. but Pi-OUT |
| `pmd-search-strategy.md` | ALWAYS | 0.53 | 0.0 | 0.6 | safe | 6.43 | High-confidence |
| `memory-injection.md` | ALWAYS | 0.39 | 0.0 | 0.5 | safe | 5.97 | Watch |
| `circuit-breaker.md` | ALWAYS | 0.26 | 0.0 | 0.75 | safe | 5.89 | Watch |
| `integrator.md` | ALWAYS | 0.21 | 0.0 | 0.85 | safe | 5.88 | Watch |
| `escalation.md` | ALWAYS | 0.25 | 0.0 | 0.75 | safe | 5.87 | Watch |
| `phase-branch.md` | ALWAYS | 0.49 | 0.0 | 0.0 | safe | 5.73 | Watch |
| `post-task-retro.md` | ALWAYS | 0.32 | 0.0 | 0.25 | safe | 5.56 | Watch |
| `session-awareness.md` | ALWAYS | 0.13 | 0.0 | 0.6 | safe | 5.42 | Watch |
| `gh-pr-fork-target.md` | ALWAYS | 0.06 | 0.0 | 0.75 | safe | 5.41 | Watch |
| `no-destructive-defaults.md` | ALWAYS | 0.05 | 0.0 | 0.75 | safe | 5.37 | Watch |

> **Scoring caveat (read before acting):** the composite is a *priority* signal driven mostly by raw
> size. A high score does NOT mean "lots of extractable fat" — `advisor-orchestrator.md` and
> `decision-queue.md` are already 6–7-refs-externalized, so their high score reflects residual
> *load-bearing* bulk (schema, mechanisms, Pi-cited headings), not slack. Phase 4 applies the
> per-section load-bearing judgment the score can't.

## Recommendations (Phase 4)

### High-confidence wins (composite ≥ 6.0)

| Priority | File | Recommendation | Est. tokens saved | Pi-impact |
|---|---|---|---|---|
| P1 | `advisor-orchestrator.md` §1 Polling loop | Extract the **incident narratives + hook false-positive/false-negative taxonomies** (the "Surface-first ritual" cf93b7ba6 story, the "SessionStart multi-lane check" FP/FN paragraphs) to `.claude/refs/advisor-orchestrator-incidents.md`. Keep terse rule statements + a refs pointer. **No §-heading Pi cites is touched** (§1 has no Pi-cited named anchor; Pi cites "Stage-shape orchestration"/"Catch-fire procedures" which stay). | ~700–900 | `.pi/` cites §1? **0 hits** — Pi-safe |
| P2 | `decision-queue.md` | Extract the two **historical/deprecated** sections — `### Next-id calculation (pre-v3 — historical only)` and `### Deprecated kinds (historical-only)` — to `.claude/refs/dq-mechanics.md` (already exists; both already have a "full historical context" pointer there). Keep a one-line "deprecated; see refs" stub. | ~500–700 | Pi cites `decision-queue.md` schema/attribution/mid-task — NOT these historical sections. Pi-safe |
| P3 | `pmd-invariants.md` | Fold the **SessionStart-ritual prose** (duplicated across branch-manager + multi-lane + session-awareness) into a single canonical statement + pointer. Lowest-risk redundancy collapse; the 0.5 redundancy term is the only reason this scored 8.0. | ~300–500 | branch-manager is Pi-shared — collapse INTO pmd-invariants (Claude-owned), leave branch-manager's copy untouched. Pi-safe |
| P4 | `multi-lane-worktree.md` | Mode-A/Mode-B narrative + the `### Brief location and trunk→phase sync` SSH-recipe block are extract candidates to `.claude/refs/multi-lane-mechanics.md` (exists). But verify Pi doesn't cite "Lane modes"/"trunk→phase sync" by name first. | ~600–900 | `.pi/` cites multi-lane-worktree §? **needs per-section grep before acting** — surfaced for review |

**Total high-confidence realistic win: ~2,100–3,000 tokens (~1–1.5% of the 200K budget).**

### Watch items (composite 3.0–5.9)

| File | Why deferred | Promotion trigger |
|---|---|---|
| **MEMORY.md** (user-scope) | **Over the 24.4 KB byte budget by ~975 bytes RIGHT NOW** (186 lines / 25,375 bytes). Already pruned this session; the residual is structural — the rules corpus is the bigger lever, but MEMORY.md is *actively truncating*. | **Already fired** — recommend a `memory-prune` byte-pass (shorten the 5–10 longest entries / move stale CLOSED-lane lines to Historical). Bytes, not lines. |
| `memory-injection.md`, `circuit-breaker.md`, `escalation.md`, `integrator.md` | Small (≤3 KB), no redundancy, broadly cited. Extraction would cost more in refs-pointer overhead than it saves. | If any grows past +50 lines OR gains a canonical-source duplicate. |
| `phase-branch.md`, `post-task-retro.md` | Mid-size, fully load-bearing (phase-branch is the PR-flow contract; post-task-retro is the Stop-hook contract). No slack. | If a duplicate of the PR-flow / retro-rubric prose appears in another always-load file. |
| `session-awareness.md`, `gh-pr-fork-target.md`, `no-destructive-defaults.md` | Tiny (<1 KB). Below any extraction threshold. | Never (floor entries). |

### Out of scope

Mandatory entries (Pi-shared invariants — Pi READS these; do NOT trim):

- `.pi/**`
- `AGENTS.md`
- `.claude/rules/branch-manager.md` — Pi subagents read it (`.pi/agents/bm-pi.md`, `.pi/skills/bm-task/SKILL.md`); composite 6.5 but Pi-OUT.
- `.claude/rules/no-cargo-output-paste.md` — `.pi/extensions/lemmy-hooks.ts` references by literal path.
- `.claude/rules/decision-queue.md` **(schema / attribution / mid-task / per-kind body)** — Junior AND Pi subagents use it as canonical contract; **only the historical/deprecated sections (P2) may be touched**.
- `.claude/lessons/` — Pi subagents read at startup.
- `.claude/skills/` — Pi shares via `.pi/settings.json`.

Other entries:

- `.claude/settings.json::skillListingBudgetFraction` — already at `0.005`; no further compression without changing Claude Code minor-version behaviour.
- All 9 SCOPED rule files — they don't auto-load in meta-work sessions; compressing them yields zero session-start savings.

## What changed since last audit

- Prior audit: `.claude/PRPs/reports/harness-audit-hooks-2026-05-22.md` — but that was a **hooks-dir audit**, not a rules-corpus auto-load audit. There is **no prior rules-corpus auto-load total** to delta against. This is the **first full rules-corpus auto-load audit** with the corrected ALWAYS/SCOPED classification.
- (The `harness-audit-2026-05-29-phase1-inventory.md` is this run's own Phase 1 artifact, not a prior audit.)
- Establishes the baseline: **ALWAYS-load = 170,895 chars / ~42,724 tokens** for future audit-to-audit comparison.

## Recommendation summary

- **Top 3 wins by token impact:**
  1. `multi-lane-worktree.md` — extract Mode-A/B narrative + SSH-recipe to existing `refs/multi-lane-mechanics.md` (pending per-section Pi grep) — ~600–900 tokens
  2. `advisor-orchestrator.md` §1 — extract incident narratives + hook taxonomies to a new `refs/` file — ~700–900 tokens
  3. `decision-queue.md` — extract 2 historical/deprecated sections to existing `refs/dq-mechanics.md` — ~500–700 tokens
- **Total estimated savings if all high-confidence wins apply: ~2,100–3,000 tokens (~1–1.5% of the 200K budget).**
- **Separate, already-firing:** MEMORY.md is ~975 bytes over its 24.4 KB hard budget — run a `memory-prune` byte-pass (independent of the rules-corpus work).
- **Pi-Coding boundary check:** **1 entry surfaces for review** — `multi-lane-worktree.md` P4 needs a per-section `.pi/` grep ("Lane modes" / "trunk→phase sync") before extracting, since `.pi/` references the file. All other high-confidence rows are Pi-safe (verified: `.pi/` cites the *headings that stay*, not the *prose that moves*).

### Honest bottom line

The pre-audit hypothesis was "the rules corpus is the 86% lever, go cut it." The audit **partially refutes
that**: the corpus auto-loads at ~21% of budget (not 30%), it's already heavily refs-externalized, and
it's a **dual-harness shared contract** where the obvious cuts would break Pi silently. The genuine,
safe win is **modest (~1–1.5%)** and concentrated in narrative-extraction that preserves Pi-cited
headings. **The strongest lever is discipline against future growth** (every new incident post-mortem
should go to `refs/` from the start, not inline) — plus the independent, already-firing MEMORY.md
byte-prune. Recommend applying P1–P3 (Pi-safe) and gating P4 on the per-section Pi grep.

---

The user reviews this report and decides which (if any) trims to apply. This skill does not edit harness files.
