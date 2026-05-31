# Session retro — 2026-05-30 — code-audit-governance-deferred

**Harness:** claude-code
**Session window:** ~2026-05-30T22:00Z → ~2026-05-30T22:30Z (~30 min)
**Branch at start:** `740dd23c7` (`governance-v0`)
**Branch at end:** `50fa639ea` (`governance-v0`)
**Files touched:** 1 (`reports/code-audit-2026-05-30.md`)
**Commits:** 1 explicit (`50fa639ea`); PMD memories: 3 (id 669 qa-result, id 676 decision)

## TL;DR

Ran `/code-audit` on the Brehon governance crates (~21k lines, 84 files). Produced 18 findings across 8 files (Fair health), committed the report, wrote two PMD memories (qa-result + deferral decision). User explicitly deferred all refactoring until after v1 ships. The audit itself executed cleanly; the main finding for process is that the `code-audit` skill's Tier 1 static-analysis step is a no-op on Rust (ruff/shellcheck not applicable), defaulting entirely to Tier 3 (Claude structural analysis), which is slower but produced accurate results.

---

## What surprised us

- **Serena MCP absent — Tier 2 entirely skipped.** The skill has three tiers; Tier 2 (Serena structural analysis) is the highest-signal cheapest tier. It's simply absent from this project's `.mcp.json`. All structural analysis fell to Tier 3 (Claude reads files). Result was still accurate but required 10+ sequential file reads vs. a batch Serena call. This is not a surprise for a Rust project (Serena is primarily Python/Dart/TS-tuned) but worth noting.

- **`config.rs` composite score 9.8 — extreme outlier.** The "longest function" metric inflated due to the const-default match dispatch functions (~3,336-line approximation spanning the full const block to the next function). The actual dispatch functions are each ~170 lines; the metric's fn-boundary heuristic miscounts the gap to the const block end as function body. The *real* issue (const sprawl, 4,216-line file) is accurate; the metric's expression of it is misleading.

- **Zero `.unwrap()` in governance runtime paths.** Across all 84 governance files, all 3 `.unwrap()` calls are in `admin_config.rs` on static regex literals — a documented safe Rust idiom. This was a genuine positive surprise; it means the safety discipline enforced by CodeRabbit + clippy deny-warnings is working.

- **Prior audit (2026-05-14) context not carried.** PMD search returned the prior audit (id 304) confirming a similar audit was run 16 days ago. That audit targeted the whole codebase (64 findings); this one targeted governance-specific files only (18 findings, narrower scope). The two are complementary, not duplicates.

## What to change

| # | Change | Expected effect | Cost | Recurrence |
|---|---|---|---|---|
| 1 | Update `code-audit` SKILL.md Phase 2 Tier 1 table to add Rust row: `cargo clippy --workspace --no-deps -- -D warnings` as the linter | Gives cheap linter signal for Rust repos instead of defaulting to zero-linter Tier 3 for all files. Rust projects have no ruff/shellcheck/tsc equivalent currently in the table. | minor (1 table row) | 1× this session; likely recurs on every Rust audit |
| 2 | Add a note to `code-audit` SKILL.md Phase 3 ranking that the `longest_fn` heuristic (next-fn-start boundary) overestimates for files with large const blocks between functions; suggest a brace-depth-based fallback for Rust | Prevents misleading composite scores on const-registry files like `config.rs` | minor (1 paragraph note) | 1× this session |
| 3 | In the audit report Task Queue, explicitly note which tasks are deferred (with deferral reason) rather than just listing them as-ready-to-queue | Prevents a future session from picking up the queue and starting work on deferred items without seeing the deferral decision | minor (template change to code-audit SKILL.md Phase 6) | 1× this session (first explicit deferral we've tracked) |

## What to carry forward

- **Scope to governance-custom files only on Rust audits.** The full workspace has 600+ `.rs` files; auditing all of them would cost far more context than value. The governance crates (~84 files, ~21k lines of custom code) are where the ROI is.
- **`config.rs` const-sprawl is a known, planned-for issue.** Do not re-audit this as a surprise in future sessions. The const growth is by-design (parity-test-enforced); the right intervention is a module split at v1 completion, not incremental refactoring mid-flight.
- **Deferral decisions should be written to PMD immediately.** Wrote PMD id 676 immediately after user said "after v1 ships." This avoids future sessions re-surfacing the audit findings without context for why they were deferred.
- **`run_transaction` closures are intentionally large.** The three 300-680 line transaction closures (`process_vote`, `process_assignment`, `process_endorsement`) are load-bearing safety boundaries. Future code review should not flag them as simplification targets without understanding the CRITICAL-TRANSACTION-BOUNDARY directive.

---

## Three-signal scoring

| Skill / Agent / Command | Saved (min) | Wasted (min) | Surprise | Notes |
|---|---:|---:|---|---|
| `/code-audit` skill (full execution) | 60 | 5 | low | Tier 1 (ruff/shellcheck) silent no-op on Rust; Tier 2 (Serena) absent; Tier 3 accurate. 5 min "wasted" on checking for tools that don't apply. |
| PMD `memory_search_hybrid` pre-write search | 2 | 0 | none | Correctly surfaced prior audit (id 304) confirming this is a second audit in a series. |
| PMD `memory_write` ×2 | 2 | 0 | none | qa-result (id 669) + deferral decision (id 676) — both clean writes. |
| `/model sonnet` command | 0 | 0 | none | Routine; no context needed. |

## Complexity scores (heavy tasks only)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---:|---:|---:|---:|
| `/code-audit` skill execution (all phases) | 1 | 1 | ~25 | ~5 (sequential reads) |

No impl-task subagents were dispatched. Complexity is advisor-only read-heavy work.

## Decisions to revisit

- `code-audit` SKILL.md Tier 1 table: add Rust row before the next governance audit (post-v1). File: `.claude/skills/code-audit/SKILL.md`, Phase 2 table.
- Module split for `config.rs` const block: schedule as first task in the post-v1 refactor sweep (PMD id 676 + report).

---

## Promotion candidates (recurrence ≥ 2 in this session, or ≥ 1 here + ≥ 1 in prior memory)

None of the three "What to change" items meet the 2× recurrence threshold for lesson promotion — all are first-session occurrences specific to the `/code-audit` skill on Rust. They are SKILL.md edits, not cross-harness lessons.

- [ ] Change #1: add Rust/clippy row to `code-audit` SKILL.md Phase 2 Tier 1 table
- [ ] Change #2: add longest-fn heuristic caveat note to `code-audit` SKILL.md Phase 3
- [ ] Change #3: add deferral-note guidance to `code-audit` SKILL.md Phase 6 task descriptions

---

_Generated by `.claude/skills/session-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_retro_task_complexity_score.md`._
