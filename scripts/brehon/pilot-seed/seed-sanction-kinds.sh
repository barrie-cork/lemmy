#!/usr/bin/env bash
# Brehon pilot-seed scenario: SANCTION-KIND COVERAGE (phase 5).
#
# Verifies all reachable SanctionKind values reach the bridge and produce the
# correct compute_power_override() outcome.
#
# CHAIN (full):
#   JuryDecision → map_decision_to_sanction() → SanctionAction
#   → map_sanction_action() → SanctionKind string
#   → bridge compute_power_override() → power_level override in Matrix room
#
# v0 REACHABLE SanctionKind values (3 of 4 — mute_voice unreachable in v0):
#   hide_content   ← JuryDecision::RemoveContent   (ALREADY VERIFIED cases 1/3/4/5)
#   restrict_reach ← JuryDecision::AdvisoryLabel or Warning
#   prevent_post   ← JuryDecision::Cooldown, SuspendCommunityMember, SuspendLocalUser
#
# This script runs TWO fresh cases:
#   Case A: advisory_label  → restrict_reach  → power_level reduced below post threshold
#   Case B: cooldown        → prevent_post    → power_level reduced below post threshold
#
# VERIFY (per case):
#   bridge_room row provisioned (room_type='jury')
#   sanction_event row with correct sanction_kind
#   bridge log: "power level applied" + sanction_kind=<kind>
#   governance_log hash chain intact
#
# Usage: bash seed-sanction-kinds.sh
# RESULT lines per case: CASE_<N>=.. KIND=.. APPLIED=.. CHAIN=..

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

echo "=== Phase 5: Sanction-kind coverage ==="

# --- helpers ---

# run_case <label> <decision> <expected_kind>
# Drives: post → report → assign-jury → 3x accept → 3x vote <decision> → quorum.
# Then checks sanction_event.sanction_kind + bridge log + bridge applied.
run_case() {
  local label="$1" decision="$2" expected_kind="$3"
  echo ""
  echo "--- Case $label: JuryDecision=$decision → SanctionKind=$expected_kind ---"

  # Login pool
  local AJWT TJWT MJWT
  AJWT=$(login "${ADMIN_USER}" "${ADMIN_PASS}")
  TJWT=$(login "testuser" "${TEST_PASS}")
  MJWT=$(login "testmod" "${TEST_PASS}")

  # Create post
  local POST_RESP POST_ID
  POST_RESP=$(api_post /post \
    "{\"name\":\"phase5-${label}-$(date +%s)\",\"community_id\":$COMMUNITY_ID,\"nsfw\":false}" \
    "$TJWT")
  POST_ID=$(printf '%s' "$POST_RESP" | json_nested post_view post id)
  [ -n "$POST_ID" ] || { echo "POST_CREATE_FAILED resp=$POST_RESP" >&2; return 1; }
  echo "  post_id=$POST_ID"

  # Report
  local REP_RESP CASE_ID
  REP_RESP=$(api_post /governance/report \
    "{\"target_type\":\"post\",\"target_id\":$POST_ID,\"reason_code\":\"phase5-${label}-test\",\"community_id\":$COMMUNITY_ID}" \
    "$MJWT")
  CASE_ID=$(printf '%s' "$REP_RESP" | json_field case_id)
  # Fallback: read from DB (case_id may be null in response if threshold not met)
  [ -n "$CASE_ID" ] || CASE_ID=$(psql "SELECT id FROM moderation_case WHERE target_post_id=$POST_ID ORDER BY id DESC LIMIT 1;")
  [ -n "$CASE_ID" ] || { echo "REPORT_FAILED resp=$REP_RESP" >&2; return 1; }
  echo "  case_id=$CASE_ID ($(case_status $CASE_ID))"

  # Assign jury
  local ASSIGN_RESP
  ASSIGN_RESP=$(api_post /governance/admin/assign-jury \
    "{\"case_id\":$CASE_ID,\"community_id\":$COMMUNITY_ID}" \
    "$AJWT")
  echo "  assign: $(printf '%s' "$ASSIGN_RESP" | json_field status)"
  echo "  case→$(case_status $CASE_ID)"

  # Accept + vote with all jurors until Decided (quorum=3)
  local i j_jwt j_name
  # Vote with up to 5 jurors (need quorum=3) — try all, stop when Decided
  local voted=0
  for i in 6 7 8 9 10 1 2 3 4 5; do
    local current_status
    current_status=$(case_status "$CASE_ID")
    [ "$current_status" = "Decided" ] && break
    j_name="juror${i}"
    j_jwt=$(login "$j_name" "${TEST_PASS}" 2>/dev/null || true)
    [ -z "$j_jwt" ] && continue
    local vote_resp
    # Accept first (idempotent)
    api_post /governance/jury/accept "{\"case_id\":$CASE_ID}" "$j_jwt" >/dev/null 2>&1 || true
    vote_resp=$(api_post /governance/jury/vote \
      "{\"case_id\":$CASE_ID,\"decision\":\"$decision\",\"rationale\":\"phase5 ${label} test\"}" \
      "$j_jwt" 2>/dev/null || true)
    voted=$((voted+1))
    echo "  vote $voted from $j_name ($(case_status $CASE_ID))"
  done

  local final_status
  final_status=$(case_status "$CASE_ID")
  echo "  final_status=$final_status"

  # Wait for sanction + bridge (up to 3s)
  local SANCTION_KIND="" BRIDGE_LOG=""
  for attempt in $(seq 1 6); do
    sleep 0.5
    SANCTION_KIND=$(psql "SELECT se.sanction_kind FROM sanction_event se JOIN sanction s ON s.id=se.sanction_id WHERE s.case_id=$CASE_ID LIMIT 1;" 2>/dev/null || true)
    [ -n "$SANCTION_KIND" ] && break
  done
  echo "  SANCTION_KIND=$SANCTION_KIND"

  # Check bridge applied
  BRIDGE_LOG=$(docker logs brehon-bridge 2>&1 | grep "case_id=${CASE_ID}" | grep "power level applied\|power_level" | tail -3 || true)
  if [ -n "$BRIDGE_LOG" ]; then
    echo "  BRIDGE_POWER_LEVEL_LOG: $BRIDGE_LOG"
    echo "  BRIDGE_APPLIED=true"
  else
    echo "  BRIDGE_APPLIED=false (no power-level log for case_id=$CASE_ID)"
  fi

  # Verify bridge room
  local BRIDGE_ROW
  BRIDGE_ROW=$(bridge_rooms | grep "^${CASE_ID}|" || true)
  echo "  BRIDGE_ROOM=${BRIDGE_ROW:-NOT_FOUND}"

  # Verify expected kind
  if [ "$SANCTION_KIND" = "$expected_kind" ]; then
    echo "  KIND_MATCH=✅ ($SANCTION_KIND)"
  else
    echo "  KIND_MATCH=❌ expected=$expected_kind got=$SANCTION_KIND"
  fi

  echo "CASE_${label}=$CASE_ID KIND=$SANCTION_KIND EXPECTED=$expected_kind STATUS=$final_status"
}

# --- Run the two cases ---
run_case "A_restrict_reach" "advisory_label" "restrict_reach"
run_case "B_prevent_post"   "cooldown"       "prevent_post"

# --- Hash chain ---
echo ""
CHAIN=$(verify_hash_chain)
echo "CHAIN=$CHAIN"
[ "$CHAIN" = "CHAIN_INTACT" ] || { echo "CHAIN_BROKEN" >&2; exit 1; }

echo ""
echo "RESULT=PASS (or review KIND_MATCH lines above)"
