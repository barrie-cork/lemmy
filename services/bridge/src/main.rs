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

mod app_actor_link;
mod appservice;
mod bridge_room;
mod config;
mod link_handler;
mod provision;
mod puppet;
mod relay;
mod room_provisioner;
mod sanction_handler;
mod soft_pause;

use anyhow::Result;
use appservice::AppState;
use config::BridgeConfig;
use puppet::PuppetMap;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, AtomicI64};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    // Install a tracing subscriber so the tracing::{info,warn,error}! calls
    // throughout the bridge actually emit. Honours RUST_LOG (e.g.
    // "info,brehon_bridge=debug"); falls back to "info" if unset. Without
    // this, every log call is a silent no-op (the bridge ran blind in the
    // pilot until this was added).
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let config = BridgeConfig::from_env()?;
    let bridge_port = config.bridge_port;
    let config_arc = Arc::new(config);
    let puppet_map = PuppetMap::new(Arc::clone(&config_arc));
    let relay_enabled = Arc::new(AtomicBool::new(true));
    let oq009_reveal_threshold = Arc::new(AtomicI64::new(1));

    let bridge_db_path = std::env::var("BRIDGE_DB_PATH")
        .unwrap_or_else(|_| "bridge.db".to_string());

    let app = appservice::router(Arc::new(AppState {
        config: Arc::clone(&config_arc),
        puppet_map,
        http_client: reqwest::Client::new(),
        relay_enabled: Arc::clone(&relay_enabled),
        oq009_reveal_threshold: Arc::clone(&oq009_reveal_threshold),
        bridge_db_path,
    }));

    let addr = SocketAddr::from(([0, 0, 0, 0], bridge_port));
    tracing::info!("bridge listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // cr-011: track poller JoinHandle so panics surface; use select! for
    // coordinated shutdown (either component exiting stops the other).
    let poller_handle = tokio::spawn(soft_pause::run_poller(
        Arc::clone(&config_arc),
        Arc::clone(&relay_enabled),
        Arc::clone(&oq009_reveal_threshold),
    ));

    tokio::select! {
        result = axum::serve(listener, app) => {
            if let Err(e) = result {
                tracing::error!("axum serve error: {e:#}");
            }
        }
        result = poller_handle => {
            match result {
                Ok(()) => tracing::info!("soft_pause poller exited cleanly"),
                Err(e) => tracing::error!("soft_pause poller panicked: {e}"),
            }
        }
    }
    Ok(())
}
