---
phase: v1-SL-d
role: impl-task
task: 4
brief_n: 1
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 4 e2e test #2 — no-sponsor path preserves v0 immediate-Decided — see .claude/PRPs/briefs/sl-d-impl-4.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 4 e2e test #2 no-sponsor Decided path`

## §2 Scope

In `crates/server/tests/e2e.rs`, anchor-insert test fn `submit_jury_vote_preserves_v0_decided_for_no_sponsor_target` AFTER Task 3's test fn body, INSIDE the same `mod v1_sl_d_fixtures` block.

**Anchor identification at task-start:**
```bash
grep -n 'submit_jury_vote_transitions_to_pending' crates/server/tests/e2e.rs | tail -1
```
Insert the new test fn after the closing `}` of that fn, before the closing `}` of `mod v1_sl_d_fixtures`.

**Test fn:**
```rust
#[tokio::test]
async fn submit_jury_vote_preserves_v0_decided_for_no_sponsor_target() -> LemmyResult<()>
```

**Setup:** bootstrap LemmyContext with Postgres testcontainer. Seed target person with ZERO active sureties (simplest: create target Person but no surety rows at all). Seed `moderation_case` in `InReview` status with `target_person_id = no-sponsor-target`, `severity = CaseSeverity::Moderate`. Seat a jury panel (mirror existing JM-c/SL-b test setup). Pre-seed N-1 jury votes for `ContentRemoval` (liability-bearing sanction).

**Drive:** invoke `submit_jury_vote` handler with the Nth (final) vote.

**Assert:**
- `response.case_decided == true`
- `response.decision == Some(JuryDecision::RemoveContent)` (or matching variant for ContentRemoval)
- `moderation_case.status == CaseStatus::Decided` (v0 path — NOT SponsorLiabilityPending)
- `moderation_case.grace_expires_at` IS NULL (no Pending transition)
- 0 `governance_log` rows with `entry_kind == "sponsor_liability_pending"` (no Pending transition)
- 1 `governance_log` row with `entry_kind == "case_decided"`
- 1 `governance_log` row with `entry_kind == "sanction_created"`
- 1 `public_case_log` row (v0 immediate write fires on Decided path)
- juror `reputation_event` rows fire immediately (count == panel_size)
- reporter `reputation_event` row fires immediately (count == 1)
- `moderation_case.appeal_window_expires_at` is `Some(t)`

Do NOT add Tests #3-#4. ONE test fn per task.

### 2.1 §G4 CANONICAL CASE OVERRIDE — MANDATORY

**Case A (LemmyResult<()>) throughout `mod v1_sl_d_fixtures`.** All helper signatures `-> LemmyResult<T>`. All test fn signatures `-> LemmyResult<()>`. Bare `?` propagation. NO `Box<dyn Error>`. NO `.map_err` bridges. Mirror `mod v1_sl_b_fixtures` (e2e.rs:11001-11924) shape verbatim.

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §10.5 (fixture mod shape), §10.6 (governance_log assertion pattern), §13 Task 4 (IMPLEMENT steps + GOTCHAs)
- `crates/server/tests/e2e.rs` — read Task 3's test fn (`submit_jury_vote_transitions_to_pending`) to find anchor insertion point; mirror its setup pattern for the new test
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A mandatory; `LemmyResult<()>` test fn, `LemmyResult<T>` helpers, bare `?`
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` requires `use diesel_async::AsyncConnection;` (already present from fix-impl-5; confirm before writing)
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — ONE anchor-Edit only; e2e.rs is now ~12,900+ lines
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect` in new code

## §3a Handover from prior cohort

Task 3 final clean commit: fix-impl-5 (added `use diesel_async::AsyncConnection` to mod use block).
DQ #195 result: `pass` — workspace-check clean.
`mod v1_sl_d_fixtures` now open at end of e2e.rs with Test #1 + helpers inside.

Key note from fix-impl-5: `use diesel_async::AsyncConnection;` is already in the mod use block — do NOT add it again.

## §4 Constraints

- **Files:** only `crates/server/tests/e2e.rs`. No other files.
- **ONE anchor-Edit** — insert after Task 3's test fn closing `}`, inside `mod v1_sl_d_fixtures`. No other edits.
- **Case A canonical shape** — `LemmyResult<()>` test fn, bare `?`. HARD constraint.
- **No new imports** — all needed imports already present from Task 3 + fix-impl-5.
- **Shape G:** after committing, push the worker branch; write a `kind: "validate-pending"` DQ entry with `workflow_run_id` from the triggered `cargo-validate-workspace.yml` run.
- **DQ mid-task push:** if a DQ blocker is filed, commit + push immediately per decision-queue.md.
- **COMMIT MESSAGE:** `test(v1-SL-d): e2e test #2 — no-sponsor path preserves v0 immediate-Decided (task 4)`
