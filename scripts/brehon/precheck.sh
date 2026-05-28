#!/usr/bin/env bash
# scripts/brehon/precheck.sh — run before queueing Junior real-work tasks.
# Exits 0 if all gates pass; non-zero on failure.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
echo "[precheck] dq-lint-durations.sh ..."
"$SCRIPT_DIR/dq-lint-durations.sh"
echo "[precheck] OK"
