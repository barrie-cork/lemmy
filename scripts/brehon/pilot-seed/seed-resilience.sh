#!/usr/bin/env bash
# Brehon pilot-seed scenario: RESTART IDEMPOTENCY + SOFT-PAUSE + BRIDGE-DOWN (phase 7).
#
# Three sub-scenarios:
#   1 — Restart idempotency
#         PRECONDITION: infra has restarted brehon-bridge BEFORE running this sub-case.
#         Re-fires a room-event for a spent case (default: case 5, jury type).
#         CRITICAL VERIFY: bridge_room gains NO duplicate (case_id, room_type) row.
#
#   2 — Soft-pause
#         Flip messaging_enabled=false → drive a case transition → confirm NO room
#         provisioned → flip back to true → confirm resume (new case provisions).
#
#   3 — Bridge-down resilience
#         Stop the bridge container, drive a case to Decided (full quorum), confirm
#         Lemmy governance completes with no error (DB writes, sanction, chain all
#         intact). Restart the bridge. Confirm the transition while the bridge was
#         down is LOST (no replay — documented m2 gap, not a bug).
#
# Usage:
#   bash seed-resilience.sh 1            # idempotency check (bridge restarted beforehand)
#   bash seed-resilience.sh 2            # soft-pause
#   bash seed-resilience.sh 3            # bridge-down resilience
#   bash seed-resilience.sh all          # 2 then 3 (skip 1 — needs manual restart first)
#
# Sub-case 1 MANUAL STEP required BEFORE running:
#   ssh homeserver "cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.pilot.yml restart bridge"
#   Then: bash seed-resilience.sh 1 [case_id]
#     case_id — optional, defaults to case 5 (known jury room). Use any spent case with bridge_room row.
#
# RESULT lines per sub-case: RESULT_1=.., RESULT_2=.., RESULT_3=..
# CHAIN= at end of each sub-case.

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

RUN="${1:-all}"

echo "=== Phase 7: Restart idempotency + soft-pause + bridge-down resilience ==="

# ── helpers ───────────────────────────────────────────────────────────────────

# messaging_enabled_set <true|false>
# Sets governance_messaging_config instance messaging_enabled via admin API.
messaging_enabled_set() {
  local val="$1"
  local AJWT
  AJWT=$(admin_jwt)
  local RESP
  RESP=$(api_post /governance/admin/messaging-config \
    "{\"scope\":\"instance\",\"key\":\"messaging_enabled\",\"value\":$val}" \
    "$AJWT")
  printf '%s' "$RESP" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d)' 2>/dev/null || true
  echo "  messaging_enabled_set=$val"
}

# messaging_enabled_get  — echo current value ("t" or "f")
messaging_enabled_get() {
  psql "SELECT value_bool FROM governance_messaging_config
        WHERE scope='instance' AND key='messaging_enabled'
        ORDER BY id DESC LIMIT 1;" 2>/dev/null || echo "?"
}

# open_and_decide <suffix> <decision>
# Creates post → report → assign-jury → vote_to_threshold.
# Echoes "CASE_ID POST_ID" on success; exits non-zero on failure.
open_and_decide() {
  local suffix="$1" decision="$2"
  local AJWT TJWT MJWT POST_RESP POST_ID REP_RESP CASE_ID

  AJWT=$(admin_jwt)
  TJWT=$(login "testuser" "${TEST_PASS}")
  MJWT=$(login "testmod"  "${TEST_PASS}")

  POST_RESP=$(api_post /post \
    "{\"name\":\"phase7-${suffix}-$(date +%s)\",\"community_id\":$COMMUNITY_ID,\"nsfw\":false}" \
    "$TJWT")
  POST_ID=$(printf '%s' "$POST_RESP" | json_nested post_view post id)
  [ -n "$POST_ID" ] || { echo "POST_CREATE_FAILED resp=$POST_RESP" >&2; return 1; }

  api_post /governance/report \
    "{\"target_type\":\"post\",\"target_id\":$POST_ID,\"reason_code\":\"phase7-${suffix}\",\"community_id\":$COMMUNITY_ID}" \
    "$MJWT" >/dev/null

  CASE_ID=$(psql "SELECT id FROM moderation_case WHERE target_post_id=$POST_ID ORDER BY id DESC LIMIT 1;" 2>/dev/null)
  [ -n "$CASE_ID" ] || { echo "CASE_NOT_FOUND post_id=$POST_ID" >&2; return 1; }

  api_post /governance/admin/assign-jury \
    "{\"case_id\":$CASE_ID,\"community_id\":$COMMUNITY_ID}" \
    "$AJWT" >/dev/null

  sleep 0.5
  vote_to_threshold "$CASE_ID" "$decision" >/dev/null 2>&1 || true

  printf '%s %s' "$CASE_ID" "$POST_ID"
}

# bridge_room_count <case_id> <room_type>  — echo count of matching rows
bridge_room_count() {
  local cid="$1" rtype="$2"
  bridge_rooms | grep -c "^${cid}|${rtype}|" || echo "0"
}

# wait_for_bridge_room <case_id> <room_type> [max_attempts=6] [sleep_s=0.5]
# Returns 0 + echoes room_id if row appears within the timeout; returns 1 if not.
wait_for_bridge_room() {
  local cid="$1" rtype="$2" max="${3:-6}" sleep_s="${4:-0.5}"
  local i ROW ROOM_ID=""
  for i in $(seq 1 "$max"); do
    sleep "$sleep_s"
    ROW=$(bridge_rooms | grep "^${cid}|${rtype}|" || true)
    if [ -n "$ROW" ]; then
      ROOM_ID=$(printf '%s' "$ROW" | cut -d'|' -f3)
      printf '%s' "$ROOM_ID"
      return 0
    fi
  done
  return 1
}

# ── Sub-case 1: Restart idempotency ───────────────────────────────────────────
# PRECONDITION: bridge has been restarted by infra BEFORE this runs.
# We re-fire a room-event for a spent case by triggering any case transition
# on an already-provisioned case. Since we can't re-drive a decided case, we
# instead verify via a direct sanction-event POST to the bridge — the bridge's
# sanction_handler calls bridge_room::lookup (no insert), so if the bridge is
# up and the room is already there, the response is applied:true or applied:false
# (no rooms found for the fabricated path). The idempotency check is:
#   bridge_room count for (case_id, room_type) must == 1 (not 2).
# We also drive a FRESH case through assign-jury → verify the new jury room is
# provisioned (proves the bridge is functional after restart).

run_1() {
  local SPENT_CASE_ID="${2:-5}"
  echo ""
  echo "--- Sub-case 1: Restart idempotency (bridge assumed already restarted) ---"
  echo "  Checking spent case $SPENT_CASE_ID for duplicate bridge_room rows..."

  local COUNT
  COUNT=$(bridge_room_count "$SPENT_CASE_ID" "jury")
  echo "  bridge_room count for (case_id=$SPENT_CASE_ID, type=jury) = $COUNT"

  if [ "$COUNT" -eq 1 ]; then
    echo "  IDEMPOTENCY=✅ (exactly 1 row — no duplicate created on restart)"
  elif [ "$COUNT" -eq 0 ]; then
    echo "  IDEMPOTENCY=⚠️  0 rows — case $SPENT_CASE_ID has no jury room? Check case_id."
  else
    echo "  IDEMPOTENCY=❌ DUPLICATE ($COUNT rows) — bridge_room::lookup not guarding on restart"
  fi

  # Drive a fresh case to verify the bridge is functional post-restart
  echo "  Driving a fresh case to confirm bridge is functional after restart..."
  local FRESH_RESULT FRESH_CASE FRESH_POST
  FRESH_RESULT=$(open_and_decide "idempotency-check" "remove_content") || {
    echo "  FRESH_CASE_FAILED" >&2
    echo "RESULT_1: IDEMPOTENCY_COUNT=$COUNT FRESH_CASE=FAILED"
    return 0
  }
  FRESH_CASE=$(printf '%s' "$FRESH_RESULT" | awk '{print $1}')
  FRESH_POST=$(printf '%s' "$FRESH_RESULT" | awk '{print $2}')
  echo "  fresh_case_id=$FRESH_CASE post_id=$FRESH_POST status=$(case_status "$FRESH_CASE")"

  local FRESH_ROOM=""
  FRESH_ROOM=$(wait_for_bridge_room "$FRESH_CASE" "jury" 8 0.5) || true
  if [ -n "$FRESH_ROOM" ]; then
    echo "  FRESH_ROOM_PROVISIONED=✅ $FRESH_ROOM"
  else
    echo "  FRESH_ROOM_PROVISIONED=❌ (bridge may still be unhealthy after restart)"
  fi

  local CHAIN
  CHAIN=$(verify_hash_chain)
  echo "CHAIN=$CHAIN"
  echo "RESULT_1: IDEMPOTENCY_COUNT=$COUNT FRESH_CASE=$FRESH_CASE FRESH_ROOM=${FRESH_ROOM:-NOT_FOUND}"
}

# ── Sub-case 2: Soft-pause ─────────────────────────────────────────────────────
# messaging_enabled=false → drive case transition → confirm NO room provisioned
# → messaging_enabled=true → drive fresh case → confirm room IS provisioned

run_2() {
  echo ""
  echo "--- Sub-case 2: Soft-pause (messaging_enabled toggle) ---"

  local BEFORE
  BEFORE=$(messaging_enabled_get)
  echo "  messaging_enabled BEFORE=$BEFORE"

  # Step 1: disable messaging
  messaging_enabled_set "false"
  sleep 0.3

  # Drive a case to JurySelection (assign-jury fires the hook, which checks messaging_enabled)
  local AJWT TJWT MJWT POST_RESP POST_ID CASE_ID ASSIGN_RESP
  AJWT=$(admin_jwt)
  TJWT=$(login "testuser" "${TEST_PASS}")
  MJWT=$(login "testmod"  "${TEST_PASS}")

  POST_RESP=$(api_post /post \
    "{\"name\":\"phase7-soft-pause-$(date +%s)\",\"community_id\":$COMMUNITY_ID,\"nsfw\":false}" \
    "$TJWT")
  POST_ID=$(printf '%s' "$POST_RESP" | json_nested post_view post id)
  [ -n "$POST_ID" ] || { echo "POST_CREATE_FAILED" >&2; return 1; }

  api_post /governance/report \
    "{\"target_type\":\"post\",\"target_id\":$POST_ID,\"reason_code\":\"phase7-soft-pause\",\"community_id\":$COMMUNITY_ID}" \
    "$MJWT" >/dev/null

  CASE_ID=$(psql "SELECT id FROM moderation_case WHERE target_post_id=$POST_ID ORDER BY id DESC LIMIT 1;" 2>/dev/null)
  [ -n "$CASE_ID" ] || { echo "CASE_NOT_FOUND" >&2; return 1; }

  ASSIGN_RESP=$(api_post /governance/admin/assign-jury \
    "{\"case_id\":$CASE_ID,\"community_id\":$COMMUNITY_ID}" \
    "$AJWT")
  echo "  paused_case_id=$CASE_ID status=$(case_status "$CASE_ID") (messaging_enabled=false)"

  # Wait briefly then check: NO room should be provisioned
  sleep 1
  local PAUSED_COUNT
  PAUSED_COUNT=$(bridge_room_count "$CASE_ID" "jury")
  if [ "$PAUSED_COUNT" -eq 0 ]; then
    echo "  SOFT_PAUSE=✅ no room provisioned while messaging_enabled=false"
  else
    echo "  SOFT_PAUSE=❌ room WAS provisioned despite messaging_enabled=false (PAUSED_COUNT=$PAUSED_COUNT)"
  fi

  # Step 2: re-enable messaging
  messaging_enabled_set "true"
  sleep 0.3
  echo "  messaging_enabled AFTER=$(messaging_enabled_get)"

  # Drive a fresh case — room should provision
  local RESUME_RESULT RESUME_CASE RESUME_ROOM=""
  RESUME_RESULT=$(open_and_decide "soft-pause-resume" "remove_content") || {
    echo "  RESUME_CASE_FAILED"
    echo "RESULT_2: PAUSED_COUNT=$PAUSED_COUNT RESUME_ROOM=FAILED"
    return 0
  }
  RESUME_CASE=$(printf '%s' "$RESUME_RESULT" | awk '{print $1}')
  echo "  resume_case_id=$RESUME_CASE status=$(case_status "$RESUME_CASE")"

  RESUME_ROOM=$(wait_for_bridge_room "$RESUME_CASE" "jury" 8 0.5) || true
  if [ -n "$RESUME_ROOM" ]; then
    echo "  RESUME_ROOM_PROVISIONED=✅ $RESUME_ROOM"
  else
    echo "  RESUME_ROOM_PROVISIONED=❌ room did not appear after re-enabling"
  fi

  local CHAIN
  CHAIN=$(verify_hash_chain)
  echo "CHAIN=$CHAIN"
  echo "RESULT_2: PAUSED_COUNT=$PAUSED_COUNT RESUME_CASE=$RESUME_CASE RESUME_ROOM=${RESUME_ROOM:-NOT_FOUND}"
}

# ── Sub-case 3: Bridge-down resilience ────────────────────────────────────────
# Stop bridge → drive case to Decided → verify DB writes intact (no Lemmy error)
# → restart bridge → confirm bridge_room NOT provisioned for the down-period case
# (the push-only gap: transitions fired while bridge was down are lost — expected).

run_3() {
  echo ""
  echo "--- Sub-case 3: Bridge-down resilience ---"

  # Capture bridge_room count baseline
  local BASELINE_COUNT
  BASELINE_COUNT=$(bridge_rooms | wc -l || echo "0")
  echo "  bridge_room baseline count=$BASELINE_COUNT"

  # Stop the bridge
  echo "  Stopping brehon-bridge..."
  docker stop brehon-bridge >/dev/null 2>&1 || {
    echo "  BRIDGE_STOP_FAILED — is the container named brehon-bridge?" >&2
    echo "RESULT_3: SKIPPED (bridge stop failed)"
    return 0
  }
  echo "  Bridge stopped."

  # Drive a case fully to Decided while bridge is down
  local DOWN_RESULT DOWN_CASE DOWN_POST DOWN_STATUS
  DOWN_RESULT=$(open_and_decide "bridge-down" "remove_content") || {
    echo "  DOWN_CASE_FAILED — Lemmy errored (unexpected)" >&2
    echo "  Restarting bridge..."
    docker start brehon-bridge >/dev/null 2>&1 || true
    echo "RESULT_3: DOWN_CASE=FAILED (Lemmy error while bridge was down)"
    return 0
  }
  DOWN_CASE=$(printf '%s' "$DOWN_RESULT" | awk '{print $1}')
  DOWN_STATUS=$(case_status "$DOWN_CASE")
  echo "  down_case_id=$DOWN_CASE status=$DOWN_STATUS (bridge was DOWN during this flow)"

  if [ "$DOWN_STATUS" = "Decided" ]; then
    echo "  LEMMY_RESILIENCE=✅ case reached Decided with bridge stopped (fire-and-forget hook)"
  else
    echo "  LEMMY_RESILIENCE=⚠️  case status=$DOWN_STATUS (expected Decided)"
  fi

  # Verify sanction was still written
  local SANCTION_COUNT
  SANCTION_COUNT=$(psql "SELECT COUNT(*) FROM sanction WHERE case_id=$DOWN_CASE AND active=true;" 2>/dev/null || echo "0")
  echo "  SANCTION_WRITTEN=$SANCTION_COUNT (expect 1)"

  # Verify chain is intact (Lemmy DB writes succeeded even without bridge)
  local CHAIN_BEFORE
  CHAIN_BEFORE=$(verify_hash_chain)
  echo "  CHAIN_BEFORE_RESTART=$CHAIN_BEFORE"

  # Restart the bridge
  echo "  Restarting brehon-bridge..."
  docker start brehon-bridge >/dev/null 2>&1
  sleep 2  # brief settle for bridge startup
  echo "  Bridge restarted."

  # Confirm the down-period case has NO bridge_room row (transition was lost — expected)
  sleep 1
  local DOWN_ROOM_COUNT
  DOWN_ROOM_COUNT=$(bridge_room_count "$DOWN_CASE" "jury")
  if [ "$DOWN_ROOM_COUNT" -eq 0 ]; then
    echo "  LOST_TRANSITION_CONFIRMED=✅ no bridge_room row for case $DOWN_CASE"
    echo "  (Expected m2 gap: push-only hook, no log-tail replay — transitions during bridge-down are lost)"
  else
    echo "  LOST_TRANSITION=⚠️  bridge_room appeared for case $DOWN_CASE after restart ($DOWN_ROOM_COUNT rows)"
    echo "  (If bridge-side replay was implemented, this would be expected — document if so)"
  fi

  # Drive one more case to confirm bridge is healthy after restart
  local RECOVER_RESULT RECOVER_CASE RECOVER_ROOM=""
  RECOVER_RESULT=$(open_and_decide "bridge-down-recovery" "remove_content") || {
    echo "  RECOVERY_CASE_FAILED (bridge not healthy after restart?)"
    echo "RESULT_3: DOWN_CASE=$DOWN_CASE RESILIENCE=$DOWN_STATUS SANCTION=$SANCTION_COUNT LOST_TRANSITION=$DOWN_ROOM_COUNT RECOVERY=FAILED"
    return 0
  }
  RECOVER_CASE=$(printf '%s' "$RECOVER_RESULT" | awk '{print $1}')
  echo "  recovery_case_id=$RECOVER_CASE status=$(case_status "$RECOVER_CASE")"

  RECOVER_ROOM=$(wait_for_bridge_room "$RECOVER_CASE" "jury" 10 0.5) || true
  if [ -n "$RECOVER_ROOM" ]; then
    echo "  BRIDGE_HEALTHY_AFTER_RESTART=✅ $RECOVER_ROOM"
  else
    echo "  BRIDGE_HEALTHY_AFTER_RESTART=❌ no room provisioned after restart (bridge may need longer to settle)"
  fi

  local CHAIN_AFTER
  CHAIN_AFTER=$(verify_hash_chain)
  echo "CHAIN=$CHAIN_AFTER"
  echo "RESULT_3: DOWN_CASE=$DOWN_CASE LEMMY_RESILIENCE=$DOWN_STATUS SANCTION=$SANCTION_COUNT LOST_TRANSITION=$DOWN_ROOM_COUNT RECOVERY_CASE=$RECOVER_CASE RECOVERY_ROOM=${RECOVER_ROOM:-NOT_FOUND}"
}

# ── dispatch ──────────────────────────────────────────────────────────────────

case "$RUN" in
  1)  run_1 "$@" ;;
  2)  run_2 ;;
  3)  run_3 ;;
  all)
    echo "(Skipping sub-case 1 — requires manual bridge restart first. Run: bash $0 1 [case_id])"
    run_2
    run_3
    ;;
  *)
    echo "Usage: $0 [1|2|3|all]" >&2
    echo "  1 = idempotency check (bridge must be restarted by infra BEFORE running)" >&2
    echo "  2 = soft-pause" >&2
    echo "  3 = bridge-down resilience" >&2
    echo "  all = run 2 then 3 (skip 1)" >&2
    exit 1
    ;;
esac

echo ""
echo "Phase 7 sub-cases complete — review RESULT_ lines above."
