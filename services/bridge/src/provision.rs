use anyhow::{Context, Result};

use crate::config::BridgeConfig;

/// Creates a Matrix community room via the AS master token.
/// Returns the newly created room ID.
///
/// M1: function module only — no HTTP route registration in this task.
/// Task 13 exposes this function through the admin router.
pub async fn create_community_room(config: &BridgeConfig, room_alias: &str) -> Result<String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .context("build HTTP client")?;
    let url = format!("{}/_matrix/client/v3/createRoom", config.tuwunel_url);
    let body = serde_json::json!({
        "room_alias_name": room_alias,
        "preset": "public_chat",
        "name": room_alias,
    });
    let resp = client
        .post(&url)
        .bearer_auth(&config.as_token)
        .json(&body)
        .send()
        .await
        .context("POST /_matrix/client/v3/createRoom failed")?
        .error_for_status()
        .context("createRoom returned non-2xx status")?;
    let json: serde_json::Value = resp.json().await.context("createRoom response is not JSON")?;
    json.get("room_id")
        .and_then(|v| v.as_str())
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("createRoom response missing room_id field"))
}
