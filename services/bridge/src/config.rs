use std::env;

use anyhow::{Context, Result};

/// Bridge daemon configuration loaded from environment variables.
///
/// All fields are required at startup. The laptop validate step (task 8
/// validate-pending-laptop) confirms this struct compiles correctly.
pub struct BridgeConfig {
    /// Base URL of the Tuwunel Matrix homeserver (e.g. "http://localhost:8448").
    pub tuwunel_url: String,
    /// Application-service token (the `as_token` from registration.yaml).
    pub as_token: String,
    /// Homeserver token (the `hs_token` from registration.yaml).
    pub hs_token: String,
    /// Port on which the bridge's own axum AS transaction server listens.
    pub bridge_port: u16,
    /// HTTP endpoint for reading Brehon governance state (read-only notify consumer).
    pub brehon_read_url: String,
    /// HTTP endpoint for Brehon Matrix-to-Brehon relay callbacks
    /// (the URL the bridge POSTs inbound Matrix DMs to).
    pub brehon_notify_url: String,
}

impl BridgeConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            tuwunel_url: env::var("TUWUNEL_URL")
                .context("TUWUNEL_URL env var required")?,
            as_token: env::var("AS_TOKEN")
                .context("AS_TOKEN env var required")?,
            hs_token: env::var("HS_TOKEN")
                .context("HS_TOKEN env var required")?,
            bridge_port: env::var("BRIDGE_PORT")
                .context("BRIDGE_PORT env var required")?
                .parse::<u16>()
                .context("BRIDGE_PORT must be a valid u16 port number")?,
            brehon_read_url: env::var("BREHON_READ_URL")
                .context("BREHON_READ_URL env var required")?,
            brehon_notify_url: env::var("BREHON_NOTIFY_URL")
                .context("BREHON_NOTIFY_URL env var required")?,
        })
    }
}
