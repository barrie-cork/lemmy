#!/usr/bin/env bash
# Brehon pilot-seed scenario: EMERGENCY REMOVAL (ADR-013) -> EmergencyRemove.
# 🚧 STUB (not run live)
#
# INTENDED FLOW (phase-4, ADR-013 mandatory path, <2s acceptance target):
#   fresh post -> admin POSTs the emergency-remove route -> case -> EmergencyRemove
#   -> governance_case_after_transition fires `emergency_remove` -> bridge provisions
#   an EMERGENCY room (emergency-case-N) and invites LEGAL_CONTACT_MXID (@legal:localhost)
#   -> jury reviews post-facto.
#
# KNOWN UNKNOWNS to resolve before promoting to ✅:
#   - Exact route + payload for admin_emergency_remove: grep -rn 'admin_emergency_remove'
#     crates/api/routes/src/lib.rs for the path; read the handler for the request struct
#     (likely {post_id|target_id, reason}). It IS a governance_case_after_transition
#     callsite (confirmed in the registry), so the hook fires -- but the emergency room
#     provisioner path (provision_emergency_room) has NEVER run live.
#   - LEGAL_CONTACT_MXID config: the emergency room invites a legal contact; confirm the
#     env/config key + that @legal:localhost (or configured) exists / is invited.
#   - LATENCY: measure API-call -> bridge_room row present. ADR-013/m2 target <2s.
#     Fire-and-forget returns fast; measure the actual provisioning latency from
#     bridge log timestamps, not the API response time.
#
# VERIFY (three surfaces): moderation_case.status=EmergencyRemove + content actually
#   removed in Lemmy; governance_log `emergency_removed` + chain intact; bridge_room
#   row (case_id, room_type='emergency') + legal-contact invite in bridge logs; latency <2s.

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

echo "RESULT=NOT_IMPLEMENTED (seed-emergency.sh is a documented stub -- see header + README)" >&2
echo "NEXT: resolve the admin_emergency_remove route+payload + LEGAL_CONTACT config, then implement + measure <2s latency." >&2
exit 2
