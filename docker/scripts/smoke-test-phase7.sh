#!/usr/bin/env bash
# smoke-test-phase7.sh
# Phase 7 admin endpoint probe — all 8 endpoints in one coordinated pass.
# Spaces calls with 3s sleeps to stay within the 180/60s rate limit.
# Exits non-zero if any endpoint returns an error (except expected 429 which retries once).
#
# Usage:
#   ADMIN_JWT=<jwt> bash docker/scripts/smoke-test-phase7.sh
#   or: bash docker/scripts/smoke-test-phase7.sh <admin_jwt>
#
# Requires: curl, python3

set -euo pipefail

API="${API_BASE:-http://localhost:8536/api/v4}"
JWT="${1:-${ADMIN_JWT:-}}"

if [ -z "$JWT" ]; then
  echo "ERROR: Pass admin JWT as first arg or set ADMIN_JWT env var" >&2
  exit 1
fi

PASS=0
FAIL=0
RESULTS=()

probe() {
  local label="$1"
  local method="$2"
  local path="$3"
  local body="${4:-}"
  local sleep_after="${5:-3}"

  local curl_args=(-s -w "\n%{http_code}" -H "Authorization: Bearer $JWT")
  if [ "$method" = "POST" ]; then
    curl_args+=(-X POST -H "Content-Type: application/json" -d "$body")
  fi

  local response
  response=$(curl "${curl_args[@]}" "$API$path")
  local http_code
  http_code=$(echo "$response" | tail -1)
  local body_out
  body_out=$(echo "$response" | head -n -1)

  # Retry once on 429
  if [ "$http_code" = "429" ]; then
    echo "  [429] $label — rate limited, waiting 65s..."
    sleep 65
    response=$(curl "${curl_args[@]}" "$API$path")
    http_code=$(echo "$response" | tail -1)
    body_out=$(echo "$response" | head -n -1)
  fi

  local error
  error=$(echo "$body_out" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('error',''))" 2>/dev/null || echo "")

  if [ "$http_code" = "200" ] && [ -z "$error" ]; then
    echo "  [OK]  $label (HTTP $http_code)"
    PASS=$((PASS + 1))
    RESULTS+=("OK  $label")
  else
    echo "  [FAIL] $label (HTTP $http_code, error=${error:-none})"
    echo "         body: $(echo "$body_out" | head -c 200)"
    FAIL=$((FAIL + 1))
    RESULTS+=("FAIL $label — HTTP $http_code error=${error:-none}")
  fi

  sleep "$sleep_after"
}

echo "=== Phase 7 admin endpoint smoke test ==="
echo "API: $API"
echo ""

probe "7.1 GET reputation-stats"         GET "/governance/admin/reputation-stats"
probe "7.2 GET dashboard"                GET "/governance/admin/dashboard"
probe "7.3 GET config"                   GET "/governance/admin/config"
probe "7.4 GET config/audit"             GET "/governance/admin/config/audit"
probe "7.5 GET rule-sets"                GET "/governance/admin/rule-sets?community_id=2"
probe "7.6 GET reputation/rollup"        GET "/governance/admin/reputation/rollup?person_id=2"
probe "7.7 POST config dry_run"          POST "/governance/admin/config" \
  '{"key":"jury.quorum","value_type":"int","value":1,"scope":"instance","reason":"smoke test dry run","dry_run":true}'
probe "7.8 POST config apply (idempotent)" POST "/governance/admin/config" \
  '{"key":"jury.quorum","value_type":"int","value":1,"scope":"instance","reason":"smoke test apply"}' 3

echo ""
echo "=== Results ==="
for r in "${RESULTS[@]}"; do echo "  $r"; done
echo ""
echo "PASS: $PASS / $((PASS + FAIL))"
[ "$FAIL" -eq 0 ] && echo "Phase 7: PASS" || { echo "Phase 7: FAIL ($FAIL failures)"; exit 1; }
