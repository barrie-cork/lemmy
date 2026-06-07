#!/usr/bin/env bash
# minimax-api.sh — thin wrapper around the MiniMax native chat endpoint.
#
# Per .claude/PRPs/handovers/comparator-minimax-auth-2026-06-07.md:
# Use POST https://api.minimax.io/v1/text/chatcompletion_v2 (native endpoint).
# The /anthropic compat shim returns spurious 1008 even with valid balance — avoid it.
#
# Usage:
#   minimax-api.sh [options] < prompt-file.txt
#   minimax-api.sh [options] --message "inline message"
#
# Options:
#   --model <id>           default: MiniMax-M3
#   --max-tokens <n>       default: 8192 (reasoning eats budget first — give enough)
#   --temperature <f>      default: 0.0 (deterministic, good for judges)
#   --system <text>        optional system prompt
#   --message <text>       inline user message (mutually exclusive with --message-file / stdin)
#   --message-file <path>  read user message from file (preferred for large prompts)
#   --output <file>        write full response JSON here (default: stdout)
#   --reason-file <file>   if set, write reasoning_content to this file separately
#   --quiet                suppress progress to stderr
#   --env-file <path>      source this file for MINIMAX_API_KEY (default: .env in CWD)
#
# Exit codes:
#   0  — success (HTTP 200, finish_reason != "error")
#   1  — API error (non-200, or finish_reason "error")
#   2  — argument error
#   3  — auth missing (MINIMAX_API_KEY not set)
#
# Key contract: NEVER echo MINIMAX_API_KEY to stdout or stderr. The process-table
# leak guard in the auto-mode classifier blocks inline key exposure.

set -euo pipefail

MODEL="MiniMax-M3"
MAX_TOKENS=8192
TEMPERATURE="0.0"
SYSTEM_PROMPT=""
MESSAGE=""
MESSAGE_FILE=""
OUTPUT_FILE=""
REASON_FILE=""
QUIET=false
ENV_FILE=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --model)       MODEL="$2";         shift 2;;
    --max-tokens)  MAX_TOKENS="$2";    shift 2;;
    --temperature) TEMPERATURE="$2";   shift 2;;
    --system)      SYSTEM_PROMPT="$2"; shift 2;;
    --message)      MESSAGE="$2";      shift 2;;
    --message-file) MESSAGE_FILE="$2"; shift 2;;
    --output)      OUTPUT_FILE="$2";   shift 2;;
    --reason-file) REASON_FILE="$2";   shift 2;;
    --quiet)       QUIET=true;         shift;;
    --env-file)    ENV_FILE="$2";      shift 2;;
    -h|--help)     sed -n '1,20p' "$0"; exit 0;;
    *) echo "ERROR: unknown arg: $1" >&2; exit 2;;
  esac
done

# Source env file for MINIMAX_API_KEY (never pass key on command line).
if [[ -n "${ENV_FILE}" ]]; then
  if [[ ! -f "${ENV_FILE}" ]]; then
    echo "ERROR: env-file not found: ${ENV_FILE}" >&2; exit 2
  fi
  # shellcheck source=/dev/null
  set -a; source "${ENV_FILE}"; set +a
elif [[ -f ".env" && -z "${MINIMAX_API_KEY:-}" ]]; then
  set -a; source ".env"; set +a
fi

if [[ -z "${MINIMAX_API_KEY:-}" ]]; then
  echo "ERROR: MINIMAX_API_KEY not set. Source a .env file or set the variable." >&2
  exit 3
fi

# Resolve message source: --message-file > --message > stdin.
TMP_REQ="$(mktemp)"
TMP_RESP="$(mktemp)"
TMP_MSG_OWN=""  # only set if WE created the temp file (so we delete it)
trap 'rm -f "${TMP_REQ}" "${TMP_RESP}" "${TMP_MSG_OWN}"' EXIT

MSG_SOURCE_FILE=""
if [[ -n "${MESSAGE_FILE}" ]]; then
  if [[ ! -f "${MESSAGE_FILE}" ]]; then
    echo "ERROR: --message-file not found: ${MESSAGE_FILE}" >&2; exit 2
  fi
  MSG_SOURCE_FILE="${MESSAGE_FILE}"  # caller's file; we do NOT delete it
elif [[ -n "${MESSAGE}" ]]; then
  TMP_MSG_OWN="$(mktemp)"
  printf '%s' "${MESSAGE}" > "${TMP_MSG_OWN}"
  MSG_SOURCE_FILE="${TMP_MSG_OWN}"
elif [[ ! -t 0 ]]; then
  TMP_MSG_OWN="$(mktemp)"
  cat > "${TMP_MSG_OWN}"
  MSG_SOURCE_FILE="${TMP_MSG_OWN}"
else
  echo "ERROR: no --message, --message-file, and no stdin" >&2; exit 2
fi

MSG_SIZE="$(wc -c < "${MSG_SOURCE_FILE}")"
if [[ "${MSG_SIZE}" -eq 0 ]]; then
  echo "ERROR: empty message" >&2; exit 2
fi

[[ "${QUIET}" == false ]] && echo "  minimax-api: model=${MODEL} max_tokens=${MAX_TOKENS} msg_bytes=${MSG_SIZE}" >&2

# Build the request JSON entirely in Python (no shell interpolation of large strings).
python3 - "${MSG_SOURCE_FILE}" "${TMP_REQ}" "${MODEL}" "${MAX_TOKENS}" "${TEMPERATURE}" "${SYSTEM_PROMPT:-}" <<'PYEOF'
import json, sys

msg_file, req_file, model, max_tokens, temperature, system_prompt = \
    sys.argv[1], sys.argv[2], sys.argv[3], int(sys.argv[4]), float(sys.argv[5]), sys.argv[6]

with open(msg_file, encoding='utf-8', errors='replace') as f:
    user_msg = f.read()

messages = []
if system_prompt:
    messages.append({"role": "system", "content": system_prompt})
messages.append({"role": "user", "content": user_msg})

request = {
    "model": model,
    "messages": messages,
    "max_tokens": max_tokens,
    "temperature": temperature,
}
with open(req_file, 'w', encoding='utf-8') as f:
    json.dump(request, f)
PYEOF

# Call the native endpoint (handover §TL;DR: use chatcompletion_v2, not /anthropic).
HTTP_STATUS="$(curl -s -w "%{http_code}" \
  -o "${TMP_RESP}" \
  -X POST "https://api.minimax.io/v1/text/chatcompletion_v2" \
  -H "Authorization: Bearer ${MINIMAX_API_KEY}" \
  -H "Content-Type: application/json" \
  --data-binary "@${TMP_REQ}")"

if [[ "${HTTP_STATUS}" != "200" ]]; then
  echo "ERROR: MiniMax API returned HTTP ${HTTP_STATUS}" >&2
  cat "${TMP_RESP}" >&2
  exit 1
fi

# Check for API-level error + extract reasoning_content in one Python pass.
# Use positional args (not shell expansion inside -c string) to avoid subshell trap.
TMP_EXTRACT="$(mktemp)"
python3 - "${TMP_RESP}" "${REASON_FILE:-}" "${TMP_EXTRACT}" <<'PYEOF'
import json, sys

resp_path, reason_file, extract_path = sys.argv[1], sys.argv[2], sys.argv[3]

try:
    with open(resp_path, encoding='utf-8') as f:
        d = json.load(f)
    choices = d.get('choices', [{}])
    finish_reason = choices[0].get('finish_reason', 'unknown') if choices else 'no_choices'
    msg = choices[0].get('message', {}) if choices else {}
    reasoning = msg.get('reasoning_content', '')
except Exception as e:
    finish_reason = f'parse_error: {e}'
    reasoning = ''

with open(extract_path, 'w') as f:
    f.write(finish_reason)

if reason_file:
    with open(reason_file, 'w', encoding='utf-8') as f:
        f.write(reasoning)
PYEOF

FINISH_REASON="$(cat "${TMP_EXTRACT}")"
rm -f "${TMP_EXTRACT}"

if [[ "${FINISH_REASON}" == "error" || "${FINISH_REASON}" == parse_error* ]]; then
  echo "ERROR: API finish_reason=${FINISH_REASON}" >&2
  cat "${TMP_RESP}" >&2
  exit 1
fi

[[ "${QUIET}" == false ]] && echo "  minimax-api: finish_reason=${FINISH_REASON}" >&2

# Output the full response JSON.
if [[ -n "${OUTPUT_FILE}" ]]; then
  cp "${TMP_RESP}" "${OUTPUT_FILE}"
  [[ "${QUIET}" == false ]] && echo "  minimax-api: response written to ${OUTPUT_FILE}" >&2
else
  cat "${TMP_RESP}"
fi
