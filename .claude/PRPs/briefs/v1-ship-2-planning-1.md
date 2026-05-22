# Planning brief: v1-ship-2 — per-endpoint e2e backfill

## 1. Role + dispatch line

`[role:planning] v1-ship-2 per-endpoint e2e backfill — see .claude/PRPs/briefs/v1-ship-2-planning-1.md`

## 2. Scope

### 2.1 What to produce

A complete plan file at `.claude/PRPs/plans/v1-ship-2.plan.md` covering:

- **4 named e2e tests** in `crates/server/tests/e2e.rs`, one per untested endpoint:
  - `POST /api/v4/governance/appeal` → `request_appeal_happy_path_and_auth_failure`
  - `GET /api/v4/governance/modlog` → `modlog_happy_path_and_unauthenticated_access`
  - `GET /api/v4/governance/reputation/me` → `get_my_reputation_happy_path_and_no_auth`
  - `POST /api/v4/governance/endorsement` → `create_endorsement_happy_path_and_conflict`
- Each test covers: (a) happy path returning expected 2xx body shape, (b) one failure mode (auth-missing 401, conflict 409, or validation 400 — pick the most natural for each endpoint).
- Tests live in a new `mod v1_ship_2_fixtures { }` section appended to `e2e.rs`, following the `mod v1_sl_b_fixtures` / `mod v1_federation_inbound_a_fixtures` shape (Case A — uniform `LemmyResult<()>`).

### 2.2 Explicit boundaries

- **DO author:** the plan file only. No Rust code.
- **DO NOT author:** implementation code, migration files, or any file outside `.claude/PRPs/plans/`.
- **DO NOT** scope any endpoint beyond the 4 named above (v1-ship-3 owns docker-compose pin + POST /report view + sponsor-liability named test).
- **DO NOT** add new DB migrations (e2e tests call existing handlers against a test container — no schema changes needed).
- The plan MUST read the canonical sibling fixture modules before writing §13 stubs (see §3 Required reading).

### 2.3 Plan structure requirements (standard template)

Follow `.claude/PRPs/templates/plan.template.md`. Key sections:

- **§5 Complexity score** — estimate per the scoring rubric; note if ≥8 and flag split-or-proceed.
- **§9 Done definition** — `cargo test --workspace --test e2e --features full` passes with all 4 new test names present and passing.
- **§13 Tasks** — Task 0 must be a pre-flight harness audit (read `.claude/rules/pre-phase-harness-audit.md`). Subsequent tasks implement the 4 tests. Mark `[P]` where safe (tasks operating on distinct helper fns in the fixture module may be `[P]` if file-overlap check passes; but all edits land in `e2e.rs` so overlap is present — plan conservatively as serial unless the planner can prove disjoint line ranges).
- **§15 DoD commands** — must include `cargo test --workspace --test e2e --features full`. See §3 note on e2e invocation on Windows.
- **§16a Stories** — one story per test, plus a composite "all 11 v0 endpoints have a named e2e test" story.

### 2.4 Acceptance criteria for plan approval

1. §13 task stubs for `e2e.rs` edits explicitly state the test fn signature as `LemmyResult<()>` (Case A shape per §3 Required reading).
2. §4 Watchpoints cite specific lines or fn names in `e2e.rs` (not concept-only watchpoints).
3. §9 DoD lists the exact cargo command with `--workspace --features full` (not `-p lemmy_server`).
4. §16a has exactly 5 stories (4 per-endpoint + 1 composite).

## 3. Required reading

### 3.1 PRD scope (authoritative)

- `.claude/PRPs/prds/v1-ship-readiness.prd.md` §2 "Phase v1-ship-2" block — the definitive scope statement. Read verbatim; do not re-derive from memory.

### 3.2 Canonical sibling fixture modules (MUST READ before writing §13 stubs)

Read these in full before writing any §13 fixture stub. The plan's §13 stubs MUST mirror the error-shape and import set of the most recent sibling.

- `crates/server/tests/e2e.rs` lines 15388–15550 (`mod v1_federation_inbound_a_fixtures`) — the most recent sibling; read import block + 2–3 fn signatures verbatim.
- `crates/server/tests/e2e.rs` lines 11653–12683 (`mod v1_sl_b_fixtures`) — the canonical Case A reference cited in `feedback_lemmy_error_no_std_error.md`.
- `crates/server/tests/e2e.rs` lines 2485–2600 (`report_to_modlog_golden_path`) — the PRD-named shape reference.

### 3.3 Mandatory lessons (file-class injection: `e2e.rs` edit)

Per `.claude/rules/advisor-orchestrator.md` §2.4 file-class table, any edit to `crates/server/tests/e2e.rs` injects these lessons into §3 Required reading:

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A/B/C enumeration. §13 stubs MUST use Case A (uniform `LemmyResult<()>` outer + helpers). The plan §13 stubs must quote the chosen case and its canonical sibling reference. **When a v1-SL-* or v1-JM-* sibling module already exists in e2e.rs, mirror its error-shape case verbatim per this lesson — canonical-schema-first gate.**
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` pattern for e2e. Read before writing any test that needs a DB connection.
- `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — §13 stubs MUST mirror sibling fixture shape; failure to do so caused the 3-cycle catch-fire on v1-SL-c-2.

### 3.4 Supporting context

- `.claude/rules/pre-phase-harness-audit.md` — Task 0 pre-flight structure.
- `.claude/PRPs/plans/v1-ship-1.plan.md` §13 — prior-phase impl tasks for context on the `governance_fixtures` module shape (the `start_postgres` + `apply_all_schema` pattern Task 0 relies on).
- `crates/server/tests/e2e.rs` lines 4049–4380 (`all_mvp_endpoints_return_non_404`) — the existing sweep test; the new per-endpoint tests should NOT duplicate its setup, but should share `governance_fixtures::start_postgres` and `governance_fixtures::apply_all_schema`.

### 3.5 Handler locations (planner must read to write accurate IMPLEMENT blocks)

The plan must cite the actual handler + route for each endpoint. Planner should `grep` these before writing §13:

```
rg "fn request_appeal\|fn list_modlog\|fn get_my_reputation\|fn create_endorsement" crates/
rg "scope.*appeal\|scope.*modlog\|scope.*reputation\|scope.*endorsement" crates/routes/
```

## 4. Constraints

1. **File ownership:** the plan file lands in `.claude/PRPs/plans/v1-ship-2.plan.md`. Planner MUST NOT commit to `crates/`, `migrations/`, or `docs/brehon-law-inspired-network/`.
2. **Commit only the plan file** — no other files. Commit subject: `docs(plan): v1-ship-2 per-endpoint e2e backfill`.
3. **DQ mid-task push:** if a blocker DQ entry is raised mid-task, commit and push it to `governance-v0` immediately per `.claude/rules/decision-queue.md` "Mid-task visibility".
4. **Attribution:** never write `answered_by: "advisor"` in a DQ entry from the planner session.
5. **e2e invocation on Windows:** §15 DoD must use `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` (the `.bat` wrapper sets up vcpkg PATH for libpq). Never bare `cargo test` on Windows. Never `-p lemmy_server --features full` (lemmy_server has no `full` feature).
6. **§13 task stubs canonical-schema-first gate:** before finalising any §13 stub that adds code to `e2e.rs`, read the canonical sibling's full module body (§3.2 above) and quote 1–2 helper fn signatures + the test fn signature in the §13 stub. If the stub introduces a new return-type shape, raise a DQ before finalising.
7. **Shape G SUSPENDED:** cargo validation runs on the laptop (validate-pending-laptop), not GH Actions. The plan §9 DoD commands are the laptop commands, not workflow dispatch. Plan §15 must not reference `cargo-validate-workspace.yml`.
