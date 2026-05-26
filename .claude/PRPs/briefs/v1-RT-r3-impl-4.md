# Brief: impl-task 4 — v1-RT-r3 e2e tests (10 tests across 5 stories)

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 task 4 — e2e tests 10 across 5 stories — see .claude/PRPs/briefs/v1-RT-r3-impl-4.md`

## 2. Scope

Implement §13 Task 4 of `.claude/PRPs/plans/v1-RT-r3.plan.md` verbatim. Add `v1_rt_r3_fixtures` module + 10 new e2e tests to `crates/server/tests/e2e.rs`.

**FILES (per plan §13 Task 4 FILES yaml):**

- creates: []
- modifies: `crates/server/tests/e2e.rs` (one new `v1_rt_r3_fixtures` module + 10 tests, ONE contiguous block)
- requires:
  - task: 1 (participation_cron module must be on phase branch — `b9876b7d8` ✓)
  - task: 2 (vote-outcome + evidence-cited emit must be on phase branch — `996765cae` ✓)
  - task: 3 (flag-bad-faith handler + route must be on phase branch — `e544ae26e` ✓)

All `requires:` already on `phase-v1-RT-r3` at tip `ddb439553`.

**IMPLEMENT (file 1 of 1):**

1. Add ONE new `v1_rt_r3_fixtures` module to `crates/server/tests/e2e.rs` right after the most recent prior fixtures module. Read the candidate siblings (`v1_rt_r2_fixtures`, `v1_sl_d_fixtures`, `v1_jm_e_fixtures`) to pick one with Case A error shape (outer `LemmyResult<()>`, helper `LemmyResult<T>`, no `.map_err` bridges). Mirror its shape verbatim.
2. Inside the module add 5 shared seeding helpers (all `LemmyResult<T>` outer):
   - `seed_active_users_in_community(pool, community_id, person_count, comments_per_user) -> LemmyResult<Vec<PersonId>>`
   - `seed_dormant_users_in_community(pool, community_id, person_count) -> LemmyResult<Vec<PersonId>>` (seeds prior participation_consistency events but no recent comments)
   - `seed_decided_case_with_panel(context, panel_size, aligned_count) -> LemmyResult<(ModerationCaseId, Vec<PersonId>)>`
   - `seed_emergency_remove_case_with_reporter(context, reporter_id) -> LemmyResult<ModerationCaseId>`
   - `count_reputation_events_for(pool, person_id, dimension, source_event_type) -> LemmyResult<i64>`
3. Add 10 `#[tokio::test(flavor = "multi_thread")]` fns (`LemmyResult<()>` outer) per the story-to-test mapping below.

**Story-to-test mapping (per plan §16a verbatim):**

- Story 1 (Task 1, activity cron):
  - `participation_activity_cron_emits_plus_one_per_active_user` — seed 3 communities × 5 active users; call `participation_cron::run_activity_batch(&context).await?`; assert 15 reputation_event rows + 3 ENTRY_KIND_PARTICIPATION_CRON_TICK governance_log entries + each event's `dedupe_key` matches `activity_cron:<community>:<person>:<iso_week>`.
  - `participation_activity_cron_idempotent_across_same_iso_week` — call `run_activity_batch` twice in the same ISO week; assert second invocation produces 0 new rows (dedupe_key partial unique index consumes via `on_conflict_do_nothing`). Gate on `chrono::Utc::now().iso_week()` returning same value at start + end of test (skip / xfail if boundary crossed mid-test).
- Story 2 (Task 1, dormancy cron):
  - `participation_dormancy_cron_emits_minus_two_per_dormant_user` — seed dormant users (prior participation events but no recent comments); call `run_dormancy_batch`; assert -2 reputation_event rows per dormant user per community + governance_log entries.
  - `participation_dormancy_cron_idempotent_across_same_iso_week` — same idempotency assertion as Story 1.
- Story 3 (Task 2, vote-outcome):
  - `vote_outcome_emits_plus_one_for_majority_aligned_jurors` — seed Decided case with N-juror panel + K majority-aligned; assert K +1 ParticipationConsistency rows with `source_event_type = VoteOutcome` + dedupe_key `vote_outcome:<case_id>:<juror_pseudonym>`.
  - `vote_outcome_emits_nothing_for_minority_jurors` — seed same shape; assert no rows for minority-voting jurors.
- Story 4 (Task 3, flag-bad-faith):
  - `flag_bad_faith_returns_403_for_non_admin` — seed non-admin user; POST `/api/v4/governance/admin/emergency-remove/flag-bad-faith` with body `{ case_id }`; assert HTTP 403. Mirror `is_admin` denial tests already in e2e.rs (search `NotAnAdmin` / `403`).
  - `flag_bad_faith_returns_400_for_non_emergency_remove_status` — seed admin + a case in `CaseStatus::Decided` (NOT EmergencyRemove); POST flag-bad-faith; assert HTTP 400 (per Task 3 mapping decision: `LemmyErrorType::Unknown` → `_ => BAD_REQUEST` in `LemmyError::status_code()`).
  - `flag_bad_faith_admin_on_emergency_remove_case_emits_minus_one` — seed admin + EmergencyRemove case + reporter; POST flag-bad-faith; assert HTTP 200 + -1 ReportingAccuracy reputation_event for reporter with `source_event_type = EvidenceBadFaith` + dedupe_key `evidence_bad_faith:<case_id>` + ENTRY_KIND_EVIDENCE_QUALITY_RECORDED governance_log entry.
- Story 5 (Task 2, evidence-cited heuristic):
  - `evidence_cited_heuristic_emits_plus_one_when_rationale_above_threshold` — seed Decided case with reporter who uploaded ≥1 `case_evidence` row + jurors whose `winning_rationales` includes a string of `≥evidence_cited_rationale_threshold_chars` chars (UTF-8 char count via `.chars().count()`, not `.len()`); assert +1 ReportingAccuracy reputation_event for reporter with `source_event_type = EvidenceCited` + dedupe_key `evidence_cited:<case_id>:<reporter_pseudonym>`.

**Story 5 fallback (per plan §16a):** if v0 e2e fixtures cannot populate `jury_vote.rationale` (rare per clarify a3d0e9941441-022), file `kind: "blocker"` DQ and defer Story 5 test to v2. The other 9 tests ship regardless.

**Test invocation pattern (canonical — every test must mirror):**

```rust
#[tokio::test(flavor = "multi_thread")]
async fn participation_activity_cron_emits_plus_one_per_active_user() -> LemmyResult<()> {
  // Set BREHON_DISABLE_PARTICIPATION_JOB=1 BEFORE build_test_context so the
  // scheduler tick never races the test's direct call. Mirror BREHON_DISABLE_SNAPSHOT_JOB
  // discipline.
  std::env::set_var("BREHON_DISABLE_PARTICIPATION_JOB", "1");
  let (context, _guard) = build_test_context().await?;
  // ... seed + call + assert ...
  Ok(())
}
```

**VALIDATE (story-checkpoint feeds §16a Stories 1-5):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-RT-r3-task4-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task4-check.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-RT-r3-task4-clippy.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-RT-r3-task4-clippy.log
# EXPECT: exit 0
```

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-RT-r3-task4-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-RT-r3-task4-test-no-run.log
# EXPECT: exit 0
```

```bash
# E2E (laptop-only — mutated by advisor-laptop session, NOT EliteDesk worker)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-RT-r3-task4-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-RT-r3-task4-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-RT-r3-task4-e2e.log"
tail -50 .claude/PRPs/debug/v1-RT-r3-task4-e2e.log
# EXPECT: E2E_EXIT_0 marker present; all 10 new tests pass + all pre-existing tests still pass.
```

**Note:** Shape G is SUSPENDED per DQ #229 — cargo runs on the laptop. Junior subagent must NOT execute VALIDATE locally.

**Post-validate:** Write `kind: "validate-pending-laptop-e2e"` DQ entry per `advisor-orchestrator.md` §5.2.
Required fields:
- `commands`: the 4 VALIDATE bash blocks above (verbatim — including `--workspace --features full` and the e2e run command). The 4th block is the e2e run; advisor-laptop runs all 4 in sequence and mutates the DQ entry.
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: `4`

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push DQ to worker branch.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: b9876b7d8
    filesCreated:
      - crates/api/api/src/governance/participation_cron.rs
    filesModified:
      - crates/api/api/src/governance/mod.rs
      - crates/routes/src/utils/scheduled_tasks.rs
    keyDecisions:
      - sql_query for both discovery queries (activity GROUP BY HAVING + dormancy NOT EXISTS anti-join)
      - on_conflict_do_nothing untargeted (partial index; targeted form not supported)
      - ENTRY_KIND_PARTICIPATION_CRON_TICK used for both source-1 and source-2 governance_log entries
    notes: validate-pending-laptop DQ c76792506538-001 PASS at advisor-laptop SHA 7c1bcddab on phase-v1-RT-r3
  - task: 2
    commit: 996765cae
    filesCreated: []
    filesModified:
      - crates/api/api/src/governance/submit_jury_vote.rs
    keyDecisions:
      - "Used &mut (&mut *conn).into() reborrow (not &mut conn.into()) inside loops per file-local convention at lines 470/474"
      - "juror_pseudonym.clone() inside json!() macro to avoid String move before Some(juror_pseudonym)"
      - "diesel::select(diesel::dsl::exists(...)) full-path usage for exists query — no extra import needed"
      - "conn.into() in last governance_log::append (Source 4a) is safe — temporary DbPool dropped end-of-statement, conn reusable after"
    notes: validate-pending-laptop DQ 1bb1ff7a00a8-001 PASS post fix-impl-1 (clippy too_many_arguments resolved via #[expect])
  - task: 3
    commit: e544ae26e
    filesCreated: []
    filesModified:
      - crates/api/api_common/src/governance.rs
      - crates/api/api/src/governance/admin_emergency_remove.rs
      - crates/api/routes/src/lib.rs
    keyDecisions:
      - LemmyErrorType::Unknown maps to HTTP 400 via _ catch-all in status_code() — Task 4's 400-arm test asserts on HTTP 400
      - emit_reputation_event_local is file-private (not shared with submit_jury_vote) — also 9-arg post task 2 signature change
      - dedupe_key = "evidence_bad_faith:{case_id}" only (no reporter segment)
    notes: validate-pending-laptop DQ 81719cf8ca8d-001 PASS post fix-impl-1
  - task: fix-impl-1
    commit: 680e77ded
    filesCreated: []
    filesModified:
      - crates/api/api/src/governance/submit_jury_vote.rs
      - crates/api/api/src/governance/admin_emergency_remove.rs
    keyDecisions:
      - "#[expect(clippy::too_many_arguments)] not #[allow] — workspace deny allow_attributes per feedback_clippy_test_style.md"
      - "submodule init required before cargo-check on fresh worktree"
    notes: validate-pending-laptop DQ 8fc5df3a3664-001 PASS at advisor-laptop SHA ddb439553 (check + clippy + test --no-run all exit 0)
```

## 3. Required reading

- `.claude/PRPs/plans/v1-RT-r3.plan.md` §13 Task 4 (authoritative IMPLEMENT + MIRROR + GOTCHA + VALIDATE)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §16a (5 stories — test names + expected counts + assertions)
- `.claude/PRPs/plans/v1-RT-r3.plan.md` §4 Surface 4 (story-to-test mapping)
- `crates/server/tests/e2e.rs` — read existing `v1_rt_r2_fixtures` / `v1_sl_d_fixtures` / `v1_jm_e_fixtures` modules at their declaration lines to pick the canonical Case A sibling (outer `LemmyResult<()>`, helper `LemmyResult<T>`, no `.map_err` bridges)
- `crates/server/tests/e2e.rs:9267-9400` — existing `emergency_remove_open_case` test region; reuse helper patterns for the flag-bad-faith 200-arm seed
- `crates/api/api/src/governance/participation_cron.rs` — Task 1 implementation; test calls `participation_cron::run_activity_batch` + `run_dormancy_batch` directly
- `crates/api/api/src/governance/submit_jury_vote.rs:968-1100` — Task 2 emit_reputation_event signature (9 args including source_event_type + dedupe_key) + Source 3 + 4a inserts
- `crates/api/api/src/governance/admin_emergency_remove.rs:447-` — Task 3 emit_reputation_event_local + flag_bad_faith handler
- `crates/api/api_common/src/governance.rs` — FlagBadFaithEmergencyReport + FlagBadFaithEmergencyReportResponse DTOs
- `crates/db_schema/src/source/governance/case_evidence.rs:20` — `uploader_id` field name (authoritative; NOT submitter_id)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (LemmyResult<T> outer + helper; mirror sibling shape verbatim)
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — SINGLE contiguous block; never scattered Edit on e2e.rs
- `.claude/lessons/feedback_async_pool_test_pattern.md` — AsyncPgConnection + DbPool::Conn pattern for e2e
- `.claude/lessons/feedback_clippy_test_style.md` + `.claude/lessons/feedback_clippy_rerun_after_fix.md` — Lemmy workspace test-style (deny unwrap/expect/allow_attributes; ? on LemmyResult)
- `.claude/lessons/feedback_features_full_workspace_only.md` + `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — cargo always uses `--workspace --features full`
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — Shape G suspended; cargo runs on laptop
- `.claude/rules/decision-queue.md` — DQ schema-v3; impl-task writes `kind: "validate-pending-laptop-e2e"`; ALWAYS use `dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh`
- ADR-013 (admin-driven posture) + ADR-015 (pseudonymisation)

## 4. Constraints

- **No edits outside `crates/server/tests/e2e.rs`.** Touching any other file is a process breach.
- **One commit** — `test(governance): r3 e2e tests across all 5 stories (task 4)`.
- **SINGLE contiguous block Edit on e2e.rs.** Per `feedback_junior_worker_e2e_edit_hang.md`: write the entire `v1_rt_r3_fixtures` module + all 10 test fns in ONE Edit call (or ONE Write call if appending at end-of-file is cleaner). Scattered edits trigger the worker-hang risk class.
- **Case A error shape** — outer `LemmyResult<()>` + helper `LemmyResult<T>` + no `.map_err` bridges. Pick the canonical sibling from the candidates listed in §3 and mirror its shape verbatim BEFORE writing the new module. If sibling uses Case B (Box<dyn Error>) reject and pick a different sibling — r3 stays Case A.
- **`BREHON_DISABLE_PARTICIPATION_JOB=1` set BEFORE `build_test_context().await?`** in every Story 1+2 test (mirror BREHON_DISABLE_SNAPSHOT_JOB discipline).
- **`evidence_cited_rationale_threshold_chars` uses `r.chars().count()` not `r.len()`** — UTF-8 character count, not byte length.
- **`case_evidence::uploader_id` is the field name** — NOT `submitter_id`.
- **flag-bad-faith 400-arm asserts HTTP 400** — per Task 3 decision (`LemmyErrorType::Unknown` → `_ => BAD_REQUEST`).
- **flag-bad-faith URL exactly** `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith`.
- **dedupe_key formats (verbatim, no reordering, no extra segments):**
  - Source 1 activity: `activity_cron:<community_id>:<person_id>:<iso_week>`
  - Source 2 dormancy: `dormancy_cron:<community_id>:<person_id>:<iso_week>`
  - Source 3 vote-outcome: `vote_outcome:<case_id>:<juror_pseudonym>`
  - Source 4a evidence-cited: `evidence_cited:<case_id>:<reporter_pseudonym>`
  - Source 4b evidence-bad-faith: `evidence_bad_faith:<case_id>` (NO reporter segment per ADR-013)
- **ISO-week boundary gate** — every idempotency test captures `chrono::Utc::now().iso_week()` at start, asserts the same value at end. If the boundary crosses mid-test, skip / xfail with explicit reason (don't flake silently).
- **No clippy `#[allow]`** — use `#[expect(...)]` only after verifying lint is intentional. Workspace denies `allow_attributes` per `feedback_clippy_test_style.md`.
- **Pre-push cargo-check is mandatory** (per `feedback_fix_impl_pre_push_cargo_check.md`). Run `bash scripts/brehon/cargo-check.sh --workspace --features full` after Edit; exit 0 required before push.
- **Mid-task DQ push** — raise `kind: "blocker"` immediately if:
  - No `v1_*_fixtures` sibling matches Case A shape (all are Case B or mixed).
  - Story 5: v0 fixtures cannot populate `jury_vote.rationale` AT ALL (defer Story 5 test to v2; ship 9 of 10 tests with this blocker resolved).
  - `case_evidence::uploader_id` field absent or renamed on phase branch.
  - `participation_cron::run_activity_batch` / `run_dormancy_batch` signature drift from plan §10.1.
- **LESSON-trailer convention** — end commit body with `LESSON:` if a durable pattern emerges (e.g. canonical Case A sibling identification, ISO-week boundary gate idiom).
- **HANDOVER trailer at commit body end** — populate per §3a shape so the next reader (verify / retro / next phase) can grep mechanically.

## 5. Forbidden-window check (advisor pre-queue)

Per `advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window non-binding for Junior daemon dispatch (cargo runs on laptop). Binding only for advisor-laptop validate-pending-laptop runs.

Current UTC at queue: Tue 07:18 UTC (primary window 16:00–02:30 — primary closes at 02:30, currently outside; next safe window 16:00 Tue). However: per the Shape-G-suspended rule, the daemon dispatch is non-binding. Laptop validation will run when user picks (user gate 4 — local vs dispatch e2e — applies to e2e step specifically).

## 6. Context

- Phase: v1-RT-r3
- Plan: `.claude/PRPs/plans/v1-RT-r3.plan.md` (on trunk + phase branch)
- Phase branch: `phase-v1-RT-r3` @ `ddb439553` (after cohort-2 + fix-impl-1 + DQ-mutation push)
- Base branch for this task: `phase-v1-RT-r3`
- Cohort 2 (tasks 1+2+3) + fix-impl-1 all PASS validate-pending-laptop at `ddb439553`: cargo-check + cargo-clippy --no-deps -- -D warnings + cargo-test --test e2e --no-run all exit 0.
- Task 4 is non-`[P]` (sole post-cohort-2 barrier task before Task 5 retro). No parallel cohort.
- All §16a story Brief-Scope outputs from cohort-2 verified by validate-pending-laptop cycle. Task 4 is the e2e gate — verifies behaviour end-to-end (HTTP + DB + governance_log).
