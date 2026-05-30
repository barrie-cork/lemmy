#!/usr/bin/env bash
# Brehon dev utility: run cargo INSIDE a Linux container that mirrors CI.
#
# This is the free, local replacement for GH Actions Shape G's Linux
# compile proof. The container reproduces .github/workflows/cargo-validate-
# workspace.yml byte-for-byte: rust:1.95 (pinned in rust-toolchain.toml) +
# `libpq-dev pkg-config`, then runs whatever cargo subcommand + flags the
# caller passes. A green run here is the same evidence a green Shape-G run
# would have been — produced on the laptop's Docker Desktop (linux/x86_64)
# for zero Actions minutes.
#
# Why this exists: validation model finalised 2026-05-30 (see
# project_laptop_canonical_cargo_runner memory). Laptop-native cargo proves
# the WINDOWS build; this proves the LINUX deploy-target build. Run it
# before merge, or on any diff that touches dependencies / migrations /
# cfg(target_os) where Windows-green != Linux-green.
#
# Usage (from any shell, any cwd inside the repo):
#     scripts/brehon/cargo-linux.sh check  --workspace --features full
#     scripts/brehon/cargo-linux.sh clippy --workspace --features full --no-deps -- -D warnings
#     scripts/brehon/cargo-linux.sh test   --no-run -p lemmy_server --test e2e
#
# The FIRST argument is the cargo subcommand (check / clippy / test / ...);
# everything after is passed through verbatim. Per
# .claude/lessons/feedback_wrapper_script_flag_silence.md, this wrapper does
# NOT hardcode scope — the caller controls `--workspace` / `-p <crate>` /
# `--features full` / `-- -D warnings`. Hardcoding any of them here would
# silently discard the caller's flags, the exact bug that lesson names.
#
# Caching: a named Docker volume `brehon-cargo-registry` persists the cargo
# registry + git caches across runs. The Linux build artifacts go in a
# host-side `target-linux/` dir (gitignored), kept SEPARATE from the Windows
# `target/` so the two toolchains never clobber each other's incremental
# state. First run is cold (full compile + crate download); subsequent runs
# are warm.
#
# Requires: Docker Desktop running with linux/x86_64 containers. Verify with
#     docker info --format '{{.OSType}}'   # must print: linux
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Pin the image to the toolchain rust-toolchain.toml declares, so a channel
# bump there is reflected here without a code edit. Falls back to a literal
# if the parse fails (defensive — the file format is stable).
TOOLCHAIN_CHANNEL="$(sed -n 's/^channel = "\(.*\)"/\1/p' "$REPO_ROOT/rust-toolchain.toml" 2>/dev/null || true)"
RUST_IMAGE="rust:${TOOLCHAIN_CHANNEL:-1.95}"

if [ "$#" -lt 1 ]; then
  echo "ERROR: first argument must be the cargo subcommand (check / clippy / test / ...)" >&2
  echo "Usage: scripts/brehon/cargo-linux.sh <subcommand> [flags...]" >&2
  exit 2
fi

# Preflight: Docker must be up and serving Linux containers. Fail loud, not
# silent — per feedback_ci_silent_failure_pattern, a missing prerequisite
# should be an obvious non-zero exit, never a misleading pass.
if ! docker info --format '{{.OSType}}' >/tmp/brehon-docker-ostype 2>/dev/null; then
  echo "ERROR: Docker is not running or not reachable. Start Docker Desktop." >&2
  exit 3
fi
if [ "$(cat /tmp/brehon-docker-ostype)" != "linux" ]; then
  echo "ERROR: Docker is in $(cat /tmp/brehon-docker-ostype) mode; switch to Linux containers." >&2
  exit 3
fi

echo "LINUX_CONTAINER_OK image=$RUST_IMAGE"
echo "repo=$REPO_ROOT  target=target-linux/  cache-volume=brehon-cargo-registry"
echo "cargo $*"
echo "---"

# On Windows/Git-Bash, -v needs a Windows-style host path; MSYS_NO_PATHCONV
# stops MSYS rewriting the in-container paths (/work, /usr/local/cargo).
# CARGO_TARGET_DIR redirects build output to the Linux-only target dir.
# The container installs the same system deps CI installs, then execs cargo.
MSYS_NO_PATHCONV=1 exec docker run --rm \
  -v "$REPO_ROOT":/work \
  -v brehon-cargo-registry:/usr/local/cargo/registry \
  -w /work \
  -e CARGO_TARGET_DIR=/work/target-linux \
  -e CARGO_TERM_COLOR=never \
  "$RUST_IMAGE" \
  bash -c "set -euo pipefail
    apt-get update -qq
    DEBIAN_FRONTEND=noninteractive apt-get install -y -qq libpq-dev pkg-config >/dev/null
    rustup component add clippy >/dev/null 2>&1 || true
    exec cargo $*"
