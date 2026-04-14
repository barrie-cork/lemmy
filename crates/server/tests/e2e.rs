//! End-to-end test harness for the Brehon governance fork.
//!
//! This file is the entry point for `cargo test --test e2e`. Phase 0
//! establishes the harness; later phases add real golden-path tests
//! that exercise the governance endpoints against a real Postgres.
//!
//! The harness uses `GenericImage` (not `testcontainers_modules::Postgres`)
//! so the image coordinates exactly match Lemmy's production
//! `docker-compose.yml`: `pgautoupgrade/pgautoupgrade:18-alpine`.

use std::error::Error;
use testcontainers::{
  GenericImage, ImageExt,
  core::{IntoContainerPort, WaitFor},
  runners::AsyncRunner,
};

#[tokio::test]
async fn postgres_container_boots() -> Result<(), Box<dyn Error>> {
  let container = GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")
    .with_exposed_port(5432.tcp())
    .with_wait_for(WaitFor::message_on_stderr(
      "database system is ready to accept connections",
    ))
    .with_env_var("POSTGRES_USER", "lemmy")
    .with_env_var("POSTGRES_PASSWORD", "password")
    .with_env_var("POSTGRES_DB", "lemmy")
    .start()
    .await?;

  let host_port = container.get_host_port_ipv4(5432).await?;

  assert!(host_port > 0, "postgres mapped port should be non-zero");

  Ok(())
}
