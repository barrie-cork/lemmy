use std::sync::{
    atomic::{AtomicBool, AtomicI64, Ordering},
    Arc,
};

use anyhow::Result;

use crate::config::BridgeConfig;

/// Runs forever: polls Brehon's read endpoint for messaging_enabled + oq009_reveal_threshold.
/// Sets `relay_enabled` and `oq009_reveal_threshold` accordingly. Never panics; logs errors and continues.
pub async fn run_poller(
    config: Arc<BridgeConfig>,
    relay_enabled: Arc<AtomicBool>,
    oq009_reveal_threshold: Arc<AtomicI64>,
) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("build soft_pause HTTP client");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        match poll_once(&client, &config.brehon_read_url, &config.bridge_callback_secret).await {
            Ok((enabled, threshold)) => {
                let prev = relay_enabled.swap(enabled, Ordering::Relaxed);
                if prev != enabled {
                    tracing::info!(messaging_enabled = enabled, "soft-pause state changed");
                }
                oq009_reveal_threshold.store(threshold, Ordering::Relaxed);
            }
            Err(e) => {
                tracing::warn!(err = %e, "soft-pause poll failed; keeping current state");
            }
        }
    }
}

async fn poll_once(client: &reqwest::Client, url: &str, secret: &str) -> Result<(bool, i64)> {
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", secret))
        .send()
        .await?
        .error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    let enabled = body
        .get("messaging_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let threshold = body
        .get("oq009_reveal_threshold")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    Ok((enabled, threshold))
}
