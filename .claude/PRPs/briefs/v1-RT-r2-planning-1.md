# Brief: planning — v1-RT-r2

## 1. Role + dispatch

`[role:planning] v1-RT-r2 — plan per-dimension chained-halving decay — see .claude/PRPs/briefs/v1-RT-r2-planning-1.md`

## 2. Scope

Plan the implementation of **v1-RT-r2**: per-dimension chained-halving reputation decay, behind the existing `feature.reputation_v1_decay_enabled` feature flag.

Per `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §11 phase 2:

> "Replace `compute_applied_delta` with per-dimension/per-direction calculator behind feature flag; add bounds clamping to `recompute_snapshot`."

**Primary target file:** `crates/api/api/src/governance/reputation_snapshot.rs`

The existing `compute_applied_delta` (line ~354) applies a single half-life halving. The TODO comment at that location reads:
> "v1 — switch to chained halving per half-life elapsed and add a regression test for age > 2× half-life."

The existing feature flag `feature.reputation_v1_decay_enabled` is already seeded in `governance_config` (from v1-RT-r1) at `crates/api/api/src/governance/config.rs:1279`.

**What the plan must produce:**

1. A revised `compute_applied_delta` (or replacement) that:
   - When `feature.reputation_v1_decay_enabled = true`: applies chained halving using **per-dimension/per-direction half-life** — `recompute_snapshot` reads 8 config keys (`decay.<dim>.positive_half_life_days` / `decay.<dim>.negative_half_life_days` for each of 4 dimensions, all seeded by r1 at config.rs:1137-1158) and passes the resolved `Duration` values into the calculator. For each complete half-life elapsed, halves the delta again (age=2×half-life → delta/4, age=3×half-life → delta/8). (Per clarify DQ a3d0e9941441-010 — PRD §5.2 + config.rs:1137-1158.)
   - When `feature.reputation_v1_decay_enabled = false`: preserves the existing single-halving behaviour using the legacy `decay.positive_half_life_days` key (no regression for existing deployments)
   - Penalty path (`original <= 0`) unchanged (penalties never decay)
   - Founder-seed path (`event.expires_at.is_some()`) unchanged (no decay on cliff-dated seeds)

2. Bounds clamping in `recompute_snapshot` (line ~213): after summing deltas per dimension, clamp each dimension's accumulated score to `[bounds.<dim>.floor, bounds.<dim>.ceiling]` using the per-dimension governance_config keys already seeded by r1 (config.rs:1161-1172). For example, `reporting_accuracy` is clamped to `[DEFAULT_BOUNDS_REPORTING_ACCURACY_FLOOR, DEFAULT_BOUNDS_REPORTING_ACCURACY_CEILING]`. Negative floors are valid — do NOT use `[0, i32::MAX]`. (Per clarify DQ a3d0e9941441-009 — PRD §5.2 + config.rs:1161-1172.)

3. Unit tests covering:
   - age = 0 (no decay)
   - age = 1×half-life (halved once — same as current)
   - age = 2×half-life (halved twice — new behaviour)
   - age = 3×half-life (halved three times)
   - feature flag = false path (single-halving preserved)
   - bounds clamp (sum overflow → clamped)

**Out of scope for this plan:**
- No new DB migrations (r1 owns the schema)
- No new governance_config keys (feature flag already seeded)
- No cron/scheduler changes (those are r3)
- No new REST endpoints
- No changes to `recompute_snapshot`'s caller signature or the 5 callsites in `create_endorsement.rs`, `revoke_endorsement.rs`, `e2e.rs`

## 3. Required reading

- `.claude/PRPs/prds/v1-reputation-tuning.prd.md` §5 (proposed solution) + §11 (r2 row) + §8 (defaults matrix — half-life defaults)
- `crates/api/api/src/governance/reputation_snapshot.rs` (full file — understand `recompute_snapshot`, `compute_applied_delta`, and the existing test suite at bottom of file)
- `crates/api/api/src/governance/config.rs` lines ~1270-1290 (feature flag default constant `DEFAULT_FEATURE_REPUTATION_V1_DECAY_ENABLED`)
- `.claude/lessons/feedback_clippy_test_style.md` — Lemmy test style (no `unwrap`/`expect`; `LemmyResult<()>` with `?`)
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — every plan §4 watchpoint must cite specific file:line
- `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` — DoD commands must be executable as written
- `.claude/PRPs/plans/v1-RT-r1.plan.md` — prior phase plan for context (task structure, MIRROR refs used, e2e patterns)
- `.claude/rules/advisor-orchestrator.md` §2.4 — mandatory file-class lesson injection table
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML (`creates:` + `modifies:`) required on every §13 task; planner must include these arrays for the brehon-verify gate and cohort-dispatch overlap check

## 4. Constraints

- **No new migrations** — r2 is compute-logic only; any proposed migration is a scope violation.
- **Feature flag must gate the new behaviour** — `feature.reputation_v1_decay_enabled` is the existing key; read it via the `config` cache already passed to `recompute_snapshot`.
- **Preserve all existing callsite signatures** — `recompute_snapshot` and `compute_applied_delta` signatures may change internally but the 5 external callers must not require edits (or if they do, enumerate all 5 in the plan with file:line).
- **DQ discipline:** write `kind: "validate-pending-laptop"` DQ entry after impl-task pushes its worker branch (Shape G suspended per DQ #229; cargo runs on laptop).
- **Mandatory lesson injection per §2.4:** any edit to `crates/api/api/src/governance/reputation_snapshot.rs` that adds or modifies tests → inject `feedback_clippy_test_style.md`.
- **MIRROR ref discipline:** cite existing sibling unit tests in the same file as MIRROR refs for new tests (the existing `assert_eq!(compute_applied_delta(...))` tests at lines ~936-957 are the canonical shape).
- **Commit attribution:** impl-task commits use `feat(rep-tuning): <description> (task N)` subject.
- **Plan §16a stories:** include at least 2 stories — (1) chained-halving logic correct, (2) feature-flag false path preserves prior behaviour.
