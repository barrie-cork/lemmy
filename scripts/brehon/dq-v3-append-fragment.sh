#!/usr/bin/env bash
# dq-v3-append-fragment.sh — append a JSON DQ entry fragment to
# .claude/decision-queue.json with a freshly-generated v3 composite id.
#
# Replaces the inline `python -c "..."` / heredoc merge pattern. Authoring
# the fragment via the Write tool means backslash paths, embedded quotes,
# and multi-line content carry across into JSON without shell or Python
# string-escaping. The script:
#
#   1. Calls dq-v3-new-entry.sh to mint a fresh <session>-<seq> id.
#   2. Reads <fragment.json> (a single entry object — NOT an array).
#   3. Injects the fresh id into the fragment.
#   4. Reads .claude/decision-queue.json, appends the fragment to
#      resolved[] (default) or pending[] (with --pending).
#   5. Writes the file back with json.dump(ensure_ascii=False, indent=2).
#   6. Prints the injected id to stdout for the caller's commit message.
#
# What the script does NOT do:
#   - Commit or push (caller's job — atomic-protocol per multi-lane rule).
#   - Validate entry schema beyond "is parseable JSON object."
#   - Touch the session-id cache (dq-v3-new-entry.sh owns that).
#   - Remove the fragment file (caller decides; rm is destructive).
#
# Usage:
#   bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> [--pending]
#
# Examples:
#   # resolved entry (default — kind: "log", advisor self-resolved)
#   bash scripts/brehon/dq-v3-append-fragment.sh .claude/dq-fragment.json
#
#   # pending entry (blocker awaiting reply, or validate-pending raise)
#   bash scripts/brehon/dq-v3-append-fragment.sh .claude/dq-fragment.json --pending
#
# Output:
#   stdout: composite id of the appended entry, e.g. "a1b2c3d4e5f6-008"
#   exit 0 on success, non-zero on error.
#
# Exit codes:
#   0  appended successfully
#   1  fragment file missing or not valid JSON
#   2  fragment is not a JSON object (e.g. accidentally an array)
#   3  decision-queue.json missing or not valid JSON
#   4  dq-v3-new-entry.sh failed
#   5  --pending flag passed with unrecognised value
#
# See also:
#   - scripts/brehon/dq-v3-new-entry.sh — id generation (always called)
#   - .claude/rules/decision-queue.md  — schema, hard refusals, attribution
#   - .claude/lessons/feedback_windows_backslash_path_dq_via_write_fragment.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

export PYTHONIOENCODING=utf-8
export PYTHONUTF8=1

# Argument parsing.
FRAGMENT=""
TARGET_ARRAY="resolved"
while [ $# -gt 0 ]; do
  case "$1" in
    --pending)
      TARGET_ARRAY="pending"
      shift
      ;;
    --resolved)
      TARGET_ARRAY="resolved"
      shift
      ;;
    -h|--help)
      sed -n '2,40p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    -*)
      echo "error: unknown flag '$1'" >&2
      exit 5
      ;;
    *)
      if [ -z "${FRAGMENT}" ]; then
        FRAGMENT="$1"
      else
        echo "error: only one fragment path accepted (got '$1' after '${FRAGMENT}')" >&2
        exit 5
      fi
      shift
      ;;
  esac
done

if [ -z "${FRAGMENT}" ]; then
  echo "error: fragment path required" >&2
  echo "usage: bash $0 <fragment.json> [--pending]" >&2
  exit 1
fi

if [ ! -f "${FRAGMENT}" ]; then
  echo "error: fragment file not found: ${FRAGMENT}" >&2
  exit 1
fi

DQ_FILE="${REPO_ROOT}/.claude/decision-queue.json"
if [ ! -f "${DQ_FILE}" ]; then
  echo "error: decision-queue.json not found: ${DQ_FILE}" >&2
  exit 3
fi

# Mint a fresh id. Capture stderr too so script failure surfaces.
NEW_ID="$(bash "${SCRIPT_DIR}/dq-v3-new-entry.sh")" || {
  echo "error: dq-v3-new-entry.sh failed" >&2
  exit 4
}

# Now merge. Python script reads three things and writes one thing:
#   stdin args: DQ_FILE, FRAGMENT, NEW_ID, TARGET_ARRAY
python3 - "${DQ_FILE}" "${FRAGMENT}" "${NEW_ID}" "${TARGET_ARRAY}" <<'PY'
import io, json, sys

dq_path, frag_path, new_id, target_array = sys.argv[1:5]
sys.stdout.reconfigure(newline='\n')

# Load fragment first — bail before touching the DQ on bad input.
try:
    with io.open(frag_path, encoding='utf-8') as f:
        fragment = json.load(f)
except json.JSONDecodeError as e:
    print(f"error: fragment is not valid JSON: {e}", file=sys.stderr)
    sys.exit(1)

if not isinstance(fragment, dict):
    print(f"error: fragment must be a JSON object, got {type(fragment).__name__}", file=sys.stderr)
    sys.exit(2)

# Load DQ.
try:
    with io.open(dq_path, encoding='utf-8') as f:
        dq = json.load(f)
except json.JSONDecodeError as e:
    print(f"error: decision-queue.json is not valid JSON: {e}", file=sys.stderr)
    sys.exit(3)

# Inject id (overwrites any pre-existing 'id' in the fragment — caller's
# fragment doesn't need to know which id will be assigned).
fragment['id'] = new_id

# Ensure target array exists; append.
if target_array not in dq:
    dq[target_array] = []
dq[target_array].append(fragment)

# Write back. ensure_ascii=False preserves UTF-8; indent=2 matches existing
# file style.
with io.open(dq_path, 'w', encoding='utf-8', newline='\n') as f:
    json.dump(dq, f, ensure_ascii=False, indent=2)
    f.write('\n')

# Print the id so the caller can use it in commit message / runlog.
print(new_id)
PY
