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

1. **Postgres image pin** — `docker/docker-compose.yml` currently uses `pgautoupgrade/pgautoupgrade:18-alpine`; PRD calls for pinning to `postgres:16.4`. Planner must verify whether the intended target is a pin within the `pgautoupgrade` family (e.g., `pgautoupgrade:16-alpine`) or a switch to plain `postgres:16.4` — see §3 watchpoint 1.

2. **`POST /report` DTO reshape** — `CreateGovernanceReportResponse` currently returns `{ case_id: Option<ModerationCaseId>, threshold_met: bool }` (defined at `crates/api/api_common/src/governance.rs:56`). PRD §7.3 requires adding `case: GovernanceCaseSummaryView` to match the read surface (`ListGovernanceCasesResponse.cases: Vec<GovernanceCaseSummaryView>` at `governance.rs:88`). The handler at `crates/api/api_crud/src/governance/create_report.rs:192` returns the response at line 325. The existing `report_to_modlog_golden_path` e2e test must also be updated to assert against the new field — locate its return assertions before authoring the brief.

3. **`two_sponsors_lose_endorsement_strength_on_sanction` e2e** — named test in `crates/server/tests/e2e.rs`, in a new `mod v1_ship_3_fixtures` appended after the existing `mod v1_federation_inbound_e_fixtures` at line 16513. Asserts the §9 done-criterion: 1 sponsee + 2 sponsors (each with `endorsement_strength: 10`) → full report→jury→vote flow → both sponsors' `endorsement_strength` decremented by the per-severity-tier delta. Substrate: `apply_sponsor_liability` at `crates/api/api/src/governance/sponsor_liability.rs:486`.

All three deliverables are confirmed file-disjoint:
- Item 1: `docker/docker-compose.yml`
- Item 2: `crates/api/api_common/src/governance.rs` + `crates/api/api_crud/src/governance/create_report.rs` + assertion update in `crates/server/tests/e2e.rs` (report_to_modlog_golden_path)
- Item 3: `crates/server/tests/e2e.rs` (new module, separate function from item 2)

→ **`[P]` cohort marker is applicable.** Tasks 1, 2, and 3 may be dispatched in parallel. Task 2 and Task 3 both touch `e2e.rs` but at disjoint locations (existing function vs. new module at end of file) — flag if Junior workers cannot coordinate safely on that file.

---

## 2. Key file anchors (verify these before authoring the plan)

| File | Anchor | Purpose |
|---|---|---|
| `docker/docker-compose.yml:84` | `image: pgautoupgrade/pgautoupgrade:18-alpine` | Task 1 edit target |
| `crates/api/api_common/src/governance.rs:56` | `struct CreateGovernanceReportResponse` | Task 2 DTO to extend |
| `crates/api/api_common/src/governance.rs:87` | `struct ListGovernanceCasesResponse` | Task 2 mirror target |
| `crates/db_views/governance_case/src/lib.rs:44` | `struct GovernanceCaseSummaryView` | Task 2 field type to add |
| `crates/api/api_crud/src/governance/create_report.rs:192` | `create_report_inner` fn | Task 2 handler to update |
| `crates/api/api_crud/src/governance/create_report.rs:325` | `Ok(CreateGovernanceReportResponse { ... })` | Task 2 return site |
| `crates/server/tests/e2e.rs:2485` | `report_to_modlog_golden_path` | Task 2 e2e assertion update needed |
| `crates/server/tests/e2e.rs:16513` | `mod v1_federation_inbound_e_fixtures` | Task 3 insertion point (append after this mod) |
| `crates/server/tests/e2e.rs:16692` | EOF | Task 3 module goes here |
| `crates/api/api/src/governance/sponsor_liability.rs:486` | `apply_sponsor_liability` | Task 3 assertion target |

---

## 3. Watchpoints for the planner

**WP-1 (Task 1 — Postgres image family):** The current image is `pgautoupgrade/pgautoupgrade:18-alpine`, NOT plain `postgres:latest`. The PRD says pin to `postgres:16.4` — but switching from `pgautoupgrade` to `postgres` is a different family, not just a tag pin. The planner must:
- Check if there is a `pgautoupgrade:16-alpine` equivalent (to stay in the same image family).
- Check Lemmy 1.0-beta upstream's `docker/docker-compose.yml` for their tested Postgres image.
- If the correct pin is `postgres:16.4` (plain), confirm this is a safe switch given the existing local data volume (same major PG16, no dump/restore needed if data was written by PG16 already).
- If uncertain: flag as a clarify DQ before authoring the plan task.

**WP-2 (Task 2 — `GovernanceCaseSummaryView` population):** `GovernanceCaseSummaryView` is a complex view-type that requires a join across `moderation_case` + `jury_assignment` + community tables (per `crates/db_views/governance_case/src/lib.rs`). The `create_report_inner` fn at line 192 currently only returns `case_id` + `threshold_met` — adding the full view requires fetching it from the DB after the insert. The planner must:
- Read `crates/db_views/governance_case/src/lib.rs` to understand how `GovernanceCaseSummaryView` is loaded (is there a `read(case_id)` fn?).
- Confirm whether the view-load can happen inside the existing transaction or requires a separate query after the fn returns.
- If a `read(case_id)` helper doesn't exist: the task may need to author one first, OR return a subset of the view (and note this as a scope flag).

**WP-3 (Task 3 — e2e pattern):** The `two_sponsors_lose_endorsement_strength_on_sanction` test follows the same sponsor-liability pattern as `sponsor_liability_with_founder_multiplier` (starting at e2e.rs:3237). Read that test's fixture setup to reuse the same jury-drive helper (`admin_assign_jury` + `accept_jury_assignment` + `submit_jury_vote` chain). Do NOT re-implement the jury drive from scratch. The `mod v1_sl_d_fixtures` at line 13562 is the canonical reference for this pattern.

**WP-4 (Task 3 — governance_config seed for per-severity delta):** The `endorsement_strength` decrement amount is read from `governance_config` at runtime by `apply_sponsor_liability`. The test must either: (a) rely on seeded defaults from the migration, or (b) INSERT the config row inline. Check which severity tier the test will use (ContentRemoval = moderate, delta = -50 per existing tests) and verify that value is present in the `governance_config` seed rows at test runtime. Do NOT hardcode the delta value in the assertion without reading it from config — use the pattern from `sponsor_liability_with_founder_multiplier` which reads `liability.endorsement_delta_moderate` via governance_config.

**WP-5 (Task 3 — `endorsement_strength` initial value):** PRD §7.3 says "2 sponsors with `endorsement_strength: 10` initial." Existing tests use 100 as the seed value (see `e2e.rs:3773`). The planner must decide whether 10 is achievable given the governance_config floor (`sponsor_liability_floor` default). If `10 + delta` would go below floor and get clamped, the assertion must assert the clamped value. Clarify the test expectation explicitly in the plan task — don't leave it ambiguous.

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
- **Stop if:** `rg "CreateGovernanceReportResponse" crates/` returns hits in more than 5 distinct files — callsite enumeration required before brief dispatch.
- **Stop if:** Task 1 Postgres image analysis concludes the switch from `pgautoupgrade` to `postgres:16.4` requires a data volume dump/restore — surface to user before coding.
- **Stop if:** `GovernanceCaseSummaryView` has no `read(case_id)` helper and the Task 2 impl would require >1 new function — flag scope and clarify.

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
