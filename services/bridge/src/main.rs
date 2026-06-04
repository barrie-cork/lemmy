// Bridge daemon entrypoint.
//
// Task 8: skeleton only — AS server and soft-pause poller are STUBS.
// Real logic wired in Tasks 9 (AS transaction server) and 12 (soft-pause).
//
// MIRROR: followed axum 0.8 minimal Router + tokio::main skeleton shape.
// No in-repo sibling; external example shape used per R8 /
// feedback_read_canonical_before_writing_spec.md (Ref MCP unavailable,
// relied on training knowledge of axum 0.8 minimal server pattern).

mod appservice;
mod config;

use anyhow::Result;
use appservice::AppState;
use config::BridgeConfig;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let config = BridgeConfig::from_env()?;
    let bridge_port = config.bridge_port;

    let app = appservice::router(Arc::new(AppState {
        config: Arc::new(config),
    }));

    let addr = SocketAddr::from(([0, 0, 0, 0], bridge_port));
    tracing::info!("bridge listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    let serve_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("axum serve failed");
    });

    // TODO(task 12): replace with real soft-pause poller (soft_pause.rs).
    let _poller = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            // soft-pause poll stub
        }
    });

    serve_handle.await?;
    Ok(())
}
