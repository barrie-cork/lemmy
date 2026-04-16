# Plan: Phase 4b — Admin Backstops, EmergencyRemove Helper, Server Wiring, Golden-Path E2E

## Summary

Phase 4b completes Phase 4 of the Brehon governance MVP. It adds the two admin backstops (`admin_assign_jury`, `admin_close_case`) that make the jury workflow runnable without reputation gating, wires the `EmergencyRemove` helper (ADR-013 surface), composes the governance runtime root at `crates/server/src/governance.rs` (task 47), finishes the deferred ed25519 signing step inside `governance_log::append`, and delivers the single most load-bearing test in v0 — `report_to_modlog_golden_path`. It also lands the OQ-006 threshold-formula placeholder scaffolding in `create_report.rs` (task 49). On completion, the end-to-end governance mechanic is **actually runnable**: report → ThresholdMet → assign-jury → 3 votes → Decided → sanction → public modlog — against a real Postgres, with the hash chain verified and the ed25519 signatures verifiable.

## Source

- [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §3 Phase 4 — tasks 44–49 (tasks 38–43 shipped in Phase 4a at `e82534667`)
- Relevant [04](docs/brehon-law-inspired-network/04-data-model-and-api.md) sections: §5 (DTOs — AdminAssignJury, AdminCloseCase are new), §6.2 (admin handlers), §8 (aggregation rules — juror pool eligibility), §12 (server composition root)
- Relevant [05](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) sections: §2 (admin backstops are part of the 11-endpoint scope), §3 (5-juror panel, quorum 3, simple majority), §6 (decision flow)
- Relevant [06](docs/brehon-law-inspired-network/06-security-and-threat-model.md) sections: §2.2.1 (emergency-remove visibility), §6.1 (GDPR redaction)
- Relevant ADRs from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): **ADR-007** (5-juror/quorum-3/simple-majority, admin backstop allowed in v0), **ADR-008** (append-only signed log — ed25519 signing lands this phase), **ADR-010** (single-admin `close-case` is a v0 simplification; quorum + delay is v2), **ADR-013** (`EmergencyRemove` exhaustive match; jury cannot un-remove), **ADR-014** (outbound-only federation — not touched in 4b), **ADR-015** (GDPR pseudonyms — every log write uses `actor_pseudonym`)

## Problem Statement

After Phase 4a, the five report/read/vote routes compile and register, but the governance mechanic **cannot be exercised end-to-end**:

1. No handler exists to move a case from `ThresholdMet` into a jury panel — the admin backstop `POST /governance/admin/assign-jury` is missing. Without it, `submit_jury_vote` has no accepted assignments to vote against.
2. No handler exists to force-close a case — the admin backstop `POST /governance/admin/close-case` is missing. This is required for emergency unblocking and the `EmergencyRemove` post-facto review path.
3. The `EmergencyRemove` case-status variant exists in the enum, but no code in v0 creates such a case. Without the helper, the ADR-013 cross-cutting requirement is not wired through.
4. `governance_log::append` leaves `signature` NULL — Phase 4a deferred ed25519 signing until the full golden-path test was available to exercise the sign + verify loop.
5. `crates/server/src/governance.rs` does not exist. The governance routes live in `lemmy_api_routes` but no composition root ties them into the server binary or schedules governance-related background jobs (stubs only in 4b).
6. No integration test exercises the full flow. Task 48 is the single most load-bearing v0 test: it is the definition-of-done for Phase 4 per [05 §4 Step 4](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md).
7. The OQ-006 threshold-formula placeholder in `create_report.rs` is a raw `const V0_THRESHOLD: i64 = 3` without the companion scaffolding (constants module, TODO marker, compile-time commentary) that Phase 5 needs to replace with a reputation-weighted formula.

## Solution Statement

Phase 4b lands in ten commits. Task 0 audits the harness per `.claude/rules/pre-phase-harness-audit.md`. Task 1 adds the two new DTO pairs to `api_common/src/governance.rs` (matching the Phase 3 derive stack verified in the existing `SubmitJuryVoteResponse`). Tasks 2–3 implement `admin_assign_jury` and `admin_close_case` under `crates/api/api/src/governance/` — the assign-jury handler uses the already-defined `random()` SQL function from `lemmy_diesel_utils::utils::functions::random` (verified at `crates/diesel_utils/src/utils.rs:239`) for eligible-juror selection and runs inside a transaction because it writes five `jury_assignment` rows plus a governance log entry. Task 4 lands the `EmergencyRemove` helper as a library function (no HTTP route) that composes the existing Lemmy remove pathway + case creation + assign-jury post-facto. Task 5 registers both admin routes under `/governance/admin/*` in `crates/api/routes/src/lib.rs`, mirroring the existing `/governance` scope pattern. Task 6 finishes the threshold placeholder scaffolding in `create_report.rs`. Task 7 creates `crates/server/src/governance.rs` as the composition root per [04 §12](docs/brehon-law-inspired-network/04-data-model-and-api.md). Task 8 wires the deferred ed25519 signing inside `governance_log::append`. Task 9 adds the dev-dependencies required for the golden-path test harness. Task 10 writes the `report_to_modlog_golden_path` test that runs through the full flow and asserts every Phase 4 post-condition.

The composition root (task 7) stays **strictly declarative** per [03 §11](docs/brehon-law-inspired-network/03-architecture.md) — zero business logic; only registration calls into `lemmy_api_routes` and scheduling stubs for the Phase 5/6 background jobs.

The golden-path test (task 10) follows the IMPLEMENTATION-PLAN §3 Phase 4 task 48 approach-B strategy: use the admin backstop `POST /governance/admin/assign-jury` to force the case into a panel state rather than filing four separate reports to cross the threshold. This keeps OQ-006 deferred and the test deterministic.

## Metadata

| Field | Value |
|---|---|
| Type | HANDLER + CROSS_CUTTING + TEST |
| Complexity | HIGH |
| Crates Affected | `api/api_common`, `api/api`, `api/routes`, `server` (lib + dev-deps) |
| v0 Step | Step 4 from [05 §4](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) — completes the "first vertical slice" |
| Dependencies | Phase 1 (schema + triggers), Phase 2 (view crates), Phase 3 (DTOs), **Phase 4a** (routes + first 5 handlers + 3 helpers) |
| Estimated Tasks | 10 + Task 0 audit |
| Upstream HEAD | `e82534667` (Phase 4a close) |

---

## Flow Design

### Before State

```
╔══════════════════════════════════════════════════════════════════╗
║  Phase 4a complete — HTTP surface exists for 5 routes:          ║
║    POST /governance/report          (create_report)             ║
║    GET  /governance/case            (get_case)                  ║
║    GET  /governance/modlog          (list_modlog)               ║
║    GET  /governance/jury/me         (list_my_jury_queue)        ║
║    POST /governance/jury/vote       (submit_jury_vote)          ║
║                                                                 ║
║  Cross-cutting helpers exist:                                   ║
║    governance_log::append           (signature left NULL)       ║
║    actor_pseudonym_helper::get_or_create                        ║
║    redaction::scrub / scrub_json                                ║
║                                                                 ║
║  BUT: cases flagged ThresholdMet have NO path to a jury panel.  ║
║       submit_jury_vote has no Accepted assignments to vote on.  ║
║       EmergencyRemove (ADR-013) has no code path.               ║
║       governance_log rows are unsigned.                         ║
║       No composition root; no integration test; no end-to-end   ║
║       verification that the mechanic actually works.            ║
╚══════════════════════════════════════════════════════════════════╝
```

### After State

```
╔══════════════════════════════════════════════════════════════════╗
║  Phase 4b completes Phase 4 — governance mechanic is runnable:  ║
║                                                                 ║
║  New admin routes:                                              ║
║    POST /governance/admin/assign-jury   → admin_assign_jury     ║
║    POST /governance/admin/close-case    → admin_close_case      ║
║                                                                 ║
║  New library function (no HTTP route):                          ║
║    emergency_remove_open_case(target, admin_id, reason)         ║
║                                                                 ║
║  Composition root:                                              ║
║    crates/server/src/governance.rs                              ║
║    — declarative: route registration + (stub) background jobs   ║
║                                                                 ║
║  governance_log::append now signs:                              ║
║    SHA-256 hash chain (Postgres trigger) + ed25519 over         ║
║    entry_hash; signature UPDATE through the trigger gate        ║
║                                                                 ║
║  Golden-path e2e test (tests/e2e.rs):                           ║
║    report_to_modlog_golden_path — the v0 vertical slice.        ║
║                                                                 ║
║  Data flow (end-to-end, verified at runtime):                   ║
║    POST /governance/report                                      ║
║      → case{status=Open, threshold_score=1}                     ║
║      → governance_log{entry_kind=report_created, signed}        ║
║    POST /governance/admin/assign-jury (admin backstop)          ║
║      → 5 jury_assignment rows {status=Accepted}                 ║
║      → governance_log{entry_kind=jury_assigned, signed} x5 +1   ║
║    POST /governance/jury/vote x3                                ║
║      → 3 jury_vote rows; quorum reached                         ║
║      → case{status=Decided}; 1 sanction row;                    ║
║        1 public_case_log row (redacted); 3 reputation_event     ║
║      → governance_log{entry_kind=case_decided, signed} + chain  ║
║    GET /governance/modlog (unauth'd)                            ║
║      → 1 redacted entry visible                                 ║
║    Hash chain verified; all signatures verify with test pubkey. ║
╚══════════════════════════════════════════════════════════════════╝
```

### Endpoint Changes

| Endpoint | Before | After |
|---|---|---|
| `POST /api/v4/governance/admin/assign-jury` | didn't exist | Admin-only; selects 5 eligible jurors via `ORDER BY random() LIMIT 5`; inserts jury_assignment rows as Accepted (v0 testability); logs assignments |
| `POST /api/v4/governance/admin/close-case` | didn't exist | Admin-only; flips any case to `Closed`; logs audit entry. Single-admin per [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (quorum+delay is v2) |
| `governance_log::append()` (internal) | Leaves `signature` NULL | Signs `entry_hash` with ed25519 key from env; `UPDATE` passes the signature-gate trigger |
| `emergency_remove_open_case()` (internal, no route) | didn't exist | Library function callable from future emergency-remove route or admin tool; ADR-013 wiring |
| `crates/server/src/governance.rs` | didn't exist | Composition root — registers routes, schedules (stub) background jobs |

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/api/src/local_user/add_admin.rs` | all | **ADMIN_AUTH_CHECK** canonical pattern: `is_admin(&local_user_view)?;` |
| P0 | `crates/api/api_utils/src/utils.rs` | 149–156 | Exact `is_admin()` signature — takes `&LocalUserView`, returns `LemmyResult<()>`, checks `local_user.admin` |
| P0 | `crates/api/api/src/community/ban.rs` | 59–108 | **TRANSACTION_BOUNDARY** pattern used by `admin_assign_jury` — `run_transaction` with `.scope_boxed()` |
| P0 | `crates/api/api/src/governance/submit_jury_vote.rs` | 80–180 | In-repo transaction pattern with named helper (`process_vote`) — exactly mirror for `admin_assign_jury` |
| P0 | `crates/api/api/src/governance/governance_log.rs` | all | **Signing-deferred TODO** at line 66 — task 8 replaces it |
| P0 | `crates/api/api_crud/src/governance/create_report.rs` | 55–200 | Mirrors case-update pattern + threshold-flip logic; task 6 updates its threshold scaffolding |
| P0 | `crates/api/routes/src/lib.rs` | 491–506 | Existing `/governance` scope registration — admin sub-scope follows the same shape |
| P0 | `crates/api/routes/src/lib.rs` | 35–40, 136 | Handler import blocks — new admin handlers added here |
| P0 | `crates/api/api_common/src/governance.rs` | 78–87 | Existing `SubmitJuryVoteResponse` — DTO derive stack to mirror for AdminAssignJury/AdminCloseCase pairs |
| P0 | `crates/db_schema/src/source/governance/moderation_case.rs` | all | `ModerationCase` shape + `ModerationCaseInsertForm` + `AsChangeset` availability |
| P0 | `crates/db_schema/src/source/governance/jury_assignment.rs` | all | `JuryAssignmentInsertForm` — task 2 inserts 5 of these |
| P0 | `crates/diesel_utils/src/utils.rs` | 239 | `define_sql_function!(fn random() -> Text);` is already declared — do NOT redeclare |
| P0 | `crates/diesel_utils/replaceable_schema/triggers.sql` | 749–847 | Hash-chain trigger, signature-gate trigger, append-only trigger — task 8 obeys the signature-gate's NULL→non-NULL constraint |
| P0 | `crates/server/src/lib.rs` | 1–120 | Server entry point — task 7's composition root is module-wired here |
| P0 | `crates/server/tests/e2e.rs` | 1–140 | Existing `governance_fixtures` module + the `can_insert_moderation_case` test — task 10 extends this file, does NOT create a new test file |
| P0 | `crates/api/api_utils/src/claims.rs` | 37–79 | `Claims::generate` — task 10 uses this to mint test JWTs |
| P0 | `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | §3 Phase 4 tasks 44–49, §4 cross-cutting, §5 testing | Authoritative task-level spec |
| P0 | `docs/brehon-law-inspired-network/04-data-model-and-api.md` | §5, §6.2, §8, §12 | Authoritative DTO + handler contracts |
| P1 | `docs/brehon-law-inspired-network/06-security-and-threat-model.md` | §2.2.1 | EmergencyRemove visibility requirement |
| P1 | `.claude/rules/pre-phase-harness-audit.md` | all | Task 0 audit procedure |
| P1 | `.claude/rules/view-crate-selectable-template.md` | all | Not directly applicable (no new view crate) but illustrates the tuple-load + map pattern if the golden-path test reads derived rows |

**External Documentation:**

| Source | Version | Section | Why |
|---|---|---|---|
| [ed25519-dalek docs.rs](https://docs.rs/ed25519-dalek/latest/ed25519_dalek/) | 2.x (workspace declares it) | `SigningKey::from_bytes`, `Signer::sign` | Task 8 signature implementation |
| [diesel_async scoped_futures](https://docs.rs/diesel-async/latest/diesel_async/scoped_futures/trait.ScopedFutureExt.html) | matches workspace | `.scope_boxed()` | Verified pattern at `community/ban.rs:106` |
| [testcontainers GenericImage](https://docs.rs/testcontainers/latest/testcontainers/generic/struct.GenericImage.html) | workspace | `with_exposed_port`, `start` | Task 10 reuses `governance_fixtures::start_postgres` |

---

## Patterns to Mirror

**ADMIN_AUTH_CHECK:**
```rust
// SOURCE: crates/api/api_utils/src/utils.rs:149-156 + crates/api/api/src/local_user/add_admin.rs
// COPY THIS PATTERN:
use lemmy_api_utils::utils::is_admin;
// Inside the handler:
is_admin(&local_user_view)?;
```

**TRANSACTION_WITH_HELPER_FN (for admin_assign_jury):**
```rust
// SOURCE: crates/api/api/src/governance/submit_jury_vote.rs:92-180
// COPY THIS PATTERN:
pub async fn admin_assign_jury(
  Json(data): Json<AdminAssignJury>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<Json<AdminAssignJuryResponse>> {
  is_admin(&local_user_view)?;
  let admin_id = local_user_view.person.id;
  let admin_pseudonym = actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?;

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;
  let data_for_tx = data.clone();
  let pseudonym_for_tx = admin_pseudonym.clone();

  let outcome = conn
    .run_transaction(|conn| {
      async move {
        process_assignment(conn, admin_id, pseudonym_for_tx, data_for_tx).await
      }
      .scope_boxed()
    })
    .await?;

  Ok(Json(outcome))
}
```

**RANDOM_JUROR_SELECTION:**
```rust
// SOURCE: crates/diesel_utils/src/utils.rs:239 (random() SQL function) +
//         crates/db_schema_file/src/schema.rs (person table)
// COPY THIS PATTERN (inside the transaction closure):
use lemmy_diesel_utils::utils::functions::random;
use lemmy_db_schema_file::schema::{person, local_user};

let eligible: Vec<PersonId> = person::table
  .inner_join(local_user::table)
  .filter(person::deleted.eq(false))
  .filter(local_user::accepted_application.eq(true))
  .filter(person::id.ne(reporter_id))         // exclude reporter (if known)
  .filter(person::id.ne(target_person_id))    // exclude target person (if applicable)
  .order(random())
  .limit(5)
  .select(person::id)
  .load::<PersonId>(conn)
  .await?;
```

**JURY_ASSIGNMENT_INSERT:**
```rust
// SOURCE: crates/db_schema/src/source/governance/jury_assignment.rs:28-34
// COPY THIS PATTERN (inside the transaction closure, after selection):
use lemmy_db_schema::source::governance::jury_assignment::JuryAssignmentInsertForm;
use lemmy_db_schema_file::{enums::JuryAssignmentStatus, schema::jury_assignment};

let forms: Vec<JuryAssignmentInsertForm> = eligible
  .into_iter()
  .map(|person_id| JuryAssignmentInsertForm {
    case_id: data.case_id,
    person_id,
    status: JuryAssignmentStatus::Accepted, // v0 testability: skip Selected → Accepted step
  })
  .collect();

diesel::insert_into(jury_assignment::table)
  .values(&forms)
  .execute(conn)
  .await?;
```

**CASE_UPDATE (admin_close_case + threshold flip precedent):**
```rust
// SOURCE: crates/api/api_crud/src/governance/create_report.rs:104-135 (flip-status pattern)
// COPY THIS PATTERN:
use lemmy_db_schema_file::{enums::CaseStatus, schema::moderation_case};
use lemmy_diesel_utils::utils::functions::now;

diesel::update(moderation_case::table.find(data.case_id))
  .set((
    moderation_case::status.eq(CaseStatus::Closed),
    moderation_case::closed_at.eq(now().nullable()),
  ))
  .execute(conn)
  .await?;
```

**GOVERNANCE_LOG_CALL:**
```rust
// SOURCE: crates/api/api_crud/src/governance/create_report.rs:~170-190
// COPY THIS PATTERN (inside the transaction closure):
use lemmy_api::governance::governance_log;
use serde_json::json;

governance_log::append(
  &mut conn.into(),
  "jury_assigned",
  json!({
    "case_id": data.case_id,
    "juror_count": forms.len(),
  }),
  Some(admin_pseudonym.clone()),
)
.await?;
```

**ED25519_SIGNING (task 8):**
```rust
// SOURCE: ed25519-dalek 2.x API + crates/diesel_utils/replaceable_schema/triggers.sql:800-827
// NEW PATTERN — inside governance_log::append, after the INSERT:
use ed25519_dalek::{Signer, SigningKey};
use std::env;

// Load signing key from env (v0 per ADR-008; external signer is v2).
let key_hex = env::var("GOVERNANCE_LOG_SIGNING_KEY")
  .map_err(|_| LemmyErrorType::Unknown("GOVERNANCE_LOG_SIGNING_KEY not set".to_string()))?;
let key_bytes: [u8; 32] = hex::decode(&key_hex)
  .map_err(|_| LemmyErrorType::Unknown("GOVERNANCE_LOG_SIGNING_KEY not hex".to_string()))?
  .try_into()
  .map_err(|_| LemmyErrorType::Unknown("GOVERNANCE_LOG_SIGNING_KEY not 32 bytes".to_string()))?;
let signing_key = SigningKey::from_bytes(&key_bytes);
let signature = signing_key.sign(&row.entry_hash).to_bytes().to_vec();

// UPDATE passes the signature-gate trigger (triggers.sql:800-827) because
// ONLY signature changes, and we transition NULL → non-NULL exactly once.
diesel::update(governance_log::table.find(row.id))
  .set(governance_log::signature.eq(signature.clone()))
  .execute(conn)
  .await?;

// Return the row with the signature patched in-memory.
Ok(GovernanceLog { signature: Some(signature), ..row })
```

**GOLDEN_PATH_TEST_SKELETON:**
```rust
// SOURCE: crates/server/tests/e2e.rs:42-140 (governance_fixtures + can_insert_moderation_case)
// COPY THIS PATTERN (extend governance_fixtures, then add the test):
#[tokio::test]
async fn report_to_modlog_golden_path() -> Result<(), Box<dyn Error>> {
  // 1. Start Postgres container, apply all migrations + replaceable schema.
  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  // 2. Build AsyncPgConnection + DbPool::Conn (see feedback_async_pool_test_pattern.md).
  // 3. Seed: 2 users (reporter=A, target=B, admin=C), 1 community, 1 post by B.
  // 4. Build actix-web TestRequest chains that call each handler directly —
  //    do NOT spin up the full server binary. Use `actix_web::test::call_service`
  //    with a scoped `App::new().service(...)`.
  // 5. Assert counts in governance_log, jury_assignment, jury_vote, sanction,
  //    public_case_log, reputation_event at each step.
  // 6. Verify hash chain: for each row, digest(prev_hash || entry_kind ||
  //    payload::text || created_at) == entry_hash.
  // 7. Verify signature: ed25519 verify against the test pubkey over entry_hash.
  Ok(())
}
```

**ROUTE_REGISTRATION (admin sub-scope):**
```rust
// SOURCE: crates/api/routes/src/lib.rs:496-506 (existing governance scope)
// COPY THIS PATTERN — add the admin sub-scope nested under /governance:
.service(
  scope("/governance")
    .route("/report", post().to(create_report))
    .route("/case", get().to(get_case))
    .route("/modlog", get().to(list_modlog))
    .service(
      scope("/jury")
        .route("/me", get().to(list_my_jury_queue))
        .route("/vote", post().to(submit_jury_vote)),
    )
    .service(
      scope("/admin")
        .route("/assign-jury", post().to(admin_assign_jury))
        .route("/close-case", post().to(admin_close_case)),
    ),
),
```

---

## Files to Change

| File | Action | Justification |
|---|---|---|
| `crates/api/api_common/src/governance.rs` | UPDATE | Add AdminAssignJury, AdminAssignJuryResponse, AdminCloseCase, AdminCloseCaseResponse DTOs |
| `crates/api/api/src/governance/admin_assign_jury.rs` | CREATE | Task 44 handler |
| `crates/api/api/src/governance/admin_close_case.rs` | CREATE | Task 45 handler |
| `crates/api/api/src/governance/admin_emergency_remove.rs` | CREATE | Task 46 helper (library fn, no route) |
| `crates/api/api/src/governance/mod.rs` | UPDATE | Add `pub mod admin_assign_jury; pub mod admin_close_case; pub mod admin_emergency_remove;` |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Task 8 — ed25519 signing step replaces the TODO at line 66 |
| `crates/api/api/Cargo.toml` | UPDATE | Task 8 — add `ed25519-dalek` + `hex` deps if not already present |
| `crates/api/api_crud/src/governance/create_report.rs` | UPDATE | Task 6 — threshold placeholder scaffolding (constants module + TODO(brehon-fork) markers) |
| `crates/api/routes/src/lib.rs` | UPDATE | Task 5 — register admin sub-scope; add handler imports |
| `crates/server/src/governance.rs` | CREATE | Task 7 — composition root |
| `crates/server/src/lib.rs` | UPDATE | Task 7 — `pub mod governance;` declaration |
| `crates/server/Cargo.toml` | UPDATE | Task 9 — dev-deps: add `lemmy_api`, `lemmy_api_crud`, `lemmy_api_common`, `lemmy_api_routes`, `lemmy_api_utils` with `{ workspace = true }` under `[dev-dependencies]` (some already in `[dependencies]` — dev-deps are added only for test-only features like `full` or test utilities); add `ed25519-dalek`, `hex`, `uuid` for signature verification + JWT minting |
| `crates/server/tests/e2e.rs` | UPDATE | Task 10 — extend `governance_fixtures` + add `report_to_modlog_golden_path` |
| `.env.example` (optional, informational) | UPDATE | Document `GOVERNANCE_LOG_SIGNING_KEY` requirement |

---

## NOT Building (v0 scope limits)

- `POST /governance/admin/emergency-remove` HTTP route — task 46 produces a library function only; the route is a v1 concern
- Supermajority voting logic — [99 ADR-007](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (v1)
- Quorum+delay on `admin_close_case` — [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (v2 — single-admin is the documented v0 simplification)
- Passkey/MFA enforcement on admin endpoints — [99 ADR-007](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (optional in v0; `webauthn-rs` wiring is v1)
- Reputation-gated juror eligibility — Phase 5 (v0 uses `not target AND not reporter`)
- Reputation-weighted threshold formula in `create_report.rs` — task 6 lands scaffolding only; the actual formula is OQ-006 for Phase 5
- External log signer / Vault / HSM — [99 ADR-008](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) (v2); v0 uses `.env`-loaded key
- Merkle roots for off-chain anchoring — v3 per [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- Jury-deadline enforcement / stale-assignment reaping background job — Phase 5 stub in task 7 is a no-op; the real scheduler lands in Phase 5/6
- Outbound federation of governance signals — Phase 6
- Remaining 6 MVP endpoints (accept/decline jury, appeal, reputation/me, endorsement, list cases) — Phase 5
- Frontend UI — not in v0

---

## Step-by-Step Tasks

Execute in order. One commit per task using `feat(scope): task N — <summary>` format per `feedback_commit_hygiene_lockfiles_and_task_labels.md`. Each task has a MIRROR reference, exact file paths, and a validation command.

### Task 0: VERIFY branch + pre-phase harness audit

- **ACTION**: Confirm branch is `governance-v0`, clean tree, HEAD is `e82534667` (Phase 4a close), and run the full audit per `.claude/rules/pre-phase-harness-audit.md`
- **IMPLEMENT**:
  ```bash
  git branch --show-current   # expect: governance-v0
  git status                  # expect: clean
  git log --oneline -1        # expect: e82534667 docs(report): Phase 4a complete ...

  # Probe 1 — cargo-check wrapper honors -p
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api > .claude/audit-phase4b-cargo-check-p.log 2>&1"
  tail -20 .claude/audit-phase4b-cargo-check-p.log
  echo "exit: $?"

  # Probe 2 — cargo-check with --features full
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-phase4b-cargo-check-features.log 2>&1"
  tail -20 .claude/audit-phase4b-cargo-check-features.log
  echo "exit: $?"

  # Probe 3 — cargo-test wrapper honors target selection
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-phase4b-cargo-test.log 2>&1"
  tail -20 .claude/audit-phase4b-cargo-test.log
  echo "exit: $?"

  # Probe 4 — workspace baseline (must be green at Phase 4a close)
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/audit-phase4b-baseline.log 2>&1"
  tail -20 .claude/audit-phase4b-baseline.log
  echo "exit: $?"

  # Probe 5 — clippy baseline using the **cargo-clippy.bat** wrapper (NOT cargo-check.bat)
  # per feedback_clippy_vs_check_wrapper.md — do NOT typo this
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --no-deps > .claude/audit-phase4b-clippy.log 2>&1"
  tail -20 .claude/audit-phase4b-clippy.log
  echo "exit: $?"
  ```
- **EXPECT**: All five probes exit 0. If Probe 1 shows multiple crates compiling, the wrapper discards `-p` — **STOP and fix the wrapper** before task 1 per `.claude/rules/pre-phase-harness-audit.md`. If Probe 5's baseline is non-zero, either fix the lint in a pre-phase commit or narrow the task 10 DoD clippy command.
- **VALIDATE**: This task produces no code changes; it's a gate. The commit for this task is optional — if all probes are green, move to task 1 without committing.

---

### Task 1: ADD admin DTOs to `api_common/src/governance.rs`

- **ACTION**: Append four new DTOs — `AdminAssignJury`, `AdminAssignJuryResponse`, `AdminCloseCase`, `AdminCloseCaseResponse` — to `crates/api/api_common/src/governance.rs`, matching the Phase 3 derive stack verified at the existing `SubmitJuryVoteResponse` (file lines 78–87)
- **IMPLEMENT**:
  ```rust
  // In crates/api/api_common/src/governance.rs, append after SubmitJuryVoteResponse.

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  /// Request payload for `POST /api/v4/governance/admin/assign-jury`.
  /// Admin-only in v0 per [99 ADR-007]; flips a case from `ThresholdMet` to
  /// a five-juror panel with assignments auto-promoted to `Accepted` for
  /// testability (Phase 5 introduces a proper Selected → Accepted flow).
  pub struct AdminAssignJury {
    pub case_id: ModerationCaseId,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  /// Response from the admin-assign-jury backstop. `assigned_person_ids`
  /// is the list of the five jurors selected.
  pub struct AdminAssignJuryResponse {
    pub case_id: ModerationCaseId,
    pub assigned_person_ids: Vec<PersonId>,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  /// Request payload for `POST /api/v4/governance/admin/close-case`.
  /// Single-admin v0 simplification per [99 ADR-010] — quorum + delay
  /// on admin close-case is a v2 item. `reason` is REQUIRED and is
  /// included verbatim in the governance log audit entry.
  pub struct AdminCloseCase {
    pub case_id: ModerationCaseId,
    pub reason: String,
  }

  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  /// Response from the admin-close-case backstop.
  pub struct AdminCloseCaseResponse {
    pub case_id: ModerationCaseId,
    pub closed: bool,
  }
  ```
- **MIRROR**: `crates/api/api_common/src/governance.rs:78-87` (SubmitJuryVoteResponse derive stack)
- **GOTCHA**: `PersonId` and `ModerationCaseId` must already be imported in this file (verify — they are used by existing DTOs from Phase 3). If not, add `use lemmy_db_schema::newtypes::{ModerationCaseId, PersonId};` to the top of the file.
- **GOTCHA**: Do NOT introduce new `ts-rs` generation. The existing Phase 3 DTOs already ride the `ts-rs` feature; mirror their pattern exactly.
- **GOTCHA**: `AdminCloseCase.reason` is a plain `String` — redaction is applied by `governance_log::append` internally when the handler writes the audit entry, so the handler does NOT scrub `reason` itself.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_common --features full > .claude/build-phase4b-task01.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task01.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0; no new errors; `cargo check -p lemmy_api_common --features full` green
- **COMMIT**: `feat(api_common): task 1 — AdminAssignJury/AdminCloseCase DTO pairs`

---

### Task 2: CREATE handler `admin_assign_jury` (task 44)

- **ACTION**: Implement `admin_assign_jury` in `crates/api/api/src/governance/admin_assign_jury.rs` per [04 §6.2](docs/brehon-law-inspired-network/04-data-model-and-api.md)
- **IMPLEMENT**: Follow the TRANSACTION_WITH_HELPER_FN pattern (see Patterns section). Handler skeleton:
  1. `is_admin(&local_user_view)?` (BEFORE the transaction — read-only auth check)
  2. Fetch admin pseudonym: `actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?`
  3. Inside `run_transaction`:
     - Read the case: `moderation_case::table.find(data.case_id).first::<ModerationCase>(conn).await?`
     - **Exhaustive match on `case.status`** per [99 ADR-013](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md):
       - `CaseStatus::Open | ThresholdMet | EmergencyRemove` → proceed
       - `CaseStatus::InPanel` → return `LemmyErrorType::NotFound` (case already has a panel — do NOT silently re-assign)
       - `CaseStatus::Decided | Closed | AdminReview` → return `LemmyErrorType::NotFound`
     - Read `target_person_id` + `creator_id` (reporter) off the case row for the exclusion filter
     - Select 5 random eligible persons via the RANDOM_JUROR_SELECTION pattern (see Patterns); **if fewer than 5 eligible persons exist, return an error** rather than partial assignment
     - Insert the 5 `JuryAssignmentInsertForm` rows with `status = Accepted` (v0 testability per [IMPLEMENTATION-PLAN §3 Phase 4 task 44](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md))
     - Update the case: `status = CaseStatus::InPanel`
     - Emit a governance log entry per assigned juror: `governance_log::append(&mut conn.into(), "jury_assigned", json!({ "case_id": ..., "juror_pseudonym": ... }), Some(admin_pseudonym.clone())).await?` — loop 5 times, each with a fresh per-juror pseudonym fetched via `actor_pseudonym_helper::get_or_create`
     - Emit one panel-assembled log entry: `governance_log::append(&mut conn.into(), "panel_assembled", json!({ "case_id": ..., "juror_count": 5 }), Some(admin_pseudonym.clone())).await?`

  **Authoritative log entry kinds emitted by `admin_assign_jury`** (task 10 pins against these):
  | entry_kind | count | notes |
  |---|---|---|
  | `jury_assigned` | 5 (one per juror) | payload includes per-juror pseudonym |
  | `panel_assembled` | 1 | payload: `{ case_id, juror_count: 5 }` |

  **Total = 6 new governance_log rows per successful `admin_assign_jury` call.** Do NOT emit a separate `case_status_changed` entry for the Open → InPanel flip — the `panel_assembled` entry IS the canonical record of that transition. Adding a third kind would double-count and force task 10's assertions to drift.
  4. Return `AdminAssignJuryResponse { case_id, assigned_person_ids }`

- **MIRROR**: `crates/api/api/src/governance/submit_jury_vote.rs:92-180` (handler + named process_* helper split so the `run_transaction` closure stays readable and the `large_futures` workspace lint is respected)
- **GOTCHA**: `&mut conn.into()` inside `governance_log::append` — the helper expects `&mut DbPool<'_>`, and inside a transaction `conn.into()` produces that. Verified at `submit_jury_vote.rs:112`.
- **GOTCHA**: `use lemmy_diesel_utils::connection::get_conn;` — same import path as `submit_jury_vote.rs`
- **GOTCHA**: `use diesel_async::scoped_futures::ScopedFutureExt;` — required for `.scope_boxed()`; this is the correct trait path per the decision-queue resolved entry #2
- **GOTCHA**: **No `_ =>` catchall in the status match.** The match must exhaustively list `CaseStatus` variants per [99 ADR-013](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). If the compiler complains about a missing variant, ADD the variant to the match — do NOT paper over with `_`.
- **GOTCHA**: **`random()` return type mismatch.** The existing `define_sql_function!(fn random() -> Text)` at `crates/diesel_utils/src/utils.rs:239` declares the return as `Text`, not `Float`. Diesel's `.order()` expects a sortable type, and Postgres coerces `Text` at runtime — but Diesel's type system may reject the call at compile time. If `.order(random())` fails to compile with a type-mismatch error, fall back to an inline SQL expression:
  ```rust
  .order(diesel::dsl::sql::<diesel::sql_types::Float>("random()"))
  ```
  This bypasses the function-level type declaration. Do NOT re-declare `random()` with a different return type — that would break other call sites in the workspace.
- **GOTCHA**: Eligibility filter in v0 is **`not target AND not reporter`** per [IMPLEMENTATION-PLAN §3 Phase 4 task 44](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md). Reputation gating is a Phase 5 concern.
- **GOTCHA**: Update `crates/api/api/src/governance/mod.rs` to add `pub mod admin_assign_jury;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-phase4b-task02.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task02.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0
- **COMMIT**: `feat(api): task 2 — admin_assign_jury handler (POST /governance/admin/assign-jury)`

---

### Task 3: CREATE handler `admin_close_case` (task 45)

- **ACTION**: Implement `admin_close_case` in `crates/api/api/src/governance/admin_close_case.rs` per [04 §6.2](docs/brehon-law-inspired-network/04-data-model-and-api.md)
- **IMPLEMENT**:
  1. `is_admin(&local_user_view)?`
  2. Validate `data.reason.trim().is_empty()` — if so, return `LemmyErrorType::Unknown("close-case reason required".into())`
  3. Fetch admin pseudonym
  4. Inside `run_transaction` (single-table-update still uses a transaction so the log append + the case update are atomic):
     - Read case; **exhaustive match on `case.status`**:
       - `Open | ThresholdMet | InPanel | Decided | EmergencyRemove | AdminReview` → proceed (admin can force-close any non-Closed case)
       - `Closed` → return `LemmyErrorType::NotFound` ("case already closed")
     - `diesel::update(moderation_case::table.find(data.case_id)).set((moderation_case::status.eq(CaseStatus::Closed), moderation_case::closed_at.eq(now().nullable()))).execute(conn).await?`
     - Emit `governance_log::append(&mut conn.into(), "admin_case_closed", json!({ "case_id": data.case_id, "reason": data.reason.clone() }), Some(admin_pseudonym.clone())).await?` — the redaction service inside `append` handles identifier scrubbing of `reason`
  5. Return `AdminCloseCaseResponse { case_id: data.case_id, closed: true }`

- **MIRROR**: `crates/api/api_crud/src/governance/create_report.rs` (case update pattern) + `crates/api/api/src/governance/submit_jury_vote.rs` (transaction wrapper)
- **GOTCHA**: The v0 simplification is documented at [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): single-admin close-case is allowed in v0; quorum + delay is v2. Add a module-level doc comment noting this.
- **GOTCHA**: **Exhaustive match** — all 7 `CaseStatus` variants must appear in the match arm. No `_ =>`.
- **GOTCHA**: `reason` is a plain `String` that flows into the governance log payload; `governance_log::append` runs `scrub_json` on the payload, so the handler does NOT scrub it separately.
- **GOTCHA**: Update `crates/api/api/src/governance/mod.rs` to add `pub mod admin_close_case;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-phase4b-task03.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task03.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0
- **COMMIT**: `feat(api): task 3 — admin_close_case handler (POST /governance/admin/close-case)`

---

### Task 4: CREATE helper `emergency_remove_open_case` (task 46)

- **ACTION**: Implement the `EmergencyRemove` wiring as a library function in `crates/api/api/src/governance/admin_emergency_remove.rs` per [IMPLEMENTATION-PLAN §3 Phase 4 task 46](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) and [06 §2.2.1](docs/brehon-law-inspired-network/06-security-and-threat-model.md)
- **IMPLEMENT**:
  ```rust
  //! `EmergencyRemove` wiring per [99 ADR-013] and [06 §2.2.1].
  //!
  //! This helper is callable from a future emergency-remove HTTP route or
  //! from a direct admin tool. v0 does NOT expose an HTTP surface for it —
  //! the function exists so the cross-cutting `EmergencyRemove` requirement
  //! ([IMPLEMENTATION-PLAN-v0.md §4.3]) is wired through the codebase and
  //! so the golden-path test in task 10 can exercise the code path.
  //!
  //! ## Per [99 ADR-013]: the jury CANNOT un-remove the content.
  //!
  //! The post-facto jury review produces accountability artefacts (vote
  //! records, reputation deltas for the admin who pressed the button) but
  //! the content stays down regardless of outcome. The helper's
  //! governance log entry is tagged `entry_kind = "emergency_removed"`
  //! and is extra-visible per [06 §2.2.1].

  pub async fn emergency_remove_open_case(
    pool: &mut DbPool<'_>,
    admin_id: PersonId,
    target: EmergencyRemoveTarget,
    reason: String,
  ) -> LemmyResult<ModerationCaseId> {
    // 1. Remove the content via Lemmy's existing remove pathway.
    //    For v0, this is a direct Diesel UPDATE on the target row's
    //    `removed` column (wrapping the existing Lemmy helper when one
    //    is available). Do NOT re-implement the remove logic — mirror
    //    the call from crates/api/api/src/post/remove.rs (or the
    //    comment/community equivalent) based on target_type.
    // 2. Insert a ModerationCase with status = EmergencyRemove.
    // 3. Call admin_assign_jury (in-process, not via HTTP) for
    //    post-facto review — reuse the assignment logic.
    // 4. Emit `governance_log::append` with entry_kind = "emergency_removed"
    //    and a visibility marker payload key.
    todo!("fill in — see MIRROR refs below")
  }
  ```
  - Define `EmergencyRemoveTarget` as a small enum that maps to `CaseTargetType` + the target row id, so callers can't pass incoherent target data
  - Wrap the whole body in a `run_transaction` so the remove + case insert + assignment + log are atomic — partial execution would leave the site in an illegal-content state
  - The post-facto jury assignment uses the same logic as `admin_assign_jury` task 2 — **factor the juror selection + insert into a private helper** inside `admin_assign_jury.rs` (e.g. `pub(super) async fn select_and_assign_panel(...)`) and call it from both task 2 and task 4. This avoids duplication of the eligibility query.

- **MIRROR**: `crates/api/api/src/post/remove.rs` (Lemmy's existing remove pathway), `crates/api/api/src/governance/admin_assign_jury.rs:<panel-selection helper>`
- **GOTCHA**: **Per [99 ADR-013]** — the function's doc comment MUST state "the jury cannot un-remove the content." Reviewers will check for this. Also, the log `entry_kind` MUST be `"emergency_removed"` (not `"case_created"`) so modlog consumers can render the visibility callout per [06 §2.2.1](docs/brehon-law-inspired-network/06-security-and-threat-model.md).
- **GOTCHA**: No HTTP route registration in task 5 for this helper — it stays library-only in v0.
- **GOTCHA**: If the existing Lemmy `Post::update` / `Comment::update` signatures don't expose a `removed: true` setter accessible from `lemmy_api`, stub the remove step with a direct `diesel::update(...).set(post::removed.eq(true))` call and TODO-flag it: `// TODO(brehon-fork): wire to canonical Lemmy remove helper in Phase 5`. Document this deviation in the final report.
- **GOTCHA**: Update `crates/api/api/src/governance/mod.rs` to add `pub mod admin_emergency_remove;`
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-phase4b-task04.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task04.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0
- **COMMIT**: `feat(api): task 4 — emergency_remove_open_case helper (ADR-013 wiring)`

---

### Task 5: WIRE admin routes in `crates/api/routes/src/lib.rs`

- **ACTION**: Register the two admin handlers in the existing `/governance` scope
- **IMPLEMENT**:

  1. In the `lemmy_api::governance::` import block at `crates/api/routes/src/lib.rs:35-40`, add:
     ```rust
     admin_assign_jury::admin_assign_jury,
     admin_close_case::admin_close_case,
     ```
  2. At `crates/api/routes/src/lib.rs:496-506`, add the admin sub-scope inside the existing `/governance` scope (see the ROUTE_REGISTRATION pattern above):
     ```rust
     .service(
       scope("/governance")
         .route("/report", post().to(create_report))
         .route("/case", get().to(get_case))
         .route("/modlog", get().to(list_modlog))
         .service(
           scope("/jury")
             .route("/me", get().to(list_my_jury_queue))
             .route("/vote", post().to(submit_jury_vote)),
         )
         .service(
           scope("/admin")
             .route("/assign-jury", post().to(admin_assign_jury))
             .route("/close-case", post().to(admin_close_case)),
         ),
     ),
     ```

- **MIRROR**: `crates/api/routes/src/lib.rs:491-506` — existing governance scope
- **GOTCHA**: **Do NOT** register `emergency_remove_open_case` as a route. Task 4 is library-only per [IMPLEMENTATION-PLAN §3 Phase 4 task 46](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md).
- **GOTCHA**: Rate limiting — the admin endpoints inherit the parent scope's rate limit (`rate_limit.message()`). A tighter admin-specific rate limit is a v1 concern.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_routes --features full > .claude/build-phase4b-task05.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task05.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0
- **COMMIT**: `feat(api_routes): task 5 — register /governance/admin/* routes`

---

### Task 6: UPDATE `create_report.rs` threshold placeholder scaffolding (task 49)

- **ACTION**: Promote the existing `V0_THRESHOLD` / `V0_REPORTER_WEIGHT` constants into a small module-level section with a TODO marker that Phase 5 can locate and replace with the reputation-weighted formula per [99 OQ-006](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md)
- **IMPLEMENT**: In `crates/api/api_crud/src/governance/create_report.rs`, restructure the existing constants into a commented block:
  ```rust
  // =============================================================================
  // Threshold formula — v0 placeholder (OQ-006)
  //
  // Phase 5 replaces this with a reputation-weighted formula per [99 OQ-006].
  // The formula shape is intentionally undefined in v0 — the scaffolding here
  // exists so the replacement site is grep-discoverable and the commit that
  // lands the real formula is trivially reviewable.
  //
  // TODO(brehon-fork, phase-5): replace V0_THRESHOLD + V0_REPORTER_WEIGHT with
  // a reputation-weighted contribution function. See [99 OQ-006] and
  // docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md §3 Phase 5.
  // =============================================================================

  /// Threshold at which an `Open` case flips to `ThresholdMet`. v0 interim
  /// constant per [99 OQ-006].
  const V0_THRESHOLD: i64 = 3;

  /// Reporter weight for the threshold score. Stubbed at 1 per report in v0.
  /// Phase 5 tunes this by reporter reputation per [99 OQ-006].
  const V0_REPORTER_WEIGHT: i64 = 1;
  ```
- **MIRROR**: Existing constants at `crates/api/api_crud/src/governance/create_report.rs:55-60`
- **GOTCHA**: **No behaviour change.** This task is strictly documentation/scaffolding — the compiled output must be identical to Phase 4a. If `cargo check` changes any generated code, something's wrong.
- **GOTCHA**: The TODO marker `TODO(brehon-fork, phase-5)` makes the replacement site grep-able. Phase 5 starts by running `rg "TODO\(brehon-fork, phase-5\)"` to find every deferred v0 stub.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api_crud --features full > .claude/build-phase4b-task06.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task06.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0; no new warnings
- **COMMIT**: `docs(api_crud): task 6 — threshold formula placeholder scaffolding (OQ-006)`

---

### Task 7: CREATE `crates/server/src/governance.rs` composition root (task 47)

- **ACTION**: Create the governance composition root per [04 §12](docs/brehon-law-inspired-network/04-data-model-and-api.md) and [03 §7.3](docs/brehon-law-inspired-network/03-architecture.md) — **declarative only, NO business logic** per [03 §11](docs/brehon-law-inspired-network/03-architecture.md)
- **IMPLEMENT**:
  ```rust
  //! Governance composition root.
  //!
  //! Ties governance routes into the server binary and schedules the
  //! governance-relevant background jobs. Stays strictly declarative per
  //! [03 §11]. No business logic lives here; every ounce of decision-making
  //! is in `crates/api/api/src/governance/*` or `crates/api/api_crud/src/governance/*`.
  //!
  //! Phase 4b ships route wiring + **stub** background jobs. The real
  //! schedulers (snapshot, sanction cleanup, jury timeout) land in Phase 5/6
  //! per [IMPLEMENTATION-PLAN-v0.md §3 Phase 5/6].

  use lemmy_api_utils::context::LemmyContext;
  use std::sync::Arc;
  use tracing::info;

  /// Register governance-relevant background jobs.
  ///
  /// In Phase 4b these are all no-ops that log a message at startup so we
  /// can confirm the composition root is reached. Phase 5 replaces them.
  pub fn schedule_governance_jobs(_context: Arc<LemmyContext>) {
    info!("governance: background jobs not yet scheduled (Phase 4b stub — see Phase 5/6)");
  }

  // The actual route registration lives in `lemmy_api_routes::config` where
  // the `/governance` scope is already wired (Phase 4a + task 5 of this
  // phase). This module exposes a hook for the server binary to call at
  // startup so the governance plane has a declared integration point, per
  // [04 §12].
  ```
- **IMPLEMENT** (server/src/lib.rs update):
  Add `pub mod governance;` to `crates/server/src/lib.rs` (top of the file, alongside other module declarations if any exist). **If `crates/server/src/lib.rs` has no module declarations** (it looks like it uses a flat file structure), add the module and verify by running `cargo check -p lemmy_server`. If the file is purely an entry point, create a minimal declaration that reaches into the composition root without introducing business logic.
- **IMPLEMENT** (call site): At startup in `crates/server/src/lib.rs`'s main `start_lemmy_server`-equivalent function (identify by the `HttpServer::new` block near line 60+), add a single line: `governance::schedule_governance_jobs(context.clone());` **before** the `HttpServer::new` call. This is the declarative integration point — it runs once at startup and for now just logs.
- **MIRROR**: Other composition-root style modules in `crates/server/` (explore during implementation — the flat server/src layout is unusual); and `crates/routes/src/scheduled_tasks.rs` for scheduling conventions
- **GOTCHA**: [03 §11](docs/brehon-law-inspired-network/03-architecture.md) forbids business logic in `crates/server/`. The `schedule_governance_jobs` function must stay a thin declarative stub. **If the impl agent finds themselves writing more than ~20 lines of Rust here, STOP and surface to the decision queue.**
- **GOTCHA**: `context.clone()` on `Arc<LemmyContext>` is cheap — do not worry about clone cost.
- **GOTCHA**: Do NOT wire `lemmy_routes::utils::scheduled_tasks::start_scheduled_tasks` into governance in this task. That integration is Phase 5, when the real background jobs exist.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features full > .claude/build-phase4b-task07.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task07.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0
- **COMMIT**: `feat(server): task 7 — governance composition root (declarative)`

---

### Task 8: ADD ed25519 signing in `governance_log::append` (deferred from 4a)

- **ACTION**: Finish the signature step deferred from Phase 4a in `crates/api/api/src/governance/governance_log.rs` — replace the TODO at line 66 with actual ed25519 signing, and UPDATE the `signature` column through the signature-gate trigger
- **IMPLEMENT**: Replace the current Phase 4a-style `append` body (insert only, leave signature NULL) with the full insert + sign + update flow per the ED25519_SIGNING pattern in the Patterns section. The flow:
  1. INSERT the row (same as Phase 4a). The Postgres trigger computes `entry_hash`; `signature` is NULL.
  2. Load `GOVERNANCE_LOG_SIGNING_KEY` from env as hex-encoded 32-byte seed; parse via `hex::decode` + `SigningKey::from_bytes`.
  3. Sign `row.entry_hash` (the 32-byte SHA-256 digest computed by the trigger) — `signing_key.sign(&row.entry_hash)`.
  4. UPDATE the row: `diesel::update(governance_log::table.find(row.id)).set(governance_log::signature.eq(signature_bytes)).execute(conn).await?`. This UPDATE passes the signature-gate trigger because only `signature` changes and it transitions NULL → non-NULL.
  5. Return the row with the signature patched in-memory (or re-SELECT — either is acceptable; re-SELECT is simpler and matches the Lemmy style).

- **IMPLEMENT** (Cargo.toml update): **Neither `ed25519-dalek` nor `hex` is in the root workspace `Cargo.toml`** (verified at plan-review time — `grep -nE '^(ed25519-dalek|hex) ' Cargo.toml` returns nothing). Both MUST be added to `[workspace.dependencies]` in the **root** `Cargo.toml` first, then referenced from `crates/api/api/Cargo.toml`. Do this as a single task-8 sub-step in the following order:

  1. Edit **root** `Cargo.toml` `[workspace.dependencies]` block — add:
     ```toml
     ed25519-dalek = { version = "2", features = ["rand_core"] }
     hex = "0.4"
     ```
     The `rand_core` feature is required for key generation paths even if v0 uses a fixed env-var key; it matches the ed25519-dalek 2.x API the plan cites. Decision-queue entry #1 resolved the version.
  2. Edit `crates/api/api/Cargo.toml` `[dependencies]` block — add:
     ```toml
     ed25519-dalek = { workspace = true }
     hex = { workspace = true }
     ```
  3. Run `cargo check -p lemmy_api --features full` to force a Cargo.lock sync.
  4. Stage the root `Cargo.toml`, `crates/api/api/Cargo.toml`, **and `Cargo.lock`** together in the same commit per `feedback_commit_hygiene_lockfiles_and_task_labels.md`.

- **IMPLEMENT** (env var documentation): Add a line to `.env.example` (or the equivalent config template in the repo — locate via `rg -l 'POSTGRES_PASSWORD' -g '*.example'`):
  ```
  # 32-byte ed25519 signing seed, hex-encoded. REQUIRED for governance log signing.
  # Generate with: openssl rand -hex 32
  GOVERNANCE_LOG_SIGNING_KEY=
  ```

- **MIRROR**: Phase 4a `governance_log.rs:46-71` structure; ed25519-dalek 2.x API docs
- **GOTCHA**: **Signature-gate trigger at triggers.sql:800-827** forbids any column change except `signature`, and forbids `signature` transitioning non-NULL → anything. The update MUST change ONLY `signature`. A `diesel::update(table).set((signature.eq(X), other.eq(Y)))` would raise the trigger exception.
- **GOTCHA**: **`entry_hash` length** — the trigger computes SHA-256 → 32 bytes. `signing_key.sign(&row.entry_hash)` produces a 64-byte signature. Store as `Vec<u8>` (maps to Postgres `bytea`).
- **GOTCHA**: **Missing key = hard error.** Do NOT default to unsigned. If `GOVERNANCE_LOG_SIGNING_KEY` is absent or malformed, return `LemmyErrorType::Unknown("GOVERNANCE_LOG_SIGNING_KEY ...")` and fail the write. Silent fallback to unsigned defeats the purpose of the signature.
- **GOTCHA**: **Test key distribution.** The golden-path test in task 10 needs a deterministic key to verify signatures. Write the test to set the env var at test setup: `std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", "0000...0001")` (use a sentinel 32-byte value). Document the test key in the test's doc comment.
- **GOTCHA**: **TODO removal.** Remove the `// TODO(brehon-fork): Phase 4b — sign ...` comment at line 66 once signing is wired. A stale TODO post-implementation is a code-review smell.
- **GOTCHA**: **Return shape.** The existing `append` returns `LemmyResult<GovernanceLog>`. After signing, return the row with `signature: Some(signature_bytes)` populated — either by patching the in-memory struct or by re-SELECTing. Re-SELECT is more defensive because it confirms the row in the DB has the signature (catches bugs where the UPDATE silently affects 0 rows).
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_api --features full > .claude/build-phase4b-task08.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task08.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0
- **COMMIT**: `feat(api): task 8 — ed25519 signing in governance_log::append`

---

### Task 9: ADD dev-dependencies to `crates/server/Cargo.toml` for task 10

- **ACTION**: Add the dev-dependencies needed by `report_to_modlog_golden_path` — the server test harness needs test-only access to the handler crates + utilities
- **IMPLEMENT**: Edit `crates/server/Cargo.toml` `[dev-dependencies]` block (lines 52–67) to add:
  ```toml
  lemmy_api = { workspace = true, features = ["full"] }
  lemmy_api_crud = { workspace = true, features = ["full"] }
  lemmy_api_common = { workspace = true, features = ["full"] }
  lemmy_api_routes = { workspace = true }
  lemmy_api_utils = { workspace = true, features = ["full"] }
  actix-web = { workspace = true }
  ed25519-dalek = { workspace = true }
  hex = { workspace = true }
  uuid = { workspace = true, features = ["v4"] }
  ```
  Some of these crates already appear in `[dependencies]` (verified: `lemmy_api_routes:26`, `lemmy_api_utils:34`, `actix-web:39`); adding them to `[dev-dependencies]` with additional features like `"full"` is sometimes necessary if the test code path needs features not enabled for the production binary. Check each: if the production dep already has the feature set needed, the `[dev-dependencies]` entry is redundant and can be skipped. **The impl agent should iteratively add only what's needed** — try to compile task 10 with minimal dev-deps first, add one at a time until `cargo test --test e2e --no-run -p lemmy_server` resolves all imports.
- **IMPLEMENT** (Cargo.lock): The lockfile MUST be staged with the Cargo.toml edit per `feedback_commit_hygiene_lockfiles_and_task_labels.md`. After the edit, run `cargo check -p lemmy_server` to force a lockfile sync, then `git add Cargo.lock crates/server/Cargo.toml`.

- **MIRROR**: Existing `[dev-dependencies]` at `crates/server/Cargo.toml:52-67`
- **GOTCHA**: **Feature flag must match test code requirements.** `lemmy_api` probably needs `features = ["full"]` for test use because Diesel traits are gated behind `full`. If task 10 hits missing-trait errors, check the feature gate first.
- **GOTCHA**: **Cargo.lock — stage together.** Phase 1 had a bug where Cargo.lock changes were left unstaged and the next task saw an inconsistent build. Stage both files in the same commit.
- **GOTCHA**: **No new top-level workspace deps without approval.** If `ed25519-dalek` / `hex` / `uuid` are not already workspace dependencies, adding them requires a root `Cargo.toml` edit. Check first with `rg '^(ed25519-dalek|hex|uuid)' Cargo.toml` at the repo root. `ed25519-dalek` and `uuid` are almost certainly present given ADR-008 + Phase 4a `actor_pseudonym_helper` requirements; `hex` may not be. If missing, add to workspace.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/build-phase4b-task09.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-task09.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0 (test binary compiles, not yet running). If the test code (to be added in task 10) hasn't been written yet, this step just compiles the existing e2e.rs — confirm the existing tests still build.
- **COMMIT**: `chore(server): task 9 — dev-dependencies for golden-path e2e test`

---

### Task 10: WRITE golden-path e2e test `report_to_modlog_golden_path` (task 48)

- **ACTION**: Extend `crates/server/tests/e2e.rs` with the Phase 4 definition-of-done test. **Do NOT create a new test file** — the existing e2e.rs + `governance_fixtures` module is the canonical v0 harness per `project_phase_1_close.md`.
- **IMPLEMENT**: The test structure follows IMPLEMENTATION-PLAN §3 Phase 4 task 48, using **approach B** (admin backstop forces threshold crossing instead of filing four separate reports — makes the test deterministic and keeps OQ-006 deferred):

  1. **Setup** — reuse `governance_fixtures::start_postgres` and `governance_fixtures::apply_all_schema`. Set env `GOVERNANCE_LOG_SIGNING_KEY` to a deterministic 32-byte hex seed (e.g. all zeros except the last byte = 1). Build an `AsyncPgConnection`-backed `DbPool::Conn` per `feedback_async_pool_test_pattern.md`.
  2. **Seed** — insert:
     - 3 `LocalUser` rows: A (reporter), B (target), C (admin, `local_user.admin = true`)
     - 5 additional `LocalUser` rows D–H to serve as the eligible juror pool (so `admin_assign_jury` has ≥5 non-target/non-reporter candidates)
     - 1 `Community` with C as owner
     - 1 `Post` by B in the community
  3. **Mint JWTs** — use `lemmy_api_utils::claims::Claims::generate` to mint tokens for A and C.
  4. **Build an actix-web `App`** with the full `/api/v4` scope — reuse `lemmy_api_routes::config`. This gives the test access to **every** governance route without wiring each one individually.
  5. **Request 1: POST /governance/report** as A targeting B's post
     - Assert HTTP 200
     - Assert response `case_id.is_some()`; remember it as `case_id`
     - Assert response `threshold_met == false` (one report at weight 1 < threshold 3)
  6. **DB assertions after step 5**:
     - `moderation_case` has 1 row with `status = Open`, `threshold_score = 1`
     - `governance_log` has 1 row with `entry_kind = "report_created"` and non-NULL `signature`
  7. **Request 2: POST /governance/admin/assign-jury** as C (admin) — approach-B backstop
     - Assert HTTP 200
     - Assert response `assigned_person_ids.len() == 5`
     - Assert none of the five are A (reporter) or B (target)
  8. **DB assertions after step 7**:
     - `jury_assignment` has 5 rows, all with `status = Accepted`
     - `moderation_case.status = InPanel`
     - `governance_log` gained exactly 5 new rows with `entry_kind = "jury_assigned"` and 1 new row with `entry_kind = "panel_assembled"`, all signed. Use a per-`entry_kind` count query (see GOTCHA below) — do NOT assert a total row count at this intermediate step.
  9. **Requests 3–5: POST /governance/jury/vote** as three of the five assigned jurors with `JuryDecision::AdvisoryLabel`
     - Assert HTTP 200 for each
     - Assert the 3rd vote returns `case_decided: true, decision: Some(AdvisoryLabel)`
  10. **DB assertions after step 9**:
      - `jury_vote` has 3 rows
      - `moderation_case.status = Decided`, `decided_at` is non-NULL, `closed_at = decided_at + 7 days`
      - `sanction` has 1 row with `action = Label`
      - `public_case_log` has 1 row with the summary scrubbed of identifiers (assert no "@" or "http" substrings in `summary` and `rationale_redacted`)
      - `reputation_event` has 3 rows for the voting jurors, each with `dimension = JuryReliability` and `delta = 10`
      - `governance_log` gained rows for each vote (3 × `jury_vote_submitted`), the decision (1 × `case_decided`), the sanction creation (1 × `sanction_created`), the public log publication (1 × `public_log_published`), and 3 reputation events (3 × `reputation_event`). All signed. Assert per-`entry_kind` counts per the GOTCHA below.
  11. **Request 6: GET /governance/modlog?community_id=...** unauthenticated (no JWT)
      - Assert HTTP 200
      - Assert the response `Vec<GovernanceModlogView>` has exactly 1 entry matching `case_id`
  12. **Hash chain verification** — for every row in `governance_log` ordered by `id ASC`, recompute `digest(prev_hash || entry_kind || payload::text || to_char(created_at AT TIME ZONE 'UTC', ...))` and assert it equals the stored `entry_hash`. This confirms the trigger is wiring correctly end-to-end. The SQL for this assertion mirrors the trigger body at `triggers.sql:781-788`.
  13. **Signature verification** — for every row in `governance_log`, load the ed25519 `VerifyingKey` from the test public key (derived from the signing seed) and verify `signature.verify_strict(&entry_hash, ...)`. Assert every row verifies.

- **MIRROR**:
  - `crates/server/tests/e2e.rs:42-140` — `governance_fixtures` module + `can_insert_moderation_case` for the migration application pattern
  - `feedback_async_pool_test_pattern.md` — the `AsyncPgConnection::establish` + `DbPool::Conn` bridge for LemmyError types
  - `crates/api/api_utils/src/claims.rs:37-79` — `Claims::generate` for test JWT minting
  - `crates/diesel_utils/replaceable_schema/triggers.sql:781-788` — hash-chain digest expression (copy into test assertion)
  - Existing actix-web integration tests in the Lemmy workspace — search for patterns via `rg 'test::call_service' crates/`

- **GOTCHA**: **`--user $(id -u):$(id -g)`** — the existing `governance_fixtures::start_postgres` uses `testcontainers::GenericImage` which spawns the container under the host user automatically. This is fine — do NOT add the flag to the container config; it's an anti-pattern with `GenericImage`. The flag is only relevant for raw `docker run` commands.
- **GOTCHA**: **Reusable panel-select helper.** If task 4 factored panel selection into a private helper inside `admin_assign_jury.rs`, task 10 calls the HTTP route, not the helper. Keep the test at the HTTP layer — it's an integration test, not a unit test.
- **GOTCHA**: **Timing + `closed_at`.** The test asserts `closed_at == decided_at + 7 days` using chrono arithmetic. Use `.signed_duration_since()` to compare durations rather than exact timestamps (DB precision may differ from Rust precision by microseconds).
- **GOTCHA**: **Log row count — assert per entry_kind, not a single total.** Off-by-one bugs in a single total count are brittle (one extra emit slips past review; one missed emit fails the test cryptically). Instead, assert a per-`entry_kind` count map:

  ```rust
  let counts: Vec<(String, i64)> = governance_log::table
    .group_by(governance_log::entry_kind)
    .select((governance_log::entry_kind, diesel::dsl::count_star()))
    .load::<(String, i64)>(&mut conn)
    .await?;
  let map: HashMap<String, i64> = counts.into_iter().collect();

  assert_eq!(map.get("report_created"),         Some(&1));
  assert_eq!(map.get("jury_assigned"),          Some(&5));
  assert_eq!(map.get("panel_assembled"),        Some(&1));
  assert_eq!(map.get("jury_vote_submitted"),    Some(&3));
  assert_eq!(map.get("case_decided"),           Some(&1));
  assert_eq!(map.get("sanction_created"),       Some(&1));
  assert_eq!(map.get("public_log_published"),   Some(&1));
  assert_eq!(map.get("reputation_event"),       Some(&3));
  ```

  The hash-chain verification loop (step 12) iterates over every row regardless of `entry_kind`, so it does not depend on the total. The expected total at end of test is `1 + 5 + 1 + 3 + 1 + 1 + 1 + 3 = 16`, but **do NOT assert the total** — assert the per-kind counts only. If a handler later adds a new entry_kind (e.g. Phase 5), the per-kind assertions still pass for the kinds they name. Cross-reference the actual `entry_kind` strings against the handler implementations before finalising — if any entry_kind differs, update the asserted key (not the count).
- **GOTCHA**: **Test duration.** The full golden path takes 10–20 seconds mostly due to Postgres container startup. Mark the test with `#[tokio::test(flavor = "multi_thread")]` if needed; do NOT try to optimise by sharing a container across tests in Phase 4b.
- **GOTCHA**: **Per `.claude/rules/cargo-output-capture.md`** — never pipe cargo through tail without capturing exit code first.
- **GOTCHA**: **Per `.claude/rules/no-cargo-output-paste.md`** — do NOT paste the full test log into the conversation. Tail 20 lines from the captured file.
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server -- --test-threads=1 report_to_modlog_golden_path > .claude/build-phase4b-task10.log 2>&1"
  status=$?; tail -40 .claude/build-phase4b-task10.log; echo "exit: $status"
  ```
- **EXPECT**: exit 0; test passes; all 16 governance_log rows verified in hash chain + signature
- **COMMIT**: `test(e2e): task 10 — report_to_modlog_golden_path (Phase 4 DoD)`

---

### Task 11: FINAL WORKSPACE VALIDATION

- **ACTION**: Full `cargo check --workspace` + clippy + e2e, same gate as Phase 4a task 10
- **VALIDATE**:
  ```bash
  cmd //c "scripts\\brehon\\cargo-check.bat --workspace > .claude/build-phase4b-final-check.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-final-check.log; echo "exit: $status"

  # Clippy — use cargo-clippy.bat NOT cargo-check.bat per feedback_clippy_vs_check_wrapper.md
  cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --no-deps > .claude/build-phase4b-final-clippy.log 2>&1"
  status=$?; tail -20 .claude/build-phase4b-final-clippy.log; echo "exit: $status"

  cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > .claude/build-phase4b-final-e2e.log 2>&1"
  status=$?; tail -40 .claude/build-phase4b-final-e2e.log; echo "exit: $status"
  ```
- **EXPECT**: All three exit 0; no new clippy warnings; all existing tests + the new `report_to_modlog_golden_path` pass
- **COMMIT**: (no code commit; this is a gate) — if a fixup is needed, commit it as `fix(<scope>): task 11 — <one-line fix description>`

---

## Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only** for v0. Phase 4b completes the first runnable vertical slice.

### Tests to Add

| Test Name | What It Validates | Task |
|---|---|---|
| `report_to_modlog_golden_path` | Full v0 flow: report → admin-assign → 3 votes → Decided → sanction → public modlog → hash chain + signatures verified | 10 |

### Tests that MUST continue to pass (from earlier phases)

| Test Name | Phase | Why |
|---|---|---|
| `phase1_migrations_round_trip` | 1 | Migrations still reversible |
| `can_insert_moderation_case` | 1 | Basic schema still honors the trigger |
| `postgres_container_boots` | 0 | Harness still works |
| All `governance_case` / `jury_queue` / `governance_modlog` view crate tests | 2a+2b | Views still return expected shapes |
| All `redaction` unit tests | 4a | 5 passing tests still pass |

### Edge Cases (covered by task 10's assertions)

- [x] Hash-chain integrity holds across all 16 rows
- [x] `actor_pseudonym` used (not person_id) in every log entry
- [x] `EmergencyRemove` branch: exhaustive match compiles (covered by tasks 2 + 3 status matches); runtime path NOT exercised in 4b (the helper's runtime test is a Phase 5 concern)
- [x] Redaction strips identifiers from `public_case_log.summary` and `rationale_redacted` (asserted in step 10)
- [x] ed25519 signatures verify for every log row (asserted in step 13)
- [x] Duplicate vote by same juror — not re-tested (already covered by Phase 4a `submit_jury_vote` logic; integration test proves the path works for the first vote, and the handler's double-vote guard is exercised by the verification that only the first 3 votes are recorded)
- [ ] Vote on non-existent case — not in the golden path; Phase 5's negative tests cover this
- [ ] `admin_close_case` on already-closed case — not in the golden path; task 3 has the guard

---

## Validation Commands

Use these exact commands — do NOT substitute npm/pnpm/etc. This is a Rust project, validated on Windows per `feedback_cargo_invocations.md`.

### Level 1: STATIC_ANALYSIS (every task)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat -p <crate> --features full > /tmp/check.log 2>&1"
status=$?; tail -20 /tmp/check.log; echo "exit: $status"
```

**EXPECT**: Exit 0, zero errors

### Level 2: WORKSPACE_CHECK (task 11)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace > /tmp/check-ws.log 2>&1"
status=$?; tail -20 /tmp/check-ws.log; echo "exit: $status"
```

**EXPECT**: Exit 0

### Level 3: CLIPPY (task 11)

```bash
# IMPORTANT: use cargo-clippy.bat, NOT cargo-check.bat — per
# feedback_clippy_vs_check_wrapper.md, the two wrappers are distinct.
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --no-deps > /tmp/clippy.log 2>&1"
status=$?; tail -20 /tmp/clippy.log; echo "exit: $status"
```

**EXPECT**: Exit 0, no new warnings on governance code

### Level 4: INTEGRATION_TEST (task 10 + task 11)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server > /tmp/e2e.log 2>&1"
status=$?; tail -40 /tmp/e2e.log; echo "exit: $status"
```

**EXPECT**: All tests pass (including `report_to_modlog_golden_path`)

### Level 5: CROSS_CUTTING_VERIFICATION

Task 10's assertions cover this automatically. Manual spot-check can be run via psql after the test:

- [x] Every row in `governance_log` has non-NULL `signature` — `SELECT COUNT(*) FROM governance_log WHERE signature IS NULL;` → expect 0
- [x] Every row's `entry_hash` matches `digest(prev_hash || entry_kind || payload::text || created_at::text, 'sha256')`
- [x] No row has a raw `person_id` in its `payload` JSON (manual eyeball — should not pass `scrub_json`)

---

## Acceptance Criteria

- [ ] Task 0: pre-phase audit green across 5 probes
- [ ] Task 1: AdminAssignJury/AdminCloseCase DTO pairs compile in `lemmy_api_common` with `--features full`
- [ ] Task 2: `admin_assign_jury` handler compiles with exhaustive status match
- [ ] Task 3: `admin_close_case` handler compiles with exhaustive status match
- [ ] Task 4: `emergency_remove_open_case` helper compiles and carries the required ADR-013 doc comment ("jury cannot un-remove")
- [ ] Task 5: Routes `POST /governance/admin/assign-jury` and `POST /governance/admin/close-case` are registered
- [ ] Task 6: Threshold placeholder has `TODO(brehon-fork, phase-5)` marker at `crates/api/api_crud/src/governance/create_report.rs`
- [ ] Task 7: `crates/server/src/governance.rs` exists as a declarative composition root (≤30 LOC of actual code)
- [ ] Task 8: `governance_log::append` signs every row; the old Phase 4a TODO is removed; `GOVERNANCE_LOG_SIGNING_KEY` is a hard requirement
- [ ] Task 9: `crates/server/Cargo.toml` dev-deps enable the test harness; Cargo.lock staged in the same commit
- [ ] Task 10: `report_to_modlog_golden_path` passes end-to-end with 16 signed log rows + verified hash chain
- [ ] Task 11: `cargo check --workspace` + `cargo clippy --workspace --no-deps` + full e2e pass
- [ ] No contradictions with the 15 ADRs
- [ ] Every `CaseStatus` match across the new handlers is exhaustive (no `_ =>`)
- [ ] Every governance log write still calls `governance_log::append` (no raw Diesel inserts into `governance_log` anywhere)
- [ ] Every log write still uses `actor_pseudonym`, never a raw `person_id` / username / email
- [ ] No new upstream carry-patches introduced (check at task 11 by diffing `crates/api/` against upstream `811d0d09c`)

---

## Completion Checklist

- [ ] Task 0: pre-phase audit passes
- [ ] Task 1: DTOs added
- [ ] Task 2: `admin_assign_jury` handler compiles
- [ ] Task 3: `admin_close_case` handler compiles
- [ ] Task 4: `emergency_remove_open_case` helper compiles
- [ ] Task 5: Admin sub-scope route registered
- [ ] Task 6: Threshold scaffolding updated
- [ ] Task 7: Composition root wired
- [ ] Task 8: ed25519 signing active
- [ ] Task 9: Test dev-deps added
- [ ] Task 10: Golden-path e2e passes
- [ ] Task 11: Full workspace validation passes
- [ ] All acceptance criteria met
- [ ] Plan file moved to `.claude/PRPs/plans/completed/`
- [ ] Phase 4b completion report at `.claude/PRPs/reports/phase-4b-admin-backstops-and-golden-path-report.md`

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| ed25519-dalek / hex not in workspace deps | CERTAIN | LOW | Confirmed missing at plan-review. Task 8 adds both to root `[workspace.dependencies]` as a deterministic first sub-step (see task 8 IMPLEMENT block). |
| `random()` return type (Text) doesn't compose with Diesel `.order()` in juror selection | MED | MED | Task 2 first attempt uses `.order(random())` (existing SQL fn at `utils.rs:239` has `-> Text` signature). If the compiler rejects on "expected sortable" or similar, fall back to `.order(diesel::dsl::sql::<diesel::sql_types::Float>("random()"))` inline. Documented as GOTCHA in task 2. |
| Signature-gate trigger rejects the UPDATE due to unrelated column change | LOW | HIGH | Only `signature` in the `.set(...)` clause. Task 10 verifies via the assertion loop — a trigger violation surfaces as an obvious test failure. |
| Fewer than 5 eligible jurors in the test seed | MED | MED | Task 10 seeds 5 extra users (D–H). Document in the test's setup comment. If this class of error shows up in production, it's a Phase 5 concern (reputation-gated eligibility). |
| Golden-path test flakes due to Postgres container startup race | LOW | MED | `WaitFor::message_on_stderr("database system is ready to accept connections")` is already in `governance_fixtures::start_postgres`. No further mitigation needed. |
| Hash-chain verification loop is brittle to PG text rendering | MED | HIGH | `feedback_postgres_jsonb_canonicalization.md` is the known gotcha. Read the bytes back via raw SQL matching the trigger body at `triggers.sql:781-788`. Do NOT attempt to recompute the hash using serde_json's compact rendering. |
| `emergency_remove_open_case` stubbed remove step doesn't match Lemmy's canonical remove pathway | HIGH | LOW | Task 4 explicitly TODO-flags this. v0 acceptable because no HTTP route invokes the helper in production. Phase 5 wires it through `Post::update_removed` etc. |
| `large_futures` workspace lint fails on `admin_assign_jury` | LOW | MED | The handler factors state into a named `process_assignment` helper fn (same split as Phase 4a's `submit_jury_vote` → `process_vote`). Already under the lint threshold. |
| `crates/server/src/lib.rs` doesn't have an obvious module-declaration site | MED | LOW | Read the file top-to-bottom during task 7. If the module system is entirely flat (no existing `pub mod X` declarations), add the `pub mod governance;` at the very top under the `use` block. |
| Phase 4b reveals a hole in Phase 4a's `submit_jury_vote` under runtime conditions | MED | HIGH | The golden-path test is the first runtime exercise. If it fails on step 10's DB assertions, diagnose against `project_brehon_phase_2_complete.md` and `project_phase_4a_complete.md`; do NOT modify Phase 4a behaviour silently. Surface via decision queue. |
| Dev-dep feature-flag combination causes a compilation-cycle-style failure | MED | MED | Task 9 adds deps incrementally. If an edge case appears, bisect via `git stash` → partial uncomment → recompile. |

---

## Design Decisions

### Signing Wired In Phase 4b, Not Deferred Further

Phase 4a deferred ed25519 signing because the golden-path test didn't exist yet to exercise the sign + verify loop. Phase 4b is the correct landing zone because: (a) task 10 actively verifies every signature, catching regressions at CI; (b) the hash-chain trigger is stable (shipped in Phase 1); (c) the env-var key loading pattern is the ADR-008 v0 shape. Postponing further would leave the log indefinitely unsigned.

### Approach B for the Golden Path (admin backstop over four reports)

[IMPLEMENTATION-PLAN §3 Phase 4 task 48](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) offers two approaches. Approach B uses `POST /governance/admin/assign-jury` to force the case into a panel state; approach A files four reports from four users to cross the threshold. Approach B is the documented preference because it makes the test deterministic and leaves OQ-006 as a true placeholder (no runtime validation of the threshold formula in v0).

### Admin Close-Case is Single-Admin in v0

Per [99 ADR-010](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md), quorum + delay on `admin_close_case` is a v2 item. Task 3 documents this as an in-code comment and the task 10 test does NOT exercise it. Phase 5 revisits.

### EmergencyRemove is Library-Only in v0

Task 46's helper is callable but not routed. Per [IMPLEMENTATION-PLAN §3 Phase 4 task 46](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): "No HTTP route is required for v0 — the function is callable from a future emergency-remove route or from a direct admin tool." The golden-path test does NOT exercise it; an `EmergencyRemove`-focused test is a Phase 5 concern.

### Composition Root is Declarative-Only

[03 §11](docs/brehon-law-inspired-network/03-architecture.md) is clear: "No business logic in `crates/server/`." Task 7's `schedule_governance_jobs` is a no-op stub that logs at startup. The real scheduler plumbing lands in Phase 5 when the first actual background job (snapshot refresh, jury timeout reaper) exists.

### Eligible-Juror Pool Seeded at 5 Extras

The golden-path test seeds 3 named users (reporter A, target B, admin C) + 5 unnamed users (D–H) to ensure `admin_assign_jury`'s random selection has a non-empty candidate pool after excluding A and B. Smaller seed counts risk flakiness on the `ORDER BY random() LIMIT 5` selection if the person table is empty.

---

## Notes

- Phase 4b completes the Phase 4 deliverable. Phase 5 starts with the 6 remaining MVP endpoints (accept/decline jury, appeal, reputation/me, endorsement, list cases).
- The ed25519 signing step finally grounds ADR-008 in actual code. Previous phases produced hash-chained-but-unsigned rows; after task 8, every row is hash-chained AND signed with the test key in the env.
- The golden-path test is the v0 milestone. If it's green, Phase 4 is truly done and the vertical slice is proven.
- A Phase 4b completion report should be written to `.claude/PRPs/reports/phase-4b-admin-backstops-and-golden-path-report.md` summarising the 10-commit stack, the signature-verification results, and any carry-patch TODOs.
- Carry-patch TODOs (if any introduced): use `TODO(brehon-fork): upstream this to LemmyNet/lemmy — PR #___` per `feedback_carry_patch_todos.md`.
- **Context budget**: Phase 4b runs 11 commits (tasks 0–10 + task 11 gate); the per-task validation logs stay on disk per `no-cargo-output-paste.md`, so the ralph loop should stay well under the 200k-token danger zone.
