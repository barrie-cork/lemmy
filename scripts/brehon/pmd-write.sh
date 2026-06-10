#!/usr/bin/env bash
# pmd-write.sh — write a memory to the Project Memory DB via MCP HTTP from Pi/headless contexts
#
# Usage: pmd-write.sh --title "Title" --content "Body" [--type TYPE] [--tags TAG1,TAG2]
# Types: pattern, decision, lesson, reference, user, feedback, project
#
# Token lives in NSSM AppEnvironmentExtra on the laptop PMD server (pmd-http-mcp service).
# Same 2-step MCP protocol as pmd-query.sh.

set -euo pipefail

TITLE=""
CONTENT=""
MEMORY_TYPE="lesson"
TAGS=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --title)   TITLE="$2";       shift 2 ;;
    --content) CONTENT="$2";     shift 2 ;;
    --type)    MEMORY_TYPE="$2"; shift 2 ;;
    --tags)    TAGS="$2";        shift 2 ;;
    *) shift ;;
  esac
done

if [[ -z "$TITLE" || -z "$CONTENT" ]]; then
  echo '{"error": "usage: pmd-write.sh --title \"...\" --content \"...\" [--type TYPE] [--tags TAGS]"}' >&2
  exit 1
fi

if [[ -z "${PMD_HTTP_TOKEN:-}" ]]; then
  for ENV_FILE in "${PMD_ENV_FILE:-}" /srv/brehon-fork/.env .env; do
    [[ -n "$ENV_FILE" && -f "$ENV_FILE" ]] || continue
    set -a; source "$ENV_FILE"; set +a
    break
  done
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

INIT_RESP=$(curl "${CURL_ARGS[@]}" \
  --dump-header /tmp/pmd-write-headers-$$ \
  -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"pmd-cli","version":"0.1"}},"id":1}' 2>&1)

SESSION_ID=$(grep -i 'mcp-session-id:' /tmp/pmd-write-headers-$$ 2>/dev/null | awk '{print $2}' | tr -d '\r' || true)
rm -f /tmp/pmd-write-headers-$$

if [[ -z "$SESSION_ID" ]]; then
  echo '{"error": "PMD initialize failed — no session-id returned"}' >&2
  echo "$INIT_RESP" >&2
  exit 1
fi

PARAMS_JSON=$(python3 -c "
import json, sys
title, content, mtype, tags = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4]
params = {'title': title, 'content': content, 'memory_type': mtype}
if tags:
    params['tags'] = tags
print(json.dumps(params))
" "$TITLE" "$CONTENT" "$MEMORY_TYPE" "$TAGS")

TOOL_RESP=$(curl "${CURL_ARGS[@]}" \
  -H "mcp-session-id: $SESSION_ID" \
  -d "{\"jsonrpc\":\"2.0\",\"method\":\"tools/call\",\"params\":{\"name\":\"memory_write\",\"arguments\":$PARAMS_JSON},\"id\":2}" 2>&1)

echo "$TOOL_RESP" | grep '^data:' | sed 's/^data: //' | python3 -c "
import sys, json
raw = sys.stdin.read().strip()
try:
    obj = json.loads(raw)
    result = obj.get('result', obj)
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
