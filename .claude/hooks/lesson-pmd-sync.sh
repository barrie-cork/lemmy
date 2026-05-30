#!/usr/bin/env bash
# PostToolUse hook: auto-sync a .claude/lessons/ file to PMD via MCP HTTP when it is
# written or edited with valid YAML frontmatter.
#
# Event: PostToolUse
# Matcher: Edit|Write
# Timeout: 15000
#
# WHY: sync-lessons-to-pmd.sh writes directly to a SQLite file via the
# PROJECT_MEMORY_DB env var. Under the HTTP-daemon topology (2026-05-30+) the
# MCP server is at http://localhost:11435/mcp and the SQLite file is managed
# server-side — the env-var path is gone. lessons written via Write/Edit tool
# therefore never reach the HTTP PMD unless manually promoted via an MCP
# memory_write call. This hook closes that gap: it fires immediately after every
# lesson Edit/Write, extracts the YAML frontmatter, and calls memory_write via
# the MCP session protocol so the lesson lands in the HTTP PMD automatically.
#
# ADVISORY ONLY — emits a notice but never blocks (exit 0 always). If the HTTP
# server is unreachable the lesson is still on disk; the weekly-review backfill
# sweep (Step 1b) is the safety net.
#
# SCOPE: only .claude/lessons/{feedback,reference}_*.md — the files the PMD
# ingests. Other .claude/lessons/ files (e.g. a README) are skipped.
#
# VALIDATION CONTRACT: mirrors lesson-frontmatter-reminder.sh — we only proceed
# if the file passes the same frontmatter check (has ---\n…\n---\n block with
# a non-empty name: field). If frontmatter is missing we stay silent (the
# frontmatter-reminder hook already warned about that).
#
# HTTP PROTOCOL: project-memory-mcp uses HTTP/SSE transport requiring a two-step
# session (initialize → tools/call with mcp-session-id). Token is read from
# .mcp.json Authorization header. Loopback requests skip auth.
#
# See: .claude/lessons/feedback_pmd_retro_check_http_store_split.md
#      .claude/rules/pmd-invariants.md §1 "Current topology"

set -o pipefail

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0

case "$TOOL_NAME" in
  Edit|Write) : ;;
  *) exit 0 ;;
esac

FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
[ -z "$FILE_PATH" ] && exit 0

# Normalise separators and scope to synced lesson files only
NORM=$(printf '%s' "$FILE_PATH" | tr '\\' '/')
case "$NORM" in
  *.claude/lessons/feedback_*.md|*.claude/lessons/reference_*.md) : ;;
  *) exit 0 ;;
esac

[ -f "$NORM" ] || exit 0

# ── 1. Parse frontmatter ──────────────────────────────────────────────────────
# Require a ---\n…\n---\n block with non-empty name:
FM_NAME=$(awk '
  /^---/ && !in_fm { in_fm=1; next }
  /^---/ &&  in_fm { in_fm=0; exit }
  in_fm && /^name:/ { sub(/^name:[[:space:]]*/, ""); print; exit }
' "$NORM")

[ -z "$FM_NAME" ] && exit 0   # no valid frontmatter — frontmatter-reminder hook handles this

FM_DESC=$(awk '
  /^---/ && !in_fm { in_fm=1; next }
  /^---/ &&  in_fm { in_fm=0; exit }
  in_fm && /^description:/ { sub(/^description:[[:space:]]*/, ""); print; exit }
' "$NORM")

FM_TYPE=$(awk '
  /^---/ && !in_fm { in_fm=1; next }
  /^---/ &&  in_fm { in_fm=0; exit }
  in_fm && /^type:/ { sub(/^type:[[:space:]]*/, ""); print; exit }
' "$NORM")
[ -z "$FM_TYPE" ] && FM_TYPE="feedback"

# Extract body (everything after closing ---)
BODY=$(awk '
  /^---/ && !in_fm { in_fm=1; next }
  /^---/ &&  in_fm { past=1; next }
  past { print }
' "$NORM")

# ── 2. Locate .mcp.json and extract HTTP URL ──────────────────────────────────
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HOOK_DIR/../.." && pwd)"
MCP_JSON="$REPO_ROOT/.mcp.json"

[ -f "$MCP_JSON" ] || exit 0

PMD_URL=$(jq -r '
  (.mcpServers // .)["project-memory"].url // empty
' "$MCP_JSON" 2>/dev/null)

[ -z "$PMD_URL" ] && exit 0   # not HTTP topology — skip silently

# Optionally extract Bearer token from Authorization header config
TOKEN=$(jq -r '
  ((.mcpServers // .)["project-memory"].headers // {})["Authorization"] // empty
' "$MCP_JSON" 2>/dev/null | sed 's/^Bearer //')

# ── 3. Check server reachable (fast 2-second probe) ──────────────────────────
if ! curl -s -m2 -o /dev/null -w "%{http_code}" "$PMD_URL" \
     ${TOKEN:+-H "Authorization: Bearer $TOKEN"} \
   | grep -qE '^[234]'; then
  echo "lesson-pmd-sync: HTTP PMD at $PMD_URL unreachable — $(basename "$NORM") not auto-synced (weekly-review Step 1b is the safety net)"
  exit 0
fi

# ── 4. MCP session: initialize ───────────────────────────────────────────────
INIT_PAYLOAD='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"lesson-pmd-sync-hook","version":"1.0"}}}'

SESSION_ID=$(curl -s -m5 \
  -X POST "$PMD_URL" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json, text/event-stream" \
  ${TOKEN:+-H "Authorization: Bearer $TOKEN"} \
  -d "$INIT_PAYLOAD" \
  -D - 2>/dev/null \
  | grep -i "^mcp-session-id:" | tr -d '\r' | awk '{print $2}')

[ -z "$SESSION_ID" ] && {
  echo "lesson-pmd-sync: could not obtain MCP session-id — $(basename "$NORM") not auto-synced"
  exit 0
}

BASENAME=$(basename "$NORM")
LESSON_FILE_PATH=".claude/lessons/$BASENAME"

# mcp_call <tool-name> <arguments-json>: POST a tools/call and echo the raw
# SSE response body. Arguments JSON is the inner "arguments" object.
mcp_call() {
  local tool="$1" args="$2"
  local payload
  payload=$(LP_TOOL="$tool" LP_ARGS="$args" python3 - <<'PYEOF'
import json, os
args = json.loads(os.environ["LP_ARGS"])
print(json.dumps({
    "jsonrpc": "2.0",
    "id": 2,
    "method": "tools/call",
    "params": {"name": os.environ["LP_TOOL"], "arguments": args},
}))
PYEOF
)
  curl -s -m10 \
    -X POST "$PMD_URL" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    -H "Mcp-Session-Id: $SESSION_ID" \
    ${TOKEN:+-H "Authorization: Bearer $TOKEN"} \
    -d "$payload" 2>/dev/null
}

# ── 5. Idempotency: find an existing PMD row for this lesson file ─────────────
# Editing a lesson must UPDATE its single row, not spawn a duplicate. The stable
# key is file_path; memory_get_file_context looks up rows BY file_path directly
# (memory_search is FTS5 token-match and misses filename-only queries). If
# multiple rows already exist for the path, pick the LOWEST id (oldest) so
# repeated edits converge on one canonical row.
TAGS="lesson,${FM_TYPE}"
SOURCE_REF=$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")

FILECTX_ARGS=$(LP_FILE="$LESSON_FILE_PATH" python3 - <<'PYEOF'
import json, os
print(json.dumps({"file_path": os.environ["LP_FILE"]}))
PYEOF
)
FILECTX_RESP=$(mcp_call "memory_get_file_context" "$FILECTX_ARGS")

EXISTING_ID=$(LP_RESP="$FILECTX_RESP" LP_FILE="$LESSON_FILE_PATH" python3 - <<'PYEOF'
import json, os
resp = os.environ.get("LP_RESP", "")
target = os.environ.get("LP_FILE", "")
ids = []
for line in resp.splitlines():
    line = line.strip()
    if line.startswith("data:"):
        line = line[5:].strip()
    if not line.startswith("{"):
        continue
    try:
        obj = json.loads(line)
    except Exception:
        continue
    result = obj.get("result")
    if not result:
        continue
    try:
        text = result["content"][0]["text"]
        rows = json.loads(text)
    except Exception:
        rows = result if isinstance(result, list) else []
    for r in rows:
        if isinstance(r, dict) and r.get("file_path") == target and r.get("id") is not None:
            try:
                ids.append(int(r["id"]))
            except Exception:
                pass
print(min(ids) if ids else "")
PYEOF
)

# ── 6. Build write-or-update arguments and call ──────────────────────────────
if [ -n "$EXISTING_ID" ]; then
  ARGS_JSON=$(LP_TITLE="$FM_NAME" LP_DESC="$FM_DESC" LP_BODY="$BODY" \
    LP_TAGS="$TAGS" LP_ID="$EXISTING_ID" python3 - <<'PYEOF'
import json, os
desc = os.environ.get("LP_DESC", "")
body = os.environ.get("LP_BODY", "")
full = (desc + "\n\n" + body).strip() if desc else body.strip()
print(json.dumps({
    "id": int(os.environ["LP_ID"]),
    "title": os.environ.get("LP_TITLE", ""),
    "content": full,
    "tags": os.environ.get("LP_TAGS", "lesson"),
}))
PYEOF
)
  RESPONSE=$(mcp_call "memory_update" "$ARGS_JSON")
  ACTION="updated existing"
  REPORT_ID="$EXISTING_ID"
else
  ARGS_JSON=$(LP_TITLE="$FM_NAME" LP_DESC="$FM_DESC" LP_BODY="$BODY" \
    LP_FILE="$LESSON_FILE_PATH" LP_SREF="$SOURCE_REF" LP_TAGS="$TAGS" python3 - <<'PYEOF'
import json, os
desc = os.environ.get("LP_DESC", "")
body = os.environ.get("LP_BODY", "")
full = (desc + "\n\n" + body).strip() if desc else body.strip()
print(json.dumps({
    "memory_type": "pattern",
    "title": os.environ.get("LP_TITLE", ""),
    "content": full,
    "file_path": os.environ.get("LP_FILE", ""),
    "source_ref": os.environ.get("LP_SREF", "unknown"),
    "source_type": "lesson-import",
    "tags": os.environ.get("LP_TAGS", "lesson"),
    "importance": 3,
}))
PYEOF
)
  RESPONSE=$(mcp_call "memory_write" "$ARGS_JSON")
  ACTION="wrote new"
  # Extract the new memory id from the result text (NOT the JSON-RPC envelope id).
  REPORT_ID=$(LP_RESP="$RESPONSE" python3 - <<'PYEOF'
import json, os, re
resp = os.environ.get("LP_RESP", "")
mid = ""
for line in resp.splitlines():
    line = line.strip()
    if line.startswith("data:"):
        line = line[5:].strip()
    if not line.startswith("{"):
        continue
    try:
        obj = json.loads(line)
    except Exception:
        continue
    result = obj.get("result")
    if not result:
        continue
    try:
        text = result["content"][0]["text"]
    except Exception:
        text = json.dumps(result)
    m = re.search(r'(?:ID|id)["\s:]+(\d+)', text)
    if m:
        mid = m.group(1)
        break
print(mid)
PYEOF
)
fi

if [ -n "$REPORT_ID" ]; then
  echo "lesson-pmd-sync: $BASENAME → ${ACTION} PMD memory ID $REPORT_ID (HTTP server)"
else
  echo "lesson-pmd-sync: ${ACTION} $BASENAME to PMD (HTTP server; no ID in response)"
fi

exit 0
