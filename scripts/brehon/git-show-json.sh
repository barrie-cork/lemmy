#!/usr/bin/env bash
# git-show-json.sh — read a JSON file out of a git ref, sidestepping
# Windows Bash↔Python /tmp + colon-arg + UTF-8 traps.
#
# Why this exists:
#   On Windows under Claude Code's Bash tool (PowerShell underneath):
#     1. `git show <branch>:<path>` mangles the colon to a semicolon
#        when <branch> contains slashes.
#     2. /tmp paths from Bash don't resolve to the same file Python
#        sees on Windows (Bash MSYS mount vs native Python).
#     3. Python 3.14 default codec on Windows is cp1252; reading
#        UTF-8 JSON via stdin pipe trips on first em-dash.
#
# This wrapper:
#   - resolves the ref to a SHA first (no slashes, no colons in the
#     git argument)
#   - writes to $LOCALAPPDATA/Temp on Windows or /tmp on Linux/macOS
#   - leaves the file on disk for downstream Python with explicit
#     encoding='utf-8'
#
# Usage:
#   ./scripts/brehon/git-show-json.sh <ref> <path-in-repo>
#
# Output:
#   stdout: absolute path to the captured file
#   exit 0 on success, non-zero on git-show failure
#
# Example:
#   path=$(./scripts/brehon/git-show-json.sh \
#     origin/junior/role-impl-task-sl-c-2-impl-1-...-md-154 \
#     .claude/decision-queue.json)
#   python -c "
#   import io, json
#   d = json.load(io.open(r'$path', encoding='utf-8'))
#   print(len(d['pending']))
#   "

set -euo pipefail

if [ "$#" -ne 2 ]; then
  echo "usage: $0 <git-ref> <path-in-repo>" >&2
  echo "  example: $0 origin/governance-v0 .claude/decision-queue.json" >&2
  exit 2
fi

REF="$1"
SRC_PATH="$2"

# Resolve the ref to a SHA (sidesteps trap 1)
SHA=$(git rev-parse --verify "${REF}^{commit}" 2>/dev/null) || {
  echo "error: cannot resolve ref: ${REF}" >&2
  exit 3
}

# Pick a tmpdir that BOTH Bash and native Windows tools agree on.
# On Windows under Git Bash, $LOCALAPPDATA is set; on Linux/macOS it isn't.
if [ -n "${LOCALAPPDATA:-}" ]; then
  # Windows. $LOCALAPPDATA is `C:\Users\<user>\AppData\Local`. Normalise
  # the slashes to forward for Bash, but keep the drive-letter form so
  # downstream Python sees the same path.
  TMPDIR_NATIVE="${LOCALAPPDATA//\\//}/Temp"
else
  # Linux / macOS — /tmp is fine.
  TMPDIR_NATIVE="/tmp"
fi

mkdir -p "${TMPDIR_NATIVE}"

# Sanitise filename (replace path separators)
SAFE_NAME=$(echo "${SRC_PATH}" | tr '/\\' '__')
OUT_PATH="${TMPDIR_NATIVE}/git-show-${SHA:0:8}-${SAFE_NAME}"

# Capture (sidesteps trap 1; <SHA>:<path> is unambiguous)
git show "${SHA}:${SRC_PATH}" > "${OUT_PATH}" || {
  echo "error: git show ${SHA}:${SRC_PATH} failed" >&2
  rm -f "${OUT_PATH}"
  exit 4
}

# Emit the path. Caller passes this to Python with encoding='utf-8'.
echo "${OUT_PATH}"
