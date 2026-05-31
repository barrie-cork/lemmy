#!/usr/bin/env bash
# PreToolUse hook: fires before Bash tool calls matching "git commit".
# Guards against cross-session git add -A sweeping unintended staged files.
# Warn-only (exit 0 always) — does not block commits to avoid false-positive paralysis.
# Pairs with feedback_cross_session_commit_attribution_collision.md §3 mitigation 3.

STAGED=$(git diff --cached --name-only 2>/dev/null)
if [ -z "$STAGED" ]; then exit 0; fi

SESSION_INTENT_FILE=".claude/.staged-intent"
if [ ! -f "$SESSION_INTENT_FILE" ]; then exit 0; fi

UNEXPECTED=""
while IFS= read -r f; do
  if ! grep -qF "$f" "$SESSION_INTENT_FILE" 2>/dev/null; then
    UNEXPECTED="${UNEXPECTED}\n  ${f}"
  fi
done <<< "$STAGED"

if [ -n "$UNEXPECTED" ]; then
  echo "WARN [pre-commit-session-guard]: staged files not in session intent (.claude/.staged-intent):" >&2
  printf "%b\n" "$UNEXPECTED" >&2
  echo "  Review: git diff --cached --name-only" >&2
  echo "  To acknowledge: add the files to .claude/.staged-intent before retrying" >&2
fi
exit 0
