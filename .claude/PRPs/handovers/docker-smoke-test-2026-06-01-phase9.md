# Handover — Docker Smoke Test Phase 9 (2026-06-01)

## Status: ALL PHASES COMPLETE

**Branch:** `governance-v0`  
**Phase 9 completed:** TOTP MFA — all 6 checks PASS.

## Final phase completion

| Phase | Status |
|---|---|
| 0–7 | ✅ PASS |
| 8 — Federation | ✅ PARTIAL PASS (ADR-014 outbound verified; BUG-15 deferred) |
| **9 — TOTP MFA** | ✅ PASS |

## Phase 9 findings

- Implementation uses TOTP (`totp-rs`, RFC 6238 SHA1), **not** WebAuthn passkeys.
- Routes: `POST /api/v4/account/auth/totp/generate` (get secret), `POST /api/v4/account/auth/totp/edit` (enable/disable).
- Login field for TOTP: `totp_2fa_token` on `/api/v3/user/login`.
- Login without token when TOTP enabled → `missing_totp_token` (400).
- Wrong token → `incorrect_totp_token` (400).
- JWT issued post-TOTP works on all governance endpoints.

## Smoke test plan corrections made

The smoke-test-plan.md Phase 9 section was labelled "Passkey MFA (needs second device)" — updated to reflect TOTP reality. All 4 original checks reframed and passed.

## Stack state (end of session)

Both stacks still running. TOTP disabled on admin account (clean state).

| Container | Status |
|---|---|
| docker-lemmy-1 (Brehon) | Up, activity sending ENABLED |
| docker-lemmy-vanilla-1 (Vanilla nightly) | Up on 8537 |
| Both postgres, proxy, ui, pictrs | Up/healthy |

## Remaining open item

**BUG-15:** Brehon hostname=`localhost` (no port). Vanilla cannot deliver AcceptFollow back to Brehon. Fix: change `hostname` in `docker/lemmy.hjson` to `host.docker.internal:8536` + clear Brehon DB volume + re-run setup. Deferred.

## For next session

Nothing blocking. Full smoke test (Phases 0–9) of Brehon governance v0 Docker stack is complete. The only remaining item is BUG-15 (optional fix for bidirectional federation).

To resume stack:
```bash
cd C:/Users/barri/Developer/brehon-fork/docker
docker compose up -d
docker compose -f docker-compose-vanilla.yml up -d
```
