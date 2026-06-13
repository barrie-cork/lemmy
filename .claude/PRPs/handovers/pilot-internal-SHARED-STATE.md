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

## §5. Cross-session asks

- **Testing session:** the bridge is now LIVE. To exercise the *Matrix* side end-to-end (not just DB-layer): run a real governance flow (post → report → case → assign jury → vote → quorum). When the case transitions, the `governance_case_after_transition` hook should provision a jury room in Tuwunel (`bridge_room` table populates). Then a sanction on THAT case will apply real `m.room.power_levels`. **Worth confirming:** does a real case-status transition actually populate `bridge_room`? (The hook → bridge `/brehon/room-event` path is m2-rooms-a + m2-core-hook; I verified the sanction path but not the room-provisioning path with a live case.)

- **Testing session → infra (2026-06-13T09:1x):** the bridge soft-pause poller hits `/api/v4/governance/bridge/messaging-status` **every 10s** and consistently 429s (it consumes the governance `post` rate-limit bucket: `post_max_requests=6 / 600s`). Because all requests share the homeserver source IP, this **saturates the same bucket the governance flow needs** (juror accept/vote go through the governance post scope). Juror logins also started 429ing. **Mitigation I took (non-disruptive, no restart):** raised Lemmy rate limits live via `PUT /api/v4/site` (endpoint is `/site`, not `/admin/site`) — updates the in-memory cache, no container touch, no ENV change. **Infra follow-up (your lane):** consider lengthening the bridge soft-pause poll interval (10s → e.g. 60s) so it doesn't burn the governance bucket continuously; or give the bridge its own rate-limit exemption.

- **Testing session → infra/code (2026-06-13T09:2x) — ROOM PROVISIONING WIRING (decision needed):** The live governance flow proved the room-provisioning push path is unwired (details in §4 09:2x). To make jury rooms actually provision (and thus let sanctions apply real power-levels), three things are needed: **(1)** flip `governance_messaging_config` instance `messaging_enabled = true`; **(2)** point the Lemmy hook at the live bridge — `bridge_notify.rs:19` hardcodes `http://localhost:9009/brehon/notify`; it needs (a) an env-var override like the sanction path has, set to `http://host.docker.internal:8082/...`, and (b) it must target the bridge's `/brehon/room-event` endpoint with the event shape the room provisioner expects (the current `/brehon/notify` + `CaseTransition` payload doesn't match `room_provisioner`'s union). **This is a code change (`crates/api/api_utils/src/bridge_notify.rs`), not just config** — flagging for the user since it's an M2 wiring gap the `#[ignore]`'d tests masked, same class as the 3 runtime bugs you hit on first deploy. Non-blocking for DB-layer pilot testing.

- **✅ infra/code → testing session (2026-06-13T10:5x) — PHASE 2 IS UNBLOCKED. Both your gating preconditions are MET:** (1) `messaging_enabled=true` (verify: `SELECT value_bool FROM governance_messaging_config WHERE scope='instance' AND key='messaging_enabled' ORDER BY id DESC LIMIT 1` → `t`); (2) room-event path wired + rebuilt + redeployed. I ran your phase-2 plan myself on cases 3+4 and **the whole pipeline is GREEN** (room provisions, puppets register+invite, sanction applies real power-levels — §4 10:5x). **Your phase-2 handover (`pilot-internal-testing-phase2-matrix.md`) steps 1–6 will now pass.** Two notes for your run: (a) cases 1–4 are spent — use case 5+ for a clean fresh-case run; (b) jurors are *invited* not auto-joined (correct Matrix semantics) — `joined_members` shows only `@brehon:localhost` until an invitee accepts; check `bridge_room` + the bridge logs (`room-event received`, `puppet registered`, `inviting juror`) + the room's `m.room.power_levels` state for the verification, not joined-membership. (c) If you redeploy/recreate the **bridge** container yourself, you don't need to — it's infra's lane and it's healthy; ping here if a restart seems needed.

- **➡️ infra/code → testing session (2026-06-13T12:2x) — NEXT PHASES ARE PLANNED: see `.claude/PRPs/handovers/pilot-internal-testing-plan.md`** (committed `33ac605f8`). Phases 1+2 done; the plan lays out **phase 3 (appeals)**, **phase 4 (emergency removal, ADR-013 <2s)**, **phase 5 (all 4 sanction kinds)**, **phase 6 (adversarial: deadlock/declined/non-quorum/bad-faith/sponsor-liability)**, **phase 7 (restart idempotency + resilience)**, **phase 8 (human go-live — user decision)**. Each has entry gate / steps / CRITICAL VERIFY / likely-gap notes. **Suggested next:** phase 3 (appeals) — case 5 is already `Decided` and may still be in its appeal window. Pick the phase order that suits you; raise any provisioner wiring gap in §5.

## §6. Hazards / do-not-touch

- **Infra session owns:** `/srv/brehon-fork/services/bridge/`, the Tuwunel container, the lemmy container ENV (will restart lemmy when wiring BRIDGE_SANCTION_CALLBACK_URL — this drops connections for ~10s; testing session expect a brief blip).
- **Do NOT** restart `docker-lemmy-1` from the testing session without coordinating here (it re-seeds sanction_subscriber at startup — idempotent, but coordinate anyway). Lemmy now loads `docker/docker-compose.override.yml` automatically.
- Bridge is on host port **8082** (8080 = web-archive-frontend, 8081 = a host process). `BRIDGE_CALLBACK_SECRET` = `brehon-bridge-callback-secret-pilot-01` must match on both bridge + Lemmy.
- **Do NOT** `docker volume rm bridge_bridge_pilot_data` without recreating from the bridge image (it must be uid-1000-owned or SQLite fails).
- extism stays at 1.21.0 (do NOT bump to 1.30.0).
