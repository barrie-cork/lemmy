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

# --- Find PMD database ---
# In a git worktree (Junior tasks), cwd is the worktree root, not the main repo.
# The PMD lives in the main repo's .project-memory/. Derive from git-common-dir.

DB=""
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

# No PMD → can't enforce, fail open
if [ -z "$DB" ] || [ ! -f "$DB" ]; then
  exit 0
fi

# sqlite3 required
if ! command -v sqlite3 &>/dev/null; then
  exit 0
fi

# --- Determine enforcement mode from current git branch ---
#
# Junior worktrees are checked out on branches named `junior/<task-slug>-<id>`.
# For those, we require a retro whose source_ref OR branch field matches the
# current branch — no coat-tailing on the previous task's retro.
#
# For any other branch (main, master, interactive sessions on the main repo),
# we fall back to a simple time-window check so one-shot work on main isn't
# forced to write a retro for every prompt.

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")

# Sanity-check: only allow branch names made of safe characters so we can
# interpolate into SQL without injection risk. Real git branch names are
# restricted by git itself, but we defend anyway.
if [[ ! "$CURRENT_BRANCH" =~ ^[a-zA-Z0-9._/-]+$ ]]; then
  CURRENT_BRANCH=""
fi

# --- Check for retro eval written in the recent window ---
#
# DESIGN HISTORY (see PMD #927):
#
#   v0 — `created_at > SESSION_START` with SESSION_START captured on first Stop
#        invocation. Bypassed by agents writing created_at='2099-12-31'.
#
#   v1 — `id > SESSION_START_ID` with MAX(id) captured on first Stop invocation.
#        Race condition: each Stop hook spawned a fresh bash with unstable
#        $PPID, so the session file was recreated after the agent had already
#        written the retro, making SESSION_START_ID > retro.id.
#
#   v2 — Simple 15-min time window with upper+lower bounds. Killed the race
#        and the 2099 forgery, but let back-to-back tasks coat-tail on each
#        other's retros. Observed on my-food-system task #27 which exited
#        cleanly without writing a retro because task #26's retro from 3 min
#        earlier was still within the window.
#
#   v3 — Branch-scoped time window for Junior worktrees, time-only for others.
#        Retro must satisfy `source_ref = $BRANCH OR branch = $BRANCH`, forcing
#        each task to write its own retro. Window extended to 30 min because
#        coat-tailing is no longer possible.

if [[ "$CURRENT_BRANCH" == junior/* ]]; then
  WINDOW_MINUTES=30
  MODE="branch-scoped on '$CURRENT_BRANCH'"
  RECENT=$(sqlite3 "$DB" \
    "SELECT COUNT(*) FROM memories
     WHERE memory_type='qa-result'
       AND title LIKE 'Task retro:%'
       AND (source_ref = '${CURRENT_BRANCH}' OR branch = '${CURRENT_BRANCH}')
       AND created_at >= datetime('now', '-${WINDOW_MINUTES} minutes')
       AND created_at <= datetime('now', '+5 minutes')" \
    2>/dev/null || echo 0)
else
  # 60-min window for advisor / interactive sessions on governance-v0 (long
  # polling-loop sessions don't need per-poll retros — bumped 2026-05-03 from
  # 15 min after duplicate-retro feedback). Junior worktree branches keep
  # the 30-min branch-scoped check above (load-bearing for Junior task eval).
  WINDOW_MINUTES=60
  MODE="time-window (branch='${CURRENT_BRANCH:-unknown}')"
  RECENT=$(sqlite3 "$DB" \
    "SELECT COUNT(*) FROM memories
     WHERE memory_type='qa-result'
       AND title LIKE 'Task retro:%'
       AND created_at >= datetime('now', '-${WINDOW_MINUTES} minutes')
       AND created_at <= datetime('now', '+5 minutes')" \
    2>/dev/null || echo 0)
fi

if [ "${RECENT:-0}" -gt 0 ]; then
  exit 0
fi

# --- emit_retro_bypass_log ---
#
# Writes one JSONL record to .claude/governance-log/retro-bypass.jsonl
# on every fail-open path. Per RLS-PMD review §4.7 +
# .claude/PRPs/plans/v1-rls-r1.plan.md §13 Task 7.
#
# Fields (per DQ #297): timestamp, session_id, attempt_count,
# prompt_hash, branch_at_fail_open, kind.
#
# Non-fatal — any error (missing dir, write race, jq absent)
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
# fresh — which is fine because we're not relying on it for enforcement, only
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
  echo "MANDATORY: Post-task retrospective not found for branch '$CURRENT_BRANCH' (attempt $ATTEMPTS/3). You MUST write a retro before exiting. Call memory_write_eval with: memory_type='qa-result', title starting 'Task retro:', source_ref='$CURRENT_BRANCH' (this exact value — the hook matches on it), a 3-signal score (goal/tests/clean), and tags including the repo name. The hook uses ${MODE} with a ${WINDOW_MINUTES}-minute window. Do NOT coat-tail on a previous task's retro, modify this hook file, forge created_at, or use raw SQL — those bypass attempts are tracked." >&2
else
  echo "MANDATORY: Post-task retrospective not found (attempt $ATTEMPTS/3). You MUST write a retro before exiting. Call memory_write_eval with: memory_type='qa-result', title starting 'Task retro:', a 3-signal score (goal/tests/clean), and tags including the repo name. The hook uses ${MODE} with a ${WINDOW_MINUTES}-minute window. Do NOT modify this hook file, forge created_at, or use raw SQL — those bypass attempts are tracked." >&2
fi
exit 2
