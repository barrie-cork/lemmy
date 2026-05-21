#!/usr/bin/env bash
# dq-v3-new-entry.sh — generate a schema-v3 composite DQ id.
#
# Returns a fresh composite id of the form "<session_id>-<seq>" where:
#   session_id  12-char hex UUID prefix, cached in .claude/.dq-session-id
#               (gitignored). Generated once per CC session; subsequent
#               calls within the same session reuse the cached value.
#   seq         per-session monotonic 3-digit counter (001, 002, ...).
#               Scanned from live DQ + all archive files.
#
# Usage:
#   NEXT_ID="$(bash scripts/brehon/dq-v3-new-entry.sh)"
#
# Output:
#   stdout: composite id, e.g. "a1b2c3d4e5f6-001"
#   exit 0 on success, non-zero on error.
#
# Cache contract: .claude/.dq-session-id is a 12-char hex string with no
# trailing newline. Subsequent calls within the same session reuse the
# cached value — scans always pick up entries already written this session.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"

# UTF-8 for Python (Windows defaults to cp1252).
export PYTHONIOENCODING=utf-8
export PYTHONUTF8=1

SESSION_ID_FILE="${REPO_ROOT}/.claude/.dq-session-id"

# Ensure session-id cache exists (no trailing newline per contract).
if [ ! -f "${SESSION_ID_FILE}" ]; then
  python3 -c "import uuid; print(uuid.uuid4().hex[:12], end='')" > "${SESSION_ID_FILE}"
fi

SESSION_ID="$(cat "${SESSION_ID_FILE}")"

# Validate: must be exactly 12 lowercase hex chars.
if ! echo "${SESSION_ID}" | grep -qE '^[a-f0-9]{12}$'; then
  echo "error: .dq-session-id contains invalid value: '${SESSION_ID}'" >&2
  exit 1
fi

# Scan live DQ + all archives for entries whose id starts with
# "<session_id>-". Pick max(seq) + 1, zero-padded to 3 digits.
python3 - "${REPO_ROOT}" "${SESSION_ID}" <<'PY'
import io, json, sys, re, glob

repo_root, session_id = sys.argv[1], sys.argv[2]
sys.stdout.reconfigure(newline='\n')

prefix = session_id + '-'
pattern = re.compile(r'^[a-f0-9]+-(\d{3})$')
max_seq = 0

files = [f"{repo_root}/.claude/decision-queue.json"]
files += sorted(glob.glob(f"{repo_root}/.claude/decision-queue-archive-*.json"))

for fpath in files:
    try:
        with io.open(fpath, encoding='utf-8') as f:
            data = json.load(f)
    except (FileNotFoundError, json.JSONDecodeError):
        continue
    for arr in ('pending', 'resolved'):
        for entry in data.get(arr, []):
            eid = str(entry.get('id', ''))
            if eid.startswith(prefix):
                m = pattern.match(eid)
                if m:
                    seq = int(m.group(1))
                    if seq > max_seq:
                        max_seq = seq

next_seq = max_seq + 1
print(f"{session_id}-{next_seq:03d}")
PY
