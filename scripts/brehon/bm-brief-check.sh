#!/usr/bin/env bash
# bm-brief-check.sh — completeness gate for bm-task briefs
# Usage: bash scripts/brehon/bm-brief-check.sh <brief-path>
# Exit 0 = all checks pass. Exit 1 = one or more checks failed (brief is NOT ready to commit).
set -euo pipefail

BRIEF="${1:-}"
if [[ -z "$BRIEF" ]]; then
  echo "Usage: $0 <brief-path>" >&2; exit 1
fi
if [[ ! -f "$BRIEF" ]]; then
  echo "ERROR: brief not found: $BRIEF" >&2; exit 1
fi

FAIL=0
check() {
  local label="$1"; local pattern="$2"
  if grep -qiE "$pattern" "$BRIEF"; then
    echo "  OK  $label"
  else
    echo "FAIL  $label (pattern: $pattern)"
    FAIL=1
  fi
}

echo "=== bm-brief-check: $BRIEF ==="
check "§1 Role+dispatch line"        "role:bm-task"
check "§2 Scope (Phase + PR)"        "Phase:|phase-"
check "§2 load-bearing SHA (trunk or phase tip)" "[0-9a-f]{7,40}"
check "§4 Constraints present"       "## 4\. Constraints|##4\. Constraints"
check "§4 ≥8 hard-refusal items"     "NEVER|Never touch|never touch"
check "§5 Success signals"           "## 5\. Success|##5\. Success"
check "HANDOVER trailer schema"      "HANDOVER:|filesCreated:|filesModified:"
check "--repo barrie-cork/lemmy"     "barrie-cork/lemmy"
check "governance-v0 base"           "governance-v0"

echo "==="
if [[ $FAIL -ne 0 ]]; then
  echo "FAIL — brief is incomplete. Fix the above items before committing." >&2
  exit 1
fi
echo "PASS — brief passes completeness gate."
