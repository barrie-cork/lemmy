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
# original-panel jurors: the existing pool. Fresh eligibles for the appeal:
APPEAL_POOL_PREFIX="${APPEAL_POOL_PREFIX:-juror}"   # juror1..juror10 exist
APPEAL_POOL_COUNT="${APPEAL_POOL_COUNT:-10}"

echo "# ensuring eligible juror pool (appeal panel excludes originals -- need headroom)"
seed_eligible_jurors "$APPEAL_POOL_PREFIX" "$APPEAL_POOL_COUNT" >/dev/null

AJWT=$(admin_jwt)
PJWT=$(login "$POSTER" "$TEST_PASS")
RJWT=$(login "$REPORTER" "$TEST_PASS")

echo "# 1. fresh post by $POSTER in community $COMMUNITY_ID"
POST_ID=$(api_post /post \
  "{\"name\":\"pilot appeal-ready $(psql "SELECT extract(epoch from now())::int;")\",\"community_id\":$COMMUNITY_ID,\"body\":\"seed-appeal-ready\"}" \
  "$PJWT" | json_field id)
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

echo "# 4. seated jurors accept + 3 vote remove_content -> quorum -> Decided"
# read the seated ORIGINAL panel's person_ids, map to usernames, accept+vote.
mapfile -t PANEL_PIDS < <(psql "SELECT person_id FROM jury_assignment WHERE case_id=$CASE_ID AND role='Original' ORDER BY id;")
voted=0
for pid in "${PANEL_PIDS[@]}"; do
  uname=$(psql "SELECT name FROM person WHERE id=$pid;")
  jjwt=$(login "$uname" "$TEST_PASS") || continue
  api_post /governance/jury/accept "{\"case_id\":$CASE_ID}" "$jjwt" >/dev/null
  if [ "$voted" -lt 3 ]; then
    api_post /governance/jury/vote "{\"case_id\":$CASE_ID,\"decision\":\"remove_content\"}" "$jjwt" >/dev/null
    voted=$((voted+1))
  fi
done

STATUS=$(case_status "$CASE_ID")
echo "STATUS=$STATUS"
[ "$STATUS" = "Decided" ] || { echo "WARN: expected Decided, got $STATUS (panel/quorum issue)" >&2; }

if [ "$DO_APPEAL" -eq 1 ]; then
  echo "# 5. fire appeal as defendant ($POSTER) -- auto-seats appeal panel + fires Appealed hook"
  AR=$(api_post /governance/appeal \
        "{\"case_id\":$CASE_ID,\"reason\":\"seed-appeal-ready verification\"}" "$PJWT")
  APPEAL_ID=$(printf '%s' "$AR" | json_field appeal_id)
  echo "APPEAL_ID=$APPEAL_ID"
  echo "STATUS=$(case_status "$CASE_ID")"
  PANEL=$(psql "SELECT string_agg(person_id::text,',') FROM jury_assignment WHERE case_id=$CASE_ID AND role='Appeal';")
  echo "PANEL=$PANEL"
  # give the fire-and-forget bridge POST a beat, then read the appeal room
  sleep 3
  APPEAL_ROOM_ID=$(bridge_rooms | awk -F'|' -v c="$CASE_ID" '$1==c && $2=="appeal" {print $3}')
  echo "APPEAL_ROOM_ID=${APPEAL_ROOM_ID:-NONE}"
  echo "HASH_CHAIN=$(verify_hash_chain)"
  if [ -n "$APPEAL_ROOM_ID" ]; then
    echo "RESULT=APPEAL_ROOM_PROVISIONED"
  else
    echo "RESULT=NO_APPEAL_ROOM (check: empty panel? bridge log empty-skip WARN? lesson sec 2/4)"
  fi
fi
