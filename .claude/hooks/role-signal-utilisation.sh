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
# the CONSUMER of role signals, not the SUBJECT. Detection used to gate on
# `git rev-parse --abbrev-ref HEAD` matching `^junior/`, but that resolves
# against the Stop-hook process cwd which is the daemon's MAIN checkout
# (e.g. /srv/brehon-fork on governance-v0), NOT the per-task worktree.
# Bm-task workers were therefore silently dropped because the daemon's main
# checkout is rarely on a junior/* branch. The reliable Junior-vs-advisor
# signal is the CLAUDE_PROMPT env carrying a `[role:X]` dispatch tag — that
# tag is only present in Junior worker prompts, never in advisor sessions.
# Per .claude/PRPs/handovers/role-customization-2026-05-24-session3.md §3.1.
#
# Dependencies (paranoid — verified one-by-one, missing tool = exit 0):
#   jq, python3 (stdin JSON parse), node OR npx (write-role-signal CLI
#   from project-memory-mcp). git used only for config_version + branch
#   resolution against the WORKER'S worktree, not the daemon's main checkout.

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

# --- Resolve role from the dispatch line in transcript_path ---
# Junior dispatch lines start with `[role:planning|impl-task|bm-task|ci-watcher] ...`.
# This IS the Junior-vs-advisor gate: advisor sessions never carry a [role:X]
# tag at the START of their first user message; Junior dispatches always do.
#
# Where to find the dispatch line:
#   - CLAUDE_PROMPT env var: DOES NOT EXIST in `claude -p` mode (confirmed
#     against https://code.claude.com/docs/en/hooks.md — env exposes
#     CLAUDE_PROJECT_DIR + plugin paths, NOT prompt content).
#   - transcript_path stdin field: Claude Code's per-session JSONL transcript
#     under ~/.claude/projects/<proj>/<session-id>.jsonl. The full dispatch
#     prompt lives in the FIRST user-message entry's `content` field —
#     starting with the `[role:X]` tag verbatim.
#
# Detection discipline (post-2026-05-25 fix — three failed predecessors below):
#   1. Branch-gate (cwd's git branch ^junior/) — broken: Stop-hook cwd is the
#      daemon's MAIN checkout, not the worker worktree (commit feaa75db9 fix).
#   2. CLAUDE_PROMPT env regex — broken: env var doesn't exist in `claude -p`
#      mode per code.claude.com/docs/en/hooks.md (commit 729b13312 fix).
#   3. `head -50 transcript | grep '[role:X]'` ANYWHERE in first 50 lines —
#      permissive: matched advisor sessions discussing the four-role model AND
#      finalize-agent transcripts whose own prompts quote the completed task's
#      [role:X] dispatch (post-task finalize-agent's first user message contains
#      `Task that was completed: [role:planning] ...`). False-positive rate
#      visible in PMD rows 561+562 (commit b80c16dcf narrowed it).
#   4. `^\[role:` anchored at first 200 bytes of first user message — broken:
#      Junior CLI wraps the dispatch in a framework prefix
#      ("You are an autonomous worker agent..." ~552 bytes), so the legitimate
#      `[role:X]` tag sits at byte ~553. EVERY production Junior worker since
#      2026-05-24 12:06 UTC silently exited the role gate (this commit fixes).
#
# The current shape we match: `^Task:\n\[role:X\]` (multi-line anchor) inside
# the FIRST user-message content string. The Junior CLI's dispatch template is:
#
#   You are an autonomous worker agent in the Junior framework.
#   ... ~400 bytes of framework prefix ...
#
#   Task:
#   [role:planning] v1-RT-r3 — plan ... — see .claude/PRPs/briefs/...
#
# What this rejects:
#   - Advisor sessions: no `Task:\n[role:X]` shape anywhere (first message is
#     either free-form user prose or, when discussing the four-role model,
#     `[role:X]` mentions appear mid-paragraph after prose, never on a line of
#     their own immediately following a bare `Task:` line).
#   - Finalize agents: their first user message is
#     `You are a git finalize agent... Task that was completed: [role:X] ...`
#     — the role tag is on the SAME line as "Task that was completed:", not on
#     a new line after `Task:\n`.
#
# Env-var fast path retained as a no-cost prefix in case some future invocation
# does set CLAUDE_PROMPT — there the role tag is at the literal start.
ROLE=""
PROMPT="${CLAUDE_PROMPT:-}"
if [[ "$PROMPT" =~ ^\[role:(planning|impl-task|bm-task|ci-watcher)\] ]]; then
  ROLE="${BASH_REMATCH[1]}"
fi
if [ -z "$ROLE" ] && [ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ] && command -v jq &>/dev/null; then
  # Extract the FIRST user-message content string. Junior CLI's dispatch
  # prompt lands here as a plain string containing the framework prefix
  # followed by `Task:\n[role:X]`. Cap at 2000 bytes — the dispatch always
  # sits within the first ~700 bytes; 2000 leaves headroom for future prompt-
  # template changes while keeping the regex cheap on a 1MB+ advisor transcript.
  _FIRST_USER=$(jq -rs '
    [.[] | select(.type? == "user" and (.message?.content | type) == "string")] | .[0]?.message?.content // ""
  ' "$TRANSCRIPT_PATH" 2>/dev/null | head -c 2000)
  # Multi-line bash regex: $'...' enables \n interpretation, then anchor
  # `^Task:` at line start followed by optional whitespace + literal newline
  # + `[role:X]` at the next line start. BASH_REMATCH[1] captures the role.
  _TASK_HEADER_RE=$'(^|\n)Task:[[:space:]]*\n\\[role:(planning|impl-task|bm-task|ci-watcher)\\]'
  if [[ "$_FIRST_USER" =~ $_TASK_HEADER_RE ]]; then
    ROLE="${BASH_REMATCH[2]}"
  fi
fi
if [ -z "$ROLE" ]; then
  # No `Task:\n[role:X]` shape in first user message — advisor session,
  # finalize agent, or ad-hoc Junior task. Skip. Early exit BEFORE the
  # expensive worktree resolution / git rev-parse / transcript jq -rcs scan.
  exit 0
fi

# --- Resolve WORKER worktree directory (NOT the daemon's main checkout) ---
# transcript_path lives under the worker's per-task worktree, e.g.:
#   /srv/brehon-fork/.junior/worktrees/job-447/.claude/transcript-XXXX.jsonl
# Walk up from transcript_path until we find a .git/ entry (worktrees have
# a .git FILE pointing at the daemon's .git/worktrees/<name> admin dir).
# Falls back to the Stop-hook process cwd if transcript_path is absent.
WORKER_DIR=""
if [ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ]; then
  _dir=$(dirname "$TRANSCRIPT_PATH")
  while [ "$_dir" != "/" ] && [ "$_dir" != "." ]; do
    if [ -e "$_dir/.git" ]; then
      WORKER_DIR="$_dir"
      break
    fi
    _dir=$(dirname "$_dir")
  done
fi
if [ -z "$WORKER_DIR" ]; then
  WORKER_DIR="${CWD_FROM_STDIN:-$(pwd)}"
fi

# --- Resolve branch from the worker worktree ---
# `git -C <worker_dir>` ensures we read the worktree's HEAD, not the daemon
# main checkout's. This is the branch the worker is actually committing on.
CURRENT_BRANCH=""
if command -v git &>/dev/null; then
  CURRENT_BRANCH=$(git -C "$WORKER_DIR" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")
fi

# --- Resolve task_id ---
# Prefer session_id from stdin (most stable); fall back to branch suffix.
TASK_ID="${SESSION_ID:-}"
if [ -z "$TASK_ID" ] && [ -n "$CURRENT_BRANCH" ]; then
  # branch shape: junior/<slug>-<id>
  TASK_ID="${CURRENT_BRANCH##*-}"
fi

# --- Resolve config_version (subtree SHA of .claude/roles/<role>) ---
# Per .claude/PRPs/handovers/role-customization-2026-05-24.md §4 T2:
# the subtree SHA pins the signal to the exact manifest state at task time.
# Read from the WORKER worktree's HEAD, not the daemon main checkout.
#
# Quirk: `git rev-parse` on missing refs prints the literal ref to STDOUT
# (not stderr) and exits 128. So we can't rely on `2>/dev/null || echo`.
# Capture exit code explicitly and discard stdout on failure.
CONFIG_VERSION=""
if command -v git &>/dev/null; then
  _GR_OUT=$(git -C "$WORKER_DIR" rev-parse "HEAD:.claude/roles/${ROLE}" 2>/dev/null) && CONFIG_VERSION="$_GR_OUT" || CONFIG_VERSION=""
fi
if [ -z "$CONFIG_VERSION" ] || [[ "$CONFIG_VERSION" == HEAD:* ]]; then
  # Role substrate not present on this branch — pre-T2 history, or the
  # ref-not-found stdout leaked through. Mark as pre-t2.
  CONFIG_VERSION="pre-t2"
fi

# --- Parse transcript for rules Read + MCP tools invoked ---
RULES_READ_JSON="[]"
MCP_TOOLS_JSON="[]"
if [ -n "$TRANSCRIPT_PATH" ] && [ -f "$TRANSCRIPT_PATH" ] && command -v jq &>/dev/null; then
  # Transcript is JSONL; each line is one event. tool_use blocks are NESTED
  # inside an assistant message's content[] array — top-level .type is
  # "assistant", never "tool_use". A flat `select(.type=="tool_use")` matches
  # nothing and always yields []. Descend through .message.content[] first.
  # (Root cause of the 197 all-empty rows, 2026-05-24..06-26.)
  RULES_READ_JSON=$(jq -rcs '
    [.[]
      | select(.type? == "assistant")
      | .message?.content?[]?
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
      | select(.type? == "assistant")
      | .message?.content?[]?
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
# on this branch that was NOT present on governance-v0 base. Read from the
# WORKER worktree's working tree + the daemon's governance-v0 ref via -C.
DQ_BLOCKERS=0
DQ_FILE="${WORKER_DIR}/.claude/decision-queue.json"
if command -v jq &>/dev/null && [ -f "$DQ_FILE" ]; then
  # Get the count of blocker entries currently on this branch.
  THIS_BRANCH_BLOCKERS=$(jq -r '
    ([.pending[]?, .resolved[]?] | map(select(.kind? == "blocker"))) | length
  ' "$DQ_FILE" 2>/dev/null || echo 0)
  # Get the count from governance-v0 base (if reachable; falls back to 0).
  BASE_BLOCKERS=$(git -C "$WORKER_DIR" show governance-v0:.claude/decision-queue.json 2>/dev/null | jq -r '
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

# --- Emit signal via CLI, or fall back to JSONL queue ---
# JSONL queue MUST live in the main checkout's .claude/, NOT the worker
# worktree's .claude/. Worker worktrees are reaped by the daemon finalize
# step; a worktree-local queue file disappears before the laptop drain
# script can collect it. The main checkout persists indefinitely.
#
# Derive main-checkout root via `git rev-parse --git-common-dir`: for a
# worktree this returns the main repo's .git/ dir (e.g.
# /srv/brehon-fork/.git); stripping the trailing /.git gives the root.
# Falls back to WORKER_DIR if git is unavailable or the command fails.
MAIN_CHECKOUT_ROOT=""
if command -v git &>/dev/null; then
  # --git-common-dir may return a relative path (e.g. ../../../.git) when
  # called from inside a worktree. Resolve to absolute via a subshell cd.
  _GIT_COMMON=$(git -C "$WORKER_DIR" rev-parse --git-common-dir 2>/dev/null || true)
  if [ -n "$_GIT_COMMON" ]; then
    _GIT_COMMON_ABS=$(cd "$WORKER_DIR" && cd "$_GIT_COMMON" 2>/dev/null && pwd || true)
    if [ -n "$_GIT_COMMON_ABS" ]; then
      MAIN_CHECKOUT_ROOT="${_GIT_COMMON_ABS%/.git}"
    fi
  fi
fi
if [ -z "$MAIN_CHECKOUT_ROOT" ] || [ ! -d "$MAIN_CHECKOUT_ROOT" ]; then
  MAIN_CHECKOUT_ROOT="$WORKER_DIR"
fi
QUEUE_DIR="${MAIN_CHECKOUT_ROOT}/.claude"
QUEUE_FILE="${QUEUE_DIR}/role-signal-queue.jsonl"
mkdir -p "$QUEUE_DIR" 2>/dev/null || true

queue_signal() {
  local reason="$1"
  local row
  row=$(cat <<JSON
{"ts":"$(date -u +%Y-%m-%dT%H:%M:%SZ)","role":"${ROLE}","kind":"utilisation","task_id":"${TASK_ID}","branch":"${CURRENT_BRANCH}","config_version":"${CONFIG_VERSION}","content":${CONTENT_JSON},"queued_reason":"${reason}"}
JSON
)
  echo "$row" >> "$QUEUE_FILE" 2>/dev/null || true
}

if [ -z "$CLI" ]; then
  queue_signal "write-role-signal CLI not found"
  exit 0
fi

if ! ROW_ID=$($CLI \
  --role "$ROLE" \
  --kind utilisation \
  --task-id "$TASK_ID" \
  --branch "$CURRENT_BRANCH" \
  --config-version "$CONFIG_VERSION" \
  --content "$CONTENT_JSON" 2>&1); then
  queue_signal "CLI exit non-zero: ${ROW_ID}"
fi

exit 0
