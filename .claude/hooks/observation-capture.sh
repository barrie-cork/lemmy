#!/usr/bin/env bash
# PostToolUse hook: shadow-mode event capture.
# Pattern-matches 4 detector rules and appends JSONL events to a per-PPID log
# under ~/.cache/tw-observations/. NEVER blocks, NEVER calls an LLM, NEVER
# writes to PMD. This is pure observation — precision will be measured after
# 7 days before wiring any downstream summarizer.
#
# Idea PMD #914, plan PMD #918 Phase 4.
#
# Event: PostToolUse
# Matcher: .*
# Timeout: 5000

set -o pipefail

LOG_DIR="${HOME}/.cache/tw-observations"
LOG_FILE="${LOG_DIR}/${PPID:-unknown}.jsonl"
mkdir -p "$LOG_DIR" 2>/dev/null || exit 0

INPUT=$(cat)
command -v jq >/dev/null 2>&1 || exit 0

TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // empty' 2>/dev/null)
[ -z "$TOOL_NAME" ] && exit 0

# Extract useful fields (may be empty depending on tool)
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty' 2>/dev/null)
FILE_PATH=$(echo "$INPUT" | jq -r '.tool_input.file_path // empty' 2>/dev/null)

# Privacy guard: drop the whole event if obvious secrets appear in the
# candidate fields. Err on the side of silence — better to lose a true
# positive than leak a token into shadow-mode logs.
SECRET_RE='(API[_-]?KEY|PASSWORD|TOKEN|SECRET|Bearer[[:space:]]|-p[[:space:]]*["'\''][^"'\'']{4,})'
if echo "$CMD $FILE_PATH" | grep -qE "$SECRET_RE"; then
  exit 0
fi

# Strip HEREDOC bodies from the command before pattern-matching. Claude
# Code's `git commit -m "$(cat <<'EOF' ... EOF)"` pattern embeds whole
# commit message bodies inside the Bash tool_input.command field, and
# those bodies frequently mention detector keywords ("docker compose up
# -d", "ssh homeserver") when describing past incidents. Without this
# sed pass, deploy-start precision collapses to 0% because every
# incident-postmortem commit fires a false deploy event.
# Deletes inclusively from the line containing `<<EOF`/`<<'EOF'` (with
# optional leading dash for `<<-EOF`) through the terminator on its own
# line. Chained commands after the heredoc close are preserved.
CMD_MATCH=$(echo "$CMD" | sed -E "/<<-?[[:space:]]*'?EOF'?/,/^[[:space:]]*EOF[[:space:]]*$/d")

DETECTOR=""
case "$TOOL_NAME" in
  Bash)
    if echo "$CMD_MATCH" | grep -qE 'docker[[:space:]]+compose[[:space:]]+up[[:space:]]+-d'; then
      DETECTOR="deploy-start"
    elif echo "$CMD_MATCH" | grep -qE '(sudo[[:space:]]+)?systemctl[[:space:]]+(restart|reload)'; then
      DETECTOR="service-restart"
    elif echo "$CMD_MATCH" | grep -qE '(^|[[:space:]])ssh[[:space:]]' \
      && echo "$CMD_MATCH" | grep -qE '(barrie@|homeserver($|[^a-zA-Z0-9_]))'; then
      DETECTOR="server-shell"
    fi
    ;;
  Edit|Write|MultiEdit)
    case "$FILE_PATH" in
      /srv/*) DETECTOR="server-file-write" ;;
    esac
    ;;
esac

[ -z "$DETECTOR" ] && exit 0

# Append one JSONL event. jq -c builds it atomically so a crash mid-write
# can't leave a half-line in the file.
jq -nc \
  --arg ts "$(date -u +%Y-%m-%dT%H:%M:%SZ)" \
  --arg host "$(hostname -s 2>/dev/null || echo unknown)" \
  --arg ppid "${PPID:-unknown}" \
  --arg detector "$DETECTOR" \
  --arg tool "$TOOL_NAME" \
  --arg cmd "$CMD" \
  --arg file "$FILE_PATH" \
  '{ts:$ts, host:$host, ppid:$ppid, detector:$detector, tool:$tool, cmd:$cmd, file:$file}' \
  >> "$LOG_FILE" 2>/dev/null || true

exit 0
