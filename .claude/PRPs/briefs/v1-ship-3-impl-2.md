# Brief: v1-ship-3 impl-task 2 — POST /report returns GovernanceCaseSummaryView

## 1. Role + dispatch line

`[role:impl-task]` v1-ship-3 Task 2 — POST /report returns GovernanceCaseSummaryView

## 2. Scope

Reshape `POST /governance/report` so its response carries a `GovernanceCaseSummaryView` of
the newly-opened or appended case. Four files, one commit.

**Produce:**
- `read_summary_for_case` helper appended to `crates/db_views/governance_case/src/impls.rs`
- `case: GovernanceCaseSummaryView` field added to `CreateGovernanceReportResponse` in
  `crates/api/api_common/src/governance.rs`
- `ProcessReportOutcome` internal struct + post-tx view fetch wired in
  `crates/api/api_crud/src/governance/create_report.rs`
- One new `assert_eq!` on `create_resp.case.case_id` in `report_to_modlog_golden_path`
  in `crates/server/tests/e2e.rs` (around line 2661)

**Do NOT:**
- Touch any file outside the four listed above
- Add new dependencies (`lemmy_db_views_governance_case` is already in api_common Cargo.toml)
- Add new migrations, new routes, new DTOs beyond the single field
- Introduce a new `LemmyErrorType` variant (use `CouldntFindObject` or closest existing)
- Commit to governance-v0 or any branch other than the task worktree branch

## 3. Required reading (in order)

Read these files before the first Edit:

**Schema / type definitions:**
- `crates/api/api_common/src/governance.rs:51-89` — `CreateGovernanceReportResponse` (line 56)
  + `ListGovernanceCasesResponse` (line 87, the read-surface mirror)
- `crates/db_views/governance_case/src/lib.rs:35-59` — `GovernanceCaseSummaryView` struct

**MIRROR refs (read these and implement to match their shape exactly):**
- `crates/db_views/governance_case/src/impls.rs:51-90` — `list_open_cases_for_community`
  (the two-round-trip pattern `read_summary_for_case` mirrors; same SELECT tuple, same
  `.left_join(...on...)`, same `submitted_counts_by_case(&[r.0])`, same `build_summary`)
- `crates/db_views/governance_case/src/impls.rs:243-294` — `submitted_counts_by_case` +
  `build_summary` helpers (reuse, do not re-implement)
- `crates/db_views/governance_case/src/impls.rs:312-355` — `read_case_detail`
  (demonstrates `.first().optional()` for the "may not exist" pattern)
- `crates/api/api_crud/src/governance/create_report.rs:61-171` — `create_report` handler
  body (the reshape target)
- `crates/api/api_crud/src/governance/create_report.rs:177-329` — `process_report` body
  (refactor: return type changes from `CreateGovernanceReportResponse` to `ProcessReportOutcome`)
- `crates/server/tests/e2e.rs:2485-2666` — `report_to_modlog_golden_path` (assertion update
  target; new assert_eq! goes around line 2661)

**Lessons (mandatory):**
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — handler stays `LemmyResult<Json<...>>`;
  helper stays `LemmyResult<Option<...>>`; no `Box<dyn Error>` introduced (no test fn is changed
  to return Box<dyn Error>)
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` — run
  `rg "CreateGovernanceReportResponse" crates/ tests/` before any edit and confirm output
  matches exactly the 5 lines in the plan §11 table; if a 6th line appears, STOP and file
  `kind: "blocker"` DQ

## 4. Constraints

1. **R11 callsite gate (mandatory first step):** before the first Edit, run
   `rg "CreateGovernanceReportResponse" crates/ tests/ 2>/dev/null`. Expected: ≤5 lines
   (definition + import + 3 usages in create_report.rs). If >5, STOP — file blocker DQ.

2. **read_summary_for_case shape:** verbatim from plan §10.1. Two-round-trip pattern:
   main query produces `Option<SummaryRow>`; `submitted_counts_by_case` aggregates for
   `&[r.0]`; `build_summary` maps. Return type: `LemmyResult<Option<GovernanceCaseSummaryView>>`.
   Signature: `pub async fn read_summary_for_case(pool: &mut DbPool<'_>, case_id: ModerationCaseId)`.

3. **ProcessReportOutcome struct:** private to `create_report.rs`, placed just above
   `process_report`. Fields: `case_id: ModerationCaseId`, `threshold_met: bool`.
   (Note: `case_id` is typed `ModerationCaseId`, not `Option<ModerationCaseId>` — the outer
   handler wraps it in `Some(...)` when building the final response.)

4. **Post-tx connection:** acquire via `&mut context.pool()` — do NOT reuse the closed-tx
   connection from `run_transaction`. Use `read_summary_for_case(&mut context.pool(), outcome.case_id).await?`.

5. **None handling:** `read_summary_for_case` returns `Option<T>`. Convert with
   `.ok_or(LemmyErrorType::CouldntFindObject)?` (or the closest variant — check
   `lemmy_utils::error::LemmyErrorType` enum at task-time). Do NOT unwrap.

6. **e2e assertion:** insert AFTER `let case_id = create_resp.case_id.expect("case_id present");`
   (line ~2661). New line:
   ```rust
   assert_eq!(
     create_resp.case.case_id, case_id.0,
     "POST /report response carries the case summary view with matching case_id"
   );
   ```
   Test fn return type (`LemmyResult<()>`) and signature are UNCHANGED. The new assert uses
   `assert_eq!` (panics on fail) — no `?` propagation needed.

7. **validate-pending-laptop DQ:** after committing + pushing, write a `kind: "validate-pending-laptop"`
   DQ entry with:
   ```json
   {
     "commands": [
       "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-3-task2-check.log 2>&1\"",
       "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-3-task2-clippy.log 2>&1\"",
       "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-3-task2-test-no-run.log 2>&1\"",
       "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full report_to_modlog_golden_path > .claude/PRPs/debug/v1-ship-3-task2-e2e.log 2>&1\""
     ],
     "branch": "<your worktree branch>",
     "phase_task": 2
   }
   ```
   Push the DQ entry immediately after writing it (mid-task push per `decision-queue.md`).

8. **Commit message:** `feat(api): POST /report returns governance case summary view (task 2)`

9. **One commit only.** All four file edits in a single commit. No partial commits.

## 3a. Handover from prior cohort

Task 1 (`docker/docker-compose.yml`) was completed and cherry-picked to `phase-v1-ship-3`
at commit `8cd5b9da1` by the advisor. That file is not in scope for Task 2.

Base branch `phase-v1-ship-3` is at `8cd5b9da1` as of task dispatch time.
