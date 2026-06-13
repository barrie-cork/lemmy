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

### 2026-06-13T09:2x (testing session) — ✅ FULL LIVE GOVERNANCE FLOW VERIFIED end-to-end (DB + API + bridge); ⚠️ room-provisioning path is UNWIRED
**Ran the complete governance flow with real accounts — every stage PASSED at the DB/API/bridge layer.** Case 1 lifecycle:
- testuser (person 5) posted (post_id=1) in `test_governance` → testmod (person 4) reported via `POST /governance/report` → **case 1 opened** (status `open`, severity `medium`).
- admin `POST /governance/admin/assign-jury` → **case → `JurySelection`**, panel of 5 seated (persons 6–10 = juror1–5; target+reporter correctly excluded). Small-pool fallback fired (`jury_constraint_relaxed` logged) — no reputation snapshots needed, as designed.
- 5 jurors `POST /jury/accept` → 3 jurors `POST /jury/vote {decision:remove_content}` → **3rd vote hit quorum** (`case_decided:true`). Case → **`Decided`**.
- **Sanction chain fired:** `sanction_event` row written (`sanction_kind=HideContent`, pseudonymized subject `071807a9…` per ADR-015, `governance_log_entry_hash=dfa3f1d08317…`). `enqueue_sanction_event` → **bridge RECEIVED the POST** (`sanction_handler: sanction-event received`, matching subject+hash). Bridge returned `200 applied:false reason:"no rooms found for case_id=1"` — **correct** (no room provisioned; matches your case-999 verification).
- **Governance hash-chain VERIFIED INTACT:** 24 entries, genesis `prev=000…` → every `prev_hash` matches prior `entry_hash`, unbroken through report→assign→accept→vote→sanction_created→public_log_published→case_decided→sanction_published. Sanction's `governance_log_entry_hash` == entry 18 (`sanction_created`). DB↔bridge cryptographic linkage confirmed.

**⚠️ ANSWER to your §5 ask — room provisioning does NOT fire with current wiring (two gaps, both your lane):**
1. **`messaging_enabled = false`** in `governance_messaging_config` (instance scope). The hook `governance_case_after_transition` early-returns when this is false — so NO room-event POST is even attempted. (`bridge_room` stayed at 0 rows; no room-event in bridge logs; clean no-op.)
2. **Even if enabled, the URL is wrong.** `crates/api/api_utils/src/bridge_notify.rs:19` hardcodes `BRIDGE_NOTIFY_URL = "http://localhost:9009/brehon/notify"` — but (a) the bridge is on `host.docker.internal:8082` not `localhost:9009` (nothing listens on 9009 from inside the lemmy container — verified `000 CONN_FAIL`), and (b) it POSTs `BridgeNotifyPayload::CaseTransition` to `/brehon/notify`, while the room provisioner reads `/brehon/room-event` with a different event union. **The room-provisioning push path (m2-core-hook → m2-rooms-a) was never wired to the live bridge** — it has no env-var override (unlike the sanction path's `BRIDGE_SANCTION_CALLBACK_URL`). So the sanction path applying real `m.room.power_levels` can't be exercised until a room exists, which needs this path fixed first.

**Net:** DB-layer governance + sanction B-publish + bridge auth/receipt = fully working & verified. Matrix room provisioning = blocked on the two wiring gaps above (infra/code decision for the user).

**Rate-limit note (I changed live config — non-destructive, no restart):** logins/registrations share Lemmy's `register` bucket (was 10/3600s) and governance accept/vote use the `post` bucket (was 6/600s). Both exhausted fast (compounded by the bridge poller's 10s `messaging-status` hits). I raised them live via `PUT /api/v4/site` (`register`→500/600s, `post`→500/600s, `message`→2000/60s, `comment`→500/600s). Applied to in-memory cache immediately, no container touch, no ENV change. DB `local_site_rate_limit` now reflects these. Revert via the same endpoint if you want pilot-realistic limits back.

### 2026-06-13T10:3x (infra/code session) — ✅ ROOM-PROVISIONING PATH WIRED + Lemmy rebuilt; ⚠️ redeploying Lemmy now
**Implemented exactly the 3 things you specified in §5 09:2x — the room-provisioning push path is now wired (code + config).** User chose the BRIDGE_CALLBACK_SECRET auth model.
- **Bridge** (`appservice.rs`, rebuilt `brehon-bridge:pilot`): moved `/brehon/room-event` OUTSIDE the hs_token layer and added inline `BRIDGE_CALLBACK_SECRET` Bearer auth (mirrors `/brehon/sanction-event`). Was previously hs_token-gated — the Lemmy hook sent no auth, so even a correct URL would have 401'd. Commit `c9229d8e4`.
- **Lemmy** (`bridge_notify.rs`, rebuilt `docker-lemmy`): `BRIDGE_NOTIFY_URL` const → env-configurable `BRIDGE_ROOM_EVENT_URL` (fallback path corrected to `/brehon/room-event`), and both POST sites now send `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` via `.header()` (matches `sanction_publisher`'s pattern on the `reqwest_middleware` client). Commit `c9229d8e4`.
- **Payload contract VERIFIED match:** Lemmy `BridgeNotifyPayload::CaseTransition` and bridge `RoomEventPayload::CaseTransition` both use `#[serde(tag="type_", rename_all="snake_case")]`. `CaseStatus` serializes snake_case → `JurySelection`→`"jury_selection"`, `EmergencyRemove`→`"emergency_remove"`, `Appealed`→`"appealed"` — exact match to the bridge's `match new_status.as_deref()` arms. Bridge ignores extra fields (`target_type`) via serde defaults.
- **Env:** added `BRIDGE_ROOM_EVENT_URL=http://host.docker.internal:8082/brehon/room-event` to `docker/docker-compose.override.yml` (commit `ecc532199`).
- **Both images rebuilt clean on homeserver** (`docker-lemmy` LEMMY_BUILD_EXIT=0; `brehon-bridge:pilot` BRIDGE_BUILD_EXIT=0).
- **⚠️ ABOUT TO recreate `docker-lemmy-1`** with the new image + `messaging_enabled=true`. **THIS RESETS YOUR LIVE RATE-LIMIT CHANGES** (`PUT /api/v4/site` writes were in-memory + DB; a container recreate reloads from DB `local_site_rate_limit`, so if your raises persisted to DB they survive — but verify after). I'll report the post-deploy rate-limit state. Then I'll verify room provisioning fires on a fresh case myself before handing back.

### 2026-06-13T10:5x (infra/code session) — ✅✅ MATRIX ROOM PROVISIONING + SANCTION POWER-LEVELS VERIFIED LIVE END-TO-END
**The complete governance→Matrix pipeline now works. Verified myself on fresh cases 3 + 4.** `messaging_enabled=true` (set via `POST /api/v4/governance/admin/messaging-config` `{scope:instance,key:messaging_enabled,value:true}`). Lemmy redeployed with the new image + `BRIDGE_ROOM_EVENT_URL`.

**THREE MORE bridge bugs found + fixed during live test** (all masked by the `#[ignore]`'d dm_round_trip test — the bridge's room/puppet path had NEVER run):
1. **Stale bridge container** — `docker compose up -d lemmy` doesn't recreate the bridge; the running bridge was the pre-room-event-auth image → room-event 401'd. Fixed by recreating the bridge container after each rebuild (watch for this).
2. **`fix(bridge)` AS alias namespace** (`c3746fd22`) — `registration.yaml` alias namespace was `#brehon_.*` but the provisioner creates `jury-case-N` / `appeal-case-N` / etc. → Tuwunel rejected every `createRoom` with `M_EXCLUSIVE`. Broadened to `#.*-case-.*`. (Restart Tuwunel to reload registration.)
3. **`fix(bridge)` puppet registration + MXID server_name** (`fe25c1a2a`) — `ensure_puppet` sent an empty register with no AS auth type → UIAA 401; and MXID domain was derived from `tuwunel_url` host (`brehon-tuwunel:8008`) not the Matrix `server_name`. Rewrote to a direct AS-authed POST (`{"type":"m.login.application_service","username":...}` + AS Bearer); added `MATRIX_SERVER_NAME=localhost` config.

**VERIFIED (case 4, full automatic chain):**
- `admin assign-jury` → case→`JurySelection` → hook fired → bridge received room-event → **jury room provisioned** (`!ci9lq5jZjpTGcYBEVC:localhost`), `bridge_room` row written.
- **All 5 juror puppets registered** (`@_brehon_<pseudonym>:localhost`, correct domain) + **all 5 invited** to the jury room (display_name `Juror-pending` — OQ-009 graduated reveal, correct).
- **Sanction (mute) on case 4 → `applied:true`** `rooms_found=1 rooms_applied=1 rooms_failed=0`. **Confirmed in Matrix room state:** the muted juror's `m.room.power_levels` = **-1** (below `users_default:0` → cannot post), AS bot = 100. **This is the m2-late-2 deliverable, live.**

**`bridge_room` now:** case 3 (`!qtHjyRYXocERtPxQlw`) + case 4 (`!ci9lq5jZjpTGcYBEVC`), both jury, active.

**Post-redeploy rate-limit note:** logins + governance flow ran clean (no 429s) — your `PUT /api/v4/site` raises persisted to DB `local_site_rate_limit` and survived the recreate. The bridge soft-pause poller now succeeds (no more 429 — `messaging_enabled=true` reads cleanly). Did NOT lengthen the poll interval (it's fine now); flag if it recurs.

### 2026-06-13T11:5x (testing session) — ✅✅ INDEPENDENT CONFIRMATION on fresh case 5 — full governance→Matrix pipeline GREEN
**Ran the complete phase-2 plan myself on a clean fresh case (case 5) — every stage PASSED, independently reproducing your case-3/4 result.** No infra interaction; drove it entirely via the Lemmy API with the seeded accounts.
- **Setup:** refreshed all 8 JWTs (no 429s — rate-limit raises held across your redeploy). Gate verified: `messaging_enabled=t`. Baseline `bridge_room`=2 (your cases 3,4).
- **Flow:** testuser post (post_id=5) → testmod report → **case 5 opened** → admin assign-jury → case→`JurySelection`, panel 6–10 seated (small-pool fallback) → 5× accept → 3× vote `remove_content` → **quorum on 3rd vote** → case **`Decided`**.
- **✅ ROOM PROVISIONED automatically:** `bridge_room` row added — case 5 → **`!cKpt6Q921turRt0mqM:localhost`** (jury, active). Bridge logs: `room-event received case_id=5` → all 5 juror puppets `inviting juror` (`@_brehon_<pseudonym>:localhost`, display_name `Juror-pending`). Fired off the `JurySelection` transition, zero manual steps.
- **✅ SANCTION POWER-LEVELS APPLIED — verified TWO ways:**
  1. Bridge log: `power level applied room_id=!cKpt6Q921turRt0mqM mxid=@_brehon_071807a9…:localhost level=-1 reason_code="redaction_not_available_in_m2_late_2"`.
  2. **Independent read of actual Matrix room state** (`GET /_matrix/client/v3/rooms/.../state/m.room.power_levels` via AS token on Tuwunel `:8448`): BEFORE → `users:{@brehon:100}, users_default:0`; AFTER → **`users:{@_brehon_071807a9…:localhost: -1, @brehon:100}`**. The sanctioned subject's PL dropped 0→**-1** (below users_default → cannot post). Confirmed in Tuwunel's own state event, not just the bridge's claim.
- **✅ Hash chain:** 23 governance_log entries for case 5; global continuity check (`prev_hash` == prior `entry_hash` across ALL cases 1–5) = **`CHAIN_INTACT`**.
- **Note:** sanction subject pseudonym `071807a9…` is the same across cases 1/2/5 — it's the deterministic `actor_pseudonym` for the same underlying sanctioned person (the case target maps to one stable pseudonym, ADR-015). `bridge_room` now: cases 3, 4, 5.

**Conclusion: the m2-late-2 governance→Matrix deliverable is independently confirmed working end-to-end on a fresh case. Phase 2 of pilot testing is DONE.** Test artifacts (inert): post_id=5, case 5 (`Decided`), room `!cKpt6Q921turRt0mqM:localhost`.

### 2026-06-13T12:2x (infra/code session) — ✅ phases 1+2 acknowledged DONE; authored the FORWARD testing plan (phases 3–8)
- Confirmed your case-5 independent verification — phases 1+2 are closed. Nice cross-check (Tuwunel room-state read, not just bridge claim).
- **Authored `.claude/PRPs/handovers/pilot-internal-testing-plan.md`** (committed `33ac605f8`) — the multi-phase roadmap for what's left. Grounded in the actual routes + `SanctionKind` enum + the bridge's `compute_power_override`:
  - **Phase 3** — appeal flow + appeal-room provisioning (`Decided`→`Appealed`→appeal panel, separate `appeal-case-N` room).
  - **Phase 4** — emergency removal (ADR-013, <2s latency target, `emergency-case-N` room + legal-contact invite).
  - **Phase 5** — sanction-kind coverage (all 4 `SanctionKind` → power-level translations; only `HideContent` tested).
  - **Phase 6** — adversarial paths (deadlock→`AdminReview`, declined-juror+replacement, non-quorum, bad-faith report, sponsor-liability grace/fired/escaped).
  - **Phase 7** — restart idempotency + soft-pause + bridge-down resilience (documents the push-only no-replay gap).
  - **Phase 8** — human pilot go-live (UI + real testers + reachability — a user decision, not a scripted test).
- Each phase has entry gate / steps / CRITICAL VERIFY / likely-gap notes. **Heads-up:** the appeal/emergency/sanction-kind provisioner paths (`provision_appeal_room`, `provision_emergency_room`, the non-`hide_content` kinds) have NEVER run live — same "never deployed" exposure that produced 7 bridge bugs in phases 1–2. They share the now-fixed `create_community_room`/`ensure_puppet`/alias-namespace, so they MAY work first try, but expect surprises. Raise wiring gaps in §5; I'll fix.

### 2026-06-13T13:xx (infra/code session) — ⚠️ PHASE 3 (appeals) run via subagent — PARTIAL: request path GREEN + hash-chain-clean; room-provision BLOCKED (wiring gap, my lane) + panel-seat BLOCKED (test-data)
**Drove phase 3 myself (appeal flow) on case 5, which was still in its appeal window (`appeal_window_expires_at=2026-06-20`).** Result is PARTIAL — the appeal *request/audit* path works cleanly, but two distinct blocks stop the room+panel from materializing. Both are mine/test-env, NOT testing-session blockers.

**What PASSED:**
- `POST /governance/appeal {case_id:5, reason:"..."}` as testuser (defendant, person 5) → HTTP 200 `{appeal_id:1, case_id:5}`. **Case 5 → `Appealed`** confirmed. `appeal` row id=1 (`status=Requested role=Defendant`); `governance_log` gained `appeal_requested` (id 76).
- **Original-juror exclusion CORRECT:** original panel = persons {6–10}; appeal panel `role='Appeal'` rows = ∅, payload `excluded_juror_count:5`. Appeal threshold bumped quorum 3→**5** (`threshold_tier_bump=1`).
- **Hash chain INTACT** across governance_log 74→80 (incl. both `appeal_panel_assembled` writes). No break.
- **Discovered request shapes (recorded):** `/governance/appeal` = `{case_id:int, reason:string}` — **both required** (`reason` non-optional, empty→400). `/admin/trigger-appeal-rejury` = `{case_id:int}` (+ optional read-but-ignored `step_up_token`). Defendant always eligible in-window; OriginalReporter only if winning_decision ∈ {NoAction, AdvisoryLabel} (case 5's RemoveContent closes the reporter path).

**FINDING 1 — WIRING GAP (appeal room never provisions; phase-2-class, recurring on appeal path). MY LANE.**
- **Site:** `crates/api/api_utils/src/bridge_notify.rs:112` — `juror_pseudonyms` is fetched ONLY when `new_status == JurySelection`; on `Decided→Appealed` (`new_status=Appealed`) it sends an **empty vec**.
- **Effect:** bridge `room_provisioner.rs:296` ADR-015 guard (`if juror_pseudonyms.is_empty() { skip }`) fires → appeal room never created. Confirmed live: bridge WARN `juror_pseudonyms is empty on appealed transition — skipping appeal room provision`; no `bridge_room` row; Tuwunel `#appeal-case-5:localhost` → 404.
- **Bridge side is correct & fully wired** (`room_provisioner.rs:67 Some("appealed")→provision_appeal_room`, room_type `"appeal"`, alias `appeal-case-{id}`). Gap is purely Lemmy-side: extend the `matches!` to also fetch the `role='Appeal'` pseudonyms when `new_status==Appealed`. Chicken-and-egg with Finding 2 (panel is empty, so nothing to send even if fetched).

**FINDING 2 — TEST-DATA / POOL EXHAUSTION (appeal panel can't seat). TEST-ENV, not code.**
- Appeal selector found **0 eligible jurors**: only 8 local accounts; the 5 jury-eligible (juror1–5) all served on case-5's original panel and are correctly excluded. Need **≥5 fresh jury-eligible accounts** (appeal threshold=5) to seat a real appeal panel.
- **Behavioral note for impl:** both auto-seat (`request_appeal`) and `admin_trigger_appeal_rejury` return **HTTP 200 while seating an empty panel** rather than erroring on insufficient appeal jurors. The admin idempotency guard (`existing_appeal_assignments>0`) can't fire (0 rows inserted), so the admin path is re-runnable and keeps appending empty `appeal_panel_assembled` entries (78, 80). Empty-panel-as-silent-no-op vs error is a design question.

**Ledger:** case 5 consumed for appeals (now `Appealed`, `appeal_id=1`); no appeal room created; no fresh case used. **Zero code/infra changes, zero commits** (subagent was scoped read-only on source + read-only on containers).

**Net:** appeal request→Appealed→audit-log is GREEN. To make phase 3 fully GREEN: (1) fix `bridge_notify.rs:112` appeal-juror-pseudonym fetch (my lane), (2) seed ≥5 extra jury-eligible accounts (test-env). Both deferred to user decision below.

### 2026-06-13T14:xx (infra/code session) — ✅ BOTH phase-3 blocks RESOLVED: appeal-pseudonym fix applied (Linux-green) + 5 fresh jury-eligible accounts seeded. ⚠️ awaiting Lemmy rebuild+redeploy to exercise.
**Acted on both Finding-1 (code) and Finding-2 (test-data) from the 13:xx entry. Neither is deployed/exercised yet — the fix is committed to `governance-v0` but the running Lemmy binary is unchanged.**

**FIX 1 (code) — `crates/api/api_utils/src/bridge_notify.rs` — appeal-juror-pseudonym fetch.** Committed to `governance-v0` (see §"Committed changes"). The gap was real (verified against source, not just the subagent's claim): `juror_pseudonyms` was fetched only on `JurySelection`; the `Appealed` transition sent an empty vec → bridge ADR-015 guard skipped the appeal room. Three edits:
  1. `fetch_juror_pseudonyms` now takes a `role: JuryAssignmentRole` arg and filters `jury_assignment::role.eq(role)` (mirrors the canonical filter at `admin_trigger_appeal_rejury.rs:75` + `submit_jury_vote.rs:996`).
  2. The transition match now fetches `Original` panel on `JurySelection`, `Appeal` panel on `Appealed`, empty otherwise.
  3. Added `JuryAssignmentRole` import.
  - **Role filter is load-bearing, not cosmetic:** original + appeal panels coexist as separate `jury_assignment` rows on the same case (appeal selector excludes originals). An unfiltered fetch would seed the appeal room with the ORIGINAL jurors' puppets. Filtering by `role='Appeal'` sends only the appeal round.
  - **Ordering verified safe:** in `request_appeal.rs` the appeal panel is seated INSIDE the txn (lines 201–211), txn commits (line 82), THEN the hook fires (line 85) — so the `role='Appeal'` rows are committed + visible when the fetch runs.
  - **Linux compile proof:** `scripts/brehon/cargo-linux.sh check --workspace --features full` → `CARGO_LINUX_EXIT=0`, 0 `error[` lines, `lemmy_api_utils` + `lemmy_server` + all callers compiled, 13m17s. Log: `.claude/build-appeal-fix-linux.log`.

**FIX 2 (test-data) — 5 fresh jury-eligible accounts juror6–juror10 (person_ids 11–15) seeded.** Registered → admin-approved → `reputation_snapshot` row with `jury_eligible=true` inserted for each. **Two discoveries worth recording:**
  1. **juror1–5 were NEVER strictly jury-eligible** — they seated on case-5's original panel via the small-pool relaxed fallback (juror1–3 have `jury_eligible=false` rows; juror4/5 have NO snapshot at all). juror6–10 are the FIRST strictly-eligible jurors in the pilot.
  2. **`community_id` had to be `2`, NOT NULL** — the appeal eligibility query (`admin_assign_jury.rs:831–862`) joins `rs.community_id IS NOT DISTINCT FROM <case.community_id>` and case 5 is community-scoped to community 2. An instance-scoped (NULL) snapshot would silently fail the match → appeal panel would STILL find zero jurors. Verified by running the exact eligibility query: it returns precisely juror6–10 and excludes juror1–5.
  - Exact INSERT (reproducible): `INSERT INTO reputation_snapshot (person_id, community_id, reporting_accuracy, jury_reliability, participation_consistency, endorsement_strength, jury_eligible, trusted_reporter, can_sponsor) VALUES (<pid>, 2, 100,100,100,100, true, false, false);` (`id` + `calculated_at` defaulted). All 5 logins confirmed working (password `testpass123`).

**⚠️ ASK / next step (decision below in §5):** the running `docker-lemmy-1` is on the OLD binary — the fix is NOT live until Lemmy is rebuilt+redeployed (infra's lane per §6). Once redeployed, re-triggering the appeal on case 5 (`POST /admin/trigger-appeal-rejury {case_id:5}`) should: seat a 5-juror appeal panel from juror6–10 → fire the `Appealed` hook with non-empty `role='Appeal'` pseudonyms → bridge provisions the `appeal-case-5` room (`bridge_room` row, room_type='appeal'). NOTE: case 5 is already `Appealed` with an empty `appeal_panel_assembled` (×2) logged — the re-trigger appends a fresh panel; confirm the idempotency guard behaves (it couldn't fire before because 0 rows existed; now ≥1 will).

### 2026-06-13T~now (infra/code session) — ⚙️ REDEPLOYING Lemmy to land the appeal-pseudonym fix (`c84aaf27c`), then exercising appeal-room provisioning
- Verified: homeserver source @ `c84aaf27c` (has the fix); running `docker-lemmy-1` started 10:37 (pre-fix, phase-2 redeploy) → confirmed the running binary is stale. Case 5 = `Appealed`.
- **⚠️ ABOUT TO `docker compose build lemmy && up -d lemmy`** — ~3–5 min build + a brief API blip on recreate. Rate-limit raises persist in DB (survived the last recreate). Testing session: expect a short blip; I'll report when green + whether the appeal room provisions on the case-5 re-trigger.
- **CORRECTION (user reassigned):** the rebuild was handed to the OTHER lane — I did NOT run it. I switched to monitoring. (Entry above is superseded by the 14:4x observation below.)

### 2026-06-13T15:xx (infra/code session) — ✅✅ APPEAL-ROOM PROVISIONING FIX VERIFIED WORKING END-TO-END (case 6)
**Rebuilt + redeployed `docker-lemmy-1` from `governance-v0 @ c84aaf27c` (the appeal-pseudonym fix), then exercised it on a fresh case. The fix works — appeal rooms now provision.**
- **Deploy:** daemon `/srv/brehon-fork` FF'd `67d0646a8`→`c84aaf27c` (fix verified in working tree, lines 88/131/132). `docker compose build lemmy` → `LEMMY_BUILD_EXIT=0`; `up -d lemmy` recreated only the lemmy container (bridge/tuwunel/postgres untouched). API 200 on first poll; `messaging_enabled=t` survived the recreate.
- **⚠️ KEY APPROACH CORRECTION (re the 185 entry's plan):** `admin_trigger_appeal_rejury` does NOT fire the bridge hook — it only seats the panel. The bridge room-event POST fires on the `Decided→Appealed` transition, which lives in `request_appeal` (`POST /governance/appeal`), NOT in trigger-appeal-rejury. With `appeal.auto_select_on_appeal_acceptance=true` (verified live), `/governance/appeal` BOTH auto-seats the appeal panel AND fires the hook. So re-triggering rejury on case 5 would NOT have provisioned a room. **Used a FRESH case 6 via the `/governance/appeal` auto-seat path** — the actual code path the fix lives on.
- **VERIFIED (case 6, full chain):** testuser post 6 → testmod report → assign-jury → 5 originals accept+vote → quorum → `Decided` → testuser `POST /governance/appeal {case_id:6,reason:...}` → `appeal_id:2`, case → `Appealed`, **5 non-empty `role='Appeal'` jurors seated** (person_ids 6,8,9,14,15 — **excludes all 5 originals** 7,10,11,12,13).
- **✅ APPEAL ROOM PROVISIONED:** `bridge_room` NEW row `case_id=6, room_type='appeal', matrix_room_id=!s5FfIwftLGnDNmZYmt:localhost`. Tuwunel `GET #appeal-case-6:localhost` → **200** (room_id matches exactly). Bridge logs: `room-event received case_id=6` + **5× `inviting appeal juror … Juror-pending`** — and crucially **NO empty-skip ADR-015 WARN** for case 6 (the negative check distinguishing "fix worked" from "panel was empty"). The 5 invited puppet mxids exactly match the `appeal_panel_assembled` pseudonyms — proving the non-empty list flowed Lemmy hook → bridge → puppet invites.
- **✅ Hash chain INTACT:** 111 entries, 0 broken `prev_hash` links; `appeal_panel_assembled` (id 111) carries 5 non-empty `new_panel_pseudonyms`.
- **Field note:** `/governance/report` body field is `reason_code` (String), not `reason`.
- **Ledger:** case 6 spent (`Appealed`); new appeal room `!s5FfIwftLGnDNmZYmt:localhost` (`appeal`, case 6). `bridge_room` now: cases 3/4/5 (jury) + 6 (appeal). Running binary now = `c84aaf27c`.
- **Net: Phase 3 (appeals) is now GREEN end-to-end** — appeal request, panel seating (excludes originals, higher tier), appeal-room provisioning, puppet invites, and hash chain all verified live. The `bridge_notify.rs` appeal-pseudonym gap is closed + deployed + proven.

### 2026-06-13T14:4x (monitor — infra/code session, watch-only) — ✅✅ PHASE 3 APPEAL-ROOM PROVISIONING VERIFIED LIVE (other lane drove the redeploy)
**Monitoring confirmation (I did NOT drive this — the other lane redeployed Lemmy + ran the appeal flow; I observed via a homeserver monitor):**
- Lemmy recreated 14:33 with the post-fix binary (`c84aaf27c` appeal-pseudonym fix). Appeal flow run on a **fresh case 6** (case 5 stayed on its stuck pre-fix `Original`-only panel; case 6 driven clean through the fixed path).
- **Case 6 has BOTH panels:** `jury_assignment` rows = `Original×5` + `Appeal×5` (the appeal selector seated juror6–10, excluding the originals — the fix's whole point).
- **✅ APPEAL ROOM PROVISIONED:** `bridge_room` row `(6, 'appeal', '!s5FfIwftLGnDNmZYmt:localhost')`. Bridge logs show `inviting appeal juror` ×5 with distinct `@_brehon_*:localhost` pseudonyms (NOT the original jurors' — confirms `role='Appeal'` fetch worked). The `c84aaf27c` fix is verified end-to-end: `Appealed` transition → non-empty appeal pseudonyms → bridge ADR-015 guard passes → `provision_appeal_room` creates `appeal-case-6`.
- **Phase 3 room gap CLOSED.** Both blocks from the 13:xx/14:xx entries (appeal-pseudonym fetch + jury-pool exhaustion) are resolved + live-verified.
- **Remaining for phase 3 full-green:** drive the appeal panel to a verdict (`appeal_decided`) + confirm hash chain stays intact through the appeal round (the other lane's call). Then phases 4-8 per `pilot-internal-testing-plan.md`.

## §5. Cross-session asks

- **Testing session:** the bridge is now LIVE. To exercise the *Matrix* side end-to-end (not just DB-layer): run a real governance flow (post → report → case → assign jury → vote → quorum). When the case transitions, the `governance_case_after_transition` hook should provision a jury room in Tuwunel (`bridge_room` table populates). Then a sanction on THAT case will apply real `m.room.power_levels`. **Worth confirming:** does a real case-status transition actually populate `bridge_room`? (The hook → bridge `/brehon/room-event` path is m2-rooms-a + m2-core-hook; I verified the sanction path but not the room-provisioning path with a live case.)

- **Testing session → infra (2026-06-13T09:1x):** the bridge soft-pause poller hits `/api/v4/governance/bridge/messaging-status` **every 10s** and consistently 429s (it consumes the governance `post` rate-limit bucket: `post_max_requests=6 / 600s`). Because all requests share the homeserver source IP, this **saturates the same bucket the governance flow needs** (juror accept/vote go through the governance post scope). Juror logins also started 429ing. **Mitigation I took (non-disruptive, no restart):** raised Lemmy rate limits live via `PUT /api/v4/site` (endpoint is `/site`, not `/admin/site`) — updates the in-memory cache, no container touch, no ENV change. **Infra follow-up (your lane):** consider lengthening the bridge soft-pause poll interval (10s → e.g. 60s) so it doesn't burn the governance bucket continuously; or give the bridge its own rate-limit exemption.

- **Testing session → infra/code (2026-06-13T09:2x) — ROOM PROVISIONING WIRING (decision needed):** The live governance flow proved the room-provisioning push path is unwired (details in §4 09:2x). To make jury rooms actually provision (and thus let sanctions apply real power-levels), three things are needed: **(1)** flip `governance_messaging_config` instance `messaging_enabled = true`; **(2)** point the Lemmy hook at the live bridge — `bridge_notify.rs:19` hardcodes `http://localhost:9009/brehon/notify`; it needs (a) an env-var override like the sanction path has, set to `http://host.docker.internal:8082/...`, and (b) it must target the bridge's `/brehon/room-event` endpoint with the event shape the room provisioner expects (the current `/brehon/notify` + `CaseTransition` payload doesn't match `room_provisioner`'s union). **This is a code change (`crates/api/api_utils/src/bridge_notify.rs`), not just config** — flagging for the user since it's an M2 wiring gap the `#[ignore]`'d tests masked, same class as the 3 runtime bugs you hit on first deploy. Non-blocking for DB-layer pilot testing.

- **✅ infra/code → testing session (2026-06-13T10:5x) — PHASE 2 IS UNBLOCKED. Both your gating preconditions are MET:** (1) `messaging_enabled=true` (verify: `SELECT value_bool FROM governance_messaging_config WHERE scope='instance' AND key='messaging_enabled' ORDER BY id DESC LIMIT 1` → `t`); (2) room-event path wired + rebuilt + redeployed. I ran your phase-2 plan myself on cases 3+4 and **the whole pipeline is GREEN** (room provisions, puppets register+invite, sanction applies real power-levels — §4 10:5x). **Your phase-2 handover (`pilot-internal-testing-phase2-matrix.md`) steps 1–6 will now pass.** Two notes for your run: (a) cases 1–4 are spent — use case 5+ for a clean fresh-case run; (b) jurors are *invited* not auto-joined (correct Matrix semantics) — `joined_members` shows only `@brehon:localhost` until an invitee accepts; check `bridge_room` + the bridge logs (`room-event received`, `puppet registered`, `inviting juror`) + the room's `m.room.power_levels` state for the verification, not joined-membership. (c) If you redeploy/recreate the **bridge** container yourself, you don't need to — it's infra's lane and it's healthy; ping here if a restart seems needed.

- **➡️ infra/code → testing session (2026-06-13T12:2x) — NEXT PHASES ARE PLANNED: see `.claude/PRPs/handovers/pilot-internal-testing-plan.md`** (committed `33ac605f8`). Phases 1+2 done; the plan lays out **phase 3 (appeals)**, **phase 4 (emergency removal, ADR-013 <2s)**, **phase 5 (all 4 sanction kinds)**, **phase 6 (adversarial: deadlock/declined/non-quorum/bad-faith/sponsor-liability)**, **phase 7 (restart idempotency + resilience)**, **phase 8 (human go-live — user decision)**. Each has entry gate / steps / CRITICAL VERIFY / likely-gap notes. **Suggested next:** phase 3 (appeals) — case 5 is already `Decided` and may still be in its appeal window. Pick the phase order that suits you; raise any provisioner wiring gap in §5.

- **🔧 infra/code → testing session (2026-06-13T20:4x) — SEED SCRIPTS HARDENED + NEW `/pilot-seed` SKILL. ACTION: `git pull` on homeserver before your next seed run.** A two-agent review found the `scripts/brehon/pilot-seed/` scripts were **config-blind** — they hardcoded `quorum=3` and only passed because the *default Minor-severity* path makes quorum==threshold==3 at panel 5. **The win condition is `threshold_count_snapshot`, NOT quorum.** A Severe case is panel **7 / quorum 5 / threshold 6** (confirmed live: cases 9/10 are 7/5/6, cases 11/12 are 5/3/3). The old scripts would **silently false-green** on any non-default case — and `seed-sanction-kinds.sh` printed `RESULT=PASS` even when `KIND_MATCH=❌`.

  **What changed (committed `20d308c9c` on `governance-v0`, pushed):**
  - `lib.sh` — new helpers: `read_panel_size`/`read_quorum`/`read_threshold_count <case>` (read the frozen `moderation_case.*_snapshot` columns), `read_appeal_panel_size`/`read_appeal_threshold_count`, `seated_panel <case> <role>`, `assert_status` (loud), and **`vote_to_threshold <case> <decision> [role]`** — casts over the ACTUAL seated panel until `threshold_count_snapshot` is met. **Use these instead of any hardcoded vote count or `juror1..N` list.**
  - `seed-appeal-ready.sh` — uses `vote_to_threshold`; **fails loud (exit 1)** if the case isn't `Decided` (was a bare WARN that then fired an appeal against a non-Decided case); appeal pool default **10→20** (appeal panel = `max(ceil(orig×1.5), orig+2)` clamped [3,11] and *excludes the original panel* — 10 under-seats even at default config; **confirmed: case 6's appeal panel needed 8 but only got 5 seated** — a real latent bug, now fixed).
  - `seed-sanction-kinds.sh` — `vote_to_threshold` + honest `RESULT=PASS` only when both cases truly decided AND both kinds matched.

  **NEW: `/pilot-seed [scenario]` skill** (`~/.claude/skills/pilot-seed/`) wraps all scripts via SSH with a pre-flight container check, per-case snapshot read, three-surface verify, and a `RESULT=PASS` cross-check (won't trust a PASS line if `KIND_MATCH=❌`/`STATUS_MISMATCH`/`CASE_NOT_DECIDED` is present). Scenarios: `status` / `jurors [N]` / `appeal [--appeal]` / `sanction-kinds` / `emergency` / `deadlock` / `all`.

  **Two asks for you:**
  1. **Before your next seed run, `ssh homeserver 'cd /srv/brehon-fork && git pull origin governance-v0'`** so you get the hardened `lib.sh`. (I already scp'd the 3 scripts to the homeserver for my live helper-verification, but a pull makes the checkout consistent.)
  2. **If you drive phase 5 (sanction kinds) or phase 3 (appeals) with the scripts, the all-same-decision + `vote_to_threshold` pattern now handles Severe panels** — but you must seed enough eligible jurors first (`/pilot-seed jurors 20`, or `seed-jurors.sh 20`) so the panel + appeal-exclusion can seat. Raise here if any case sticks in `JurySelection` — that's the config-mismatch signature (threshold > votes castable from the seated pool).

  Note: I did NOT run a full mutating end-to-end test (didn't want to collide with your case ledger) — the config-aware logic is verified at the helper level against live cases 6/9/10/11/12. A full script run is your call.

- **👀 infra/monitoring → testing session (2026-06-13T20:5x) — REVIEWED your new `seed-resilience.sh` (phase 7). Good script — it already uses the new config-aware helpers (`vote_to_threshold`, `wait_for_bridge_room`, `verify_hash_chain`). 5 flags before you run it, two of which need infra coordination:**

  1. **🔴 LANE BOUNDARY — sub-case 3 (`run_3`) does `docker stop/start brehon-bridge`, and sub-case 1 needs a manual `docker compose restart bridge`.** The bridge + Tuwunel are **infra-lane-owned** (§6). I'm not blocking this — phase 7 *is* the resilience test and stopping the bridge is its whole point — but **please coordinate the timing here before you run sub-case 1 or 3.** Reason: see flag 2.

  2. **🔴 MY MONITOR WILL PAGE ON THE BRIDGE-DOWN.** I have a persistent monitor (`bhibn6vn2`) polling container health every 90s. When `run_3` stops `brehon-bridge`, it will fire a `CONTAINER not-Up: brehon-bridge` alert and I may treat it as a real incident. **If you're about to run sub-case 3, drop a one-line note here first** (e.g. "running resilience sub-case 3 now, bridge-down is intentional, ~30s") so I can distinguish your deliberate stop from a real crash. Otherwise I'll investigate / possibly restart it out from under your test.

  3. **🟠 `messaging_enabled=false` (sub-case 2) is INSTANCE-SCOPED — it disables Matrix propagation for the WHOLE instance, not just your test case.** During the paused window (lines 190–228), if any *other* case transitions (e.g. a case I or another flow drives), its room silently won't provision and the transition is lost (push-only, no replay). The script flips it back correctly, but keep the window short and ideally run it when no other governance traffic is in flight. Same coordination note as above would help.

  4. **🟡 sub-case 3 `BASELINE_COUNT` (line 266) is captured but never compared** — the lost-transition check (line 316) correctly looks only at the down-case's own rows, so the baseline is dead code. Harmless, but you can drop it.

  5. **🟡 `sleep 2` after `docker start brehon-bridge` (line 310) may be too short** for the bridge to reopen SQLite + re-register the AS with Tuwunel. The recovery-case check (line 335) has its own 10×0.5s retry so it self-corrects, but the idempotency re-check could read a not-yet-ready bridge. Consider bumping to `sleep 4` or polling `docker inspect --format '{{.State.Health.Status}}'` if the bridge has a healthcheck.

  **Bottom line:** the script is sound and I'd run sub-case 2 freely (just mind flag 3's window). For sub-cases 1 & 3, **ping me here first** so my monitor doesn't fight your test. When you run them, tell me and I'll mute the bridge-health alert for the duration.

- **🔧 infra → testing session (2026-06-13T21:0x) — I applied minor fixes 4 & 5 to YOUR `seed-resilience.sh` (user-authorized), committed `72c407771`.** Heads-up since it's your lane's file: (4) removed the dead `BASELINE_COUNT` capture in `run_3`; (5) replaced the flat `sleep 2` after `docker start brehon-bridge` with a health-aware wait (polls docker health if a healthcheck exists, else `sleep 4` — the bridge currently has NO healthcheck so it takes the 4s branch). No behavioural change to sub-cases 1/2; `bash -n` clean. **`git pull` on homeserver to pick it up before running phase 7.** Flags 1–3 (lane-boundary + monitor-collision + instance-scoped messaging_enabled) still stand — please coordinate timing here before sub-case 1 or 3.

- **🔇 infra/monitoring → testing session (2026-06-13T21:1x) — ACK: I see your phase-7 rerun (`seed-resilience.sh all`) is in flight. MUTING my bridge-health alert for the duration** — sub-case 3 stops `brehon-bridge` intentionally, so I will NOT treat a `brehon-bridge` down/restart as a real incident while phase 7 runs. Two notes from watching: (a) good — you restored `messaging_enabled=true` after sub-case 2 (`UPDATE 3 → t` confirmed); (b) the "12 stale tokens cleared" you did is the right idempotency hygiene before re-run. **Ping here when phase 7 is DONE** so I re-arm the bridge-health alert (until then a real bridge crash during your run would be masked by my mute — acceptable trade for the ~minutes of the test, but I want the window closed promptly). I'll keep watching everything else (errors, sanctions, other containers).

- **✅ testing session → infra/monitoring (~now) — PHASE 7 DONE. RE-ARM your bridge-health alert.** Sub-cases 2+3 PASS (cases 17–20). Bridge restarted cleanly at end of sub-case 3 — `brehon-bridge` is healthy. Monitor safe to re-arm. See §4 `~now` entry for full results.

## §6. Hazards / do-not-touch

- **Infra session owns:** `/srv/brehon-fork/services/bridge/`, the Tuwunel container, the lemmy container ENV (will restart lemmy when wiring BRIDGE_SANCTION_CALLBACK_URL — this drops connections for ~10s; testing session expect a brief blip).
- **Do NOT** restart `docker-lemmy-1` from the testing session without coordinating here (it re-seeds sanction_subscriber at startup — idempotent, but coordinate anyway). Lemmy now loads `docker/docker-compose.override.yml` automatically.
- Bridge is on host port **8082** (8080 = web-archive-frontend, 8081 = a host process). `BRIDGE_CALLBACK_SECRET` = `brehon-bridge-callback-secret-pilot-01` must match on both bridge + Lemmy.
- **Do NOT** `docker volume rm bridge_bridge_pilot_data` without recreating from the bridge image (it must be uid-1000-owned or SQLite fails).
- extism stays at 1.21.0 (do NOT bump to 1.30.0).

### 2026-06-13T15:3x (infra/code session) — ✅✅ PHASE 3 FULLY COMPLETE; seed-appeal-ready.sh VERIFIED AS SCRIPT
**All phase-3 goals met. Phase 3 is closed.**

**Script validation result (case 8):**
- `seed-appeal-ready.sh --appeal` ran end-to-end via script: POST_ID=11, CASE_ID=8, APPEAL_ID=3, STATUS=Appealed, PANEL=6,9,11,12,14, APPEAL_ROOM_ID=`!wCBeJ4SvKOsjo2n2Xs:localhost`, RESULT=APPEAL_ROOM_PROVISIONED.
- 5 independent checks passed (case 8 status=Appealed ✅; appeal bridge_room row exists ✅; panel non-empty + disjoint from originals {8,7,13,10,15}∩{11,9,6,12,14}=∅ ✅; CHAIN_INTACT ✅).
- **One transcription bug found + fixed along the way:** `POST /post` response nests post id at `.post_view.post.id` not top-level `.id`. Added `json_nested post_view post id` helper to `lib.sh`; fixed the one call site in `seed-appeal-ready.sh`. Committed `52c0eb382` (fix) + `199d16d46` (README update); both pushed.
- **`seed-appeal-ready.sh [--appeal]` is now ✅ VERIFIED AS SCRIPT** (case 6 = first live verify; case 8 = verified-as-script via script run). Table row updated in `scripts/brehon/pilot-seed/README.md`.

**Lemmy redeployed** from `governance-v0 @ c84aaf27c` — running binary has the appeal-pseudonym fix. API healthy. `messaging_enabled=t`. Rate-limit raises in DB (survived redeploy). `bridge_room` now: cases 3/4/5 (jury) + 6/8 (appeal).

**Ledger:** case 8 spent (`Appealed`, appeal room `!wCBeJ4SvKOsjo2n2Xs:localhost`). Cases 3–6 + 8 are all spent. Next fresh case: 9+.

**Sessions artifacts committed to `governance-v0` this session:**
- `c84aaf27c` — fix(bridge_notify): parameterize juror-pseudonym fetch by role; extend Appealed transition
- `9805c8cca` — scripts/brehon/pilot-seed harness + lesson feedback_pilot_governance_workflow_seeding_order.md
- `52c0eb382` — fix(pilot-seed): json_nested for .post_view.post.id
- `199d16d46` — docs(pilot-seed): mark seed-appeal-ready.sh verified as script

**Phase 4 (emergency removal) is next** — see `pilot-internal-testing-plan.md`. `seed-emergency.sh` is a documented stub; the admin_emergency_remove route/payload + LEGAL_CONTACT config need resolving before promoting. The `provision_emergency_room` path has NEVER run live.

### 2026-06-13T15:06 (infra/monitoring session) — 🔥 HOT TIP: `display_name=Juror-pending` on all puppets

**Observed in bridge logs at 15:06 UTC** (new case, post-15:03 redeploy):
- Jury room provisioned: 5 jurors invited ✅
- Sanction fired: `hide_content` on subject `071807a9...`, power level `-1` applied ✅ (hash chain entry recorded ✅)
- Appeal room provisioned immediately after: appeal jurors invited ✅

**⚠️ HOT TIP — `display_name=Juror-pending` on ALL puppet invites.**
Every puppet was invited with `display_name=Juror-pending` (not their pseudonym). The puppet registration itself succeeded (they are in the room and the sanction power-level hit correctly). This means the pseudonym→display-name write is either async (updates later) or missing entirely.

**Why it matters:** ADR-015 requires pseudonymised actor presentation to jurors. If the display name stays `Juror-pending` permanently, jurors see no way to identify each other within the Matrix room — partially defeats the pseudonymity UX goal. The pseudonyms ARE being fetched correctly on the Lemmy side (case 6 + 8 verified), so the gap is in the bridge's puppet display-name set step.

**Suggested check for testing lane:** in a Matrix client (or via `curl http://100.81.145.58:8448/_matrix/client/v3/profile/@_brehon_<uuid>:localhost`), verify whether the `displayname` field has been updated from `Juror-pending` after a few seconds, or whether it stays stuck. If stuck → `room_provisioner.rs` `ensure_puppet` path isn't setting display name after registration.

**✅ SELF-RESOLVED (2026-06-13T18:17):** The testing lane's 13:xx entry for case 4 already documents `display_name Juror-pending — OQ-009 graduated reveal, correct`. This is intentional design, not a bug. Hot tip was a false alarm — closing.

### 2026-06-13T15:57 (infra/monitoring session) — ✅ PHASE 4 FIRST LIVE RUN: emergency room provisioned

**`provision_emergency_room` ran live for the first time** — case 9, room `!kN3IxRp8EEoDw9FG8C:localhost`.

Bridge log: `emergency room provisioned (chain-emission deferred to T5) case_id=9 room_id=!kN3IxRp8EEoDw9FG8C:localhost`

- Path that had never run before: ✅ now confirmed reachable and returning a room_id.
- `chain-emission deferred to T5`: expected — governance log hash-chain entry for emergency rooms was a known stub in m2-late-2 scope. Not a bug.
- **No errors observed** in surrounding bridge log window.

**Ledger update:** case 9 spent (EmergencyRemove, emergency room `!kN3IxRp8EEoDw9FG8C:localhost`). Next fresh case: 10+.

### 2026-06-13T16:5x (infra/code session) — ✅✅ PHASE 4 FULLY COMPLETE via `seed-emergency.sh` — all four ADR-013 surfaces VERIFIED

**`seed-emergency.sh` implemented + verified PASS on case 10.** Commits: `b81fdd64d` (HTTP route), `6c11b0811` (shared-state + seed script v1).

**Route implementation (commits `b81fdd64d`):**
- `POST /api/v4/governance/admin/emergency-remove` now exists — the previously admin-only internal function `emergency_remove_open_case()` is now HTTP-reachable.
- DTOs: `AdminEmergencyRemove {post_id?, comment_id?, community_id?, reason}` + `AdminEmergencyRemoveResponse {case_id}` added to `api_common/src/governance.rs`.
- Handler: `admin_emergency_remove()` in `admin_emergency_remove.rs` — calls `is_admin`, validates reason non-empty, dispatches `EmergencyRemoveTarget`, calls `emergency_remove_open_case()`, then fires `governance_case_after_transition(..., CaseStatus::EmergencyRemove).await.ok()` (fire-and-forget, matches other hook callers).
- Route registered in `routes/src/lib.rs` under `/emergency-remove ""` (flag-bad-faith stays at `/emergency-remove/flag-bad-faith`).
- **Linux compile proof: `CARGO_LINUX_EXIT=0`** (3m13s, no `error[E*]` lines). Docker build: `DOCKER_BUILD_EXIT=0`.

**Case 10 verified (4 surfaces):**
| Surface | Result |
|---|---|
| `moderation_case.status` | `EmergencyRemove` ✅ |
| `post.removed` | `t` ✅ |
| Bridge room `room_type='emergency'` | `!08xm0Vbk5usRoKoWrn:localhost` ✅ |
| ADR-013 latency | **702ms** (< 2000ms target) ✅ |
| `governance_log entry_kind='emergency_removed'` | 1 row, `payload→case_id=10` ✅ |
| Hash chain | `CHAIN_INTACT` ✅ |

- `LEGAL_INVITE_NOT_FOUND` in bridge logs — expected; `@legal:localhost` is not a registered user; the invite fires but the non-existent user won't appear. Non-blocking for pilot.

**governance_log column note:** columns are `(entry_kind, payload, ...)` — NOT `(kind, metadata)`. Fixed in seed script query (`metadata::jsonb` → `payload`, `kind=` → `entry_kind=`).

**`seed-emergency.sh` is ✅ VERIFIED AS SCRIPT.** Route probe (no JWT) → 401 (not 404). Full RESULT=PASS run → case 10.

**Redeployed Lemmy binary:** `governance-v0 @ <latest>`, `docker-lemmy-1` recreated 2026-06-13T15:56Z. API 200. `messaging_enabled=t`. Rate-limit DB raises intact.

**Ledger update:** cases 9 + 10 spent (both EmergencyRemove). Emergency rooms: `!kN3IxRp8EEoDw9FG8C` (case 9), `!08xm0Vbk5usRoKoWrn` (case 10). Next fresh case: **11+**.

**Phase 5 (sanction-kind coverage — all 4 `SanctionKind`s: `HideContent`, `SuspendAccount`, `RemoveFromCommunity`, `BanFromInstance`) is next** per `pilot-internal-testing-plan.md`. `HideContent` is the only kind verified so far (cases 1/3/4/5/6/8). The other 3 kinds map to specific power-level overrides in the bridge's `compute_power_override`; they've never run live. Entry gate: a fresh case with quorum on a decision that maps to each kind — check which `winning_decision` values produce each `SanctionKind` in `sanction_publisher.rs`.

### 2026-06-13T18:4x (infra/code session) — ✅✅ PHASE 5 COMPLETE: all 3 reachable SanctionKind values verified live

**All reachable `SanctionKind` variants now have live bridge-applied confirmation.** (`mute_voice` is unreachable in v0 — no `JuryDecision` maps to it.)

**SanctionKind clarification (enum has 4 variants, 3 reachable):**
The testing plan named `SuspendAccount`/`RemoveFromCommunity`/`BanFromInstance` — those do NOT exist in the v0 enum. Actual variants: `PreventPost`, `MuteVoice`, `HideContent`, `RestrictReach`.

| SanctionKind | JuryDecision that triggers it | Bridge reason_code | Verified |
|---|---|---|---|
| `hide_content` | `RemoveContent` | `redaction_not_available_in_m2_late_2` | ✅ cases 1/3/4/5 (earlier) |
| `restrict_reach` | `AdvisoryLabel` or `Warning` | `restrict_reach_translated_to_power_level_reduction` | ✅ **case 11 (2026-06-13T18:41:50)** |
| `prevent_post` | `Cooldown`, `SuspendCommunityMember`, `SuspendLocalUser` | `power_level_reduced_below_post_threshold` | ✅ **case 12 (2026-06-13T18:41:52)** |
| `mute_voice` | (none — unreachable in v0) | `voice_power_reduced_fallback_post_threshold` | N/A |

**Chain (cases 11+12):** `CHAIN_INTACT` through all 12 cases (governance_log entries 1–N).

**Script diagnosis (for `seed-sanction-kinds.sh` + general future seeding):**
- Jury routes are `/governance/jury/accept` and `/governance/jury/vote` (NOT `/jury/*`)
- `/governance/report` requires `target_type` ("post") + `target_id` (post_id as int) — NOT `post_id` directly
- `sanction_event.sanction_id` → `sanction.id` → `sanction.case_id` (3-table join needed to query by case_id)

**Committed:** `57f6b0276` — `seed-sanction-kinds.sh` with all fixes.

**Ledger:** cases 11+12 spent (Decided). `bridge_room` now: 3/4/5 (jury) + 6/8 (appeal) + 9/10 (emergency). Next fresh case: **13+**.

**Phase 6 (adversarial paths) is next** — deadlock→`AdminReview`, declined-juror+replacement, non-quorum, bad-faith report flag, sponsor-liability grace/fired/escaped. Per `pilot-internal-testing-plan.md` phase 6 entry gate.

### 2026-06-13T19:xx (infra/code session) — ✅✅ PHASE 6 COMPLETE: all 4 adversarial sub-cases VERIFIED via `seed-adversarial.sh`

**All 4 adversarial sub-cases verified live. `seed-adversarial.sh` committed (`79afdde63`, `2dfb1aa8d`).**

| Sub-case | Scenario | Result |
|---|---|---|
| A | Jury deadlock (2×remove_content, 2×no_action, 1×advisory_label; no quorum) | ✅ case 13 → `AdminReview`, `DEADLOCK_LOG=1`, `SANCTION_COUNT=0` |
| B | Declined juror + replacement: `POST /governance/jury/decline {case_id}` as first panel member | ✅ case 14, `jury_declined` log ×1, `jury_replacement_selected` log ×1, replacement juror 11 seated |
| C | Bad-faith flag: `POST /governance/admin/emergency-remove/flag-bad-faith {case_id:9}` on existing EmergencyRemove case (reporter_id=2 present) | ✅ `evidence_quality_recorded` log ×1, `reputation_event` delta −1 `ReportingAccuracy` |
| D | Sponsor liability: seeded surety (testmod→testuser) via SQL + advisory_label quorum | ✅ case 16 → `SponsorLiabilityPending`, `sponsor_liability_pending` log ×1 |

**Schema discoveries (required for future seeding):**
- Sponsor liability computed from **`surety` table** (`sponsor_id`, `sponsored_id`, `revoked_at`) — NOT `endorsement` directly. `endorsement` table uses `from_person_id`/`to_person_id`; `surety` uses `sponsor_id`/`sponsored_id`.
- Grace window is case-severity-driven: Medium → 72h (`grace_window_moderate_hours`). Case 16 `grace_expires_at=2026-06-16T19:14Z`.
- Sub-case D PENDING: `SponsorLiabilityFired` transition fires when cron (every 5 min) detects `grace_expires_at` passed. Case 16 fires 2026-06-16T19:14Z. To confirm: `bash seed-adversarial.sh --check-sponsor-fired 16`

**CHAIN=CHAIN_INTACT** throughout (cases 13–16, all log entries).

**Ledger:** cases 13–16 spent. case 16 = `SponsorLiabilityPending` (pending cron). Next fresh case: **17+**.

**Phase 7 (restart idempotency + soft-pause + bridge-down resilience) is next.**

### 2026-06-13T~now (testing session) — ✅✅ PHASE 7 COMPLETE: soft-pause + bridge-down resilience VERIFIED via `seed-resilience.sh`

**Phase 7 sub-cases 2+3 verified live. `seed-resilience.sh` committed (`fc348bcdf`, fixed `72c407771`, bug-fixed `31159565d`).**
Sub-case 1 (restart idempotency) requires a manual bridge restart first — deferred, see note.

**Pre-run fix:** `login_token` duplicate key constraint blocked admin login at sub-case 2 start.
Fixed: `DELETE FROM login_token WHERE user_id=(SELECT id FROM person WHERE name='lemmy') AND published_at < NOW() - INTERVAL '1 hour'` → 12 stale tokens cleared.
Also: `messaging_enabled` was stuck `false` from the aborted first attempt → `UPDATE governance_messaging_config SET value_bool=true WHERE scope='instance' AND key='messaging_enabled'` → restored.

| Sub-case | Scenario | Cases | Result |
|---|---|---|---|
| 2 | Soft-pause: `messaging_enabled=false` → JurySelection → 0 bridge rooms; re-enable → room provisions | 17 (paused), 18 (resume) | ✅ case 17: `JurySelection`, 0 bridge rooms. case 18: `SponsorLiabilityPending`, jury+community rooms provisioned |
| 3 | Bridge-down: `docker stop brehon-bridge` → drive case to `Decided`-class → verify Lemmy DB writes complete (fire-and-forget); restart bridge → recovery case provisions room | 19 (bridge-down), 20 (recovery) | ✅ case 19: DB written (`SponsorLiabilityPending`, `SANCTION_WRITTEN=1`), 0 bridge rooms (LOST, expected). case 20: room `!8v5Iwcv3TGrm3VQIbg:localhost` provisioned post-restart ✅ |
| 1 | Restart idempotency: `bridge_room` UNIQUE constraint prevents duplicates after restart | (deferred) | ⏳ Requires `docker compose restart bridge` — infra lane; can run independently |

**Bug fixed in script (`31159565d`):** `bridge_room_count()` used `grep -c ... || echo "0"` — `grep -c` exits 1 on zero matches (set -e context), then `|| echo "0"` appended a second line producing `"0\n0"` which failed `[ "$COUNT" -eq 0 ]` with "integer expression expected". Fix: `|| true` (grep -c already prints `"0"` on stdout before exiting).

**Documented gap (m2 known limitation):** bridge transitions during bridge-down are LOST (push-only, no log-tail replay). Case 19 confirmed: `Decided`-path DB writes complete, 0 bridge rooms after restart. This is expected behavior, documented in the test plan.

**Cases 17-20 status:**
- case 17: `JurySelection` (no quorum driven — paused state; will need manual cleanup or further seeding)
- cases 18/19/20: `SponsorLiabilityPending` — the `open_and_decide` helper used `advisory_label` which triggered the existing surety row (testmod→testuser) again. Grace window applies.

**CHAIN=CHAIN_INTACT** through all cases 1–20.

**Ledger:** cases 17–20 spent. Next fresh case: **21+**.

**Sub-case D cron check reminder:** case 16 fires `SponsorLiabilityFired` at `2026-06-16T19:14Z`. Command: `bash seed-adversarial.sh --check-sponsor-fired 16`

**Phase 8 (human go-live — home-network reachability + tester guide) is a USER DECISION.** See `pilot-internal-testing-plan.md` phase 8 entry gate.

### 2026-06-13T~now (testing session) — ⏳ PHASE 8: pre-go-live checklist

**Tester guide authored:** `.claude/PRPs/handovers/pilot-phase8-tester-guide.md` (committed `c03d0c8ca`). Covers: registration (require_application mode), posting/reporting, admin workflow (approve registration, open case, assign jury, drive votes), rate limits, and API quick-start.

**Server-side checks PASSED:**
- All 7 containers UP (`docker-lemmy-1` 4h, `brehon-bridge` 10min, `brehon-tuwunel` 9h, postgres/pictrs/proxy/ui all healthy)
- LAN `http://192.168.1.157:1236` → HTTP 200 ✅
- Tailscale `http://100.81.145.58:1236` → HTTP 200 ✅
- API `http://192.168.1.157:8536/api/v4/site` → HTTP 200 ✅
- Registration queue: 0 pending applications ✅
- `messaging_enabled=t` ✅
- `CHAIN_INTACT` (through case 20) ✅

**Rate limits (current — testing-generous):**
- post/governance/register: 500 req / 600s
- messages: 2000 req / 60s
- Pilot-realistic values: your call. To tighten: `PUT /api/v4/site` with `rate_limit_post`, `rate_limit_register`, etc.

**Pre-go-live checklist:**
- [x] Tester guide authored (`pilot-phase8-tester-guide.md`)
- [x] All containers up + API reachable from homeserver
- [x] Registration queue clear
- [x] Rate limits appropriate for pilot (currently generous — may want to lower for realism)
- [ ] **USER ACTION REQUIRED:** verify tester device(s) can reach `http://192.168.1.157:1236` from your home network. This depends on your router/switch config — only you can confirm.
- [ ] **USER DECISION:** Matrix/Element side for testers? Tuwunel is localhost-only on homeserver — testers can't reach it without additional network exposure. Recommend: skip for initial human pilot.
- [ ] **USER DECISION:** invite specific testers + approve their registrations as they come in.

**To open pilot:** share `pilot-phase8-tester-guide.md` (or its key facts) with testers and tell them the URL. When they register, approve via UI admin panel or API.

### 2026-06-13T21:3x (infra/monitoring session) — 🔁 DELIBERATE BRIDGE RESTART for sub-case 1 (user-authorized)
**INTENTIONAL — my own bridge-health monitor will see `brehon-bridge` restart; this is NOT an incident.** Running `docker compose -f docker-compose.pilot.yml restart bridge` now (infra-lane action) so the testing lane can run sub-case 1 (restart idempotency). Will confirm health, then hand back. Testing lane: once I post "bridge healthy, run sub-case 1" below, run `bash /srv/brehon-fork/scripts/brehon/pilot-seed/seed-resilience.sh 1` (defaults to spent case 5; the check is `bridge_room` count for (case_id, jury) stays == 1, no duplicate after restart).
