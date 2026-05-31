[role:planning] v1-quality-r3c — Issue #165 + #166 + BREHON_DISABLE_* audit

---

## §1. Role + dispatch

**Role:** `[role:planning]`  
**Summary:** Author `.claude/PRPs/plans/v1-quality-r3c.plan.md` covering three quality items: (1) fix stale `.coderabbit.yaml` v0 "exactly 11 endpoints" rule (Issue #165); (2) add sponsor-allowlist HTTP-path coverage to the existing route sweep test (Issue #166 gap); (3) add BREHON_DISABLE_SNAPSHOT_JOB and BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB test coverage (BREHON_DISABLE_* audit — the two uncovered gates).

---

## §2. Scope

**Produce:** `.claude/PRPs/plans/v1-quality-r3c.plan.md` — a complete Brehon sub-phase plan following `.claude/PRPs/templates/plan.template.md`.

**Explicit boundaries:**

**Task 1 — Issue #165: fix `.coderabbit.yaml` stale endpoint-count rule**
- In scope: edit `.coderabbit.yaml` path instruction for `crates/api/api_common/src/governance.rs` (lines ~139-145) — remove or scope the "EXACTLY 11 endpoints" assertion so it no longer fires on v1 PRs adding sanctioned v1 governance endpoints
- Out of scope: do NOT remove the ADR-010 endpoint-count intent entirely — the rule should still flag NEW endpoints that are NOT in the v0 MVP list AND do NOT have a v1 PRD-backed justification. The fix is "scope the assertion to v0 era" or "reference the PRD/OQ-020 as the sanctioned extension path", not "delete the rule body"
- File: `.coderabbit.yaml` only. No Rust changes.

**Task 2 — Issue #166 gap: sponsor-allowlist HTTP-path coverage**
- In scope: add the two missing sponsor-allowlist routes to the `all_mvp_endpoints_return_non_404` Phase A sweep in `crates/server/tests/e2e.rs` at approximately line 4229. Specifically:
  - `POST /api/v4/governance/admin/sponsor-allowlist/add`
  - `POST /api/v4/governance/admin/sponsor-allowlist/remove`
- Out of scope: do NOT add Phase B (happy-path) assertions — this task extends Phase A only (non-404 sweep). Direct-handler tests already exist at e2e.rs ~line 18122
- File: `crates/server/tests/e2e.rs` — ONE Edit adding the two route entries to the endpoints slice. The rate_limit bucket bump (per `feedback_rate_limit_debug_config_post_bucket.md`) is ALREADY present in the existing test — do NOT add a second `set_config` call; just extend the array
- **Critical:** verify the rate-limit bump is present before authoring (check lines ~4086-4162 for `set_config` or `with_debug_config`). If absent, Task 2 must also add it.

**Task 3 — BREHON_DISABLE_* audit: cover SNAPSHOT_JOB and FED_REPLAY_CLEANUP_JOB**
- In scope: add e2e tests that toggle `BREHON_DISABLE_SNAPSHOT_JOB=1` and `BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB=1` via `EnvVarGuard` and assert that the corresponding job function exits early (does NOT write to the DB / emit a reputation_event) when the gate is set
- Pattern to mirror: the existing `BREHON_DISABLE_PARTICIPATION_JOB` guard tests at e2e.rs ~lines 17592-17731 — use the same `EnvVarGuard::set` + assert-no-write shape
- File: `crates/server/tests/e2e.rs` — new test functions for each gate
- Out of scope: BREHON_DISABLE_APPEAL_WINDOW_JOB, BREHON_DISABLE_GRACE_CHECK_JOB, BREHON_DISABLE_PARTICIPATION_JOB (all already covered)

**Task ordering:** T1 (`.coderabbit.yaml`) is independent and very small — do NOT mark `[P]`; serial. T2 and T3 both edit `e2e.rs` — they MUST be serial (no `[P]` — shared file, no index.lock hazard but Edit tool conflicts). T3 after T2.

**Commit only:** `.claude/PRPs/plans/v1-quality-r3c.plan.md`  
**Do NOT author:** implementation code, migration files, test fixtures, or other plan files.

---

## §3. Required reading

### §3a. Pre-populated research (do NOT re-run these greps — results are given)

**Issue #165 — `.coderabbit.yaml` stale rule location:**
```
.coderabbit.yaml:141   path: "crates/api/api_common/src/governance.rs"
.coderabbit.yaml:141     instructions: |
.coderabbit.yaml:141       Phase 3 DTO scope check. v0 is EXACTLY 11 endpoints per ADR-010 and
.coderabbit.yaml:141       05-mvp-and-delivery-plan.md §2. Flag new DTOs that do not map to one
.coderabbit.yaml:141       of those 11. RevokeEndorsement DTO is allowed even though the
.coderabbit.yaml:141       endpoint is deferred to v1 (explicit Phase 3 task 35 exception).
.coderabbit.yaml:141       Flag business logic inside DTO impls — DTOs are shape-only.
```
The fix must retain the "flag business logic in DTOs" and "DTOs are shape-only" clauses — only the "EXACTLY 11" count assertion needs updating. Suggested replacement: reference OQ-020 / PRD v1 §181 as the sanctioned extension path; remove the hard count assertion.

**Issue #166 — current HTTP sweep test structure:**
- Function: `all_mvp_endpoints_return_non_404` at e2e.rs ~line 4081 (attribute `#[tokio::test(flavor = "multi_thread")]`)
- Phase A sweep array: lines ~4165-4229 — 14 entries (11 MVP + 3 admin backstops)
- Missing from sweep: `POST /api/v4/governance/admin/sponsor-allowlist/add` and `POST /api/v4/governance/admin/sponsor-allowlist/remove` (registered at `crates/api/routes/src/lib.rs:526-528`)
- Rate-limit bucket: check whether `set_config` bump is present in the test setup — if NOT, it must be added (the Phase A sweep hits multiple POST routes and will trip the 6-call bucket)
- **Unique anchor for edit** (verbatim — occurrence count: 1):
  ```
      (
        "GET",
        "/api/v4/governance/admin/reputation-stats",
        "",
        &[200, 400, 401],
      ),
    ];
  ```
  Insert the two new sponsor-allowlist entries BEFORE the `];` closing line.

**BREHON_DISABLE_* audit — uncovered gates (from grep results):**
- `BREHON_DISABLE_SNAPSHOT_JOB` (line 222 in `scheduled_tasks.rs`) — calls `lemmy_api::governance::reputation_snapshot::run_snapshot_batch` — zero set_var / EnvVarGuard in e2e.rs
- `BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB` (line 414 in `scheduled_tasks.rs`) — zero set_var / EnvVarGuard in e2e.rs
- Already covered (do NOT duplicate): `BREHON_DISABLE_APPEAL_WINDOW_JOB` (lines 10843+, 11072+), `BREHON_DISABLE_GRACE_CHECK_JOB` (lines 12811+, 12942+), `BREHON_DISABLE_PARTICIPATION_JOB` (lines 17592, 17647, 17687, 17731)
- Pattern to mirror: lines 17592-17731 (`BREHON_DISABLE_PARTICIPATION_JOB` guard tests) — use `EnvVarGuard::set("BREHON_DISABLE_SNAPSHOT_JOB", "1")` and assert the target function exits early without a DB write

**BREHON_DISABLE_SNAPSHOT_JOB function body:** calls `reputation_snapshot::run_snapshot_batch(&context)`. A test should assert that, with the guard set, no new `reputation_snapshot` rows are written to the DB. The simplest probe: count `reputation_snapshot` rows before and after calling the scheduler tick; with gate set they must be equal.

**BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB function body:** at `scheduled_tasks.rs:396-418` — reads `fed_replay_cleanup_job_enabled` from context and calls an unspecified cleanup. The planner MUST read `scheduled_tasks.rs` lines ~390-425 and `crates/routes/src/utils/scheduled_tasks.rs` to establish the function name before authoring Task 3's IMPLEMENT section.

**Issue #158 gate — DO NOT include in scope:**
- `grep -rn "emit_reputation_event" crates/ --include="*.rs" -l` returned ONLY 2 files:
  - `crates/api/api/src/governance/admin_emergency_remove.rs`
  - `crates/api/api/src/governance/submit_jury_vote.rs`
- 2 callers = premature-DRY gate NOT met. Defer to r3d. Do NOT add to plan.

**Shape G status:** SUSPENDED — GH Actions minutes exhausted (billing cycle resets 2026-06-01). Validation path: `validate-pending-laptop` (local `cargo-test.bat` + `cargo-check.bat`). Do NOT write validate-pending (Shape G) DQ entries. Plan §15 DoD must use laptop-runnable commands only.

**Lesson injections fired:**
- `e2e.rs` edits (Tasks 2 and 3): `feedback_lemmy_error_no_std_error.md` (case B pattern for new test functions returning LemmyResult), `feedback_async_pool_test_pattern.md` (AsyncPgConnection::establish for direct DB probes), `feedback_fix_impl_pre_locate_e2e_anchors.md` (uniqueness gate for every Edit anchor)
- Rate-limit bucket for route sweep: `feedback_rate_limit_debug_config_post_bucket.md` (6-call POST bucket — verify set_config is already present)

### §3b. Files to read before authoring

1. `.claude/PRPs/templates/plan.template.md` — canonical plan structure
2. `.coderabbit.yaml` lines 135-165 — full context of the stale rule + adjacent rules to understand the scope of edits
3. `crates/server/tests/e2e.rs` lines 4081-4248 — the full `all_mvp_endpoints_return_non_404` test body (Phase A + start of Phase B) to understand what the sponsor-allowlist entries need to look like
4. `crates/server/tests/e2e.rs` lines 17570-17740 — the PARTICIPATION_JOB guard test pattern (canonical DISABLE gate test shape to mirror for Tasks 2/3)
5. `crates/routes/src/utils/scheduled_tasks.rs` lines 200-430 — the four job functions + their BREHON_DISABLE_ gates (snapshot, appeal, grace, participation, fed_replay_cleanup)
5a. `crates/db_schema/src/source/governance/federation_inbox_nonce.rs` — `FederationInboxNonceInsertForm` (line 30: `peer_instance: String` + `activity_id: String`) + `delete_older_than(window_days: i64, conn: &mut AsyncPgConnection)` (line 37). **Task 3 FED_REPLAY_CLEANUP test shape** (per DQ `a3d0e9941441-043`): the guard lives in the scheduler closure, NOT in `delete_older_than` itself. Two tests: (a) gate=unset → seed old row → call `delete_older_than(1, conn)` → assert row is gone; (b) gate=set → seed old row → skip calling `delete_older_than` (mirroring scheduler guard) → assert row still present. This is the only viable test shape since the scheduler closure is not callable from test context.
6. `.claude/lessons/feedback_rate_limit_debug_config_post_bucket.md` — LOAD-BEARING for Task 2: the `set_config` bump IS already present in lines 4135-4146 of `all_mvp_endpoints_return_non_404`; DO NOT add a second `set_config` call; extend the endpoints array only
7. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — new test functions must return `LemmyResult<()>` with `?`
8. `.claude/lessons/feedback_async_pool_test_pattern.md` — direct DB probe pattern for the DISABLE gate tests
9. `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim anchors for every Edit target; uniqueness gate

---

## §4. Constraints

1. **No Rust authorship** — produce only the plan file. Do not write or suggest Rust implementation.
2. **plan.template.md §13 task ordering** — T1 (`.coderabbit.yaml`) is independent. T2 and T3 both edit `e2e.rs`; mark both non-`[P]`, serial (T2 then T3). A cohort of 3 with shared `e2e.rs` would hit index.lock hazard per `feedback_cohort_shared_git_index_contention.md`.
3. **DQ mid-task push** — any blocker DQ from the planner must commit + push immediately per `.claude/rules/decision-queue.md` "Mid-task visibility".
4. **Attribution integrity** — write `answered_by: "planner"` on any DQ entry you self-resolve. Never write `answered_by: "advisor"`.
5. **Anchor uniqueness gate (mandatory before finalising §13 IMPLEMENT)** — for every `old_string` Edit anchor in Tasks 2 and 3, confirm `grep -c '<anchor>' crates/server/tests/e2e.rs` returns `1`. If any anchor is non-unique, revise it until unique. Document the confirmed-unique anchors in plan §13.
6. **Shape G suspended** — plan §15 DoD must use validate-pending-laptop path: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"`. No `gh workflow run`. No validate-pending DQ entries.
7. **plan.template.md §5 complexity score** — after scoring, if T3 (new DISABLE gate tests in e2e.rs) scores > 8, add a split-or-proceed DQ in pending[] for the advisor.
8. **MIRROR-ref discipline** — plan §13 IMPLEMENT sections for e2e.rs edits must cite the sibling test line range (e.g. `MIRROR: crates/server/tests/e2e.rs:17592-17731` for the PARTICIPATION_JOB pattern). Required by impl-task subagent spec.
9. **§16a stories** — every task must have a verifiable story (path, expected output, grep-level structural check). For `.coderabbit.yaml`: "grep 'EXACTLY 11' returns 0 matches". For e2e.rs tasks: test function names must exist and pass.
10. **HANDOVER trailer** — commit body must include the HANDOVER trailer per the bm-task brief template discipline (not bm-task; but plan commit should include a `PLAN_COMPLETE:` line summarising the task count + DoD commands, per plan.template.md).

---

## HANDOVER

This brief is complete. Dispatch as:

```
[role:planning] v1-quality-r3c — see .claude/PRPs/briefs/v1-quality-r3c-planning-1.md
```

Base branch: `governance-v0` (commit `ff97b84cf` or later — the worker forks from daemon-local HEAD, which should have been ff'd to origin by the time the task runs).

Advisor gates after planning completes:
1. DoD smoke test (every §15 command literally)
2. Watchpoint specificity gate (§4 must cite specific table/file/function)
3. Anchor uniqueness gate (every e2e.rs Edit anchor, grep-confirmed = 1)
4. User gate 1 (plan approval)
