---
phase: v1-SL-d
role: impl-task
task: 6
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 6 e2e test #4 — wrapper preserves v0 outputs (compose-equivalence) — see .claude/PRPs/briefs/sl-d-impl-6.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 6 e2e test #4 wrapper preserves v0 outputs compose-equivalence`

## §2 Scope

In `crates/server/tests/e2e.rs`, anchor-insert test fn `apply_sponsor_liability_wrapper_preserves_v0_outputs` AFTER Task 5's test fn body, INSIDE the same `mod v1_sl_d_fixtures` block. This is the LAST test in `mod v1_sl_d_fixtures`; the closing `}` of the mod follows immediately after.

**Anchor identification at task-start:**
```bash
grep -n 'submit_jury_vote_no_action_skips_liability_machinery' crates/server/tests/e2e.rs | tail -1
```
Insert the new test fn after the closing `}` of that fn, before the closing `}` of `mod v1_sl_d_fixtures`.

**Test fn:**
```rust
#[tokio::test]
async fn apply_sponsor_liability_wrapper_preserves_v0_outputs() -> LemmyResult<()>
```

**Setup:** bootstrap LemmyContext with Postgres testcontainer. Seed sponsee + 2 sponsors with active sureties. Seed `moderation_case` in `CaseStatus::SponsorLiabilityPending` with `grace_expires_at = now() - 1 minute` (expired). Seed a sanction row.

**Drive:** invoke `run_grace_check_batch(&context).await` — this fires `apply_sponsor_liability` (the thin wrapper) internally at `sponsor_liability_grace.rs:510`. Mirror the pattern already used in SL-c-2 tests at e2e.rs:11932.

**Note on visibility:** `apply_sponsor_liability` is `pub(crate)` and not directly callable from `crates/server/tests/`. Drive via `run_grace_check_batch` which is `pub` — established precedent in SL-c-2 fixtures.

**Assert:**
- `case.status == CaseStatus::SponsorLiabilityFired` (SL-c scheduler sets this after wrapper returns)
- 2 `reputation_event` rows for sponsors (ordered by sponsor_id ASC); each row: `dimension = EndorsementStrength`, `reason = "sponsor_liability_applied"`, `source_case_id = Some(case_id)`, delta = expected per-sponsor value computed from config seeds
- 2 `governance_log` rows with `entry_kind == "sponsor_liability_applied"`; each payload has: `sponsor_pseudonym` (string), `severity = "moderate"`, `pre_multiplier_delta`, `post_multiplier_delta`, `final_delta`, `multiplier` fields per the payload shape at `sponsor_liability.rs:322-337`
- 0 `governance_log` rows with `entry_kind == "sponsor_liability_clamped"` (no clamp engaged)
- 1 `governance_log` row with `entry_kind == "sponsor_liability_fired"` (SL-c summary)
- All 2 `_applied` payload fields BYTE-IDENTICAL to what the v0 single-pass body would have produced (re-compute expected from seed fixtures and compare)

Do NOT add Task 7's unit tests. ONE test fn per task.

### 2.1 §G4 CANONICAL CASE OVERRIDE — MANDATORY

**Case A (LemmyResult<()>) throughout `mod v1_sl_d_fixtures`.** All helper signatures `-> LemmyResult<T>`. All test fn signatures `-> LemmyResult<()>`. Bare `?` propagation. NO `Box<dyn Error>`. NO `.map_err` bridges. Mirror `mod v1_sl_b_fixtures` (e2e.rs:11001-11924) shape verbatim.

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §10.4 (governance_log payload schemas — sponsor_liability_applied / _clamped shapes), §10.5 (fixture mod shape), §13 Task 6 (IMPLEMENT steps + GOTCHAs)
- `crates/server/tests/e2e.rs:11932` — SL-c-2 fixture's `run_grace_check_batch` call pattern; mirror verbatim for setup + drive
- `crates/api/api/src/governance/sponsor_liability.rs:322-353` — v0 payload field set for `sponsor_liability_applied` entries; use to verify BYTE-IDENTICAL assertion
- `crates/api/api/src/governance/sponsor_liability_grace.rs:510` — where the scheduler calls `apply_sponsor_liability`; understand the Fire branch
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A mandatory; `LemmyResult<()>` test fn, `LemmyResult<T>` helpers, bare `?`
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` requires `use diesel_async::AsyncConnection;` (already present from fix-impl-5; confirm before writing)
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — ONE anchor-Edit only; e2e.rs is now ~13,500+ lines
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect` in new code

## §3a Handover from prior cohort

Task 5 final clean commit: `test(v1-SL-d): e2e test #3 — NoAction path skips liability machinery (task 5)`.
DQ #197 result: `pass` — workspace-check clean.
`mod v1_sl_d_fixtures` now contains Test #1 + Test #2 + Test #3 + helpers inside.

Key notes:
- `use diesel_async::AsyncConnection;` already in the mod use block — do NOT add again.
- This test is the LAST inside `mod v1_sl_d_fixtures`. The closing `}` of the mod follows immediately after `Ok(())`.

**GOTCHA (wrapper visibility):** `apply_sponsor_liability` is `pub(crate)` — not callable from `crates/server/tests/e2e.rs` directly. Drive via `run_grace_check_batch` instead (it's `pub` and SL-c-2 already uses this pattern).

**GOTCHA (config-driven delta):** re-compute expected delta from config seeds for the assertion. Helper `compute_expected_delta(...)` may be defined inline or as a fixture-mod helper.

**GOTCHA (closing the mod):** this test's `Ok(())` is the last line before `mod v1_sl_d_fixtures` closes. Ensure the closing `}` of the mod is preserved.

## §4 Constraints

- **Files:** only `crates/server/tests/e2e.rs`. No other files.
- **ONE anchor-Edit** — insert after Task 5's test fn closing `}`, inside `mod v1_sl_d_fixtures`. No other edits.
- **Case A canonical shape** — `LemmyResult<()>` test fn, bare `?`. HARD constraint.
- **No new imports** — all needed imports already present from Task 3 + fix-impl-5.
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
- **COMMIT MESSAGE:** `test(v1-SL-d): e2e test #4 — wrapper preserves v0 outputs (compose-equivalence) (task 6)`
