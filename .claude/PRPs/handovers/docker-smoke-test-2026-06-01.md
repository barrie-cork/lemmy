# Handover — Docker Smoke Test Session 3 close (2026-06-01)

## Current state

**Branch:** `governance-v0`  
**Last commit:** `9efbc7499` — docs(smoke-test): Phase 7 PASS — admin endpoints fully verified  
**Docker stack:** Running on localhost:8536 (API), localhost:1236 (UI). All 5 containers healthy.

## Phase completion

| Phase | Status |
|---|---|
| 0 — Boot health | ✅ PASS |
| 1 — Auth baseline | ✅ PASS |
| 2 — Reputation baseline | ✅ PASS |
| 3 — Case lifecycle | ✅ PASS |
| 4 — Appeal flow | ✅ PASS |
| 5 — Endorsement | ✅ PASS |
| 6 — Log integrity | ✅ PASS |
| 7 — Admin endpoints | ✅ PASS |
| **8 — Federation** | **⏳ NEXT** |
| 9 — Passkey/MFA | ⏳ |

## Live DB state (Docker volume — not persisted to git)

| Item | Value |
|---|---|
| Admin JWT (sub=1, person_id=2) | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxIiwiaXNzIjoibG9jYWxob3N0IiwiaWF0IjoxNzgwMzI4NDkyLCJleHAiOjE3ODA5MzMyOTJ9.vWXSVZr-FAFTpiN0baDdBwVKI4LpUshBtZAlsLheqQQ` |
| Reporter1 JWT (sub=2, person_id=4) | `eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIyIiwiaXNzIjoibG9jYWxob3N0IiwiaWF0IjoxNzgwMzI4NDk2LCJleHAiOjE3ODA5MzMyOTZ9.emHslA57L_NhvsZ29QfoF3StuUCxjTl99gHzXmj_4_Y` |
| Community | `governancetest` (id=2) |
| Post id=1 | author=admin (person_id=2) |
| Case id=1 | status=`Appealed`, winning_decision=`Warning` |
| Appeal id=1 | status=`Requested`, requester=admin (defendant) |
| governance_log | 17 entries, hash chain intact |
| governance_config patches | panel_size=1, quorum=1, sponsor_min_account_age_days=0, sponsor_min_endorsement_strength=0 |
| moderation_case.target_person_id | =2 (manually set for appeal smoke test) |
| local_user.accepted_application | =true for admin (person_id=2) |

## Next action: Phase 8 — Federation

**Goal:** Verify outbound-only federation with a second vanilla Lemmy instance (ADR-014). Brehon governance signals are fork-only AP types — the remote won't parse them, but federation must not error or block.

**Approach:** Spin up a second vanilla Lemmy stack on a different port (e.g. 8537) from `C:/Users/barri/Developer/brehon-fork/docker/`. It can use the same Dockerfile with a different `lemmy.hjson` (different hostname, different port). Then:

1. Configure federation between the two instances (add each other as allowed instances)
2. Create a post on the Brehon instance; verify it federates to the vanilla instance
3. Confirm no AP errors in Brehon instance logs for governance signal types

**VERIFY AT RESUME:** Check `docker ps` to confirm the primary stack (port 8536) is still running — Docker volumes survive session boundaries but the containers may have been stopped.

## Key files

| File | Purpose |
|---|---|
| `.claude/PRPs/reports/docker-smoke-test-plan.md` | Full test plan with all 14 bugs and per-phase checklists |
| `.claude/PRPs/reports/docker-smoke-test-findings.md` | Structured API findings for docs improvement (committed `96e2ebb9c`) |
| `docker/docker-compose.yml` | Primary Brehon stack |
| `docker/lemmy.hjson` | Lemmy config (hostname, admin credentials, federation settings) |
