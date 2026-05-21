#!/usr/bin/env bash
# SessionStart hook: WARN if another phase-v1-* worktree shows recent commit
# activity on a branch different from this session's CWD. Catches the
# concurrent-advisor-session race that surfaced 2026-05-21 (this session
# pushed to governance-v0 while another session was driving fed-in-c + the
# v1-dq-schema-r1 schema migration; see project_concurrent_advisor_sessions_2026_05_21.md
# and feedback_falsifiable_hypothesis_before_structural_fix.md).
#
# Event: SessionStart
# Timeout: 5000
#
# Exit 0 = continue (always; this hook is WARN-not-FAIL — mirrors
# pmd-canonical-guard.sh per .claude/rules/pmd-invariants.md §5 rationale).
#
# Detection rule:
#   1. Read `git worktree list` for all phase-v1-* / phase-* worktrees.
#   2. For each worktree NOT in this session's CWD, check the age of the
#      worktree branch's tip commit (`git log -1 --format=%ct`).
#   3. If any other worktree's tip is < THRESHOLD_MINUTES (default 30 min)
#      AND its branch differs from this session's branch → WARN.
#
# The 30-min threshold is a heuristic: the prior session likely had a write
# action within that window. False positives (worktree active but no commit
# yet) and false negatives (long-running session that hasn't committed) both
# exist — accepted because the hook is WARN-not-FAIL.

set -euo pipefail

# --- Threshold (minutes since last commit on the other branch's tip) ---
THRESHOLD_MINUTES="${SESSION_START_MULTI_LANE_THRESHOLD:-30}"

# --- Resolve current CWD branch + worktree path ---
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || true)
CURRENT_WORKTREE=$(git rev-parse --show-toplevel 2>/dev/null || true)

if [ -z "$CURRENT_BRANCH" ] || [ -z "$CURRENT_WORKTREE" ]; then
  # Not a git repo or git unavailable; skip silently.
  exit 0
fi

# --- Walk git worktree list ---
# Output format: `<path> <sha> [<branch>]` — one line per worktree.
# Bare repos and detached worktrees are skipped (no `[<branch>]`).
THRESHOLD_SECONDS=$((THRESHOLD_MINUTES * 60))
NOW=$(date +%s)
WARNED=0

while IFS= read -r line; do
  # Parse worktree path + branch from the format `<path> <sha> [<branch>]`.
  # Robust against paths with spaces by anchoring on the trailing `[<branch>]`.
  if ! [[ "$line" =~ ^(.+)[[:space:]]+[0-9a-f]+[[:space:]]+\[(.+)\]$ ]]; then
    continue
  fi
  WT_PATH="${BASH_REMATCH[1]}"
  WT_BRANCH="${BASH_REMATCH[2]}"

  # Trim trailing whitespace from path (the regex's `(.+)` captures
  # eagerly; the space before `[` is consumed but trailing tabs may remain).
  WT_PATH="${WT_PATH%% }"
  WT_PATH="${WT_PATH%%	}"

  # Skip this session's own worktree.
  if [ "$WT_BRANCH" = "$CURRENT_BRANCH" ]; then
    continue
  fi

  # Only consider phase branches (the multi-lane discipline applies to
  # phase-v1-*, phase-*, phase-brehon-*; not chore branches or feature
  # branches that don't get a dedicated lane).
  case "$WT_BRANCH" in
    phase-v1-*|phase-v2-*|phase-brehon-*) ;;
    *) continue ;;
  esac

  # Get last commit timestamp on the branch tip.
  TIP_TS=$(git log -1 --format=%ct "$WT_BRANCH" 2>/dev/null || true)
  if [ -z "$TIP_TS" ]; then
    continue
  fi

  AGE=$((NOW - TIP_TS))
  if [ "$AGE" -lt "$THRESHOLD_SECONDS" ]; then
    if [ "$WARNED" -eq 0 ]; then
      echo "session-start-multi-lane-check WARN: another worktree has recent activity (≤${THRESHOLD_MINUTES} min) on a different branch" >&2
      echo "  This session: $CURRENT_WORKTREE [$CURRENT_BRANCH]" >&2
      WARNED=1
    fi
    AGE_MIN=$((AGE / 60))
    echo "  Other lane:   $WT_PATH [$WT_BRANCH] (last commit ${AGE_MIN}m ago)" >&2
  fi
done < <(git worktree list 2>/dev/null)

if [ "$WARNED" -eq 1 ]; then
  echo "  Action: before any state-changing call, confirm this CWD is the intended lane." >&2
  echo "  Per .claude/rules/multi-lane-worktree.md: lane-dedicated worktrees own phase-branch DQ writes; canonical brehon-fork is reserved for governance-v0 meta-edits." >&2
fi

exit 0
