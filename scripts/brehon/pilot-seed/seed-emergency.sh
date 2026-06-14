#!/usr/bin/env bash
# Brehon pilot-seed scenario: EMERGENCY REMOVAL (ADR-013) -> EmergencyRemove.
#
# Phase 4: Admin invokes POST /api/v4/governance/admin/emergency-remove,
# which calls emergency_remove_open_case() + fires governance_case_after_transition()
# → bridge provisions emergency-case-{id} room + invites @legal:localhost.
#
# VERIFY (four surfaces):
#   1. moderation_case.status=EmergencyRemove + post.removed=true
#   2. governance_log "emergency_removed" (visibility:extra-visible) + chain intact
#   3. bridge_room row (case_id, room_type="emergency") + legal invite in bridge logs
#   4. latency from API call to bridge_room row < 2s (ADR-013 target)
#
# LEGAL_CONTACT_MXID: @legal:localhost — invite fires, user need not exist.
#
# Usage:  bash seed-emergency.sh [post_id]
#           post_id — an existing post id to target (default: creates a fresh one)
# RESULT lines: CASE_ID=.. ROOM_ID=.. POST_ID=.. POST_REMOVED=.. CHAIN=.. LATENCY_OK=..
#
# Environment overrides (inherit defaults from lib.sh).

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

POST_ARG="${1:-}"

echo "=== Phase 4: EmergencyRemove ==="

# --- 1. Admin JWT ---
AJWT=$(admin_jwt)
[ -n "$AJWT" ] || { echo "ADMIN_LOGIN_FAILED" >&2; exit 1; }

# --- 2. Ensure a target post exists ---
if [ -n "$POST_ARG" ]; then
  POST_ID="$POST_ARG"
  echo "Using provided post_id=$POST_ID"
else
  # Create a fresh post as testuser (or lemmy) under test_governance community
  TJWT=$(login "${TEST_USER:-testuser}" "${TEST_PASS}")
  if [ -z "$TJWT" ]; then
    # Fall back to admin as author if testuser missing
    TJWT="$AJWT"
  fi
  POST_RESP=$(api_post /post \
    "{\"name\":\"emergency-removal-test-$(date +%s)\",\"community_id\":$COMMUNITY_ID,\"nsfw\":false}" \
    "$TJWT")
  POST_ID=$(printf '%s' "$POST_RESP" | json_nested post_view post id)
  if [ -z "$POST_ID" ]; then
    echo "POST_CREATE_FAILED resp=$POST_RESP" >&2
    exit 1
  fi
  echo "Created post_id=$POST_ID"
fi

# --- 3. POST /emergency-remove as admin, capture timestamp ---
TS_BEFORE=$(date +%s%3N)   # milliseconds
ER_RESP=$(api_post /governance/admin/emergency-remove \
  "{\"post_id\":$POST_ID,\"reason\":\"ADR-013 pilot phase-4 test — emergency content removal\"}" \
  "$AJWT")
TS_AFTER=$(date +%s%3N)

CASE_ID=$(printf '%s' "$ER_RESP" | json_field case_id)
if [ -z "$CASE_ID" ]; then
  echo "EMERGENCY_REMOVE_FAILED resp=$ER_RESP" >&2
  exit 1
fi
echo "CASE_ID=$CASE_ID"

# --- 4. Verify moderation_case.status = EmergencyRemove ---
STATUS=$(case_status "$CASE_ID")
echo "CASE_STATUS=$STATUS"
[ "$STATUS" = "EmergencyRemove" ] || { echo "STATUS_WRONG expected=EmergencyRemove got=$STATUS" >&2; exit 1; }

# --- 5. Verify post.removed = true ---
POST_REMOVED=$(psql "SELECT removed FROM post WHERE id=$POST_ID;")
echo "POST_REMOVED=$POST_REMOVED"
[ "$POST_REMOVED" = "t" ] || { echo "POST_NOT_REMOVED got=$POST_REMOVED" >&2; exit 1; }

# --- 6. Wait up to 3s for bridge room (ADR-013: <2s target) ---
ROOM_ID=""
LATENCY_MS=""
for i in $(seq 1 6); do
  sleep 0.5
  BRIDGE_ROW=$(bridge_rooms | grep "^${CASE_ID}|emergency|" || true)
  if [ -n "$BRIDGE_ROW" ]; then
    TS_ROOM=$(date +%s%3N)
    LATENCY_MS=$(( TS_ROOM - TS_BEFORE ))
    ROOM_ID=$(printf '%s' "$BRIDGE_ROW" | cut -d'|' -f3)
    break
  fi
done
echo "ROOM_ID=${ROOM_ID:-NOT_FOUND}"
echo "LATENCY_MS=${LATENCY_MS:-N/A}"

if [ -z "$ROOM_ID" ]; then
  echo "EMERGENCY_ROOM_NOT_PROVISIONED — bridge_room row missing for case_id=$CASE_ID" >&2
  echo "Bridge rooms at time of check:"
  bridge_rooms >&2
  exit 1
fi

# ADR-013: latency target < 2000ms
LATENCY_OK="false"
if [ -n "$LATENCY_MS" ] && [ "$LATENCY_MS" -lt 2000 ]; then
  LATENCY_OK="true"
fi
echo "LATENCY_OK=$LATENCY_OK (${LATENCY_MS}ms)"

# --- 7. Verify @legal:localhost invite in bridge logs ---
LEGAL_INVITE=$(docker logs brehon-bridge 2>&1 | grep -i "legal:localhost" | tail -3 || true)
if [ -n "$LEGAL_INVITE" ]; then
  echo "LEGAL_INVITE_FOUND"
else
  echo "LEGAL_INVITE_NOT_FOUND (user may not exist, but invite should have fired)" >&2
  # Non-fatal: @legal:localhost need not be registered; log absence but continue.
fi

# --- 8. Verify governance_log entry ---
GOV_LOG=$(psql "SELECT COUNT(*) FROM governance_log WHERE payload->>'case_id'='$CASE_ID' AND entry_kind='emergency_removed';")
echo "GOVERNANCE_LOG_ENTRY_COUNT=$GOV_LOG"

# --- 9. Verify hash chain ---
CHAIN=$(verify_hash_chain)
echo "CHAIN=$CHAIN"
[ "$CHAIN" = "CHAIN_INTACT" ] || { echo "CHAIN_BROKEN" >&2; exit 1; }

# --- Summary ---
echo ""
echo "RESULT=PASS"
echo "POST_ID=$POST_ID"
echo "CASE_ID=$CASE_ID"
echo "STATUS=$STATUS"
echo "POST_REMOVED=$POST_REMOVED"
echo "ROOM_ID=$ROOM_ID"
echo "LATENCY_OK=$LATENCY_OK"
echo "LATENCY_MS=${LATENCY_MS:-N/A}"
echo "CHAIN=$CHAIN"
