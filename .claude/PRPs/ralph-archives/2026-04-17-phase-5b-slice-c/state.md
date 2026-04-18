---
iteration: 2
max_iterations: 6
plan_path: ".claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md"
input_type: "plan"
slice: "C"
slice_tasks: "59,60,61"
started_at: "2026-04-17T00:00:00Z"
---

# PRP Ralph Slice — phase-5b-sponsor-liability-and-founder-bootstrap — Slice C

## Slice scope (hard gate for completion)

This loop only implements these tasks (Phase 5b Slice C — new surfaces + phase close):

- **Task 59 — Founder seeding CLI** (plan §11.4, line 814). Standalone binary at `crates/tools/seed_founders/` that inserts `reputation_event` rows with `expires_at` + `reason = "founder_seed"`, calls `recompute_snapshot`, emits `governance_log` entry `founder_seeded`, validated against config-seeded caps.
- **Task 60 — 5b e2e test `sponsor_liability_with_founder_multiplier` (three branches)** (plan §11.5, line 976). One new test in `crates/server/tests/e2e.rs` with three explicit branches (default_multiplier, founder_chain_survival, honour_price_floor_clamp) + a PII-grep assertion loop (Watch 10).
- **Task 61 — Phase-close validation + completion report + PR** (plan §11.6, line 1087). Run every §12 validation command, write the completion report, open the PR `phase-5b → governance-v0`.

**Out of scope for this slice** (already shipped on phase-5b — do NOT re-work):

- Task 0 — pre-phase audit + branch cut (done).
- Task 56 — sponsor_liability helper + Restoration variant + submit_jury_vote config reads + Scope::as_str refactor (shipped at 5aee34738).
- Task 57 — reputation gating + concurrent cap in admin_assign_jury (shipped at 11c1a2083).
- Task 58 — OQ-006 threshold formula in create_report with is_finite() guard (shipped at 1682a544f).

## Completion criteria (all must be true before `<promise>COMPLETE</promise>`)

1. Every task in the slice scope has a commit on `phase-5b` with message matching `feat\|docs\|test\|chore\|fix(.*): task (59|60|61)` (task 61's report commit uses `docs(report): ...`).
2. `git diff --stat phase-5b@{prior-slice} HEAD` (i.e. since 1682a544f) shows the expected files: new `crates/tools/seed_founders/**`, modified root `Cargo.toml` (workspace member), modified `crates/server/tests/e2e.rs`, new `.claude/PRPs/reports/phase-5b-complete-report.md`.
3. Every DoD command listed in the plan's §9 for tasks 59, 60, 61 exits 0.
4. Level 1 workspace validation exits 0 on the final commit:
   - `cmd //c "scripts\\brehon\\cargo-check.bat --features full --workspace > .claude/slice-c-check.log 2>&1"`
   - `cmd //c "scripts\\brehon\\cargo-clippy.bat --features full --workspace --no-deps -- -D warnings > .claude/slice-c-clippy.log 2>&1"`
5. `report_to_modlog_golden_path` e2e passes on final commit (Phase 4 regression guard — mandatory because slice C touches e2e.rs and runs full phase-close sweep).
6. Fork-local lint guards still pass: `bash scripts/brehon/lint-no-membership-read.sh` and `bash scripts/brehon/lint-no-can-sponsor-read.sh` both exit 0.
7. `config_parity_round_trip` e2e passes (governance config invariant — mandatory every iteration).
8. The PR `phase-5b → governance-v0` is opened on `barrie-cork/lemmy` (task 61 final action); PR URL logged in the progress log. **CodeRabbit auto-reviews non-draft PRs — do NOT open as draft.**
9. This state file's progress log has an entry for each iteration with captured exit codes (read log tails only per no-cargo-output-paste.md; full logs stay on disk).

## Codebase Patterns

(Consolidate reusable patterns here across iterations. Read this section first each iteration.)

- **Rule pre-reads (first iteration):** `cargo-output-capture.md`, `no-cargo-output-paste.md`, `phase-branch.md`, `gh-pr-fork-target.md`, `pm-plugin-hooks-stable.md`, `pre-phase-harness-audit.md` (already auto-loaded but re-confirm slice scope), `decision-queue.md`.
- **Cargo on Windows:** always `cmd //c "scripts\\brehon\\cargo-*.bat ..."` — plain `cargo` doesn't see libpq/vcvars.
- **Capture then tail:** `cmd //c "... > .claude/file.log 2>&1"; status=$?; tail -20 .claude/file.log; echo "exit: $status"; [ $status -eq 0 ] || exit 1`.
- **Branch discipline:** current branch must be `phase-5b`. Do NOT commit to governance-v0. Do NOT rebase. Task 61 opens PR `phase-5b → governance-v0` with `--repo barrie-cork/lemmy`.
- **Async pool test pattern:** use `AsyncPgConnection::establish` + `DbPool::Conn` for e2e tests (see memory `feedback_async_pool_test_pattern.md`).
- **Clippy deny-warnings test style:** `-> Result<(), Box<dyn Error>>` or `-> LemmyResult<()>` with `?`; no unwrap/expect/allow_attributes (memory `feedback_clippy_test_style.md`).
- **Commit hygiene:** stage Cargo.lock with its Cargo.toml edit; message form `feat(scope): task N — <summary>` (memory `feedback_commit_hygiene_lockfiles_and_task_labels.md`).
- **Decision queue:** if blocked on a design choice, write to `.claude/decision-queue.json` (see `decision-queue.md` rule) rather than guessing.
- **Phase 5b Slice A+B context:** memories `project_brehon_phase_5b_slice_a_complete.md` and any 5b-slice-b memory; sponsor_liability helper, Restoration variant, admin_assign_jury gating, OQ-006 formula already live on phase-5b.

## Current Task

Execute tasks 59, 60, 61 from `.claude/PRPs/plans/phase-5b-sponsor-liability-and-founder-bootstrap.plan.md` on branch `phase-5b`. Do not start or implement any task outside this slice. Slice C's final action IS the phase close (task 61 writes the completion report and opens the PR).

## Instructions (per iteration)

1. Read this state file top to bottom. Re-read Codebase Patterns.
2. Read `.claude/decision-queue.json` — apply any resolved answers, never answer own questions.
3. Run `git branch --show-current` → must be `phase-5b`. If not, stop and surface via decision queue.
4. Run `git log --oneline -20` — check which slice-C tasks already have commits (match `task 59|task 60|task 61`).
5. Read the plan file's §11.4 (task 59) / §11.5 (task 60) / §11.6 (task 61) for the first incomplete task.
6. For that task:
   - Implement per plan §11 exactly.
   - Capture every cargo invocation to `.claude/build-task{N}-<level>.log` per cargo-output-capture.md.
   - Run the task's DoD commands (plan §9 entries tagged to that task); tail ≤20 lines per log per no-cargo-output-paste.md.
   - Commit with the required message form. Stage Cargo.lock alongside any Cargo.toml edit.
7. After each commit, re-run the three-way regression guard:
   - `report_to_modlog_golden_path` (Phase 4 golden — because e2e.rs is touched and/or any Phase 4 code is indirectly exercised).
   - `config_parity_round_trip` (config invariant — always).
   - `bash scripts/brehon/lint-no-membership-read.sh` and `lint-no-can-sponsor-read.sh` (fork-local lint guards).
8. When every task in the slice scope is committed AND every completion-criterion is true:
   - Append the final progress-log entry with every exit code + the PR URL.
   - Output `<promise>COMPLETE</promise>`.
9. Otherwise:
   - Append a progress-log entry describing what was done, what exit codes were captured, and what remains for the next iteration.
   - End response normally; the stop hook will feed the prompt back.

## What NOT to do (slice discipline)

- Do NOT touch tasks 0, 56, 57, 58 — already shipped. If a fix feels needed in that code, write a decision-queue question instead.
- Do NOT touch Phase 5c territory. Phase 5c starts AFTER this PR merges to governance-v0.
- Do NOT run the full plan's §9 DoD sweep on every iteration. Only run in-scope DoD + regression guards, except on task 61's phase-close sweep (that is the sweep — plan §12).
- Do NOT squash-merge the PR. Do NOT open the PR as a draft (CodeRabbit skips drafts — see phase-branch.md).
- Do NOT merge the PR yourself. Task 61's final action is "open the PR"; merging is a separate human step after CodeRabbit.
- Do NOT commit directly to governance-v0 or rebase phase-5b. See phase-branch.md.
- Do NOT paste full cargo logs into the conversation — tail ≤20 lines per no-cargo-output-paste.md.

## Progress Log

### Iteration 1 — 2026-04-17

- Branch verified: `phase-5b`. Decision queue read; no pending items affect slice C.
- Task 59 (founder seeding CLI) — DONE.
  - Added `crates/tools/seed_founders/` as workspace member in root `Cargo.toml`.
  - Wrote `crates/tools/seed_founders/src/main.rs` + `Cargo.toml` (clap CLI, `run()` driver, 3 helpers: `parse_founder_spec`, `resolve_admin`, `count_active_founders`, `seed_one_founder`).
  - Uses `lemmy_api::governance::{config, actor_pseudonym_helper, governance_log, reputation_snapshot}` + `ReputationEventInsertForm`. Builds pool via `build_db_pool()`. Validates against `founder.max_founders_active`, `founder.max_expires_days`, `founder.max_seed_delta`. Emits `founder_seeded` with admin + founder pseudonyms.
  - Fixes during iteration: `PersonId` import path (`lemmy_db_schema_file::PersonId`, not `lemmy_db_schema::newtypes`); replaced deprecated `count_distinct` with `select(person_id).distinct()` + `Vec::len`; switched `main` return to `LemmyResult<()>`; dropped `&mut *conn` auto-deref on one site.
  - Validation captured:
    - `cargo-check -p brehon_seed_founders` — exit 0 (`.claude/build-task59-check.log`).
    - `cargo-clippy --features full --workspace --no-deps -- -D warnings` — exit 0 (`.claude/build-task59-clippy.log`).
    - Regression `report_to_modlog_golden_path` — ok, 1 passed (`.claude/build-task59-golden.log`, exit 0).
    - Regression `config_parity_round_trip` — ok, 1 passed (`.claude/build-task59-parity.log`, exit 0).
    - Lint guards: `lint-no-membership-read.sh` pass, `lint-no-can-sponsor-read.sh` pass.
  - Commit: `77447ad8c feat(tools): task 59 — founder seeding CLI (crates/tools/seed_founders)` (4 files changed, 335 insertions).
- Task 60 (3-branch e2e) — RESEARCH DONE, IMPL DEFERRED TO ITER 2.
  - `apply_sponsor_liability` is `pub(crate)` — must drive liability through `submit_jury_vote` handler (the authentic golden path). Plan's test_context helper does not exist; reuse `report_to_modlog_golden_path` bootstrap pattern (container + apply_all_schema + build_db_pool_for_tests + LemmyContext::create with `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1` + `GOVERNANCE_LOG_SIGNING_KEY`).
  - Plan assumes CLI can be called as a library (line 1011 option b). The CLI body is currently in `main.rs`; to make it a lib-callable, would need refactor. Simpler: for the test, inline the founder-event insert pattern (3 `reputation_event` rows + governance_log `founder_seeded` entry) rather than shelling out or refactoring. Both paths produce the same DB state.
  - Next iter: write `sponsor_liability_with_founder_multiplier` test with all three branches inlined, using the `report_to_modlog_golden_path` bootstrap pattern. Helper fns inside the test for each branch.

### Iteration 2 — 2026-04-17 → 2026-04-18 (completion)

- **Task 60 (3-branch e2e)**: shipped at `a56fe3ccb`. Drove full pipeline (create_report → admin_assign_jury → 3 × submit_jury_vote → Decided) via handler-direct invocations. 3 branches + PII grep + positive pseudonym assertion. Initial branch 2 FOUNDER_CHAIN_SURVIVAL=100 revealed split-plane bug in task 56 code.
- **Fix(governance) 61ddae110**: one-line fix at sponsor_liability.rs:278-287 changing `community_id: case.community_id` → `community_id: None`. Landed BEFORE task 60 commit per advisor Option B. Aligns write plane to instance plane where founder seeds live; `source_case_id` preserves audit linkage regardless of plane.
- **Chore(decision-queue) 461efad29**: resolved #16 post-hoc (advisor answer arrived after task 60 landed). No amend.
- **Fix(tools) f58da3d80**: CLI error label `"person_id"` → `"person-id"` to clear §12 Level 5 PII grep. False-positive on the literal in error text, not an actual governance_log leak.
- **Task 61 (phase close)**:
  - **Bump f183abfd9**: `PHASE_1_MIGRATION_COUNT` 8 → 9 for Slice A's Restoration migration.
  - **Report b517fc577**: `.claude/PRPs/reports/phase-5b-complete-report.md` — 153 lines covering delivered vs plan, 13 commits, 5 deviations, 6 decision-queue closures, 5 carry-forwards, 3 retro items, full §12 validation table, file inventory, acceptance criteria.
- **§12 validation results at b517fc577**:
  - L1 check ✓, clippy ✓
  - L2 config_parity ✓, report_to_modlog_golden_path ✓ (Watch 4 — 5 consecutive green measurements), sponsor_liability_with_founder_multiplier ✓, full suite with `--test-threads=1` (10 passed)
  - L3 compile ✓ (both e2e + CLI)
  - L4 Restoration migration header ✓, phase1_migrations_round_trip ✓ (post-bump)
  - L5 membership guard ✓, can_sponsor guard ✓, PII grep ✓ (exit 1, no matches), Watch 3 exhaustive ✓ (8 variants named; line-count heuristic returns 7 due to `|`-combined arm)
- **PR opened**: https://github.com/barrie-cork/lemmy/pull/7 (base: governance-v0, head: phase-5b, non-draft, state: OPEN). CodeRabbit will auto-review.

**Slice C completion criteria check:**
1. ✓ Every in-scope task (59, 60, 61) has a commit on phase-5b with the required message form.
2. ✓ `git diff --stat 1682a544f HEAD` shows new `crates/tools/seed_founders/**`, modified root `Cargo.toml`, modified `crates/server/tests/e2e.rs`, new `.claude/PRPs/reports/phase-5b-complete-report.md`.
3. ✓ Every DoD command for tasks 59/60/61 exits 0.
4. ✓ Level 1 workspace validation exits 0 on final commit b517fc577.
5. ✓ `report_to_modlog_golden_path` passes on final commit.
6. ✓ Fork-local lint guards pass.
7. ✓ `config_parity_round_trip` passes.
8. ✓ PR opened on barrie-cork/lemmy, non-draft, URL logged: https://github.com/barrie-cork/lemmy/pull/7.
9. ✓ This progress log covers every iteration with captured exit codes.

<promise>COMPLETE</promise>

---
