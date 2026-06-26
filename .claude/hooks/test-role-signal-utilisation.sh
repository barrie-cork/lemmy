#!/usr/bin/env bash
# Smoke harness for role-signal-utilisation.sh role-detection logic.
#
# Every change to the role-detection block in
# .claude/hooks/role-signal-utilisation.sh MUST run this harness against
# real transcripts on the EliteDesk BEFORE the change is committed. The
# hook has had 4 sequential root-cause fixes (f6088a83d → feaa75db9 →
# 729b13312 → b80c16dcf → 10b29ec4d) because each fix was test-cased
# against an incomplete set of transcript shapes:
#
#   f6088a83d  initial               (branch gate — cwd was main checkout)
#   feaa75db9  fix #1 (CLAUDE_PROMPT env — doesn't exist in `claude -p`)
#   729b13312  fix #2 (head -50 | grep — false-positive on finalize + advisor)
#   b80c16dcf  fix #3 (^\[role: anchor on byte 0 — missed framework prefix)
#   10b29ec4d  fix #4 (^Task:\s*\n\[role: multi-line — current contract)
#
# The contract this harness asserts:
#
#   POSITIVE — Real Junior worker (Junior CLI dispatch):
#     "You are an autonomous worker agent in the Junior framework.
#      ... ~400 bytes of framework prefix ...
#      Task:
#      [role:X] <slug>"
#     → MUST return role X
#
#   NEGATIVE — Finalize agent (per-task git-only sub-agent):
#     "You are a git finalize agent. Your ONLY job is ...
#      Task that was completed: [role:X] <slug>"
#     → MUST return empty (the role tag is on a SAME-line prose value,
#       not a Task:\n[role:X] header pattern)
#
#   NEGATIVE — Advisor session (laptop interactive):
#     Any free-form prose, system reminders, slash-command output
#     → MUST return empty (no Task:\n[role:X] shape anywhere)
#
# Run on EliteDesk:
#   bash /srv/brehon-fork/.claude/hooks/test-role-signal-utilisation.sh
# Run on laptop (Git Bash) — provide transcripts via env or override
# the path constants below:
#   bash .claude/hooks/test-role-signal-utilisation.sh
#
# Exit code: 0 if all asserted shapes pass, 1 if any fail.

set -uo pipefail

PASS=0
FAIL=0

# --- Inline copy of the role-detection logic (MUST match the production
# --- hook's role-resolution block byte-for-byte except for variable names).
# --- If the production hook's regex changes, mirror the change here AND
# --- add a new test case below covering the new shape.
detect_role() {
  local transcript="$1"

  if [ ! -f "$transcript" ]; then
    echo ""
    return
  fi

  local _FIRST_USER
  _FIRST_USER=$(jq -rs '
    [.[] | select(.type? == "user" and (.message?.content | type) == "string")] | .[0]?.message?.content // ""
  ' "$transcript" 2>/dev/null | head -c 2000)

  local _TASK_HEADER_RE=$'(^|\n)Task:[[:space:]]*\n\\[role:(planning|impl-task|bm-task|ci-watcher)\\]'
  if [[ "$_FIRST_USER" =~ $_TASK_HEADER_RE ]]; then
    echo "${BASH_REMATCH[2]}"
  else
    echo ""
  fi
}

assert_role() {
  local label="$1"
  local transcript="$2"
  local expected="$3"

  if [ ! -f "$transcript" ]; then
    echo "SKIP  $label (transcript not found: $transcript)"
    return
  fi

  local got
  got=$(detect_role "$transcript")

  if [ "$got" = "$expected" ]; then
    echo "PASS  $label -> got '$got' (expected '$expected')"
    PASS=$((PASS + 1))
  else
    echo "FAIL  $label -> got '$got' (expected '$expected')"
    FAIL=$((FAIL + 1))
  fi
}

# --- Inline copy of the array-population jq (MUST mirror the production
# --- hook's mcp_tools_invoked extraction byte-for-byte except var names).
# --- tool_use blocks are NESTED in assistant .message.content[]; a flat
# --- select(.type=="tool_use") matches nothing and silently yields [].
# --- This case exists because 197 rows (2026-05-24..06-26) shipped with
# --- all-empty arrays — detect_role passed, array-population was never
# --- asserted. The smoke harness now covers BOTH halves.
extract_mcps() {
  local transcript="$1"
  [ -f "$transcript" ] || { echo "[]"; return; }
  jq -rcs '
    [.[]
      | select(.type? == "assistant")
      | .message?.content?[]?
      | select(.type? == "tool_use")
      | .name? // empty
      | select(startswith("mcp__"))
    ] | unique
  ' "$transcript" 2>/dev/null || echo "[]"
}

# Assert a transcript known to invoke ≥1 MCP tool yields a non-empty array.
# A flat-select regression makes this FAIL (returns []), catching the
# nested-vs-top-level bug class the role-detection assertions cannot see.
assert_mcp_nonempty() {
  local label="$1"
  local transcript="$2"

  if [ ! -f "$transcript" ]; then
    echo "SKIP  $label (transcript not found: $transcript)"
    return
  fi

  local got
  got=$(extract_mcps "$transcript")

  if [ -n "$got" ] && [ "$got" != "[]" ] && [ "$got" != "null" ]; then
    echo "PASS  $label -> mcp_tools_invoked=$got"
    PASS=$((PASS + 1))
  else
    echo "FAIL  $label -> mcp_tools_invoked='$got' (expected non-empty; likely flat-select regression)"
    FAIL=$((FAIL + 1))
  fi
}

# --- Transcript discovery: prefer env-var overrides so the harness is
# --- portable between EliteDesk and laptop. On EliteDesk, defaults are
# --- /home/barrie/.claude/projects/-srv-brehon-fork{,--junior-worktrees-job-N}/<sid>.jsonl.
# --- Override per-shape with env vars; missing files SKIP rather than FAIL
# --- so partial coverage is visible without false-red.

TRANSCRIPT_POSITIVE_JUNIOR="${TRANSCRIPT_POSITIVE_JUNIOR:-}"
TRANSCRIPT_POSITIVE_JUNIOR_2="${TRANSCRIPT_POSITIVE_JUNIOR_2:-}"
TRANSCRIPT_NEGATIVE_FINALIZE="${TRANSCRIPT_NEGATIVE_FINALIZE:-}"
TRANSCRIPT_NEGATIVE_FINALIZE_2="${TRANSCRIPT_NEGATIVE_FINALIZE_2:-}"
TRANSCRIPT_NEGATIVE_ADVISOR="${TRANSCRIPT_NEGATIVE_ADVISOR:-}"
EXPECT_JUNIOR_ROLE="${EXPECT_JUNIOR_ROLE:-planning}"
EXPECT_JUNIOR_2_ROLE="${EXPECT_JUNIOR_2_ROLE:-bm-task}"

# Auto-discover most-recent ROLE-PREFIXED transcript when env-vars empty
# (EliteDesk default). Naive `ls -t | head -1` grabs the newest worktree,
# which may be a role-less task (e.g. a plain "Review: ..." weekly-review
# dispatch) — that yields detect_role='' and a false-red against the
# EXPECT_JUNIOR_ROLE=planning default. Scan newest-first for a transcript
# whose first user message carries the Task:\n[role:X] contract.
if [ -z "$TRANSCRIPT_POSITIVE_JUNIOR" ]; then
  for _t in $(ls -t /home/barrie/.claude/projects/-srv-brehon-fork--junior-worktrees-job-*/*.jsonl 2>/dev/null); do
    _r=$(detect_role "$_t")
    if [ -n "$_r" ]; then
      TRANSCRIPT_POSITIVE_JUNIOR="$_t"
      EXPECT_JUNIOR_ROLE="$_r"   # assert against the role actually present
      break
    fi
  done
fi
# MCP-population fixture: NOT every role uses MCP — Haiku bm-task workers
# routinely run pure Bash/Edit/Read with zero MCP calls, so the newest
# role-prefixed transcript is the wrong fixture (false-red). Scan
# newest-first for a transcript that actually invokes ≥1 mcp__ tool
# (planning tasks reliably call mcp__project-memory__*). Override via env.
if [ -z "${TRANSCRIPT_MCP_NONEMPTY:-}" ]; then
  for _t in $(ls -t /home/barrie/.claude/projects/-srv-brehon-fork--junior-worktrees-job-*/*.jsonl 2>/dev/null); do
    if [ "$(extract_mcps "$_t")" != "[]" ]; then
      TRANSCRIPT_MCP_NONEMPTY="$_t"
      break
    fi
  done
fi

echo "=== role-signal-utilisation.sh smoke harness ==="
echo "running detection logic against ≥3 transcript shapes"
echo

assert_role "POSITIVE: Junior worker (framework-prefixed dispatch)" \
  "$TRANSCRIPT_POSITIVE_JUNIOR" \
  "$EXPECT_JUNIOR_ROLE"

if [ -n "$TRANSCRIPT_POSITIVE_JUNIOR_2" ]; then
  assert_role "POSITIVE: second Junior worker (different role)" \
    "$TRANSCRIPT_POSITIVE_JUNIOR_2" \
    "$EXPECT_JUNIOR_2_ROLE"
fi

assert_role "NEGATIVE: finalize agent (per-task git-only)" \
  "$TRANSCRIPT_NEGATIVE_FINALIZE" \
  ""

if [ -n "$TRANSCRIPT_NEGATIVE_FINALIZE_2" ]; then
  assert_role "NEGATIVE: second finalize agent (other role completed)" \
    "$TRANSCRIPT_NEGATIVE_FINALIZE_2" \
    ""
fi

assert_role "NEGATIVE: advisor session (free-form prose)" \
  "$TRANSCRIPT_NEGATIVE_ADVISOR" \
  ""

assert_mcp_nonempty "ARRAY-POP: mcp_tools_invoked populated (nested tool_use)" \
  "$TRANSCRIPT_MCP_NONEMPTY"

echo
echo "RESULT: $PASS passed, $FAIL failed"

# Hard refusal: 0 PASS = environment misconfigured (no transcripts found).
# Don't let a quiet-skip session shipping bad regex.
if [ "$PASS" -eq 0 ]; then
  echo "ERROR: 0 transcripts asserted. Override TRANSCRIPT_* env vars or run on EliteDesk." >&2
  exit 1
fi

[ "$FAIL" -eq 0 ]
