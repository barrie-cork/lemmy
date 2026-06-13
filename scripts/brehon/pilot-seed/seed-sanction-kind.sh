#!/usr/bin/env bash
# Brehon pilot-seed scenario: SANCTION-KIND coverage -- each SanctionKind -> its
# m.room.power_levels translation.  🚧 STUB (not run live; only hide_content done)
#
# INTENDED FLOW (phase-5): a case WITH a provisioned room -> a sanction of the
# given KIND -> bridge `compute_power_override` maps it to a power-level + reason_code
# -> verify the subject's PL in actual Tuwunel room state.
#
# SanctionKind -> bridge behavior (from sanction_handler.rs::compute_power_override):
#   prevent_post    -> events_default - 1  | power_level_reduced_below_post_threshold
#   mute_voice      -> (voice-specific PL) | (mute_voice branch -- READ the actual level)
#   hide_content    -> events_default - 1  | redaction_not_available_in_m2_late_2  [✅ done, case 5]
#   restrict_reach  -> events_default - 1  | restrict_reach_translated_to_power_level_reduction
#
# KNOWN UNKNOWNS to resolve before promoting to ✅:
#   - HOW the jury's decision selects the SanctionKind. Prefer a REAL quorum-driven
#     sanction where the kind is chosen by the vote/decision (not a hand-POSTed
#     sanction-event), so the test exercises the real selection path. Find where
#     submit_jury_vote maps JuryDecision -> SanctionKind.
#   - mute_voice's actual power level (may differ from -1) -- read the branch.
#   - The M2 scope limits baked into reason codes (redaction_not_available...,
#     ban==power-reduction-not-membership-ban) are DELIBERATE, not bugs -- confirm
#     they behave as the reason code claims.
#
# Usage (once implemented):  bash seed-sanction-kind.sh <prevent_post|mute_voice|restrict_reach|hide_content>
# VERIFY (three surfaces): sanction_event.sanction_kind; bridge applied:true +
#   expected reason_code in logs; Tuwunel m.room.power_levels for the subject == table.

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

KIND="${1:-}"
echo "RESULT=NOT_IMPLEMENTED (seed-sanction-kind.sh is a documented stub -- see header + README) requested_kind=${KIND:-none}" >&2
echo "NEXT: find JuryDecision->SanctionKind mapping for a real quorum-driven sanction, then implement per-kind PL verification." >&2
exit 2
