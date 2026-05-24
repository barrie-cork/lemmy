#!/usr/bin/env bash
# Stop hook: emit ONE role-signal row to PMD measuring what this Junior task
# actually read/used vs. what its role manifest loaded.
#
# Goal: feed /check-role-health with evidence for per-role context-management
# decisions. Strip candidates are rules loaded into a role's startup context
# that this hook NEVER observes the agent Reading explicitly across N tasks,
# and MCP servers whose tools are NEVER invoked.
#
# Event: Stop
# Timeout: 10000
#
# Exit code: 0 ALWAYS. Signals are advisory; this hook never blocks a Junior
# task completion. Stop hooks fire in PARALLEL per current Claude Code docs
# (https://code.claude.com/docs/en/hooks.md), so this hook is independent
# of retro-check.sh — no ordering dependency.
#
# Junior-only gate: the laptop advisor session also runs Stop hooks but is
# the CONSUMER of role signals, not the SUBJECT. We exit 0 early unless the
# current branch is junior/*, matching the existing retro-check.sh §"branch
# detection" pattern.
#
# Dependencies (paranoid — verified one-by-one, missing tool = exit 0):
#   git, jq, sqlite3 (read-only on PMD), python3 (stdin JSON parse),
#   node OR npx (write-role-signal CLI from project-memory-mcp).

set -euo pipefail

# --- Read Stop-hook stdin JSON (per code.claude.com/docs/en/hooks.md) ---
# Fields: session_id, transcript_path, cwd, permission_mode, hook_event_name.
STDIN_JSON=""
if [ ! -t 0 ]; then
  STDIN_JSON=$(cat || true)
fi

# Helper: extract one field from STDIN_JSON via python3 (jq's `-r '.field'`
# pattern would also work, but python3 is more reliably present across the
# Brehon two-machine surface; jq is checked separately below for blocker count).
extract_field() {
  local field="$1"
  if [ -z "$STDIN_JSON" ] || ! command -v python3 &>/dev/null; then
    echo ""
    return
  fi
  python3 -c "
import json, sys
try:
    d = json.loads(sys.argv[1])
    print(d.get(sys.argv[2], ''))
except Exception:
    pass
" "$STDIN_JSON" "$field" 2>/dev/null || true
}

SESSION_ID=$(extract_field "session_id")
TRANSCRIPT_PATH=$(extract_field "transcript_path")
CWD_FROM_STDIN=$(extract_field "cwd")

# --- Skip on non-Junior branches ---
if ! command -v git &>/dev/null; then
  exit 0
fi
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
if [[ ! "$CURRENT_BRANCH" =~ ^junior/ ]]; then
  exit 0
fi

# --- Resolve role from the CLAUDE_PROMPT env (dispatch line) ---
# Junior dispatch lines start with `[role:planning|impl-task|bm-task|ci-watcher] ...`
ROLE=""
PROMPT="${CLAUDE_PROMPT:-}"
if [[ "$PROMPT" =~ \[role:(planning|impl-task|bm-task|ci-watcher)\] ]]; then
  ROLE="${BASH_REMATCH[1]}"
fi
if [ -z "$ROLE" ]; then
  # No role tag in prompt — likely an ad-hoc Junior task, skip signal emission.
  exit 0
fi

# --- Resolve task_id ---
# Prefer session_id from stdin (most stable); fall back to branch suffix.
TASK_ID="${SESSION_ID:-}"
if [ -z "$TASK_ID" ]; then
  # branch shape: junior/<slug>-<id>
  TASK_ID="${CURRENT_BRANCH##*-}"
fi

# --- Resolve config_version (subtree SHA of .claude/roles/<role>) ---
# Per .claude/PRPs/handovers/role-customization-2026-05-24.md §4 T2:
# the subtree SHA pins the signal to the exact manifest state at task time.
#
# Quirk: `git rev-parse` on missing refs prints the literal ref to STDOUT
# (not stderr) and exits 128. So we can't rely on `2>/dev/null || echo`.
# Capture exit code explicitly and discard stdout on failure.
CONFIG_VERSION=""
_GR_OUT=$(git rev-parse "HEAD:.claude/roles/${ROLE}" 2>/dev/null) && CONFIG_VERSION="$_GR_OUT" || CONFIG_VERSION=""
if [ -z "$CONFIG_VERSION" ] || [[ "$CONFIG_VERSION" == HEAD:* ]]; then
  # Role substrate not present on this branch — pre-T2 history, or the
  # ref-not-found stdout leaked through. Mark as pre-t2.
  CONFIG_VERSION="pre-t2"
fi

# --- Parse transcript for rules Read + MCP tools invoked ---
RULES_READ_JSON="[]"
MCP_TOOLS_JSON="[]"
if [ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ] && command -v jq &>/dev/null; then
  # Transcript is JSONL; each line is one event. tool_use events carry the
  # tool name and (for Read) the file_path input.
  RULES_READ_JSON=$(jq -rcs '
    [.[]
      | select(.type? == "tool_use" and .name? == "Read")
      | .input?.file_path? // empty
      | select(test("\\.claude/rules/.*\\.md$"))
      | sub(".*\\.claude/rules/"; "")
    ] | unique
  ' "$TRANSCRIPT_PATH" 2>/dev/null || echo "[]")
  if [ -z "$RULES_READ_JSON" ] || [ "$RULES_READ_JSON" = "null" ]; then
    RULES_READ_JSON="[]"
  fi
  MCP_TOOLS_JSON=$(jq -rcs '
    [.[]
      | select(.type? == "tool_use")
      | .name? // empty
      | select(startswith("mcp__"))
    ] | unique
  ' "$TRANSCRIPT_PATH" 2>/dev/null || echo "[]")
  if [ -z "$MCP_TOOLS_JSON" ] || [ "$MCP_TOOLS_JSON" = "null" ]; then
    MCP_TOOLS_JSON="[]"
  fi
fi

# --- Count DQ blockers added on this branch vs governance-v0 base ---
# A blocker = an entry with kind:"blocker" appearing in pending[] or resolved[]
# on this branch that was NOT present on governance-v0 base.
DQ_BLOCKERS=0
if command -v jq &>/dev/null && [ -f .claude/decision-queue.json ]; then
  # Get the count of blocker entries currently on this branch.
  THIS_BRANCH_BLOCKERS=$(jq -r '
    ([.pending[]?, .resolved[]?] | map(select(.kind? == "blocker"))) | length
  ' .claude/decision-queue.json 2>/dev/null || echo 0)
  # Get the count from governance-v0 base (if reachable; falls back to 0).
  BASE_BLOCKERS=$(git show governance-v0:.claude/decision-queue.json 2>/dev/null | jq -r '
    ([.pending[]?, .resolved[]?] | map(select(.kind? == "blocker"))) | length
  ' 2>/dev/null || echo 0)
  # Delta — clamped to >=0 (a branch could have fewer if blockers were resolved+pruned).
  DQ_BLOCKERS=$((THIS_BRANCH_BLOCKERS - BASE_BLOCKERS))
  if [ "$DQ_BLOCKERS" -lt 0 ]; then
    DQ_BLOCKERS=0
  fi
fi

# --- Assemble content JSON ---
# Hand-built JSON to keep dependencies minimal. Field names match the consumer
# contract at /check-role-health (Task 4).
CONTENT_JSON=$(cat <<JSON
{"rules_read":${RULES_READ_JSON},"mcp_tools_invoked":${MCP_TOOLS_JSON},"dq_blockers_added":${DQ_BLOCKERS}}
JSON
)

# --- Emit via write-role-signal CLI ---
# Resolve CLI: prefer the package's `bin` entry via the canonical dist path.
CLI=""
DIST_PATH="C:/Users/barri/Developer/MCPs/project-memory-mcp/dist/scripts/write-role-signal.js"
DIST_PATH_LINUX="/home/barrie/MCPs/project-memory-mcp/dist/scripts/write-role-signal.js"
if [ -f "$DIST_PATH" ] && command -v node &>/dev/null; then
  CLI="node $DIST_PATH"
elif [ -f "$DIST_PATH_LINUX" ] && command -v node &>/dev/null; then
  CLI="node $DIST_PATH_LINUX"
elif command -v write-role-signal &>/dev/null; then
  CLI="write-role-signal"
fi

if [ -z "$CLI" ]; then
  # No CLI reachable; fall back to JSONL queue.
  QUEUE_DIR=".claude"
  mkdir -p "$QUEUE_DIR" 2>/dev/null || true
  QUEUE_FILE="${QUEUE_DIR}/role-signal-queue.jsonl"
  QUEUE_ROW=$(cat <<JSON
{"ts":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","role":"${ROLE}","kind":"utilisation","task_id":"${TASK_ID}","branch":"${CURRENT_BRANCH}","config_version":"${CONFIG_VERSION}","content":${CONTENT_JSON},"queued_reason":"write-role-signal CLI not found"}
JSON
)
  echo "$QUEUE_ROW" >> "$QUEUE_FILE" 2>/dev/null || true
  exit 0
fi

# Try to emit; on any error, queue.
if ! ROW_ID=$($CLI \
  --role "$ROLE" \
  --kind utilisation \
  --task-id "$TASK_ID" \
  --branch "$CURRENT_BRANCH" \
  --config-version "$CONFIG_VERSION" \
  --content "$CONTENT_JSON" 2>&1); then
  QUEUE_DIR=".claude"
  mkdir -p "$QUEUE_DIR" 2>/dev/null || true
  QUEUE_FILE="${QUEUE_DIR}/role-signal-queue.jsonl"
  QUEUE_ROW=$(cat <<JSON
{"ts":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","role":"${ROLE}","kind":"utilisation","task_id":"${TASK_ID}","branch":"${CURRENT_BRANCH}","config_version":"${CONFIG_VERSION}","content":${CONTENT_JSON},"queued_reason":"CLI exit non-zero: ${ROW_ID}"}
JSON
)
  echo "$QUEUE_ROW" >> "$QUEUE_FILE" 2>/dev/null || true
fi

exit 0
