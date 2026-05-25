# Brief: impl-task 1 — v1-RT-r3 participation cron module + scheduler tick

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 task 1 — participation cron module + scheduler tick — see .claude/PRPs/briefs/v1-RT-r3-impl-1.md`

## 2. Scope

Implement §13 Task 1 of `.claude/PRPs/plans/v1-RT-r3.plan.md` verbatim.

**FILES (per plan §13 Task 1 FILES yaml):**

- creates: `crates/api/api/src/governance/participation_cron.rs` (Sources 1 + 2 cron module)
- modifies: `crates/api/api/src/governance/mod.rs` (add `pub mod participation_cron`)
- modifies: `crates/routes/src/utils/scheduled_tasks.rs` (new RunningGuard + scheduler tick block)
- requires: [] (Task 0 already done; no inter-task dependencies)

**IMPLEMENT:** Follow plan §13 Task 1 IMPLEMENT blocks verbatim (3 files). Mirror:
- `crates/api/api/src/governance/appeal_window_expiry.rs` for cron helper module shape
- `crates/api/api/src/governance/sponsor_liability_grace.rs:115-185` for per-row run_transaction pattern + per-batch outer error policy (catch per-community errors with `tracing::warn`; outer fn returns `Ok(...)` regardless)
- `crates/routes/src/utils/scheduled_tasks.rs:201-255` for RunningGuard + scheduler tick block

**VALIDATE (story-checkpoint feeds §16a Stories 1 + 2):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task1-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-task1-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-task1-clippy.log
# EXPECT: exit 0
```

**Note:** Shape G is SUSPENDED per DQ #229 — cargo commands run on the laptop, not the Junior worker. Junior subagent must NOT execute the VALIDATE block locally. Instead, after the code edits + commit + push:

**Post-validate:** Write `kind: "validate-pending-laptop"` DQ entry per `advisor-orchestrator.md` §5.2.
Required fields:
- `commands`: the 2 VALIDATE bash blocks above (verbatim including `--workspace --features full` + `tail -20|-40`)
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: `1`

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push DQ to worker branch.

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r3.plan.md` §13 Task 1 (authoritative IMPLEMENT + MIRROR + GOTCHA + VALIDATE)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §10.1 (cron module shape verbatim)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §10.8 (scheduler tick block verbatim)
- `crates/api/api/src/governance/sponsor_liability_grace.rs:115-185` (run_transaction pattern + per-community error policy)
- `crates/api/api/src/governance/appeal_window_expiry.rs` (canonical cron helper shape)
- `crates/routes/src/utils/scheduled_tasks.rs:201-255` (existing RunningGuard pattern)
- `crates/routes/src/utils/scheduled_tasks.rs:163` (`all_active_counts` as query design reference per DQ a3d0e9941441-023)
- `crates/db_schema_file/src/schema.rs:201-227` (comment table field names)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — both crons wrap each per-community batch in `run_transaction`
- `.claude/lessons/feedback_features_full_workspace_only.md` + `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — every cargo command uses `--workspace --features full`, NEVER `-p <crate> --features full`
- `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — Lemmy workspace test-style; clippy denies unwrap/expect/allow_attributes
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — Shape G suspended; cargo runs on laptop via `kind: "validate-pending-laptop"`
- `.claude/rules/decision-queue.md` (DQ schema-v3; impl-task writes `kind: "validate-pending-laptop"` per §"Polling-loop routing per kind"; Junior never writes `answered_by: "advisor"` per Hard refusal #1; ALWAYS use `dq-v3-new-entry.sh` per Hard refusal #2/#9)

## 4. Constraints

- **No edits outside the FILES yaml block above.** Touching any other file is a process breach.
- **One commit per logical unit** — group the 3 file edits into ONE commit with subject `feat(governance): participation_cron module + scheduler tick (task 1)`.
- **No clippy `#[allow]` to silence warnings** — Lemmy denies `clippy::allow_attributes`. Use `#[expect(...)]` if absolutely needed AND only after verifying the lint is intentional (per `feedback_clippy_test_style.md`).
- **Use Diesel `.on_conflict_do_nothing()` (untargeted form)** for `reputation_event` inserts — partial unique index requires untargeted form, NOT `.on_conflict(reputation_event::dedupe_key).do_nothing()` (per plan §13 Task 1 GOTCHA + `feedback_postgres_jsonb_canonicalization.md` co-located on the migration).
- **ISO week format:** `format!("{}-W{:02}", iso.year(), iso.week())` from `chrono::Datelike::iso_week` per plan §13 Task 1 GOTCHA.
- **Env-var check FIRST in tick closure** (BEFORE guard acquisition) — mirror `scheduled_tasks.rs:320-325` (sponsor-liability-grace block).
- **Outer fn returns Ok(...) regardless of per-community errors** — mirror `sponsor_liability_grace.rs:165-176`. Per-community error → `tracing::warn` + continue, never `?` out of the outer.
- **No new Cargo.toml deps** — `lemmy_routes` already depends on `lemmy_api`.
- **Mid-task DQ push** — if you raise `kind: "blocker"` (e.g. unexpected sibling fixture shape, missing config key), commit + push the DQ immediately on the worker branch per `decision-queue.md` §"Mid-task visibility".
- **Pre-push cargo-check discipline NOT required for this task** — VALIDATE runs on laptop via validate-pending-laptop, not worker.
- **LESSON-trailer convention** — if you discover a durable pattern during impl, end commit body with `LESSON: <one-line observation>` per `.claude/rules/pmd-invariants.md` invariant #4.

## 5. Forbidden-window check (advisor pre-queue)

Per `advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window check is binding for the laptop validate-pending-laptop cargo runs. Advisor checks at queue time AND at DQ mutation time. Junior worker itself is `claude -p` (low memory) so daemon-side forbidden-window check is non-binding for the worker dispatch.

Current UTC at queue: Mon 19:08 UTC (primary window 16:00–02:30 — OK).

## 6. Context

- Phase: v1-RT-r3
- Plan: `.claude/PRPs/plans/v1-RT-r3.plan.md` (on trunk + phase branch)
- Phase branch: `phase-v1-RT-r3` @ `0401709e0` (post-DQ-1b8527b076d4-001-answered)
- Base branch for this task: `phase-v1-RT-r3`
- Cohort 2 — Tasks 1 + 2 + 3 dispatched in parallel (`[P]` per plan §13); YAML overlap check confirmed disjoint file sets; all 3 have `requires: []`.
- Prior cohort handover (Task 0): all 19 probes pass; DQ 1b8527b076d4-001 answered Option A by advisor (commit `0401709e0`); no carry-forwards beyond the per-task GOTCHAs already cited.
- After this task ships, advisor laptop runs the VALIDATE block locally and mutates the validate-pending-laptop entry to `result: "pass"|"fail"`.
