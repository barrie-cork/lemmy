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
    /// URL for POST /governance/room-event (binary callback). Read from
    /// BREHON_ROOM_EVENT_URL but not yet consumed — the bridge currently
    /// receives room-events rather than POSTing them. Retained as config
    /// surface for a future bridge→binary room-event callback.
    #[allow(dead_code)]
    pub brehon_room_event_url: String,
    /// Bearer secret for bridge<->binary auth (BRIDGE_CALLBACK_SECRET).
    pub bridge_callback_secret: String,
    /// MXID for legal contact in emergency rooms.
    pub legal_contact_mxid: String,
    /// The Matrix homeserver's `server_name` (the MXID domain), e.g. "localhost".
    /// This is NOT the same as the host:port in `tuwunel_url` — puppet MXIDs and
    /// room IDs use this domain. Defaults to "localhost".
    pub matrix_server_name: String,
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
            brehon_room_event_url: std::env::var("BREHON_ROOM_EVENT_URL")
                .unwrap_or_else(|_| "http://localhost:8536/governance/room-event".to_string()),
            bridge_callback_secret: std::env::var("BRIDGE_CALLBACK_SECRET")
                .expect("BRIDGE_CALLBACK_SECRET must be set"),
            legal_contact_mxid: std::env::var("LEGAL_CONTACT_MXID")
                .unwrap_or_else(|_| "@legal:localhost".to_string()),
            matrix_server_name: std::env::var("MATRIX_SERVER_NAME")
                .unwrap_or_else(|_| "localhost".to_string()),
        })
    }
}
