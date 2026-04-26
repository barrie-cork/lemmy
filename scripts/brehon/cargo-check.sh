#!/usr/bin/env bash
# Brehon dev utility: run `cargo check` on macOS / Linux.
#
# Linux-side sibling of scripts/brehon/cargo-check.bat. Unlike the Windows
# script, no vcvars setup is needed — system `cc`/`ld` is sufficient on
# macOS and Linux. Callers pass scope flags (`--workspace`, `-p <crate>`,
# `--features full`, `--all-targets`, etc); this wrapper just sets the
# repo root and invokes cargo with the user-supplied args.
#
# Usage (from any shell, any cwd inside the repo):
#     scripts/brehon/cargo-check.sh --workspace
#     scripts/brehon/cargo-check.sh -p lemmy_db_schema
#     scripts/brehon/cargo-check.sh --workspace --features full
#
# Per .claude/lessons/feedback_wrapper_script_flag_silence.md, this wrapper
# accepts `$@` so the caller controls scope. Hardcoding `--workspace` here
# would silently discard `-p <crate>` etc — the exact bug the lesson names.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "TOOLCHAIN_OK"
command -v cargo && cargo --version
echo "---"
cd "$REPO_ROOT"
exec cargo check "$@"
