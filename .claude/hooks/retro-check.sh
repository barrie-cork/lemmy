#!/usr/bin/env bash
# Stop hook: block exit if no post-task retro eval was written recently.
# Deployed to .claude/hooks/retro-check.sh on server repos.
#
# Event: Stop
# Timeout: 10000
#
# Exit 0 = allow stop
# Exit 2 = block stop (stderr becomes additionalContext for the model)
#
# Retry safety: after 3 blocks in the same bash process lineage, fail open.
# (Each Stop hook invocation is a fresh bash, so this only catches true loops
# where the agent repeatedly tries to exit within a single parent claude pass.)

set -euo pipefail

# --- Skip conditions ---

# Skip for weekly-review and audit tasks (they have their own reporting)
PROMPT="${CLAUDE_PROMPT:-}"
if echo "$PROMPT" | grep -qiE "weekly-review|SKILL\.md.*weekly|security.*audit|security.*scan"; then
  exit 0
fi

# --- Find PMD database (sqlite fallback path) ---
# In a git worktree (Junior tasks), cwd is the worktree root, not the main repo.
# The PMD lives in the main repo's .project-memory/. Derive from git-common-dir.

DB=""
GIT_COMMON=""
MAIN_REPO=""
if [ -n "${PROJECT_MEMORY_DB:-}" ]; then
  DB="$PROJECT_MEMORY_DB"
else
  GIT_COMMON=$(git rev-parse --git-common-dir 2>/dev/null || true)
  if [ -n "$GIT_COMMON" ] && [ "$GIT_COMMON" != ".git" ]; then
    MAIN_REPO=$(dirname "$GIT_COMMON")
    DB="${MAIN_REPO}/.project-memory/memory.db"
  fi
  if [ -z "$DB" ] || [ ! -f "$DB" ]; then
    DB=".project-memory/memory.db"
  fi
fi

# --- Discover HTTP PMD endpoint from .mcp.json (Option A) ---
# When the project-memory MCP is configured as HTTP (type=http), Junior workers
# write retros to the HTTP server (e.g. laptop at Tailscale IP) — a separate store
# from the daemon-local sqlite. Query the HTTP server preferentially using the MCP
# session protocol (two-step: POST initialize -> POST tools/call).
# See feedback_pmd_retro_check_http_store_split.md for the topology incident.
PMD_HTTP_URL=""
PMD_HTTP_TOKEN=""
if command -v jq &>/dev/null; then
  _MCP_JSON="${MAIN_REPO:-.}/.mcp.json"
  if [ ! -f "$_MCP_JSON" ]; then
    _MCP_JSON=".mcp.json"
  fi
  if [ -f "$_MCP_JSON" ]; then
    _MCP_TYPE=$(jq -r '.mcpServers["project-memory"].type // ""' "$_MCP_JSON" 2>/dev/null || true)
    if [ "$_MCP_TYPE" = "http" ]; then
      PMD_HTTP_URL=$(jq -r '.mcpServers["project-memory"].url // ""' "$_MCP_JSON" 2>/dev/null || true)
      PMD_HTTP_TOKEN=$(jq -r '(.mcpServers["project-memory"].headers.Authorization // "") | ltrimstr("Bearer ")' "$_MCP_JSON" 2>/dev/null || true)
    fi
  fi
fi

# No PMD at all -> can't enforce, fail open
_HAVE_SQLITE=0
_HAVE_HTTP=0
{ [ -n "$DB" ] && [ -f "$DB" ] && command -v sqlite3 &>/dev/null && _HAVE_SQLITE=1; } || true
{ [ -n "$PMD_HTTP_URL" ] && [ -n "$PMD_HTTP_TOKEN" ] && command -v curl &>/dev/null && _HAVE_HTTP=1; } || true
if [ "$_HAVE_SQLITE" -eq 0 ] && [ "$_HAVE_HTTP" -eq 0 ]; then
  exit 0
fi

# --- Determine enforcement mode from current git branch ---
#
# DESIGN HISTORY (see PMD #927):
#
#   v0 -- `created_at > SESSION_START` with SESSION_START captured on first Stop
#        invocation. Bypassed by agents writing created_at='2099-12-31'.
#
#   v1 -- `id > SESSION_START_ID` with MAX(id) captured on first Stop invocation.
#        Race condition: each Stop hook spawned a fresh bash with unstable
#        $PPID, so the session file was recreated after the agent had already
#        written the retro, making SESSION_START_ID > retro.id.
#
#   v2 -- Simple 15-min time window with upper+lower bounds. Killed the race
#        and the 2099 forgery, but let back-to-back tasks coat-tail on each
#        other's retros. Observed on my-food-system task #27 which exited
#        cleanly without writing a retro because task #26's retro from 3 min
#        earlier was still within the window.
#
#   v3 -- Branch-scoped time window for Junior worktrees, time-only for others.
#        Retro must satisfy `source_ref = $BRANCH OR branch = $BRANCH`, forcing
#        each task to write its own retro. Window extended to 30 min because
#        coat-tailing is no longer possible.
#
#   v4 -- Added HTTP PMD check (Option A). When .mcp.json configures the
#        project-memory MCP as HTTP (not stdio), Junior workers write retros to
#        the HTTP server -- a separate store from the daemon-local sqlite.
#        The hook now queries the HTTP server via MCP session protocol (initialize
#        -> tools/call -> memory_search), falls back to sqlite for environments
#        where the HTTP server is absent. See feedback_pmd_retro_check_http_store_split.md.

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")

# Sanity-check: only allow branch names made of safe characters so we can
# interpolate into SQL without injection risk. Real git branch names are
# restricted by git itself, but we defend anyway.
if [[ ! "$CURRENT_BRANCH" =~ ^[a-zA-Z0-9._/-]+$ ]]; then
  CURRENT_BRANCH=""
fi

if [[ "$CURRENT_BRANCH" == junior/* ]]; then
  WINDOW_MINUTES=30
  MODE="branch-scoped on '$CURRENT_BRANCH'"
else
  WINDOW_MINUTES=60
  MODE="time-window (branch='${CURRENT_BRANCH:-unknown}')"
fi

# --- HTTP PMD check (Option A -- preferred when MCP type=http) ---
# Two-step MCP session: POST initialize to get mcp-session-id header, then POST
# tools/call to search for recent retros. Python3 parses SSE JSON response and
# applies the same branch-scope + time-window filter as the sqlite path.
HTTP_RECENT=0
if [ "$_HAVE_HTTP" -eq 1 ] && command -v python3 &>/dev/null; then
  _HDR=$(mktemp /tmp/.pmd-rc-XXXXXX 2>/dev/null || echo "/tmp/.pmd-rc-$$")
  # Step 1: initialize MCP session, capture session ID from response header
  curl -s -m 5 \
    -H "Authorization: Bearer $PMD_HTTP_TOKEN" \
    -H "Accept: application/json, text/event-stream" \
    -H "Content-Type: application/json" \
    -X POST "$PMD_HTTP_URL" \
    -D "$_HDR" \
    -d '{"jsonrpc":"2.0","method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"retro-check","version":"1.0"}},"id":1}' \
    > /dev/null 2>/dev/null || true
  _SID=$(grep -i 'mcp-session-id' "$_HDR" 2>/dev/null | awk '{print $2}' | tr -d '\r' || true)
  rm -f "$_HDR" 2>/dev/null || true

  if [ -n "$_SID" ]; then
    # Step 2: search recent retros (memory_search; branch filtering is client-side)
    _SR=$(curl -s -m 10 \
      -H "Authorization: Bearer $PMD_HTTP_TOKEN" \
      -H "Accept: application/json, text/event-stream" \
      -H "Content-Type: application/json" \
      -H "Mcp-Session-Id: $_SID" \
      -X POST "$PMD_HTTP_URL" \
      -d '{"jsonrpc":"2.0","method":"tools/call","params":{"name":"memory_search","arguments":{"query":"Task retro","memory_type":"qa-result","limit":20}},"id":2}' \
      2>/dev/null | grep '^data:' | head -1 | sed 's/^data: //' || true)

    if [ -n "$_SR" ]; then
      _JMODE="other"
      [[ "$CURRENT_BRANCH" == junior/* ]] && _JMODE="junior" || true
      # Write Python script to temp file to avoid shell-quoting + heredoc stdin conflict
      _PYSC=$(mktemp /tmp/.pmd-rc-py-XXXXXX 2>/dev/null || echo "/tmp/.pmd-rc-py-$$")
      cat > "$_PYSC" << 'PYEOF'
import sys, json
from datetime import datetime, timezone, timedelta
try:
    data = json.loads(sys.stdin.read())
    text = data['result']['content'][0]['text']
    memories = json.loads(text)
    branch, window, mode = sys.argv[1], int(sys.argv[2]), sys.argv[3]
    now = datetime.now(timezone.utc)
    cutoff = now - timedelta(minutes=window)
    future = now + timedelta(minutes=5)
    count = 0
    for m in memories:
        title = m.get('title', '')
        if not (title.startswith('Task retro:') or title.startswith('Session retro:')):
            continue
        created = m.get('created_at', '')
        try:
            dt = datetime.strptime(created, '%Y-%m-%d %H:%M:%S').replace(tzinfo=timezone.utc)
        except Exception:
            continue
        if not (cutoff <= dt <= future):
            continue
        if mode == 'junior':
            if m.get('source_ref') != branch and m.get('branch') != branch:
                continue
        count += 1
    print(count)
except Exception:
    print(0)
PYEOF
      HTTP_RECENT=$(echo "$_SR" | python3 "$_PYSC" "$CURRENT_BRANCH" "$WINDOW_MINUTES" "$_JMODE" 2>/dev/null || echo 0)
      rm -f "$_PYSC" 2>/dev/null || true
    fi
  fi
fi

if [ "${HTTP_RECENT:-0}" -gt 0 ]; then
  exit 0
fi

# --- SQLite fallback ---
# Used when HTTP server is not configured, unreachable, or returns no match.
RECENT=0
if [ "$_HAVE_SQLITE" -eq 1 ]; then
  if [[ "$CURRENT_BRANCH" == junior/* ]]; then
    # Junior worktrees: post-task-retro skill prescribes 'Task retro:%' titles.
    # /session-retro is for advisor/interactive sessions only -- would never fire
    # on a junior/* branch. So keep the branch-scoped Junior check title-strict.
    RECENT=$(sqlite3 "$DB" \
      "SELECT COUNT(*) FROM memories
       WHERE memory_type='qa-result'
         AND title LIKE 'Task retro:%'
         AND (source_ref = '${CURRENT_BRANCH}' OR branch = '${CURRENT_BRANCH}')
         AND created_at >= datetime('now', '-${WINDOW_MINUTES} minutes')
         AND created_at <= datetime('now', '+5 minutes')" \
      2>/dev/null || echo 0)
  else
    # Advisor / interactive sessions on governance-v0 / phase-v* branches.
    # Two retro disciplines satisfy this gate:
    #   - post-task-retro skill: title 'Task retro:%' (Junior task end OR mid-
    #     session interactive task close).
    #   - /session-retro skill: title 'Session retro:%' (ad-hoc session
    #     retrospective via SKILL.md Step 5 PMD eval).
    # Pre-2026-05-24 the hook only matched 'Task retro:%' -- a /session-retro
    # eval row did not release the hook, producing the user-visible 3-ESC
    # fail-open pattern even when discipline was followed. Per session-retro
    # 2026-05-24-task4-followups proposal Section "Stop hook title-prefix gap".
    # 60-min window unchanged (long polling sessions don't need per-poll retros).
    RECENT=$(sqlite3 "$DB" \
      "SELECT COUNT(*) FROM memories
       WHERE memory_type='qa-result'
         AND (title LIKE 'Task retro:%' OR title LIKE 'Session retro:%')
         AND created_at >= datetime('now', '-${WINDOW_MINUTES} minutes')
         AND created_at <= datetime('now', '+5 minutes')" \
      2>/dev/null || echo 0)
  fi
fi

if [ "${RECENT:-0}" -gt 0 ]; then
  exit 0
fi

# --- emit_retro_bypass_log ---
#
# Writes one JSONL record to .claude/governance-log/retro-bypass.jsonl
# on every fail-open path. Per RLS-PMD review Section 4.7 +
# .claude/PRPs/plans/v1-rls-r1.plan.md Section 13 Task 7.
#
# Fields (per DQ #297): timestamp, session_id, attempt_count,
# prompt_hash, branch_at_fail_open, kind.
#
# Non-fatal -- any error (missing dir, write race, jq absent)
# silently exits the function. Bypass instrumentation must not
# itself become a Stop hook failure mode.
#
# NOTE: function MUST be defined BEFORE the fail-open call site
# (line ~143). Bash executes top-to-bottom; calling an undefined
# function silently fails on most shells. Task 10 dogfood caught
# the original Task 7 placement (function appended after `exit 2`,
# unreachable on every code path).
emit_retro_bypass_log() {
  local attempts="$1"
  local branch="$2"
  local logdir=".claude/governance-log"
  local logfile="${logdir}/retro-bypass.jsonl"
  mkdir -p "$logdir" 2>/dev/null || return 0
  command -v jq >/dev/null 2>&1 || return 0
  local ts
  ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local prompt_hash
  prompt_hash="$(printf '%s' "${CLAUDE_PROMPT:-}" | sha256sum 2>/dev/null | head -c 16)"
  [ -z "$prompt_hash" ] && prompt_hash="unknown"
  local session_id="${CLAUDE_SESSION_ID:-${PPID:-unknown}}"
  jq -nc \
    --arg ts "$ts" \
    --arg sid "$session_id" \
    --argjson att "$attempts" \
    --arg ph "$prompt_hash" \
    --arg br "$branch" \
    '{timestamp:$ts, session_id:$sid, attempt_count:$att, prompt_hash:$ph, branch_at_fail_open:$br, kind:"retro_bypass"}' \
    >> "$logfile" 2>/dev/null || true
}

# --- Retry safety: fail open after 3 blocks in the same bash lineage ---
# Use PPID as a best-effort loop detector. If PPID is stable (which we're no
# longer assuming for correctness), this works. If not, every invocation starts
# fresh -- which is fine because we're not relying on it for enforcement, only
# for loop prevention.

SESSION_DIR="/tmp/cc-retro-sessions"
mkdir -p "$SESSION_DIR"
SESSION_FILE="${SESSION_DIR}/${PPID:-unknown}"

ATTEMPTS=0
if [ -f "$SESSION_FILE" ]; then
  ATTEMPTS=$(head -1 "$SESSION_FILE" 2>/dev/null || echo 0)
fi

if [ "$ATTEMPTS" -ge 3 ]; then
  rm -f "$SESSION_FILE"
  emit_retro_bypass_log "$ATTEMPTS" "$CURRENT_BRANCH"
  exit 0
fi

ATTEMPTS=$((ATTEMPTS + 1))
echo "$ATTEMPTS" > "$SESSION_FILE"

# --- Block exit ---

if [[ "$CURRENT_BRANCH" == junior/* ]]; then
  echo "MANDATORY: Post-task retrospective not found for branch '$CURRENT_BRANCH' (attempt $ATTEMPTS/3). You MUST write a retro before exiting. Call memory_write_eval with: memory_type='qa-result', title starting 'Task retro:', source_ref='$CURRENT_BRANCH' (this exact value -- the hook matches on it), a 3-signal score (goal/tests/clean), and tags including the repo name. The hook uses ${MODE} with a ${WINDOW_MINUTES}-minute window. Do NOT coat-tail on a previous task's retro, modify this hook file, forge created_at, or use raw SQL -- those bypass attempts are tracked." >&2
else
  echo "MANDATORY: Post-task retrospective not found (attempt $ATTEMPTS/3). You MUST write a retro before exiting. Call memory_write_eval with: memory_type='qa-result', title starting 'Task retro:', a 3-signal score (goal/tests/clean), and tags including the repo name. The hook uses ${MODE} with a ${WINDOW_MINUTES}-minute window. Do NOT modify this hook file, forge created_at, or use raw SQL -- those bypass attempts are tracked." >&2
fi
exit 2
