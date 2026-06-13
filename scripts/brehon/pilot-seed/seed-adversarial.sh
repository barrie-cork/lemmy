#!/usr/bin/env bash
# Brehon pilot-seed scenario: ADVERSARIAL / NEGATIVE PATHS (phase 6).
#
# Sub-cases (each uses a fresh case):
#   A — Jury deadlock   → AdminReview + governance_log "jury_deadlock"
#   B — Declined juror  → jury_declined + jury_replacement_selected + replacement seated
#   C — Bad-faith flag  → evidence_quality_recorded (on an EmergencyRemove case)
#   D — Sponsor liability: Pending → Fired (needs active surety + cron window)
#
# Sub-case D is noted below: SponsorLiabilityPending triggers only when the
# sanctioned person has active sureties (endorsements with revoked_at=NULL).
# We seed a surety row directly via SQL so we don't need an endorsement UI.
#
# CRITICAL VERIFY per sub-case:
#   A: case.status=AdminReview, governance_log entry_kind='jury_deadlock', NO sanction_event
#   B: jury_assignment row status=Declined, replacement row exists, governance_log
#      'jury_declined'+'jury_replacement_selected'
#   C: governance_log 'evidence_quality_recorded', reputation_event delta written
#   D: case.status=SponsorLiabilityPending, then (after cron) SponsorLiabilityFired
#      governance_log chain intact throughout
#
# Usage: bash seed-adversarial.sh [A|B|C|D|all]   (default: all)

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

RUN="${1:-all}"

echo "=== Phase 6: Adversarial / negative paths ==="

# ── shared helpers ────────────────────────────────────────────────────────────

# open_case <post_suffix>  → prints CASE_ID
# Creates post → report → returns case_id
open_case() {
  local suffix="$1"
  local AJWT TJWT MJWT POST_RESP POST_ID REP_RESP CASE_ID
  AJWT=$(admin_jwt)
  TJWT=$(login "testuser" "${TEST_PASS}")
  MJWT=$(login "testmod"  "${TEST_PASS}")

  POST_RESP=$(api_post /post \
    "{\"name\":\"phase6-${suffix}-$(date +%s)\",\"community_id\":$COMMUNITY_ID,\"nsfw\":false}" \
    "$TJWT")
  POST_ID=$(printf '%s' "$POST_RESP" | json_nested post_view post id)
  [ -n "$POST_ID" ] || { echo "POST_CREATE_FAILED resp=$POST_RESP" >&2; return 1; }

  api_post /governance/report \
    "{\"target_type\":\"post\",\"target_id\":$POST_ID,\"reason_code\":\"phase6-${suffix}\",\"community_id\":$COMMUNITY_ID}" \
    "$MJWT" >/dev/null

  CASE_ID=$(psql "SELECT id FROM moderation_case WHERE target_post_id=$POST_ID ORDER BY id DESC LIMIT 1;")
  [ -n "$CASE_ID" ] || { echo "CASE_OPEN_FAILED" >&2; return 1; }

  # Assign jury
  api_post /governance/admin/assign-jury \
    "{\"case_id\":$CASE_ID,\"community_id\":$COMMUNITY_ID}" "$AJWT" >/dev/null

  printf '%s %s' "$CASE_ID" "$POST_ID"
}

# panel_names <case_id>  → newline-separated juror usernames on this case
panel_names() {
  psql "SELECT p.name FROM jury_assignment ja
        JOIN person p ON p.id=ja.person_id
        WHERE ja.case_id=$1 AND ja.role='Original' AND ja.status='Selected'
        ORDER BY ja.id;"
}

# ── Sub-case A: Jury deadlock → AdminReview ───────────────────────────────────

run_A() {
  echo ""
  echo "--- Sub-case A: Jury deadlock → AdminReview ---"

  read -r CASE_ID POST_ID <<< "$(open_case deadlock)"
  echo "  case_id=$CASE_ID post_id=$POST_ID → $(case_status $CASE_ID)"

  # Need panel members to vote with SPLIT decisions so no threshold is reached.
  # 5-juror panel, quorum=3. Strategy: 2 vote remove_content, 2 vote no_action,
  # 1 vote advisory_label → no decision gets 3 votes → deadlock on 5th vote.
  mapfile -t JURORS < <(panel_names "$CASE_ID")
  echo "  panel: ${JURORS[*]}"

  local decisions=("remove_content" "remove_content" "no_action" "no_action" "advisory_label")
  local i=0
  for j_name in "${JURORS[@]}"; do
    j_jwt=$(login "$j_name" "${TEST_PASS}" 2>/dev/null || true)
    [ -z "$j_jwt" ] && continue
    api_post /governance/jury/accept "{\"case_id\":$CASE_ID}" "$j_jwt" >/dev/null 2>&1 || true
    local d="${decisions[$i]:-no_action}"
    local resp
    resp=$(api_post /governance/jury/vote \
      "{\"case_id\":$CASE_ID,\"decision\":\"$d\",\"rationale\":\"phase6 deadlock test\"}" \
      "$j_jwt" 2>/dev/null || true)
    local decided
    decided=$(printf '%s' "$resp" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("case_decided",""))' 2>/dev/null || echo "")
    echo "  vote $((i+1))/5 $j_name($d): case_decided=$decided status=$(case_status $CASE_ID)"
    i=$((i+1))
  done

  local STATUS
  STATUS=$(case_status "$CASE_ID")
  echo "  FINAL_STATUS=$STATUS"
  [ "$STATUS" = "AdminReview" ] || echo "  ⚠️  EXPECTED AdminReview, got $STATUS"

  # Check governance log
  local DL_LOG
  DL_LOG=$(psql "SELECT COUNT(*) FROM governance_log WHERE payload->>'case_id'='$CASE_ID' AND entry_kind='jury_deadlock';")
  echo "  JURY_DEADLOCK_LOG_COUNT=$DL_LOG"

  # Confirm NO sanction_event
  local SANCTION_COUNT
  SANCTION_COUNT=$(psql "SELECT COUNT(*) FROM sanction_event se JOIN sanction s ON s.id=se.sanction_id WHERE s.case_id=$CASE_ID;")
  echo "  SANCTION_EVENT_COUNT=$SANCTION_COUNT (expect 0)"

  echo "RESULT_A: STATUS=$STATUS DEADLOCK_LOG=$DL_LOG SANCTION_COUNT=$SANCTION_COUNT"
}

# ── Sub-case B: Declined juror + replacement ──────────────────────────────────

run_B() {
  echo ""
  echo "--- Sub-case B: Declined juror + replacement ---"

  read -r CASE_ID POST_ID <<< "$(open_case decline)"
  echo "  case_id=$CASE_ID → $(case_status $CASE_ID)"

  mapfile -t JURORS < <(panel_names "$CASE_ID")
  echo "  panel: ${JURORS[*]}"

  # Have the FIRST juror decline
  local J0="${JURORS[0]}"
  local J0JWT
  J0JWT=$(login "$J0" "${TEST_PASS}" 2>/dev/null || true)
  local DEC_RESP
  DEC_RESP=$(api_post /governance/jury/decline \
    "{\"case_id\":$CASE_ID,\"reason\":\"phase6 decline test\"}" \
    "$J0JWT")
  echo "  decline resp: $(printf '%s' "$DEC_RESP" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d)' 2>/dev/null || echo "$DEC_RESP")"

  # Check jury_assignment rows
  echo "  jury_assignment rows after decline:"
  psql "SELECT p.name, ja.status FROM jury_assignment ja
        JOIN person p ON p.id=ja.person_id
        WHERE ja.case_id=$CASE_ID AND ja.role='Original'
        ORDER BY ja.id;" | while read -r row; do echo "    $row"; done

  # Check governance log
  local DEC_LOG REP_LOG
  DEC_LOG=$(psql "SELECT COUNT(*) FROM governance_log WHERE payload->>'case_id'='$CASE_ID' AND entry_kind='jury_declined';")
  REP_LOG=$(psql "SELECT COUNT(*) FROM governance_log WHERE payload->>'case_id'='$CASE_ID' AND entry_kind='jury_replacement_selected';")
  echo "  JURY_DECLINED_LOG=$DEC_LOG"
  echo "  JURY_REPLACEMENT_SELECTED_LOG=$REP_LOG"

  echo "RESULT_B: DECLINED_LOG=$DEC_LOG REPLACEMENT_LOG=$REP_LOG"
}

# ── Sub-case C: Bad-faith report flag ─────────────────────────────────────────

run_C() {
  echo ""
  echo "--- Sub-case C: Bad-faith report flag (on existing EmergencyRemove case) ---"

  # Use any existing EmergencyRemove case (cases 9 or 10)
  local ER_CASE
  ER_CASE=$(psql "SELECT id FROM moderation_case WHERE status='EmergencyRemove' LIMIT 1;")
  if [ -z "$ER_CASE" ]; then
    echo "  No EmergencyRemove case found — creating one first"
    TJWT=$(login "testuser" "${TEST_PASS}")
    POST_RESP=$(api_post /post \
      "{\"name\":\"phase6-badfaith-$(date +%s)\",\"community_id\":$COMMUNITY_ID,\"nsfw\":false}" \
      "$TJWT")
    POST_ID=$(printf '%s' "$POST_RESP" | json_nested post_view post id)
    AJWT=$(admin_jwt)
    ER_RESP=$(api_post /governance/admin/emergency-remove \
      "{\"post_id\":$POST_ID,\"reason\":\"phase6 bad-faith test setup\"}" "$AJWT")
    ER_CASE=$(printf '%s' "$ER_RESP" | json_field case_id)
    echo "  Created ER case_id=$ER_CASE"
  else
    echo "  Using existing EmergencyRemove case_id=$ER_CASE"
  fi

  local AJWT
  AJWT=$(admin_jwt)

  # Check pre-condition: does the case have a reporter (creator_id)?
  local REPORTER_ID
  REPORTER_ID=$(psql "SELECT creator_id FROM moderation_case WHERE id=$ER_CASE;")
  echo "  reporter_id=$REPORTER_ID"

  # Check for de-dupe: if already flagged, skip
  local ALREADY
  ALREADY=$(psql "SELECT COUNT(*) FROM reputation_event WHERE dedupe_key='evidence_bad_faith:${ER_CASE}';")
  if [ "$ALREADY" -gt 0 ]; then
    echo "  Already flagged (dedupe_key exists, count=$ALREADY) — using different ER case"
    # Try the other one
    ER_CASE=$(psql "SELECT id FROM moderation_case WHERE status='EmergencyRemove' AND id<>$ER_CASE LIMIT 1;")
    echo "  Switching to case_id=$ER_CASE"
    REPORTER_ID=$(psql "SELECT creator_id FROM moderation_case WHERE id=$ER_CASE;")
    ALREADY=$(psql "SELECT COUNT(*) FROM reputation_event WHERE dedupe_key='evidence_bad_faith:${ER_CASE}';")
  fi

  if [ -z "$REPORTER_ID" ]; then
    echo "  SKIP: case has no reporter (creator_id=NULL) — emergency-remove cases typically have no reporter"
    echo "RESULT_C: SKIPPED (no reporter on ER case)"
    return 0
  fi

  if [ "$ALREADY" -gt 0 ]; then
    echo "  SKIP: already flagged on both ER cases — dedupe prevents re-flagging"
    echo "RESULT_C: SKIPPED (already flagged)"
    return 0
  fi

  # Flag it
  local FLAG_RESP
  FLAG_RESP=$(api_post /governance/admin/emergency-remove/flag-bad-faith \
    "{\"case_id\":$ER_CASE}" "$AJWT")
  echo "  flag resp: $(printf '%s' "$FLAG_RESP" | python3 -c 'import sys,json; d=json.load(sys.stdin); print(d)' 2>/dev/null || echo "$FLAG_RESP")"

  # Verify governance_log
  local EQR_LOG
  EQR_LOG=$(psql "SELECT COUNT(*) FROM governance_log WHERE payload->>'case_id'='$ER_CASE' AND entry_kind='evidence_quality_recorded';")
  echo "  EVIDENCE_QUALITY_RECORDED_LOG=$EQR_LOG"

  # Verify reputation_event
  local REP_EVENT
  REP_EVENT=$(psql "SELECT delta,dimension FROM reputation_event WHERE dedupe_key='evidence_bad_faith:${ER_CASE}';")
  echo "  REPUTATION_EVENT=$REP_EVENT"

  echo "RESULT_C: EQR_LOG=$EQR_LOG REP_EVENT=$REP_EVENT"
}

# ── Sub-case D: Sponsor liability ─────────────────────────────────────────────
#
# Requires: a sanctioned person with an active surety (endorsement with revoked_at=NULL).
# Endorsements are seeded via SQL (no endorsement-creation UI in pilot stack).
# surety table columns: id, sponsor_id, sponsored_id, community_id, revoked_at, created_at
# (sponsor_liability is computed from surety rows, not endorsement rows directly)

run_D() {
  echo ""
  echo "--- Sub-case D: Sponsor liability Pending → (Fired after cron) ---"

  # Check surety table exists
  local ST
  ST=$(psql "SELECT to_regclass('public.surety');" 2>/dev/null || echo "")
  if [ -z "$ST" ] || [ "$ST" = "" ]; then
    echo "  SKIP: surety table does not exist (may not be migrated yet)"
    echo "RESULT_D: SKIPPED (surety table absent)"
    return 0
  fi

  # compute_sponsor_liability queries surety.sponsored_id = target_person_id AND revoked_at IS NULL
  # We seed a surety row: testmod (sponsor_id) sponsors testuser (sponsored_id)
  local SPONSOR_ID SPONSORED_ID
  SPONSORED_ID=$(psql "SELECT id FROM person WHERE name='testuser' LIMIT 1;")
  SPONSOR_ID=$(psql "SELECT id FROM person WHERE name='testmod' LIMIT 1;")

  # Idempotent: only insert if no active surety exists
  local EXISTING_SURETY
  EXISTING_SURETY=$(psql "SELECT id FROM surety WHERE sponsor_id=$SPONSOR_ID AND sponsored_id=$SPONSORED_ID AND revoked_at IS NULL AND (community_id=$COMMUNITY_ID OR community_id IS NULL) LIMIT 1;" 2>/dev/null || echo "")
  if [ -z "$EXISTING_SURETY" ]; then
    psql "INSERT INTO surety (sponsor_id, sponsored_id, community_id)
          VALUES ($SPONSOR_ID, $SPONSORED_ID, $COMMUNITY_ID);" >/dev/null 2>&1 || {
      echo "  SKIP: could not insert surety (schema may differ)"
      echo "RESULT_D: SKIPPED (surety insert failed)"
      return 0
    }
    echo "  Seeded surety: testmod($SPONSOR_ID) sponsors testuser($SPONSORED_ID)"
  else
    echo "  Existing surety id=$EXISTING_SURETY"
  fi

  # Open a case targeting testuser (person 5) + drive to sanction (any decision ≠ no_action)
  read -r CASE_ID POST_ID <<< "$(open_case sponsor)"
  echo "  case_id=$CASE_ID post_id=$POST_ID"

  mapfile -t JURORS < <(panel_names "$CASE_ID")
  local i=0
  for j_name in "${JURORS[@]}"; do
    j_jwt=$(login "$j_name" "${TEST_PASS}" 2>/dev/null || true)
    [ -z "$j_jwt" ] && continue
    api_post /governance/jury/accept "{\"case_id\":$CASE_ID}" "$j_jwt" >/dev/null 2>&1 || true
    local resp
    resp=$(api_post /governance/jury/vote \
      "{\"case_id\":$CASE_ID,\"decision\":\"advisory_label\",\"rationale\":\"phase6 sponsor test\"}" \
      "$j_jwt" 2>/dev/null || true)
    local decided
    decided=$(printf '%s' "$resp" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("case_decided",""))' 2>/dev/null || echo "")
    i=$((i+1))
    echo "  vote $i: case_decided=$decided status=$(case_status $CASE_ID)"
    [ "$decided" = "True" ] || [ "$decided" = "true" ] && break
  done

  local STATUS
  STATUS=$(case_status "$CASE_ID")
  echo "  STATUS_AFTER_QUORUM=$STATUS"

  # If SponsorLiabilityPending, cron (job.grace_check_interval_minutes=5) fires AFTER
  # grace_expires_at (Medium case = 72h, Minor = 24h, Severe = 168h). Print wait advisory.
  if [ "$STATUS" = "SponsorLiabilityPending" ]; then
    local SLP_LOG GRACE_EXP
    SLP_LOG=$(psql "SELECT COUNT(*) FROM governance_log WHERE payload->>'case_id'='$CASE_ID' AND entry_kind='sponsor_liability_pending';")
    GRACE_EXP=$(psql "SELECT grace_expires_at FROM moderation_case WHERE id=$CASE_ID;")
    echo "  SPONSOR_LIABILITY_PENDING_LOG=$SLP_LOG ✅"
    echo "  grace_expires_at=$GRACE_EXP (cron fires within 5 min after that)"
    echo ""
    echo "  ⏳ SponsorLiabilityPending confirmed. Check after grace_expires_at passes:"
    echo "     bash $0 --check-sponsor-fired $CASE_ID"
    echo "     or:  docker exec docker-postgres-1 psql -U lemmy lemmy -t -A -c \"SELECT status FROM moderation_case WHERE id=$CASE_ID;\""
  else
    echo "  ⚠️  Expected SponsorLiabilityPending, got $STATUS"
    echo "     (Possible: testuser not in endorsement scope, or decision maps to None)"
  fi

  echo "RESULT_D: STATUS=$STATUS"
}

# ── Sponsor-fired check (invoked with --check-sponsor-fired <case_id>) ────────

if [ "${RUN}" = "--check-sponsor-fired" ] && [ -n "${2:-}" ]; then
  CASE_ID="$2"
  STATUS=$(case_status "$CASE_ID")
  echo "case_id=$CASE_ID status=$STATUS"
  FIRED_LOG=$(psql "SELECT COUNT(*) FROM governance_log WHERE payload->>'case_id'='$CASE_ID' AND entry_kind='sponsor_liability_fired';")
  echo "SPONSOR_LIABILITY_FIRED_LOG=$FIRED_LOG"
  CHAIN=$(verify_hash_chain)
  echo "CHAIN=$CHAIN"
  exit 0
fi

# ── Run selected sub-cases ────────────────────────────────────────────────────

case "$RUN" in
  A) run_A ;;
  B) run_B ;;
  C) run_C ;;
  D) run_D ;;
  all)
    run_A
    run_B
    run_C
    run_D
    ;;
  *) echo "Usage: $0 [A|B|C|D|all|--check-sponsor-fired <case_id>]" >&2; exit 1 ;;
esac

# ── Hash chain ────────────────────────────────────────────────────────────────
echo ""
CHAIN=$(verify_hash_chain)
echo "CHAIN=$CHAIN"
[ "$CHAIN" = "CHAIN_INTACT" ] || { echo "CHAIN_BROKEN" >&2; exit 1; }
echo ""
echo "Phase 6 sub-cases complete — review RESULT_ lines above."
