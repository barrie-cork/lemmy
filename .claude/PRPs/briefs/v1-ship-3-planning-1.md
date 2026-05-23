# Planning Brief — v1-ship-3: Tactical Polish Bundle

**Phase:** v1-ship-3
**Branch:** phase-v1-ship-3 (cut from governance-v0 @ f8d4ee33f)
**Authored:** 2026-05-23
**Authored by:** advisor (canonical brehon-fork / ship-3 session)
**PRD:** `.claude/PRPs/prds/v1-ship-readiness.prd.md` §7.3
**Plan target:** `.claude/PRPs/plans/v1-ship-3.plan.md`

---

## 1. What this phase delivers

Three independent, file-disjoint deliverables bundled to amortize branch/PR overhead:

1. **Postgres image pin** — `docker/docker-compose.yml:84` uses `pgautoupgrade/pgautoupgrade:18-alpine`. **Do NOT switch to `postgres:16.4`** (DQ ship3clarify01-001): that would be a PG18→PG16 downgrade requiring a data volume dump/restore. The image is already effectively pinned (specific tag, not `:latest`). Task 1 scope: (a) check if a more specific minor-version tag like `pgautoupgrade:18.4-alpine` is available and prefer it; (b) if no minor tag exists, add an inline YAML comment documenting the intentional tag choice. One-line edit (possibly zero if `18-alpine` is the most specific available tag for PG18).

2. **`POST /report` DTO reshape** — `CreateGovernanceReportResponse` currently returns `{ case_id: Option<ModerationCaseId>, threshold_met: bool }` (defined at `crates/api/api_common/src/governance.rs:56`). PRD §7.3 requires adding `case: GovernanceCaseSummaryView` to match the read surface (`ListGovernanceCasesResponse.cases: Vec<GovernanceCaseSummaryView>` at `governance.rs:88`). **Clarified (DQ ship3clarify01-002):** No `read_summary_for_case(case_id)` helper exists in `impls.rs` — Task 2 MUST author one new function: `read_summary_for_case(pool, case_id) -> LemmyResult<Option<GovernanceCaseSummaryView>>` in `crates/db_views/governance_case/src/impls.rs`, following the two-round-trip pattern of `list_open_cases_for_community` (impls.rs:51) but filtered by `moderation_case::id.eq(case_id)`. The handler at `crates/api/api_crud/src/governance/create_report.rs:192` then calls this helper after the insert to populate the response. The existing `report_to_modlog_golden_path` e2e test must be updated to assert against the new field.

3. **`two_sponsors_lose_endorsement_strength_on_sanction` e2e** — named test in `crates/server/tests/e2e.rs`, in a new `mod v1_ship_3_fixtures` appended after the existing `mod v1_federation_inbound_e_fixtures` at line 16513. Asserts the §9 done-criterion: 1 sponsee + 2 sponsors (each with `endorsement_strength: 10`) → full report→jury→vote flow → both sponsors' `endorsement_strength` reach `0`. **Clarified (DQ ship3clarify01-003):** Use ContentRemoval (moderate severity, `DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE = -50` at `config.rs:831`). Floor = 0 (`DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR` at `config.rs:835`). Clamped delta = max(-50, 0 - 10) = -10. Final assertion: both sponsors' `endorsement_strength == 0`. Read the delta from `governance_config` at test runtime (key `"liability.endorsement_delta_moderate"`) — do NOT hardcode -10. Substrate: `apply_sponsor_liability` at `crates/api/api/src/governance/sponsor_liability.rs:486`.

All three deliverables are confirmed file-disjoint:
- Item 1: `docker/docker-compose.yml`
- Item 2: `crates/api/api_common/src/governance.rs` + `crates/api/api_crud/src/governance/create_report.rs` + assertion update in `crates/server/tests/e2e.rs` (report_to_modlog_golden_path)
- Item 3: `crates/server/tests/e2e.rs` (new module, separate function from item 2)

→ **`[P]` cohort marker is applicable.** Tasks 1, 2, and 3 may be dispatched in parallel. Task 2 and Task 3 both touch `e2e.rs` but at disjoint locations (existing function vs. new module at end of file) — flag if Junior workers cannot coordinate safely on that file.

---

## 2. Key file anchors (verify these before authoring the plan)

| File | Anchor | Purpose |
|---|---|---|
| `docker/docker-compose.yml:84` | `image: pgautoupgrade/pgautoupgrade:18-alpine` | Task 1: check for minor-version tag; add comment if none found |
| `crates/api/api_common/src/governance.rs:56` | `struct CreateGovernanceReportResponse` | Task 2 DTO to extend |
| `crates/api/api_common/src/governance.rs:87` | `struct ListGovernanceCasesResponse` | Task 2 mirror target |
| `crates/db_views/governance_case/src/lib.rs:44` | `struct GovernanceCaseSummaryView` | Task 2 field type to add |
| `crates/api/api_crud/src/governance/create_report.rs:192` | `create_report_inner` fn | Task 2 handler to update |
| `crates/api/api_crud/src/governance/create_report.rs:325` | `Ok(CreateGovernanceReportResponse { ... })` | Task 2 return site |
| `crates/server/tests/e2e.rs:2485` | `report_to_modlog_golden_path` | Task 2 e2e assertion update needed |
| `crates/server/tests/e2e.rs:16513` | `mod v1_federation_inbound_e_fixtures` | Task 3 insertion point (append after this mod) |
| `crates/server/tests/e2e.rs:16692` | EOF | Task 3 module goes here |
| `crates/db_views/governance_case/src/impls.rs:51` | `list_open_cases_for_community` | Task 2 MIRROR for new `read_summary_for_case` fn |
| `crates/api/api/src/governance/config.rs:831` | `DEFAULT_DELTAS_SPONSOR_LIABILITY_MODERATE = -50` | Task 3 delta constant reference |
| `crates/api/api/src/governance/config.rs:835` | `DEFAULT_LIABILITY_SPONSOR_LIABILITY_FLOOR = 0` | Task 3 floor constant reference |
| `crates/api/api/src/governance/sponsor_liability.rs:486` | `apply_sponsor_liability` | Task 3 assertion target |

---

## 3. Watchpoints for the planner

**WP-1 (Task 1 — Postgres image family) — RESOLVED (DQ ship3clarify01-001):** Do not switch to `postgres:16.4`. `pgautoupgrade:18-alpine` runs PG18; switching to PG16 requires a data volume dump/restore (major downgrade). Task 1 scope is narrowed: check if `pgautoupgrade:18.4-alpine` (minor-version pin) exists; if yes, prefer it. If only `18-alpine` is available, add a YAML comment documenting the intentional choice. No family switch.

**WP-2 (Task 2 — `GovernanceCaseSummaryView` population) — RESOLVED (DQ ship3clarify01-002):** No `read_summary_for_case(case_id)` helper exists in `impls.rs`. Task 2 must author exactly 1 new function: `read_summary_for_case(pool: &mut DbPool<'_>, case_id: ModerationCaseId) -> LemmyResult<Option<GovernanceCaseSummaryView>>` in `crates/db_views/governance_case/src/impls.rs`. Mirror pattern: `list_open_cases_for_community` (impls.rs:51) — same two round-trips (main SELECT + `submitted_counts_by_case`), filtered by `moderation_case::id.eq(case_id)`, returning `Option` (`.first().optional()`). The handler calls this after the insert; the view fetch happens in a separate connection get (after the insert transaction commits) — not inside the insert transaction.

**WP-3 (Task 3 — e2e pattern):** The `two_sponsors_lose_endorsement_strength_on_sanction` test follows the same sponsor-liability pattern as `sponsor_liability_with_founder_multiplier` (starting at e2e.rs:3237). Read that test's fixture setup to reuse the same jury-drive helper (`admin_assign_jury` + `accept_jury_assignment` + `submit_jury_vote` chain). Do NOT re-implement the jury drive from scratch. The `mod v1_sl_d_fixtures` at line 13562 is the canonical reference for this pattern.

**WP-4 (Task 3 — governance_config seed for per-severity delta):** The `endorsement_strength` decrement amount is read from `governance_config` at runtime by `apply_sponsor_liability`. The test must either: (a) rely on seeded defaults from the migration, or (b) INSERT the config row inline. Check which severity tier the test will use (ContentRemoval = moderate, delta = -50 per existing tests) and verify that value is present in the `governance_config` seed rows at test runtime. Do NOT hardcode the delta value in the assertion without reading it from config — use the pattern from `sponsor_liability_with_founder_multiplier` which reads `liability.endorsement_delta_moderate` via governance_config.

**WP-5 (Task 3 — `endorsement_strength` initial value) — RESOLVED (DQ ship3clarify01-003):** Seed sponsors at `endorsement_strength = 10`. Use ContentRemoval (moderate). Clamped delta = max(-50, 0 - 10) = -10. Final value = 10 + (-10) = 0. Assert `endorsement_strength == 0` for both sponsors. Read the delta from `governance_config` at test runtime (key `"liability.endorsement_delta_moderate"`) — do NOT hardcode the -10 arithmetic; pattern from `sponsor_liability_with_founder_multiplier`.

---

## 4. DoD gates (cargo + e2e)

Per `feedback_plan_dod_dry_run_at_write.md` — these must be executable as written:

**Task 1:** `grep -E "image:.*(latest|nightly)" docker/docker-compose.yml` — must return zero lines (or only the intentional `lemmy-ui:nightly` dev line).

**Task 2:** `cargo check --workspace --features full` — must exit 0. Then: `cargo test --workspace --features full --test e2e -- report_to_modlog_golden_path --nocapture` — must pass (verifies the updated e2e assertion compiles and runs).

**Task 3:** `cargo test --workspace --features full --test e2e -- two_sponsors_lose_endorsement_strength_on_sanction --nocapture` — must pass.

**validate-pending-laptop gate (per `feedback_laptop_default_for_validate_pending.md`):**
```
cmd //c "scripts\brehon\cargo-test.bat --workspace --test e2e --features full -- v1_ship_3_fixtures --nocapture > .claude/PRPs/debug/v1-ship-3-module-e2e.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-ship-3-module-e2e.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-ship-3-module-e2e.log"
```
(run_in_background: true)

---

## 5. Lesson injections (mandatory, per advisor-orchestrator §2.4)

All three tasks touch `crates/server/tests/e2e.rs` (Tasks 2 and 3 directly; Task 1 does not):

**Inject for Tasks 2 and 3:**
- `feedback_lemmy_error_no_std_error.md` — Case A discipline: outer `LemmyResult<()>`; use `?` not `Box<dyn Error>`; `.map_err` only when crossing the LemmyResult boundary.
- `feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` pattern for e2e fixture DB connections.
- `feedback_clippy_test_style.md` — `#![deny(unwrap, expect)]` applies in tests; use `?` throughout.

**Inject for Task 2 (DTO reshape):**
- `feedback_fix_impl_enumerate_all_callsites.md` — `rg "CreateGovernanceReportResponse" crates/` before authoring to enumerate ALL callsites that construct or destructure the response type; any missed callsite → E0063 at compile.

**Do NOT inject** `feedback_junior_worker_e2e_edit_hang.md` for Task 3 (new module at EOF = append, not edit of existing lines; hang risk is for large mid-file edits). Inject it only if Task 2's `report_to_modlog_golden_path` assertion update touches >100 existing lines.

---

## 6. Scope boundaries (stop-and-ask tripwires)

- **Stop if:** planner proposes any DB migration — ship-3 is compute-logic + e2e only; schema is complete from ship-1/SL-d.
- **Stop if:** `rg "CreateGovernanceReportResponse" crates/` returns hits in more than 5 distinct files — callsite enumeration required before brief dispatch. (Current count: 2 files — clear.)
- ~~Stop if Task 1 Postgres image requires dump/restore~~ — RESOLVED (DQ ship3clarify01-001): no family switch; stay in pgautoupgrade.
- ~~Stop if GovernanceCaseSummaryView has no read(case_id) helper and >1 new fn needed~~ — RESOLVED (DQ ship3clarify01-002): 1 new fn (`read_summary_for_case`) required and in scope.

---

## 7. Not in scope for v1-ship-3

- RT-r2 decay calculator (separate lane: `brehon-fork-rt-r2`)
- Any additional e2e tests beyond the three deliverables above
- Operator runbook, performance testing, release pipeline (all deferred per PRD §1.3)
- ADR-014 federation outbound — not touched
- Any change to `governance_log` hash-chain machinery

---

## 8. Plan structure guidance

The plan should have 3 §13 tasks:

| Task # | Deliverable | Files | `[P]`? |
|---|---|---|---|
| T1 | docker-compose.yml Postgres pin | `docker/docker-compose.yml` | Yes — disjoint |
| T2 | `POST /report` DTO reshape + e2e update | `governance.rs`, `create_report.rs`, `e2e.rs` (report_to_modlog update) | Yes — disjoint from T1/T3 |
| T3 | `two_sponsors_lose_endorsement_strength_on_sanction` e2e | `e2e.rs` (new module at EOF) | Yes — append-only, disjoint from T2's edit site |

`[P]` is valid for all three: the files are disjoint across T1/T2/T3 (T2 edits `governance.rs` + `create_report.rs` + an existing e2e function; T3 appends a new module at EOF — these are distinct edit locations in `e2e.rs`).

Plan complexity: **5/10** per PRD §7.3. Estimated wall-clock: 3-5 days.

---

## 9. Pre-queue checklist (advisor to run before dispatching planning Junior)

- [ ] `/brehon-clarify` run on this brief — WP-1 (Postgres image family) and WP-2 (GovernanceCaseSummaryView load path) are the two open questions that may need user input before planning proceeds.
- [ ] `git -C C:/Users/barri/Developer/brehon-fork show governance-v0:.claude/PRPs/briefs/v1-ship-3-planning-1.md` — must succeed (brief committed before dispatch).
- [ ] `rg "CreateGovernanceReportResponse" crates/` — count distinct files; if >5, split impl tasks.
- [ ] `memory_search_hybrid("ship-3 sponsor liability e2e endorsement_strength", limit: 5)` — check for any prior lessons on the 2-sponsor pattern.

---

_Authored by advisor. Read-only reference for planning Junior. Do not modify during the planning run._
