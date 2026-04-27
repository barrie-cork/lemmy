#!/usr/bin/env bash
# PostToolUse hook: remind agent to read back files after 5+ edits.
# Advisory only — outputs additionalContext, does NOT block.
#
# Event: PostToolUse
# Matcher: Edit|Read
# Timeout: 5000
# Tracks edit count per session in /tmp/cc-edit-count-$PPID.
# Resets counter when a Read tool call is detected.

set -o pipefail

COUNTER_FILE="/tmp/cc-edit-count-${PPID}"
THRESHOLD=5

INPUT=$(cat)
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null) || exit 0

case "$TOOL_NAME" in
  Read)
    # Reset counter on Read — agent is reading back files
    if [ -f "$COUNTER_FILE" ]; then
      rm -f "$COUNTER_FILE"
    fi
    exit 0
    ;;
  Edit)
    # Increment edit counter
    COUNT=0
    if [ -f "$COUNTER_FILE" ]; then
      COUNT=$(cat "$COUNTER_FILE" 2>/dev/null || echo 0)
    fi
    COUNT=$((COUNT + 1))
    echo "$COUNT" > "$COUNTER_FILE"

    # Only remind at exactly the threshold (not every edit after)
    if [ "$COUNT" -eq "$THRESHOLD" ]; then
      echo "You've made ${THRESHOLD}+ file edits without reading any back. Spot-check 2-3 modified files with the Read tool to catch silent edit failures (wrong match, partial replacement)."
    fi
    exit 0
    ;;
  *)
    exit 0
    ;;
esac
