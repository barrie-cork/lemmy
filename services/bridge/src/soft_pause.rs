use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use anyhow::Result;

use crate::config::BridgeConfig;

/// Runs forever: polls Brehon's read endpoint for messaging_enabled.
/// Sets `relay_enabled` accordingly. Never panics; logs errors and continues.
pub async fn run_poller(config: Arc<BridgeConfig>, relay_enabled: Arc<AtomicBool>) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        match poll_once(&client, &config.brehon_read_url).await {
            Ok(enabled) => {
                let prev = relay_enabled.swap(enabled, Ordering::Relaxed);
                if prev != enabled {
                    tracing::info!(messaging_enabled = enabled, "soft-pause state changed");
                }
            }
            Err(e) => {
                tracing::warn!(err = %e, "soft-pause poll failed; keeping current state");
            }
        }
    }
}

async fn poll_once(client: &reqwest::Client, url: &str) -> Result<bool> {
    let resp = client.get(url).send().await?.error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    Ok(body
        .get("messaging_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false))
}
