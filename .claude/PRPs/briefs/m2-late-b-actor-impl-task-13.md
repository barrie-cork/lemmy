---
role: impl-task
task_number: 13
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_lemmy_error_no_std_error.md
  - feedback_async_pool_test_pattern.md
  - feedback_validate_pending_laptop_write_then_stop.md
---

# impl-task brief — m2-late-b-actor Task 13: e2e tests `actor_app_link.rs`

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 13 of 13
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Tasks 10-12 merged (routes + handlers must exist before e2e tests call them).

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-13 e2e-tests actor-app-link — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-13.md
```

---

## 2. Scope

**Produce:**
- New file `crates/server/tests/e2e/actor_app_link.rs` — 5 integration tests
- Edit `crates/server/tests/e2e.rs` — add `include!("e2e/actor_app_link.rs");` after line 153
- `validate-pending-laptop` DQ entry (commit + push + STOP)

**Do NOT:**
- Touch `crates/api/**`, `crates/db_schema/**`, or any file outside the two listed above
- Run `cargo test` yourself — validation is delegated to the laptop advisor
- Run `cargo-linux.sh` — these are workspace tests, not bridge tests

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 13" — 5 test specs verbatim
2. `crates/server/tests/e2e/m2_late.rs` — **MIRROR** (full file): boot sequence, `spawn_mock_subscriber`, `EnvVarGuard`, governance_log assert, `seed_person_local` helper, `SIGNING_SEED_HEX` constant, import block
3. `crates/server/tests/e2e.rs` lines 150-160 — **MIRROR**: the `include!` insertion point (after `include!("e2e/m2_late.rs");` on line 153)
4. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **MANDATORY**: tests return `LemmyResult<()>` with `?`, NOT `Result<(), Box<dyn Error>>`
5. `.claude/lessons/feedback_async_pool_test_pattern.md` — **MANDATORY**: `AsyncPgConnection::establish` + `DbPool::Conn`
6. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Structure

The test file follows the exact `m2_late.rs` pattern:

```rust
mod actor_app_link_fixtures {
  //! Process-env safety constraint: every test in this module mutates
  //! process-wide env vars. Safety contingent on --test-threads=1.
  use crate::common::governance_fixtures;
  // ... imports ...
  
  const SIGNING_SEED_HEX: &str =
    "0000000000000000000000000000000000000000000000000000000000000001";
  
  // APP test keypair — fixed seed for deterministic test countersignatures.
  // The test owns both halves of the dual signature.
  const APP_SIGNING_SEED_HEX: &str =
    "0101010101010101010101010101010101010101010101010101010101010101";

  async fn spawn_mock_subscriber() -> LemmyResult<(String, oneshot::Receiver<Vec<u8>>)> {
    // MIRROR m2_late.rs verbatim
  }
  
  // Helper: create a link via POST /api/v4/governance/link
  // Returns (brehon_actor_id: String, nonce: String, expires_at: String, brehon_signature: Vec<u8>)
  // by reading the payload captured from the mock subscriber.

  #[tokio::test(flavor = "current_thread")]
  pub async fn link_actor_creates_dual_signed_claim_and_logs() -> LemmyResult<()> { ... }
  
  #[tokio::test(flavor = "current_thread")]
  pub async fn link_confirm_rejects_bad_app_signature() -> LemmyResult<()> { ... }
  
  #[tokio::test(flavor = "current_thread")]
  pub async fn link_confirm_rejects_bad_bearer() -> LemmyResult<()> { ... }
  
  #[tokio::test(flavor = "current_thread")]
  pub async fn revoke_link_is_prospective_and_logged() -> LemmyResult<()> { ... }
  
  #[tokio::test(flavor = "current_thread")]
  pub async fn pseudonym_not_raw_identity_in_payload() -> LemmyResult<()> { ... }
}
```

### Imports needed (add to m2_late.rs MIRROR imports)

```rust
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use lemmy_api::governance::actor_app_link::{link_actor, link_confirm, revoke_link};
use lemmy_api_common::governance::{LinkActorRequest, LinkConfirmRequest};
use lemmy_db_schema_file::schema::{actor_app_link, governance_log};
use actix_web::web::Query;
use actix_web::test::TestRequest;
```

### Test specs

**Test 1: `link_actor_creates_dual_signed_claim_and_logs`**
- Boot env guards: `GOVERNANCE_LOG_SIGNING_KEY=SIGNING_SEED_HEX`, `BRIDGE_CALLBACK_SECRET=test-secret-actor-link`, `BREHON_BRIDGE_NOTIFY_URL=<mock_url>`
- Spawn mock subscriber, start Postgres, apply schema, build pool + context
- Seed one user (`actor_link_user`)
- Build `LocalUserView` for that user
- Call `link_actor(Query(LinkActorRequest { app_id: "matrix".into(), app_local_id: "@alice:test.invalid".into() }), context.clone(), local_user_view)` — this POSTs the claim to mock subscriber
- Wait (timeout 3s) for mock subscriber body
- Parse body as `serde_json::Value` (the `LinkClaimPayload`)
- Assert `brehon_actor_id` field is present and non-empty
- Assert `brehon_actor_id` != `"actor_link_user"` (pseudonym, ADR-015)
- Assert `brehon_signature` is present and non-empty
- Set `BRIDGE_LINK_APP_PUBKEY` env guard to pubkey derived from `APP_SIGNING_SEED_HEX`
- Countersign: `signed_bytes = format!("{}\n{}", nonce, app_local_id)`, sign with `APP_SIGNING_SEED_HEX` ed25519 key → `app_signature: Vec<u8>`
- Call `link_confirm(Json(LinkConfirmRequest { brehon_actor_id, app_id, app_local_id, nonce, app_signature }), context.clone(), req)` where `req` has `Authorization: Bearer test-secret-actor-link`
- Assert HTTP 200
- Assert one `actor_app_link` row in DB
- Assert one `actor_app_link_created` governance_log entry

**Test 2: `link_confirm_rejects_bad_app_signature`**
- Same boot as test 1
- Drive `link_actor` to get a valid Brehon-signed claim
- POST `/link/confirm` with a garbage `app_signature` (wrong bytes)
- Assert 4xx response
- Assert zero `actor_app_link` rows

**Test 3: `link_confirm_rejects_bad_bearer`**
- Same boot
- POST `/link/confirm` without `BRIDGE_CALLBACK_SECRET` (wrong or missing bearer)
- Assert 401
- Assert zero `actor_app_link` rows

**Test 4: `revoke_link_is_prospective_and_logged`**
- Boot + create a valid link (drive test 1 flow) with the same `LocalUserView`
- Call `revoke_link(Json(LinkActorRequest { app_id: "matrix".into(), app_local_id: "@alice:test.invalid".into() }), context.clone(), local_user_view)` — JWT-authed, no bearer needed
- Assert `actor_app_link` row has `revoked_at IS NOT NULL`
- Assert the `actor_app_link_created` entry is STILL present (chain not rewritten)
- Assert one new `actor_app_link_revoked` governance_log entry

**Test 5: `pseudonym_not_raw_identity_in_payload`**
- Boot + seed user named `"link_pseudonym_test_user"`
- Call `link_actor`, capture mock subscriber body
- Assert `brehon_actor_id` in payload != `"link_pseudonym_test_user"` (ADR-015)
- Parse `brehon_actor_id` as UUID (it must be a UUID string, not a username)

### GOTCHA — handler call signatures (READ `actor_app_link.rs` first)

Exact signatures (verified from merged code):

- `link_actor(data: Query<LinkActorRequest>, context: Data<LemmyContext>, local_user_view: LocalUserView)` — JWT-authed
- `link_confirm(data: Json<LinkConfirmRequest>, context: Data<LemmyContext>, req: HttpRequest)` — bearer-authed; `req` needs `Authorization: Bearer <secret>`; use `actix_web::test::TestRequest::default().insert_header(("Authorization", "Bearer test-secret-actor-link")).to_http_request()`
- `revoke_link(data: Json<LinkActorRequest>, context: Data<LemmyContext>, local_user_view: LocalUserView)` — JWT-authed; reuses `LinkActorRequest` (same `app_id` + `app_local_id` fields)

### GOTCHA — app countersignature bytes

`link_confirm` verifies the app countersig over `format!("{}\n{}", nonce, app_local_id)` — NOT the full claim bytes. The signed payload is just `nonce + "\n" + app_local_id`.

### GOTCHA — `BRIDGE_LINK_APP_PUBKEY` env var

`link_confirm` reads the app's ed25519 public key from `BRIDGE_LINK_APP_PUBKEY` (hex, 32 bytes). Set this to the pubkey derived from `APP_SIGNING_SEED_HEX` via `EnvVarGuard` in tests that call `link_confirm`.

Derive the pubkey in-test:
```rust
let app_signing_key = SigningKey::from_bytes(&hex::decode(APP_SIGNING_SEED_HEX).unwrap().try_into().unwrap());
let app_pubkey_hex = hex::encode(app_signing_key.verifying_key().to_bytes());
let _g_app_pubkey = crate::EnvVarGuard::set("BRIDGE_LINK_APP_PUBKEY", &app_pubkey_hex);
```

### GOTCHA — `BREHON_BRIDGE_NOTIFY_URL` env var

`link_actor` POSTs the claim to `BREHON_BRIDGE_NOTIFY_URL`. Set this to the mock subscriber URL via `EnvVarGuard`. Without it, the POST will fail or go to a wrong host.

### GOTCHA — `--test-threads=1`

These tests mutate process env vars. They MUST run single-threaded. The validate-pending-laptop command uses `cargo test --test e2e actor_app_link` which by default uses single-threaded for the `--test e2e` harness (the `nextest` or `cargo test` runner uses 1 thread per test binary; since all tests are in one binary this is fine).

### validate-pending-laptop (MANDATORY)

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": ["cargo test --test e2e actor_app_link"],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 13
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**. Do NOT run cargo test yourself.

---

## 5. Commit

```
feat(e2e): actor_app_link integration tests — 5 tests dual-sig link flow (task 13)
```

Stage only: `crates/server/tests/e2e/actor_app_link.rs`, `crates/server/tests/e2e.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `crates/server/tests/e2e/actor_app_link.rs` created with 5 tests
- [ ] `include!("e2e/actor_app_link.rs");` added in `e2e.rs` after line 153 (after `m2_late.rs` include)
- [ ] All 5 test fns are `#[tokio::test(flavor = "current_thread")] pub async fn ... -> LemmyResult<()>`
- [ ] `link_actor_creates_dual_signed_claim_and_logs`: asserts `actor_app_link` row + `actor_app_link_created` log entry
- [ ] `link_confirm_rejects_bad_app_signature`: asserts 4xx + zero rows
- [ ] `link_confirm_rejects_bad_bearer`: asserts 401 + zero rows
- [ ] `revoke_link_is_prospective_and_logged`: asserts `revoked_at IS NOT NULL` + both log entries present
- [ ] `pseudonym_not_raw_identity_in_payload`: asserts UUID string != username
- [ ] ADR-015 checked in tests 1 + 5: `brehon_actor_id` != seeded username
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-13
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/server/tests/e2e/actor_app_link.rs
    - crates/server/tests/e2e.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "APP_SIGNING_SEED_HEX fixed test keypair — test owns both halves of dual signature"
    - "BREHON_BRIDGE_NOTIFY_URL EnvVarGuard sets mock subscriber URL for link_actor POST"
    - "revoke_link test: asserts prospective revoke (revoked_at set, _created entry retained)"
    - "e2e tests run laptop-advisor-side (cargo test --test e2e actor_app_link)"
  notes: "Task 13 of 13 — final task. After e2e pass, advisor proceeds to bm-pr."
```
