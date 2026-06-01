# Handover — Docker Smoke Test Phase 8 (2026-06-01)

## Current state

**Branch:** `governance-v0`  
**Last commit:** `ec505ffaa` — feat(smoke-test): Phase 8 federation infrastructure + results  
**Docker stack:** Both stacks running.

## Stack status

| Container | Status | Port |
|---|---|---|
| docker-lemmy-1 (Brehon) | Up, activity sending ENABLED | 8536 (via proxy) |
| docker-lemmy-vanilla-1 (Vanilla nightly) | Up | 8537 (direct) |
| docker-postgres-1 | Up (healthy) | 5433 |
| docker-postgres-vanilla-1 | Up (healthy) | 5434 |
| docker-proxy-1 | Up | 1236, 8536 |
| docker-lemmy-ui-1 | Up (healthy) | — |
| docker-pictrs-1 / docker-pictrs-vanilla-1 | Up | — |

## Phase completion

| Phase | Status |
|---|---|
| 0–7 | ✅ PASS (see handover 2026-06-01) |
| **8 — Federation** | ✅ PARTIAL PASS |
| 9 — Passkey/MFA | ⏳ NEXT |

## Phase 8 results

**ADR-014 core claim VERIFIED:** Brehon outbound AP (Follow, Create) sent to vanilla nightly without error. Vanilla accepted without error. 0 dead instances throughout.

**What passed:**
- 8.1 Resolve vanilla user from Brehon ✅
- 8.3 Resolve vanilla community from Brehon ✅
- 8.4 Follow vanilla community (Follow activity delivered, vanilla accepted) ✅
- 8.5 Post Create activity sent (0 errors, was_skipped: false) ✅
- 8.6 No governance-specific AP type errors ✅
- 8.7 0 dead instances ✅

**What blocked bidirectional loop:**
- Brehon's AP id is `https://localhost/...` — from inside the vanilla container, `localhost` = vanilla itself, not Brehon
- Vanilla's AcceptFollow cannot reach `https://localhost/inbox`
- Post stayed `federation_pending: true` — waiting for AcceptFollow that never arrives
- **BUG-15:** Brehon hostname in hjson is `localhost` (no port). Fix: change to `host.docker.internal:8536` + clear Brehon DB volume + re-run setup. Deferred.

## Key credentials (for next session)

Get fresh JWTs by running:
```bash
# Brehon
curl -s -X POST http://localhost:8536/api/v3/user/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email":"lemmy","password":"lemmylemmy"}' | python3 -c "import sys,json; print(json.load(sys.stdin).get('jwt',''))"

# Vanilla
curl -s -X POST http://localhost:8537/api/v3/user/login \
  -H "Content-Type: application/json" \
  -d '{"username_or_email":"lemmy_vanilla","password":"lemmylemmy"}' | python3 -c "import sys,json; print(json.load(sys.stdin).get('jwt',''))"
```

Note: If Brehon login returns `too_many_requests`, restart the container to reset in-memory rate limit:
```bash
docker compose -f docker-compose.yml -f docker-compose-fed-enable.yml restart lemmy
```

## For Phase 9 (Passkey/MFA)

Needs a WebAuthn client. The Brehon stack supports `webauthn-rs` (ADR-012). The UI is at `http://localhost:1236`. Needs a second browser or device capable of WebAuthn ceremony.

## Infrastructure files committed (ec505ffaa)

- `docker/docker-compose-vanilla.yml` — vanilla nightly stack
- `docker/lemmy-vanilla.hjson` — vanilla config (host.docker.internal:8537 hostname, federation enabled)
- `docker/docker-compose-fed-enable.yml` — removes LEMMY_DISABLE_ACTIVITY_SENDING from Brehon
- `docker/lemmy.hjson` — added `federation { concurrent_sends_per_instance: 1 }`
