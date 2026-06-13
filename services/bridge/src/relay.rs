// 1:1 DM relay — both directions.
//
// Brehon→Matrix: a Brehon bridge-notify call arrives (from the
//   bridge_notify HTTP endpoint added in Tree B Task 6). The relay
//   resolves/creates a puppet via PuppetMap::ensure_puppet, opens a
//   DM room if necessary, and sends the message as the puppet.
//   M1: send_as_puppet is a stub; full implementation gated by Task 13.
//
// Matrix→Brehon: an inbound AS transaction (from Tuwunel) is handed
//   off here by handle_transactions in appservice.rs. The relay maps
//   each Matrix event back to a Brehon user ID and POSTs to the
//   Brehon notify callback URL (brehon_notify_url).
//
// M1 scope: text messages (m.text), image upload + m.image, voice
//   upload + m.audio. The <3s round-trip target is asserted in Task
//   13's integration test; this task is cargo-gated only.
//
// This whole module is the 1:1 DM-relay feature. It is intentionally not yet
// routed into (the pilot exercises governance room provisioning + sanctions,
// not 1:1 DM), so its items are unused at the crate level. Allow dead_code
// module-wide rather than per-item — these are a coherent pending feature, not
// stragglers. Remove the allow when DM relay is wired (the dm_round_trip test).
#![allow(dead_code)]

use std::sync::Arc;

use anyhow::{Context, Result};
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
        relay_matrix_event(event, config, http_client).await?;
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

    // Infer Brehon recipient from a puppet MXID in the event.
    // M1: best-effort — only hits when state_key is a puppet MXID.
    // Task 13 wires the full room-membership lookup.
    let brehon_recipient = match infer_brehon_recipient(event) {
        Some(r) => r,
        None => {
            tracing::debug!(room_id = %room_id, "cannot infer Brehon recipient, skipping");
            return Ok(());
        }
    };

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
        .timeout(std::time::Duration::from_secs(10))
        .json(&payload)
        .send()
        .await
        .context("POST Matrix event to brehon_notify_url")?
        .error_for_status()
        .context("brehon_notify_url returned non-2xx")?;

    Ok(())
}

/// Best-effort: extract the Brehon user-id from a puppet MXID in the event.
/// The puppet MXID format is @_brehon_{brehon_user}:{server_name}.
/// M1: only checks state_key. Task 13 replaces with room-membership lookup.
fn infer_brehon_recipient(event: &Value) -> Option<String> {
    let check = |s: &str| -> Option<String> {
        let localpart = s.strip_prefix('@')?.split(':').next()?;
        localpart.strip_prefix("_brehon_").map(str::to_owned)
    };

    if let Some(v) = event.get("state_key").and_then(|v| v.as_str()) {
        if let Some(b) = check(v) {
            return Some(b);
        }
    }

    None
}

/// Brehon→Matrix: send a DM from a Brehon user as their puppet to a
/// recipient puppet's DM room.
///
/// M1 stub: ensures puppets exist in the map, then returns a placeholder
/// room id. Full AS-client send is gated by Task 13's integration test.
pub async fn send_as_puppet(
    payload: &BridgeNotifyPayload,
    _config: Arc<BridgeConfig>,
    puppet_map: Arc<PuppetMap>,
) -> Result<String> {
    let sender_mxid = puppet_map.ensure_puppet(&payload.brehon_sender).await?;
    let _recipient_mxid = puppet_map.ensure_puppet(&payload.brehon_recipient).await?;

    tracing::info!(
        sender = %sender_mxid,
        "TODO(task 13): wire send_as_puppet via AS client"
    );

    Ok(String::from("stub-room"))
}
