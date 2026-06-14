#!/usr/bin/env bash
# Brehon pilot-seed — shared helpers for governance test-scenario seeding.
#
# Standardized seeding so test scenarios start from a known, CORRECTLY-SCOPED
# state instead of ad-hoc curl/SQL (which bit us twice in phase-3: the
# reputation_snapshot.community_id scope trap, and the wrong-handler-doesn't-
# fire-the-bridge-hook trap). Canonical knowledge:
# .claude/lessons/feedback_pilot_governance_workflow_seeding_order.md (PMD #990).
#
# RUN LOCATION: on the homeserver (these talk to localhost:8536 + the postgres
# container). From the laptop:  ssh homeserver "bash /srv/brehon-fork/scripts/brehon/pilot-seed/<script>.sh ..."
# or copy the dir over. Source this lib from each scenario script.
#
# CONTRACT: every scenario script is idempotent where it can be (re-running
# re-derives or no-ops), prints machine-readable result lines (CASE_ID=..,
# ROOM_ID=.., PANEL=..), and exits non-zero on any failed precondition.
#
# Env overrides (all have pilot defaults):
#   API           Lemmy API base                 (default http://localhost:8536/api/v4)
#   PG_CONTAINER  postgres container name        (default docker-postgres-1)
#   PG_DB / PG_USER                              (default lemmy / lemmy)
#   COMMUNITY_ID  community to scope cases to     (default 2 = test_governance)
#   ADMIN_USER / ADMIN_PASS                      (default lemmy / lemmylemmy)
#   TEST_PASS     password for all juror/test accounts (default testpass123)

set -euo pipefail

API="${API:-http://localhost:8536/api/v4}"
PG_CONTAINER="${PG_CONTAINER:-docker-postgres-1}"
PG_DB="${PG_DB:-lemmy}"
PG_USER="${PG_USER:-lemmy}"
COMMUNITY_ID="${COMMUNITY_ID:-2}"
ADMIN_USER="${ADMIN_USER:-lemmy}"
ADMIN_PASS="${ADMIN_PASS:-lemmylemmy}"
TEST_PASS="${TEST_PASS:-testpass123}"

# --- low-level -------------------------------------------------------------

# psql <sql>  — run read/write SQL, tuples-only, unaligned. Stdout = result.
psql() {
  docker exec "$PG_CONTAINER" psql -U "$PG_USER" "$PG_DB" -t -A -c "$1"
}

# api_post <path> <json> [jwt]  — POST, echo response body.
api_post() {
  local path="$1" body="$2" jwt="${3:-}"
  if [ -n "$jwt" ]; then
    curl -s -X POST "$API$path" -H 'Content-Type: application/json' \
      -H "Authorization: Bearer $jwt" -d "$body"
  else
    curl -s -X POST "$API$path" -H 'Content-Type: application/json' -d "$body"
  fi
}

# login <user> <pass>  — echo the JWT (bare). Fails loud on no token.
login() {
  local u="$1" p="$2" resp jwt
  resp=$(api_post /account/auth/login "{\"username_or_email\":\"$u\",\"password\":\"$p\"}")
  jwt=$(printf '%s' "$resp" | python3 -c 'import sys,json; print(json.load(sys.stdin).get("jwt",""))' 2>/dev/null || true)
  if [ -z "$jwt" ]; then
    echo "LOGIN_FAILED user=$u resp=$resp" >&2
    return 1
  fi
  printf '%s' "$jwt"
}

admin_jwt() { login "$ADMIN_USER" "$ADMIN_PASS"; }

# json_field <key>  — read a top-level field from stdin JSON.
json_field() { python3 -c "import sys,json; print(json.load(sys.stdin).get('$1',''))"; }

# json_nested <key1> <key2> <key3>  — read a three-level nested field from stdin JSON.
# e.g. POST /post response: .post_view.post.id  ->  json_nested post_view post id
json_nested() { python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('$1',{}).get('$2',{}).get('$3',''))"; }

# --- governance helpers ----------------------------------------------------

# person_id <username>  — echo the person.id for a username, or empty.
person_id() { psql "SELECT id FROM person WHERE name='$1' LIMIT 1;"; }

# make_jury_eligible <person_id>  — insert the community-SCOPED reputation_snapshot
# that the eligibility query's `community_id IS NOT DISTINCT FROM case.community_id`
# join actually matches. THIS IS THE SCOPE TRAP — community_id MUST equal the
# case's community (COMMUNITY_ID), never NULL. Idempotent on (person_id,community_id).
make_jury_eligible() {
  local pid="$1"
  psql "INSERT INTO reputation_snapshot
          (person_id, community_id, reporting_accuracy, jury_reliability,
           participation_consistency, endorsement_strength, jury_eligible,
           trusted_reporter, can_sponsor)
        VALUES ($pid, $COMMUNITY_ID, 100, 100, 100, 100, true, false, false)
        ON CONFLICT (person_id, community_id)
        DO UPDATE SET jury_eligible=true,
                      reporting_accuracy=100, jury_reliability=100,
                      participation_consistency=100, endorsement_strength=100;" >/dev/null
}

# register_and_approve <username>  — register (require_application mode) + admin-approve.
# Idempotent: skips if the person already exists. Echoes the person_id.
register_and_approve() {
  local u="$1" pid app_id ajwt
  pid=$(person_id "$u")
  if [ -n "$pid" ]; then printf '%s' "$pid"; return 0; fi
  api_post /user/register \
    "{\"username\":\"$u\",\"password\":\"$TEST_PASS\",\"password_verify\":\"$TEST_PASS\",\"show_nsfw\":false,\"answer\":\"pilot test account\"}" \
    >/dev/null
  ajwt=$(admin_jwt)
  # find the pending application id for this user and approve it
  app_id=$(curl -s "$API/admin/registration_application/list?unread_only=true" \
            -H "Authorization: Bearer $ajwt" \
          | python3 -c "import sys,json
d=json.load(sys.stdin)
for a in d.get('registration_applications',[]):
    if a.get('creator',{}).get('name')=='$u': print(a['registration_application']['id']); break" 2>/dev/null || true)
  if [ -n "$app_id" ]; then
    curl -s -X PUT "$API/admin/registration_application/approve" \
      -H 'Content-Type: application/json' -H "Authorization: Bearer $ajwt" \
      -d "{\"id\":$app_id,\"approve\":true}" >/dev/null
  fi
  pid=$(person_id "$u")
  printf '%s' "$pid"
}

# seed_eligible_jurors <prefix> <count>  — ensure <count> registered + approved +
# community-scoped jury-eligible accounts named <prefix>1..<prefix>N exist.
# Echoes a space-separated list of their person_ids. Use to guarantee a strict
# eligible pool that survives the original-panel exclusion (see lesson §4).
seed_eligible_jurors() {
  local prefix="$1" count="$2" i pid ids=()
  for i in $(seq 1 "$count"); do
    pid=$(register_and_approve "${prefix}${i}")
    [ -n "$pid" ] && make_jury_eligible "$pid" && ids+=("$pid")
  done
  printf '%s' "${ids[*]}"
}

# case_status <case_id>  — echo moderation_case.status.
case_status() { psql "SELECT status FROM moderation_case WHERE id=$1;"; }

# --- config-aware helpers (read the FROZEN snapshots, never hardcode 5/3) ----
#
# The governance system is config-table-driven (panel_size / quorum / threshold
# vary per community x severity_tier x status_tier — 127 keys). But by the time a
# case is in JurySelection, admin_assign_jury has COLLAPSED all of that into three
# integer columns on moderation_case. Read those, and a seeder is config-agnostic
# by construction. THE WIN CONDITION IS threshold_count, NOT quorum: a case decides
# when the first decision (enum order) reaches threshold_count_snapshot votes;
# quorum is only a floor gate ("don't tally until this many are in"); deadlock
# fires when vote_count == panel_size with no decision at threshold. Hardcoding 3
# only worked because the default Minor-severity path makes quorum==threshold==3 at
# panel 5 — Severe cases are 7/5/6 and 3 votes can NEVER decide them.
# Refs: submit_jury_vote.rs:343-466, admin_assign_jury.rs:152-213.

# read_panel_size / read_quorum / read_threshold_count <case_id>
# Echo the frozen snapshot (blank if NULL — case not yet jury-assigned).
read_panel_size()      { psql "SELECT panel_size_snapshot      FROM moderation_case WHERE id=$1;"; }
read_quorum()          { psql "SELECT quorum_snapshot          FROM moderation_case WHERE id=$1;"; }
read_threshold_count() { psql "SELECT threshold_count_snapshot FROM moderation_case WHERE id=$1;"; }

# read_appeal_panel_size / read_appeal_threshold_count <case_id>
# Appeal-round snapshots (populated when the appeal panel is seated).
read_appeal_panel_size()      { psql "SELECT panel_size_snapshot      FROM appeal WHERE case_id=$1 ORDER BY id DESC LIMIT 1;"; }
read_appeal_threshold_count() { psql "SELECT threshold_count_snapshot FROM appeal WHERE case_id=$1 ORDER BY id DESC LIMIT 1;"; }

# seated_panel <case_id> <role>  — echo seated jurors' person_ids for a role
# ('Original'|'Appeal'), ordered. Replaces hardcoded juror-id lists: reads who the
# selector ACTUALLY seated (the relaxation cascade can change identities).
seated_panel() { psql "SELECT person_id FROM jury_assignment WHERE case_id=$1 AND role='$2' ORDER BY id;"; }

# assert_status <case_id> <expected>  — fail LOUD (return 1) when status differs.
# Replaces bare WARN-then-proceed (the false-green class). Under `set -e` a caller
# that does NOT swallow this with `||` will abort, honouring the suite's
# "fails loud on a failed precondition" contract.
assert_status() {
  local got; got=$(case_status "$1")
  [ "$got" = "$2" ] || { echo "STATUS_MISMATCH case=$1 expected=$2 got=$got threshold=$(read_threshold_count "$1") panel=$(read_panel_size "$1")" >&2; return 1; }
}

# vote_to_threshold <case_id> <decision> [role=Original]
# Cast <decision> from the ACTUAL seated panel until vote_count >= the case's
# threshold_count_snapshot. Reads the threshold + the seated jurors at runtime, so
# it adapts to any panel size / severity. Returns 0 iff the case reaches Decided
# (Original) — i.e. the exit reflects REALITY, not a vote tally. Each juror accepts
# (idempotent) then votes once. Stops early once Decided.
vote_to_threshold() {
  local cid="$1" decision="$2" role="${3:-Original}" pid uname jjwt cast=0 need
  need=$(read_threshold_count "$cid")
  [ -n "$need" ] || { echo "VOTE_TO_THRESHOLD_NO_SNAPSHOT case=$cid (not jury-assigned?)" >&2; return 1; }
  local pids; mapfile -t pids < <(seated_panel "$cid" "$role")
  [ "${#pids[@]}" -gt 0 ] || { echo "VOTE_TO_THRESHOLD_EMPTY_PANEL case=$cid role=$role" >&2; return 1; }
  for pid in "${pids[@]}"; do
    [ "$(case_status "$cid")" = "Decided" ] && break
    uname=$(psql "SELECT name FROM person WHERE id=$pid;")
    jjwt=$(login "$uname" "$TEST_PASS") || continue
    api_post /governance/jury/accept "{\"case_id\":$cid}" "$jjwt" >/dev/null 2>&1 || true
    api_post /governance/jury/vote "{\"case_id\":$cid,\"decision\":\"$decision\",\"rationale\":\"pilot-seed vote_to_threshold\"}" "$jjwt" >/dev/null
    cast=$((cast+1))
  done
  echo "VOTES_CAST=$cast NEEDED=$need SEATED=${#pids[@]} role=$role" >&2
  [ "$(case_status "$cid")" = "Decided" ]
}

# bridge_rooms  — echo the bridge_room rows (case_id|room_type|matrix_room_id per line).
# Reads the bridge's SQLite by copying it out (no sqlite3 CLI in container).
bridge_rooms() {
  docker cp brehon-bridge:/data/bridge.db /tmp/pilot-seed-br.db 2>/dev/null
  python3 -c "import sqlite3
for r in sqlite3.connect('/tmp/pilot-seed-br.db').execute('SELECT case_id,room_type,matrix_room_id FROM bridge_room'):
    print('|'.join(str(x) for x in r))" 2>/dev/null || true
}

# verify_hash_chain  — echo CHAIN_INTACT or CHAIN_BROKEN@<id>. Global check.
verify_hash_chain() {
  psql "SELECT CASE WHEN bool_and(ok) THEN 'CHAIN_INTACT'
               ELSE 'CHAIN_BROKEN@'||min(id) FILTER (WHERE NOT ok) END
        FROM (SELECT id, (prev_hash IS NOT DISTINCT FROM
                lag(entry_hash) OVER (ORDER BY id)) AS ok
              FROM governance_log) t
        WHERE id > (SELECT min(id) FROM governance_log);"
}
