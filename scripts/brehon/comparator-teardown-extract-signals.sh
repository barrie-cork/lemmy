#!/usr/bin/env bash
# comparator-teardown-extract-signals.sh — SSH to the daemon host, run
# comparator-token-extract.sh on the challenger trace, and copy
# token-signals.json to the local results dir.
#
# Problem solved: trace.jsonl is ~276MB on the EliteDesk — not practical
# to copy to the laptop. This script pre-extracts the token signals on the
# daemon and brings back only the small JSON file (~1KB), enabling
# comparator-judge.sh to run fully from the laptop without needing the trace.
#
# Usage:
#   comparator-teardown-extract-signals.sh \
#     --experiment <id>                          e.g. planning-001
#     [--daemon-host <ssh-alias>]                default: homeserver
#     [--daemon-repo <path>]                     default: /srv/brehon-fork
#     [--runs-dir <local-dir>]                   default: .claude/PRPs/comparator/runs/
#     [--dry-run]                                print what would run, don't execute
#
# Outputs:
#   <runs-dir>/<experiment>/judge/token-signals.json   (local copy, ready for judge)
#
# Call timing: after the challenger arm finishes (or crashes), before running
# comparator-judge.sh. Safe to call multiple times (idempotent — overwrites
# token-signals.json with freshest extraction).

set -euo pipefail

EXPERIMENT=""
DAEMON_HOST="homeserver"
DAEMON_REPO="/srv/brehon-fork"
RUNS_DIR=""
DRY_RUN=false

while [[ $# -gt 0 ]]; do
  case "$1" in
    --experiment)   EXPERIMENT="$2";   shift 2;;
    --daemon-host)  DAEMON_HOST="$2";  shift 2;;
    --daemon-repo)  DAEMON_REPO="$2";  shift 2;;
    --runs-dir)     RUNS_DIR="$2";     shift 2;;
    --dry-run)      DRY_RUN=true;      shift;;
    -h|--help)      sed -n '1,25p' "$0"; exit 0;;
    *) echo "ERROR: unknown arg: $1" >&2; exit 2;;
  esac
done

if [[ -z "${EXPERIMENT}" ]]; then
  echo "ERROR: --experiment is required" >&2; exit 2
fi

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
RUNS_DIR="${RUNS_DIR:-${REPO_ROOT}/.claude/PRPs/comparator/runs}"

LOCAL_EXP_DIR="${RUNS_DIR}/${EXPERIMENT}"
LOCAL_JUDGE_DIR="${LOCAL_EXP_DIR}/judge"
LOCAL_SIGNALS="${LOCAL_JUDGE_DIR}/token-signals.json"

# Daemon-side paths.
DAEMON_EXP_DIR="${DAEMON_REPO}/.claude/PRPs/comparator/runs/${EXPERIMENT}"
DAEMON_CHALLENGER_DIR="${DAEMON_EXP_DIR}/challenger"
DAEMON_TRACE="${DAEMON_CHALLENGER_DIR}/trace.jsonl"
DAEMON_META="${DAEMON_CHALLENGER_DIR}/meta.json"
DAEMON_TOKEN_EXTRACT="${DAEMON_REPO}/scripts/brehon/comparator-token-extract.sh"
DAEMON_TMP_SIGNALS="/tmp/comparator-${EXPERIMENT}-token-signals-$$.json"

echo "comparator-teardown-extract-signals:"
echo "  experiment:      ${EXPERIMENT}"
echo "  daemon:          ${DAEMON_HOST}:${DAEMON_REPO}"
echo "  trace path:      ${DAEMON_TRACE}"
echo "  local output:    ${LOCAL_SIGNALS}"

if [[ "${DRY_RUN}" == true ]]; then
  echo ""
  echo "  [dry-run] would run on daemon:"
  echo "    ssh ${DAEMON_HOST} \"bash ${DAEMON_TOKEN_EXTRACT} ${DAEMON_TRACE} --meta ${DAEMON_META} > ${DAEMON_TMP_SIGNALS}\""
  echo "  [dry-run] would copy to local:"
  echo "    scp ${DAEMON_HOST}:${DAEMON_TMP_SIGNALS} ${LOCAL_SIGNALS}"
  exit 0
fi

# Step 1: verify trace exists on daemon.
echo "  checking trace exists on daemon..."
if ! ssh "${DAEMON_HOST}" "test -f '${DAEMON_TRACE}'" 2>/dev/null; then
  echo "ERROR: trace not found on daemon: ${DAEMON_HOST}:${DAEMON_TRACE}" >&2
  echo "  hint: check the experiment ran and produced output" >&2
  exit 2
fi

# Step 2: get trace size for informational display.
TRACE_SIZE="$(ssh "${DAEMON_HOST}" "wc -c < '${DAEMON_TRACE}' 2>/dev/null || echo 0" 2>/dev/null || echo "?")"
echo "  trace size on daemon: ${TRACE_SIZE} bytes"

# Step 3: run comparator-token-extract.sh on the daemon.
echo "  running token extraction on daemon..."
META_FLAG=""
if ssh "${DAEMON_HOST}" "test -f '${DAEMON_META}'" 2>/dev/null; then
  META_FLAG="--meta ${DAEMON_META}"
fi

# shellcheck disable=SC2029
ssh "${DAEMON_HOST}" "
  set -euo pipefail
  cd '${DAEMON_REPO}'
  bash '${DAEMON_TOKEN_EXTRACT}' '${DAEMON_TRACE}' ${META_FLAG} > '${DAEMON_TMP_SIGNALS}'
  echo \"  signals written: \$(wc -c < '${DAEMON_TMP_SIGNALS}') bytes\"
"

# Step 4: copy the signals file back to the local results dir.
mkdir -p "${LOCAL_JUDGE_DIR}"
echo "  copying token-signals.json from daemon..."
scp "${DAEMON_HOST}:${DAEMON_TMP_SIGNALS}" "${LOCAL_SIGNALS}"

# Step 5: clean up daemon temp file.
ssh "${DAEMON_HOST}" "rm -f '${DAEMON_TMP_SIGNALS}'" 2>/dev/null || true

echo "  token-signals.json extracted: ${LOCAL_SIGNALS}"
echo ""
echo "  Next step: run comparator-judge.sh --experiment ${EXPERIMENT} (token-signals.json is ready)"
