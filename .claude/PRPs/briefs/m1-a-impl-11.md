# [role:impl-task] m1-a task-11 1:1 dm relay (text/image/voice) — see .claude/PRPs/briefs/m1-a-impl-11.md

## 1. Role + dispatch

`[role:impl-task] m1-a task-11 dm-relay — see .claude/PRPs/briefs/m1-a-impl-11.md`

Branch from: `phase-m1-a` (current tip `00d2653e4`)
Model: Sonnet 4.6

## 2. Scope

**Creates:**
- `services/bridge/src/relay.rs` — `handle_inbound` (Matrix→Brehon) + `send_as_puppet` (Brehon→Matrix text/image/voice)

**Modifies:**
- `services/bridge/src/appservice.rs` — replace the `TODO(task 11)` in `handle_transactions` with a call to `relay::handle_inbound`; add `brehon_notify_url: String` field to `AppState`; pass it through to the relay call
- `services/bridge/src/main.rs` — add `mod relay;`; read `BREHON_NOTIFY_URL` env var; pass it into `AppState`
- `services/bridge/src/config.rs` — add `pub brehon_notify_url: String` field (loaded from `BREHON_NOTIFY_URL` env var)

**Does NOT create or modify:**
- `services/bridge/src/puppet.rs` — Task 10 complete
- `services/bridge/src/provision.rs` — Task 12
- `services/bridge/src/soft_pause.rs` — Task 12
- Any file outside `services/bridge/`
- `Cargo.toml` (no new deps — `reqwest 0.12`, `matrix-sdk 0.18`, `serde_json` already present from Task 8)

**Commit subject (exact):**
```
feat(bridge): 1:1 DM relay text/image/voice both directions (task 11)
```

## 3. Required reading

- **R8 (mandatory):** This crate is NOT Lemmy-workspace. Use `anyhow::Result` not `LemmyResult`. No Diesel, no `--features full`, no `use lemmy_*`. External conventions only.
- `services/bridge/src/config.rs` — current fields (add `brehon_notify_url`).
- `services/bridge/src/appservice.rs` — read the full file: `AppState` struct, `handle_transactions` handler (the `TODO(task 11)` on line ~102), the auth middleware, and the router. You will modify `AppState` and `handle_transactions`.
- `services/bridge/src/puppet.rs` — read `ensure_puppet` signature: `pub async fn ensure_puppet(&self, brehon_user: &str) -> Result<MatrixUserId>`. The relay calls this.
- `services/bridge/src/main.rs` — current `mod appservice; mod config; mod puppet;` + `AppState` construction. You will add `mod relay;` and wire `BREHON_NOTIFY_URL`.
- **`m1.plan.md` §13 Task 11** — IMPLEMENT + GOTCHA sections.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ, commit, push, STOP. Do NOT run cargo on the daemon.

**NO Lemmy lessons apply.** Do not inject: `feedback_lemmy_error_no_std_error.md`, `feedback_features_full_workspace_only.md`, Diesel/migration lessons, `feedback_async_pool_test_pattern.md`. Tree A is a different toolchain (R8).

## 4. Constraints

### §4.1 What to implement

#### `services/bridge/src/config.rs` — add one field

Add `pub brehon_notify_url: String` to `BridgeConfig`:

```rust
pub struct BridgeConfig {
    pub tuwunel_url: String,
    pub as_token: String,
    pub hs_token: String,
    pub bridge_port: u16,
    pub brehon_read_url: String,
    /// HTTP endpoint for Brehon Matrix-to-Brehon relay callbacks
    /// (the URL the bridge POSTs inbound Matrix DMs to).
    pub brehon_notify_url: String,
}
```

In `from_env()` add:
```rust
brehon_notify_url: env::var("BREHON_NOTIFY_URL")
    .context("BREHON_NOTIFY_URL env var required")?,
```

#### `services/bridge/src/relay.rs` — new file

```rust
// 1:1 DM relay — both directions.
//
// Brehon→Matrix: a Brehon bridge-notify call arrives (from the
//   bridge_notify HTTP endpoint added in Tree B Task 6). The relay
//   resolves/creates a puppet via PuppetMap::ensure_puppet, opens a
//   DM room if necessary, and sends the message as the puppet.
//
// Matrix→Brehon: an inbound AS transaction (from Tuwunel) is handed
//   off here by handle_transactions in appservice.rs. The relay maps
//   each Matrix event back to a Brehon user ID and POSTs to the
//   Brehon notify callback URL (brehon_notify_url).
//
// M1 scope: text messages (m.text), image upload + m.image, voice
//   upload + m.audio. The <3s round-trip target is asserted in Task
//   13's integration test; this task is cargo-gated only.

use std::sync::Arc;

use anyhow::{Context, Result};
use matrix_sdk::{
    ruma::{
        api::client::room::create_room::v3::Request as CreateRoomRequest,
        events::room::message::{
            AudioInfo, AudioMessageEventContent, ImageInfo, ImageMessageEventContent,
            MessageType, RoomMessageEventContent, TextMessageEventContent,
        },
        OwnedRoomId, OwnedUserId,
    },
    Client,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{config::BridgeConfig, puppet::PuppetMap};

/// Payload of an inbound Brehon bridge-notify call (Brehon→Matrix direction).
/// Sent by Brehon Task 6 bridge_notify endpoint to the bridge.
#[derive(Debug, Deserialize)]
pub struct BridgeNotifyPayload {
    /// Brehon sender user-id (used to look up/create the puppet).
    pub brehon_sender: String,
    /// Brehon recipient user-id (used to look up/create the recipient puppet).
    pub brehon_recipient: String,
    /// Message content — one of text/image/voice.
    pub content: BridgeMessageContent,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum BridgeMessageContent {
    Text { body: String },
    Image { url: String, mimetype: String, size: Option<u64> },
    Voice { url: String, duration_ms: Option<u32> },
}

/// Outbound payload: bridge POSTs this to brehon_notify_url when a
/// Matrix DM arrives addressed to a Brehon-managed puppet.
#[derive(Debug, Serialize)]
pub struct MatrixToBrehonPayload {
    /// The Matrix sender MXID (e.g. @alice:homeserver.local).
    pub matrix_sender: String,
    /// The Brehon user-id inferred from the recipient puppet MXID.
    pub brehon_recipient: String,
    /// The room the event arrived in.
    pub room_id: String,
    /// Simplified event content.
    pub content: OutboundContent,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OutboundContent {
    Text { body: String },
    Image { url: String },
    Audio { url: String },
    Unknown { event_type: String },
}

/// Matrix→Brehon: process inbound AS transaction events and POST each
/// relevant DM back to brehon_notify_url.
///
/// Called from appservice.rs handle_transactions.
pub async fn handle_inbound(
    events: &[Value],
    config: &BridgeConfig,
    http_client: &reqwest::Client,
) -> Result<()> {
    for event in events {
        if let Err(e) = relay_matrix_event(event, config, http_client).await {
            // Log and continue — one failed relay must not drop the whole transaction.
            tracing::warn!(err = %e, "failed to relay Matrix event to Brehon");
        }
    }
    Ok(())
}

async fn relay_matrix_event(
    event: &Value,
    config: &BridgeConfig,
    http_client: &reqwest::Client,
) -> Result<()> {
    let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");

    // Only relay m.room.message events.
    if event_type != "m.room.message" {
        return Ok(());
    }

    let sender = event
        .get("sender")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();
    let room_id = event
        .get("room_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_owned();

    // Skip puppet-originated events to avoid relay loops.
    if sender.contains("_brehon_") {
        return Ok(());
    }

    // Infer Brehon recipient from the room membership (best-effort: use
    // the puppet member of this DM room whose localpart starts with _brehon_).
    // For M1 this is simplified: extract from the event's state_key or
    // the room alias. If we cannot determine it, skip.
    let brehon_recipient = infer_brehon_recipient(event);
    if brehon_recipient.is_none() {
        tracing::debug!(room_id = %room_id, "cannot infer Brehon recipient, skipping");
        return Ok(());
    }
    let brehon_recipient = brehon_recipient.unwrap();

    let content_value = event.get("content").cloned().unwrap_or(Value::Null);
    let msg_type = content_value
        .get("msgtype")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let outbound = match msg_type {
        "m.text" => OutboundContent::Text {
            body: content_value
                .get("body")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
        },
        "m.image" => OutboundContent::Image {
            url: content_value
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
        },
        "m.audio" => OutboundContent::Audio {
            url: content_value
                .get("url")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_owned(),
        },
        other => OutboundContent::Unknown {
            event_type: other.to_owned(),
        },
    };

    let payload = MatrixToBrehonPayload {
        matrix_sender: sender,
        brehon_recipient,
        room_id,
        content: outbound,
    };

    http_client
        .post(&config.brehon_notify_url)
        .json(&payload)
        .send()
        .await
        .context("POST Matrix event to brehon_notify_url")?
        .error_for_status()
        .context("brehon_notify_url returned non-2xx")?;

    Ok(())
}

/// Best-effort: extract the Brehon user-id from the puppet MXID in the
/// `unsigned.membership` or a `_brehon_` localpart in the room membership.
/// For M1 this uses a simple string extraction from the sender/recipient
/// localpart — M2 replaces with B-actor-linked lookup.
fn infer_brehon_recipient(event: &Value) -> Option<String> {
    // Look for a `_brehon_` localpart in the event's state or content.
    // The puppet MXID format is @_brehon_{brehon_user}:{server_name}.
    // For M1: best-effort scan of known fields.
    let check = |s: &str| -> Option<String> {
        // e.g. "@_brehon_alice:tuwunel.local" → Some("alice")
        let localpart = s.strip_prefix('@')?.split(':').next()?;
        localpart
            .strip_prefix("_brehon_")
            .map(str::to_owned)
    };

    // Check "state_key", "content.related_user_id", or "unsigned" fields.
    if let Some(v) = event.get("state_key").and_then(|v| v.as_str()) {
        if let Some(b) = check(v) { return Some(b); }
    }

    None // M1: cannot reliably infer from the transaction alone; skip
}

/// Brehon→Matrix: send a DM from a Brehon user as their puppet to a
/// recipient puppet's DM room. Creates the DM room on first use.
///
/// Called externally when the bridge receives a bridge_notify POST.
pub async fn send_as_puppet(
    payload: &BridgeNotifyPayload,
    config: Arc<BridgeConfig>,
    puppet_map: Arc<PuppetMap>,
) -> Result<String> {
    // Resolve puppets for sender and recipient.
    let sender_mxid = puppet_map.ensure_puppet(&payload.brehon_sender).await?;
    let _recipient_mxid = puppet_map.ensure_puppet(&payload.brehon_recipient).await?;

    // Build a matrix-sdk Client for the sender puppet using the AS master token.
    // M1: uses the AS token for all puppet actions (simplified — M2 will use
    // per-puppet tokens issued by the homeserver).
    let client = Client::builder()
        .homeserver_url(&config.tuwunel_url)
        .build()
        .await
        .context("build sender puppet matrix-sdk Client")?;

    // For M1: derive a stable room alias for this Brehon DM pair.
    // The alias is deterministic: #brehon_dm_{sender}_{recipient}:{server_name}
    let server_name = config
        .tuwunel_url
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .to_owned();
    let room_alias = format!(
        "#brehon_dm_{}_{}:{}",
        payload.brehon_sender, payload.brehon_recipient, server_name
    );

    // Attempt to join or create the DM room.
    // Best-effort room lookup via alias; create on 404.
    let room = match client.join_room_by_id_or_alias(
        matrix_sdk::ruma::RoomOrAliasId::parse(&room_alias)
            .context("parse DM room alias")?,
        &[],
    ).await {
        Ok(r) => r,
        Err(_) => {
            // Room doesn't exist yet — create it.
            let mut req = CreateRoomRequest::new();
            req.is_direct = true;
            let response = client
                .create_room(req)
                .await
                .context("create DM room")?;
            client
                .get_room(&response.room_id)
                .ok_or_else(|| anyhow::anyhow!("newly created room not found in client store"))?
        }
    };

    // Send the message.
    let msg_content = match &payload.content {
        BridgeMessageContent::Text { body } => {
            RoomMessageEventContent::new(MessageType::Text(
                TextMessageEventContent::plain(body.clone()),
            ))
        }
        BridgeMessageContent::Image { url, mimetype, size } => {
            let mut info = ImageInfo::new();
            info.mimetype = Some(mimetype.clone());
            info.size = size.map(|s| s.try_into().unwrap_or(matrix_sdk::ruma::UInt::MAX));
            RoomMessageEventContent::new(MessageType::Image(
                ImageMessageEventContent::new(
                    String::from("image"),
                    matrix_sdk::ruma::events::room::MediaSource::Plain(
                        matrix_sdk::ruma::OwnedMxcUri::try_from(url.as_str())
                            .context("parse image mxc URL")?,
                    ),
                ).info(Box::new(info)),
            ))
        }
        BridgeMessageContent::Voice { url, duration_ms } => {
            let mut info = AudioInfo::new();
            info.duration = duration_ms.map(|d| {
                matrix_sdk::ruma::UInt::try_from(d as u64).unwrap_or(matrix_sdk::ruma::UInt::MAX)
            });
            let mut audio_content = AudioMessageEventContent::new(
                String::from("voice note"),
                matrix_sdk::ruma::events::room::MediaSource::Plain(
                    matrix_sdk::ruma::OwnedMxcUri::try_from(url.as_str())
                        .context("parse audio mxc URL")?,
                ),
            ).info(Box::new(info));
            // Mark as voice note per MSC3245 (stable in Matrix 1.7 as m.voice).
            // The `voice` flag is an extra field in event content.
            // Simplified M1: use serde_json extra fields via message type extension.
            RoomMessageEventContent::new(MessageType::Audio(audio_content))
        }
    };

    room.send(msg_content).await.context("send message as puppet")?;

    tracing::info!(
        sender = %sender_mxid,
        room = %room.room_id(),
        "relayed Brehon→Matrix message"
    );

    Ok(room.room_id().to_string())
}
```

**IMPORTANT GOTCHAS for relay.rs:**

1. **matrix-sdk API shape** — the exact method names and struct shapes for `matrix-sdk 0.18` may differ from the above. Check what compiles. If `join_room_by_id_or_alias` doesn't exist in 0.18, use `client.resolve_room_alias(&alias_id).await` then `client.join_room_by_id(&room_id).await`. If `ImageInfo` or `AudioInfo` builder methods differ, use the struct's fields directly.

2. **Voice note MSC3245** — if `matrix-sdk 0.18` has no dedicated `VoiceMessageEventContent`, wrap the audio content with `serde_json::json!({ "m.voice": {} })` in an extra-field extension. For M1 simplicity, an `m.audio` without the voice flag is acceptable if the flag causes compile errors — the integration test (Task 13) asserts delivery not voice-flag presence.

3. **Puppet tokens** — M1 uses the AS master token for all puppet actions (simplified). `Client::builder().homeserver_url(...).build()` creates an unauthenticated client. For `room.send()` to work as a puppet, the client needs to be authenticated with the AS token OR use AS impersonation (`?user_id=@puppet:server`). M1: use AS master token + `user_id` query param. The matrix-sdk 0.18 `ClientBuilder` may have `.set_custom_auth_data()` or similar — use whatever API is available. If the appservice_url/registration-based AS client API exists in 0.18, use it; otherwise fall back to reqwest-direct for `/_matrix/client/v3/...?user_id=@puppet:server` calls. **Do not block on this** — if the AS client API is complex, simplify the Brehon→Matrix path to a best-effort stub that logs `TODO(task 13): wire puppet send via AS client` and returns the room id. The integration test gates this, not the cargo check.

4. **No new Cargo.toml deps** — `reqwest` is already present (matrix-sdk pulls it). Use `reqwest::Client` for the brehon_notify_url POST. DO NOT add reqwest again to Cargo.toml.

5. **`infer_brehon_recipient` M1 limitation** — the function is intentionally limited for M1. It returns `None` for most events (only hits if state_key is a puppet MXID). This is correct scope — the integration test in Task 13 will wire the full room-membership lookup. Do NOT over-engineer this for M1.

#### `services/bridge/src/appservice.rs` — wire relay

Replace the `TODO(task 11)` in `handle_transactions` with:

```rust
// Wire relay::handle_inbound (Task 11).
// Needs AppState access — handler must accept State<Arc<AppState>>.
```

**You must also add `State(state): State<Arc<AppState>>` to `handle_transactions`** — the current handler doesn't extract `AppState`. Also add a `reqwest::Client` to `AppState` so it can be shared across requests:

```rust
pub struct AppState {
    pub config: Arc<BridgeConfig>,
    pub puppet_map: Arc<crate::puppet::PuppetMap>,
    pub http_client: reqwest::Client,  // Task 11: shared reqwest client for relay posts
}
```

Updated `handle_transactions`:

```rust
async fn handle_transactions(
    State(state): State<Arc<AppState>>,
    Path(txn_id): Path<String>,
    Json(body): Json<PushEventsBody>,
) -> impl IntoResponse {
    tracing::info!(
        txn_id = %txn_id,
        event_count = body.events.len(),
        "received AS transaction"
    );
    if let Err(e) = relay::handle_inbound(
        &body.events,
        &state.config,
        &state.http_client,
    ).await {
        tracing::warn!(err = %e, "relay::handle_inbound error");
    }
    (StatusCode::OK, Json(serde_json::json!({})))
}
```

Add `use crate::relay;` to `appservice.rs` imports.

#### `services/bridge/src/main.rs` — add mod relay + http_client

```rust
mod relay;
// ... after existing mods
```

In `main()`, construct `AppState` with `http_client: reqwest::Client::new()`:

```rust
let app = appservice::router(Arc::new(AppState {
    config: config_arc,
    puppet_map,
    http_client: reqwest::Client::new(),
}));
```

No new env vars read in `main.rs` — config already has `brehon_notify_url` via `BridgeConfig`.

### §4.2 M1 simplification notes

- **Brehon→Matrix route** (`send_as_puppet`): For M1, if the AS client setup is complex, it is acceptable to stub this as `tracing::info!("TODO: send_as_puppet via AS client"); Ok(String::from("stub-room"))`. The cargo check gates compile; Task 13's docker-compose integration test gates send behaviour.

- **Matrix→Brehon route** (`handle_inbound`): similarly, the `infer_brehon_recipient` function returning `None` for most events is correct M1 scope.

- **`reqwest::Client` in `AppState`**: this is a shared client (connection pool reuse). Construct once in `main()`, pass in.

### §4.3 validate-pending-laptop DQ (MANDATORY — write-then-stop)

After committing, write a `kind: "validate-pending-laptop"` entry to `.claude/decision-queue.json`, commit+push it, then **STOP**. Do NOT run `cargo check` yourself.

DQ entry shape:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<current worker branch>",
  "phase_task": 11,
  "commands": ["cd services/bridge && cargo check"],
  "question": "Tree-A Task 11: 1:1 DM relay compiles inside services/bridge — laptop runs cargo check.",
  "options": ["pass", "fail"],
  "context": "Task 11 created services/bridge/src/relay.rs (handle_inbound + send_as_puppet), added mod relay to main.rs, added reqwest::Client to AppState, wired relay::handle_inbound into handle_transactions. VALIDATE: cd services/bridge && cargo check must be GREEN (exit 0). NOT --workspace. NOT --features full.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "id": "<generate via bash scripts/brehon/dq-v3-new-entry.sh>"
}
```

Generate id: `bash scripts/brehon/dq-v3-new-entry.sh`
Append via: `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`
Commit: `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised DQ <id> — task-11 validate-pending-laptop"`
Push to `origin <current-branch>`. Then STOP — do not run cargo check.

### §4.4 Attribution

Do NOT write `answered_by: "advisor"` in any DQ entry. You are `impl`.
Do NOT commit to `governance-v0` or `main`.
Commit only on the current worker branch.

## 5. Expected outputs

After this task:
- `services/bridge/src/relay.rs` exists with `handle_inbound` and `send_as_puppet`
- `services/bridge/src/config.rs` has `brehon_notify_url: String` field
- `services/bridge/src/appservice.rs` has `http_client: reqwest::Client` in `AppState`, `relay::handle_inbound` called in `handle_transactions`
- `services/bridge/src/main.rs` has `mod relay;` and constructs `AppState` with `http_client`
- A `validate-pending-laptop` DQ entry is in `pending[]` with `phase_task: 11`
- Worker branch pushed to origin
