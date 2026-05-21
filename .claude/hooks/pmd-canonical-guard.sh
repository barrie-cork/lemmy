#!/usr/bin/env bash
# SessionStart hook: WARN if this lane's .mcp.json PROJECT_MEMORY_DB diverges
# from the canonical cross-lane path. Per
# feedback_mcp_canonical_pmd_path_enforce_at_session_start.md.
# Deployed to .claude/hooks/pmd-canonical-guard.sh on server repos.
#
# Event: SessionStart
# Timeout: 5000
#
# Exit 0 = continue (always; this hook is WARN-not-FAIL per the lesson's
# WARN-vs-FAIL rationale — a configuration error fixed by editing .mcp.json
# and restarting; blocking tool calls would frustrate, not fix).
#
# CWD reliance: reads .mcp.json from CWD root. Callers that need to test a
# sentinel path must cd to a directory containing a sentinel .mcp.json before
# invoking (Task 10 dogfood sub-run (b) does exactly this).

set -euo pipefail

# --- .mcp.json absence handling ---
# Absence is safe: some lanes resolve canonical by default (no explicit .mcp.json).
if [ ! -f ".mcp.json" ]; then
  exit 0
fi

# --- Compute canonical target ---
# Mirror retro-check.sh lines 32-41: derive from git-common-dir so the guard
# and the Stop hook agree by construction.
# Falls back to the script's own repo location to handle sentinel-dir probes
# (where CWD is not a git repository).
CANONICAL=""
GIT_COMMON=$(git rev-parse --git-common-dir 2>/dev/null || true)
if [ -z "$GIT_COMMON" ] || [ "$GIT_COMMON" = ".git" ]; then
  # CWD is not a git worktree; try the script's own repo location.
  SCRIPT_PATH="${BASH_SOURCE[0]:-}"
  if [ -n "$SCRIPT_PATH" ]; then
    SCRIPT_DIR=$(cd "$(dirname "$SCRIPT_PATH")" 2>/dev/null && pwd || true)
    if [ -n "$SCRIPT_DIR" ]; then
      GIT_COMMON=$(git -C "$SCRIPT_DIR" rev-parse --git-common-dir 2>/dev/null || true)
    fi
  fi
fi
if [ -n "$GIT_COMMON" ] && [ "$GIT_COMMON" != ".git" ]; then
  MAIN_REPO=$(dirname "$GIT_COMMON")
  CANONICAL="${MAIN_REPO}/.project-memory/memory.db"
fi

if [ -z "$CANONICAL" ]; then
  # Cannot determine canonical path; fail open — do not block the session.
  exit 0
fi

# --- Select python binary ---
PYTHON_BIN=""
if command -v python3 &>/dev/null; then
  PYTHON_BIN="python3"
elif command -v python &>/dev/null; then
  PYTHON_BIN="python"
else
  echo "pmd-canonical-guard: python unavailable — skipping PROJECT_MEMORY_DB check" >&2
  exit 0
fi

# --- Extract running value from .mcp.json ---
# UTF-8-safe one-liner from feedback_pmd_cross_lane_canonical_db.md §"Diagnosis recipe" step 1.
RUNNING=$("$PYTHON_BIN" -c "import json,io;print(json.load(io.open('.mcp.json',encoding='utf-8'))['mcpServers']['project-memory']['env']['PROJECT_MEMORY_DB'])" 2>/dev/null || true)

if [ -z "$RUNNING" ]; then
  # Key missing from .mcp.json — absence of explicit value is safe.
  exit 0
fi

# --- Normalise paths ---
# Resolve to absolute paths; case-fold Windows drive letter (C: vs c:).
normalise_path() {
  local raw="$1"
  "$PYTHON_BIN" -c "
import os, sys
p = sys.argv[1]
if len(p) >= 2 and p[1] == ':':
    p = p[0].lower() + p[1:]
p = os.path.normpath(os.path.expanduser(p))
print(p)
" "$raw" 2>/dev/null || echo "$raw"
}

RUNNING_NORM=$(normalise_path "$RUNNING")
CANONICAL_NORM=$(normalise_path "$CANONICAL")

# --- Compare and emit WARN on mismatch ---
if [ "$RUNNING_NORM" = "$CANONICAL_NORM" ]; then
  exit 0
fi

echo "pmd-canonical-guard WARN: PROJECT_MEMORY_DB mismatch detected at session start" >&2
echo "  Running (.mcp.json):        $RUNNING" >&2
echo "  Canonical (git-common-dir): $CANONICAL" >&2
echo "  Fix: edit '.mcp.json' PROJECT_MEMORY_DB to '$CANONICAL' and restart the MCP — the running MCP cached its handle at startup, see the sequencing constraint in feedback_pmd_cross_lane_canonical_db.md" >&2

exit 0
