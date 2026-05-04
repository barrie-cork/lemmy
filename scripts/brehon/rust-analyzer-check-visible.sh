#!/usr/bin/env bash
# Brehon dev utility: rust-analyzer check wrapper with agent-visible logs.
#
# rust-analyzer's `check.overrideCommand` requires stdout to contain only
# Cargo JSON messages. This wrapper therefore tees stdout unchanged to a .pi
# jsonl file and sends wrapper/status output to stderr only.
#
# Intended VSCode setting:
#   "rust-analyzer.check.overrideCommand": [
#     "scripts/brehon/rust-analyzer-check-visible.sh",
#     "--workspace",
#     "--message-format=json",
#     "--all-targets",
#     "--keep-going"
#   ]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PI_DIR="$REPO_ROOT/.pi"
STDOUT_LOG="$PI_DIR/rust-analyzer-check.jsonl"
STDERR_LOG="$PI_DIR/rust-analyzer-check.stderr.log"

mkdir -p "$PI_DIR"
: > "$STDOUT_LOG"
: > "$STDERR_LOG"

{
  echo "TOOLCHAIN_OK"
  command -v cargo
  cargo --version
  echo "rust-analyzer-check-visible args: $*"
  echo "---"
} >> "$STDERR_LOG" 2>&1

cd "$REPO_ROOT"

# Preserve cargo's JSON stdout for rust-analyzer while making the same stream
# readable to pi. Preserve stderr and cargo's exit status as well.
set +e
cargo check "$@" \
  > >(tee "$STDOUT_LOG") \
  2> >(tee -a "$STDERR_LOG" >&2)
status=$?
set -e

echo "exit=$status" >> "$STDERR_LOG"
exit "$status"
