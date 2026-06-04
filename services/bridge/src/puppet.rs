// Bridge-local puppet account map.
//
// Maintains a simple in-memory map: Brehon user-id → Matrix user-id.
// Ephemeral-to-M1: persisted only for process lifetime. M2 replaces
// this with B-actor-linked portable IDs (ADR-016).
//
// GOTCHA: the puppet namespace MUST match registration.yaml
// (Task 13 wires the actual file). For M1, use a fixed localpart
// prefix "_brehon_" in the AS user namespace. E.g.:
//   @_brehon_alice:tuwunel.local

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use matrix_sdk::Client;

use crate::config::BridgeConfig;

/// Brehon user-id (opaque string, e.g. username or UUID from Brehon PM).
pub type BrehonUserId = String;
/// Matrix user-id (fully qualified, e.g. @_brehon_alice:tuwunel.local).
pub type MatrixUserId = String;

/// Encode a Brehon user-id into a valid Matrix MXID localpart.
/// Matrix localparts allow [a-z0-9._\-=]; other bytes are hex-encoded as =XX.
fn localpart_escape(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            let lc = c.to_lowercase().next().unwrap_or(c);
            if lc.is_ascii_alphanumeric() || matches!(lc, '.' | '_' | '-') {
                vec![lc]
            } else {
                format!("={:02x}", lc as u32).chars().collect()
            }
        })
        .collect()
}

pub struct PuppetMap {
    inner: Mutex<HashMap<BrehonUserId, MatrixUserId>>,
    config: Arc<BridgeConfig>,
}

impl PuppetMap {
    pub fn new(config: Arc<BridgeConfig>) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(HashMap::new()),
            config,
        })
    }

    /// Return the Matrix user-id for `brehon_user`, creating a puppet if
    /// this is the first contact. Idempotent: repeated calls for the same
    /// user return the cached id without touching the homeserver.
    pub async fn ensure_puppet(&self, brehon_user: &str) -> Result<MatrixUserId> {
        // Fast-path: already in map.
        {
            let guard = self.inner.lock().unwrap();
            if let Some(mxid) = guard.get(brehon_user) {
                return Ok(mxid.clone());
            }
        }

        // Derive puppet localpart and mxid.
        let localpart = format!("_brehon_{}", localpart_escape(brehon_user));
        // The homeserver domain is extracted from tuwunel_url.
        // e.g. "http://localhost:8448" → "localhost:8448"
        let server_name = self
            .config
            .tuwunel_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .to_owned();
        let mxid = format!("@{localpart}:{server_name}");

        // Register/login the puppet via the AS admin token.
        // matrix-sdk Client with the AS token (not the puppet's own token —
        // for M1 we use the AS master token to register the puppet via the
        // /_matrix/client/v3/register?kind=guest endpoint or equivalent;
        // the exact API depends on Tuwunel version. For M1 the puppet
        // register call is best-effort: log and continue on error.
        let client = Client::builder()
            .homeserver_url(&self.config.tuwunel_url)
            .build()
            .await
            .context("build puppet matrix-sdk Client")?;

        // Attempt puppet registration. Tuwunel may reject duplicate usernames
        // with M_USER_IN_USE (puppet already exists) — treat as success.
        let register_result = client
            .matrix_auth()
            .register(
                matrix_sdk::ruma::api::client::account::register::v3::Request::new(),
            )
            .await;

        match register_result {
            Ok(_) => {
                tracing::info!(mxid = %mxid, "puppet registered");
            }
            Err(e) => {
                let err_str = e.to_string();
                if err_str.contains("M_USER_IN_USE") {
                    // M_USER_IN_USE → puppet already exists, not an error.
                    tracing::debug!(mxid = %mxid, "puppet already exists, reusing");
                } else {
                    return Err(anyhow::anyhow!("puppet registration failed: {}", e));
                }
            }
        }

        // Only cache on confirmed-exists or newly-created.
        self.inner
            .lock()
            .unwrap()
            .insert(brehon_user.to_owned(), mxid.clone());

        Ok(mxid)
    }
}
