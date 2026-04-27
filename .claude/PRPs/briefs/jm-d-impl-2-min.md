[role:impl-task] jm-d-impl-2-min — minimal ModerationCase struct sync (1 field add, no business logic)

## 1. Scope

Add the single missing column from `moderation_case` schema to the Rust `ModerationCase` struct + `ModerationCaseInsertForm` struct, so `cargo test --test e2e --workspace --features full --no-run` compiles. This unblocks runtime validation of the #99 deadlock fix already on `phase-v1-JM-d`.

**This task does NOT do:**
- Schema regen via `cargo run -p lemmy_diesel_utils -- print-schema` (the schema file is already correct).
- Any `AppealRequesterRole` enum / Appeal v1 columns / `JuryAssignmentInsertForm` `role` field work — that's all parked under the full JM-d task 2 plan §13.
- Any R3 sweep on `JuryAssignmentInsertForm`, `AppealInsertForm`, or other InsertForm call sites.
- Any `submit_jury_vote.rs` edit to write `winning_decision` to the column. The column stays unwritten in this task. That write belongs to JM-d task 3.
- Any e2e.rs touch. **Hard refusal:** do not Edit `crates/server/tests/e2e.rs` from this task. The 8945-line file has hung Junior workers twice (PMD `feedback_junior_worker_e2e_edit_hang`).

The full JM-d task 2 work is parked under `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §13 Task 2 and stays parked. This task is a **drift stub** in the sense of PMD #14 (`feedback_build_what_tests_exercise`): the struct field is added so tests can compile and read the column back, but the column is not written by current code paths and tests will see NULL — which is correct for the #99 deadlock-fix validation, since that fix doesn't depend on `winning_decision` being non-NULL.

## 2. Files to edit (exhaustive — only these)

- `crates/db_schema/src/source/governance/moderation_case.rs` — 2 edits:
  1. Line 5 `use lemmy_db_schema_file::{...enums::{CaseSeverity, CaseStatus, CaseStatusTier, CaseTargetType, SeverityTier}};` → add `JuryDecision` to the `enums::{...}` import list (alphabetical or after `CaseTargetType` per existing style).
  2. After the existing `pub appeal_window_expires_at: Option<DateTime<Utc>>,` field on `ModerationCase` (around line ~76, before the closing `}` of the struct), add: `pub winning_decision: Option<JuryDecision>,` with a `///` doc comment one-liner: `v1-JM-d: jury winning decision frozen at submit_jury_vote time. NULL until JM-d task 3 ships the write — current code reads back NULL.`
  3. Mirror the same field addition in `ModerationCaseInsertForm` after the existing `pub appeal_window_expires_at: Option<DateTime<Utc>>,` line (no doc comment needed; the InsertForm fields don't have per-field doc comments in the existing pattern).

That's it. Nothing else in this file changes.

## 3. Required reading

Read these in order before writing the edit:

1. `crates/db_schema/src/source/governance/moderation_case.rs` — full file. Pay attention to field ORDER on `ModerationCase` (must match schema column order — Diesel `Queryable` is positional).
2. `crates/db_schema_file/src/schema.rs` line 771–797 (the `moderation_case` `diesel::table!` block) — confirm `winning_decision -> Nullable<JuryDecision>` is the LAST column (it is, position 25).
3. `crates/db_schema/src/source/governance/jury_assignment.rs` line 1–20 — sibling pattern for `JuryDecision` enum import path (uses `enums::{JuryAssignmentRole, JuryAssignmentStatus}` — `JuryDecision` lives in the same module).
4. `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §10.2 (lines around 254 — the doc-comment shape for `winning_decision` on `ModerationCase`). Use the simpler one-liner above; full doc-comment lands when the full task 2 ships.
5. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — confirms the e2e.rs hard-refusal in §1.
6. `.claude/lessons/feedback_build_what_tests_exercise.md` — confirms the drift-stub pattern.

## 4. Constraints

- **e2e.rs hard refusal.** Do not Edit `crates/server/tests/e2e.rs` from this task. `Read` is fine if needed for verification, but no `Edit`. If a downstream cascade error appears in e2e.rs after the struct extension, surface via DQ pending entry — do not attempt to fix.
- **Field order matters.** `winning_decision` must be the last field on `ModerationCase` (matching schema column order). Same on `ModerationCaseInsertForm` (after `appeal_window_expires_at`).
- **No business logic.** The column gets added to the struct. No code anywhere writes to it. The `submit_jury_vote.rs` change in plan §13 Task 3 stays parked.
- **No schema regen.** `crates/db_schema_file/src/schema.rs` is already correct (line 796 has the column). Do not re-run `print-schema`.
- **MIRROR-ref discipline.** `JuryDecision` import path mirrors `crates/db_schema/src/source/governance/jury_assignment.rs:1-5` — match that style. The doc-comment shape on the new field mirrors the `appeal_window_expires_at` doc-comment (just shorter).
- **Decision-queue mid-task push.** Per `.claude/rules/decision-queue.md`: if anything in §2 or §3 turns out to be wrong (schema column not actually last, doc-comment shape doesn't match, JuryDecision import path differs from sibling), file a DQ pending entry from `impl-task` rather than guessing.
- **File-ownership boundary.** This task touches exactly one file (`moderation_case.rs`). Any other crate touch requires a DQ entry first.
- **Forbidden window.** This brief is being authored at 20:48 UTC 2026-04-27 — clear of all forbidden windows. The next forbidden window starts 02:55 UTC. Task should complete well within that.

## 5. Validation gates (DoD)

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > /tmp/jm-d-impl-2-min-check.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-impl-2-min-check.log
# Expected: exit 0

bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/jm-d-impl-2-min-clippy.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-impl-2-min-clippy.log
# Expected: exit 0

bash scripts/brehon/cargo-test.sh --test e2e --no-run -p lemmy_server > /tmp/jm-d-impl-2-min-test-no-run.log 2>&1
echo "exit: $?"; tail -10 /tmp/jm-d-impl-2-min-test-no-run.log
# Expected: exit 0 — this is the gate that DQ #56 names. Without this exit 0, the task has not unblocked the deadlock-fix validation channel.
```

If `cargo test --test e2e --no-run` still emits E0277 errors after the field is added in correct position, **stop**. The cause is either field-ordering wrong or `JuryDecision` mapping mismatched in the import — file a DQ entry rather than retrying.

## 6. Commit message

```
fix(v1-JM-d): minimal ModerationCase struct sync — add winning_decision field

Adds `winning_decision: Option<JuryDecision>` to ModerationCase + ModerationCaseInsertForm
to match the schema's 25-column shape. Drift stub per PMD #14: column is unwritten by
current code paths; full JM-d task 2 (which writes the column from submit_jury_vote)
remains parked.

Unblocks `cargo test --test e2e --workspace --features full --no-run`, which is
required to runtime-validate the submit_jury_vote deadlock fix on `phase-v1-JM-d`
(commit 27b212e).

Resolves DQ #56 option (b).
```
