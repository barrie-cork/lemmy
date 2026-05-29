#!/usr/bin/env bash
# scripts/brehon/precheck.sh — run before queueing Junior real-work tasks.
# Exits 0 if all gates pass; non-zero on failure.
#
# Derives REPO_ROOT from SCRIPT_DIR so the lint operates on the canonical
# .claude/decision-queue.json regardless of caller CWD. Mirrors the sibling
# pattern in scripts/brehon/dq-v3-append-fragment.sh.
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
echo "[precheck] dq-lint-durations.sh ..."
"$SCRIPT_DIR/dq-lint-durations.sh" "${REPO_ROOT}/.claude/decision-queue.json"
echo "[precheck] OK"
