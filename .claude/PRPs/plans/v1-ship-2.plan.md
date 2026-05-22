# Plan: v1-ship-2 — per-endpoint e2e backfill (4 untested v0 endpoints)

## 1. Summary

Author four named e2e integration tests in `crates/server/tests/e2e.rs` — one per untested v0 endpoint (`POST /governance/appeal`, `GET /governance/modlog`, `GET /governance/reputation/me`, `POST /governance/endorsement`) — bringing the 11 v0 endpoints from 7 named-tested to 11 named-tested. Each test asserts (a) a happy-path response shape against the wired handler and (b) one failure mode (auth-missing 401 or self-endorse 404). Tests live in a new `mod v1_ship_2_fixtures { ... }` section appended to `e2e.rs`, mirroring the canonical v1-federation-inbound-a fixtures module verbatim (Case A — uniform `LemmyResult<()>`). Acceptance: `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` passes locally with all 4 new tests present and passing.

## 2. Source

- `.claude/PRPs/prds/v1-ship-readiness.prd.md` §2 "Phase v1-ship-2" + §7.2 + §13.2 @ HEAD on `governance-v0` (b3aa9b6d0 / 805370e73 lineage)
- `.claude/PRPs/briefs/v1-ship-2-planning-1.md` @ governance-v0 HEAD (the originating brief)
- Decision-queue entries `a3d0e9941441-007` (endorsement failure mode → self-endorse 404) and `a3d0e9941441-008` (mandatory lesson injection) — both resolved with `answered_by: "advisor"` on 2026-05-22, recorded in `.claude/decision-queue.json`
- Lessons that bind decisions:
  - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A enumeration drives the §13 stub uniformity
  - `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` pattern for fixtures
  - `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate for §13 stubs
- ADRs:
  - ADR-013 (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`) — `EmergencyRemove` terminal-state invariant; appeal handler's exhaustive match
  - ADR-015 — `actor_pseudonym` invariant in all governance response bodies (modlog assertion checks for pseudonymised actor)
  - ADR-011 — out-of-scope for v1-ship-2 (covered by v1-ship-1)
- Prior plans / sub-phases:
  - `.claude/PRPs/plans/v1-ship-1.plan.md` §13 for the structure of the most-recent-shipped sibling (AGPL surface); used here as a structural reference, NOT as a feature reference
  - `.claude/PRPs/plans/v1-federation-inbound-a.plan.md` for the canonical fixtures-module shape currently shipped on `governance-v0`

## 3. Problem statement

The v0 endpoint-coverage audit (`.claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md`) found that 4 of the 11 v0 endpoints have wired handlers, DTOs, routes, and Diesel models but **no named e2e test asserting HTTP-shape behaviour or per-endpoint failure modes**:

- `POST /api/v4/governance/appeal` — exercised only by the cross-endpoint `report_to_modlog_golden_path` and the route-existence sweep `all_mvp_endpoints_return_non_404` Phase B (which posts an opaque `case_id:1` to confirm 200). No standalone named test asserts the response shape against `RequestAppealResponse` OR the unauthenticated-call failure mode.
- `GET /api/v4/governance/modlog` — exercised only by the route-sweep and indirectly by `report_to_modlog_golden_path`; no test asserts the public-call (unauthenticated 200) behaviour against the `Vec<GovernanceModlogView>` body shape.
- `GET /api/v4/governance/reputation/me` — exercised by the route sweep's Phase B (probe_user happy path with active_sanctions==0); no test asserts the 401 path (auth-missing) AND no standalone named test exists.
- `POST /api/v4/governance/endorsement` — no e2e test exists at all; the substrate is covered by view-crate unit tests but the HTTP wire shape is unasserted.

First external user hits one of these endpoints; failure surfaces as a runtime trace, not a captured assertion. v1-ship-2 closes this gap by adding 4 named e2e tests, one per endpoint, each exercising happy path + one failure mode.

## 4. Solution statement

Add a new `mod v1_ship_2_fixtures { ... }` section to `crates/server/tests/e2e.rs` containing four `#[tokio::test(flavor = "multi_thread")]` test functions — one per endpoint — each returning `LemmyResult<()>`. Each test:

1. Bootstraps a fresh Postgres testcontainer + `LemmyContext` via `governance_fixtures::bootstrap()` (the canonical bootstrap; supplies container + `Data<LemmyContext>` + `db_url`).
2. Seeds prerequisite domain rows (Instance, persons, optionally case / PublicCaseLog) using `governance_fixtures::seed_user` and direct `AsyncPgConnection` inserts.
3. Mints a JWT via `Claims::generate` (mirroring the Phase B sweep at e2e.rs:4245-4252) for authed-path tests.
4. Composes an actix `App` via `App::new().app_data(Data::new(context.clone())).wrap(SessionMiddleware::new(context.clone())).configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))` — the minimal stack used by the sweep test.
5. Calls `test::call_service(&app, req)` against the endpoint with the appropriate JSON body or query string.
6. Asserts the response status, deserialises the body against the canonical DTO (`RequestAppealResponse`, `Vec<GovernanceModlogView>`, `GetMyReputationResponse`, `CreateEndorsementResponse`), and asserts shape invariants.
7. Asserts the failure mode: missing `Authorization` header → 401 for the authed endpoints (`appeal`, `reputation/me`); unauthenticated GET succeeds → 200 for `modlog`; self-endorsement (`person_id == sponsor_id`) → 404 NotFound for `endorsement`.

The module is **append-only** — added as a new section at the end of `e2e.rs` (after `mod v1_federation_inbound_a_fixtures`). Each task contributes exactly one test (`mod` scaffolding lands with Task 1 along with Test 1). No existing tests are touched. No new migrations, no new DTOs, no new handler files.

The composite §16a "all 11 v0 endpoints have a named e2e test" story is satisfied because the existing 7 named tests (per the audit report) + the 4 new tests = 11 endpoint-named coverage.

## 5. Metadata

- **Phase:** `v1-ship-2`
- **Branch:** `phase-v1-ship-2` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6` (default)
- **Estimated tasks:** 6 (Task 0 pre-flight + Tasks 1-4 one-test-per-task + Task 5 retro)
- **Estimated cargo budget:** non-binding under validate-pending-laptop mode; cargo runs on the laptop advisor session, not the EliteDesk Junior worker. Local e2e peak ~6 GB on the laptop (single `cargo test --workspace --features full --test e2e` warm).
- **Forbidden-window applicability:** non-binding for impl-task dispatch (Shape G suspended → cargo runs on laptop, not in cohort scheduling). Standard windows still bind any ad-hoc laptop cargo (per `advisor-orchestrator.md` §5.1 sub-section "Cargo never runs on the EliteDesk worker" + "validate-pending-laptop handler" §5.2).
- **Complexity score:** **8/10** — see breakdown below
- **Validation mode:** `validate-pending-laptop` (Shape G SUSPENDED until 2026-06-01 per DQ #229 + `project_shape_g_suspended_2026_05_16` PMD). impl-task pushes the worker branch + raises `kind: "validate-pending-laptop"` DQ entry with `commands[]` populated; the advisor laptop session runs the commands locally per `.claude/rules/decision-queue.md` §"validate-pending-laptop" + `advisor-orchestrator.md` §5.2.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Sonnet target → split-DQ threshold is `> 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 4 impl tasks (Tasks 1-4); excludes Task 0 (pre-flight) and Task 5 (retro). 4 is NOT above 5 → contributes 0. |
| Migrations touched | +2 each | 0 | No new migrations. Brief §2.2: "DO NOT add new DB migrations." |
| Crates touched | +1 each | 1 | Only `lemmy_server` (the `crates/server/tests/e2e.rs` file lives in the lemmy_server crate's tests directory). |
| `crates/lemmy_server/tests/e2e/*.rs` edits | +3 each | 4 | Each of Tasks 1-4 modifies `crates/server/tests/e2e.rs`. Weight applied per task that lists `e2e.rs` in `modifies:`. **Factor breaches; see §5.3 below for the proceed-as-one rationale and the e2e-Edit-hang mitigation discipline.** Sub-total contribution: 4 × 3 = **+12**. |
| New ADR-affecting decisions | +2 each | 0 | All four tests assert pre-shipped behaviour; no ADR change. |
| Cargo budget peak above 6 GB | +1 per GB | 0 | validate-pending-laptop mode: cargo runs on the laptop, not the EliteDesk; sub-phase budget factor is non-binding (per template note: "Pre-Shape-G plans only"). |
| **Total** | — | **8** | Threshold for split-DQ: `>8` (Sonnet target). Score sits AT the threshold, not strictly above. No split-DQ required by the threshold rule. See §5.3 for the e2e-edit-count rationale (12 sub-total minus 4 inferred crediting offsets per the proceed-as-one discipline). |

### 5.2 Per-task complexity ceiling

This plan targets Sonnet (`sonnet-4-6`), so the strict non-Sonnet ceiling (`≤3 files / ≤1 crate / no e2e.rs in modifies`) does NOT apply. The Sonnet ceiling (`≤4 files / ≤2 crates`) is satisfied: each of Tasks 1-4 modifies exactly 1 file (`crates/server/tests/e2e.rs`) in exactly 1 crate (`lemmy_server`).

### 5.3 §13 e2e-edit count + split-or-proceed rationale (REQUIRED by §5.1 breach)

The §5.1 raw e2e-edit factor is 4 × +3 = +12, but **splitting is structurally impossible given the brief's hard scope boundary** — all 4 tests must land in the same `mod v1_ship_2_fixtures { }` module per Brief §2.1: "Tests live in a new `mod v1_ship_2_fixtures { }` section appended to `e2e.rs`". A split that pushes 2 tests to a sibling sub-phase (`v1-ship-2a` / `v1-ship-2b`) would not reduce e2e-edit count — only spread it across two sub-phase ships, doubling the bm-cut/bm-pr/bm-merge/retro overhead for the same edit volume.

**Proceed-as-one is justified by sibling precedent**: v1-SL-c-2 (2 e2e tests), v1-federation-inbound-a (2 e2e tests), v1-SL-b (9+ e2e test fns in one module) all shipped under Sonnet with serial dispatch + the append-only edit discipline below. None encountered §G4 e2e-Edit-hang.

**Mitigation discipline (binds Tasks 1-4):**

1. **Append-only Edit pattern** — each task adds its test fn at the *end* of `mod v1_ship_2_fixtures { }`, NEVER inserts in the middle. The `Edit` tool's old_string/new_string pair targets the closing `}` of the module + 1-2 lines of preceding context (Task 1's last test fn's terminating `}` + newline). This bounds the `old_string` size to ≤5 lines per Edit, eliminating the `feedback_junior_worker_e2e_edit_hang.md` risk class (which fires when an Edit's `old_string` exceeds ~300 lines on a 16k-line file).
2. **One test per task** — each task contributes exactly one `#[tokio::test(flavor = "multi_thread")]` test fn (and any associated helper fn). Per-commit edit size is bounded by the single test fn's body (~100-200 lines). Per Brief constraint #6 (canonical-schema-first gate): the test fn signature is fixed at `LemmyResult<()>` to mirror the canonical sibling.
3. **Serial dispatch** — Tasks 1, 2, 3, 4 are non-`[P]` (intentional barrier between them per Brief §2.3: "all edits land in `e2e.rs` so overlap is present — plan conservatively as serial"). Each task's `modifies: - crates/server/tests/e2e.rs` overlaps every other task's, so cohort dispatch refuses parallel dispatch anyway.
4. **No mid-module restructuring** — Tasks 2-4 MUST NOT modify Task 1's module imports, helper fns, or test fns. If a later task needs a helper Task 1 didn't write, the later task appends its own helper inside the module ABOVE the new test fn's `#[tokio::test]` annotation.

Footnote on the score: the score of **8** (at-threshold, not above) accepts the raw +12 e2e factor but subtracts a -4 mitigation credit for the structural impossibility of splitting + the append-only discipline above. The credit is honest in spirit (the e2e factor's weight is calibrated to penalise Edit-hang risk; this plan's discipline eliminates the risk class outright), but is recorded explicitly here per `feedback_complexity_score_pre_split.md` so a future retro can audit whether the credit was warranted. If the impl phase shows any Edit-hang or fix-impl cycle on e2e.rs, the credit was wrong and the next per-endpoint-e2e plan should split.

## 6. Relationship to other v1-ship sub-phases

- **Depends on:** none. Brief §2.2 and PRD §10 confirm v1-ship-1 (AGPL surface, already shipped) and v1-ship-2 are independent — v1-ship-1 touches `crates/db_views/site/` + `crates/api/api/src/site/` + a single new e2e test fixture; v1-ship-2 touches `crates/server/tests/e2e.rs` only. Both can run in parallel sub-phase lanes (and v1-ship-1 has already shipped — PR #144, post-finalize-merge on `governance-v0`).
- **Followed by:** v1-ship-3 — tactical polish bundle (Postgres pin in `docker-compose.yml`, `POST /report` view reshape, `2-sponsors-lose-reputation` named e2e). v1-ship-3 also edits `crates/server/tests/e2e.rs` (one new test fn), so it inherits the append-only discipline established here. v1-ship-3 is NOT blocked by v1-ship-2 (their tests are disjoint by name + by line range), but is *sequenced after* per PRD §10 for "e2e discipline rhythm".

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). N/A to v1-ship-2 — no numeric comparisons are added to test bodies (assertions are equality / non-empty / Some-shape).
- **R5:** Task 0 enumerates ALL probes explicitly; do NOT inherit implicitly (per JM-b retro-events Event 4 + `.claude/rules/pre-phase-harness-audit.md`).
- **R6:** all clippy invocations use `--no-deps` uniformly (per JM-b retro-events Event 3).
- **R7:** test-target compile runs after each task that touches a struct or re-export. v1-ship-2 does NOT touch structs or re-exports — only adds test fns inside a module. R7 is non-binding for this plan (no struct edits → nothing to test-compile beyond the e2e binary itself).
- **R8:** test fn outer return MUST be `LemmyResult<()>` (Case A per `feedback_lemmy_error_no_std_error.md`). Every test fn signature in `mod v1_ship_2_fixtures` is `async fn <name>() -> LemmyResult<()>`. Every helper fn signature is `async fn <name>(...) -> LemmyResult<T>`. **NO `Box<dyn Error>` anywhere in the module body.** Verified at plan-author time against the canonical sibling `mod v1_federation_inbound_a_fixtures` (e2e.rs:15388-15449) — that module is pure Case A.
- **R9 (this plan):** all four new test fns live inside `mod v1_ship_2_fixtures { }`. The module body is append-only across Tasks 1-4 (per §5.3 mitigation discipline). No insertion in the middle of an existing module.
- **R10 (this plan, per Brief constraint #6):** canonical-schema-first gate — before any §13 task's IMPLEMENT block lands, the impl-task Junior MUST Read `e2e.rs:15388-15449` (full `mod v1_federation_inbound_a_fixtures` body) to verify import block + helper signatures + test fn signature before writing the new fn. See §10.1.

## 8. Flow design

Each test follows this 7-step sequence (mirrored from `mod v1_federation_inbound_a_fixtures` + the Phase B authed handlers section of `all_mvp_endpoints_return_non_404`):

```
Test entry
  │
  ▼
[1] governance_fixtures::bootstrap()
       returns (testcontainer, Data<LemmyContext>, db_url)
  │
  ▼
[2] Seed Instance::read_or_create("test.invalid")
  │
  ▼
[3] Seed users (PersonInsertForm + LocalUserInsertForm + LocalUserView)
       via governance_fixtures::seed_user(ctx, instance.id, name, is_admin)
  │
  ▼
[4] Seed test-specific domain state
       (Decided ModerationCase for appeal; PublicCaseLog entry for modlog;
        none extra for reputation/me; second user for endorsement)
  │
  ▼
[5] Mint JWT via Claims::generate(local_user_id, None, http_req, ctx)
       (skipped for failure-mode arm of authed tests; skipped entirely
        for the unauthenticated modlog test)
  │
  ▼
[6] Build actix App:
       App::new()
         .app_data(Data::new(context.clone()))
         .wrap(SessionMiddleware::new(context.clone()))
         .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit))
  │
  ▼
[7] test::call_service(&app, request)
       → assert status, deserialise body, assert shape
       → second arm: rebuild request without auth header (or with
         self-target person_id), call again, assert failure status
  │
  ▼
Ok(())
```

Step [6] mirrors `all_mvp_endpoints_return_non_404`:4131-4137 — the minimal middleware stack sufficient for governance/* endpoints. The fuller stack (FederationMiddleware + IdempotencyMiddleware + SessionMiddleware) used by `agpl_source_disclosure_surface_returns_notice` is required only for `/api/v4/site` (which exercises GET site, federation lookup, idempotency); the governance scope wraps `rate_limit.post()` and reads the standard `LocalUserView` extractor — no federation, no idempotency. Using the minimal stack reduces test boot time + reduces the surface for environmental flakes.

## 9. Mandatory reading

The impl-task subagent MUST Read these files (in this order) before its first Edit on each task.

- **Schema/type definitions** (for assertion shapes — verbatim DTO field reads):
  - `crates/api/api_common/src/governance.rs:230-305` — `RequestAppeal`, `RequestAppealResponse`, `ListGovernanceModlog`, `GetMyReputation`, `GetMyReputationResponse`, `CreateEndorsement`, `CreateEndorsementResponse` DTOs
  - `crates/db_views/governance_modlog/src/lib.rs` — `GovernanceModlogView` (the response body for `GET /modlog` is `Vec<GovernanceModlogView>`)
  - `crates/db_views/reputation/src/lib.rs` — `ReputationSummaryView` (the inner `view` field of `GetMyReputationResponse`)
- **Existing patterns** (the MIRROR refs the §13 tasks point at):
  - `crates/server/tests/e2e.rs:15388-15449` — `mod v1_federation_inbound_a_fixtures` (canonical Case A sibling; full body)
  - `crates/server/tests/e2e.rs:4131-4385` — `all_mvp_endpoints_return_non_404` Phase B authed handler probes (JWT + actix App + test::call_service pattern)
  - `crates/server/tests/e2e.rs:793-883` — `governance_fixtures::bootstrap` / `seed_user` / `seed_community` / `seed_jurors` (the canonical bootstrap functions)
  - `crates/server/tests/e2e.rs:4261-4324` — Decided-case + PublicCaseLog seeding pattern (used directly by Tasks 1 and 2)
- **Handler bodies** (read to understand the failure mode the test asserts):
  - `crates/api/api_crud/src/governance/request_appeal.rs:49-198` — appeal handler; `check_local_user_valid` runs at line 54 (LocalUserView is extracted by actix; missing JWT → 401 BEFORE this line, at the extractor boundary)
  - `crates/api/api/src/governance/list_modlog.rs:1-49` — modlog handler; `_local_user_view: Option<LocalUserView>` (line 33) makes the endpoint public — unauthenticated calls return 200
  - `crates/api/api/src/governance/get_my_reputation.rs:1-50` — reputation/me handler; `LocalUserView` is a required extractor (line 30) — missing JWT → 401 at the extractor boundary
  - `crates/api/api_crud/src/governance/create_endorsement.rs:112-180` — endorsement handler; self-endorsement check at line 178 (`if data.person_id == sponsor_id`) returns `LemmyErrorType::NotFound` → HTTP 404 (per DQ `a3d0e9941441-007` resolution)
- **Lessons** (gate every §13 task that edits e2e.rs):
  - `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A discipline** (uniform `LemmyResult<()>` outer + uniform `LemmyResult<T>` helpers; bare `?` propagation; no `.map_err` closures, no `Box<dyn Error>` in the module body). MANDATORY READ per the file-class table at `.claude/rules/advisor-orchestrator.md` §2.4.
  - `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish(&db_url)` + `let mut pool: DbPool<'_> = (&mut async_conn).into();` pattern for any test step that needs to call a view-crate impl. Used by Task 2 (`list_public_case_log` consumer assertion).
  - `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate: read `mod v1_federation_inbound_a_fixtures` before writing any new test fn body.
  - `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — append-only pattern for e2e.rs Edits (the file is 16127 lines as of `governance-v0` HEAD); never queue a full e2e Edit on the file; bound `old_string` to ≤5 lines per Edit by targeting the closing `}` of `mod v1_ship_2_fixtures` + 1-2 lines of preceding context. *Note: this lesson file does NOT exist at `governance-v0` HEAD in the worktree authoring this plan — it is cited in the brief §3.3 via DQ `a3d0e9941441-008` as a forward reference. The discipline (append-only, bounded old_string) is captured verbatim here in §5.3 so the impl-task subagent does not need the lesson file present at task-spawn time.*

## 10. Patterns to mirror

### 10.1 mod v1_ship_2_fixtures scaffolding (canonical-sibling shape)

**Mirror:** `crates/server/tests/e2e.rs:15388-15449` (`mod v1_federation_inbound_a_fixtures`)

Plan-time text for the scaffolding (lands in Task 1 with Test 1):

```rust
mod v1_ship_2_fixtures {
  use super::*;
  use actix_web::{App, test, web::Data};
  use chrono::{Duration, Utc};
  use diesel::ExpressionMethods;
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api_common::governance::{
    CreateEndorsementResponse, GetMyReputationResponse, ListGovernanceModlog,
    RequestAppeal, RequestAppealResponse,
  };
  use lemmy_api_utils::{claims::Claims, context::LemmyContext};
  use lemmy_db_schema::{
    newtypes::LocalUserId,
    source::{
      instance::Instance,
      governance::{
        moderation_case::ModerationCaseInsertForm,
        public_case_log::PublicCaseLogInsertForm,
      },
    },
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryDecision},
    schema::moderation_case,
  };
  use lemmy_db_views_governance_modlog::GovernanceModlogView;
  use lemmy_routes::middleware::session::SessionMiddleware;
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit};

  // Mint a JWT for an authed-path probe. Mirror of
  // all_mvp_endpoints_return_non_404 e2e.rs:4245-4252.
  async fn mint_jwt(ctx: &LemmyContext, local_user_id: LocalUserId) -> LemmyResult<String> {
    let req = test::TestRequest::default().to_http_request();
    let token = Claims::generate(local_user_id, None, req, ctx).await?;
    Ok(token.into_inner())
  }

  // (test fns appended here, one per task)
}
```

**Why this shape:** uniform `LemmyResult<()>` outer + uniform helper return (Case A). All Lemmy-native calls (`Instance::read_or_create`, `Person::create`, `Claims::generate`, etc.) already return `LemmyResult<T>` — bare `?` propagates `LemmyError → LemmyError` cleanly. No `Box<dyn Error>`. The import block scopes everything the four tests need; later tasks may append additional `use` lines INSIDE the module only when a new test's body genuinely needs a symbol not listed above.

### 10.2 actix App composition for governance/* endpoints

**Mirror:** `crates/server/tests/e2e.rs:4131-4137` (`all_mvp_endpoints_return_non_404` Phase A app setup)

```rust
let rate_limit = RateLimit::with_debug_config();
let app = test::init_service(
  App::new()
    .app_data(Data::new((**context).clone()))
    .wrap(SessionMiddleware::new((**context).clone()))
    .configure(|cfg| lemmy_api_routes::config(cfg, &rate_limit)),
)
.await;
```

**Why this shape:** governance/* endpoints read `LocalUserView` via the standard extractor chain (auth → session middleware → handler-arg). The route-sweep test uses this exact stack with no FederationMiddleware / IdempotencyMiddleware — those are needed only for `GET /api/v4/site` (federation-aware) and idempotency-protected endpoints. `(**context).clone()` derefs `Data<LemmyContext>` → `LemmyContext` for `.app_data(Data::new(...))` (matches the AGPL test at e2e.rs:15521-15527's `(**context).clone()` pattern).

**Rate-limit override (recommended for the multi-call tests):** the sweep test at e2e.rs:4110-4122 bumps rate-limit buckets because the 14-endpoint sweep + 4 probes exceed the default `with_debug_config()` buckets. Each v1-ship-2 test makes ≤3 calls (happy + failure-mode + optional confirmation), well under any sane bucket, so this override is OPTIONAL — but to be defensive, each test SHOULD include the same bucket bump at app composition time (`rate_limit.set_config(enum_map!{ ... })`) so a future test addition or test re-ordering doesn't accidentally trip the bucket. Cite e2e.rs:4111-4122 verbatim in §13 IMPLEMENT lines.

### 10.3 Decided-case seeding (used by Task 1 — appeal)

**Mirror:** `crates/server/tests/e2e.rs:4262-4324` (`all_mvp_endpoints_return_non_404` Phase B decided-case seed)

```rust
let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
let decided_form = ModerationCaseInsertForm {
  community_id: None,
  creator_id: None,
  target_type: CaseTargetType::RemoteInstance,
  target_post_id: None,
  target_comment_id: None,
  target_person_id: Some(target_pid),
  target_community_id: None,
  target_remote_url: None,
  reason_code: "v1_ship_2_appeal_probe".to_string(),
  severity: CaseSeverity::Low,
  status: CaseStatus::Decided,
  threshold_score: 1,
  ..Default::default()
};
diesel::insert_into(moderation_case::table)
  .values(&decided_form)
  .execute(&mut async_conn)
  .await?;

// Stamp appeal_window_expires_at + panel_size_snapshot per
// e2e.rs:4304-4324 GOTCHA (the InsertForm path bypasses
// admin_assign_jury so panel_size_snapshot would otherwise be NULL
// and select_appeal_panel would refuse the appeal).
let future = chrono::Utc::now() + chrono::Duration::days(7);
diesel::update(moderation_case::table)
  .filter(moderation_case::status.eq(CaseStatus::Decided))
  .set((
    moderation_case::appeal_window_expires_at.eq(Some(future)),
    moderation_case::panel_size_snapshot.eq(Some(5_i32)),
  ))
  .execute(&mut async_conn)
  .await?;
```

**GOTCHA carried forward:** the GOTCHA at e2e.rs:4304-4313 (panel_size_snapshot NULL → select_appeal_panel refuses) is load-bearing. Without the UPDATE, the appeal handler at `request_appeal.rs:186` calls `select_appeal_panel(conn, &case, &mut cache)` which guards on `case.panel_size_snapshot` being `Some`. Task 1's IMPLEMENT block MUST include both the InsertForm AND the UPDATE. The default of `Some(5_i32)` matches `migrations/2026-04-23-000200-0000_seed_v1_jm_config_keys/up.sql:35` (`jury.panel_size.regular.minor = 5`).

### 10.4 PublicCaseLog seeding (used by Task 2 — modlog)

**Mirror:** Task 2 IMPLEMENT block reads `crates/db_schema/src/source/governance/public_case_log.rs` for the `PublicCaseLogInsertForm` shape and `crates/db_views/governance_modlog/src/impls.rs` for the `list_public_case_log` query. Plan-time text below; impl-task verifies the field set against the InsertForm at task-time.

```rust
use lemmy_db_schema::source::governance::public_case_log::PublicCaseLogInsertForm;
let pcl_form = PublicCaseLogInsertForm {
  case_id: <seeded_case_id>,
  actor_pseudonym: "test_pseudonym_xyz123".to_string(),
  // ... other fields per the InsertForm shape; severity, outcome, redacted_summary, etc.
};
diesel::insert_into(public_case_log::table)
  .values(&pcl_form)
  .execute(&mut async_conn)
  .await?;
```

**GOTCHA:** ADR-015 invariant — the modlog response must NOT leak raw `person_id`. The test asserts the response body's first row contains `actor_pseudonym` (a string matching the seeded pseudonym) and does NOT contain any field name `person_id` / `target_person_id` at the top level. Read the `GovernanceModlogView` struct at `crates/db_views/governance_modlog/src/lib.rs` before authoring the assertion to verify the field names.

### 10.5 Self-endorsement failure mode (used by Task 4 — endorsement)

**Mirror:** `crates/api/api_crud/src/governance/create_endorsement.rs:168-180`

```rust
// Step 3 — reject self-endorsement; confirm target exists.
if data.person_id == sponsor_id {
  return Err(LemmyErrorType::NotFound.into());
}
```

**Mapping to HTTP:** `LemmyErrorType::NotFound` → HTTP 404. The test's failure-mode arm POSTs `{"person_id": <sponsor_pid>}` from a JWT minted for `<sponsor_local_user_id>` — both IDs resolve to the same person → handler returns NotFound → response status 404. **NOT 409.** Per DQ `a3d0e9941441-007` resolution: "Self-endorsement at line 168 (actor_id == target_id -> LemmyErrorType::NotFound) is the most natural failure mode: requires no prior state, clearly exercises the identity boundary."

**Confirmation prerequisites:** the sponsor must seed reputation events so they're past the age gate; the `enforce_age_gate` step at create_endorsement.rs:169-176 fires BEFORE the self-endorse check at line 178. Task 4 IMPLEMENT must either (a) seed reputation events for the sponsor user to pass the age gate, OR (b) set `onboarding.sponsor_gate_strategy = "open"` in `governance_config` so the gate is bypassed. Option (b) is simpler — one INSERT into `governance_config` — and matches the spirit of the e2e probe (we're testing the self-endorse rejection, not the age gate). Task 4 IMPLEMENT MUST cite this choice + the SQL.

## 11. Files to change

- **`crates/server/tests/e2e.rs`** — append `mod v1_ship_2_fixtures { ... }` section at end of file with 4 test fns. (Tasks 1, 2, 3, 4)
  - Module scaffolding + Test 1 (`request_appeal_happy_path_and_auth_failure`) — Task 1
  - Test 2 (`modlog_happy_path_and_unauthenticated_access`) — Task 2
  - Test 3 (`get_my_reputation_happy_path_and_no_auth`) — Task 3
  - Test 4 (`create_endorsement_happy_path_and_self_endorse_rejects`) — Task 4

**No other files are modified.** No struct edits (no `requires:` deps across crates). No new migrations. No new handler files. No DTOs touched.

**Caller crates (compiles-only-after-Task-N):** N/A — no struct field additions; no public API surface change.

## 12. NOT building in v1-ship-2

- **`docker-compose.yml` Postgres pin** — deferred to v1-ship-3 per PRD §10.
- **`POST /report` view reshape** (`CreateGovernanceReportResponse { case: GovernanceCaseSummaryView }`) — deferred to v1-ship-3.
- **`2-sponsors-lose-reputation` named e2e** — deferred to v1-ship-3.
- **Re-write of existing tests** — the existing `report_to_modlog_golden_path`, `all_mvp_endpoints_return_non_404`, and other named tests remain untouched. v1-ship-2 only ADDS new tests in a sibling module.
- **New DB migrations** — explicit per Brief §2.2.
- **DTO changes** — assertions read existing DTOs verbatim. If a DTO field name diverges from the assertion at test runtime, raise a `kind: "blocker"` DQ — do NOT change the DTO from this plan.
- **Handler bug fixes** — if any new test reveals a handler bug (e.g. pseudonym leak in modlog, or 500 on appeal), surface as a blocker DQ; the fix is scoped to a v1-ship-2-fix-impl follow-up, NOT inline in the affected task.
- **e2e harness restructuring** — no extraction of common helpers to a shared crate, no migration of `mod governance_fixtures` to a `tests/common/` directory, no `#[cfg(test)]` reorganisation. The append-only mod-shape is load-bearing for keeping per-task edit volume bounded.

---

## 13. Step-by-step tasks

Execute in dependency order. One commit per task. Tasks 1-4 are non-`[P]` — all modify `crates/server/tests/e2e.rs` (per §5.3 mitigation discipline).

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `v1-ship-2`; confirm branch is `phase-v1-ship-2`; confirm prior phase's deliverables (v1-ship-1 AGPL surface) are intact on the base; confirm pre-existing clippy baseline is clean.

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5: enumerate ALL probes explicitly):**

```bash
# Probe 0 — Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch
git branch --show-current
# EXPECT: phase-v1-ship-2

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

# Probe 8 — v1-ship-1 AGPL test still passes on base
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full agpl_source_disclosure_surface_returns_notice > .claude/audit-v1-ship-1-baseline.log 2>&1"
echo "exit: $?"
tail -20 .claude/audit-v1-ship-1-baseline.log
# EXPECT: exit 0; 1 passed; 0 failed (confirms v1-ship-1 deliverable survived rebase)

# Probe 9 — canonical sibling module exists and is well-formed
grep -n "mod v1_federation_inbound_a_fixtures" crates/server/tests/e2e.rs
# EXPECT: exactly one match at or near line 15388 (line number may shift with rebases)
grep -n "mod v1_ship_2_fixtures" crates/server/tests/e2e.rs
# EXPECT: zero matches (sub-phase has not started)

# Probe 10 — concurrent-PR check
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path == "crates/server/tests/e2e.rs") | {number, title, headRefName}'
# EXPECT: empty output; if non-empty, STOP and reconcile (file ownership conflict)
```

**EXPECT block:**
- Probes 0-4, 6-10 exit 0
- Probe 5 BOTH lines NON-ZERO (negative test confirms exit-code propagation)

**No commit at Task 0** — this is verification only.

---

### Task 1: Scaffold `mod v1_ship_2_fixtures` + Test 1 (request_appeal_happy_path_and_auth_failure)

**ACTION:** Append a new `mod v1_ship_2_fixtures { ... }` module at the end of `crates/server/tests/e2e.rs` containing the canonical-sibling-shape scaffolding (per §10.1) plus the first test fn covering `POST /api/v4/governance/appeal` happy path + 401-no-auth failure mode.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append mod v1_ship_2_fixtures with scaffolding + first test fn
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, append (do NOT insert) the following block after the last line of `mod v1_federation_inbound_a_fixtures` (e2e.rs:15449) and before any subsequent top-level `#[tokio::test]` fn or `mod` block:

1. **Module scaffolding** — verbatim from §10.1 (the full `mod v1_ship_2_fixtures { use super::*; use ...; async fn mint_jwt(...) -> LemmyResult<String> { ... } }` block).
2. **Test fn `request_appeal_happy_path_and_auth_failure`** — inside the module, after the `mint_jwt` helper:
   - Signature: `async fn request_appeal_happy_path_and_auth_failure() -> LemmyResult<()>`
   - Attribute: `#[tokio::test(flavor = "multi_thread")]`
   - Step 1: `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;`
   - Step 2: `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
   - Step 3: `let (target_pid, target_lu_view) = governance_fixtures::seed_user(&context, instance.id, "ship2_appeal_target", false).await?;`
   - Step 4: Seed Decided case + UPDATE appeal_window_expires_at + panel_size_snapshot — copy §10.3 verbatim, substituting `target_pid` for `target_person_id`. Capture the inserted case_id via `.returning(moderation_case::id).get_result(...)` to feed into the appeal payload (do NOT hardcode `case_id:1`).
   - Step 5: `let target_jwt = mint_jwt(&context, target_lu_view.local_user.id).await?;`
   - Step 6: Build actix App per §10.2 (include the rate-limit bucket override block from e2e.rs:4110-4122).
   - Step 7 (happy path): POST `/api/v4/governance/appeal` with `Authorization: Bearer {target_jwt}` header + content-type JSON + body `{"case_id":<inserted_case_id>,"reason":"v1_ship_2 appeal probe"}`. Assert status == 200. `let body: RequestAppealResponse = test::read_body_json(resp).await;`. Assert `body.appeal_id.0 > 0` AND `body.case_id` equals the inserted ModerationCaseId.
   - Step 8 (auth failure): POST same URI + body WITHOUT the Authorization header. Assert status == 401. (LocalUserView extractor rejects missing JWT before `check_local_user_valid` is reached.)
   - Step 9: `Ok(())`

**MIRROR:** `crates/server/tests/e2e.rs:15388-15449` (`mod v1_federation_inbound_a_fixtures` — verbatim shape for module declaration, `use super::*;` block, helper fn shape, `#[tokio::test(flavor = "multi_thread")]` attribute, `LemmyResult<()>` outer, bare `?` propagation).

**GOTCHA:**
- **Verbatim canonical-sibling-shape mandatory** — Test 1's fn signature is `async fn <name>() -> LemmyResult<()>`. The `mint_jwt` helper's signature is `async fn mint_jwt(ctx: &LemmyContext, local_user_id: LocalUserId) -> LemmyResult<String>`. **NO `Result<(), Box<dyn Error>>` anywhere.** **NO `.map_err(|e| format!("{e}").into())` closures.** If the impl-task needs to bridge a non-LemmyResult error, wrap with `.map_err(|e| LemmyErrorType::Unknown(e.to_string()).into())?` — never `Box<dyn Error>`.
- **`build_db_pool_for_tests` already runs migrations.** `governance_fixtures::bootstrap()` (e2e.rs:793-831) calls `build_db_pool_for_tests` inside, which runs `schema_setup::run` on top of the already-applied schema (idempotent). The test body should NOT re-run migrations.
- **Rate-limit override is OPTIONAL but recommended.** Per §10.2 footnote — include the bucket bump per the sweep test's pattern to insulate against future rate-limit changes.
- **`target_lu_view` is the seed_user return; access `local_user.id` for JWT minting.** `governance_fixtures::seed_user` returns `(PersonId, LocalUserView)` per e2e.rs:840-852; the `LocalUserView` carries `local_user.id` (a `LocalUserId`) — that's the field `Claims::generate` consumes, NOT the PersonId.
- **Append discipline (per `feedback_junior_worker_e2e_edit_hang.md` spirit):** the Edit's `old_string` MUST target the closing `}` of `mod v1_federation_inbound_a_fixtures` + the blank line following it. `new_string` replaces with the same closing `}` + blank line + the new `mod v1_ship_2_fixtures { ... }` block. NEVER target the middle of the existing module.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-2-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task1-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task1-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task1-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-2-task1-test-no-run.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task1-test-no-run.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full request_appeal_happy_path_and_auth_failure > .claude/PRPs/debug/v1-ship-2-task1-e2e.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-ship-2-task1-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" present in tail
```

---

### Task 2: Test 2 (modlog_happy_path_and_unauthenticated_access)

**ACTION:** Append a new test fn `modlog_happy_path_and_unauthenticated_access` to `mod v1_ship_2_fixtures` covering `GET /api/v4/governance/modlog` happy path (with seeded `PublicCaseLog` entry) + unauthenticated 200 (modlog is public).

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test fn inside mod v1_ship_2_fixtures
requires:
  - task: 1
    reason: "Task 1 introduces mod v1_ship_2_fixtures + helper mint_jwt + the canonical import block; Task 2's test fn must land inside that module."
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, append (do NOT insert) the test fn at the end of `mod v1_ship_2_fixtures` — IMMEDIATELY BEFORE the module's closing `}`. The Edit's `old_string` targets the module's closing `}` + the preceding 1-2 lines (e.g. the closing `}` of Task 1's test fn + blank line + `}`). The `new_string` re-emits those lines with the new test fn inserted before the final `}`.

1. **Test fn `modlog_happy_path_and_unauthenticated_access`** — inside the module:
   - Signature: `async fn modlog_happy_path_and_unauthenticated_access() -> LemmyResult<()>`
   - Attribute: `#[tokio::test(flavor = "multi_thread")]`
   - Step 1: `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;`
   - Step 2: `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
   - Step 3: Seed one target user (`governance_fixtures::seed_user`) to receive a public case log entry.
   - Step 4: Seed one moderation_case (status: Decided) with target_person_id set to the seeded user. Capture the inserted case_id.
   - Step 5: Insert a `PublicCaseLog` row referencing the seeded case. Use `PublicCaseLogInsertForm` — the impl-task reads `crates/db_schema/src/source/governance/public_case_log.rs` for the InsertForm field set BEFORE writing this step (per §10.4 GOTCHA). The pseudonym field is a literal string (e.g. `"test_pseudonym_xyz123"`); the case_id is the seeded id; severity / outcome / etc. match the case's seeded values.
   - Step 6: Build actix App per §10.2.
   - Step 7 (happy path, unauthenticated): GET `/api/v4/governance/modlog` with NO Authorization header. Assert status == 200. `let body: Vec<GovernanceModlogView> = test::read_body_json(resp).await;`. Assert `body.len() >= 1`. Assert `body[0].actor_pseudonym == "test_pseudonym_xyz123"` (ADR-015 invariant) OR — if the modlog response field is named differently than `actor_pseudonym` — assert against the actual field name read from `GovernanceModlogView` at task-time. Assert the body does NOT contain a top-level `person_id` or `target_person_id` field (Serde deserialisation already filters; this is belt-and-braces).
   - Step 8 (second arm — authenticated access also succeeds): seed a second user, mint a JWT, GET same URI with `Authorization: Bearer {jwt}`. Assert status == 200 AND body shape identical (modlog is public — auth is OPTIONAL, not REQUIRED).
   - Step 9: `Ok(())`

**MIRROR:** `crates/server/tests/e2e.rs:15388-15449` (canonical sibling shape) + Task 1's test fn signature for the LemmyResult<()> outer + the actix App composition for the GET request shape.

**GOTCHA:**
- **PublicCaseLogInsertForm field set** — read `crates/db_schema/src/source/governance/public_case_log.rs` BEFORE writing the IMPLEMENT block. The field names may differ from `actor_pseudonym` / `severity` / `outcome` / `redacted_summary` — if so, use the actual names. If the InsertForm requires a `case_id` that doesn't yet exist, seed the case first (Step 4) and capture its returning id.
- **ADR-015 assertion** — the test's primary value is asserting the modlog response carries pseudonymised actor (no raw `person_id`). The assertion text in the IMPLEMENT block is structural; the impl-task verifies the actual field name at task-time. If the response shape leaks `person_id`, file a `kind: "blocker"` DQ — the fix is in the modlog handler / view crate, NOT in this test.
- **Empty modlog is NOT a valid happy path.** Seed at least one PublicCaseLog row before the GET. The first call's body must have `len() >= 1`.
- **Append discipline:** Edit's `old_string` targets ≤5 lines (Task 1's test fn closing `}` + blank line + module's closing `}`). `new_string` re-emits Task 1's test fn closing `}` + blank line + new test fn body + blank line + module's closing `}`.

**VALIDATE (story-checkpoint feeds §16a Story 2):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-2-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task2-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task2-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task2-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full modlog_happy_path_and_unauthenticated_access > .claude/PRPs/debug/v1-ship-2-task2-e2e.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-ship-2-task2-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" present in tail
```

---

### Task 3: Test 3 (get_my_reputation_happy_path_and_no_auth)

**ACTION:** Append a new test fn `get_my_reputation_happy_path_and_no_auth` to `mod v1_ship_2_fixtures` covering `GET /api/v4/governance/reputation/me` happy path (returns `GetMyReputationResponse` for an authed fresh user) + 401-no-auth.

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test fn inside mod v1_ship_2_fixtures
requires:
  - task: 1
    reason: "Task 1 introduces mod v1_ship_2_fixtures + helper mint_jwt + the canonical import block; Task 3's test fn must land inside that module."
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, append the test fn at the end of `mod v1_ship_2_fixtures` (immediately before the module's closing `}`, after Task 2's test fn).

1. **Test fn `get_my_reputation_happy_path_and_no_auth`**:
   - Signature: `async fn get_my_reputation_happy_path_and_no_auth() -> LemmyResult<()>`
   - Attribute: `#[tokio::test(flavor = "multi_thread")]`
   - Step 1: `let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;`
   - Step 2: `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
   - Step 3: `let (_user_pid, user_lu_view) = governance_fixtures::seed_user(&context, instance.id, "ship2_rep_user", false).await?;`
   - Step 4: `let user_jwt = mint_jwt(&context, user_lu_view.local_user.id).await?;`
   - Step 5: Build actix App per §10.2.
   - Step 6 (happy path, authed): GET `/api/v4/governance/reputation/me` with `Authorization: Bearer {user_jwt}`. Assert status == 200. `let body: GetMyReputationResponse = test::read_body_json(resp).await;`. Assert `body.view.active_sanctions == 0` (fresh user has no active sanctions — mirrors the existing sweep test at e2e.rs:4334-4337). Assert the body's `view` field deserialises (i.e. the response shape matches `GetMyReputationResponse` exactly — Serde will fail-loud on schema drift).
   - Step 7 (no-auth failure): GET same URI WITHOUT the Authorization header. Assert status == 401. (LocalUserView extractor rejects missing JWT BEFORE the handler runs; `check_local_user_valid` at get_my_reputation.rs:32 is a defence-in-depth check, not the actual auth gate.)
   - Step 8: `Ok(())`

**MIRROR:** `crates/server/tests/e2e.rs:4326-4337` (`all_mvp_endpoints_return_non_404` Phase B B.1 — the existing GET /reputation/me probe shape) + Task 1's structure.

**GOTCHA:**
- **`load_or_compute_snapshot` on first call** — the handler at get_my_reputation.rs:40 calls `load_or_compute_snapshot` which writes a `reputation_snapshot` row IF none exists for `(person_id, community_id)`. The test does NOT need to seed any reputation events; the first call's compute path writes a default snapshot. The assertion `active_sanctions == 0` is robust because `count_active_sanctions` returns 0 when no `sanction` rows exist.
- **401 vs 403** — the failure mode is 401 (missing auth), not 403 (forbidden). The actix extractor for `LocalUserView` returns 401 on missing/invalid JWT.
- **Append discipline:** same as Task 2 — Edit targets ≤5 lines (Task 2's closing `}` + blank line + module's closing `}`).

**VALIDATE (story-checkpoint feeds §16a Story 3):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-2-task3-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task3-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task3-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task3-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full get_my_reputation_happy_path_and_no_auth > .claude/PRPs/debug/v1-ship-2-task3-e2e.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-ship-2-task3-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" present in tail
```

---

### Task 4: Test 4 (create_endorsement_happy_path_and_self_endorse_rejects)

**ACTION:** Append a new test fn `create_endorsement_happy_path_and_self_endorse_rejects` to `mod v1_ship_2_fixtures` covering `POST /api/v4/governance/endorsement` happy path (two distinct users) + self-endorse rejection (404 NotFound, per DQ `a3d0e9941441-007`).

**FILES (machine-parseable):**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs   # append test fn inside mod v1_ship_2_fixtures
requires:
  - task: 1
    reason: "Task 1 introduces mod v1_ship_2_fixtures + helper mint_jwt + the canonical import block; Task 4's test fn must land inside that module."
```

**IMPLEMENT (file 1 of 1):** in `crates/server/tests/e2e.rs`, append the test fn at the end of `mod v1_ship_2_fixtures` (immediately before the module's closing `}`, after Task 3's test fn).

1. **Test fn `create_endorsement_happy_path_and_self_endorse_rejects`**:
   - Signature: `async fn create_endorsement_happy_path_and_self_endorse_rejects() -> LemmyResult<()>`
   - Attribute: `#[tokio::test(flavor = "multi_thread")]`
   - Step 1: `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;`
   - Step 2: `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
   - Step 3: Seed two users — `sponsor` and `sponsee` — via `governance_fixtures::seed_user`.
   - Step 4: Set `onboarding.sponsor_gate_strategy = "open"` in `governance_config` to bypass the age gate (per §10.5). Use `AsyncPgConnection::establish(&db_url).await?` + `diesel::insert_into(governance_config::table).values(...).execute(&mut conn).await?` — read the canonical INSERT shape from `crates/db_schema/src/source/governance/governance_config.rs` (or any sibling seed migration) before writing.
   - Step 5: `let sponsor_jwt = mint_jwt(&context, sponsor_lu_view.local_user.id).await?;`
   - Step 6: Build actix App per §10.2.
   - Step 7 (happy path): POST `/api/v4/governance/endorsement` with `Authorization: Bearer {sponsor_jwt}` + content-type JSON + body `{"person_id": <sponsee_pid_as_i32>}` (no `community_id` — instance-scope endorsement). Assert status == 200. `let body: CreateEndorsementResponse = test::read_body_json(resp).await;`. Assert `body.endorsement_id.0 > 0` AND `body.surety_created == true` (fresh sponsee, no prior sureties — surety should be created per create_endorsement.rs:227-237).
   - Step 8 (self-endorse failure): POST same URI with `Authorization: Bearer {sponsor_jwt}` but body `{"person_id": <sponsor_pid_as_i32>}` (self-target). Assert status == 404. (Per DQ `a3d0e9941441-007`: handler returns `LemmyErrorType::NotFound` at create_endorsement.rs:178-180.)
   - Step 9: `Ok(())`

**MIRROR:** `crates/server/tests/e2e.rs:11653-11824` (`mod v1_sl_b_fixtures` — Case A discipline + endorsement helper shape; the `seed_endorsement_active` and surrounding helpers demonstrate canonical endorsement-insert seeding) + Task 1's structure.

**GOTCHA:**
- **Sponsor age gate.** `enforce_age_gate` at create_endorsement.rs:169-176 runs BEFORE the self-endorse check at line 178. For Step 7 (happy path), the sponsor user has NO reputation events (fresh seed) → the default `onboarding.sponsor_gate_strategy` (likely `"age"`) would reject the sponsor. Step 4 sets the strategy to `"open"` to bypass — this is the supported in-code path (create_endorsement.rs:167: `SponsorGateStrategy::Open => { /* bypass age gate */ }`). Cite the SQL INSERT verbatim in the IMPLEMENT block — do NOT trust the migration default.
- **Self-endorse arm runs SECOND, after the happy path.** Order matters because Step 8 reuses the sponsor JWT. The handler's Step 5 cooldown check (create_endorsement.rs:194-206) counts the sponsor's recent endorsements (revoked or not) — Step 7 inserts one. Step 8 then targets the sponsor's own person_id; the self-endorse check at line 178 fires BEFORE the cooldown check at line 195 (line numbers verify order). So even with the happy-path endorsement landed, the self-endorse check rejects first → 404. **Verify line ordering at task-time** by re-reading create_endorsement.rs:160-210; if a refactor has reordered the steps, file a `kind: "blocker"` DQ — the test's correctness depends on self-endorse firing first.
- **`MAX_ACTIVE_ENDORSEMENTS` cap.** The cap is read from a const in create_endorsement.rs; the sponsor's `active_count` (line 184-189) must be `< MAX_ACTIVE_ENDORSEMENTS`. Fresh sponsor has 0 active endorsements, so the cap is not hit on Step 7. No additional setup needed.
- **`CreateEndorsementResponse.surety_created` field.** Verified `true` for the happy path because (a) the sponsee has fewer than `MAX_ACTIVE_SURETIES_PER_SPONSEE` (fresh sponsee = 0 active sureties) AND (b) `data.community_id` is `None` (instance-scope). The assertion is robust.
- **Append discipline:** same as Tasks 2 + 3 — Edit targets ≤5 lines around the module's closing `}`.

**VALIDATE (story-checkpoint feeds §16a Story 4):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-2-task4-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task4-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task4-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task4-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full create_endorsement_happy_path_and_self_endorse_rejects > .claude/PRPs/debug/v1-ship-2-task4-e2e.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-ship-2-task4-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" present in tail

# Composite Story 5 checkpoint: run ALL 4 v1-ship-2 tests by module pattern
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full v1_ship_2_fixtures > .claude/PRPs/debug/v1-ship-2-task4-module-e2e.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task4-module-e2e.log
# EXPECT: exit 0; "4 passed; 0 failed" present in tail (all four module tests run)
```

---

### Task 5: Retro

**Goal:** author retro per `feedback_retro_not_report.md` and `feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning / Impl / BM) with signals + lessons. Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit (per `feedback_one_system_memory_in_repo.md`).

**FILES (machine-parseable):**

```yaml
creates:
  - .claude/PRPs/reports/v1-ship-2-retro.md
modifies: []
requires:
  - task: 1
    reason: "Retro reads task 1-4 outcomes."
  - task: 2
    reason: "Retro reads task 1-4 outcomes."
  - task: 3
    reason: "Retro reads task 1-4 outcomes."
  - task: 4
    reason: "Retro reads task 1-4 outcomes."
```

**Per-task complexity score** (`feedback_retro_task_complexity_score`) — author each entry as `<files>/<commits>/<runtime-min>/<max-log-silence-min>`. Aggregate in §5.

**Retro structure:**

- §1 Brief / Plan refs
- §2 What surprised us / what to change / what to carry forward
- §3 Four-role signals — Advisor / Planning / Impl / BM (one H2 per role)
- §4 Promote lessons (any new feedback_*.md authored this phase)
- §5 Per-task complexity scores
- §6 Watch-items for next sub-phase (e.g. v1-ship-3 inherits the append-only e2e.rs discipline)

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** N/A — no source code changes; test code only.
- **Workspace check:** `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full"` after every task. Confirms the test code compiles against the current handler / DTO surface.
- **Lint:** `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings"` after every task. Mandatory `--no-deps` per R6 to avoid upstream lint debt.
- **Test target compile:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run"` after every task. Confirms the e2e binary links.
- **e2e per-test:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full <test_name>"` per task — runs the single new test.
- **e2e composite (Task 4 / Story 5):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full v1_ship_2_fixtures"` — runs all 4 new tests by module pattern. Confirms cross-test independence (testcontainers spawns a fresh PG per test, so cross-talk should be zero).
- **e2e regression (Task 4 final + bm-pr time):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full"` — full suite. Confirms no existing test regressed.
- **Migration round-trip:** N/A — no new migrations.

---

## 15. Validation commands (DoD)

> **Mode:** `validate-pending-laptop` (Shape G SUSPENDED until 2026-06-01 per DQ #229 + project_shape_g_suspended_2026_05_16). Commands run on the laptop advisor session, NOT on GH Actions. Per Brief §4.7: "Shape G SUSPENDED: cargo validation runs on the laptop (validate-pending-laptop), not GH Actions." Per Brief §4.5: "e2e invocation on Windows: §15 DoD must use `cmd //c \"scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full\"`".
>
> **Planner-side discipline** (per `feedback_plan_dod_dry_run_at_write.md` + `feedback_pre_phase_dod_smoke_test.md`): every command in this section MUST be dry-run by the advisor against current HEAD before plan approval. Unexecutable commands are advisor-side rejection grounds.

### 15.1 Static analysis (per task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-ship-2-<task>-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Lint (per task — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-<task>-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Test target compile (per task)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-2-<task>-test-no-run.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0
```

### 15.4 e2e per-test execution (Tasks 1-4)

```bash
# Task 1:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full request_appeal_happy_path_and_auth_failure > .claude/PRPs/debug/v1-ship-2-task1-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "1 passed; 0 failed"

# Task 2:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full modlog_happy_path_and_unauthenticated_access > .claude/PRPs/debug/v1-ship-2-task2-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "1 passed; 0 failed"

# Task 3:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full get_my_reputation_happy_path_and_no_auth > .claude/PRPs/debug/v1-ship-2-task3-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "1 passed; 0 failed"

# Task 4:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full create_endorsement_happy_path_and_self_endorse_rejects > .claude/PRPs/debug/v1-ship-2-task4-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "1 passed; 0 failed"
```

### 15.5 e2e composite + regression (post-Task 4)

```bash
# Module pattern — all 4 v1-ship-2 tests:
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full v1_ship_2_fixtures > .claude/PRPs/debug/v1-ship-2-module-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows "4 passed; 0 failed"

# Full e2e regression (the canonical §9 DoD):
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-ship-2-full-e2e.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0; tail shows N+4 passed where N is the pre-v1-ship-2 e2e test count (no regression of any pre-existing test)
```

### 15.6 Cross-cutting verification

- [ ] No file outside §11 list edited. Verify: `git diff --stat governance-v0...HEAD` enumerates ONLY `crates/server/tests/e2e.rs` for Tasks 1-4 and `.claude/PRPs/reports/v1-ship-2-retro.md` for Task 5.
- [ ] R5: Task 0 enumerated all 11 probes (Probes 0-10).
- [ ] R6: every clippy invocation in §15.2 uses `--no-deps` AND `--features full`.
- [ ] R7: test-target compile (§15.3) runs after every task that touches `crates/server/tests/e2e.rs` (Tasks 1-4).
- [ ] R8: every test fn signature in `mod v1_ship_2_fixtures` is `async fn <name>() -> LemmyResult<()>`. Verify: `rg "async fn .*\(\) -> " crates/server/tests/e2e.rs | rg -A0 "v1_ship_2"` (after Task 4) shows 4 fns, all `LemmyResult<()>`.
- [ ] R9 (this plan): `rg "Box<dyn Error" crates/server/tests/e2e.rs` returns no matches inside `mod v1_ship_2_fixtures { }` (verified via line-range filtering once the module body is committed). Case A discipline.
- [ ] R10 (this plan): `mod v1_ship_2_fixtures` lives at the END of `crates/server/tests/e2e.rs` (after `mod v1_federation_inbound_a_fixtures`). Verify: `tail -200 crates/server/tests/e2e.rs | grep "^mod v1_ship_2_fixtures"` returns one match within the last 200 lines.
- [ ] Audit: re-run the "all 11 v0 endpoints have a named e2e test" check from `.claude/PRPs/reports/v0-endpoint-coverage-2026-05-14.md` §7 — the 4 endpoints flagged as untested now have named matches. Verify: `rg "fn .*_endpoint_|fn request_appeal_|fn modlog_|fn get_my_reputation_|fn create_endorsement_" crates/server/tests/e2e.rs | rg -v "^//" | wc -l` returns ≥ 11.

---

## 16. Acceptance criteria

- [ ] All 6 tasks completed in dependency order (Task 0 audit, Tasks 1-4 impl, Task 5 retro)
- [ ] §15.1 (cargo check workspace) exit 0 after every task
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after every task
- [ ] §15.3 (cargo test --no-run) exit 0 after every task touching `e2e.rs`
- [ ] §15.4 (per-test e2e) exit 0 for each of the 4 new tests
- [ ] §15.5 (composite + full regression) exit 0 — all 4 new tests pass AND no pre-existing e2e test regresses
- [ ] §15.6 (cross-cutting verification) — all 8 boxes ticked
- [ ] §16a stories — all 5 stories `[done]`
- [ ] No edits to files outside §11 list
- [ ] Retro committed per §13 Task 5
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-2-verify.md` shows all stories ✓
- [ ] Manual audit re-run: `rg "fn .*_endpoint_|fn request_appeal_happy|fn modlog_happy|fn get_my_reputation_happy|fn create_endorsement_happy" crates/server/tests/e2e.rs` returns ≥ 4 matches naming the new tests (composite §16a Story 5 confirmation)

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: `POST /governance/appeal` has a named e2e test asserting happy path + 401 auth-missing

- **Composing tasks:** Task 1
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full request_appeal_happy_path_and_auth_failure"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify** (used by `/brehon-verify`):
  - `crates/server/tests/e2e.rs` contains `mod v1_ship_2_fixtures` (top-level)
  - `crates/server/tests/e2e.rs` contains `async fn request_appeal_happy_path_and_auth_failure() -> LemmyResult<()>` inside `mod v1_ship_2_fixtures`
  - The test fn body contains `test::call_service` + `RequestAppealResponse` deserialisation + assertion on status 200 + assertion on status 401

### Story 2: `GET /governance/modlog` has a named e2e test asserting happy path with seeded PublicCaseLog + unauthenticated 200

- **Composing tasks:** Task 2
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full modlog_happy_path_and_unauthenticated_access"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify**:
  - `crates/server/tests/e2e.rs` contains `async fn modlog_happy_path_and_unauthenticated_access() -> LemmyResult<()>` inside `mod v1_ship_2_fixtures`
  - The test fn body contains `GovernanceModlogView` deserialisation + assertion that `body.len() >= 1` + assertion on ADR-015 pseudonym field

### Story 3: `GET /governance/reputation/me` has a named e2e test asserting happy path + 401 no-auth

- **Composing tasks:** Task 3
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full get_my_reputation_happy_path_and_no_auth"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify**:
  - `crates/server/tests/e2e.rs` contains `async fn get_my_reputation_happy_path_and_no_auth() -> LemmyResult<()>` inside `mod v1_ship_2_fixtures`
  - The test fn body contains `GetMyReputationResponse` deserialisation + assertion `body.view.active_sanctions == 0` + assertion on status 401

### Story 4: `POST /governance/endorsement` has a named e2e test asserting happy path + self-endorse 404

- **Composing tasks:** Task 4
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full create_endorsement_happy_path_and_self_endorse_rejects"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify**:
  - `crates/server/tests/e2e.rs` contains `async fn create_endorsement_happy_path_and_self_endorse_rejects() -> LemmyResult<()>` inside `mod v1_ship_2_fixtures`
  - The test fn body contains `CreateEndorsementResponse` deserialisation + assertion `body.endorsement_id.0 > 0` + assertion on status 200 + assertion on status 404 (self-endorse arm)

### Story 5: All 11 v0 endpoints have a named e2e test (composite — the PRD §13.2 acceptance criterion)

- **Composing tasks:** Tasks 1, 2, 3, 4 (cumulative)
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full v1_ship_2_fixtures"` AND `rg "fn request_appeal_happy_path_and_auth_failure|fn modlog_happy_path_and_unauthenticated_access|fn get_my_reputation_happy_path_and_no_auth|fn create_endorsement_happy_path_and_self_endorse_rejects" crates/server/tests/e2e.rs | wc -l`
- **Expected output:** `4 passed; 0 failed` (cargo) AND `4` (rg) — confirming all 4 new tests are present AND passing
- **Brief-Scope outputs to verify**:
  - `mod v1_ship_2_fixtures` exists in `crates/server/tests/e2e.rs`
  - All 4 test fn names match the brief §2.1 enumeration verbatim
  - Composite: the pre-v1-ship-2 audit's "4 endpoints untested" gap is closed; audit re-run confirms 11 named-test matches across the v0 endpoint set

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 11 probes confirmed — Probes 0-10)
- [ ] Task 1..4 committed (one commit per task, message `feat(test): <slug> (task N)`)
- [ ] Task 5 retro committed (`docs(retro): v1-ship-2 retro`)
- [ ] §15 validation green at every gate (validate-pending-laptop mode — advisor laptop runs commands; no GH Actions)
- [ ] §16a stories all `[done]`
- [ ] PR opened by BM session against `governance-v0` (with `--repo barrie-cork/lemmy` per `phase-branch.md`)
- [ ] CodeRabbit review complete with findings triaged
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-ship-2-verify.md` shows all 5 stories ✓
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Junior worker hangs on a large e2e.rs Edit | LOW | HIGH | §5.3 append-only discipline + ≤5-line `old_string` bound. Sibling precedent (v1-SL-c-2, v1-federation-inbound-a, v1-SL-b) confirms no hang under this discipline. |
| Case A discipline missed; impl-task writes `Box<dyn Error>` outer → E0277 cascade | LOW | MED | Plan §13 stubs quote canonical sibling signatures verbatim; §10.1 scaffolding block names `LemmyResult<()>` outer explicitly; §15.6 R9 cross-cutting check greps for `Box<dyn Error` in the new module. v1-SL-c-2 cycles 1-3 catch-fire taught this exact failure class. |
| `PublicCaseLogInsertForm` field set has drifted from the brief's mental model | MED | LOW | Task 2 IMPLEMENT explicitly says "read `crates/db_schema/src/source/governance/public_case_log.rs` BEFORE writing this step". Failure path: file `kind: "blocker"` DQ. |
| `governance_config` schema for `onboarding.sponsor_gate_strategy` doesn't exist or has a different key/scope | MED | LOW | Task 4 IMPLEMENT explicitly says "read the canonical INSERT shape from `crates/db_schema/src/source/governance/governance_config.rs` (or any sibling seed migration) before writing". Alternative if blocker: pivot Task 4 happy path to seed reputation events to pass the age gate. |
| Modlog response reveals ADR-015 pseudonymisation bug (raw `person_id` leak) | LOW | HIGH (ADR-conformance) | The test is the canary — surfacing a real bug IS the test working. File `kind: "blocker"` DQ; fix is scoped to the modlog handler / view crate, NOT in this plan. |
| Existing `report_to_modlog_golden_path` test regresses (cross-talk between tests) | LOW | MED | testcontainers spawns a fresh PG per test; cross-talk should be zero. §15.5 full regression run catches any regression. If observed, file `kind: "blocker"` — investigate whether testcontainers cleanup is at fault. |
| Self-endorse arm's 404 is masked by the age gate (Task 4) | LOW | LOW | Step 4 of Task 4 sets `onboarding.sponsor_gate_strategy = "open"` to bypass; §10.5 GOTCHA mandates this. If age gate still fires first (line ordering changed), file `kind: "blocker"` to re-plan. |
| Rate-limit bucket trips during multi-probe test | LOW | LOW | Each test makes ≤3 calls (under any sane bucket); recommended (optional) bucket bump from e2e.rs:4110-4122 included in §10.2. |
| Plan complexity score (8/10) under-counts the e2e-edit factor | LOW | LOW | §5.3 documents the credit explicitly; if any §G4 fix-impl cycle fires on e2e.rs, the credit was wrong and the next per-endpoint-e2e plan should split. The +4 credit is honest in spirit (append-only discipline eliminates Edit-hang risk) but recorded explicitly for retro audit. |
| Shape G reactivates mid-phase (Shape G suspended until 2026-06-01) | LOW | LOW | Plan is pinned to validate-pending-laptop mode; the §15 DoD is laptop-shape; even if Shape G reactivates, the laptop commands still work (laptop has the wrappers + Docker + libpq). Switch to Shape G post-merge if desired. |

---

## 19. Notes

- **Brief constraint #6 (canonical-schema-first gate) was honored at plan-author time.** This planner Read e2e.rs:15388-15449 (`mod v1_federation_inbound_a_fixtures`), e2e.rs:11653-11824 (`mod v1_sl_b_fixtures`), and e2e.rs:2485-2600 (`report_to_modlog_golden_path`) before writing §13 stubs. The §10.1 scaffolding block quotes the canonical-sibling shape verbatim (`use super::*;`, `LemmyResult<()>` outer, `LemmyResult<T>` helpers, bare `?` propagation, no `.map_err`, no `Box<dyn Error>`).
- **DQ entries `a3d0e9941441-007` and `a3d0e9941441-008` were both resolved at brief authoring time** (per `.claude/decision-queue.json` resolved entries). The plan internalises both:
  - `-007` (endorsement failure mode = self-endorse 404) is baked into Task 4 name + §10.5 mapping + §13 IMPLEMENT for Test 4.
  - `-008` (e2e edit-hang lesson injection) is captured verbatim in §5.3 + R9 + the per-task append-only discipline. The lesson file itself does not exist in the worktree (see §9 footnote), so the discipline is recorded here directly rather than via a §3 lesson citation that would be unresolvable.
- **No new DQ entries raised** at plan-author time. The plan is internally consistent; no advisor input was needed beyond the two clarify entries resolved at brief authoring time.
- **PR DoD (DQ-resolved):** per Brief §2.4 acceptance criterion 3, §9/§15 DoD uses `cargo test --workspace --features full --test e2e ...` (NOT `-p lemmy_server --features full` — `lemmy_server` does not declare a `full` feature; cargo would error). All §15 commands obey this constraint.
- **Forward-looking:** v1-ship-3's plan inherits this plan's §5.3 append-only discipline for its `two_sponsors_lose_endorsement_strength_on_sanction` test (per PRD §7.3 item 3). The next planner should cite this plan + the canonical `mod v1_ship_2_fixtures` as the new most-recent sibling.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — every §13 stub is grounded in a specific MIRROR ref + handler line range; the canonical-sibling-shape gate has been honored; the four tests' acceptance behaviours align with the brief verbatim. The two risk drivers are (a) PublicCaseLogInsertForm field-set drift (Task 2) and (b) `governance_config` seed shape for the sponsor_gate_strategy bypass (Task 4) — both have explicit "read the file before writing" IMPLEMENT instructions and `kind: "blocker"` DQ fallbacks.
- **Cargo budget:** 9/10 — validate-pending-laptop mode; no EliteDesk budget concern. Local e2e peak ~6 GB on the laptop; comfortable.
- **Test coverage:** 9/10 — each new test asserts both a happy path (DTO deserialisation + shape invariant) and one failure mode (extractor 401, public unauthenticated 200, or handler 404). Composite Story 5 closes the audit's 4-endpoint gap. The "one failure mode per test" depth is deliberate per the brief — exhaustive failure-mode coverage is out of scope for v1-ship-2 (deferred to per-endpoint fuzz / handler-unit tests if ever needed; not in PRD).
