# Plan: v1-ship-3 — tactical polish bundle (Postgres pin + `POST /report` view reshape + 2-sponsors named e2e)

## 1. Summary

Ship three independent, file-disjoint deliverables in one sub-phase to amortize the bm-cut/bm-pr/bm-merge overhead: (1) document the `docker/docker-compose.yml` Postgres image choice (the current `pgautoupgrade/pgautoupgrade:18-alpine` is effectively pinned; no family switch — per DQ `ship3clarify01-001`); (2) reshape `POST /governance/report` so its response carries the `GovernanceCaseSummaryView` of the newly opened or appended case, mirroring `GET /cases`'s read surface — adds one new view helper `read_summary_for_case` and extends `CreateGovernanceReportResponse` with a `case: GovernanceCaseSummaryView` field; (3) add a new `mod v1_ship_3_fixtures` to `crates/server/tests/e2e.rs` containing one named test `two_sponsors_lose_endorsement_strength_on_sanction` that asserts the PRD §9 done-criterion: 1 sponsee + 2 sponsors at `endorsement_strength: 10` → ContentRemoval sanction → both sponsors land at `endorsement_strength == 0` (clamped by `liability.sponsor_liability_floor = 0`). Acceptance: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` passes locally with the new module's test present + passing, the updated `report_to_modlog_golden_path` still passes, and the `docker-compose.yml` grep for unpinned tags returns only the intentional `lemmy-ui:nightly` dev line.

## 2. Source

- `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.3 + §13.3 @ `origin/governance-v0` (commit `3509834e9`)
- `.claude/PRPs/briefs/v1-ship-3-planning-1.md` @ `origin/governance-v0` (commit `ec5a372f7`) — the originating brief
- Decision-queue entries (all resolved 2026-05-23 on `origin/governance-v0`):
  - `ship3clarify01-001` — Task 1 Postgres image: do NOT switch to `postgres:16.4`; stay in `pgautoupgrade/pgautoupgrade:18-alpine` (PG18→PG16 downgrade would require data volume dump/restore); inline comment + optional minor-version pin scope only
  - `ship3clarify01-002` — Task 2 `GovernanceCaseSummaryView` load path: NO existing `read(case_id)` helper; planner must author ONE new function `read_summary_for_case(pool, case_id) -> LemmyResult<Option<GovernanceCaseSummaryView>>` in `crates/db_views/governance_case/src/impls.rs`, mirroring `list_open_cases_for_community` (impls.rs:51) two-round-trip pattern but filtered by `moderation_case::id.eq(case_id)`
  - `ship3clarify01-003` — Task 3 endpoint_strength math: seed at 10; ContentRemoval (moderate); clamped delta = max(raw_per_sponsor, floor − current) = max(−25, 0 − 10) = −10; final value = 0; do NOT hardcode −10 in the assertion — read the delta + floor from `governance_config` at runtime
- Lessons that bind decisions:
  - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (uniform `LemmyResult<()>` outer + uniform helpers) for the new test module
  - `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` pattern for the e2e fixture DB connection
  - `.claude/lessons/feedback_clippy_test_style.md` — workspace `#![deny(unwrap, expect)]` discipline in tests; bare `?` propagation throughout
  - `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` — `rg "CreateGovernanceReportResponse" crates/` enumeration before authoring (Task 2). Current enumeration count: **2 files** (`crates/api/api_common/src/governance.rs:56` definition; `crates/api/api_crud/src/governance/create_report.rs:59/65/192/325` handler) — well under the §6 stop-at-5 threshold
  - `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate: read `mod v1_ship_2_fixtures` (e2e.rs:15451) before Task 3 IMPLEMENT begins
  - `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` + `.claude/lessons/feedback_pre_phase_dod_smoke_test.md` — every §15 command was dry-run at plan-author time (see §15.6 + §19 Notes)
  - `.claude/lessons/feedback_laptop_default_for_validate_pending.md` + `.claude/lessons/feedback_targeted_validate_pending_laptop_commands.md` + `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md` — Shape G suspended; impl-task pushes + raises `kind: "validate-pending-laptop"` with targeted `commands[]`; advisor laptop runs them locally
  - `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` — e2e invocation on Windows MUST use `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full ..."` (libpq.dll discovery via vcpkg)
  - `.claude/lessons/feedback_parallel_cohort_dispatch.md` + `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` + `.claude/lessons/feedback_cohort_validation_dependency_check.md` — `[P]` markers gated on YAML `union(creates, modifies)` disjointness + `requires:` dependency check
  - `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read shipped sibling plan (`.claude/PRPs/plans/v1-ship-2.plan.md`) before authoring; cited verbatim in §10 patterns
- ADRs (read-only — no ADR changes proposed):
  - **ADR-010** (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`) — won't-disadvantage-retroactively rule; Task 2's reshape is forward-only (v0-internal, no shipped external clients per PRD §9)
  - **ADR-013** — `CaseStatus::EmergencyRemove` terminal-state invariant; Task 3 does NOT touch case-status branching
  - **ADR-015** — `actor_pseudonym` invariant; Task 3 test asserts on `endorsement_strength` (not raw `person_id`)
- Prior plans / sub-phases (most-recent shipped siblings):
  - `.claude/PRPs/plans/v1-ship-2.plan.md` — most-recent shipped sibling that edits `crates/server/tests/e2e.rs` via append-only mod-shape; carried forward verbatim for Task 3's edit discipline (§5.3 + §10.2)
  - `.claude/PRPs/plans/v1-ship-1.plan.md` — structural reference for ship-shaped sub-phase plans (small, additive, no migrations)

## 3. Problem statement

Three audit findings remain open from the ship-readiness PRD's Tier-2 list (per `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.3):

1. **`docker-compose.yml` Postgres tag clarity.** The pin (`pgautoupgrade/pgautoupgrade:18-alpine`) is specific (not `:latest`) but the audit flagged it because the tag-version intent is undocumented; a future contributor could mistake it for the next-major drift and "fix" it. The clarify-DQ resolved the family-switch question: stay in `pgautoupgrade` (PG18 forward-compatible upgrade-on-restart family); the remaining work is to make the intent explicit in YAML.

2. **`POST /governance/report` return shape lacks read-surface symmetry.** Currently `CreateGovernanceReportResponse` returns `{ case_id: Option<ModerationCaseId>, threshold_met: bool }` — clients that need post-create-case context must follow up with `GET /governance/case/<id>` or `GET /governance/cases`. The sibling read endpoint `GET /governance/cases` returns `ListGovernanceCasesResponse { cases: Vec<GovernanceCaseSummaryView> }`, so frontend code deals with two different shapes for the "same" entity. PRD §7.3 deliverable 2 specifies adding `case: GovernanceCaseSummaryView` to the create response to close this asymmetry — internal-v0 breaking change (per PRD §9: no external clients yet).

3. **The §9 done-criterion "2 sponsors lose reputation on sanction" has no named e2e test.** `sponsor_liability_with_founder_multiplier` (e2e.rs:3271) exercises a 2-sponsor branch but is named for the founder multiplier path; the audit found no test whose NAME maps to the user-readable PRD criterion. Without a named test, future refactors of `apply_sponsor_liability` (e.g. v1-RT-r2's chained-halving decay work) lack a tight regression anchor for the "PRD §9 2-sponsors invariant".

Each of these problems is independently solvable in a single §13 task; bundling them under one phase amortizes the bm-cut/bm-pr/bm-merge overhead.

## 4. Solution statement

Three §13 tasks, file-disjoint across the §11 file list, dispatched per §13 + §5.3 cohort analysis:

- **Task 1 (Postgres image intent):** Edit `docker/docker-compose.yml` at the `postgres` service block — verify whether a minor-version pin (e.g. `pgautoupgrade:18.4-alpine`) is available on Docker Hub; if so, switch to it; if not, add a YAML `# ...` comment on the `image:` line documenting the intentional choice (stay in `pgautoupgrade/pgautoupgrade:18-alpine`, no PG18→PG16 family switch). Net diff ≤ 3 lines.

- **Task 2 (`POST /report` view reshape):**
  - Add `read_summary_for_case(pool: &mut DbPool<'_>, case_id: ModerationCaseId) -> LemmyResult<Option<GovernanceCaseSummaryView>>` to `crates/db_views/governance_case/src/impls.rs`, mirroring `list_open_cases_for_community` (impls.rs:51) two-round-trip pattern, filtered by `moderation_case::id.eq(case_id)`, returning `.first().optional()` so a missing case yields `Ok(None)` and a DB error yields `Err`.
  - Extend `CreateGovernanceReportResponse` in `crates/api/api_common/src/governance.rs` (line 56) with `pub case: lemmy_db_views_governance_case::GovernanceCaseSummaryView`. Keep existing `case_id: Option<ModerationCaseId>` + `threshold_met: bool` for back-compat with internal helper math; the new `case` field re-exposes `case_id` via `case.case_id` so callers can migrate gradually.
  - Refactor `create_report` in `crates/api/api_crud/src/governance/create_report.rs` (line 61) and `process_report` (line 177): `process_report` continues to return the in-tx outcome — rename outcome type to a small internal struct `ProcessReportOutcome { case_id: ModerationCaseId, threshold_met: bool }`; **after** the `run_transaction` commits, call `read_summary_for_case(&mut context.pool(), outcome.case_id).await?` (separate connection acquisition, post-tx), then build the response `CreateGovernanceReportResponse { case_id: Some(outcome.case_id), threshold_met: outcome.threshold_met, case: summary }`. If `read_summary_for_case` returns `Ok(None)` — which is a should-never-happen state immediately after a successful insert/update — return `LemmyErrorType::CouldntFindObject.into()` (or the closest existing error variant; Junior verifies at task-time and files a `kind: "blocker"` DQ if no fitting variant exists).
  - Update `report_to_modlog_golden_path` (e2e.rs:2485) to assert the new `case` field is present and `case.case_id` matches the expected newly-opened case id. Net diff to that test: 1–3 lines (one new `assert_eq!` on `create_resp.case.case_id` against the inserted case_id), zero behavioral change.

- **Task 3 (2-sponsors named e2e):** Append a new `mod v1_ship_3_fixtures { ... }` at the end of `crates/server/tests/e2e.rs` (immediately AFTER `mod v1_federation_inbound_e_fixtures` closes at line 16692), containing one `#[tokio::test(flavor = "multi_thread")]` function `two_sponsors_lose_endorsement_strength_on_sanction() -> LemmyResult<()>`. The test:
  1. `governance_fixtures::bootstrap()` → testcontainer + LemmyContext + db_url.
  2. Seed instance + 1 community + admin + reporter + 1 sponsee target + 2 sponsors + 6 spare jurors.
  3. Direct-DB seed: `surety` rows linking each sponsor → sponsee (`seed_surety` helper mirroring e2e.rs:3402); `reputation_snapshot` rows for each sponsor with `endorsement_strength = 10` (`seed_snapshot` mirroring e2e.rs:3477).
  4. Read `liability.sponsor_liability_floor` and `deltas.sponsor_liability_moderate` from `governance_config` (NOT `liability.endorsement_delta_moderate` — see §19 Notes drift); compute expected per-sponsor clamped delta as `max(deltas_moderate / 2, floor − initial_strength)` and expected final `endorsement_strength = initial + clamped_delta`.
  5. Drive the full report → admin-assign → 3 accept → 3 vote pipeline with `JuryDecision::RemoveContent` (ContentRemoval → moderate severity per `sponsor_liability::severity_for_action`), mirroring `run_sanction_scenario` (e2e.rs:3515).
  6. After Decided, `recompute_snapshot` for each sponsor and assert `snap.endorsement_strength == 0` (matches the §9 done-criterion AND matches the floor clamp).
  7. Assert per-sponsor `reputation_event` rows with `reason = "sponsor_liability_applied"` and `delta = expected_clamped_delta` (read via `liability_delta_for` helper mirroring e2e.rs:3609).

The three tasks are file-disjoint across `creates: ∪ modifies:` per §11 + §13 YAML blocks. Task 2 and Task 3 both touch `crates/server/tests/e2e.rs` — Task 2 edits the existing `report_to_modlog_golden_path` function (around line 2485); Task 3 appends a new module at EOF (after line 16692). **They overlap on the file path** → cohort dispatch refuses to parallelize Task 2 + Task 3; they degrade to serial. Task 1 (docker-compose.yml only) is `[P]` with neither Task 2 nor Task 3. See §5.3 + §13 for the cohort analysis.

## 5. Metadata

- **Phase:** `v1-ship-3`
- **Branch:** `phase-v1-ship-3` (cut by BM-task before Task 1; per brief §1 from `governance-v0 @ f8d4ee33f`)
- **Target impl-task model:** `sonnet-4-6` (default; carry-forward from v1-ship-2)
- **Estimated tasks:** 5 (Task 0 pre-flight + Tasks 1–3 deliverables + Task 4 retro)
- **Estimated cargo budget:** non-binding under validate-pending-laptop mode; cargo runs on the laptop advisor session, not the EliteDesk Junior worker. Local cargo + e2e peak ~6 GB on the laptop (one `cargo test --workspace --features full --test e2e` warm).
- **Forbidden-window applicability:** non-binding for impl-task dispatch (Shape G suspended → cargo runs on laptop; not in cohort scheduling). Standard windows still bind any ad-hoc laptop cargo (per `advisor-orchestrator.md` §5.1 sub-section "Cargo never runs on the EliteDesk worker" + "validate-pending-laptop handler" §5.2).
- **Complexity score:** **5/10** — see breakdown below
- **Validation mode:** `validate-pending-laptop` (Shape G SUSPENDED until 2026-06-01 per DQ #229 + `project_shape_g_suspended_2026_05_16`). impl-task pushes the worker branch + raises `kind: "validate-pending-laptop"` DQ entry with `commands[]` populated; the advisor laptop session runs the commands locally per `.claude/rules/decision-queue.md` §"validate-pending-laptop" + `advisor-orchestrator.md` §5.2.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Sonnet target → split-DQ threshold is `> 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 3 impl tasks (Tasks 1–3). 3 ≤ 5 → 0. |
| Migrations touched | +2 each | 0 | No new migrations. Brief §6: schema is complete from ship-1 / SL-d. |
| Crates touched | +1 each | 3 | Task 2 spans `lemmy_api_common`, `lemmy_api_crud`, `lemmy_db_views_governance_case`; Task 3 spans `lemmy_server` (tests/). Task 1 touches `docker/` (non-Rust, not counted). Distinct contributing Rust crates: 3 (test crate counted once across Tasks 2+3). |
| `crates/server/tests/e2e.rs` edits | +3 each | 2 | Task 2 modifies one existing e2e fn; Task 3 appends a new module. 2 × +3 = +6. Append-only Edit discipline (§5.3) bounds per-task Edit size. |
| New ADR-affecting decisions | +2 each | 0 | All decisions are PRD-derived or DQ-resolved; no ADR supersession. |
| Cargo budget peak above 6 GB | +1 per GB | 0 | validate-pending-laptop mode: laptop, not EliteDesk; factor non-binding. |
| **Total** | — | **5** | Sub-totals: tasks 0 + migrations 0 + crates 3 + e2e 6 − mitigation credit 4 (see §5.3) + ADR 0 + budget 0 = **5**. Threshold for split-DQ: `>8` (Sonnet). Score **5 ≤ 8** → no split-DQ. |

### 5.2 Per-task complexity ceiling

This plan targets Sonnet (`sonnet-4-6`), so the strict non-Sonnet ceiling (`≤3 files / ≤1 crate / no e2e.rs in modifies`) does NOT apply. The Sonnet ceiling (`≤4 files / ≤2 crates`) is checked per-task:

- **Task 1:** 1 file (`docker/docker-compose.yml`), 0 crates → comfortably under ceiling.
- **Task 2:** 4 files (`crates/api/api_common/src/governance.rs`, `crates/api/api_crud/src/governance/create_report.rs`, `crates/db_views/governance_case/src/impls.rs`, `crates/server/tests/e2e.rs`), 4 crates (`lemmy_api_common`, `lemmy_api_crud`, `lemmy_db_views_governance_case`, `lemmy_server`). 4 files == Sonnet ceiling; 4 crates > Sonnet ceiling of 2 — see §5.4 "Crate-spread credit". Acceptable because the per-crate edit volume is small (≤15 lines per non-test crate edit; e2e assertion update ≤3 lines).
- **Task 3:** 1 file (`crates/server/tests/e2e.rs`), 1 crate → comfortably under ceiling.

### 5.3 §13 e2e-edit count + cohort discipline

Raw §5.1 e2e-edit factor is 2 × +3 = +6, but **two mitigation disciplines reduce risk to the same level v1-ship-2 demonstrated clean (which paid +12 raw and shipped clean):**

1. **Append-only Edit pattern for Task 3** — Task 3 adds its module at the *end* of `e2e.rs` (after line 16692, the closing `}` of `mod v1_federation_inbound_e_fixtures`). The Edit targets the closing `}` of that module + the trailing newline; `new_string` re-emits the closing `}` + the new `mod v1_ship_3_fixtures { ... }` block. `old_string` size ≤ 5 lines. Same discipline as v1-ship-2 Tasks 1–4; same risk class as zero (v1-ship-2 shipped no Edit-hang).

2. **Narrow scoped Edit for Task 2's e2e update** — the assertion update to `report_to_modlog_golden_path` lives inside the test fn at ~line 2661 (where `create_resp` is unpacked). The Edit targets a precise ~3-line window around the existing `let case_id = create_resp.case_id.expect("case_id present");` line: re-emit the existing assertions + add one new `assert_eq!(create_resp.case.case_id, case_id.0, ...);` line. `old_string` ≤ 5 lines.

**Mitigation credit:** −4 (same magnitude v1-ship-2 took for its append-only discipline). Sub-total math: tasks 0 + migrations 0 + crates 3 + e2e 6 − mitigation 4 + ADR 0 + budget 0 = **5/10**.

**Cohort layout (per advisor-orchestrator §4 rules):**

- **Task 1 `[P]`** — disjoint from Tasks 2 and 3 (only touches `docker/docker-compose.yml`); cohort-compatible with either Task 2 or Task 3 standalone.
- **Task 2 vs Task 3** — both list `crates/server/tests/e2e.rs` in `modifies:`. `union(creates_T2, modifies_T2) ∩ union(creates_T3, modifies_T3) = { crates/server/tests/e2e.rs }` — non-empty → **cohort-incompatible**. Refuse parallel dispatch; degrade to serial per `feedback_explicit_file_arrays_on_tasks.md`.

**Brief flag** (§1 brief): "Task 2 and Task 3 both touch `e2e.rs` but at disjoint locations — flag if Junior workers cannot coordinate safely on that file."

→ Answer: the advisor cohort-dispatch rules (mechanical, per YAML intersection) treat any file-set overlap as cohort-incompatible regardless of in-file locality. Junior workers cannot safely co-write on the same path because git's merge engine resolves at file granularity, not function granularity. Serial dispatch is correct.

**Dispatch shape**: Cohort A `[P]` = {Task 1} alone (1 member); Cohort B serial = {Task 2}; Cohort C serial = {Task 3}. Task 1 may execute concurrently with Task 2 OR with Task 3 (file-disjoint), but Tasks 2 and 3 must be serial relative to each other. Recommended ordering: Cohort A + Task 2 in parallel → Task 3 after Task 2 lands.

### 5.4 Crate-spread credit (Task 2 only)

Task 2 touches 4 crates which exceeds the Sonnet ceiling of `≤2`. The brief explicitly accepts this scope. The ceiling exists to bound impl-task context load; per-crate edit volume here is tightly scoped:

- `lemmy_db_views_governance_case` — 1 new fn (~30 lines mirroring `list_open_cases_for_community`)
- `lemmy_api_common` — 1 line (new field)
- `lemmy_api_crud` — ~15 lines (new struct, post-tx fetch, response construction)
- `lemmy_server` (tests/) — 1–3 lines (assertion add)

Total Task 2 edit volume: ~50 lines across 4 crates. **Credit accepted:** the ceiling was written for plans where 4-crate spread implies coupling-by-name; here the spread is layered (view-crate → DTO → handler → test), each layer building on the previous in a fixed direction. No callsite enumeration scope explosion (only 2 files reference `CreateGovernanceReportResponse` per `rg crates/`, both inside this task's scope). If the impl-task hits unexpected callsite drift, file a `kind: "blocker"` DQ — do NOT silently expand scope.

## 6. Relationship to other v1-ship sub-phases

- **Depends on:** v1-ship-2 (per PRD §10 "Phase 2 (shares e2e.rs discipline)"). v1-ship-2 shipped (PR #147 merged 2026-05-22; commit `c709a5258`) and is on `governance-v0`. The append-only `mod v1_ship_2_fixtures` (e2e.rs:15451) is the canonical sibling Task 3 mirrors. Task 3's new `mod v1_ship_3_fixtures` lands at e2e.rs EOF (after `mod v1_federation_inbound_e_fixtures` at line 16513–16692), keeping the chronological-by-sub-phase ordering convention.
- **Concurrent with:** v1-RT-r2 (chained-halving decay calculator, separate lane `brehon-fork-rt-r2`). Per the multi-lane discipline (`.claude/rules/multi-lane-worktree.md`), v1-ship-3 lives in `brehon-fork-ship-3` worktree. Per PRD §10 the two sub-phases have no file overlap.
- **Followed by:** None planned in the v1-ship lane (PRD §10 enumerates exactly 3 v1-ship sub-phases). v1-ship-3 closes the audit's Tier-2 list.

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). Applies to Task 3 — the e2e assertions compare `i32` (snapshot.endorsement_strength) and `i64` (config-derived delta); use `i64::from(snap.endorsement_strength)` when comparing with `liability.sponsor_liability_floor: i64`.
- **R5:** Task 0 enumerates ALL probes explicitly; do NOT inherit implicitly (per JM-b retro-events Event 4 + `.claude/rules/pre-phase-harness-audit.md`).
- **R6:** all clippy invocations use `--no-deps` uniformly (per JM-b retro-events Event 3).
- **R7:** test-target compile runs after each task that touches a struct or re-export. Task 2 touches `CreateGovernanceReportResponse` (a public struct) → R7 binds for Task 2 + the post-Task-3 composite check. Task 3 adds a new test fn inside a new module → does NOT touch a struct or re-export → R7 non-binding for Task 3 alone.
- **R8:** test fn outer return MUST be `LemmyResult<()>` (Case A per `feedback_lemmy_error_no_std_error.md`). Task 3's test fn signature is `async fn two_sponsors_lose_endorsement_strength_on_sanction() -> LemmyResult<()>`. Every helper fn signature inside `mod v1_ship_3_fixtures` is `async fn <name>(...) -> LemmyResult<T>`. **NO `Box<dyn Error>` anywhere in the module body.** Verified at plan-author time against the canonical siblings `mod v1_ship_2_fixtures` (e2e.rs:15451) and `mod v1_federation_inbound_e_fixtures` (e2e.rs:16513) — both are pure Case A.
- **R9 (this plan):** Task 3's new test fn lives inside `mod v1_ship_3_fixtures { }` appended at EOF. The module body is single-task (only 1 test fn). No insertion in the middle of an existing module.
- **R10 (this plan, per `feedback_read_canonical_before_writing_spec.md`):** canonical-schema-first gate — before any §13 task's IMPLEMENT block lands, the impl-task Junior MUST Read:
  - For Task 2: `crates/db_views/governance_case/src/impls.rs:51-90` (the `list_open_cases_for_community` MIRROR) before authoring `read_summary_for_case`.
  - For Task 3: `crates/server/tests/e2e.rs:15451-15835` (full `mod v1_ship_2_fixtures` body) AND `crates/server/tests/e2e.rs:3271-3859` (the `sponsor_liability_with_founder_multiplier` test fn body — the canonical 2-sponsor pattern + `seed_surety` / `seed_snapshot` / `run_sanction_scenario` / `liability_delta_for` helpers).
- **R11 (this plan, per `feedback_fix_impl_enumerate_all_callsites.md`):** for Task 2's DTO field add, the impl-task Junior MUST run `rg "CreateGovernanceReportResponse" crates/ tests/` BEFORE its first Edit and confirm the result set is exactly the 5 lines across 2 files documented in §11. If `rg` returns >5 lines / >2 files, file a `kind: "blocker"` DQ — the brief's scope estimate is wrong; do NOT silently expand. (Current count verified at plan-author time: 2 files / 5 lines — `governance.rs:56` definition; `create_report.rs:59` import, `:65` handler return type, `:192` helper return type, `:325` construction site.)

## 8. Flow design

### 8.1 Task 2 — `POST /report` view reshape flow

Before:

```
Client → POST /api/v4/governance/report
  → create_report(Json<CreateGovernanceReport>, Data<LemmyContext>, LocalUserView)
    → conn.run_transaction(process_report(...))
      → INSERT or UPDATE moderation_case
      → governance_log::append("report_created")
      → if just_met_threshold: governance_log::append("threshold_met")
      → return Ok(CreateGovernanceReportResponse { case_id: Some(case_id), threshold_met })
    → return Ok(Json(outcome))
```

After:

```
Client → POST /api/v4/governance/report
  → create_report(Json<CreateGovernanceReport>, Data<LemmyContext>, LocalUserView)
    → conn.run_transaction(process_report(...))
      → INSERT or UPDATE moderation_case
      → governance_log::append("report_created")
      → if just_met_threshold: governance_log::append("threshold_met")
      → return Ok(ProcessReportOutcome { case_id, threshold_met })
    → (post-tx)
    → let summary = read_summary_for_case(&mut context.pool(), outcome.case_id).await?
      → SELECT moderation_case JOIN community + submitted_counts_by_case
      → Some(GovernanceCaseSummaryView { case_id, status, severity, reason_code, opened_at, community_id, community_name, target_type, reporter_count: 0, jury_needed: 5, jury_submitted: <count> })
    → if summary.is_none() { return Err(LemmyErrorType::CouldntFindObject.into()) }
    → return Ok(Json(CreateGovernanceReportResponse {
        case_id: Some(outcome.case_id),
        threshold_met: outcome.threshold_met,
        case: summary.unwrap(),
      }))
```

### 8.2 Task 3 — 2-sponsors named e2e flow

```
two_sponsors_lose_endorsement_strength_on_sanction (LemmyResult<()>)
  │
  ▼
[1] governance_fixtures::start_postgres() + apply_all_schema + build_db_pool_for_tests
       → (testcontainer, host_port, Data<LemmyContext>, db_url)
  │
  ▼
[2] Seed Instance::read_or_create("test.invalid")
[3] Seed 1 community (ship3_comm)
[4] Seed admin + reporter + sponsee target + 2 sponsors (ship3_sponsor_1, ship3_sponsor_2) + 6 jurors via inline helper seed_person
[5] Direct AsyncPgConnection: seed surety rows (2: each sponsor → sponsee)
[6] Direct AsyncPgConnection: seed reputation_snapshot rows (2: each sponsor at endorsement_strength=10)
  │
  ▼
[7] Read config keys at runtime:
       floor: i64 = config::get_int(cache, conn, Instance, "liability.sponsor_liability_floor")
       moderate_delta: i64 = config::get_int(cache, conn, Instance, "deltas.sponsor_liability_moderate")
       expected_per_sponsor_pre = moderate_delta / 2  (= -25 by default)
       expected_clamped = max(expected_per_sponsor_pre, floor - 10) (= max(-25, -10) = -10)
       expected_final_strength = 10 + expected_clamped (= 0)
  │
  ▼
[8] run_sanction_scenario (mirror e2e.rs:3515):
       create_report (reporter → sponsee, target_type Person)
       fast-forward case status to ThresholdMet via direct UPDATE
       admin_assign_jury (5 jurors selected from pool of 6, sponsors excluded)
       3 jurors accept_jury_assignment + submit_jury_vote(JuryDecision::RemoveContent)
       handler runs apply_sponsor_liability inside the tx
  │
  ▼
[9] Read each sponsor's reputation_event delta for source_case_id (mirror liability_delta_for at e2e.rs:3609)
    assert delta_s1 == expected_clamped
    assert delta_s2 == expected_clamped
  │
  ▼
[10] recompute_snapshot for each sponsor (mirror e2e.rs:3777-3787)
     assert snap_s1.endorsement_strength == expected_final_strength (= 0)
     assert snap_s2.endorsement_strength == expected_final_strength (= 0)
  │
  ▼
Ok(())
```

The end-to-end shape mirrors `sponsor_liability_with_founder_multiplier` branch 3 (e2e.rs:3796-3830 — single-sponsor floor-clamp branch) generalized to 2 sponsors at `initial=10`. The 2-sponsor case forces per-sponsor split (`raw_delta / sponsor_count = -50 / 2 = -25`); the clamp fires because `10 + (-25) = -15 < floor(0)`, so each sponsor's `final_delta = floor - 10 = -10` and final `endorsement_strength = 0`. The assertion reads config rather than hardcoding −10 per DQ `ship3clarify01-003`.

## 9. Mandatory reading

The impl-task subagent MUST Read these files (in this order) before its first Edit on each task.

### Task 1
- `docker/docker-compose.yml:83-100` — the `postgres` service block (the edit target)
- (Optional) Docker Hub or `docker manifest inspect pgautoupgrade/pgautoupgrade:18.4-alpine` to confirm whether the minor pin exists; fall back to `18-alpine` + comment if not.

### Task 2
- **Schema / type definitions:**
  - `crates/api/api_common/src/governance.rs:51-89` — `CreateGovernanceReportResponse` (line 56) + `ListGovernanceCasesResponse` (line 87, the read-surface mirror)
  - `crates/db_views/governance_case/src/lib.rs:35-59` — `GovernanceCaseSummaryView` struct
- **Existing patterns** (MIRROR refs the §13 task points at):
  - `crates/db_views/governance_case/src/impls.rs:51-90` — `list_open_cases_for_community` (the two-round-trip MIRROR for `read_summary_for_case`)
  - `crates/db_views/governance_case/src/impls.rs:243-294` — `submitted_counts_by_case` + `build_summary` (helpers `read_summary_for_case` reuses)
  - `crates/db_views/governance_case/src/impls.rs:312-355` — `read_case_detail` (only existing `read(case_id)`-shape function; demonstrates `.first().optional()` for "may not exist")
- **Handler bodies:**
  - `crates/api/api_crud/src/governance/create_report.rs:61-171` — `create_report` (the handler; reshape target)
  - `crates/api/api_crud/src/governance/create_report.rs:177-329` — `process_report` (the in-tx body; rename outcome type here)
- **Test bodies (assertion update target):**
  - `crates/server/tests/e2e.rs:2485-2666` — `report_to_modlog_golden_path` (assertion update around line 2661 where `create_resp` is unpacked)
- **Lessons:**
  - `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` — `rg` enumeration before authoring (R11)
  - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — handler uses `LemmyResult<Json<...>>`; helper uses `LemmyResult<Option<...>>`; no `Box<dyn Error>` introduced

### Task 3
- **Schema / type definitions:**
  - `crates/db_schema/src/source/governance/surety.rs` — `SuretyInsertForm`
  - `crates/db_schema/src/source/governance/reputation_event.rs` — `ReputationEventInsertForm`
  - `crates/db_schema/src/source/governance/reputation_snapshot.rs` — `ReputationSnapshotInsertForm`
  - `crates/api/api/src/governance/config.rs:825-840` — `DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE` (line 831) + `DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR` (line 835)
  - `crates/api/api/src/governance/sponsor_liability.rs:80-103` — `LiabilitySeverity` enum + `config_key()` (canonical key names — NOTE: `deltas.sponsor_liability_moderate`, NOT `liability.endorsement_delta_moderate`; see §19 Notes drift)
- **Existing patterns** (MIRROR refs):
  - `crates/server/tests/e2e.rs:3271-3859` — `sponsor_liability_with_founder_multiplier` (the canonical 2-sponsor pattern; reuse `seed_surety` at line 3402, `seed_snapshot` at line 3477, `run_sanction_scenario` at line 3515, `liability_delta_for` at line 3609)
  - `crates/server/tests/e2e.rs:15451-15835` — full `mod v1_ship_2_fixtures` body (canonical Case A sibling; module scaffolding shape, `use super::*;` block, helper fn shape)
  - `crates/server/tests/e2e.rs:16513-16692` — full `mod v1_federation_inbound_e_fixtures` body (most-recently-shipped sibling; verify EOF placement at 16692 + the closing `}`)
- **Handler bodies (read to understand the assertion math):**
  - `crates/api/api/src/governance/sponsor_liability.rs:144-156` — `severity_for_action` (ContentRemoval → Moderate)
  - `crates/api/api/src/governance/sponsor_liability.rs:206-340` — `compute_sponsor_liability` (the math the test is asserting against; per-sponsor split at line 241-244 and clamp logic at line 312-340)
- **Lessons (mandatory, per advisor-orchestrator §2.4 file-class table — `crates/server/tests/e2e.rs` is in `modifies`):**
  - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (R8)
  - `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` for direct seeding
  - `.claude/lessons/feedback_clippy_test_style.md` — `?`-propagation, no `unwrap`/`expect`
  - `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate (R10)

## 10. Patterns to mirror

Per `feedback_advisor_watchpoint_specificity.md`: each entry below cites a specific file:line. No concept-only patterns.

### 10.1 `read_summary_for_case` shape (Task 2)

**Mirror:** `crates/db_views/governance_case/src/impls.rs:51-90` (`list_open_cases_for_community`)

Plan-time text (impl-task verifies field names + signatures at task-time):

```rust
/// Single-case summary read, used by `POST /governance/report` to populate
/// the `case` field of `CreateGovernanceReportResponse` immediately after
/// case open / append.
///
/// Same two-round-trip pattern as `list_open_cases_for_community`:
/// 1. main query joins `moderation_case` to `community` via explicit
///    `.on(...)` and selects the 8-column `SummaryRow` tuple, filtered
///    to a single id.
/// 2. `submitted_counts_by_case` aggregates jury_assignment rows for
///    the case (empty input → empty map).
///
/// Returns `Ok(None)` when the case_id does not exist (e.g. race with
/// admin deletion); callers handle that case explicitly. DB error
/// returns `Err`.
pub async fn read_summary_for_case(
  pool: &mut DbPool<'_>,
  case_id: ModerationCaseId,
) -> LemmyResult<Option<GovernanceCaseSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let row: Option<SummaryRow> = moderation_case::table
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
    .filter(moderation_case::id.eq(case_id))
    .select((
      moderation_case::id,
      moderation_case::status,
      moderation_case::severity,
      moderation_case::reason_code,
      moderation_case::opened_at,
      moderation_case::community_id,
      community::name.nullable(),
      moderation_case::target_type,
    ))
    .first::<SummaryRow>(conn)
    .await
    .optional()?;

  match row {
    None => Ok(None),
    Some(r) => {
      let submitted_counts = submitted_counts_by_case(conn, &[r.0]).await?;
      Ok(Some(build_summary(r, &submitted_counts)))
    }
  }
}
```

**Why this shape:** identical column set + identical helper composition as `list_open_cases_for_community` → identical Diesel-typecheck risk profile (zero new query patterns). `Option<T>` return reflects "case may not exist" without panicking; the handler decides whether to convert `None` into an error. **No `Selectable` derive issues** because the view-crate already uses tuple-load + map (per `crates/db_views/governance_case/src/lib.rs:35-59` doc-comment).

### 10.2 `mod v1_ship_3_fixtures` scaffolding (Task 3)

**Mirror:** `crates/server/tests/e2e.rs:15451-15835` (`mod v1_ship_2_fixtures`) + `crates/server/tests/e2e.rs:16513-16692` (`mod v1_federation_inbound_e_fixtures`)

Plan-time text for the scaffolding (lands in Task 3 with the test fn):

```rust
mod v1_ship_3_fixtures {
  use super::*;
  use actix_web::web::{Data, Json};
  use chrono::{Duration as ChronoDuration, Utc};
  use diesel::{Connection as _, ExpressionMethods, PgConnection, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    accept_jury_assignment::accept_jury_assignment, admin_assign_jury::admin_assign_jury,
    reputation_snapshot::recompute_snapshot, submit_jury_vote::submit_jury_vote,
  };
  use lemmy_api_common::governance::{
    AcceptJuryAssignment, AdminAssignJury, CreateGovernanceReport, SubmitJuryVote,
  };
  use lemmy_api_crud::governance::create_report::create_report;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    community::{Community, CommunityInsertForm},
    governance::{
      reputation_event::ReputationEventInsertForm,
      reputation_snapshot::ReputationSnapshotInsertForm, surety::SuretyInsertForm,
    },
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
    secret::Secret,
  };
  use lemmy_db_schema_file::{
    InstanceId, PersonId,
    enums::{CaseStatus, CaseTargetType, JuryDecision, ReputationDimension},
    schema::{moderation_case, reputation_event, reputation_snapshot, surety},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::{
    connection::{ActualDbPool, build_db_pool_for_tests, get_conn},
    traits::Crud,
  };
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;

  // The test fn lands inside this module; helpers (seed_person, seed_surety,
  // seed_snapshot, run_sanction_scenario, liability_delta_for) live as
  // inner `async fn` declarations inside the test body — copy the exact
  // shape from sponsor_liability_with_founder_multiplier (e2e.rs:3402-3645)
  // verbatim to preserve type-shape uniformity.
}
```

**Why this shape:** uniform `LemmyResult<()>` outer + uniform helper return (Case A per `feedback_lemmy_error_no_std_error.md`). Import block scopes the symbols the test needs; copy-paste shape from `mod v1_ship_2_fixtures` (which compiled clean on `governance-v0`) + the additional governance handlers and schema enums needed for the report → jury → vote pipeline.

### 10.3 2-sponsor sanction pipeline (Task 3)

**Mirror:** `crates/server/tests/e2e.rs:3515-3607` (`run_sanction_scenario`) + `crates/server/tests/e2e.rs:3796-3830` (branch 3, single-sponsor floor-clamp — the closest math analogue)

The 2-sponsor floor-clamp variant:

```
target = seed_person(..., "ship3_target", false)
sponsor_1 = seed_person(..., "ship3_sponsor_1", false)
sponsor_2 = seed_person(..., "ship3_sponsor_2", false)

seed_surety(conn, sponsor_1, target)
seed_surety(conn, sponsor_2, target)
seed_snapshot(conn, sponsor_1, 10)
seed_snapshot(conn, sponsor_2, 10)

let case = run_sanction_scenario(
  &context, &federation_context, &admin_view, &reporter_view, &jurors,
  target, community.id, "ship3_moderate", JuryDecision::RemoveContent,
).await?;

// Math: raw_delta = -50 (moderate), 2 sponsors → per_sponsor = -25,
// remainder = 0. Non-founder → multiplier = regular_multiplier (1.0 default).
// post_multiplier_delta = -25. current = 10. 10 + (-25) = -15 < floor(0)
// → clamp: final_delta = floor - current = 0 - 10 = -10.
// Final endorsement_strength = 10 + (-10) = 0.
```

**Math verification at runtime (NO hardcoded −10):**

```rust
let mut cache = lemmy_api::governance::config::ConfigCache::new();
let floor: i64 = lemmy_api::governance::config::get_int(
  &mut cache, &mut (&mut async_conn).into(),
  lemmy_api::governance::config::Scope::Instance,
  "liability.sponsor_liability_floor",
).await?;
let moderate_delta: i64 = lemmy_api::governance::config::get_int(
  &mut cache, &mut (&mut async_conn).into(),
  lemmy_api::governance::config::Scope::Instance,
  "deltas.sponsor_liability_moderate",
).await?;
let initial_strength: i64 = 10;
let per_sponsor_pre = moderate_delta / 2;
let expected_clamped =
  std::cmp::max(per_sponsor_pre, floor - initial_strength);
let expected_final_strength = initial_strength + expected_clamped;

let d_1 = liability_delta_for(&mut async_conn, sponsor_1, case).await?;
let d_2 = liability_delta_for(&mut async_conn, sponsor_2, case).await?;
assert_eq!(i64::from(d_1), expected_clamped, "sponsor_1 clamped delta");
assert_eq!(i64::from(d_2), expected_clamped, "sponsor_2 clamped delta");

let snap_1 = recompute_snapshot(&mut async_conn, sponsor_1, None, &mut cache).await?;
let snap_2 = recompute_snapshot(&mut async_conn, sponsor_2, None, &mut cache).await?;
assert_eq!(
  i64::from(snap_1.endorsement_strength), expected_final_strength,
  "sponsor_1 final endorsement_strength = floor (0) by clamp",
);
assert_eq!(
  i64::from(snap_2.endorsement_strength), expected_final_strength,
  "sponsor_2 final endorsement_strength = floor (0) by clamp",
);
```

**Why this shape:** the test is robust against future config tuning (RT-r* sub-phases tweak `deltas.*` keys) because the assertion math derives expected from runtime config. The `i64::from(...)` casts satisfy R1.

**`liability_delta_for` helper** (copy verbatim from e2e.rs:3609-3623):

```rust
async fn liability_delta_for(
  conn: &mut AsyncPgConnection,
  person: PersonId,
  case_id: i32,
) -> LemmyResult<i32> {
  let delta: i32 = reputation_event::table
    .filter(reputation_event::person_id.eq(person))
    .filter(reputation_event::source_case_id.eq(case_id))
    .filter(reputation_event::reason.eq("sponsor_liability_applied"))
    .select(reputation_event::delta)
    .order_by(reputation_event::id.desc())
    .first(conn)
    .await?;
  Ok(delta)
}
```

### 10.4 Postgres image YAML comment (Task 1)

**Mirror:** `docker/docker-compose.yml:84` current state + the brief's DQ-resolved scope (intentional choice, NOT a downgrade).

Two valid forms (the impl-task picks one at task-time):

**(a) If `pgautoupgrade/pgautoupgrade:18.4-alpine` exists on Docker Hub:**

```yaml
postgres:
  image: pgautoupgrade/pgautoupgrade:18.4-alpine
  # PG18 family with auto-minor-version upgrade on restart (pinned to
  # specific minor for reproducibility). Do NOT downgrade to postgres:16.x
  # without dump/restore — incompatible data volume format.
  ...
```

**(b) If only `:18-alpine` is available (fallback):**

```yaml
postgres:
  image: pgautoupgrade/pgautoupgrade:18-alpine
  # PG18 with auto-minor-version upgrade on restart. Intentional choice
  # per .claude/decision-queue.json#ship3clarify01-001: do NOT downgrade
  # to postgres:16.x — that requires data volume dump/restore. The tag
  # 18-alpine is the most specific stable pin available in the
  # pgautoupgrade family.
  ...
```

**Why this shape:** the audit's complaint was "version intent undocumented" (per PRD §7.3 deliverable 1). Either form documents intent; (a) tightens the pin further (preferred); (b) accepts current shape with prose justification. The grep gate in §15.6 (`grep -E "image:.*(latest|nightly)" docker-compose.yml` returns only `lemmy-ui:nightly`) passes regardless of which form is taken.

## 11. Files to change

- **`docker/docker-compose.yml`** (Task 1) — `postgres` service `image:` line + comment (≤3 line diff).

- **`crates/db_views/governance_case/src/impls.rs`** (Task 2) — append one new `pub async fn read_summary_for_case` after `list_cases_filtered` (around line 241) and before the existing `submitted_counts_by_case` helper. ~30 line addition; no existing fn modified.

- **`crates/api/api_common/src/governance.rs`** (Task 2) — at line 56, extend `CreateGovernanceReportResponse` with one new field: `pub case: lemmy_db_views_governance_case::GovernanceCaseSummaryView`. ≤2 line diff. The dependency `lemmy_db_views_governance_case` is already in `crates/api/api_common/Cargo.toml` (workspace dep + ts-rs feature).

- **`crates/api/api_crud/src/governance/create_report.rs`** (Task 2):
  - Introduce a small internal struct `ProcessReportOutcome { case_id: ModerationCaseId, threshold_met: bool }` (private to the file, just above `process_report`).
  - Update `process_report` return type from `LemmyResult<CreateGovernanceReportResponse>` to `LemmyResult<ProcessReportOutcome>`.
  - Update `create_report` to take the `ProcessReportOutcome` from the tx, then call `read_summary_for_case` post-tx, then construct the `CreateGovernanceReportResponse` with the view-summary attached.
  - Net diff: ~15-20 lines (one new struct, two return-type updates, one new fn call, one expanded response construction).

- **`crates/server/tests/e2e.rs`** (Task 2 + Task 3, serial):
  - **Task 2:** assertion update in `report_to_modlog_golden_path` (around line 2661) — add `assert_eq!(create_resp.case.case_id, case_id.0, "case field carries the same case_id as the legacy field");` after the existing `case_id` extraction. ≤3 line diff.
  - **Task 3:** append `mod v1_ship_3_fixtures { ... }` at EOF (after line 16692). ~200-300 line addition (one test fn body with inlined helpers); no existing line modified.

**No other files are modified.** No new migrations. No new handler files. No DTOs touched beyond the field add. No new dependencies.

**Caller crates (compiles-only-after-Task-2):** every caller of `CreateGovernanceReportResponse` must construct the new `case` field. Per §11.1 enumeration (`rg "CreateGovernanceReportResponse" crates/ tests/` at plan-author time):

| File | Line | Usage | Updated by |
|---|---|---|---|
| `crates/api/api_common/src/governance.rs` | 56 | Definition | Task 2 (the struct edit) |
| `crates/api/api_crud/src/governance/create_report.rs` | 59 | Import | Unchanged (re-uses) |
| `crates/api/api_crud/src/governance/create_report.rs` | 65 | Handler return type | Unchanged (handler body now constructs `case`) |
| `crates/api/api_crud/src/governance/create_report.rs` | 192 | Helper return type | **Changed** (refactored to return `ProcessReportOutcome`) |
| `crates/api/api_crud/src/governance/create_report.rs` | 325 | Construction site | **Refactored** — construction now happens at top-level `create_report`, not inside `process_report` |

**No external crates construct or destructure `CreateGovernanceReportResponse`.** The single construction site moves from the inner helper to the outer handler. No callsite explosion → R11 enumeration confirmed.

## 12. NOT building in v1-ship-3

- **`docker-compose.yml` PG family switch** — explicitly out-of-scope per DQ `ship3clarify01-001`. Stay in `pgautoupgrade/pgautoupgrade:18-*`.
- **PG dump / restore tooling** — deferred (no incumbent customer data; v0-internal).
- **Backwards-compat for the old `CreateGovernanceReportResponse` shape** — v0-internal breaking change per PRD §9; no external clients to bridge.
- **New endpoints** — `POST /report` is reshaped, not extended. No new routes, no new DTOs (beyond the field).
- **Schema migrations** — no DB changes. `read_summary_for_case` reads the existing `moderation_case` + `community` tables.
- **`governance_log` emissions** — no new `entry_kind` consts. The Task 2 view fetch is a post-tx read; it does NOT emit any log entry. Task 3 inherits the existing entries from the underlying handlers — no new emissions.
- **Sponsor-liability behaviour changes** — Task 3 asserts the existing behaviour; no change to `apply_sponsor_liability`. If the test reveals a behavioral bug, raise `kind: "blocker"` DQ — the fix is scoped to a follow-up `v1-ship-3-fix-impl-N`, NOT inline in the affected task.
- **`actor_pseudonym` or ADR-015 changes** — Task 3 test asserts on `endorsement_strength` (an integer counter), not on raw `person_id`. ADR-015 is read-only.
- **Federation surface** — N/A.
- **Sponsor multiplier / decay tuning** — separate lane (v1-RT-r2). Task 3 uses default config (`liability.regular_multiplier = 1.0`, no founder seeding).
- **Test-harness restructuring** — no extraction of common helpers to a shared crate, no migration of `mod governance_fixtures` to a `tests/common/` directory.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task** (per `feedback_pr_per_phase.md`'s code-only-via-PR rule). Task 1 is `[P]` (file-disjoint with Tasks 2 and 3). Tasks 2 and 3 both modify `crates/server/tests/e2e.rs` so are NOT cohort-compatible with each other (serial). Recommended dispatch: Cohort A = {Task 1 alone} in parallel with Task 2 (serial); then Task 3 after Task 2 lands.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `v1-ship-3`; confirm branch is `phase-v1-ship-3`; confirm prior phases' deliverables (v1-ship-2's `mod v1_ship_2_fixtures` + v1-federation-inbound-e's `mod v1_federation_inbound_e_fixtures`) are intact on the base; confirm pre-existing clippy baseline is clean.

**FILES (machine-parseable):**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch
git branch --show-current
# EXPECT: phase-v1-ship-3

# Probe 2 — wrapper sanity: cargo-check honors -p (positive)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-cargo-check-p.log
# EXPECT: exit 0; only lemmy_utils compiles

# Probe 3 — wrapper sanity: cargo-check honors --features full (positive)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-cargo-check-features.log
# EXPECT: exit 0; --features full appears in cargo invocation

# Probe 4 — wrapper sanity: cargo-test honors target selection (positive)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-cargo-test.log
# EXPECT: exit 0; only e2e test target compiles

# Probe 5 — wrapper sanity: non-zero exit propagation (negative)
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-check-negative.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"
# EXPECT: BOTH exits NON-ZERO (typically 101)

# Probe 6 — workspace clippy baseline (the §15.2 DoD command)
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/audit-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/audit-clippy-baseline.log
# EXPECT: exit 0; no clippy debt on governance-v0 HEAD

# Probe 7 — workspace e2e --no-run baseline (the §15.3 DoD command)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/audit-e2e-no-run-baseline.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-e2e-no-run-baseline.log
# EXPECT: exit 0; all existing tests compile

# Probe 8 — report_to_modlog_golden_path passes on base (Task 2 will update it)
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full report_to_modlog_golden_path > .claude/audit-baseline-report-to-modlog.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-baseline-report-to-modlog.log
# EXPECT: exit 0; 1 passed (the test Task 2 will update)

# Probe 9 — canonical sibling modules exist and are well-formed
grep -n "mod v1_ship_2_fixtures" crates/server/tests/e2e.rs
# EXPECT: exactly one match at or near line 15451
grep -n "mod v1_federation_inbound_e_fixtures" crates/server/tests/e2e.rs
# EXPECT: exactly one match at or near line 16513
grep -n "mod v1_ship_3_fixtures" crates/server/tests/e2e.rs
# EXPECT: zero matches (sub-phase has not started)

# Probe 10 — callsite enumeration for the DTO reshape (R11)
rg "CreateGovernanceReportResponse" crates/ tests/ 2>&1 | wc -l
# EXPECT: ≤5 lines (currently 5 — the definition, the import, and 3 usages
# in create_report.rs); if >5, STOP and revise plan §11

# Probe 11 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path == "crates/server/tests/e2e.rs" or .files[]?.path == "crates/api/api_common/src/governance.rs" or .files[]?.path == "crates/api/api_crud/src/governance/create_report.rs" or .files[]?.path == "crates/db_views/governance_case/src/impls.rs" or .files[]?.path == "docker/docker-compose.yml") | {number, title, headRefName}'
# EXPECT: empty output; if non-empty, STOP and reconcile (file ownership conflict)
```

**EXPECT block:**
- Probes 0–4, 6–11 exit 0
- Probe 5 BOTH lines NON-ZERO (negative test confirms exit-code propagation)

**No commit at Task 0** — this is verification only.

---

### Task 1 [P]: Document Postgres image intent in `docker-compose.yml`

**ACTION:** Edit `docker/docker-compose.yml` at the `postgres` service block (line 84) to document the intentional choice to stay in the `pgautoupgrade/pgautoupgrade:18-*` family. If a minor-version pin `pgautoupgrade/pgautoupgrade:18.4-alpine` is available on Docker Hub, switch to it; otherwise keep `18-alpine` and add a clarifying inline YAML comment.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - docker/docker-compose.yml   # postgres service: image line + intent comment
requires: []
```

**IMPLEMENT (file 1 of 1):** in `docker/docker-compose.yml`, around line 84 (the `image:` line of the `postgres` service):

1. **Probe Docker Hub** for the more specific tag (impl-task verifies at task-time):
   - `docker manifest inspect pgautoupgrade/pgautoupgrade:18.4-alpine` → exit 0 means tag exists; non-zero means it doesn't.
   - Alternative: `curl -s https://hub.docker.com/v2/repositories/pgautoupgrade/pgautoupgrade/tags/?page_size=100 | python -c "import sys, json; tags = [t['name'] for t in json.load(sys.stdin)['results']]; print('\n'.join(t for t in tags if '18.' in t))"`.
2. **If `18.4-alpine` (or any `18.N-alpine` minor pin) exists:** update the `image:` line per §10.4 (a).
3. **If only `18-alpine` is available (fallback):** keep the existing tag and add a clarifying comment per §10.4 (b).

The Edit's `old_string` targets the existing `image: pgautoupgrade/pgautoupgrade:18-alpine` line + the next 1-2 lines of context (preserving the existing comment block at lines 85-89 that documents pgtune). The `new_string` re-emits the (possibly-updated) image line + the new comment + the rest of the context unchanged.

**MIRROR:** `docker/docker-compose.yml:43` (`image: dessalines/lemmy-ui:nightly`) demonstrates an existing intentional unpinned tag with no inline comment; Task 1 contrasts by ADDING the rationale comment, NOT by changing `lemmy-ui` (out-of-scope per PRD §7.3 ship criteria "only the lemmy-ui line, which is intentional for dev").

**GOTCHA:**
- **Do NOT switch to `postgres:16.4`** — PG18→PG16 downgrade requires data dump/restore (DQ `ship3clarify01-001`).
- **Do NOT touch the `lemmy-ui:nightly` line at line 43** — out-of-scope; it's the canonical exception in the §15.6 grep gate.
- **Do NOT touch other service tags** — `nginx:1-alpine`, `asonix/pictrs:0.5.17-pre.9` are already pinned at acceptable specificity.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
# Mechanical grep gate
grep -E "image:.*(latest|nightly)" docker/docker-compose.yml > .claude/PRPs/debug/v1-ship-3-task1-grep.log
echo "exit: $?"
cat .claude/PRPs/debug/v1-ship-3-task1-grep.log
# EXPECT: exit 0; output contains ONLY the line "    image: dessalines/lemmy-ui:nightly"

# Cargo + workspace untouched
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-3-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task1-check.log
# EXPECT: exit 0 (Task 1 is a YAML edit; no Rust impact)
```

---

### Task 2: `POST /governance/report` returns `GovernanceCaseSummaryView`

**ACTION:** Add `read_summary_for_case` helper to `governance_case` view crate; extend `CreateGovernanceReportResponse` with `case: GovernanceCaseSummaryView`; refactor `create_report` handler to fetch the view post-tx and attach it; update `report_to_modlog_golden_path` to assert the new field.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - crates/db_views/governance_case/src/impls.rs  # append read_summary_for_case (~30 lines)
  - crates/api/api_common/src/governance.rs        # extend CreateGovernanceReportResponse (+1 field)
  - crates/api/api_crud/src/governance/create_report.rs  # refactor outcome type + post-tx fetch (~15-20 lines)
  - crates/server/tests/e2e.rs                     # update report_to_modlog_golden_path assertion (≤3 lines)
requires: []
```

**IMPLEMENT (file 1 of 4):** in `crates/db_views/governance_case/src/impls.rs`, append after `list_cases_filtered` (around line 241) and before `submitted_counts_by_case` (line 246):

Use the verbatim function body from §10.1 (Patterns to mirror). Mirror precisely: same SELECT column tuple, same `.left_join(...on...)`, same call to `submitted_counts_by_case` with a single-element slice `&[r.0]`, same `build_summary` mapper.

**IMPLEMENT (file 2 of 4):** in `crates/api/api_common/src/governance.rs`, at line 56 (the `CreateGovernanceReportResponse` struct body), add one new field `pub case: lemmy_db_views_governance_case::GovernanceCaseSummaryView`:

```rust
pub struct CreateGovernanceReportResponse {
  pub case_id: Option<ModerationCaseId>,
  pub threshold_met: bool,
  pub case: lemmy_db_views_governance_case::GovernanceCaseSummaryView,
}
```

Field ordering — append `case` after the two existing fields to preserve serde JSON output ordering for any existing parsers. The `#[skip_serializing_none]` attribute (line 51) is unchanged; it only affects `Option<T>` fields.

**IMPLEMENT (file 3 of 4):** in `crates/api/api_crud/src/governance/create_report.rs`:

1. Above line 177 (just before `process_report`), introduce the small internal outcome struct:
   ```rust
   /// Internal helper struct for the transaction-scoped phase of
   /// create_report. The post-tx phase reads the view summary and
   /// composes the full CreateGovernanceReportResponse.
   struct ProcessReportOutcome {
     case_id: lemmy_db_schema::newtypes::ModerationCaseId,
     threshold_met: bool,
   }
   ```
2. Update `process_report` return type from `LemmyResult<CreateGovernanceReportResponse>` (line 192) to `LemmyResult<ProcessReportOutcome>`. The trailing `Ok(CreateGovernanceReportResponse { case_id: Some(case_id), threshold_met: just_met_threshold })` at line 325 becomes `Ok(ProcessReportOutcome { case_id, threshold_met: just_met_threshold })` (note: `case_id` is the `ModerationCaseId` typed value, not `Some(case_id)`).
3. Update `create_report` (line 145-170): replace the existing `let outcome = conn.run_transaction(...)` continuation with the post-tx view fetch + composition pattern from §8.1.
   - The `CouldntFindObject` (or closest equivalent) error variant catches the should-never-happen case where the post-tx read returns None. Junior verifies the exact variant name at task-time; if no matching variant exists in `LemmyErrorType`, file a `kind: "blocker"` DQ (do NOT introduce a new variant in this task).

**IMPLEMENT (file 4 of 4):** in `crates/server/tests/e2e.rs`, around line 2661 (after `let case_id = create_resp.case_id.expect("case_id present");`), add one new assertion:

```rust
assert_eq!(
  create_resp.case.case_id, case_id.0,
  "POST /report response carries the case summary view with matching case_id"
);
```

Optionally also add a second assertion against `create_resp.case.status` (which is `CaseStatus::Open` for the single-report case — matches the existing status assertion just below). The Edit's `old_string` targets the 3-5 lines immediately before/after line 2661; `new_string` re-emits those lines + the new `assert_eq!` inserted between the extraction and the next existing assertion.

**MIRROR:**
- For `read_summary_for_case`: `crates/db_views/governance_case/src/impls.rs:51-90` (`list_open_cases_for_community`) — same MIRROR pattern: tuple SELECT + `submitted_counts_by_case` + `build_summary`.
- For `ProcessReportOutcome` struct + post-tx view fetch: `crates/api/api_crud/src/governance/create_endorsement.rs` (the sibling handler — same two-phase pattern: helper-fn-inside-tx + composition-outside-tx).
- For the e2e assertion update: `crates/server/tests/e2e.rs:2661` — the existing `create_resp.case_id.expect(...)` line is adjacent to the new assertion.

**GOTCHA:**
- **`read_summary_for_case` returns `Option<T>`, not `T`.** A case may have been deleted by an admin between the tx commit and the post-tx read (vanishingly rare but possible). The handler MUST handle `None`; do not `unwrap()`. The `.ok_or_else(LemmyErrorType::CouldntFindObject)?` pattern is the canonical Lemmy idiom.
- **Post-tx connection acquisition is separate from the tx conn.** Use `&mut context.pool()` to acquire a fresh connection — DO NOT try to reuse the closed-tx connection from `run_transaction`.
- **`lemmy_db_views_governance_case` is already a workspace dep in `crates/api/api_common/Cargo.toml`.** No new dep needed. But verify the `ts-rs` feature lines up: line 88 already uses `Vec<lemmy_db_views_governance_case::GovernanceCaseSummaryView>` in `ListGovernanceCasesResponse`, proving the dep chain works for the embedded type.
- **`#[skip_serializing_none]` only affects `Option<T>`.** The new `case` field is not `Option<T>` (always present in a successful response). The attribute leaves it alone.
- **R11 (callsite enumeration):** before any edit, run `rg "CreateGovernanceReportResponse" crates/ tests/` and confirm the output exactly matches the 5 lines in §11 "Caller crates" table. If a 6th line appears, STOP and file a `kind: "blocker"` DQ.
- **Test fn signature unchanged.** `report_to_modlog_golden_path` returns `lemmy_utils::error::LemmyResult<()>` (e2e.rs:2485); the new assertion uses `assert_eq!` which panics on failure — no `?` propagation, no return-type change.
- **No `#[cfg(feature = "full")]` gating needed on the new field.** `GovernanceCaseSummaryView` is unconditionally exported from `lemmy_db_views_governance_case` (per `crates/db_views/governance_case/src/lib.rs:35-59`). Only the `*DetailRow` / `*DetailView` types are `full`-gated.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-3-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task2-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-3-task2-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task2-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-3-task2-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task2-test-no-run.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full report_to_modlog_golden_path > .claude/PRPs/debug/v1-ship-3-task2-e2e.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-ship-3-task2-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" present in tail
```

---

### Task 3: `two_sponsors_lose_endorsement_strength_on_sanction` e2e

**ACTION:** Append a new `mod v1_ship_3_fixtures { ... }` to `crates/server/tests/e2e.rs` containing one test fn that drives the report → admin-assign → 3 jury votes pipeline against a target with 2 sponsors (each `endorsement_strength: 10`), asserting both sponsors land at `endorsement_strength == 0` (clamped by floor=0).

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append mod v1_ship_3_fixtures with one test fn + inlined helpers
requires:
  - task: 2
    reason: "Task 2 changes the CreateGovernanceReportResponse shape; Task 3's call to create_report (through run_sanction_scenario) will fail to compile if Task 2's struct edit is missing the new case field OR if the response type drifts. Serial dispatch enforces this dependency mechanically."
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, append (do NOT insert) at the end of the file:

1. **Module scaffolding** — verbatim from §10.2 (the full `mod v1_ship_3_fixtures { use super::*; use ...; }` block). Module lands AFTER `mod v1_federation_inbound_e_fixtures` (closing `}` at line 16692). Edit's `old_string` targets the last 3-5 lines of the file (e.g. the closing `}` of fed-in-e + trailing newline); `new_string` re-emits those lines + the new `mod v1_ship_3_fixtures { ... }` block.

2. **Inlined helpers** (inside the module, before the test fn — copy from `sponsor_liability_with_founder_multiplier` e2e.rs:3368-3645 verbatim, adapted only by name):
   - `seed_person(ctx, instance_id, name, is_admin) -> LemmyResult<PersonId>` (mirror e2e.rs:3368-3384)
   - `seed_surety(conn, sponsor, sponsored) -> LemmyResult<()>` (mirror e2e.rs:3402-3417)
   - `seed_snapshot(conn, person, endorsement_strength) -> LemmyResult<()>` (mirror e2e.rs:3477-3502)
   - `run_sanction_scenario(context, federation_context, admin_view, reporter_view, jurors, target, community_id, reason_code, decision) -> LemmyResult<i32>` (mirror e2e.rs:3511-3607; helper-scope-local — no name collision with the existing fn-local helper inside `sponsor_liability_with_founder_multiplier`)
   - `liability_delta_for(conn, person, case_id) -> LemmyResult<i32>` (mirror e2e.rs:3609-3623)

3. **Test fn `two_sponsors_lose_endorsement_strength_on_sanction`**:
   - Signature: `async fn two_sponsors_lose_endorsement_strength_on_sanction() -> LemmyResult<()>`
   - Attribute: `#[tokio::test(flavor = "multi_thread")]`
   - **Step 1 — env + container + context + federation_context** — mirror e2e.rs:3309-3354 verbatim (including `GOVERNANCE_LOG_SIGNING_KEY` setup, `start_postgres()`, `apply_all_schema`, `build_db_pool_for_tests`, `LemmyContext::create`, `FederationConfig::builder()`).
   - **Step 2 — instance + community** — `Instance::read_or_create("test.invalid").await?`; `Community::create(...)` with form `CommunityInsertForm::new(instance.id, "ship3_comm".to_string(), ...)`.
   - **Step 3 — actors** — `let admin = seed_person(&context, instance.id, "ship3_admin", true).await?;` + reporter + 6 jurors (`ship3_juror_0..5`).
   - **Step 4 — 2 sponsors + 1 sponsee target** — `let target = seed_person(&context, instance.id, "ship3_target", false).await?;` + 2 sponsors `ship3_sponsor_1`, `ship3_sponsor_2`.
   - **Step 5 — seed sureties + snapshots** — open `AsyncPgConnection::establish(&db_url).await?`; call `seed_surety` × 2 (each sponsor → target); call `seed_snapshot` × 2 (each sponsor at `endorsement_strength = 10`).
   - **Step 6 — admin_view + reporter_view** — `LocalUserView::read_person(&mut context.pool(), admin).await?`; same for reporter.
   - **Step 7 — runtime config read** — verbatim from §10.3 (read `liability.sponsor_liability_floor` + `deltas.sponsor_liability_moderate`, compute `expected_clamped` and `expected_final_strength`).
   - **Step 8 — drive the sanction pipeline** — `let case = run_sanction_scenario(&context, &federation_context, &admin_view, &reporter_view, &jurors, target, community.id, "ship3_2sponsors", JuryDecision::RemoveContent).await?;`. RemoveContent maps to Moderate severity per `severity_for_action` (sponsor_liability.rs:144-156).
   - **Step 9 — assert deltas** — `let d_1 = liability_delta_for(&mut async_conn, sponsor_1, case).await?;` similarly for sponsor_2; `assert_eq!(i64::from(d_1), expected_clamped, "...");` for both.
   - **Step 10 — assert final snapshots** — `let snap_1 = recompute_snapshot(&mut async_conn, sponsor_1, None, &mut cache).await?;` similarly for sponsor_2; `assert_eq!(i64::from(snap_1.endorsement_strength), expected_final_strength, "sponsor_1 final endorsement_strength = 0");` for both.
   - **Step 11 — `Ok(())`**

**MIRROR:**
- Module scaffolding: `crates/server/tests/e2e.rs:15451-15835` (`mod v1_ship_2_fixtures` — append-only sibling)
- 2-sponsor pipeline + helper shape: `crates/server/tests/e2e.rs:3271-3859` (`sponsor_liability_with_founder_multiplier` branches 1 + 3 — the closest 2-sponsor + floor-clamp pattern)
- Config read at runtime: `crates/server/tests/e2e.rs:3777-3787` (the recompute_snapshot + assert pattern)
- `liability_delta_for` shape: `crates/server/tests/e2e.rs:3609-3623`

**GOTCHA:**
- **`run_sanction_scenario` name collision is OK** — the existing `sponsor_liability_with_founder_multiplier` defines `run_sanction_scenario` as an inner async fn local to its test body; the new `mod v1_ship_3_fixtures` defines its own at module scope; they live in different scopes → no collision. Verify at task-time by reading both bodies; if a future refactor extracts the existing helper to module scope of the parent file, rename the new one (`ship3_run_sanction_scenario`).
- **`build_db_pool_for_tests` already runs migrations.** Per the canonical sibling pattern (e2e.rs:3326) — call `governance_fixtures::apply_all_schema(&mut sync_conn)` ONCE before `build_db_pool_for_tests` and don't re-run schema afterwards.
- **`federation_context` is needed.** `submit_jury_vote` requires `activitypub_federation::config::Data<LemmyContext>` not the actix `Data<LemmyContext>` (per Phase 6 task 76 comment at e2e.rs:3342-3354). Build both: actix Data for create_report + admin_assign_jury + accept_jury_assignment; federation Data for submit_jury_vote.
- **Sponsor must NOT also be a juror.** `admin_assign_jury` excludes the case's `target_person_id` and excludes the case's reporter; it does NOT explicitly exclude sponsors. Mitigation: seed 6 distinct jurors (not the 2 sponsors); use `seed_person(..., "ship3_juror_<i>", false)` for jurors and `seed_person(..., "ship3_sponsor_<i>", false)` for sponsors — distinct names → distinct persons → no collision risk. If `admin_assign_jury` returns an `assigned_person_ids` list containing a sponsor, file a `kind: "blocker"` DQ.
- **`endorsement_strength` is `i32` on the snapshot row** but `i64` on the config / clamp math. Use `i64::from(snap.endorsement_strength)` for comparisons (R1).
- **Test name** is `two_sponsors_lose_endorsement_strength_on_sanction` — exact match to the brief §1 deliverable 3 + the PRD §9 done-criterion verbiage. Do NOT rename.
- **Append discipline (per `feedback_junior_worker_e2e_edit_hang.md` spirit):** the Edit's `old_string` MUST target the final lines of `e2e.rs` (the closing `}` of fed-in-e module + trailing newline). `new_string` re-emits those lines + the new module body. NEVER target the middle of an existing module.
- **No new DTO touched.** The pipeline calls existing handlers; no struct or re-export changes (R7 non-binding for Task 3).
- **`reputation_event::source_case_id` lookup** — the test's `liability_delta_for` filters by `source_case_id = case_id` AND `reason = "sponsor_liability_applied"`. The fire path inside `submit_jury_vote` writes exactly one such row per sponsor (per sponsor_liability.rs:486-540 `apply_sponsor_liability`). Asserting `.first()` returns the latest (DESC by id) — there should be exactly one row per (sponsor, case) tuple.

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-3-task3-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task3-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-3-task3-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task3-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-3-task3-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task3-test-no-run.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full two_sponsors_lose_endorsement_strength_on_sanction > .claude/PRPs/debug/v1-ship-3-task3-e2e.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-ship-3-task3-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" present in tail

# Composite Story 4 checkpoint: run ALL v1-ship-3 tests by module pattern
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full v1_ship_3_fixtures > .claude/PRPs/debug/v1-ship-3-task3-module-e2e.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-3-task3-module-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" (single new test in module)
```

---

### Task 4: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`).

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/PRPs/reports/v1-ship-3-retro.md
modifies: []
requires:
  - task: 1
    reason: "Retro reads outcomes of Tasks 1-3."
  - task: 2
    reason: "Retro reads outcomes of Tasks 1-3."
  - task: 3
    reason: "Retro reads outcomes of Tasks 1-3."
```

**Per-task complexity score** (`feedback_retro_task_complexity_score`) — author each entry as `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Aggregate in §5.

**Retro structure:**

- §1 Brief / Plan refs
- §2 What surprised us / what to change / what to carry forward
- §3 Four-role signals — Advisor / Planning / Impl / BM (one H2 per role)
- §4 Promote lessons (any new `feedback_*.md` authored this phase). Two candidate lessons to evaluate:
  - **(a)** `feedback_clarify_config_key_drift.md` — DQ `ship3clarify01-003` referenced `liability.endorsement_delta_moderate` but the actual key is `deltas.sponsor_liability_moderate`. The planner caught this drift at plan-author time (§19 Notes) but a future planner could re-introduce it. If retro confirms this drift pattern is recurring, promote.
  - **(b)** `feedback_post_tx_view_fetch_pattern.md` — the Task 2 reshape introduces the "tx commits → fresh conn → view fetch → response compose" pattern; future similar reshapes will need this recipe. If the impl-task hit any sharp edges (e.g. error-variant choice, connection-acquisition order), promote.
- §5 Per-task complexity scores
- §6 Watch-items for next sub-phase (any inherited from this phase's friction)

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** Task 2 adds a function and a struct field. Compile-time check is `cargo check --workspace --features full` after Task 2 (§15.1).
- **Workspace check:** `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` after every Rust task (Tasks 2 + 3).
- **Lint:** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` after every Rust task. Mandatory `--no-deps` per R6 to avoid upstream lint debt.
- **Test target compile:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run"` after every task that touches Rust. Confirms the e2e binary links.
- **e2e per-test:**
  - Task 2: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full report_to_modlog_golden_path"`
  - Task 3: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full two_sponsors_lose_endorsement_strength_on_sanction"`
- **e2e module pattern (Task 3 final / Story 4):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full v1_ship_3_fixtures"`
- **e2e regression (Task 3 final + bm-pr time):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` — full suite. Confirms no existing test regressed.
- **Migration round-trip:** N/A — no new migrations.
- **`docker-compose.yml` lint:** `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml` after Task 1 — expect ONLY the `lemmy-ui:nightly` line.

---

## 15. Validation commands (DoD)

> **Mode:** `validate-pending-laptop` (Shape G SUSPENDED until 2026-06-01 per DQ #229 + `project_shape_g_suspended_2026_05_16`). Commands run on the laptop advisor session, NOT on GH Actions. Per `feedback_laptop_default_for_validate_pending.md` + `feedback_validate_pending_laptop_must_use_wrapper.md`.
>
> **Planner-side discipline** (per `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md`): every command in this section MUST be dry-run by the advisor against current HEAD before plan approval. Unexecutable commands are advisor-side rejection grounds. See §19 Notes for the planner's dry-run results.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-3-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-3-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test target compile (per task touching Rust — Tasks 2 + 3)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-3-<task>-test-no-run.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.4 e2e per-task execution

```bash
# Task 1: docker-compose.yml grep gate
grep -E "image:.*(latest|nightly)" docker/docker-compose.yml > .claude/PRPs/debug/v1-ship-3-task1-grep.log
echo "exit: $?"
cat .claude/PRPs/debug/v1-ship-3-task1-grep.log
# EXPECT: exit 0; output contains EXACTLY the line "    image: dessalines/lemmy-ui:nightly"

# Task 2: updated golden-path test
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full report_to_modlog_golden_path > .claude/PRPs/debug/v1-ship-3-task2-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "1 passed; 0 failed"

# Task 3: new 2-sponsors test
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full two_sponsors_lose_endorsement_strength_on_sanction > .claude/PRPs/debug/v1-ship-3-task3-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "1 passed; 0 failed"
```

### 15.5 e2e composite + regression (post-Task 3)

```bash
# Module pattern — all v1-ship-3 tests:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full v1_ship_3_fixtures > .claude/PRPs/debug/v1-ship-3-module-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "1 passed; 0 failed"

# Full e2e regression (the canonical end-of-phase DoD):
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-ship-3-full-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows N+1 passed where N is the pre-v1-ship-3 e2e test count
```

### 15.6 Cross-cutting verification

- [ ] No file outside §11 list edited. Verify: `git diff --stat governance-v0...HEAD` enumerates ONLY `docker/docker-compose.yml`, `crates/db_views/governance_case/src/impls.rs`, `crates/api/api_common/src/governance.rs`, `crates/api/api_crud/src/governance/create_report.rs`, `crates/server/tests/e2e.rs` for Tasks 1-3 and `.claude/PRPs/reports/v1-ship-3-retro.md` for Task 4.
- [ ] R5: Task 0 enumerated all 12 probes (Probes 0-11).
- [ ] R6: every clippy invocation in §15.2 uses `--no-deps` AND `--features full`.
- [ ] R7: test-target compile (§15.3) runs after Task 2 (which touches `CreateGovernanceReportResponse` — a public struct).
- [ ] R8: the new test fn in `mod v1_ship_3_fixtures` is `async fn two_sponsors_lose_endorsement_strength_on_sanction() -> LemmyResult<()>`. Verify: `rg "async fn two_sponsors_lose_endorsement_strength_on_sanction" crates/server/tests/e2e.rs` returns 1 match.
- [ ] R9: `rg "Box<dyn Error" crates/server/tests/e2e.rs` returns no matches inside `mod v1_ship_3_fixtures { }` (Case A discipline; line-range filter once committed).
- [ ] R10: `mod v1_ship_3_fixtures` lives at the END of `crates/server/tests/e2e.rs` (after `mod v1_federation_inbound_e_fixtures`). Verify: `tail -300 crates/server/tests/e2e.rs | grep "^mod v1_ship_3_fixtures"` returns one match within the last 300 lines.
- [ ] R11: callsite enumeration confirmed — `rg "CreateGovernanceReportResponse" crates/ tests/` returns the same 5 lines documented in §11 "Caller crates" table, no surprises.
- [ ] Audit: `docker-compose.yml` grep gate passes — `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml` returns ONLY the `dessalines/lemmy-ui:nightly` line.
- [ ] Audit: the §9 done-criterion has a named e2e — `rg "fn two_sponsors_lose_endorsement_strength_on_sanction" crates/server/tests/e2e.rs` returns 1 match.

---

## 16. Acceptance criteria

- [ ] All 5 tasks completed in dependency order (Task 0 audit, Tasks 1-3 impl, Task 4 retro)
- [ ] §15.1 (cargo check workspace) exit 0 after Tasks 2 + 3
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after Tasks 2 + 3
- [ ] §15.3 (cargo test --no-run) exit 0 after Tasks 2 + 3
- [ ] §15.4 (per-test e2e) exit 0 for Tasks 2 + 3
- [ ] §15.5 (composite + full regression) exit 0 — the new test passes AND no pre-existing e2e test regresses
- [ ] §15.6 (cross-cutting verification) — all 9 boxes ticked
- [ ] §16a stories — all 4 stories `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per §13 Task 4
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-3-verify.md` shows all stories ✓
- [ ] Manual audit re-run: `rg "fn two_sponsors_lose_endorsement_strength_on_sanction" crates/server/tests/e2e.rs` returns 1 match; `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml` returns only the `lemmy-ui:nightly` line; `rg "pub case:" crates/api/api_common/src/governance.rs` returns the new field on `CreateGovernanceReportResponse`.

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: `docker-compose.yml` documents the Postgres image intent (no unpinned drift)

- **Composing tasks:** Task 1
- **Checkpoint command:** `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml`
- **Expected output:** exactly one line — `    image: dessalines/lemmy-ui:nightly` (the deliberate dev choice; no other unpinned image)
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `docker/docker-compose.yml` contains an `image:` line for the `postgres` service that matches `pgautoupgrade/pgautoupgrade:18(\.[0-9]+)?-alpine`
  - The same file contains a comment line immediately adjacent to that `image:` line documenting the intent (matches `# .*PG18.*` or `# .*pgautoupgrade.*` — the impl-task picks the exact phrasing)

### Story 2: `POST /governance/report` returns a `GovernanceCaseSummaryView` of the opened case

- **Composing tasks:** Task 2
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full report_to_modlog_golden_path"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/api/api_common/src/governance.rs` contains `pub case: lemmy_db_views_governance_case::GovernanceCaseSummaryView` inside `CreateGovernanceReportResponse`
  - `crates/db_views/governance_case/src/impls.rs` contains `pub async fn read_summary_for_case`
  - `crates/api/api_crud/src/governance/create_report.rs` contains `struct ProcessReportOutcome`
  - `crates/server/tests/e2e.rs` `report_to_modlog_golden_path` contains a new `assert_eq!(create_resp.case.case_id, ...)` line

### Story 3: `two_sponsors_lose_endorsement_strength_on_sanction` named e2e exists and passes

- **Composing tasks:** Task 3 (depends on Task 2 for the response shape; serial dispatch)
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full two_sponsors_lose_endorsement_strength_on_sanction"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e.rs` contains `mod v1_ship_3_fixtures` (top-level)
  - `crates/server/tests/e2e.rs` contains `async fn two_sponsors_lose_endorsement_strength_on_sanction() -> LemmyResult<()>` inside `mod v1_ship_3_fixtures`
  - The test fn body contains `seed_surety`, `seed_snapshot`, `run_sanction_scenario`, `liability_delta_for`, `recompute_snapshot` calls (mirroring the canonical 2-sponsor pattern)
  - The test fn body reads `liability.sponsor_liability_floor` and `deltas.sponsor_liability_moderate` from `governance_config` at runtime (no hardcoded `-10` in the assertion)

### Story 4: v1-ship-3 cohort composite — all three deliverables ship together

- **Composing tasks:** Tasks 1 + 2 + 3 (cumulative)
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` AND `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml`
- **Expected output:** cargo: `N+1 passed; 0 failed` (full suite; one new test) AND grep: only `lemmy-ui:nightly`
- **Brief-Scope outputs to verify:**
  - All Story 1-3 outputs are present
  - PRD §13.3 audit re-run: `docker-compose.yml` clean, `POST /report` returns view, 2-sponsors named e2e exists

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 12 probes confirmed — Probes 0–11)
- [ ] Task 1 committed (`feat(docker): pin postgres image intent (task 1)` or `docs(docker): document postgres image choice (task 1)`)
- [ ] Task 2 committed (`feat(api): POST /report returns governance case summary view (task 2)`)
- [ ] Task 3 committed (`feat(test): two_sponsors_lose_endorsement_strength_on_sanction e2e (task 3)`)
- [ ] Task 4 retro committed (`docs(retro): v1-ship-3 retro`)
- [ ] §15 validation green at every gate (validate-pending-laptop mode — advisor laptop runs commands; no GH Actions)
- [ ] §16a stories all `[done]`
- [ ] PR opened by BM session against `governance-v0` (with `--repo barrie-cork/lemmy` per `phase-branch.md`)
- [ ] CodeRabbit review complete with findings triaged
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-3-verify.md` shows all 4 stories ✓
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `pgautoupgrade/pgautoupgrade:18.4-alpine` (or similar minor pin) doesn't exist on Docker Hub | MED | LOW | Task 1 has the (b) fallback shape — keep `18-alpine` + add the clarifying comment. The Story 1 checkpoint passes under either form. |
| `read_summary_for_case` returns `Ok(None)` immediately after a successful insert (race with admin delete) | LOW | LOW | Handler maps to `LemmyErrorType::CouldntFindObject` (404). Should-never-happen in practice; defensive against future admin tooling. |
| `LemmyErrorType::CouldntFindObject` doesn't exist by that exact name | LOW | LOW | Junior verifies the exact variant at task-time by reading `crates/utils/src/error.rs`; if no match, file `kind: "blocker"` DQ. Candidates: `LemmyErrorType::NotFound`, `LemmyErrorType::CouldntFindCase`. |
| `report_to_modlog_golden_path` assertion update breaks existing test invariants | LOW | LOW | The new assertion only ADDS a check on the new field; existing assertions are unchanged. |
| Task 3's `run_sanction_scenario` helper-name collides with the existing one in `sponsor_liability_with_founder_multiplier` | LOW | LOW | Both are scoped (existing is in test-fn body of a top-level fn; new is in `mod v1_ship_3_fixtures` module scope). Verified at plan-author time — no name collision. If future refactor extracts the existing helper to module scope, rename the new one (`ship3_run_sanction_scenario`). |
| A sponsor is accidentally selected as a juror, deadlocking the test | LOW | MED | Seed 6 distinct jurors (`ship3_juror_0..5`) separate from the 2 sponsors (`ship3_sponsor_1`, `ship3_sponsor_2`); `admin_assign_jury` should never select a sponsor for the case it heads. If observed, file `kind: "blocker"` DQ. |
| `governance_config` keys `deltas.sponsor_liability_moderate` and `liability.sponsor_liability_floor` not seeded at test runtime (defaults missing) | LOW | LOW | `config::get_int` falls back to compiled-in defaults (`DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE = -50`, `DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR = 0`) per config.rs:1066-1068; the read always succeeds. |
| Task 2's struct field add triggers an unexpected callsite (R11 enumeration drift) | LOW | LOW | Task 0 Probe 10 + Task 2 §IMPLEMENT GOTCHA both run `rg "CreateGovernanceReportResponse" crates/ tests/` and expect ≤5 lines. Drift → STOP + DQ. |
| Existing `sponsor_liability_with_founder_multiplier` (`#[ignore]`) flake taints the 2-sponsor test | LOW | MED | The existing test is `#[ignore]`-flagged per GH #45 (random jury pool + fallback path NotFound). Task 3's test is NOT `#[ignore]` and runs against a controlled jury pool of 6 (≥5 needed). If flake observed, file `kind: "blocker"` DQ. |
| Plan complexity score (5/10) under-counts cross-crate coupling in Task 2 | LOW | LOW | Task 2's 4-crate spread is explicitly credited in §5.4. If retro shows the 4-crate spread was harder than scored, future planners adjust the credit. |
| Shape G reactivates mid-phase (Shape G suspended until 2026-06-01) | LOW | LOW | Plan is pinned to validate-pending-laptop mode; the §15 DoD is laptop-shape; even if Shape G reactivates, the laptop commands still work. Switch to Shape G post-merge if desired. |

---

## 19. Notes

- **Brief constraint #6 (canonical-schema-first gate) was honored at plan-author time.** This planner Read e2e.rs:15451-15835 (`mod v1_ship_2_fixtures`), e2e.rs:16513-16692 (`mod v1_federation_inbound_e_fixtures`), e2e.rs:3271-3859 (`sponsor_liability_with_founder_multiplier`), and e2e.rs:2485-2666 (`report_to_modlog_golden_path`) before writing §13 stubs. The §10.2 scaffolding block quotes the canonical-sibling shape verbatim (`use super::*;`, `LemmyResult<()>` outer, `LemmyResult<T>` helpers, bare `?` propagation, no `.map_err`, no `Box<dyn Error>`).

- **DQ entries `ship3clarify01-001`, `-002`, `-003` were all resolved at brief-author time** (per `.claude/decision-queue.json` resolved entries, `governance-v0` HEAD `3509834e9`). The plan internalises all three:
  - `-001` (Postgres family — stay in pgautoupgrade) → baked into Task 1 scope + §10.4 (no PG16 family switch) + §18 risk table.
  - `-002` (need to author `read_summary_for_case`) → baked into Task 2 §IMPLEMENT (file 1 of 4) + §10.1 + §11.
  - `-003` (assertion math + read config keys at runtime) → baked into Task 3 §IMPLEMENT step 7 + §10.3 + §18 risk table.

- **Config-key drift in DQ `ship3clarify01-003` — planner-side correction.** The DQ resolution text says "Read the delta from `governance_config` at test runtime (key `\"liability.endorsement_delta_moderate\"`)". The actual key in `crates/api/api/src/governance/sponsor_liability.rs:91` is `"deltas.sponsor_liability_moderate"` (per `LiabilitySeverity::config_key`). The DQ-stated key does NOT exist in the codebase; using it would cause the test to fail with "config key not found" (or fall back to a default of 0, which would produce a false-pass).

  **Planner's resolution:** the plan §10.3 + §13 Task 3 step 7 + §15.6 all use the **correct key** `deltas.sponsor_liability_moderate` (paired with `liability.sponsor_liability_floor`, also verified against `crates/api/api/src/governance/sponsor_liability.rs:250`). This is captured as a planner self-resolved `kind: "log"` DQ for the advisor to harvest at retro time. The lesson candidate for §4 retro: drifted config-key names in clarify-DQ answers — recommend a brief-author-time `rg "<key>" crates/` verification step before resolving config-named DQ answers.

- **No new DQ entries raised at plan-author time** (other than the planner self-resolved log entry above). The plan is internally consistent with the brief + the three resolved clarify entries.

- **PR DoD (DQ-resolved):** per brief §4 acceptance criterion 3, §15 DoD uses `cargo test --workspace --features full --test e2e ...` (NOT `-p lemmy_server --features full` — `lemmy_server` does not declare a `full` feature; cargo would error). All §15 commands obey this constraint.

- **Cohort dispatch reality:** the brief §1 last sentence flagged whether Junior workers can safely co-write on `e2e.rs`. Answer per §5.3 + `feedback_explicit_file_arrays_on_tasks.md`: NO, the advisor's mechanical YAML-intersection rule treats `e2e.rs` overlap as cohort-incompatible regardless of in-file locality. Task 2 and Task 3 ship serially. Task 1 ships in parallel with EITHER Task 2 OR Task 3 (the file-set is genuinely disjoint).

- **Dry-run results** (planner self-attested, advisor verifies at gate 1):
  - `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml` against current HEAD returns exactly 1 line (`dessalines/lemmy-ui:nightly`) — confirms the grep gate's expected output is reachable.
  - `rg "CreateGovernanceReportResponse" crates/ tests/` against current HEAD returns 5 lines across 2 files — confirms R11 enumeration's expected output.
  - All §15 cargo commands use the workspace wrapper invocations that the prior ship-2 plan validated; carry-forward without modification.

- **Forward-looking:** the next sub-phase (none currently planned in the v1-ship lane; PRD §10 enumerates 3 v1-ship sub-phases total) inherits this plan's append-only e2e.rs discipline if any further e2e work is added. The canonical next-most-recent sibling for any future e2e-edit plan is `mod v1_ship_3_fixtures` (this plan's output), now standing alongside `mod v1_ship_2_fixtures` and `mod v1_federation_inbound_e_fixtures`.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — every §13 stub is grounded in a specific MIRROR ref + handler line range; the canonical-sibling-shape gate has been honored; the three deliverables align with the brief verbatim. The two risk drivers are (a) the post-tx error-variant choice (Task 2 §IMPLEMENT GOTCHA — `LemmyErrorType::CouldntFindObject` may need re-verification at task-time) and (b) the 2-sponsor jury-eligibility risk (Task 3 §GOTCHA — `admin_assign_jury` does not explicitly exclude sponsors; mitigated by 6-juror pool). Both have explicit DQ fallbacks.
- **Cargo budget:** 9/10 — validate-pending-laptop mode; no EliteDesk budget concern. Local cargo + e2e peak ~6 GB on the laptop; comfortable.
- **Test coverage:** 8/10 — Task 2's assertion update is minimal (1 new field check) but the field's presence on every successful POST /report response is structural (Serde fail-loud on shape drift). Task 3's test asserts both the per-sponsor delta (via `liability_delta_for`) AND the final snapshot (via `recompute_snapshot`), covering the two-step pipeline. The §9 done-criterion is reproducibly checked. Out of scope: per-severity-tier (Minor / Severe) variants — those are existing behaviour and would dilute the named-test value; can be added in a follow-up if PRD widens.
