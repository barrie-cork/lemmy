# auto-phase handover — m3-core-entry-kinds (auto-refreshed)

**Refreshed:** 2026-06-17T22:40:00Z · stage `bm-cut-running`

## RESUME block (self-contained)
- **Sub-phase:** `m3-core-entry-kinds` (M3 town halls — Phase 2). Branch will be `phase-m3-core-entry-kinds`.
- **Stage:** `bm-cut-running` — Junior #688 (`[role:bm-task]` bm-cut) queued, cutting the phase branch from `governance-v0` @ `0438d2b64`.
- **Last commit on governance-v0:** `0438d2b64` — chore(advisor): bm-cut brief for m3-core-entry-kinds.
- **Plan:** `.claude/PRPs/plans/m3-core-entry-kinds.plan.md` (LOW complexity, SCHEMA, 4 tasks; 2 Junior `crates/` edits + 1 laptop validate + 1 advisor `.claude/` reconcile). No `[P]` markers → serial single-task cohorts.
- **Next concrete action (re-verify on resume):** poll #688 → on `done`, verify `phase-m3-core-entry-kinds` exists on origin → advance to `bm-cut-done` → author planning brief? **NO** — the plan already exists, so per the state machine bm-cut-done goes straight to authoring impl cohorts (skip planning). See note below.
- **DQ pending ids:** none.
- **Concurrent activity:** none (single canonical worktree; daemon 0 running before #688).

## NOTE — planning already done
The governing plan exists at `324d72d2f`/`0438d2b64`. This is NOT a fresh planning run.
After bm-cut completes, the state machine goes `bm-cut-done → impl-cohort-1` directly
(Task 1: db_schema consts), skipping the `planning-running` + gate-1 plan-approval stages
that apply only when a plan must be authored. **However** — the auto-phase state machine's
canonical path runs planning after bm-cut. Since the plan is pre-authored, the advisor will
run the DoD-smoke + watchpoint gate (gate 1) against the EXISTING plan before impl dispatch,
to honor the plan-approval user gate. Do not skip gate 1.
