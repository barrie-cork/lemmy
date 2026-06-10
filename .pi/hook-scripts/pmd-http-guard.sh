#!/usr/bin/env bash
# Pi SessionStart hook: verify the PMD endpoint used by Pi/RLS accepts an
# authenticated StreamableHTTP MCP initialize request. Never blocks. Loads token
# from PMD_HTTP_TOKEN or PMD_ENV_FILE when available.

set -o pipefail

if [[ -z "${PMD_HTTP_TOKEN:-}" ]]; then
  for env_file in "${PMD_ENV_FILE:-}" /srv/brehon-fork/.env .env; do
    [[ -n "$env_file" && -f "$env_file" ]] || continue
    # shellcheck disable=SC1090
    set -a; source "$env_file"; set +a
    break
  done
fi

TOKEN="${PMD_HTTP_TOKEN:-}"
AUTH=()
[[ -n "$TOKEN" ]] && AUTH=(-H "Authorization: Bearer $TOKEN")

candidates=()
[[ -n "${PMD_HTTP_URL:-}" ]] && candidates+=("$PMD_HTTP_URL")
if [[ -f .mcp.json ]] && command -v jq >/dev/null 2>&1; then
  url=$(jq -r '(.mcpServers // .)["project-memory"].url // empty' .mcp.json 2>/dev/null || true)
  [[ -n "$url" ]] && candidates+=("$url")
fi
candidates+=("http://localhost:11435/mcp" "http://100.104.171.26:11435/mcp")

INIT='{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"pi-pmd-http-guard","version":"1.0"}},"id":1}'

seen=""
last_failure=""
for url in "${candidates[@]}"; do
  [[ -n "$url" ]] || continue
  case " $seen " in *" $url "*) continue ;; esac
  seen="$seen $url"

  headers=$(mktemp /tmp/pi-pmd-guard-headers.XXXXXX)
  body=$(mktemp /tmp/pi-pmd-guard-body.XXXXXX)
  code=$(curl -s -m4 -X POST "$url" \
    -H "Content-Type: application/json" \
    -H "Accept: application/json, text/event-stream" \
    "${AUTH[@]}" \
    --dump-header "$headers" \
    -o "$body" \
    -w '%{http_code}' \
    -d "$INIT" 2>/dev/null || true)
  session_id=$(grep -i '^mcp-session-id:' "$headers" 2>/dev/null | awk '{print $2}' | tr -d '\r' || true)

  if [[ "$code" =~ ^2[0-9][0-9]$ && -n "$session_id" ]]; then
    echo "pmd-http-guard: MCP initialize OK $url (http $code)"
    rm -f "$headers" "$body"
    exit 0
  fi

  last_failure="$url http ${code:-000} session_id=$([[ -n "$session_id" ]] && echo yes || echo no)"
  rm -f "$headers" "$body"
done

if [[ -z "$TOKEN" ]]; then
  echo "pmd-http-guard WARN: PMD_HTTP_TOKEN missing; authenticated MCP initialize could not be verified. Tried:${seen:- <none>}. Run scripts/brehon/pmd-doctor.sh for details." >&2
else
  echo "pmd-http-guard WARN: PMD authenticated MCP initialize failed. Tried:${seen:- <none>}. Last failure: ${last_failure:-none}. Run scripts/brehon/pmd-doctor.sh for details." >&2
fi
exit 0
