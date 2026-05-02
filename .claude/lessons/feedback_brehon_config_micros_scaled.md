---
name: brehon_config_micros_scaled
description: Brehon governance config values are micros-scaled and depend on reputation_snapshot — never assume integer counts when writing tests
type: feedback
---

Many Brehon governance config keys (e.g. `report.case_threshold_micros`,
`report.base_weight`, `report.clamp_min/max`, `report.recency_half_life_hours`)
are **micros-scaled** (× 1,000,000) and feed formulas that combine them with
**`reputation_snapshot` row values** for the calling user (e.g.
`reporting_accuracy`, `jury_reliability`, `participation_consistency`,
`endorsement_strength`).

A test that drives a handler exercising one of these formulas (`create_report`,
`reputation_snapshot::recompute_snapshot`, `sponsor_liability::*`, `jury` weight
math) cannot reason about counts of inputs — it has to compute the expected
weight in micros from the formula + config defaults + the seeded snapshot
values, and compare against the appropriate threshold.

**Why:** Phase 5b task 58 replaced the Phase 4 `V0_THRESHOLD = 3` /
`V0_REPORTER_WEIGHT = 1` constants with the OQ-006 config-driven formula.
Old test comments referencing "v0 threshold = 3 reports" are now stale —
the actual threshold is `report.case_threshold_micros` (default
`3_000_000`) compared against an **accumulated micros score**. Misreading
those comments as "3 reports = threshold" is the trap that hit Task 3 of
v1-JM-e (4 reports were needed because (a) default threshold = 3,000,000
and (b) the comparison is strict `>`, not `>=`).

**How to apply:**

When writing tests that drive these handlers:

1. **Seed `reputation_snapshot` for the calling user** with deterministic
   values (use `v1_jm_b_fixtures::seed_jury_eligible_snapshots` /
   `_scoped` — they set all four dimensions to 100). Without seeded
   snapshots, `load_or_compute_snapshot` falls back to `recompute_snapshot`
   which produces baseline values that vary with seed conditions and make
   weight math non-deterministic.

2. **Look up the relevant `DEFAULT_*` constant** in
   `crates/api/api/src/governance/config.rs` (e.g.
   `DEFAULT_REPORT_CASE_THRESHOLD_MICROS = 3_000_000`,
   `DEFAULT_REPORT_BASE_WEIGHT = 1.0`).

3. **Compute the per-call weight from the formula**, then divide threshold
   by per-call weight to determine how many calls are needed.
   `report.create_report` formula:
   `weight_micros = base_weight × clamp(accuracy/100, clamp_min, clamp_max) × exp(-hours_old / half_life) × 1_000_000`

4. **Mind strict `>` vs `>=`** — `create_report.rs:157` is
   `new_score > threshold_micros`, so accumulating exactly `threshold_micros`
   does NOT cross. Add one more increment to be safe.

5. **Or override the config** for the test: insert into
   `governance_config` with a low threshold value before driving the
   handler. The Phase 5a task 50 migration created
   `governance_config_current` view that picks the latest row per
   `(scope, key)`, so newer rows win.

**Generalises to:** any test that drives a Brehon governance handler with
config-driven math. Sanction-weight, reputation-event, sponsor-liability,
jury-threshold, and federation-decay all share this micros + config +
snapshot pattern. **Old V0_* constants in test comments are misleading
historical artifacts** — verify against current code before using them as
test parameters.

**Symptom to recognise:** a test asserting `resp.threshold_met` after
N reports panics with `assertion failed: resp.threshold_met` despite
the count "looking right" per old documentation. First check: was N
computed from `DEFAULT_REPORT_CASE_THRESHOLD_MICROS / 1_000_000` and
adjusted for strict `>`? Second check: did you seed `reputation_snapshot`
for the reporter so `accuracy = 100` and weight per call = 1,000,000?
