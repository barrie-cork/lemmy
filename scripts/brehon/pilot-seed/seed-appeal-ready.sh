#!/usr/bin/env bash
# Brehon pilot-seed scenario: drive a FRESH case through the full happy path to
# `Decided`, leaving the defendant ready to appeal. Optionally fire the appeal
# itself (which provisions the Matrix appeal room).
#
# This is the flow VERIFIED end-to-end on case 6 (2026-06-13, fix c84aaf27c):
#   testuser post -> testmod report -> admin assign-jury -> 5 jurors accept+vote
#   -> quorum -> Decided.  Then (with --appeal): testuser POST /governance/appeal,
#   which auto-seats the appeal panel AND fires the Decided->Appealed bridge hook
#   (the ONLY route that fires it — admin_trigger_appeal_rejury does NOT; lesson §1),
#   provisioning the appeal-case-N room.
#
# PRECONDITION the script enforces: enough strictly-eligible jurors exist for the
# appeal panel to seat AFTER the original panel is excluded. It calls seed-jurors
# to guarantee a fresh pool (juror6.. by default) before driving the case.
#
# Usage:   bash seed-appeal-ready.sh [--appeal]
#   (no flag)  stop at Decided; print CASE_ID for a manual appeal
#   --appeal   also fire the appeal + verify the appeal room provisioned
# Output:  CASE_ID=.. POST_ID=.. STATUS=.. [APPEAL_ID=.. APPEAL_ROOM_ID=.. PANEL=..]

set -euo pipefail
DIR="$(cd "$(dirname "$0")" && pwd)"; . "$DIR/lib.sh"

DO_APPEAL=0; [ "${1:-}" = "--appeal" ] && DO_APPEAL=1

POSTER="${POSTER:-testuser}"     # defendant (post author + appeal requester)
REPORTER="${REPORTER:-testmod}"
APPEAL_POOL_PREFIX="${APPEAL_POOL_PREFIX:-juror}"
# Headroom math: the appeal panel EXCLUDES the original panel and is sized
# max(ceil(orig*1.5), orig+2) clamped [3,11] (admin_assign_jury.rs:1065). At the
# largest legal original panel (9, founder+severe) the appeal panel can need 11,
# AND those 11 must be disjoint from the original 9 -> a worst-case pool of 20.
# A flat 10 under-seats the appeal even at DEFAULT config (orig 5 -> appeal
# max(8,7)=8 needed, pool 10 - 5 excluded = 5 free < 8). Default to 20.
APPEAL_POOL_COUNT="${APPEAL_POOL_COUNT:-20}"

echo "# ensuring eligible juror pool (appeal panel excludes originals -- need disjoint headroom; default 20)"
seed_eligible_jurors "$APPEAL_POOL_PREFIX" "$APPEAL_POOL_COUNT" >/dev/null

AJWT=$(admin_jwt)
PJWT=$(login "$POSTER" "$TEST_PASS")
RJWT=$(login "$REPORTER" "$TEST_PASS")

echo "# 1. fresh post by $POSTER in community $COMMUNITY_ID"
POST_ID=$(api_post /post \
  "{\"name\":\"pilot appeal-ready $(psql "SELECT extract(epoch from now())::int;")\",\"community_id\":$COMMUNITY_ID,\"body\":\"seed-appeal-ready\"}" \
  "$PJWT" | json_nested post_view post id)
[ -n "$POST_ID" ] || { echo "POST_FAILED" >&2; exit 1; }
echo "POST_ID=$POST_ID"

echo "# 2. report by $REPORTER -> opens case"
api_post /governance/report \
  "{\"target_type\":\"post\",\"target_id\":$POST_ID,\"reason_code\":\"seed appeal-ready\"}" \
  "$RJWT" >/dev/null
# newest case on this post
CASE_ID=$(psql "SELECT id FROM moderation_case WHERE target_post_id=$POST_ID ORDER BY id DESC LIMIT 1;")
[ -n "$CASE_ID" ] || { echo "CASE_OPEN_FAILED" >&2; exit 1; }
echo "CASE_ID=$CASE_ID"

echo "# 3. admin assign-jury -> JurySelection"
api_post /governance/admin/assign-jury "{\"case_id\":$CASE_ID}" "$AJWT" >/dev/null

echo "# 4. seated jurors accept + vote remove_content until threshold_count -> Decided"
# Config-aware: vote_to_threshold reads threshold_count_snapshot + the actual
# seated Original panel, so it adapts to any severity/panel size (Severe = 7/5/6,
# not 5/3/3). Hardcoding 3 votes would never decide a Severe case.
echo "# panel=$(read_panel_size "$CASE_ID") quorum=$(read_quorum "$CASE_ID") threshold=$(read_threshold_count "$CASE_ID")"
vote_to_threshold "$CASE_ID" "remove_content" "Original" || true

STATUS=$(case_status "$CASE_ID")
echo "STATUS=$STATUS"
# Fail LOUD if not Decided — proceeding to appeal against a non-Decided case is
# the false-green that violates the suite's "fail loud on precondition" contract.
if [ "$STATUS" != "Decided" ]; then
  echo "RESULT=CASE_NOT_DECIDED (need $(read_threshold_count "$CASE_ID") concurring votes on a panel of $(read_panel_size "$CASE_ID"); check eligible-pool size + severity_tier)" >&2
  exit 1
fi

if [ "$DO_APPEAL" -eq 1 ]; then
  echo "# 5. fire appeal as defendant ($POSTER) -- auto-seats appeal panel + fires Appealed hook"
  AR=$(api_post /governance/appeal \
        "{\"case_id\":$CASE_ID,\"reason\":\"seed-appeal-ready verification\"}" "$PJWT")
  APPEAL_ID=$(printf '%s' "$AR" | json_field appeal_id)
  echo "APPEAL_ID=$APPEAL_ID"
  echo "STATUS=$(case_status "$CASE_ID")"
  PANEL=$(psql "SELECT string_agg(person_id::text,',') FROM jury_assignment WHERE case_id=$CASE_ID AND role='Appeal';")
  echo "PANEL=$PANEL"
  echo "APPEAL_PANEL_SIZE=$(read_appeal_panel_size "$CASE_ID") APPEAL_THRESHOLD=$(read_appeal_threshold_count "$CASE_ID")"
  # bounded retry for the fire-and-forget bridge POST (replaces a flat sleep 3 that
  # flaked under load) — read the appeal room up to ~6s, break as soon as it lands.
  APPEAL_ROOM_ID=""
  for _ in $(seq 1 12); do
    sleep 0.5
    APPEAL_ROOM_ID=$(bridge_rooms | awk -F'|' -v c="$CASE_ID" '$1==c && $2=="appeal" {print $3}')
    [ -n "$APPEAL_ROOM_ID" ] && break
  done
  echo "APPEAL_ROOM_ID=${APPEAL_ROOM_ID:-NONE}"
  echo "HASH_CHAIN=$(verify_hash_chain)"
  if [ -n "$APPEAL_ROOM_ID" ]; then
    echo "RESULT=APPEAL_ROOM_PROVISIONED"
  else
    echo "RESULT=NO_APPEAL_ROOM (check: empty panel? bridge log empty-skip WARN? lesson sec 2/4)"
  fi
fi
