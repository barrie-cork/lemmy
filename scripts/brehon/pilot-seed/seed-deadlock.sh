#!/usr/bin/env bash
# Brehon pilot-seed scenario: jury DEADLOCK -> AdminReview.  🚧 STUB (not run live)
#
# INTENDED FLOW (phase-6 adversarial path, v1-JM-c):
#   fresh case -> Decided-path setup (post -> report -> assign-jury) -> all panel
#   jurors accept, but votes SPLIT so no JuryDecision meets threshold_count_snapshot
#   -> submit_jury_vote deadlock branch fires -> case -> AdminReview,
#   `jury_deadlock` governance_log entry, NO sanction / NO room power change.
#
# KNOWN UNKNOWNS to resolve before promoting to ✅:
#   - The vote split that guarantees no-quorum for a panel_size=5/quorum=3 case:
#     e.g. 2x remove_content + 2x no_action + 1x advisory_label (no decision hits 3).
#     Confirm the exact JuryDecision variants + that ALL panel jurors must vote
#     before the deadlock branch evaluates (read submit_jury_vote.rs::process_vote
#     step 5 -- the deadlock check fires when votes == panel_size_snapshot).
#   - Whether deadlock fires the governance_case_after_transition hook for
#     AdminReview (grep confirms admin? -- submit_jury_vote IS a callsite; verify
#     the AdminReview arm POSTs, and that the bridge does NOT provision/power-change).
#
# VERIFY (three surfaces): moderation_case.status=AdminReview; governance_log has
#   `jury_deadlock` with the tally payload + chain intact; bridge_room UNCHANGED
#   (deadlock must NOT provision a room or apply power-levels).

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

echo "RESULT=NOT_IMPLEMENTED (seed-deadlock.sh is a documented stub -- see header + README)" >&2
echo "NEXT: resolve the no-quorum vote split + confirm AdminReview hook/no-room behavior, then implement." >&2
exit 2
