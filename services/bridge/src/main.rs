// Bridge daemon entrypoint.
//
// Task 8: skeleton — AS server and soft-pause poller wired in Tasks 9/12.
// Task 12: soft_pause::run_poller replaces the TODO stub; relay_enabled
//   Arc<AtomicBool> wired into AppState so handle_transactions can gate relay.
//
// MIRROR: followed axum 0.8 minimal Router + tokio::main skeleton shape.
// No in-repo sibling; external example shape used per R8 /
// feedback_read_canonical_before_writing_spec.md (Ref MCP unavailable,
// relied on training knowledge of axum 0.8 minimal server pattern).

mod appservice;
mod config;
mod provision;
mod puppet;
mod relay;
mod soft_pause;

use anyhow::Result;
use appservice::AppState;
use config::BridgeConfig;
use puppet::PuppetMap;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = BridgeConfig::from_env()?;
    let bridge_port = config.bridge_port;
    let config_arc = Arc::new(config);
    let puppet_map = PuppetMap::new(Arc::clone(&config_arc));
    let relay_enabled = Arc::new(AtomicBool::new(true));

    let app = appservice::router(Arc::new(AppState {
        config: Arc::clone(&config_arc),
        puppet_map,
        http_client: reqwest::Client::new(),
        relay_enabled: Arc::clone(&relay_enabled),
    }));

    let addr = SocketAddr::from(([0, 0, 0, 0], bridge_port));
    tracing::info!("bridge listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // cr-011: track poller JoinHandle so panics surface; use select! for
    // coordinated shutdown (either component exiting stops the other).
    let poller_handle = tokio::spawn(soft_pause::run_poller(
        Arc::clone(&config_arc),
        Arc::clone(&relay_enabled),
    ));

    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                tracing::error!("axum serve error: {e:#}");
            }
        }
        result = poller_handle => {
            match result {
                Ok(Ok(())) => tracing::info!("soft_pause poller exited cleanly"),
                Ok(Err(e)) => tracing::error!("soft_pause poller error: {e:#}"),
                Err(e) => tracing::error!("soft_pause poller panicked: {e}"),
            }
        }
    }
    Ok(())
}
