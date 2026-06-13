# Dockerfile for the Brehon Matrix application-service bridge.
#
# The bridge is a workspace-EXCLUDED standalone crate (services/bridge/),
# built with its own committed Cargo.lock. It does NOT build against the
# Lemmy workspace. Authored 2026-06-13 (pilot-internal); the M1-a stack
# (docker-compose.yml) noted this Dockerfile as deferred "Task 15".
#
# Build context MUST be services/bridge/ (not the repo root):
#   docker build -t brehon-bridge services/bridge/
#
# rusqlite "bundled" compiles SQLite from source (needs cc); matrix-sdk's
# default TLS pulls aws-lc-sys (needs cmake + clang + a C toolchain).

ARG RUST_VERSION=1.95

FROM rust:${RUST_VERSION}-bookworm AS builder
WORKDIR /build

# Build deps for rusqlite(bundled) + aws-lc-sys (matrix-sdk TLS backend).
RUN apt-get update && apt-get install -y --no-install-recommends \
    cmake clang libclang-dev pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests first for layer caching, then source.
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# --locked: fail if Cargo.lock is out of date (reproducible build).
RUN cargo build --release --locked && mv target/release/brehon-bridge /brehon-bridge

# Runner — slim Debian with CA certs + libssl for outbound HTTPS.
FROM debian:bookworm-slim AS runner
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder --chmod=0755 /brehon-bridge /usr/local/bin/brehon-bridge

LABEL org.opencontainers.image.title="brehon-bridge"
LABEL org.opencontainers.image.description="Brehon Matrix application-service bridge"
LABEL org.opencontainers.image.licenses="AGPL-3.0-or-later"

# Non-root runtime user + a writable data dir for the bridge_room SQLite DB
# (BRIDGE_DB_PATH defaults under /data). The dir must be owned by the runtime
# user BEFORE a named volume is mounted there, else SQLite "unable to open
# database file" — a fresh named volume inherits the mountpoint's ownership.
RUN useradd -m -u 1000 -s /bin/bash bridge \
    && mkdir -p /data \
    && chown bridge:bridge /data
USER bridge

ENTRYPOINT ["brehon-bridge"]
EXPOSE 8080
STOPSIGNAL SIGTERM
