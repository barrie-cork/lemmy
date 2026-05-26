# Brief: impl-task 4 — v1-RT-r3 e2e tests (10 tests across 5 stories)

## 1. Role + dispatch

`[role:impl-task] v1-RT-r3 task 4 — e2e tests 10 across 5 stories — see .claude/PRPs/briefs/v1-RT-r3-impl-4.md`

## 2. Scope

Implement §13 Task 4 of `.claude/PRPs/plans/v1-RT-r3.plan.md` verbatim. Add `v1_rt_r3_fixtures` module + 10 new e2e tests to `crates/server/tests/e2e.rs`.

**FILES (per plan §13 Task 4 FILES yaml):**

- creates: []
- modifies: `crates/server/tests/e2e.rs` (one new `v1_rt_r3_fixtures` module appended at end-of-file)
- requires:
  - task: 1 (participation_cron — `b9876b7d8` ✓ on phase tip)
  - task: 2 (vote-outcome + evidence-cited emit — `996765cae` ✓)
  - task: 3 (flag-bad-faith handler + route — `e544ae26e` ✓)
  - task: fix-impl-1 (#[expect(too_many_arguments)] on emit_reputation_event{_local} — `680e77ded` ✓)

All `requires:` on `phase-v1-RT-r3` @ `ddb439553`.

### 2.1 Canonical sibling (single — read FIRST, mirror verbatim)

**`v1_ship_3_fixtures` at `crates/server/tests/e2e.rs:16699-17099`** is the sole canonical mirror for this task. Do NOT read other sibling fixtures modules. The audit at `.claude/PRPs/reports/e2e-rs-code-quality-audit-2026-05-26.md` Axis 1 confirms all 13 sibling modules are Case A (outer `LemmyResult<()>`, helper `LemmyResult<T>`, `?` propagation); `v1_ship_3_fixtures` is the most recent and the closest fit.

Mirror verbatim:
- Module preamble (16699-16737): `use super::*;`, then explicit `use lemmy_*::*` blocks.
- Test attribute: `#[tokio::test(flavor = "multi_thread")]` (16739) — every r3 test uses this.
- Outer signature: `async fn <test_name>() -> LemmyResult<()>` (16740).
- Boot sequence (16744-16783): `unsafe { std::env::set_var(...) }`, then `governance_fixtures::start_postgres().await?`, `apply_all_schema(&mut sync_conn)?`, `build_db_pool_for_tests()`, `LemmyContext::create(...)`, `FederationConfig::builder()...build().await.map_err(|e| anyhow::anyhow!("{e}"))?`.
- Nested async helper pattern: `async fn seed_xxx(conn: &mut AsyncPgConnection, ...) -> LemmyResult<T>` returning `?`-propagated values (16795, 16834, 16851, 16877).
- `Option → Result` conversion: `.ok_or_else(|| anyhow::anyhow!("..."))?` (16931).
- Multi-arg helper: `#[expect(clippy::too_many_arguments, reason = "...")]` (16903-16906).
- Result close: `Ok(())` (17097).

### 2.2 IMPLEMENT (single contiguous block — append at file end)

1. Append ONE new `v1_rt_r3_fixtures` module after the closing `}` of `v1_ship_3_fixtures` (current end-of-file at line 17099).
2. Inside the module, define 5 shared async seeding helpers, all `LemmyResult<T>` outer + `?` propagation only:
   - `seed_active_users_in_community(conn, community_id, person_count, comments_per_user) -> LemmyResult<Vec<PersonId>>`
   - `seed_dormant_users_in_community(conn, community_id, person_count) -> LemmyResult<Vec<PersonId>>` (seeds prior participation_consistency events but no recent comments)
   - `seed_decided_case_with_panel(context, panel_size, aligned_count) -> LemmyResult<(ModerationCaseId, Vec<PersonId>)>`
   - `seed_emergency_remove_case_with_reporter(context, reporter_id) -> LemmyResult<ModerationCaseId>`
   - `count_reputation_events_for(conn, person_id, dimension, source_event_type) -> LemmyResult<i64>`
3. Define `EnvVarGuard` struct + `Drop` impl (see §2.3) inside the module.
4. Add 10 `#[tokio::test(flavor = "multi_thread")]` fns (`LemmyResult<()>` outer) per the story-to-test mapping in §2.4.

### 2.3 EnvVarGuard pattern (mandatory for env-var mutating tests)

Audit Axis 3 (`.claude/PRPs/reports/e2e-rs-code-quality-audit-2026-05-26.md` lines 10845–13373) flagged 7 sites where inline-restore env vars leak when a `?` between `set_var` and the restore short-circuits the success path. Every Story 1+2 test must use this guard:

```rust
struct EnvVarGuard {
  key: &'static str,
  prev: Option<String>,
}

impl EnvVarGuard {
  fn set(key: &'static str, value: &str) -> Self {
    let prev = std::env::var(key).ok();
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var(key, value);
    }
    Self { key, prev }
  }
}

impl Drop for EnvVarGuard {
  fn drop(&mut self) {
    // SAFETY: same justification — single-threaded test runner.
    unsafe {
      match &self.prev {
        Some(prev) => std::env::set_var(self.key, prev),
        None => std::env::remove_var(self.key),
      }
    }
  }
}
```

Bind it BEFORE `build_test_context().await?` and let RAII handle restore on every exit path (success, `?`, panic):

```rust
let _participation_guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
let (context, _ctx_guard) = build_test_context().await?;
```

### 2.4 Story-to-test mapping (per plan §16a verbatim)

- **Story 1** (Task 1, activity cron):
  - `participation_activity_cron_emits_plus_one_per_active_user` — seed 3 communities × 5 active users; call `participation_cron::run_activity_batch(&context).await?`; assert `count_reputation_events_for(reputation_dimension=ParticipationConsistency, source_event_type=ActivityCron) == 15`; assert each event's delta == `DEFAULT_DELTAS_PARTICIPATION_WEEKLY_ACTIVE` (use the const, not `1`); assert 3 governance_log entries with `entry_kind == ENTRY_KIND_PARTICIPATION_CRON_TICK`; assert each `dedupe_key` matches `activity_cron:<community>:<person>:<iso_week>`.
  - `participation_activity_cron_idempotent_across_same_iso_week` — call `run_activity_batch` twice in same ISO week; assert second invocation produces 0 new rows. Capture `chrono::Utc::now().iso_week()` at start + end; skip / xfail if boundary crossed mid-test.
- **Story 2** (Task 1, dormancy cron):
  - `participation_dormancy_cron_emits_minus_two_per_dormant_user` — seed dormant users (prior participation events but no recent comments); call `run_dormancy_batch`; assert delta == `DEFAULT_DELTAS_PARTICIPATION_DORMANT` (use the const) per dormant user per community + governance_log entries.
  - `participation_dormancy_cron_idempotent_across_same_iso_week` — same idempotency assertion.
- **Story 3** (Task 2, vote-outcome):
  - `vote_outcome_emits_plus_one_for_majority_aligned_jurors` — seed Decided case with N-juror panel + K majority-aligned; assert K ParticipationConsistency rows with `source_event_type = VoteOutcome`; assert delta per row == `DEFAULT_DELTAS_PARTICIPATION_JUROR_ALIGNED` (use the const); dedupe_key `vote_outcome:<case_id>:<juror_pseudonym>`.
  - `vote_outcome_emits_nothing_for_minority_jurors` — same shape; assert no rows for minority-voting jurors.
- **Story 4** (Task 3, flag-bad-faith):
  - `flag_bad_faith_returns_403_for_non_admin` — seed non-admin user; POST `/api/v4/governance/admin/emergency-remove/flag-bad-faith` with `{ case_id }`; assert HTTP 403.
  - `flag_bad_faith_returns_400_for_non_emergency_remove_status` — seed admin + case in `CaseStatus::Decided`; POST flag-bad-faith; assert HTTP 400 (per Task 3 mapping: `LemmyErrorType::Unknown` → `_ => BAD_REQUEST`).
  - `flag_bad_faith_admin_on_emergency_remove_case_emits_minus_one` — seed admin + EmergencyRemove case + reporter; POST flag-bad-faith; assert HTTP 200 + ReportingAccuracy reputation_event for reporter with delta == `DEFAULT_DELTAS_EVIDENCE_BAD_FAITH` (use the const); `source_event_type = EvidenceBadFaith`; dedupe_key `evidence_bad_faith:<case_id>`; ENTRY_KIND_EVIDENCE_QUALITY_RECORDED governance_log entry.
- **Story 5** (Task 2, evidence-cited heuristic):
  - `evidence_cited_heuristic_emits_plus_one_when_rationale_above_threshold` — seed Decided case with reporter who uploaded ≥1 `case_evidence` row + jurors whose `winning_rationales` includes a string of `DEFAULT_PARTICIPATION_EVIDENCE_CITED_RATIONALE_THRESHOLD_CHARS` chars (UTF-8 char count via `.chars().count()`, not `.len()`); assert ReportingAccuracy reputation_event for reporter with delta == `DEFAULT_DELTAS_EVIDENCE_CITED` (use the const); `source_event_type = EvidenceCited`; dedupe_key `evidence_cited:<case_id>:<reporter_pseudonym>`.

**Story 5 fallback (per plan §16a):** if v0 e2e fixtures cannot populate `jury_vote.rationale`, file `kind: "blocker"` DQ and defer Story 5 test to v2. The other 9 tests ship regardless.

### 2.5 Two-Edit split (MANDATORY per audit-driven discipline)

Edit budget: **2 Edit calls maximum** on `e2e.rs`. Do NOT scatter — and do NOT cram all 600+ lines into one Edit (the worker's prior cycle on a single-edit approach hit `error_max_turns` at 151 turns / $9.46 / 0 Edits on a smaller scope; per `feedback_junior_worker_e2e_edit_hang.md`).

- **Edit 1 (~150 lines):** append `mod v1_rt_r3_fixtures { ... }` skeleton + `use` block (mirror 16699-16737) + `EnvVarGuard` struct + Drop impl + 5 seed helper signatures + helper bodies. End the Edit with an empty test slot comment `// TESTS: 10 tests follow in Edit 2 — see brief §2.4 for mapping`.
- **Edit 2 (~450 lines):** add 10 `#[tokio::test(flavor = "multi_thread")]` fns replacing the placeholder comment. One contiguous block at the position of the placeholder.

After Edit 1 run `bash scripts/brehon/cargo-check.sh --workspace --features full` and `tail -20` the log — confirm exit 0 before starting Edit 2. If Edit 1 fails the check, fix in-place via additional Edit calls on the new module only (do not touch other sibling modules; do not exceed the 2-Edit cap for the test block itself).

## 3. Required reading

(Order is load-bearing: canonical sibling FIRST, schema reads SECOND, lessons THIRD. Stop reading at the line range named — do not expand to neighboring modules.)

### 3.1 Canonical sibling (read in full FIRST)

- `crates/server/tests/e2e.rs:16699-17099` — `v1_ship_3_fixtures` (THE sole mirror). Read end-to-end. Note the `use` block (16699-16737), the `EnvVarGuard`-equivalent inline-restore pattern at 16744-16753 (which this brief replaces with RAII per §2.3), the nested async helpers, the `?` propagation throughout, the `.ok_or_else(|| anyhow::anyhow!(...))?` at 16931, and the `#[expect(clippy::too_many_arguments, reason = "...")]` at 16903.

### 3.2 Schema + handler reads (SECOND)

- `crates/api/api/src/governance/participation_cron.rs` — Task 1 implementation; tests call `participation_cron::run_activity_batch(&context).await?` + `run_dormancy_batch(&context).await?`.
- `crates/api/api/src/governance/submit_jury_vote.rs:968-1100` — Task 2 `emit_reputation_event` signature (9 args; `#[expect(clippy::too_many_arguments)]` per fix-impl-1) + Source 3 (vote-outcome) + Source 4a (evidence-cited) inserts.
- `crates/api/api/src/governance/admin_emergency_remove.rs:447+` — Task 3 `emit_reputation_event_local` (file-private, 9-arg) + `flag_bad_faith` handler signature.
- `crates/api/api_common/src/governance.rs` — `FlagBadFaithEmergencyReport` + `FlagBadFaithEmergencyReportResponse` DTOs.
- `crates/db_schema/src/source/governance/case_evidence.rs:20` — `uploader_id` field name (authoritative; NOT `submitter_id`).
- `crates/api/api/src/governance/config.rs:813-1023` — `DEFAULT_*` constants. The test must import (not re-declare):
  - `DEFAULT_DELTAS_PARTICIPATION_WEEKLY_ACTIVE` (Story 1)
  - `DEFAULT_DELTAS_PARTICIPATION_DORMANT` (Story 2)
  - `DEFAULT_DELTAS_PARTICIPATION_JUROR_ALIGNED` (Story 3)
  - `DEFAULT_DELTAS_EVIDENCE_BAD_FAITH` (Story 4)
  - `DEFAULT_DELTAS_EVIDENCE_CITED` (Story 5)
  - `DEFAULT_PARTICIPATION_EVIDENCE_CITED_RATIONALE_THRESHOLD_CHARS` (Story 5)

### 3.3 Lessons (THIRD — load-bearing only)

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (outer `LemmyResult<()>` + helper `LemmyResult<T>` + no `.map_err` bridges).
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — single contiguous block; 2-Edit cap per §2.5.
- `.claude/lessons/feedback_clippy_test_style.md` + `feedback_clippy_rerun_after_fix.md` — workspace denies `unwrap_used`, `expect_used`, `allow_attributes`. Test bodies must use `?` propagation; helpers must return `LemmyResult<T>`.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection` + `DbPool::Conn` pattern.
- `.claude/lessons/feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` — cargo always `--workspace --features full`.
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — Shape G suspended; cargo on laptop.
- `.claude/rules/decision-queue.md` — DQ schema-v3; impl-task writes `kind: "validate-pending-laptop-e2e"`; ALWAYS use `dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh`.

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
      - "&mut (&mut *conn).into() reborrow (not &mut conn.into()) inside loops per file-local convention at lines 470/474"
      - "juror_pseudonym.clone() inside json!() to avoid String move before Some(juror_pseudonym)"
      - "diesel::select(diesel::dsl::exists(...)) full-path usage — no extra import"
      - "conn.into() in last governance_log::append (Source 4a) is safe — temporary DbPool dropped end-of-statement"
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
      - emit_reputation_event_local is file-private (not shared with submit_jury_vote) — 9-arg signature post task 2
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

## 4. Constraints

- **No edits outside `crates/server/tests/e2e.rs`.** Touching any other file is a process breach.
- **Two-Edit cap** (per §2.5). Edit 1 = module skeleton + helpers (~150 lines). Edit 2 = 10 tests (~450 lines). No third Edit on this block. Fix-in-place additional Edits to address cargo-check failures are allowed but must stay inside the new `v1_rt_r3_fixtures` module.
- **One commit:** `test(governance): r3 e2e tests across all 5 stories (task 4)`.
- **Case A error shape only** (per audit Axis 1 + canonical sibling `v1_ship_3_fixtures`). Outer `LemmyResult<()>` + helper `LemmyResult<T>` + `?` propagation. NO `Box<dyn Error>`.
- **NO `.unwrap()` and NO `.expect(...)` in test bodies or helpers** (per audit Axis 2 / DQ `a3d0e9941441-027` + `a3d0e9941441-028`):
  - `Option → Result`: `.ok_or_else(|| anyhow::anyhow!("..."))?` (mirror canonical sibling line 16931).
  - `Result<T, E>` already: `?` directly.
  - `Result::err()` patterns: replace with `match result { Ok(_) => panic!("expected error"), Err(e) => assert_eq!(e.status_code(), StatusCode::FORBIDDEN) }` (no `.err().unwrap()`).
  - The workspace `[workspace.lints.clippy]` denies `unwrap_used` + `expect_used` + `allow_attributes` — clippy will reject any unwrap/expect.
- **NO clippy `#[allow]`** — use `#[expect(..., reason = "...")]` only. Workspace denies `allow_attributes` per `feedback_clippy_test_style.md`.
- **EnvVarGuard RAII (per §2.3) for every Story 1+2 test** that sets `BREHON_DISABLE_PARTICIPATION_JOB=1`. Inline `set_var` + `set_var(prev)` after-the-fact pattern is forbidden (leaks on `?` short-circuit; audit Axis 3 finding).
- **Import `DEFAULT_*` consts, do not inline numeric literals** for any threshold or delta (per DQ `a3d0e9941441-029`). The 6 consts to import are listed in §3.2.
- **`evidence_cited_rationale_threshold_chars` length comparison uses `r.chars().count()`** — UTF-8 character count, not `.len()` (byte count).
- **`case_evidence::uploader_id` is the field name** — NOT `submitter_id`.
- **flag-bad-faith 400-arm asserts HTTP 400** — per Task 3 decision (`LemmyErrorType::Unknown` → `_ => BAD_REQUEST`).
- **flag-bad-faith URL exactly** `POST /api/v4/governance/admin/emergency-remove/flag-bad-faith`.
- **dedupe_key formats (verbatim, no reordering, no extra segments):**
  - Source 1 activity: `activity_cron:<community_id>:<person_id>:<iso_week>`
  - Source 2 dormancy: `dormancy_cron:<community_id>:<person_id>:<iso_week>`
  - Source 3 vote-outcome: `vote_outcome:<case_id>:<juror_pseudonym>`
  - Source 4a evidence-cited: `evidence_cited:<case_id>:<reporter_pseudonym>`
  - Source 4b evidence-bad-faith: `evidence_bad_faith:<case_id>` (NO reporter segment per ADR-013)
- **ISO-week boundary gate** — every idempotency test captures `chrono::Utc::now().iso_week()` at start, asserts same value at end. If boundary crosses mid-test, skip / xfail with explicit reason.
- **Pre-push cargo-check is mandatory** (per `feedback_fix_impl_pre_push_cargo_check.md`). Run `bash scripts/brehon/cargo-check.sh --workspace --features full` after Edit 1 AND after Edit 2; exit 0 required before push.
- **Mid-task DQ push** — raise `kind: "blocker"` immediately if:
  - `v1_ship_3_fixtures` shape doesn't match this brief's §2.1 description (sibling drifted post-brief authorship).
  - Story 5: v0 fixtures cannot populate `jury_vote.rationale` AT ALL (defer Story 5; ship 9 of 10).
  - `case_evidence::uploader_id` field absent or renamed.
  - `participation_cron::run_activity_batch` / `run_dormancy_batch` signature drift from plan §10.1.
  - Any `DEFAULT_*` const named in §3.2 is absent from `config.rs` (sibling drifted).
- **LESSON-trailer convention** — end commit body with `LESSON:` if a durable pattern emerges (e.g. EnvVarGuard RAII canonical, two-Edit cap respected).
- **HANDOVER trailer at commit body end** — populate per §3a shape.

## 5. VALIDATE (story-checkpoint feeds §16a Stories 1-5)

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
# EXPECT: E2E_EXIT_0 marker present; 10 new tests pass; pre-existing tests still pass.
```

**Note:** Shape G is SUSPENDED per DQ #229 — cargo runs on the laptop. Junior subagent must NOT execute VALIDATE locally.

**Post-validate:** Write `kind: "validate-pending-laptop-e2e"` DQ entry per `advisor-orchestrator.md` §5.2.
Required fields:
- `commands`: the 4 VALIDATE bash blocks above verbatim.
- `branch`: `"phase-v1-RT-r3"`
- `phase_task`: `4`

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push DQ to worker branch.

## 6. Forbidden-window check (advisor pre-queue)

Per `advisor-orchestrator.md` §5.1: Shape G suspended → forbidden-window non-binding for Junior daemon dispatch (cargo runs on laptop). Binding only for advisor-laptop validate-pending-laptop runs.

## 7. Context

- Phase: v1-RT-r3
- Plan: `.claude/PRPs/plans/v1-RT-r3.plan.md`
- Phase branch: `phase-v1-RT-r3` @ `ddb439553` (post cohort-2 + fix-impl-1 + DQ-mutation)
- Base branch for this task: `phase-v1-RT-r3`
- Cohort 2 (tasks 1+2+3) + fix-impl-1 all PASS validate-pending-laptop at `ddb439553`: cargo-check + cargo-clippy --no-deps -- -D warnings + cargo-test --test e2e --no-run all exit 0.
- Task 4 is non-`[P]` (sole post-cohort-2 barrier task before Task 5 retro). No parallel cohort.
- Prior Junior attempt (#474) hit `error_max_turns` at 151 turns ($9.46) on the looser-scoped brief that listed 4 candidate sibling modules. This re-author names ONE canonical sibling (§2.1), enforces the 2-Edit cap (§2.5), and bakes audit constraints (§4) into the brief body — root-cause fix per DQ `a3d0e9941441-030`.
