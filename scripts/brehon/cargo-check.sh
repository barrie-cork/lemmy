#!/usr/bin/env bash
# Brehon dev utility: run `cargo check --workspace` on macOS / Linux.
#
# This is the Unix-side sibling of scripts/brehon/cargo-check.bat. Unlike the
# Windows script, no vcvars setup is needed — the system `cc`/`ld` toolchain is
# already sufficient for Rust's default targets on macOS and Linux.
#
# Usage (from any shell, any cwd inside the repo):
#     scripts/brehon/cargo-check.sh
#
# Mirrors the `.bat` side's behaviour: change to the repo root, print which
# `cargo` is being used for diagnostic parity, then run the check.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "TOOLCHAIN_OK"
command -v cargo && cargo --version
echo "---"
cd "$REPO_ROOT"
cargo check --workspace
