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

        // Derive puppet localpart and mxid. The MXID domain is the Matrix
        // server_name (e.g. "localhost"), NOT the host:port of tuwunel_url —
        // those differ in containerised deploys (tuwunel_url is
        // http://brehon-tuwunel:8008 but server_name is localhost).
        let localpart = format!("_brehon_{}", localpart_escape(brehon_user));
        let server_name = &self.config.matrix_server_name;
        let mxid = format!("@{localpart}:{server_name}");

        // Register the puppet via the application-service flow. An AS must
        // register users in its namespace with auth type
        // `m.login.application_service` + the AS Bearer token; a bare register
        // triggers User-Interactive Auth (UIAA 401). matrix-sdk's register
        // helper does not expose the AS login type cleanly across versions, so
        // POST the AS register directly (mirrors provision.rs's raw reqwest).
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .build()
            .context("build puppet HTTP client")?;
        let url = format!(
            "{}/_matrix/client/v3/register",
            self.config.tuwunel_url
        );
        let body = serde_json::json!({
            "type": "m.login.application_service",
            "username": localpart,
        });
        let resp = client
            .post(&url)
            .bearer_auth(&self.config.as_token)
            .json(&body)
            .send()
            .await
            .context("POST /_matrix/client/v3/register (AS puppet) failed")?;

        if resp.status().is_success() {
            tracing::info!(mxid = %mxid, "puppet registered");
        } else {
            let status = resp.status();
            let body_text = resp.text().await.unwrap_or_else(|_| String::new());
            // M_USER_IN_USE → puppet already exists, idempotent reuse.
            if body_text.contains("M_USER_IN_USE") {
                tracing::debug!(mxid = %mxid, "puppet already exists, reusing");
            } else {
                return Err(anyhow::anyhow!(
                    "puppet registration failed: HTTP {status} {body_text}"
                ));
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
