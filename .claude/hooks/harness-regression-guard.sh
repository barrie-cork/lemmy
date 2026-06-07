#!/usr/bin/env bash
# SessionStart hook: WARN if a rule file that was intentionally removed by a
# prior harness-audit reappears as an always-load rule in .claude/rules/.
#
# Motivation (incident harness-audit-2026-06-07): the 2026-05-31 consolidation
# of circuit-breaker + escalation + integrator + post-task-retro into
# universal-guards.md was silently reverted by the cross-repo sync tool's
# auto-commit on 2026-06-04 (commit 999317082), re-introducing ~2.6k tokens of
# duplicate always-load content. It went undetected for 3 days because nothing
# re-checks that a previously-applied context-budget trim is STILL applied.
# This guard closes that cross-session detection gap.
#
# Event: SessionStart
# Timeout: 5000
#
# Exit 0 = continue (always; WARN-not-FAIL, mirrors session-start-multi-lane-check.sh
# + pmd-canonical-guard.sh per .claude/rules/pmd-invariants.md §5 rationale).
#
# Source of truth: .claude/refs/harness-deleted-rules.txt — a tombstone ledger,
# one rule basename per line (e.g. `circuit-breaker.md`). Lines starting with `#`
# and blank lines are ignored. The harness-audit skill appends a basename here
# whenever it ships a P1 deletion. A rule on this list that is ALSO present as a
# standalone file in .claude/rules/ is a regression — it loads at session start
# and (per the originating audit) duplicates content already consolidated
# elsewhere.

set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || true)
if [ -z "$REPO_ROOT" ]; then
  # Not a git repo or git unavailable; skip silently.
  exit 0
fi

LEDGER="$REPO_ROOT/.claude/refs/harness-deleted-rules.txt"
RULES_DIR="$REPO_ROOT/.claude/rules"

# No ledger → nothing to guard (the guard is opt-in via the ledger's existence).
if [ ! -f "$LEDGER" ]; then
  exit 0
fi

REGRESSED=()
while IFS= read -r raw || [ -n "$raw" ]; do
  # Strip CR (Windows checkout), leading/trailing whitespace.
  line="${raw%$'\r'}"
  line="${line#"${line%%[![:space:]]*}"}"
  line="${line%"${line##*[![:space:]]}"}"
  # Skip blanks + comments.
  [ -z "$line" ] && continue
  case "$line" in \#*) continue ;; esac

  # A ledger entry is a bare basename. Reject anything with a slash (defensive:
  # the ledger lists rule BASENAMES, not paths).
  case "$line" in */*) continue ;; esac

  if [ -f "$RULES_DIR/$line" ]; then
    REGRESSED+=("$line")
  fi
done < "$LEDGER"

if [ "${#REGRESSED[@]}" -gt 0 ]; then
  echo "harness-regression-guard WARN: ${#REGRESSED[@]} intentionally-removed rule(s) have REAPPEARED as always-load:" >&2
  for r in "${REGRESSED[@]}"; do
    echo "  - .claude/rules/$r  (on the harness-audit delete-ledger; duplicate always-load content)" >&2
  done
  echo "  Likely cause: the cross-repo sync tool (homeserver/scripts/sync-shared-skills.sh) re-shipped them." >&2
  echo "  Check homeserver RULE_EXCLUDE[brehon-fork] still lists these; then re-delete locally." >&2
  echo "  Ledger: .claude/refs/harness-deleted-rules.txt  ·  origin incident: .claude/PRPs/reports/harness-audit-2026-06-07.md" >&2
fi

exit 0
