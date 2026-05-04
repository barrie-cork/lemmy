#!/bin/bash

# SessionStart hook — pre-phase wrapper-script audit reminder.
#
# Enforces .claude/rules/pre-phase-harness-audit.md by detecting when a new
# session starts on a phase-* branch without a completed audit, and injecting
# a strong system reminder so the agent runs the four wrapper probes before
# task 1.
#
# The hook itself does not run cargo (each probe is a multi-minute build that
# would block session startup). It only detects the gap and surfaces it.
#
# Skip conditions (per the rule):
#   - Not on a phase-* branch (governance-v0, main, etc.)
#   - Audit-complete flag exists for this branch
#   - Audit logs already present from prior probes (mid-loop resume case)

set -euo pipefail

CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "")

if [[ -z "$CURRENT_BRANCH" ]]; then
  exit 0
fi

# Only fire on phase branches.
if [[ ! "$CURRENT_BRANCH" =~ ^phase- ]]; then
  exit 0
fi

# Sanitize branch name for filename use.
BRANCH_SAFE="${CURRENT_BRANCH//\//-}"
FLAG_FILE=".claude/audit-${BRANCH_SAFE}-complete.flag"

if [[ -f "$FLAG_FILE" ]]; then
  exit 0
fi

# CR #85: the documented skip condition includes "audit logs already present
# from prior probes (mid-loop resume case)". Treat any of the four expected
# audit logs as evidence the audit was started — don't re-emit the reminder.
if [[ -f ".claude/audit-cargo-check-p.log" ]] \
  || [[ -f ".claude/audit-cargo-check-features.log" ]] \
  || [[ -f ".claude/audit-cargo-test.log" ]] \
  || [[ -f ".claude/audit-cargo-test-negative.log" ]]; then
  exit 0
fi

REMINDER="Pre-phase wrapper audit not yet completed for branch ${CURRENT_BRANCH}.

Per .claude/rules/pre-phase-harness-audit.md, run the 4 probes BEFORE task 1:

  1. cmd //c \"scripts\\\\brehon\\\\cargo-check.bat -p lemmy_utils > .claude/audit-cargo-check-p.log 2>&1\"
  2. cmd //c \"scripts\\\\brehon\\\\cargo-check.bat -p lemmy_db_schema --features full > .claude/audit-cargo-check-features.log 2>&1\"
  3. cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/audit-cargo-test.log 2>&1\"
  4. cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/audit-cargo-test-negative.log 2>&1\" — expect NON-ZERO exit (exit-code propagation check)

Capture exit codes; tail each log; verify probe 4 prints non-zero. After all 4
pass, mark complete with:

  touch ${FLAG_FILE}

Skip ONLY if resuming a partially-failed loop or running a hotfix mini-phase
(per the rule's 'When to skip' section)."

python3 - "$REMINDER" <<'PY'
import json, sys
print(json.dumps({
    "hookSpecificOutput": {
        "hookEventName": "SessionStart",
        "additionalContext": sys.argv[1]
    }
}))
PY

exit 0
