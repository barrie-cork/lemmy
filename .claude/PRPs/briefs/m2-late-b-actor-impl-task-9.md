---
role: impl-task
task_number: 9
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G
  - feedback_governance_type_state_handlers.md           # handler-file touches governance/
  - feedback_cheap_model_arm_drops_adr_constraints.md    # ADR-008 + ADR-015 load-bearing
  - feedback_multi_write_handlers_need_transactions.md   # link_confirm has 2+ DB writes
---

# impl-task brief — m2-late-b-actor Task 9: DTOs + handlers `actor_app_link.rs`

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 9 of 13
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Tasks 1–7 merged (migration, schema, model, newtype, mod-export, consts+wrapper, shim).

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-9 dtos-handlers actor-app-link — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-9.md
```

---

## 2. Scope

**Produce:**
- 3 new DTOs in `crates/api/api_common/src/governance.rs`: `LinkActorRequest`, `LinkClaimPayload`, `LinkConfirmRequest`
- New file `crates/api/api/src/governance/actor_app_link.rs` with `link_actor`, `link_confirm`, `revoke_link` handlers
- `pub mod actor_app_link;` addition to `crates/api/api/src/governance/mod.rs` (alphabetical)
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Touch `crates/api/routes/src/lib.rs` (Task 10)
- Touch `services/bridge/**` (Tasks 11-12)
- Touch `crates/server/tests/e2e/**` (Task 13)
- Run cargo yourself

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 9" — full IMPLEMENT, MIRROR, and GOTCHA sections
2. `crates/api/api/src/governance/request_appeal.rs` lines 53-93 — **PRIMARY MIRROR**: authed handler shape (`check_local_user_valid`, `context.pool()`, `LemmyContext`)
3. `crates/api/api/src/governance/bridge_auth.rs` — **MIRROR**: `verify_bridge_secret(&req)` pattern
4. `crates/api/api/src/governance/room_event_handler.rs` lines 26-36 — **MIRROR**: bearer-authed handler shape
5. `crates/api/api/src/governance/sanction_publisher.rs` lines 139-208 — **MIRROR**: Bearer POST out + scrubbed append pattern
6. `crates/api/api/src/governance/actor_pseudonym_helper.rs` lines 41-80 — **MIRROR**: `get_or_create` + idempotent UniqueViolation insert pattern
7. `crates/api/api_common/src/governance.rs` lines 1-30 — the v0 endpoint-count carve-out comment block; new DTOs need entries here
8. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**
9. `.claude/lessons/feedback_cheap_model_arm_drops_adr_constraints.md` — **MANDATORY** (ADR-008 + ADR-015 load-bearing)
10. `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — **MANDATORY** (`link_confirm` has INSERT + governance_log::append)

---

## 4. Constraints

### ADR-015 — LOAD-BEARING (mandatory DoD line)

Every claim byte buffer and every governance_log payload MUST contain `actor_pseudonym` (the UUID string from `get_or_create`), **NEVER** `person.id`, username, or email.

`person.id` (PersonId) is only used as input to `get_or_create` — it MUST NOT appear in any claim or log payload.

**WHY this cannot be deferred:** a raw person_id in a cross-app payload defeats the pseudonymity the entire governance log depends on (ADR-015) and leaks Lemmy identity to every connected app permanently.

**DoD:** `grep -n 'person\.id\|local_user\.email\|\.name' crates/api/api/src/governance/actor_app_link.rs` shows `person.id` ONLY as input to `get_or_create`, never in a payload or claim bytes buffer.

### ADR-008 — LOAD-BEARING (mandatory DoD line)

`link_confirm` and `revoke_link` MUST call `governance_log::append(...)` before returning success.

**WHY this cannot be deferred:** ADR-008 — every write to mapping state is tamper-evidence-relevant. An unlogged link is an unauditable cross-app identity binding.

**DoD:** `grep -c 'governance_log::append' crates/api/api/src/governance/actor_app_link.rs` returns ≥ 2.

### Dual-signature — hard reject (ADR-016 OQ-ADR016-03c)

`link_confirm` MUST reject if EITHER the Brehon ed25519 signature (already in the claim payload received from the bridge) OR the app countersignature fails `ed25519_dalek::VerifyingKey::verify_strict`. A single-signed or unsigned claim is a hard reject, not a warning or degraded accept.

### Implementation

#### Step 1: Add 3 DTOs to `crates/api/api_common/src/governance.rs`

First, read the top of the file to understand the carve-out comment block (lines 1-20). Then add a carve-out entry for the new user-facing endpoints (LinkActor, RevokeLink) and add the 3 struct definitions after the existing DTOs.

**`LinkActorRequest`** — query params for GET `/link` (user-facing, JWT-authed):
```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct LinkActorRequest {
  pub app_id: String,
  pub app_local_id: String,
}
```

**`LinkClaimPayload`** — bridge-wire DTO (NOT ts-rs exported — Convention B, bridge-internal):
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkClaimPayload {
  pub brehon_actor_id: String,   // actor_pseudonym UUID — ADR-015
  pub app_id: String,
  pub app_local_id: String,
  pub nonce: String,             // UUID v4 single-use
  pub expires_at: DateTime<Utc>,
  pub brehon_signature: Vec<u8>, // ed25519 signature over the claim bytes
}
```

**`LinkConfirmRequest`** — POST `/link/confirm` from bridge (bearer-authed, NOT JWT):
```rust
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LinkConfirmRequest {
  pub nonce: String,
  pub app_signature: Vec<u8>,  // ed25519 countersignature by app
  pub app_local_id: String,
}
```

**GOTCHA — carve-out comment:** `LinkActorRequest` and a `RevokeLinkRequest` (implicit body) are new user-facing surfaces. Add them to the v0 endpoint-count carve-out comment at the top of the file:
```
//   - `LinkActorRequest`, `LinkConfirmRequest`, `LinkClaimPayload` — B-actor portable-ID
//     link endpoints (ADR-016). `LinkClaimPayload` is bridge-wire (Convention B, no ts-rs).
```

**GOTCHA — `DateTime<Utc>` import:** `chrono::{DateTime, Utc}` is already imported in the file; verify before adding.

#### Step 2: Create `crates/api/api/src/governance/actor_app_link.rs`

This is the new handler file. Mirror `request_appeal.rs` for authed handlers and `room_event_handler.rs` for bearer handler.

**Imports block:**
```rust
use crate::governance::{
  actor_pseudonym_helper,
  bridge_auth,
  governance_log,
};
use actix_web::{web::Data, web::Json, web::Query, HttpRequest, HttpResponse};
use chrono::{Duration, Utc};
use ed25519_dalek::VerifyingKey;
use lemmy_api_common::{
  context::LemmyContext,
  governance::{LinkActorRequest, LinkClaimPayload, LinkConfirmRequest},
  utils::check_local_user_valid,
};
use lemmy_db_schema::source::governance::{
  actor_app_link::{ActorAppLink, ActorAppLinkInsertForm},
  governance_log::{sign_link_claim, ENTRY_KIND_ACTOR_APP_LINK_CREATED, ENTRY_KIND_ACTOR_APP_LINK_REVOKED},
};
use lemmy_db_schema::source::governance::redaction::scrub_json;
use lemmy_db_schema_file::schema::actor_app_link;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use lemmy_db_utils::connection::get_conn;
use diesel_async::RunQueryDsl;
use diesel::prelude::*;
use serde_json::json;
use uuid::Uuid;
```

**`link_actor` handler (GET, JWT-authed):**

```rust
#[tracing::instrument(skip(context, local_user_view))]
pub async fn link_actor(
  data: Query<LinkActorRequest>,
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
) -> LemmyResult<HttpResponse> {
  check_local_user_valid(&local_user_view)?;

  let person_id = local_user_view.person.id;
  let pool = &mut context.pool();

  // Resolve or create pseudonym — person.id used ONLY here, never in payload
  let brehon_actor_id = actor_pseudonym_helper::get_or_create(pool, person_id).await?;

  // Build claim bytes: brehon_actor_id + app_id + app_local_id + nonce + expires_at
  let nonce = Uuid::new_v4().to_string();
  let expires_at = Utc::now() + Duration::minutes(5);
  let claim_bytes_str = format!(
    "{}\n{}\n{}\n{}\n{}",
    brehon_actor_id, data.app_id, data.app_local_id, nonce,
    expires_at.to_rfc3339()
  );
  let claim_bytes = claim_bytes_str.as_bytes();

  let brehon_signature = sign_link_claim(claim_bytes)?;

  let payload = LinkClaimPayload {
    brehon_actor_id: brehon_actor_id.clone(),
    app_id: data.app_id.clone(),
    app_local_id: data.app_local_id.clone(),
    nonce,
    expires_at,
    brehon_signature,
  };

  // POST claim to bridge callback — fire-and-forget, log-and-swallow on error (ADR-012)
  let bridge_url = std::env::var("BRIDGE_LINK_CLAIM_URL").unwrap_or_default();
  if !bridge_url.is_empty() {
    let bridge_secret = std::env::var("BRIDGE_CALLBACK_SECRET").unwrap_or_default();
    let client = context.client();
    let _ = client
      .post(&bridge_url)
      .header("Authorization", format!("Bearer {}", bridge_secret))
      .json(&payload)
      .send()
      .await
      .map_err(|e| tracing::warn!("link_actor bridge POST failed: {e}"));
  }

  Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}
```

**`link_confirm` handler (POST, bearer-authed from bridge):**

```rust
#[tracing::instrument(skip(context, req))]
pub async fn link_confirm(
  data: Json<LinkConfirmRequest>,
  context: Data<LemmyContext>,
  req: HttpRequest,
) -> LemmyResult<HttpResponse> {
  // Bearer auth FIRST
  bridge_auth::verify_bridge_secret(&req)?;

  // Reconstruct the claim bytes that Brehon originally signed
  // NOTE: claim bytes must be reconstructed or the nonce/payload re-retrieved
  // For v0: reconstruct from request fields (nonce must match one stored in a
  // short-lived nonce table or in-memory). For now: verify app countersig over
  // the data fields — implement nonce store as an in-memory LRU or small table.
  // MINIMUM: verify app_signature using the configured app pubkey.

  // Load app pubkey from env (hex-encoded ed25519 verifying key)
  let app_pubkey_hex = std::env::var("BRIDGE_LINK_APP_PUBKEY")
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidQuery))?;
  let app_pubkey_bytes = hex::decode(&app_pubkey_hex)
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidQuery))?;
  let app_pubkey_arr: [u8; 32] = app_pubkey_bytes.try_into()
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidQuery))?;
  let verifying_key = VerifyingKey::from_bytes(&app_pubkey_arr)
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidQuery))?;

  // Verify app signature over (nonce + app_local_id) — dual-signature defence
  let signed_bytes = format!("{}\n{}", data.nonce, data.app_local_id);
  let sig_bytes: [u8; 64] = data.app_signature.as_slice().try_into()
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidQuery))?;
  let app_sig = ed25519_dalek::Signature::from_bytes(&sig_bytes);
  verifying_key
    .verify_strict(signed_bytes.as_bytes(), &app_sig)
    .map_err(|_| LemmyError::from(LemmyErrorType::InvalidQuery))?;

  // TODO(brehon-fork): nonce expiry + single-use check (in-memory LRU table)
  // For now: signature verification is the gate; nonce replay is a v1 hardening
  // (see Task 13 test: link_confirm_rejects_bad_app_signature covers dual-sig)

  // Retrieve the pending link claim context from nonce storage (v0: env or request body)
  // The nonce ties to (brehon_actor_id, app_id, app_local_id) stored during link_actor
  // For v0: client sends all fields in the callback from bridge (bridge stores from claim)
  // REQUIRE bridge to send brehon_actor_id + app_id in the confirm callback body
  // (LinkConfirmRequest will be extended by bridge to carry these OR looked up by nonce)
  // PLACEHOLDER: read from a nonce store — for v0 use request body fields + nonce as key

  let pool = &mut context.pool();
  let conn = &mut get_conn(pool).await?;

  // We need brehon_actor_id and app_id — these come from the nonce store.
  // For v0: the bridge sends them in a separate field (extend LinkConfirmRequest).
  // The plan says to insert ActorAppLink here. We need brehon_actor_id as ActorPseudonymId.
  // APPROACH: parse them from extended confirm request or from a nonce table.

  // NOTE TO IMPL-TASK AGENT: Task 9 plan §IMPLEMENT says the bridge sends
  // (nonce, app_signature, app_local_id) and we look up the rest from a nonce store.
  // For v0, implement a simple in-memory HashMap<nonce, (brehon_actor_id_str, app_id)>
  // stored in LemmyContext or a task-local Arc<Mutex<HashMap>>. The link_actor handler
  // stores the nonce there on issue; link_confirm looks up and removes it (single-use).
  //
  // ALTERNATIVELY: extend LinkConfirmRequest with brehon_actor_id + app_id fields
  // (bridge echoes them from the claim) — simpler for v0, matches bridge capability.
  // READ the plan §IMPLEMENT carefully and pick the approach that compiles cleanly
  // against the existing LemmyContext and ActorAppLinkInsertForm types.

  Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}
```

**`revoke_link` handler (POST, JWT-authed):**

```rust
#[tracing::instrument(skip(context, local_user_view))]
pub async fn revoke_link(
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,
  // app_id + app_local_id as query params or JSON body — mirror LinkActorRequest shape
) -> LemmyResult<HttpResponse> {
  check_local_user_valid(&local_user_view)?;

  let person_id = local_user_view.person.id;
  let pool = &mut context.pool();

  // Resolve pseudonym — person.id used ONLY here, never in payload
  let brehon_actor_id_str = actor_pseudonym_helper::get_or_create(pool, person_id).await?;

  // UPDATE actor_app_link SET revoked_at = now() WHERE brehon_actor_id = ? AND app_id = ? AND revoked_at IS NULL
  let conn = &mut get_conn(pool).await?;

  // ... diesel UPDATE ...

  // ADR-008: log BEFORE return
  let scrubbed = scrub_json(json!({
    "brehon_actor_pseudonym": brehon_actor_id_str,
    // app_id from request
  }));
  governance_log::append(
    pool,
    ENTRY_KIND_ACTOR_APP_LINK_REVOKED,
    scrubbed,
    Some(brehon_actor_id_str),
  ).await?;

  Ok(HttpResponse::Ok().json(json!({ "ok": true })))
}
```

**IMPORTANT:** The above pseudocode/outline shows the SHAPE. Your job is to make it compile — read the MIRROR files for exact Diesel query syntax, exact LemmyContext method calls, exact `governance_log::append` signature, and exact `scrub_json` / `LocalUserView` usage. The MIRROR files are:
- `request_appeal.rs:53-93` — `check_local_user_valid` + `context.pool()` + `LocalUserView` import
- `sanction_publisher.rs:139-208` — `governance_log::append` call shape + scrub_json usage
- `actor_pseudonym_helper.rs:41-80` — `get_or_create` return type + UniqueViolation idempotent insert
- `room_event_handler.rs:26-36` — `verify_bridge_secret` + bearer pattern

**ADR-008 hard gate:** `link_confirm`'s INSERT into `actor_app_link` AND `governance_log::append(ENTRY_KIND_ACTOR_APP_LINK_CREATED, ...)` MUST both be inside a single `conn.run_transaction(...)` call (per `feedback_multi_write_handlers_need_transactions`).

#### Step 3: Add `pub mod actor_app_link;` to `crates/api/api/src/governance/mod.rs`

Insert alphabetically. The file currently has `pub mod accept_jury_assignment;` as the first entry. `actor_app_link` sorts before `actor_pseudonym_helper` (app < pse) and before `accept_jury_assignment` is debatable — check alphabetically: `accept` < `actor`, so `actor_app_link` sorts AFTER `accept_jury_assignment`. Insert between `accept_jury_assignment` and `actor_pseudonym_helper`.

Final order: `accept_jury_assignment`, `actor_app_link`, `actor_pseudonym_helper`, `admin_assign_jury`, ...

Also export the 3 handler functions so Task 10's route wiring can import them. Add to mod.rs (or to the handler file itself as `pub`):
```rust
pub use actor_app_link::{link_actor, link_confirm, revoke_link};
```

### validate-pending-laptop (MANDATORY)

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": ["cargo check -p lemmy_api_common", "cargo check -p lemmy_api"],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 9
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**.

---

## 5. Commit

```
feat(api): add DTOs + link_actor/link_confirm/revoke_link handlers (task 9)
```

Stage only:
- `crates/api/api_common/src/governance.rs`
- `crates/api/api/src/governance/actor_app_link.rs`
- `crates/api/api/src/governance/mod.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `LinkActorRequest`, `LinkClaimPayload`, `LinkConfirmRequest` structs present in `api_common/src/governance.rs`
- [ ] Carve-out comment entry added for new B-actor DTOs
- [ ] `link_actor`, `link_confirm`, `revoke_link` functions present in `actor_app_link.rs`
- [ ] `pub mod actor_app_link;` present in `mod.rs` alphabetically (after `accept_jury_assignment`, before `actor_pseudonym_helper`)
- [ ] ADR-015: `grep -n 'person\.id\|local_user\.email\|\.name' crates/api/api/src/governance/actor_app_link.rs` — `person.id` ONLY as input to `get_or_create`, NEVER in any payload/claim
- [ ] ADR-008: `grep -c 'governance_log::append' crates/api/api/src/governance/actor_app_link.rs` returns ≥ 2
- [ ] `link_confirm` calls `bridge_auth::verify_bridge_secret(&req)?` FIRST
- [ ] `link_confirm`'s INSERT + append are wrapped in `run_transaction`
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-9
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/api/api_common/src/governance.rs
    - crates/api/api/src/governance/actor_app_link.rs
    - crates/api/api/src/governance/mod.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "3 DTOs: LinkActorRequest (ts-rs), LinkClaimPayload (bridge-wire, no ts-rs), LinkConfirmRequest"
    - "link_confirm: bearer auth FIRST; dual-sig verify; INSERT + append in transaction"
    - "ADR-015: person.id only to get_or_create, never in claim/log payload"
    - "ADR-008: both link_confirm and revoke_link call governance_log::append before return"
  notes: "Task 9 of 13. Task 10 (route wiring) follows."
```
