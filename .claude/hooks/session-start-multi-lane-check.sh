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
  # phase-v1-*, phase-v2-*, phase-m1-*, phase-m2-*, phase-m3-*,
  # phase-brehon-*; not chore branches or feature branches that don't
  # get a dedicated lane). The m1/m2/m3 patterns were added 2026-06-07
  # — the ADR-016 V2→M1/M2/M3 rename produced phase-m2-* branch names
  # (e.g. phase-m2-late-1) that the original v1/v2-only match skipped,
  # so the hook silently ignored every M-track lane.
  case "$WT_BRANCH" in
    phase-v1-*|phase-v2-*|phase-m1-*|phase-m2-*|phase-m3-*|phase-brehon-*) ;;
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

# --- Lane-mode drift check (added 2026-06-07, m2-late-1 T1 RCA) -----------
# A lane declared Mode A (dedicated worktree) but operated Mode B (no
# worktree, bare checkout in canonical) was the originating drift behind
# the T1 validate-pending-laptop failure. This block compares the DECLARED
# mode (lane_mode: field in the lane's bootstrap handover) against REALITY
# (does a brehon-fork-<lane> worktree exist?). WARN-not-FAIL on mismatch.
#
# Only meaningful when this session is on a phase branch (the lane is the
# point of comparison). On governance-v0 (canonical), skip — canonical is
# allowed to drive any lane in Mode B.
case "$CURRENT_BRANCH" in
  phase-v1-*|phase-v2-*|phase-m1-*|phase-m2-*|phase-m3-*|phase-brehon-*)
    # Derive the lane slug: strip the phase-<track>- prefix.
    #   phase-m2-late-1 -> m2-late-1 ; phase-v1-RT-r3 -> v1-RT-r3
    LANE_SLUG="${CURRENT_BRANCH#phase-}"
    # Bootstrap handover candidates: <lane>-bootstrap.md, or a prefix match
    # (phase-m2-late-1's handover is m2-late-bootstrap.md — drop trailing -N).
    REPO_ROOT="$CURRENT_WORKTREE"
    # The canonical checkout holds the handovers; if this is a lane worktree
    # the handovers are shared via the same .claude/ tree, so REPO_ROOT works
    # for both. Try exact then de-suffixed slug.
    HANDOVER=""
    for cand in "$LANE_SLUG" "${LANE_SLUG%-*}"; do
      f="$REPO_ROOT/.claude/PRPs/handovers/${cand}-bootstrap.md"
      if [ -f "$f" ]; then HANDOVER="$f"; break; fi
    done
    if [ -n "$HANDOVER" ]; then
      DECLARED_MODE=$(grep -m1 '^lane_mode:' "$HANDOVER" 2>/dev/null | sed -E 's/^lane_mode:[[:space:]]*([AB]).*/\1/' || true)
      if [ -n "$DECLARED_MODE" ]; then
        # Reality: does a lane worktree exist on this branch?
        if git worktree list 2>/dev/null | grep -qE "brehon-fork-[^ ]+[[:space:]].*\[${CURRENT_BRANCH}\]"; then
          ACTUAL_MODE="A"
        else
          ACTUAL_MODE="B"
        fi
        if [ "$DECLARED_MODE" != "$ACTUAL_MODE" ]; then
          echo "session-start-multi-lane-check WARN: lane-mode DRIFT for [$CURRENT_BRANCH]" >&2
          echo "  Declared (handover lane_mode:): Mode $DECLARED_MODE" >&2
          echo "  Actual (worktree reality):      Mode $ACTUAL_MODE" >&2
          echo "  Handover: $HANDOVER" >&2
          if [ "$DECLARED_MODE" = "A" ]; then
            echo "  Mode A declared but no lane worktree — create it (git worktree add ../brehon-fork-<lane> $CURRENT_BRANCH) or flip the handover to lane_mode: B." >&2
            echo "  In Mode B do NOT bare-checkout this branch in canonical; validate-pending-laptop uses a throwaway worktree (advisor-validation.md Sequence step 1)." >&2
          fi
        fi
      fi
    fi
    ;;
esac

exit 0
