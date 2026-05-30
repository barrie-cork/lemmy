#!/usr/bin/env bash
# SessionStart hook: guard PMD configuration at session start.
#
# HTTP topology (2026-05-30+): project-memory MCP is an HTTP daemon at
# localhost:11435. The env-var PROJECT_MEMORY_DB path check is no longer
# applicable (HTTP configs have no env section). This hook now has two modes:
#
#   HTTP mode  (.mcp.json has type=http for project-memory):
#     → checks that the HTTP server is reachable; WARNs if not.
#     → the old env-var divergence check is SKIPPED (no env section to read).
#
#   stdio mode (.mcp.json has command/args/env for project-memory, legacy):
#     → runs the original PROJECT_MEMORY_DB canonical-path divergence check.
#
# Both modes exit 0 always (WARN-not-FAIL). Per
# feedback_mcp_canonical_pmd_path_enforce_at_session_start.md +
# feedback_pmd_retro_check_http_store_split.md.
#
# Event: SessionStart
# Timeout: 5000

set -euo pipefail

# --- .mcp.json absence handling ---
if [ ! -f ".mcp.json" ]; then
  exit 0
fi

# --- Detect HTTP vs stdio topology ---
PMD_TYPE=$(python3 -c "
import json,io
d=json.load(io.open('.mcp.json',encoding='utf-8'))
s=d.get('mcpServers',d).get('project-memory',{})
print(s.get('type','stdio'))
" 2>/dev/null || echo "stdio")

if [ "$PMD_TYPE" = "http" ]; then
  # HTTP mode: check server reachability only
  PMD_URL=$(python3 -c "
import json,io
d=json.load(io.open('.mcp.json',encoding='utf-8'))
s=d.get('mcpServers',d).get('project-memory',{})
print(s.get('url',''))
" 2>/dev/null || true)
  if [ -z "$PMD_URL" ]; then
    echo "pmd-canonical-guard WARN: HTTP topology but no url found in .mcp.json project-memory config" >&2
    exit 0
  fi
  if ! curl -s -m3 -o /dev/null "$PMD_URL" 2>/dev/null; then
    echo "pmd-canonical-guard WARN: PMD HTTP server unreachable at $PMD_URL — memory_search_hybrid and memory_write will fail this session" >&2
  fi
  exit 0
fi
# stdio mode falls through to the original PROJECT_MEMORY_DB divergence check below

# --- Compute canonical target ---
# Mirror retro-check.sh lines 32-41: derive from git-common-dir so the guard
# and the Stop hook agree by construction.
#
# Per CR cp-1 (PR #140): in a non-worktree checkout (the canonical brehon-fork
# CWD), `git rev-parse --git-common-dir` returns the literal `.git` —
# a relative path. The condition `GIT_COMMON != ".git"` on the original
# Task 3 implementation prevented construction in that case, producing
# a silent false-negative (the guard never warned in the canonical lane).
# Fix: when --git-common-dir returns `.git`, use --show-toplevel to get
# the repo root, then construct `${root}/.project-memory/memory.db`.
# Falls back to the script's own repo location for sentinel-dir probes
# (where CWD is not a git repository).
CANONICAL=""
MAIN_REPO=""
# Helper: anchor a possibly-relative git-common-dir to a base dir, then
# return the absolute MAIN_REPO (dirname of the .git). Per PR #140 cr-9:
# git rev-parse --git-common-dir can return a relative path in worktree
# or custom GIT_DIR setups; dirname on a relative path preserves the
# relative form, and downstream os.path.abspath() then anchors to the
# wrong CWD. Resolve to absolute up-front.
_resolve_main_repo() {
  local common="$1"
  local base_dir="$2"  # repo CWD or SCRIPT_DIR
  # Already absolute (POSIX `/foo` or Windows `C:/foo` / `C:\foo`)?
  case "$common" in
    /*) echo "$(dirname "$common")"; return ;;
    [A-Za-z]:/*|[A-Za-z]:\\*) echo "$(dirname "$common")"; return ;;
  esac
  # Relative path: anchor to base_dir, then dirname.
  local resolved
  resolved=$(cd "$base_dir/$common/.." 2>/dev/null && pwd || true)
  echo "$resolved"
}
GIT_COMMON=$(git rev-parse --git-common-dir 2>/dev/null || true)
if [ -n "$GIT_COMMON" ]; then
  if [ "$GIT_COMMON" = ".git" ]; then
    # Non-worktree checkout (canonical lane). Derive repo root via --show-toplevel.
    MAIN_REPO=$(git rev-parse --show-toplevel 2>/dev/null || true)
  else
    MAIN_REPO=$(_resolve_main_repo "$GIT_COMMON" "$(pwd)")
  fi
fi
if [ -z "$MAIN_REPO" ]; then
  # CWD is not a git repository; try the script's own repo location (sentinel probe).
  SCRIPT_PATH="${BASH_SOURCE[0]:-}"
  if [ -n "$SCRIPT_PATH" ]; then
    SCRIPT_DIR=$(cd "$(dirname "$SCRIPT_PATH")" 2>/dev/null && pwd || true)
    if [ -n "$SCRIPT_DIR" ]; then
      SCRIPT_COMMON=$(git -C "$SCRIPT_DIR" rev-parse --git-common-dir 2>/dev/null || true)
      if [ -n "$SCRIPT_COMMON" ]; then
        if [ "$SCRIPT_COMMON" = ".git" ]; then
          MAIN_REPO=$(git -C "$SCRIPT_DIR" rev-parse --show-toplevel 2>/dev/null || true)
        else
          MAIN_REPO=$(_resolve_main_repo "$SCRIPT_COMMON" "$SCRIPT_DIR")
        fi
      fi
    fi
  fi
fi
if [ -n "$MAIN_REPO" ]; then
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
# Per CR cr-2 (PR #140): use os.path.abspath() so a relative PROJECT_MEMORY_DB
# (e.g. './.project-memory/memory.db' in a lane .mcp.json) compares correctly
# against the absolute canonical path. Without abspath, normpath alone would
# leave the relative form and the string comparison would silently
# false-positive a mismatch.
normalise_path() {
  local raw="$1"
  "$PYTHON_BIN" -c "
import os, sys
p = sys.argv[1]
if len(p) >= 2 and p[1] == ':':
    p = p[0].lower() + p[1:]
p = os.path.abspath(os.path.normpath(os.path.expanduser(p)))
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
