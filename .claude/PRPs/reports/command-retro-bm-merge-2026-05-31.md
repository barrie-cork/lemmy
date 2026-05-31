# Command retro — /bm-merge — 2026-05-31

**Spec path:** `.claude/commands/bm/bm-merge.md`
**Spec last modified:** `e66cf272b` (2026-05-22 — 9 days; codified Phase 5.5 post-condition + hard-refusal contract, applying prior retro B1+B2)
**Prior retro:** `.claude/PRPs/reports/command-retro-bm-merge-2026-05-22.md` — composite 0.68
**New invocations since prior retro:** 2 (invocations #7 and #8)
**Cumulative invocations:** 8
**Score sources:** PMD · v1-quality-r3-retro.md · v1-quality-r3b-retro.md

---

## TL;DR

`/bm-merge` continues net-positive. The prior retro's two Class B proposals (B1: Phase 5.5 post-condition + B2: hard-refusal contract) were both applied in `e66cf272b` — confirmed present in spec lines 339–375 and 86–122. The two new invocations (#7 quality-r3, #8 quality-r3b) both executed correctly. One new Class B proposal: add `UNSTABLE` to the Phase 2.2 mergeStateStatus table with the adr-compliance bypass guidance — the state has been observed 2× and the spec defaults to "Other → STOP: unexpected merge state" which is safe but under-specified. Composite score rises slightly to 0.71 (8 invocations, net +83 min).

---

## Prior retro follow-through

| Item | Proposal | Status |
|---|---|---|
| B1 | Codify Phase 5.5 trust-but-verify post-condition in spec | **APPLIED** — `e66cf272b` (spec lines 339–375) |
| B2 | Promote hard-refusal contract (5 NEVER clauses) to spec body | **APPLIED** — `e66cf272b` (spec lines 86–122) |

Both proposals from the prior retro shipped before invocations #7 and #8. Invocation #7 (quality-r3) saw `--admin` bypass with local scan pass — the hard-refusal contract correctly didn't block on this (the refusals are about bad BM behaviors, not about merge-blocking states). Invocation #8 (quality-r3b) was the cleanest bm-merge in the corpus: ~2 min, all post-conditions passed, no surprises.

---

## Findings

### What surprised us

- **UNSTABLE is not in the Phase 2.2 mergeStateStatus table.** Both invocations #7 (quality-r3, adr-compliance CI flaky) and this phase's environment have `UNSTABLE` as the GitHub status when a non-blocking check is pending/failed-but-non-required. The spec table has CLEAN / BEHIND / BLOCKED / DIRTY / UNKNOWN / HAS_HOOKS / Other — no UNSTABLE row. BM defaulted to "Other → STOP: Unexpected merge state" which is safe, but the carry-forward from v1-quality-r3 retro explicitly names the bypass path: local scan pass + `--admin`. Two observations warrant a Class C watch entry; three would trigger Class B application.

  **Wait — recurrence check:** v1-quality-r3 retro §2.1 explicitly flags this as a carry-forward action: "Add a note to the bm-merge brief template: if `mergeStateStatus: UNSTABLE` due to adr-compliance and the local scan exits 0, skip the re-trigger loop and go straight to acknowledge-comment + `--admin`." That's a second observation on top of the quality-r3b phase. Plus the UNSTABLE state has now occurred in at least 2 distinct phases (quality-r3 PR #169 + quality-r3b PR #170). That's 2× confirmed — Class C watch (one more occurrence triggers Class B upgrade).

- **Both new invocations were clean.** Prior retro noted 4/6 clean invocations. Now 6/8 — the friction is still concentrated in the older episodes. Phase 5.5 post-condition has fired correctly in all subsequent invocations.

### What's working

- **Phase 5.5 post-condition** — fired and passed on both new invocations. Codifying it in `e66cf272b` worked.
- **Hard-refusal contract** — no violations in invocations #7 or #8. Contract text in spec is being applied.
- **`--admin` bypass path** — not in the spec, but the advisor-side bm-merge brief carried the instruction. First use in quality-r3; clean execution.

---

## Three-signal scoring

### New invocations (#7, #8)

| # | Context | Saved (min) | Wasted (min) | Surprise | Source |
|---|---|---:|---:|---|---|
| 7 | v1-quality-r3 PR #169 — bm-merge #537, blocked UNSTABLE (adr-compliance flaky); BM defaulted correctly to STOP; advisor handled `--admin` bypass | 15 | 3 | low | v1-quality-r3-retro.md §4 BM-task signals |
| 8 | v1-quality-r3b PR #170 — bm-merge #547, fully clean ~2 min | 18 | 0 | none | v1-quality-r3b-retro.md §per-role |

### Cumulative metrics (8 invocations)

| Metric | Prior (6 inv) | Delta | Current (8 inv) |
|---|---|---|---|
| Total saved (min) | 89 | +33 | 122 |
| Total wasted (min) | 42 | +3 | 45 |
| **Net (min)** | **+47** | **+30** | **+77** |
| Surprise events (med+high) | 2 | +0 | 2 |
| Friction score | 0.32 | — | 0.27 |
| **Composite score** | **0.68** | **+0.03** | **0.71** |

Anti-inflation check: 0.71 within the 0.60-0.75 band. 6 of 8 invocations clean (18 saved / 0 wasted / no surprise). Rising slowly — correct trajectory for a verb whose one bad episode is getting further in the past.

---

## Class A — Minor edits auto-applied

_No minor edits applied._ All lesson paths resolve:
- `feedback_coderabbit_block_merge_critical.md` ✓
- `feedback_branch_manager_pm_split.md` ✓
- `feedback_l14_runlog_on_trunk_self_conflicts_with_bm_pr.md` ✓
- `feedback_bm_false_success_advisor_post_condition_catch.md` ✓ (referenced in Phase 5.5)

No typos, no dead URLs, no broken paths, no frontmatter drift.

---

## Class B — Major proposals (surfaced for user review)

| # | Proposal | Evidence | Recurrence | Suggested diff |
|---|---|---|---|---|
| B3 | Add `UNSTABLE` row to Phase 2.2 mergeStateStatus table with adr-compliance bypass guidance | v1-quality-r3-retro.md §2.1 carry-forward + §4 BM-task; v1-quality-r3b retro per-role BM signal | **2×** (one more = auto-apply threshold) | see B3 below |

### B3 — Add UNSTABLE row to Phase 2.2 table (detailed diff)

**File:** `.claude/commands/bm/bm-merge.md`
**Location:** Phase 2.2, mergeStateStatus table (currently line ~217)

**Current table:**
```markdown
| `mergeStateStatus` | Action |
|---|---|
| `CLEAN` | OK |
| `BEHIND` | STOP: "Branch behind base — rebase first via separate ASK" |
| `BLOCKED` | STOP: "Branch protection blocks merge — check settings" |
| `DIRTY` | STOP: "Merge conflicts — resolve on phase branch first" |
| `UNKNOWN` | RETRY in 10s (GitHub mid-compute), then STOP if still UNKNOWN |
| `HAS_HOOKS` | OK |
| Other | STOP: "Unexpected merge state — investigate" |
```

**Proposed addition (insert after `HAS_HOOKS` row):**
```markdown
| `UNSTABLE` | STOP: check which CI check is non-CLEAN. If the failing check is `adr-compliance` AND `scripts/brehon/check-adr-compliance.sh` (or equivalent local scan) exits 0 → proceed with `gh pr merge --admin` after advisor notes the bypass in DQ; otherwise STOP and wait for CI to resolve. |
```

**Rationale:** UNSTABLE = GitHub knows about non-required-but-failing checks. The most common case in this repo is the `adr-compliance` workflow_dispatch re-trigger failure (unreliable in private repos). The local scan is the authoritative check; passing locally with an `--admin` bypass is the documented path (used in quality-r3). The spec's "Other → STOP: Unexpected" is safe fallback but costs ~20 min investigation per occurrence when the actual resolution is known.

**User decision:** ☐ Apply  ☐ Modify (note: ____)  ☐ Decline

---

## Class C — Watch entries

**Adding to MEMORY.md "Watch / promote-if-recurs":**

> `WATCH (2x): bm-merge Phase 2.2 UNSTABLE → missing spec row for adr-compliance bypass path. Promote to B3 auto-apply if 3rd occurrence.`

This is a new entry (not already in MEMORY.md watch section).

---

## Spec-vintage signal

| Signal | Value |
|---|---|
| Spec last modified | 2026-05-22 (`e66cf272b`) |
| Age in days | 9 |
| Vintage class | Recent (< 14d) — still in the post-incident hardening window |
| Interpretation | Prior retro's B1+B2 applied and validated across 2 clean invocations. No regression. UNSTABLE gap is an additive finding, not a regression from the recent edit. |

---

## Decisions to revisit

- **B3 at 3rd UNSTABLE occurrence** — if quality-r3c or RT-r5 produces another UNSTABLE mergeStateStatus, apply B3 directly without another retro cycle (recurrence threshold reached).
- **`--admin` bypass guidance location** — currently lives in bm-merge briefs (advisor-side) not in the spec itself. B3 would add the guidance to the spec, making it available to any BM dispatch regardless of brief author. This is the same "brief template → spec body" promotion pattern as B2.
- **Sweep: other BM verbs with UNSTABLE exposure** — `bm-pr`, `bm-poll-cr` also interact with GitHub PR state. A future `/command-retro bm-pr` should check if UNSTABLE appears in those tables.

---

_Generated by `.claude/skills/command-retro/SKILL.md`. Lessons consulted:
`feedback_retro_not_report.md`, `feedback_four_role_retro_signals.md`,
`feedback_principles_not_rules.md`. Calibration:
`.claude/rules/evaluation-calibration.md`._
