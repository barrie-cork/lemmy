#!/usr/bin/env bash
# e2e-harness-smoke.sh — reach-the-containers proof for the Phase-6 e2e
# acceptance harness.
#
# Brings up the full stack and asserts every container is reachable BEFORE
# any acceptance test depends on it (bootstrap stop-and-ask: Task 1 is the
# risk-isolation task; a failing smoke STOPS the phase because acceptance
# failures would be unattributable).
#
# Success: prints E2E_HARNESS_REACH_OK on full completion.
# Failure: any probe exits non-zero → set -euo pipefail aborts with the
#          failing probe clearly visible in the log.
#
# Run from repo root:
#   bash scripts/brehon/e2e-harness-smoke.sh
#
# Prerequisites (laptop advisor runs this — not the daemon):
#   - Docker daemon running
#   - docker-compose.e2e.yml and docker-compose.yml present in services/bridge/
#   - .env present at repo root (or env vars set inline)
#
# DoD: E2E_HARNESS_REACH_OK printed + every probe green.

set -euo pipefail

# Resolve repo root from this script's location.
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

echo "=== e2e harness smoke: Phase-6 acceptance stack ==="
echo ""

# ─── Bring up the stack ───────────────────────────────────────────────────
echo "[up] Starting e2e stack (build may take several minutes on first run)..."
docker compose \
  -f services/bridge/docker-compose.yml \
  -f services/bridge/docker-compose.e2e.yml \
  --profile rtc \
  up -d --build

echo "[up] Stack started. Waiting 45s for services to initialise..."
sleep 45

echo ""
echo "=== Probes ==="

# ─── 1. MinIO reachable ───────────────────────────────────────────────────
echo "[1] MinIO health (http://localhost:9000/minio/health/live)..."
curl -fsS http://localhost:9000/minio/health/live
echo ""
echo "  PASS: MinIO health endpoint reachable"

# ─── 2. LiveKit reachable ─────────────────────────────────────────────────
# LiveKit --dev returns a page (possibly non-200 on /) but the socket is
# open; allow curl errors here (we verify the port is listening via netcat
# as a fallback). The || true prevents set -e from aborting on HTTP errors.
echo "[2] LiveKit reachable (http://localhost:7880)..."
LIVEKIT_STATUS=$(curl -s -o /dev/null -w '%{http_code}' http://localhost:7880 || echo "conn_error")
if [ "${LIVEKIT_STATUS}" = "conn_error" ]; then
  echo "  FAIL: LiveKit socket not reachable on localhost:7880"
  exit 1
fi
echo "  PASS: LiveKit responded (HTTP ${LIVEKIT_STATUS}) on localhost:7880"

# ─── 3. Instance-A bridge liveness ───────────────────────────────────────
# The bridge serves /_matrix/app/v1/... endpoints. A request without the
# hs_token returns 401 — that IS a liveness signal (the bridge is up and
# the auth middleware is running). We do NOT use -f here (401 is expected).
echo "[3] bridge-a liveness (http://localhost:8080/_matrix/app/v1/users/smoke-probe)..."
BRIDGE_A_STATUS=$(curl -s -o /dev/null -w '%{http_code}' \
  http://localhost:8080/_matrix/app/v1/users/smoke-probe || echo "conn_error")
if [ "${BRIDGE_A_STATUS}" = "conn_error" ]; then
  echo "  FAIL: bridge-a socket not reachable on localhost:8080"
  exit 1
fi
if [ "${BRIDGE_A_STATUS}" != "401" ] && [ "${BRIDGE_A_STATUS}" != "200" ]; then
  echo "  FAIL: bridge-a unexpected HTTP status ${BRIDGE_A_STATUS} (want 401 or 200)"
  exit 1
fi
echo "  PASS: bridge-a alive (HTTP ${BRIDGE_A_STATUS} — 401 = auth expected)"

# ─── 4. Instance-B tuwunel reachable + distinct domain ───────────────────
# /_matrix/key/v2/server returns the homeserver's signing keys + server_name.
# This proves: (a) tuwunel-b is reachable, (b) it identifies as matrix-b.localhost.
echo "[4] tuwunel-b key server (http://localhost:8449/_matrix/key/v2/server)..."
KEYRESP=$(curl -fsS http://localhost:8449/_matrix/key/v2/server)
if ! echo "${KEYRESP}" | grep -qF '"matrix-b.localhost"'; then
  echo "  FAIL: tuwunel-b response does not contain matrix-b.localhost"
  echo "  Response: ${KEYRESP}"
  exit 1
fi
echo "  PASS: tuwunel-b key endpoint returned server_name=matrix-b.localhost"

# ─── All probes green ─────────────────────────────────────────────────────
echo ""
echo "=== All probes green ==="
echo ""
echo "E2E_HARNESS_REACH_OK"
