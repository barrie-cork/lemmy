#!/usr/bin/env bash
# pmd-query.sh — query the Project Memory DB (PMD) via MCP HTTP from Pi/headless contexts
#
# Usage: pmd-query.sh <query> [--limit N] [--tags TAG1,TAG2]
# Output: JSON array of memory objects on stdout
#
# Requires: PMD_HTTP_URL (default: http://100.104.171.26:11435/mcp) and
#           PMD_HTTP_TOKEN in environment (or /srv/brehon-fork/.env sourced).
# Token lives in NSSM AppEnvironmentExtra on the laptop PMD server (pmd-http-mcp service).
#
# Protocol: StreamableHTTP MCP — 2-step: initialize (get session-id) then tools/call.

set -euo pipefail

QUERY="${1:-}"
if [[ -z "$QUERY" ]]; then
  echo '{"error": "usage: pmd-query.sh <query> [--limit N] [--tags TAGS]"}' >&2
  exit 1
fi

LIMIT=5
TAGS=""
shift || true
while [[ $# -gt 0 ]]; do
  case "$1" in
    --limit) LIMIT="$2"; shift 2 ;;
    --tags)  TAGS="$2"; shift 2 ;;
    *) shift ;;
  esac
done

# Source .env if token not already in environment
if [[ -z "${PMD_HTTP_TOKEN:-}" ]]; then
  ENV_FILE="${PMD_ENV_FILE:-/srv/brehon-fork/.env}"
  if [[ -f "$ENV_FILE" ]]; then
    # shellcheck disable=SC1090
    set -a; source "$ENV_FILE"; set +a
  fi
fi

PMD_URL="${PMD_HTTP_URL:-http://100.104.171.26:11435/mcp}"
TOKEN="${PMD_HTTP_TOKEN:-}"

AUTH_HEADER=""
if [[ -n "$TOKEN" ]]; then
  AUTH_HEADER="Authorization: Bearer $TOKEN"
fi

CURL_ARGS=(-s -m 15 -X POST "$PMD_URL"
  -H "Content-Type: application/json"
  -H "Accept: application/json, text/event-stream"
)
if [[ -n "$AUTH_HEADER" ]]; then
  CURL_ARGS+=(-H "$AUTH_HEADER")
fi

# Step 1: initialize — get session ID
INIT_RESP=$(curl "${CURL_ARGS[@]}" \
  --dump-header /tmp/pmd-init-headers-$$ \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"pmd-cli","version":"0.1"}},"id":1}' 2>&1)

SESSION_ID=$(grep -i 'mcp-session-id:' /tmp/pmd-init-headers-$$ 2>/dev/null | awk '{print $2}' | tr -d '\r' || true)
rm -f /tmp/pmd-init-headers-$$

if [[ -z "$SESSION_ID" ]]; then
  echo '{"error": "PMD initialize failed — no session-id returned"}' >&2
  echo "$INIT_RESP" >&2
  exit 1
fi

# Build tool params
PARAMS_JSON=$(python3 -c "
import json, sys
params = {'query': sys.argv[1], 'limit': int(sys.argv[2])}
if sys.argv[3]:
    params['tags'] = sys.argv[3]
print(json.dumps(params))
" "$QUERY" "$LIMIT" "$TAGS")

TOOL_NAME="memory_search_hybrid"

# Step 2: tools/call
TOOL_RESP=$(curl "${CURL_ARGS[@]}" \
  -H "mcp-session-id: $SESSION_ID" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"$TOOL_NAME\",\"arguments\":$PARAMS_JSON},\"id\":2}" 2>&1)

# Extract data: line from SSE response and parse result
echo "$TOOL_RESP" | grep '^data:' | sed 's/^data: //' | python3 -c "
import sys, json
raw = sys.stdin.read().strip()
try:
    obj = json.loads(raw)
    result = obj.get('result', obj)
    # MCP tool result has content array with text items
    content = result.get('content', [])
    for item in content:
        if item.get('type') == 'text':
            print(item['text'])
            break
    else:
        print(json.dumps(result))
except Exception as e:
    print(raw)
"
