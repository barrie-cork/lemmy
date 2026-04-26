#!/usr/bin/env bash
# Brehon dev utility: run `cargo clippy` on macOS / Linux.
#
# Linux-side sibling of scripts/brehon/cargo-clippy.bat. No vcvars setup is
# needed — system `cc`/`ld` is sufficient on macOS and Linux. Callers pass
# scope and lint flags as positional args; this wrapper just sets the repo
# root and invokes cargo with the user-supplied args.
#
# Usage (from any shell, any cwd inside the repo):
#     scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings
#     scripts/brehon/cargo-clippy.sh -p lemmy_db_schema --features full --no-deps -- -D warnings
#
# Per .claude/lessons/feedback_wrapper_script_flag_silence.md, this wrapper
# accepts `$@` so the caller controls scope. Per
# feedback_features_full_p_crate_incompatible.md, do not combine `-p <crate>`
# with `--features full` unless the crate explicitly defines the `full`
# feature — the safe combinations are `--workspace --features full` or
# `-p <crate> --features <crate-specific>`.
#
# Per feedback_clippy_no_deps_uniform.md (if cited in lessons): always pass
# `--no-deps` to clippy so external-crate diagnostics don't flood output.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "TOOLCHAIN_OK"
command -v cargo && cargo --version
echo "---"
cd "$REPO_ROOT"
exec cargo clippy "$@"
