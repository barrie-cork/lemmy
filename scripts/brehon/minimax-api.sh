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
#   --message <text>       inline user message (mutually exclusive with stdin)
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
    --message)     MESSAGE="$2";       shift 2;;
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

# Read message from stdin if --message not given.
if [[ -z "${MESSAGE}" ]]; then
  if [[ -t 0 ]]; then
    echo "ERROR: no --message and no stdin" >&2; exit 2
  fi
  MESSAGE="$(cat)"
fi

if [[ -z "${MESSAGE}" ]]; then
  echo "ERROR: empty message" >&2; exit 2
fi

[[ "${QUIET}" == false ]] && echo "  minimax-api: model=${MODEL} max_tokens=${MAX_TOKENS}" >&2

# Build the messages array. Include system message if given.
if [[ -n "${SYSTEM_PROMPT}" ]]; then
  MESSAGES_JSON="[{\"role\":\"system\",\"content\":$(printf '%s' "${SYSTEM_PROMPT}" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))')},{\"role\":\"user\",\"content\":$(printf '%s' "${MESSAGE}" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))')}]"
else
  MESSAGES_JSON="[{\"role\":\"user\",\"content\":$(printf '%s' "${MESSAGE}" | python3 -c 'import sys,json; print(json.dumps(sys.stdin.read()))')}]"
fi

REQUEST_JSON="{\"model\":\"${MODEL}\",\"messages\":${MESSAGES_JSON},\"max_tokens\":${MAX_TOKENS},\"temperature\":${TEMPERATURE}}"

# Write request to a temp file to avoid argument-length limits and key exposure.
TMP_REQ="$(mktemp)"
TMP_RESP="$(mktemp)"
trap 'rm -f "${TMP_REQ}" "${TMP_RESP}"' EXIT

printf '%s' "${REQUEST_JSON}" > "${TMP_REQ}"

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

# Check for API-level error in the response.
FINISH_REASON="$(python3 -c "
import sys, json
try:
    d = json.load(open('${TMP_RESP}'))
    print(d.get('choices', [{}])[0].get('finish_reason', 'unknown'))
except Exception as e:
    print('parse_error: ' + str(e))
" 2>/dev/null || echo "parse_error")"

if [[ "${FINISH_REASON}" == "error" || "${FINISH_REASON}" == parse_error* ]]; then
  echo "ERROR: API finish_reason=${FINISH_REASON}" >&2
  cat "${TMP_RESP}" >&2
  exit 1
fi

[[ "${QUIET}" == false ]] && echo "  minimax-api: finish_reason=${FINISH_REASON}" >&2

# Extract reasoning_content to a separate file if requested.
if [[ -n "${REASON_FILE}" ]]; then
  python3 -c "
import json, sys
d = json.load(open('${TMP_RESP}'))
rc = d.get('choices', [{}])[0].get('message', {}).get('reasoning_content', '')
with open('${REASON_FILE}', 'w') as f:
    f.write(rc)
" 2>/dev/null || true
fi

# Output the full response JSON.
if [[ -n "${OUTPUT_FILE}" ]]; then
  cp "${TMP_RESP}" "${OUTPUT_FILE}"
  [[ "${QUIET}" == false ]] && echo "  minimax-api: response written to ${OUTPUT_FILE}" >&2
else
  cat "${TMP_RESP}"
fi
