#!/bin/bash

# UserPromptSubmit hook — inject decision-queue + task-hopper state.
#
# Fires before every user prompt. Reads .claude/decision-queue.json and
# .claude/task-hopper.json, summarises any active blockers / escalations,
# and emits a one-line system reminder via additionalContext so coordination
# state is ambient instead of probed-for.
#
# Stays silent when nothing is pending — no noise added to the conversation
# in the steady state.

set -euo pipefail

DQ_FILE=".claude/decision-queue.json"
HOPPER_FILE=".claude/task-hopper.json"

if [[ ! -f "$DQ_FILE" ]] && [[ ! -f "$HOPPER_FILE" ]]; then
  exit 0
fi

SUMMARY=$(python3 - <<'PY' 2>/dev/null || echo ""
import json, os

parts = []

dq_path = ".claude/decision-queue.json"
hopper_path = ".claude/task-hopper.json"

if os.path.exists(dq_path):
    try:
        with open(dq_path, encoding="utf-8") as f:
            dq = json.load(f)
        pending = dq.get("pending", [])
        if pending:
            ids = ", ".join(f"#{p.get('id', '?')}" for p in pending[:3])
            more = "" if len(pending) <= 3 else f" (+{len(pending)-3} more)"
            parts.append(f"DQ pending: {len(pending)} [{ids}{more}]")
    except Exception:
        pass

if os.path.exists(hopper_path):
    try:
        with open(hopper_path, encoding="utf-8") as f:
            hopper = json.load(f)
        tasks = hopper.get("tasks", [])
        in_prog = [t for t in tasks if t.get("status") == "in_progress"]
        escalated = [t for t in tasks if t.get("status") == "escalated"]
        if in_prog:
            ids = ", ".join(t.get("id", "?") for t in in_prog[:3])
            parts.append(f"hopper in_progress: {len(in_prog)} [{ids}]")
        if escalated:
            ids = ", ".join(t.get("id", "?") for t in escalated[:3])
            parts.append(f"hopper escalated: {len(escalated)} [{ids}]")
    except Exception:
        pass

if parts:
    print(" | ".join(parts))
PY
)

if [[ -z "$SUMMARY" ]]; then
  exit 0
fi

python3 - "$SUMMARY" <<'PY'
import json, sys
summary = sys.argv[1]
print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "UserPromptSubmit",
        "additionalContext": f"Coordination state: {summary}"
    }
}))
PY

exit 0
