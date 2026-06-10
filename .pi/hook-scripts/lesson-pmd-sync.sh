#!/usr/bin/env bash
# Pi PostToolUse hook: auto-sync valid .claude/lessons/{feedback,reference}_*.md
# files to the PMD HTTP daemon. Advisory only; never blocks.

set -o pipefail

INPUT=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0
case "$TOOL_NAME" in Edit|Write) : ;; *) exit 0 ;; esac

FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null) || exit 0
[[ -n "$FILE_PATH" ]] || exit 0
NORM=$(printf '%s' "$FILE_PATH" | tr '\\' '/')
case "$NORM" in *.claude/lessons/feedback_*.md|*.claude/lessons/reference_*.md) : ;; *) exit 0 ;; esac
[[ -f "$NORM" ]] || exit 0

# Require valid frontmatter with non-empty name; missing-frontmatter warnings are
# handled by lesson-frontmatter-reminder.sh.
FM_NAME=$(awk '/^---/ && !in_fm { in_fm=1; next } /^---/ && in_fm { exit } in_fm && /^name:/ { sub(/^name:[[:space:]]*/, ""); print; exit }' "$NORM")
[[ -n "$FM_NAME" ]] || exit 0
FM_DESC=$(awk '/^---/ && !in_fm { in_fm=1; next } /^---/ && in_fm { exit } in_fm && /^description:/ { sub(/^description:[[:space:]]*/, ""); print; exit }' "$NORM")
FM_TYPE=$(awk '/^---/ && !in_fm { in_fm=1; next } /^---/ && in_fm { exit } in_fm && /^type:/ { sub(/^type:[[:space:]]*/, ""); print; exit }' "$NORM")
[[ -n "$FM_TYPE" ]] || FM_TYPE="feedback"
BODY=$(awk '/^---/ && !in_fm { in_fm=1; next } /^---/ && in_fm { past=1; next } past { print }' "$NORM")

# Locate repo root from this hook's path.
HOOK_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$HOOK_DIR/../.." && pwd)"

# Load token from env or known per-host env files. Do not print the token.
if [[ -z "${PMD_HTTP_TOKEN:-}" ]]; then
  for env_file in "${PMD_ENV_FILE:-}" /srv/brehon-fork/.env "$REPO_ROOT/.env"; do
    [[ -n "$env_file" && -f "$env_file" ]] || continue
    # shellcheck disable=SC1090
    set -a; source "$env_file"; set +a
    break
  done
fi
TOKEN="${PMD_HTTP_TOKEN:-}"
AUTH=()
[[ -n "$TOKEN" ]] && AUTH=(-H "Authorization: Bearer $TOKEN")

# Candidate endpoints support: explicit env, HTTP .mcp.json, Mac/P50 loopback,
# and EliteDesk→P50 Tailscale URL used by the Junior daemon.
candidates=()
[[ -n "${PMD_HTTP_URL:-}" ]] && candidates+=("$PMD_HTTP_URL")
if [[ -f "$REPO_ROOT/.mcp.json" ]]; then
  mcp_url=$(jq -r '(.mcpServers // .)["project-memory"].url // empty' "$REPO_ROOT/.mcp.json" 2>/dev/null || true)
  [[ -n "$mcp_url" ]] && candidates+=("$mcp_url")
fi
candidates+=("http://localhost:11435/mcp" "http://100.104.171.26:11435/mcp")

PMD_URL=""
seen=""
for url in "${candidates[@]}"; do
  [[ -n "$url" ]] || continue
  case " $seen " in *" $url "*) continue ;; esac
  seen="$seen $url"
  code=$(curl -s -m2 -o /dev/null -w '%{http_code}' "$url" "${AUTH[@]}" 2>/dev/null || true)
  if [[ "$code" =~ ^(2|3|4)[0-9][0-9]$ ]]; then
    PMD_URL="$url"
    break
  fi
done

if [[ -z "$PMD_URL" ]]; then
  echo "lesson-pmd-sync: PMD HTTP unreachable — $(basename "$NORM") not auto-synced"
  exit 0
fi

curl_base=(-s -m 10 -X POST "$PMD_URL" -H "Content-Type: application/json" -H "Accept: application/json, text/event-stream")
[[ -n "$TOKEN" ]] && curl_base+=(-H "Authorization: Bearer $TOKEN")

headers=$(mktemp /tmp/pi-lesson-pmd-headers.XXXXXX)
init='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"pi-lesson-pmd-sync","version":"1.0"}}}'
curl "${curl_base[@]}" --dump-header "$headers" -d "$init" >/tmp/pi-lesson-pmd-init.$$ 2>/dev/null || true
SESSION_ID=$(grep -i '^mcp-session-id:' "$headers" | awk '{print $2}' | tr -d '\r' || true)
rm -f "$headers" /tmp/pi-lesson-pmd-init.$$
[[ -n "$SESSION_ID" ]] || { echo "lesson-pmd-sync: could not obtain MCP session-id — $(basename "$NORM") not auto-synced"; exit 0; }

mcp_call() {
  local tool="$1" args="$2" payload
  payload=$(LP_TOOL="$tool" LP_ARGS="$args" python3 - <<'PY'
import json, os
print(json.dumps({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":os.environ["LP_TOOL"],"arguments":json.loads(os.environ["LP_ARGS"])}}))
PY
)
  curl "${curl_base[@]}" -H "Mcp-Session-Id: $SESSION_ID" -d "$payload" 2>/dev/null
}

BASENAME=$(basename "$NORM")
LESSON_FILE_PATH=".claude/lessons/$BASENAME"
TAGS="lesson,$FM_TYPE"
SOURCE_REF=$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)

filectx_args=$(LP_FILE="$LESSON_FILE_PATH" python3 - <<'PY'
import json, os
print(json.dumps({"file_path": os.environ["LP_FILE"]}))
PY
)
filectx_resp=$(mcp_call "memory_get_file_context" "$filectx_args")
existing_id=$(LP_RESP="$filectx_resp" LP_FILE="$LESSON_FILE_PATH" python3 - <<'PY'
import json, os
resp=os.environ.get('LP_RESP',''); target=os.environ.get('LP_FILE',''); ids=[]
for line in resp.splitlines():
    line=line.strip()
    if line.startswith('data:'): line=line[5:].strip()
    if not line.startswith('{'): continue
    try: obj=json.loads(line)
    except Exception: continue
    result=obj.get('result')
    try:
        text=result['content'][0]['text']; rows=json.loads(text)
    except Exception:
        rows=result if isinstance(result,list) else []
    for r in rows:
        if isinstance(r,dict) and r.get('file_path')==target and r.get('id') is not None:
            try: ids.append(int(r['id']))
            except Exception: pass
print(min(ids) if ids else '')
PY
)

if [[ -n "$existing_id" ]]; then
  args=$(LP_ID="$existing_id" LP_TITLE="$FM_NAME" LP_DESC="$FM_DESC" LP_BODY="$BODY" LP_TAGS="$TAGS" python3 - <<'PY'
import json, os
full=((os.environ.get('LP_DESC','')+'\n\n'+os.environ.get('LP_BODY','')).strip())
print(json.dumps({"id": int(os.environ['LP_ID']), "title": os.environ['LP_TITLE'], "content": full, "tags": os.environ.get('LP_TAGS','lesson')}))
PY
)
  mcp_call "memory_update" "$args" >/tmp/pi-lesson-pmd-sync.$$ || true
  echo "lesson-pmd-sync: $BASENAME → updated PMD memory ID $existing_id"
else
  args=$(LP_TITLE="$FM_NAME" LP_DESC="$FM_DESC" LP_BODY="$BODY" LP_FILE="$LESSON_FILE_PATH" LP_SREF="$SOURCE_REF" LP_TAGS="$TAGS" python3 - <<'PY'
import json, os
full=((os.environ.get('LP_DESC','')+'\n\n'+os.environ.get('LP_BODY','')).strip())
print(json.dumps({"memory_type":"pattern", "title": os.environ['LP_TITLE'], "content": full, "file_path": os.environ['LP_FILE'], "source_ref": os.environ['LP_SREF'], "source_type":"lesson-import", "tags": os.environ.get('LP_TAGS','lesson'), "importance":3}))
PY
)
  mcp_call "memory_write" "$args" >/tmp/pi-lesson-pmd-sync.$$ || true
  echo "lesson-pmd-sync: $BASENAME → wrote/queued PMD memory via $PMD_URL"
fi
rm -f /tmp/pi-lesson-pmd-sync.$$
exit 0
