#!/usr/bin/env bash
# Brehon pilot-seed scenario: EMERGENCY REMOVAL (ADR-013) -> EmergencyRemove.
# 🚧 STUB -- not run live; blocked on one code gap described below.
#
# ARCHITECTURE (confirmed by source reading 2026-06-13):
#
#   The internal function emergency_remove_open_case() in
#   crates/api/api/src/governance/admin_emergency_remove.rs is fully
#   implemented:
#     1. Flips target.removed = true (post/comment/community)
#     2. Opens a ModerationCase with status=EmergencyRemove
#     3. Seats a post-facto jury (reuses select_eligible_jurors)
#     4. Emits "emergency_removed" governance_log (visibility:"extra-visible")
#
#   The bridge's provision_emergency_room() is also fully implemented
#   (services/bridge/src/room_provisioner.rs:393):
#     - Creates room alias: emergency-case-{case_id}
#     - Invites LEGAL_CONTACT_MXID (default: @legal:localhost)
#     - Writes bridge_room row (case_id, room_type="emergency")
#     - ADR-013: reported party is explicitly ABSENT (see comment at :429)
#
# GAP -- NO HTTP ROUTE (module docstring, line 6-7):
#   "v0 does NOT expose an HTTP surface for it -- the function exists so the
#    cross-cutting EmergencyRemove requirement is wired through the codebase
#    and so integration tests can exercise the code path."
#
#   The only registered route under /emergency-remove is:
#     POST /api/v4/governance/admin/emergency-remove/flag-bad-faith
#   which is a post-case action (flag the reporter as bad-faith), NOT the
#   case-opening route.
#
#   governance_case_after_transition() is NOT called by admin_emergency_remove.rs,
#   so the bridge never receives a room-event for EmergencyRemove transitions --
#   the bridge side is wired correctly but the Lemmy hook never fires.
#
# WHAT'S NEEDED TO UNBLOCK PHASE 4:
#   Add an HTTP handler that calls emergency_remove_open_case(), then calls
#   governance_case_after_transition(..., CaseStatus::EmergencyRemove).
#
#   Minimal request struct:
#     { post_id: i32, reason: string }   (or comment_id / community_id variants)
#   Route: POST /api/v4/governance/admin/emergency-remove  (or /emergency-remove/post)
#
#   After that handler exists, this script can be:
#     1. POST the emergency-remove route as admin -> get case_id
#     2. sleep 3 (fire-and-forget bridge POST)
#     3. Check bridge_room row (room_type="emergency")
#     4. Check @legal:localhost was invited (bridge logs)
#     5. Measure latency: API call timestamp -> bridge_room row timestamp (<2s ADR-013 target)
#     6. Check governance_log "emergency_removed" entry + chain intact
#
# LEGAL_CONTACT_MXID: configured in bridge via LEGAL_CONTACT_MXID env var,
#   default "@legal:localhost". The @legal:localhost user does NOT exist yet --
#   the invite will fire but the user won't be present. This is fine for phase-4
#   testing (invite fires = provisioner ran); real pilot would need the legal
#   contact registered.
#
# Usage (once implemented):  bash seed-emergency.sh [post_id]
# VERIFY (three surfaces):
#   moderation_case.status=EmergencyRemove + post.removed=true;
#   governance_log "emergency_removed" (visibility:extra-visible) + chain intact;
#   bridge_room row (case_id, room_type="emergency") + legal invite in bridge logs;
#   latency from API call to bridge_room row < 2s (ADR-013 target).

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

echo "RESULT=NOT_IMPLEMENTED (seed-emergency.sh is a documented stub -- see header)" >&2
echo "GAP: no HTTP route for emergency_remove_open_case(); add admin handler first, then implement this script." >&2
exit 2
