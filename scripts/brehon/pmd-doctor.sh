#!/usr/bin/env bash
# pmd-doctor.sh — diagnose Project Memory DB (PMD) HTTP/MCP connectivity.
#
# Checks endpoint discovery, token availability, authenticated MCP initialize,
# and (when initialize succeeds) tools/list. Never prints secrets.

set -u -o pipefail

ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"

load_env() {
  if [[ -z "${PMD_HTTP_TOKEN:-}" ]]; then
    for env_file in "${PMD_ENV_FILE:-}" /srv/brehon-fork/.env "$ROOT/.env"; do
      [[ -n "$env_file" && -f "$env_file" ]] || continue
      # shellcheck disable=SC1090
      set -a; source "$env_file"; set +a
      LOADED_ENV="$env_file"
      break
    done
  fi
}

endpoint_candidates() {
  [[ -n "${PMD_HTTP_URL:-}" ]] && printf '%s\n' "$PMD_HTTP_URL"
  if [[ -f "$ROOT/.mcp.json" ]] && command -v jq >/dev/null 2>&1; then
    jq -r '(.mcpServers // .)["project-memory"].url // empty' "$ROOT/.mcp.json" 2>/dev/null || true
  fi
  printf '%s\n' "http://localhost:11435/mcp" "http://100.104.171.26:11435/mcp"
}

curl_mcp() {
  local url="$1" session_id="${2:-}" payload="$3" body="$4" headers="$5" code
  local -a args
  args=(-s -m 8 -X POST "$url"
    -H "Content-Type: application/json"
    -H "Accept: application/json, text/event-stream")
  if [[ -n "${PMD_HTTP_TOKEN:-}" ]]; then
    args+=(-H "Authorization: Bearer ${PMD_HTTP_TOKEN}")
  fi
  if [[ -n "$session_id" ]]; then
    args+=(-H "Mcp-Session-Id: $session_id")
  fi
  code=$(curl "${args[@]}" \
    --dump-header "$headers" -o "$body" -w '%{http_code}' \
    -d "$payload" 2>/dev/null || true)
  printf '%s' "$code"
}

json_has_error() {
  python3 - "$1" <<'PY'
import json, pathlib, sys
text = pathlib.Path(sys.argv[1]).read_text(errors='replace')
for line in text.splitlines():
    line = line.strip()
    if line.startswith('data:'):
        line = line[5:].strip()
    if not line.startswith('{'):
        continue
    try:
        obj = json.loads(line)
    except Exception:
        continue
    if 'error' in obj:
        sys.exit(0)
sys.exit(1)
PY
}

LOADED_ENV=""
load_env

printf 'PMD doctor\n'
printf 'repo: %s\n' "$ROOT"
printf 'env file loaded: %s\n' "${LOADED_ENV:-<none>}"
printf 'token: %s\n' "$([[ -n "${PMD_HTTP_TOKEN:-}" ]] && echo present || echo missing)"
if [[ ! -f "$ROOT/.mcp.json" ]]; then
  printf 'mcp config: %s\n' '<none>'
elif command -v jq >/dev/null 2>&1; then
  printf 'mcp config: %s\n' "$ROOT/.mcp.json"
else
  printf 'mcp config: %s (jq missing; cannot parse)\n' "$ROOT/.mcp.json"
fi
printf '\n'

INIT='{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"pmd-doctor","version":"1.0"}},"id":1}'
TOOLS='{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}'

status=1
seen=""
while IFS= read -r url; do
  [[ -n "$url" ]] || continue
  case " $seen " in *" $url "*) continue ;; esac
  seen="$seen $url"

  body=$(mktemp /tmp/pmd-doctor-body.XXXXXX)
  headers=$(mktemp /tmp/pmd-doctor-headers.XXXXXX)
  code=$(curl_mcp "$url" "" "$INIT" "$body" "$headers")
  session_id=$(grep -i '^mcp-session-id:' "$headers" 2>/dev/null | awk '{print $2}' | tr -d '\r' || true)

  if [[ "$code" =~ ^2[0-9][0-9]$ && -n "$session_id" ]] && ! json_has_error "$body"; then
    printf 'OK initialize: %s (http %s, session-id returned)\n' "$url" "$code"
    tools_body=$(mktemp /tmp/pmd-doctor-tools-body.XXXXXX)
    tools_headers=$(mktemp /tmp/pmd-doctor-tools-headers.XXXXXX)
    tools_code=$(curl_mcp "$url" "$session_id" "$TOOLS" "$tools_body" "$tools_headers")
    if [[ "$tools_code" =~ ^2[0-9][0-9]$ ]] && ! json_has_error "$tools_body"; then
      printf 'OK tools/list: %s (http %s)\n' "$url" "$tools_code"
      status=0
    else
      printf 'WARN tools/list failed: %s (http %s)\n' "$url" "${tools_code:-000}"
      sed -n '1,5p' "$tools_body" | sed 's/^/  body: /'
    fi
    rm -f "$tools_body" "$tools_headers"
  else
    printf 'FAIL initialize: %s (http %s, session-id %s)\n' "$url" "${code:-000}" "$([[ -n "$session_id" ]] && echo yes || echo no)"
    sed -n '1,3p' "$body" | sed 's/^/  body: /'
  fi
  rm -f "$body" "$headers"
done < <(endpoint_candidates)

if [[ -z "$seen" ]]; then
  printf 'FAIL: no PMD endpoint candidates found\n'
fi

exit "$status"
