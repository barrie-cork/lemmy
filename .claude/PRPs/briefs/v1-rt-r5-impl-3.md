# Brief: v1-RT-r5 Task 3 — rollup cron registration in `scheduled_tasks.rs`

## 1. Role + dispatch line

`[role:impl-task]` v1-rt-r5-task-3-rollup-cron-scheduled-tasks — see `.claude/PRPs/briefs/v1-rt-r5-impl-3.md`

## 2. Scope

**Produce:** add `ROLLUP_CRON_RUNNING` AtomicBool + `RollupCronRunningGuard` + `reputation_rollup_cron` clokwerk registration to `crates/routes/src/utils/scheduled_tasks.rs`.

**Exact deliverables:**
1. Module-scope guard (mirror `PARTICIPATION_CRON_RUNNING` at `:108-114`):
   ```rust
   static ROLLUP_CRON_RUNNING: AtomicBool = AtomicBool::new(false);
   struct RollupCronRunningGuard;
   impl Drop for RollupCronRunningGuard {
       fn drop(&mut self) { ROLLUP_CRON_RUNNING.store(false, Ordering::Release); }
   }
   ```
2. Registration block (mirror participation `:471-516`) placed **AFTER** the participation block and **BEFORE** the run loop at `:518`. Env-disable: `BREHON_DISABLE_ROLLUP_JOB`. Cadence from `job.rollup_interval_days` (default 7, `.max(1)`).
3. One commit on the task branch covering only this file.
4. One `validate-pending-laptop` DQ entry: `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`; then **STOP** — do NOT run cargo yourself.

**Do NOT touch:** `crates/api/**`, `crates/db_schema/**`, `crates/server/tests/e2e.rs`, any migration, any plan or PRD file.

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r5.plan.md` §10.3 (cron guard + registration mirror with full code snippet)
- `.claude/PRPs/plans/v1-RT-r5.plan.md` §13 Task 3 (ACTION, FILES, IMPLEMENT, MIRROR, GOTCHA, VALIDATE)
- `.claude/lessons/feedback_features_full_workspace_only.md` — always `--workspace --features full`, never `-p <crate> --features full`
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — `-p lemmy_routes --features full` silently fails; workspace only
- `.claude/lessons/feedback_clippy_test_style.md` — R1: `i64::from(...)` not `as` casts

**Mandatory MIRROR reads (read these BEFORE writing):**
- `crates/routes/src/utils/scheduled_tasks.rs:105-114` — participation guard (exact module-scope pattern to copy)
- `crates/routes/src/utils/scheduled_tasks.rs:471-518` — participation registration + run loop position (placement constraint: AFTER participation block, BEFORE `:518`)

## 4. Constraints

- **validate-pending-laptop DQ discipline:** after committing, write a `kind: "validate-pending-laptop"` DQ entry with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`; commit + push the DQ change; then **STOP**. Do NOT run cargo-check yourself. The advisor runs it locally.
- **Placement GOTCHA:** the registration block MUST come BEFORE the `loop { scheduler.run_pending().await; ... }` run loop at `:518`. A cron registered after the run loop never fires.
- **Module-scope guard MUST be at module scope** (not inside a function). Mirror the `PARTICIPATION_CRON_RUNNING` static exactly (same `AtomicBool`, `Ordering::Release` in Drop).
- **Env-disable variable name:** `BREHON_DISABLE_ROLLUP_JOB` (matches plan §10.3).
- **run_rollup_batch call path:** `lemmy_api::governance::reputation_snapshot::run_rollup_batch(&context)` (Task 1 added this function).
- **Single file, single commit.** Commit message: `feat(reputation): Task 3 — rollup cron guard + registration in scheduled_tasks (v1-RT-r5)`.
- **DQ attribution:** `from: "impl"`, `answered_by: null`. Never write `answered_by: "advisor"`.
- **Mid-task push:** push the DQ commit immediately after writing it (do not wait for finalize).
