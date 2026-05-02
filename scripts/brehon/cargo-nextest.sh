#!/usr/bin/env bash
# Brehon dev utility: run `cargo nextest run` on macOS / Linux / Git Bash
# with the same libpq env as cargo-test.sh.
#
# Per Perplexity research 2026-05-02: nextest's process-per-test model
# resolves the LazyLock<Settings> singleton problem without code changes.
# Concurrency caps come from .config/nextest.toml (threads-required),
# not from this wrapper.
#
# Pre-flight (one-time per machine): cargo install cargo-nextest
# Verify:                            cargo nextest --version
#
# Usage:
#     scripts/brehon/cargo-nextest.sh run --workspace --features full --test e2e
#     scripts/brehon/cargo-nextest.sh run --workspace --features full --test e2e -E 'test(postgres_container_boots)'
#
# The `--test-threads=1` injection that cargo-test.sh does for e2e is
# intentionally OMITTED — nextest handles concurrency via
# .config/nextest.toml.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "TOOLCHAIN_OK"
command -v cargo && cargo --version
if ! cargo nextest --version >/dev/null 2>&1; then
  echo "CARGO_NEXTEST_NOT_INSTALLED: run \`cargo install cargo-nextest\` first."
  exit 1
fi
cargo nextest --version

# ---- libpq discovery on Windows (vcpkg-based, x64-windows dynamic triplet) ----
# Mirror of cargo-test.sh — needed when invoking nextest from Git Bash on
# Windows. Without these env vars, lemmy_server links with no /LIBPATH for
# libpq and fails at MSVC link.exe with `LNK1181: cannot open input file
# 'libpq.lib'`. Per feedback_pq_sys_stale_cache.md: the env vars MUST be
# set in the same shell that invokes cargo, because cargo's pq-sys build
# fingerprint hashes PQ_LIB_DIR.
case "${OSTYPE:-}" in
  msys*|cygwin*|win32*)
    : "${VCPKG_ROOT:=C:\\Users\\barri\\Developer\\vcpkg}"
    export VCPKG_ROOT
    export PQ_LIB_DIR="${VCPKG_ROOT}\\installed\\x64-windows\\lib"
    export PQ_INCLUDE_DIR="${VCPKG_ROOT}\\installed\\x64-windows\\include"
    export PATH="/c/Users/barri/Developer/vcpkg/installed/x64-windows/bin:${PATH}"
    echo "PQ_LIB_DIR=${PQ_LIB_DIR}"
    ;;
esac
echo "---"
cd "$REPO_ROOT"

exec cargo nextest "$@"
