#!/usr/bin/env bash
# scripts/brehon/dq-lint-durations.sh — flag DQ entries where resolved_at < timestamp.
# Exits 0 if no negative-duration entries; non-zero with a list otherwise.
# Composite-id-aware (schema-v3): reports by `id` verbatim.
#
set -euo pipefail
DQ_PATH="${1:-.claude/decision-queue.json}"
[ -f "$DQ_PATH" ] || { echo "FATAL: not found: $DQ_PATH" >&2; exit 2; }

python3 -c "
import json, sys
from datetime import datetime
def parse(t):
    if not t: return None
    return datetime.fromisoformat(t.replace('Z', '+00:00'))
with open('$DQ_PATH') as f:
    dq = json.load(f)
bad = []
for arr in ('pending', 'resolved'):
    for e in dq.get(arr, []):
        ts = parse(e.get('timestamp'))
        ra = parse(e.get('resolved_at'))
        if ts and ra and ra < ts:
            delta = ts - ra
            bad.append((str(e.get('id')), e.get('timestamp'), e.get('resolved_at'), str(delta)))
for (eid, ts, ra, d) in bad:
    print(f'DQ-LINT FAIL: entry \"{eid}\" has resolved_at ({ra}) earlier than timestamp ({ts}) by {d}')
sys.exit(1 if bad else 0)
"
