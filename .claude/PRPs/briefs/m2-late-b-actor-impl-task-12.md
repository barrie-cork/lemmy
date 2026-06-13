---
role: impl-task
task_number: 12
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_bridge_validates_on_linux_not_windows.md
  - feedback_validate_pending_laptop_write_then_stop.md  # validate via cargo-linux.sh
---

# impl-task brief — m2-late-b-actor Task 12: CREATE bridge link handler + wire

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 12 of 13
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Task 11 merged (`app_actor_link.rs` must exist before this handler calls it).

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-12 bridge-link-handler axum-wiring — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-12.md
```

---

## 2. Scope

**Produce:**
- New file `services/bridge/src/link_handler.rs` — axum handler for `/brehon/link-claim`
- Edit `services/bridge/src/appservice.rs` — add route AFTER `.route_layer`
- Edit `services/bridge/src/config.rs` — add `brehon_signing_pubkey` + `brehon_link_confirm_url` + `bridge_signing_key` env vars
- Edit `services/bridge/src/main.rs` — add `mod link_handler;`
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Touch `crates/**` (Tasks 1-10)
- Touch `services/bridge/src/app_actor_link.rs` (Task 11)
- Run cargo yourself — Linux-only via cargo-linux.sh

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 12"
2. `services/bridge/src/sanction_handler.rs` lines 90-163 — **MIRROR**: bearer check pattern, `(StatusCode, Json).into_response()`, AppState access
3. `services/bridge/src/appservice.rs` lines 229-259 — **MIRROR**: router function; route AFTER `.route_layer`
4. `services/bridge/src/config.rs` — **MIRROR**: env-var field pattern; add 3 new fields
5. `services/bridge/src/main.rs` — **MIRROR**: `mod` declaration list
6. `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — **MANDATORY**
7. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Implementation

**New file: `services/bridge/src/link_handler.rs`**

The handler authenticates with `BRIDGE_CALLBACK_SECRET` (bearer, like `sanction_handler`), then:
1. Verifies Brehon's ed25519 signature on the claim bytes using `brehon_signing_pubkey`
2. Countersigns with the bridge's own ed25519 key (`bridge_signing_key`)
3. Calls `app_actor_link::upsert`
4. POSTs `LinkConfirmRequest` to `brehon_link_confirm_url` with `Bearer {bridge_callback_secret}`

Payload shape (incoming from Brehon's `/link` handler POST):
```rust
#[derive(Debug, Deserialize)]
pub struct LinkClaimPayload {
    pub brehon_actor_id: String,   // pseudonym UUID string (ADR-015)
    pub app_id: String,
    pub app_local_id: String,
    pub nonce: String,
    pub expires_at: String,
    pub brehon_signature: Vec<u8>, // ed25519 sig over claim_bytes
}
```

Claim bytes (reconstruct for verification — MUST match what Brehon signs in `link_actor`):
```
"{brehon_actor_id}\n{app_id}\n{app_local_id}\n{nonce}\n{expires_at}"
```

ed25519 verification (mirror `verify_strict` — must not use `verify`):
```rust
use ed25519_dalek::{VerifyingKey, Signature, Verifier};

let pubkey_bytes: [u8; 32] = hex::decode(&state.config.brehon_signing_pubkey)?
    .try_into()
    .map_err(|_| anyhow::anyhow!("brehon_signing_pubkey wrong length"))?;
let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes)?;
let signature = Signature::from_slice(&payload.brehon_signature)?;
verifying_key.verify_strict(&claim_bytes, &signature)?;
```

Countersigning (bridge signs the same claim_bytes with its own key):
```rust
use ed25519_dalek::{SigningKey, Signer};

let bridge_key_bytes: [u8; 32] = hex::decode(&state.config.bridge_signing_key)?
    .try_into()
    .map_err(|_| anyhow::anyhow!("bridge_signing_key wrong length"))?;
let bridge_signing_key = SigningKey::from_bytes(&bridge_key_bytes);
let bridge_signature = bridge_signing_key.sign(&claim_bytes).to_bytes().to_vec();
```

Then POST to `brehon_link_confirm_url` with `LinkConfirmRequest` JSON:
```json
{
  "brehon_actor_id": "<pseudonym UUID>",
  "app_id": "<app_id>",
  "app_local_id": "<app_local_id>",
  "nonce": "<nonce>",
  "expires_at": "<expires_at>",
  "brehon_signature": [/* bytes */],
  "app_signature": [/* bridge countersig bytes */]
}
```

Response shape from this handler (not from Brehon):
- 401 → bad or missing Bearer
- 400 → Brehon sig verification failed (do NOT call upsert)
- 500 → infra error (open DB, POST to Brehon failed)
- 200 `{"linked": true}` on success

**ADR-015 (load-bearing):** `brehon_actor_id` passed to `app_actor_link::upsert` is the pseudonym UUID string from the payload. NEVER a Lemmy `person_id`, username, or email. The Brehon binary ensures this upstream.

**Wire in `appservice.rs` (AFTER `.route_layer`):**

Read the router function. The new route goes AFTER the closing `.route_layer(...)` line and BEFORE the `.with_state(state)` call — matching the `/brehon/sanction-event` pattern:

```rust
.route("/brehon/link-claim", post(link_handler::handle_link_claim))
```

The import at the top of `appservice.rs` also needs `link_handler` added to the `use crate::{...}` block.

**Config additions (`config.rs`):**

Add three new required fields to `BridgeConfig`:
```rust
/// Ed25519 public key (32-byte hex) for verifying Brehon's link-claim signatures.
pub brehon_signing_pubkey: String,
/// Ed25519 private key seed (32-byte hex) for the bridge's countersignature.
pub bridge_signing_key: String,
/// URL to POST LinkConfirmRequest to (e.g. "http://localhost:8536/api/v4/governance/link/confirm").
pub brehon_link_confirm_url: String,
```

And in `from_env()`:
```rust
brehon_signing_pubkey: env::var("BREHON_SIGNING_PUBKEY")
    .context("BREHON_SIGNING_PUBKEY env var required")?,
bridge_signing_key: env::var("BRIDGE_SIGNING_KEY")
    .context("BRIDGE_SIGNING_KEY env var required")?,
brehon_link_confirm_url: env::var("BREHON_LINK_CONFIRM_URL")
    .context("BREHON_LINK_CONFIRM_URL env var required")?,
```

**Wire module in `main.rs`:**

Add `mod link_handler;` alphabetically after `mod config;` and before `mod provision;`.

**GOTCHA — route order:** The `/brehon/link-claim` route MUST be added AFTER `.route_layer(middleware::from_fn_with_state(state.clone(), hs_token_auth))` so it does NOT inherit the Matrix hs_token middleware. Self-auths via `BRIDGE_CALLBACK_SECRET` inside the handler itself.

**GOTCHA — ed25519_dalek version:** Check `services/bridge/Cargo.toml` for the already-pinned version (it's in use for `sanction_handler` countersig). Use the same import path, do NOT add a new dep.

**GOTCHA — Linux-only:** Same as Task 11. Do NOT run cargo locally.

### validate-pending-laptop (MANDATORY — Linux + clippy)

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": [
       "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
       "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml -- -D warnings"
     ],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 12
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**.

---

## 5. Commit

```
feat(bridge): link-claim handler handle_link_claim + route wiring (task 12)
```

Stage only: `services/bridge/src/link_handler.rs`, `services/bridge/src/appservice.rs`, `services/bridge/src/config.rs`, `services/bridge/src/main.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `services/bridge/src/link_handler.rs` created with `handle_link_claim` handler
- [ ] Handler checks `BRIDGE_CALLBACK_SECRET` bearer FIRST → 401 on mismatch
- [ ] Brehon ed25519 sig verified with `verify_strict` on reconstructed claim_bytes → 400 on failure, NO upsert
- [ ] Bridge countersigs with `bridge_signing_key` ed25519
- [ ] `app_actor_link::upsert(conn, app_local_id, brehon_actor_id)` called on verify success
- [ ] POSTs `LinkConfirmRequest` to `brehon_link_confirm_url` with bearer
- [ ] `/brehon/link-claim` route added AFTER `.route_layer` in `appservice.rs`
- [ ] `brehon_signing_pubkey`, `bridge_signing_key`, `brehon_link_confirm_url` fields in `config.rs`
- [ ] `mod link_handler;` in `main.rs`
- [ ] ADR-015: `brehon_actor_id` to upsert is the pseudonym UUID, asserted in a comment
- [ ] `validate-pending-laptop` DQ committed + pushed (both check + clippy commands)

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-12
  branch: phase-m2-late-b-actor
  filesModified:
    - services/bridge/src/link_handler.rs
    - services/bridge/src/appservice.rs
    - services/bridge/src/config.rs
    - services/bridge/src/main.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "Route added AFTER route_layer so hs_token middleware doesn't gate it"
    - "verify_strict (not verify) for Brehon sig — re-point defence"
    - "validate via cargo-linux.sh check + clippy (Docker rust:1.95)"
    - "brehon_actor_id stored/forwarded is pseudonym UUID (ADR-015)"
  notes: "Task 12 of 13. Task 13 (e2e tests, laptop-advisor validates) follows."
```
