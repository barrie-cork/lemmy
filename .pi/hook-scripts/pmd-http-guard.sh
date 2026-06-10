#!/usr/bin/env bash
# Pi SessionStart hook: verify the PMD endpoint used by Pi/RLS is reachable.
# Never blocks. Loads token from PMD_HTTP_TOKEN or PMD_ENV_FILE when available.

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

seen=""
for url in "${candidates[@]}"; do
  [[ -n "$url" ]] || continue
  case " $seen " in *" $url "*) continue ;; esac
  seen="$seen $url"
  code=$(curl -s -m3 -o /dev/null -w '%{http_code}' "$url" "${AUTH[@]}" 2>/dev/null || true)
  if [[ "$code" =~ ^(2|3|4)[0-9][0-9]$ ]]; then
    # 401/405 still proves the daemon is reachable; auth/tool calls are checked by write/query paths.
    echo "pmd-http-guard: reachable $url (http $code)"
    exit 0
  fi
done

echo "pmd-http-guard WARN: PMD HTTP endpoint unreachable. Tried:${seen:- <none>}. Pi PMD query/write and lesson sync will be degraded until pmd-http-mcp or the configured endpoint is up." >&2
exit 0
