# End-to-end tests

Integration tests for the Brehon governance fork live here. They run as:

```bash
cargo test -p lemmy_server --test e2e -- --test-threads=1
```

`--test-threads=1` is **required**. Several fixtures (`report_to_modlog_golden_path`,
`admin_config_fixtures::bootstrap`, etc.) mutate process-wide env vars
(`LEMMY_DATABASE_URL`, `GOVERNANCE_LOG_SIGNING_KEY`) inside `unsafe` blocks so
that `SETTINGS` (a `LazyLock`) resolves to the per-test Postgres container.
Parallel execution would race these writes across test boundaries. CI enforces
this via `.github/workflows/cargo-test-e2e.yml`; local runs must pass the flag
explicitly.

## Prerequisites

Docker must be running on the host. The harness uses
[`testcontainers-rs`](https://github.com/testcontainers/testcontainers-rs) to
boot an ephemeral `pgautoupgrade/pgautoupgrade:18-alpine` container per test
run, matching the Postgres image pinned in `docker/docker-compose.yml`.

The first run may take 60 - 120 seconds while Docker pulls the image.
Subsequent runs are fast.

## Running a single test

```bash
cargo test -p lemmy_server --test e2e <test_name> -- --test-threads=1
```

## Phase status

Phase 0 establishes the harness with one smoke test (`postgres_container_boots`)
that proves Docker is reachable and the container exposes a port. Later
phases (1+) will add real golden-path tests covering the governance endpoints
defined in `docs/brehon-law-inspired-network/04-data-model-and-api.md`.
