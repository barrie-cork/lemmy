---
phase: v1-SL-d
role: impl-task
task: 5
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 5 e2e test #3 — NoAction path skips liability machinery — see .claude/PRPs/briefs/sl-d-impl-5.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 5 e2e test #3 NoAction path skips liability machinery`

## §2 Scope

In `crates/server/tests/e2e.rs`, anchor-insert test fn `submit_jury_vote_no_action_skips_liability_machinery` AFTER Task 4's test fn body, INSIDE the same `mod v1_sl_d_fixtures` block.

**Anchor identification at task-start:**
```bash
grep -n 'submit_jury_vote_preserves_v0_decided_for_no_sponsor_target' crates/server/tests/e2e.rs | tail -1
```
Insert the new test fn after the closing `}` of that fn, before the closing `}` of `mod v1_sl_d_fixtures`.

**Test fn:**
```rust
#[tokio::test]
async fn submit_jury_vote_no_action_skips_liability_machinery() -> LemmyResult<()>
```

**Setup:** bootstrap LemmyContext with Postgres testcontainer. Seed sponsee + 2 sponsors with active sureties (mirror Test #1 setup). Seed `moderation_case` in `InReview` status. Seat a jury panel. Pre-seed N-1 jury votes for `JuryDecision::NoAction`.

**Drive:** invoke `submit_jury_vote` handler with the Nth (final) NoAction vote.

**Assert:**
- `response.case_decided == true`
- `response.decision == Some(JuryDecision::NoAction)`
- `moderation_case.status == CaseStatus::Decided` (NoAction = no liability; `map_decision_to_sanction` returns None; the entire `if let Some((scope, action)) = ...` block at lines 423-481 is skipped; `compute_sponsor_liability` is never called)
- `moderation_case.grace_expires_at` IS NULL (no Pending transition)
- 0 `governance_log` rows with `entry_kind == "sponsor_liability_pending"`
- 0 `governance_log` rows with `entry_kind == "sanction_created"` (NoAction writes no sanction row)
- 0 `reputation_event` rows for sponsors (liability machinery not called)
- 1 `governance_log` row with `entry_kind == "case_decided"` (still emitted on NoAction path per JM-c shipped behaviour)
- juror `reputation_event` rows fire (count == panel_size)
- reporter `reputation_event` row fires (count == 1)

Do NOT add Tests #4. ONE test fn per task.

### 2.1 §G4 CANONICAL CASE OVERRIDE — MANDATORY

**Case A (LemmyResult<()>) throughout `mod v1_sl_d_fixtures`.** All helper signatures `-> LemmyResult<T>`. All test fn signatures `-> LemmyResult<()>`. Bare `?` propagation. NO `Box<dyn Error>`. NO `.map_err` bridges. Mirror `mod v1_sl_b_fixtures` (e2e.rs:11001-11924) shape verbatim.

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §10.5 (fixture mod shape), §10.6 (governance_log assertion pattern), §13 Task 5 (IMPLEMENT steps + GOTCHAs)
- `crates/server/tests/e2e.rs` — read Task 4's test fn (`submit_jury_vote_preserves_v0_decided_for_no_sponsor_target`) to find anchor insertion point; mirror its setup pattern for the new test
- `crates/api/api/src/governance/submit_jury_vote.rs:423` — `map_decision_to_sanction` gate; NoAction → None → block skipped; `compute_sponsor_liability` never called
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A mandatory; `LemmyResult<()>` test fn, `LemmyResult<T>` helpers, bare `?`
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` requires `use diesel_async::AsyncConnection;` (already present from fix-impl-5; confirm before writing)
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — ONE anchor-Edit only; e2e.rs is now ~13,100+ lines
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect` in new code

## §3a Handover from prior cohort

Task 4 final clean commit: `test(v1-SL-d): e2e test #2 — no-sponsor path preserves v0 immediate-Decided (task 4)`.
DQ #196 result: `pass` — workspace-check clean.
`mod v1_sl_d_fixtures` now contains Test #1 (transitions_to_pending) + Test #2 (no-sponsor Decided) + helpers.

Key note: `use diesel_async::AsyncConnection;` is already in the mod use block — do NOT add it again.

**GOTCHA (NoAction short-circuit):** at `submit_jury_vote.rs:423`, `if let Some((scope, action)) = map_decision_to_sanction(winning_decision)` is the gate. NoAction → `map_decision_to_sanction` returns None → the block is skipped. `compute_sponsor_liability` never runs. SL-d's mutation does NOT change this gate. Test #3 asserts this.

**GOTCHA (case_decided log on NoAction):** the `case_decided` log at `submit_jury_vote.rs:649-659` fires unconditionally (path-agnostic). NoAction path still emits it. Assert exactly 1 `case_decided` row.

## §4 Constraints

- **Files:** only `crates/server/tests/e2e.rs`. No other files.
- **ONE anchor-Edit** — insert after Task 4's test fn closing `}`, inside `mod v1_sl_d_fixtures`. No other edits.
- **Case A canonical shape** — `LemmyResult<()>` test fn, bare `?`. HARD constraint.
- **No new imports** — all needed imports already present from Task 3 + fix-impl-5.
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
- **COMMIT MESSAGE:** `test(v1-SL-d): e2e test #3 — NoAction path skips liability machinery (task 5)`
