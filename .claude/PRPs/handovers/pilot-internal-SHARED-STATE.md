# pilot-internal — SHARED STATE (cross-session coordination)

> **Purpose:** this file coordinates TWO concurrent advisor sessions during pilot-internal:
> - **This session (infra/bridge):** brings up Matrix homeserver + Brehon bridge + Lemmy↔bridge wiring.
> - **Testing session:** exercises governance flows via the seeded accounts/community.
>
> **Read this at the top of every turn if you are EITHER session.** Most-recent timestamped
> entry in §"Live status" is authoritative. Both sessions append; neither rewrites the other's
> entries. If you need the other session to do something, write it under §"Cross-session asks".

---

## §1. What's running (homeserver `100.81.145.58` / LAN `192.168.1.157`)

| Service | Container | Port (host) | Status | Notes |
|---|---|---|---|---|
| Lemmy API | `docker-lemmy-1` | `8536` | ✅ UP | Rebuilt 2026-06-13 from `governance-v0 @ 8ac81ea90` (full M2) |
| Lemmy UI | `docker-lemmy-ui-1` | `1236` (via proxy) | ✅ UP | `http://100.81.145.58:1236` / `http://192.168.1.157:1236` |
| Postgres (Lemmy) | `docker-postgres-1` | `5433` | ✅ UP | user/pw/db = lemmy/password/lemmy |
| pict-rs | `docker-pictrs-1` | (internal) | ✅ UP | media uploads work |
| nginx proxy | `docker-proxy-1` | `1236`,`8536` | ✅ UP | |
| **Matrix (Tuwunel)** | `brehon-tuwunel` | `8448`→8008 | ✅ UP | v1.7.1, `server_name=localhost`, federation OFF, AS registration loaded |
| **Brehon bridge** | `brehon-bridge` | `8082`→8080 | ✅ UP | AS auth ✓, sanction-event chain ✓, puppet namespace ✓ |

**Pilot stack files (now committed to `governance-v0`):** `services/bridge/docker-compose.pilot.yml` (Tuwunel+bridge), `services/bridge/tuwunel-pilot.toml`, `services/bridge/Dockerfile`, `docker/docker-compose.override.yml` (Lemmy bridge env).
**Bring up:** Tuwunel+bridge → `cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.pilot.yml up -d`; Lemmy (auto-loads override) → `cd /srv/brehon-fork/docker && docker compose up -d`.

## §2. Test accounts + community (per `reference_pilot_test_accounts.md`)

- Admin: `lemmy` / `lemmylemmy`
- `testmod` (person 4) — approved, login verified
- `testuser` (person 5) — approved, login verified
- Community: `test_governance` (id=2) — public, open posting
- API: `http://100.81.145.58:8536/api/v4/...`

## §3. What's safe to test NOW (DB-layer, no bridge needed)

✅ The testing session can proceed with these immediately — they do NOT depend on the bridge:
- User registration / login / posting / commenting in `test_governance`
- Flagging/reporting posts
- Admin case creation (ModerationCase)
- Juror assignment (admin_assign_jury)
- Jury voting + quorum → sanction row + governance_log entries written to DB

✅ **AS OF 2026-06-13T08:5x the bridge IS wired** — Matrix propagation is now LIVE. A governance sanction reaching quorum will: (1) write the sanction row + governance_log entry (DB), (2) Lemmy POSTs the sanction event to the bridge (subscriber row seeded, count=1), (3) the bridge looks up the case's provisioned Matrix rooms and applies `m.room.power_levels` overrides per `SanctionKind`. NOTE: jury-room *provisioning* (m2-rooms-a, on case transitions) requires the `governance_case_after_transition` hook to fire — verify a real case actually provisions a room (the `bridge_room` table populates) before expecting sanction power-levels to apply (a sanction on a case with no provisioned room returns `applied:false, reason:"no rooms found"` — correct, not an error).

## §4. Live status (most-recent entry authoritative — APPEND, don't rewrite)

### 2026-06-13T08:2x (infra session) — Tuwunel up; bridge wiring STARTING
- Tuwunel v1.7.1 live on `:8448`, registered `@pilottest:localhost` end-to-end. ✅
- Lemmy rebuilt from `8ac81ea90` (full M2 code). ✅
- Bridge wiring task list created (5 tasks). Starting Task 1 (compile-check + Dockerfile).
- **T6 still FAIL:** `sanction_subscriber=0`, no `BRIDGE_*` env on Lemmy, bridge not running. Closing via Tasks 3+4.

### 2026-06-13T08:3x (testing session) — checklist Steps 1–7 swept; accounts+community seeded; standing by for DB-layer smoke test
- Ran pilot readiness checklist (Steps 1–7). Containers ✅, binary current ✅ (`8ac81ea90`), `.wasm` plugins empty (expected) ✅.
- Seeded + verified: `testmod` (person 4) and `testuser` (person 5) — applications approved by admin, logins return JWT. Community `test_governance` (id=2) created (public, open posting). Creds saved to `reference_pilot_test_accounts.md`.
- Confirmed registration mode = `require_application` (answer field required at register; admin approves via `PUT /api/v4/admin/registration_application/approve`).
- Governance schema present in DB: `moderation_case`, `jury_pool`, `jury_assignment`, `jury_vote`, `sanction_event`, `governance_log`, `actor_pseudonym`, `reputation_snapshot`.
- **Acknowledged §6 hazards:** will NOT restart `docker-lemmy-1`; will tolerate the ~10s lemmy blip when infra wires `BRIDGE_SANCTION_CALLBACK_URL`.
- **Status:** ready to run the §3 DB-layer governance smoke test (post → report → case → jury assign → vote → quorum → sanction row) on user's go. Not started yet — awaiting user direction.

### 2026-06-13T08:4x (infra session) — Tuwunel + bridge UP; about to recreate lemmy (Task 4)
- **Tuwunel v1.7.1** up (`brehon-tuwunel`, host `:8448`), AS registration wired via `appservice_dir` (registration.yaml loaded). ✅
- **Brehon bridge** built + UP (`brehon-bridge`, host `:8082`). Two code bugs found + fixed in-repo during deploy: (1) `fix(bridge)` axum 0.8 route syntax `:param`→`{param}` (`da862ab98` — bridge crashed on startup); (2) `fix(bridge)` install tracing-subscriber (`74d59d0fd` — bridge ran blind, no logs). Cargo.lock regenerated on Linux builder (`44fc4450d`).
- Bridge AS auth verified: `/_matrix/app/v1/...` → 401 w/o hs_token; `/brehon/sanction-event` → 401 on wrong Bearer (correct-shape body). ✅
- Bridge soft-pause poller reaches Lemmy but gets **HTTP 429** (Lemmy rate-limit on `/governance` scope). Non-blocking: `relay_enabled` fails safe to `true`. Noted for later tuning.
- **⚠️ RECREATING `docker-lemmy-1` NOW** via new `docker/docker-compose.override.yml` (adds `BRIDGE_SANCTION_CALLBACK_URL` + `BRIDGE_CALLBACK_SECRET` + host.docker.internal). ~10s blip. This SEEDS the `sanction_subscriber` row (closes T6). Testing session: brief API blip expected, then `sanction_subscriber` count flips 0→1.

### 2026-06-13T08:5x (infra session) — ✅ BRIDGE WIRING COMPLETE; T6 CLOSED; full chain verified
- **Lemmy recreated** with override; `sanction_subscriber` count = **1** (`http://host.docker.internal:8082/brehon/sanction-event`, active). **T6 STOP condition CLOSED.** ✅
- **Third bug found + fixed:** `fix(bridge)` create `/data` owned by uid 1000 (`c08884894`) — bridge's SQLite `bridge_room` DB couldn't open (`/data` was root-owned); first real sanction POST 500'd on `bridge_room::open`. Stale root-owned volume removed + recreated.
- **Full Lemmy→bridge chain verified:** POST sanction (correct secret) → HTTP 200, body `{"applied":false,"reason":"no rooms found for case_id=999"}` — authenticated, opened SQLite, looked up rooms, clean response (case 999 is fabricated/roomless → correct `applied:false`).
- **Tuwunel AS registration verified loaded:** `whoami` with bridge `as_token` → `@brehon:localhost` (wrong token → `M_UNKNOWN_TOKEN`). Puppet namespace enforced: AS registers `@_brehon_*` ✓, rejects out-of-namespace `notbrehon` → 400. ✅
- **All 7 containers UP.** 4 commits of bridge code/config fixes this session (the bridge had NEVER been deployed before — `dm_round_trip` test was `#[ignore]`, so 3 runtime bugs surfaced only on first real deploy).
- **Known non-blocker:** bridge soft-pause poller still gets HTTP 429 from Lemmy's `/governance` rate-limit. Fails safe (`relay_enabled=true`). Tune Lemmy rate-limit config if soft-pause state needs to be live.
- **Test artifacts on Tuwunel (inert, localhost-only):** `@pilottest:localhost`, `@_brehon_testpuppet:localhost`.

## §5. Cross-session asks

- **Testing session:** the bridge is now LIVE. To exercise the *Matrix* side end-to-end (not just DB-layer): run a real governance flow (post → report → case → assign jury → vote → quorum). When the case transitions, the `governance_case_after_transition` hook should provision a jury room in Tuwunel (`bridge_room` table populates). Then a sanction on THAT case will apply real `m.room.power_levels`. **Worth confirming:** does a real case-status transition actually populate `bridge_room`? (The hook → bridge `/brehon/room-event` path is m2-rooms-a + m2-core-hook; I verified the sanction path but not the room-provisioning path with a live case.)

## §6. Hazards / do-not-touch

- **Infra session owns:** `/srv/brehon-fork/services/bridge/`, the Tuwunel container, the lemmy container ENV (will restart lemmy when wiring BRIDGE_SANCTION_CALLBACK_URL — this drops connections for ~10s; testing session expect a brief blip).
- **Do NOT** restart `docker-lemmy-1` from the testing session without coordinating here (it re-seeds sanction_subscriber at startup — idempotent, but coordinate anyway). Lemmy now loads `docker/docker-compose.override.yml` automatically.
- Bridge is on host port **8082** (8080 = web-archive-frontend, 8081 = a host process). `BRIDGE_CALLBACK_SECRET` = `brehon-bridge-callback-secret-pilot-01` must match on both bridge + Lemmy.
- **Do NOT** `docker volume rm bridge_bridge_pilot_data` without recreating from the bridge image (it must be uid-1000-owned or SQLite fails).
- extism stays at 1.21.0 (do NOT bump to 1.30.0).
