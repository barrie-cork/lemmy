// Bridge daemon entrypoint.
//
// Task 8: skeleton only — AS server and soft-pause poller are STUBS.
// Real logic wired in Tasks 9 (AS transaction server) and 12 (soft-pause).
//
// MIRROR: followed axum 0.8 minimal Router + tokio::main skeleton shape.
// No in-repo sibling; external example shape used per R8 /
// feedback_read_canonical_before_writing_spec.md (Ref MCP unavailable,
// relied on training knowledge of axum 0.8 minimal server pattern).

mod config;

use anyhow::Result;
use axum::Router;
use config::BridgeConfig;
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<()> {
    let config = BridgeConfig::from_env()?;

    // TODO(task 9): mount real AS transaction routes (appservice.rs).
    let app = Router::new();

    let addr = SocketAddr::from(([0, 0, 0, 0], config.bridge_port));
    tracing::info!("bridge listening on {addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;

    // TODO(task 9): replace with real axum serve once routes are wired.
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
