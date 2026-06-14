# pilot-internal — SHARED STATE (cross-session coordination)

> **Purpose:** this file coordinates concurrent advisor sessions during pilot-internal:
> - **infra/bridge session:** Matrix homeserver + Brehon bridge + Lemmy↔bridge wiring.
> - **testing session:** exercises governance flows via the seeded accounts/community.
> - **validate/merge lane:** m2-late-b-actor (B-actor) validation + PR (added 2026-06-13, §5 last entry).
>
> **Read this at the top of every turn.** Most-recent entry in §4/§5 is authoritative.
> Both sessions append; neither rewrites the other's entries. Cross-session requests go under §5.
>
> **Pruned 2026-06-13 (advisor):** §4 phase-1–7 play-by-play collapsed to the results table below
> (all phases verified ✅; full narrative in git history `pilot-internal-SHARED-STATE.md@e1677fcee`).
> §5 asks + §6 hazards preserved verbatim — they carry LIVE obligations.

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

**Pilot stack files (committed to `governance-v0`):** `services/bridge/docker-compose.pilot.yml` (Tuwunel+bridge), `services/bridge/tuwunel-pilot.toml`, `services/bridge/Dockerfile`, `docker/docker-compose.override.yml` (Lemmy bridge env).
**Bring up:** Tuwunel+bridge → `cd /srv/brehon-fork/services/bridge && docker compose -f docker-compose.pilot.yml up -d`; Lemmy (auto-loads override) → `cd /srv/brehon-fork/docker && docker compose up -d`.

## §2. Test accounts + community (per `reference_pilot_test_accounts.md`)

- Admin: `lemmy` / `lemmylemmy`
- `testmod` (person 4) — approved, login verified
- `testuser` (person 5) — approved, login verified
- Community: `test_governance` (id=2) — public, open posting
- API: `http://100.81.145.58:8536/api/v4/...`

## §3. What's safe to test NOW (DB-layer, no bridge needed)

✅ The testing session can proceed with these immediately (do NOT depend on the bridge):
- User registration / login / posting / commenting in `test_governance`
- Flagging/reporting posts; admin case creation (ModerationCase); juror assignment (admin_assign_jury)
- Jury voting + quorum → sanction row + governance_log entries written to DB

✅ **Bridge is wired (2026-06-13T08:5x)** — Matrix propagation is LIVE. A sanction reaching quorum: (1) writes sanction row + governance_log entry (DB), (2) Lemmy POSTs the sanction event to the bridge, (3) bridge looks up the case's provisioned rooms + applies `m.room.power_levels` per `SanctionKind`. Jury-room *provisioning* (m2-rooms-a) requires the `governance_case_after_transition` hook to fire — a sanction on a case with no provisioned room returns `applied:false, reason:"no rooms found"` (correct, not an error).

## §4. Live status — PHASES 1–7 COMPLETE (results table; play-by-play in git history)

**Pipeline VERIFIED end-to-end:** governance flow → DB writes → bridge → Matrix room provisioning → `m.room.power_levels`. `CHAIN_INTACT` throughout all cases 1–20.

| Phase | Scope | Verified | Cases (spent) |
|---|---|---|---|
| 1+2 | Full governance→Matrix pipeline (room provision, puppet register+invite, sanction power-levels) | ✅✅ live + independent confirm | 1–5 |
| 3 | Appeals — `seed-appeal-ready.sh [--appeal]` verified as script; appeal-room provisioning fix (`c84aaf27c`) | ✅✅ | 6, 8 (appeal rooms provisioned, panel disjoint from originals) |
| 4 | Emergency removal (ADR-013) — `seed-emergency.sh`, all 4 surfaces; **702ms < 2000ms target** | ✅✅ | 9, 10 (EmergencyRemove) |
| 5 | Sanction kinds — 3 reachable `SanctionKind`s (`restrict_reach`, `prevent_post`, + HideContent from earlier) | ✅✅ | 11, 12 |
| 6 | Adversarial — A deadlock→`AdminReview`; B declined-juror+replacement; D sponsor-liability→`SponsorLiabilityPending` | ✅✅ via `seed-adversarial.sh` | 13, 14, 16 |
| 7 | Resilience — soft-pause (`messaging_enabled=false`), bridge-down (DB writes complete, rooms lost-expected), restart idempotency (`bridge_room` UNIQUE survives restart, count stays 1) | ✅✅ via `seed-resilience.sh` + direct verify | 17–20 |

**Ledger: cases 1–20 spent. Next fresh case: 21+.**

**⏰ LIVE CRON REMINDER (sub-case D):** case 16 fires `SponsorLiabilityFired` at **2026-06-16T19:14Z** (Medium severity → 72h grace, `grace_expires_at=2026-06-16T19:14Z`, cron every 5 min). To confirm after that time: `bash seed-adversarial.sh --check-sponsor-fired 16`. Sponsor liability is computed from the **`surety` table** (`sponsor_id`/`sponsored_id`/`revoked_at`), NOT `endorsement`.

**Documented m2 limitation (not a bug):** bridge transitions during bridge-down are LOST (push-only, no log-tail replay). Case 19 confirmed `Decided`-path DB writes complete with 0 bridge rooms after restart. Also: `chain-emission deferred to T5` for emergency rooms is a known m2-late-2 stub. `display_name=Juror-pending` on puppets is intentional (OQ-009 graduated reveal), not a defect.

### 2026-06-13 — ⏳ PHASE 8: pre-go-live checklist (USER DECISIONS OPEN)

**Tester guide:** `.claude/PRPs/handovers/pilot-phase8-tester-guide.md` (committed `c03d0c8ca`) — registration (require_application mode), posting/reporting, admin workflow, rate limits, API quick-start.

**Server-side checks PASSED:** all 7 containers UP; LAN `http://192.168.1.157:1236` → 200; Tailscale `http://100.81.145.58:1236` → 200; API → 200; registration queue clear; `messaging_enabled=t`; `CHAIN_INTACT` (through case 20).

**Rate limits (current — testing-generous):** post/governance/register 500/600s; messages 2000/60s. Pilot-realistic values are your call (`PUT /api/v4/site`).

**Pre-go-live checklist:**
- [x] Tester guide authored · [x] containers up + API reachable · [x] registration queue clear · [x] rate limits set (generous)
- [ ] **USER ACTION:** verify tester device(s) can reach `http://192.168.1.157:1236` from your home network (router/switch config — only you can confirm).
- [ ] **USER DECISION:** Matrix/Element for testers? Tuwunel is localhost-only on homeserver — testers can't reach it without extra network exposure. Recommend: skip for initial human pilot.
- [ ] **USER DECISION:** invite specific testers + approve their registrations as they arrive.

**To open pilot:** share `pilot-phase8-tester-guide.md` with testers + the URL; approve registrations via UI admin panel or API.

## §5. Cross-session asks

- **Testing session:** the bridge is now LIVE. To exercise the *Matrix* side end-to-end (not just DB-layer): run a real governance flow (post → report → case → assign jury → vote → quorum). When the case transitions, the `governance_case_after_transition` hook should provision a jury room in Tuwunel (`bridge_room` table populates). Then a sanction on THAT case will apply real `m.room.power_levels`. **Worth confirming:** does a real case-status transition actually populate `bridge_room`? (The hook → bridge `/brehon/room-event` path is m2-rooms-a + m2-core-hook; I verified the sanction path but not the room-provisioning path with a live case.)

- **Testing session → infra (2026-06-13T09:1x):** the bridge soft-pause poller hits `/api/v4/governance/bridge/messaging-status` **every 10s** and consistently 429s (it consumes the governance `post` rate-limit bucket: `post_max_requests=6 / 600s`). Because all requests share the homeserver source IP, this **saturates the same bucket the governance flow needs** (juror accept/vote go through the governance post scope). Juror logins also started 429ing. **Mitigation I took (non-disruptive, no restart):** raised Lemmy rate limits live via `PUT /api/v4/site` (endpoint is `/site`, not `/admin/site`) — updates the in-memory cache, no container touch, no ENV change. **Infra follow-up (your lane):** consider lengthening the bridge soft-pause poll interval (10s → e.g. 60s) so it doesn't burn the governance bucket continuously; or give the bridge its own rate-limit exemption.

- **Testing session → infra/code (2026-06-13T09:2x) — ROOM PROVISIONING WIRING (resolved 10:5x):** flagged that the room-provisioning push path was unwired (`bridge_notify.rs:19` hardcoded `http://localhost:9009/brehon/notify`). **RESOLVED** — room-event path wired + rebuilt + redeployed; pipeline GREEN on cases 3+4. Retained for the record: the fix was a code change to `crates/api/api_utils/src/bridge_notify.rs`, same class as the 3 runtime bugs hit on first deploy that the `#[ignore]`'d tests masked.

- **🔧 infra/code → testing session (2026-06-13T20:4x) — SEED SCRIPTS HARDENED + `/pilot-seed` SKILL. ACTION: `git pull` on homeserver before your next seed run.** The `scripts/brehon/pilot-seed/` scripts were **config-blind** (hardcoded `quorum=3`; the win condition is `threshold_count_snapshot`, NOT quorum — a Severe case is panel 7 / quorum 5 / threshold 6). Fixed in `lib.sh` (new `read_panel_size`/`read_quorum`/`read_threshold_count`/`vote_to_threshold` helpers — use these instead of any hardcoded vote count), `seed-appeal-ready.sh` (fails loud if not `Decided`; appeal pool 10→20), `seed-sanction-kinds.sh` (honest PASS). Committed `20d308c9c`. **NEW `/pilot-seed [scenario]` skill** (`~/.claude/skills/pilot-seed/`) — SSH wrapper with pre-flight check + three-surface verify + PASS cross-check. **Asks:** (1) `ssh homeserver 'cd /srv/brehon-fork && git pull origin governance-v0'` before next seed; (2) seed enough eligible jurors (`/pilot-seed jurors 20`) so Severe panels + appeal-exclusion can seat — a case stuck in `JurySelection` is the config-mismatch signature.

- **✅ PHASE 7 resilience coordination (2026-06-13T20:5x–21:5x) — CLOSED.** Lane-boundary (sub-case 3 stops `brehon-bridge`, infra-owned) + monitor-collision (infra's 90s health monitor `bhibn6vn2`) + instance-scoped `messaging_enabled=false` were coordinated live; infra muted then re-armed the bridge-health alert. Minor script fixes 4+5 applied (`72c407771`). All sub-cases PASS (cases 17–20). Bridge healthy post-restart. **No open action.**

- **🆕 advisor → validate/merge lane (2026-06-13T~now) — m2-late-b-actor MERGE GATES. Your cargo-check approach is correct; THREE gates remain before `bm-pr` opens. Do NOT open the PR after only the cargo check + DQ mutation.**
  1. **DQ mutation must land on the PHASE branch tip `c1cb20227` (`origin/phase-m2-late-b-actor`), not the detached worker commit `49aaeebbc`.** The validate worktree is detached at `49aaeebbc` (one commit short of the daemon-merge tip). The cargo result is valid (identical task-13 code), but write `result:pass` + `answered_by:"advisor-laptop"` onto the entry on `c1cb20227` and push that, or the resolved entry won't be on the branch the PR is cut from. (Advisor already pushed daemon→origin: `bb71b7c06..c1cb20227`.)
  2. **`bm-pr` STOPS without a `validate-pending-laptop-linux` entry at `result:pass`.** The diff touches `migrations/2026-06-13-…_add_actor_app_link/{up,down}.sql`, `services/bridge/Cargo.toml`, and 6 bridge `.rs` files → the diff-scoped Linux-compile gate is **triggered** (`feedback_linux_compile_proof_is_a_gate.md`). Raise + run `./scripts/brehon/cargo-linux.sh check --workspace --features full` (Docker `rust:1.95`), mutate to `pass`. This is a HARD gate, not optional.
  3. **Run the 5 e2e tests (Level 2) — cargo-check only proves the test FILE compiles, not that the tests pass.** `actor_app_link.rs`: dual-sig, revoke, nonce, bad-sig, pseudonym. Also Level 4 (migration round-trip) per plan §6/§7. **AND merge-forward FIRST:** `governance-v0` is 50+ commits ahead of the phase branch (whole pilot lane landed after it branched) — merge `governance-v0` into `phase-m2-late-b-actor` before `bm-pr` or the PR diff will be huge + conflict-prone. Plan acceptance §6: `.claude/PRPs/plans/m2-late-b-actor.plan.md`.
  - **§16a stories:** the plan predates §16a-story-retrofit (plan Notes line). Only needed if `/brehon-verify` requires them — advisor will retrofit from the 5 acceptance criteria at verify time if so. Not a blocker for your validation pass.

## §6. Hazards / do-not-touch

- **Infra session owns:** `/srv/brehon-fork/services/bridge/`, the Tuwunel container, the lemmy container ENV (will restart lemmy when wiring BRIDGE_SANCTION_CALLBACK_URL — this drops connections for ~10s; testing session expect a brief blip).
- **Do NOT** restart `docker-lemmy-1` from the testing session without coordinating here (it re-seeds sanction_subscriber at startup — idempotent, but coordinate anyway). Lemmy now loads `docker/docker-compose.override.yml` automatically.
- Bridge is on host port **8082** (8080 = web-archive-frontend, 8081 = a host process). `BRIDGE_CALLBACK_SECRET` = `brehon-bridge-callback-secret-pilot-01` must match on both bridge + Lemmy.
- **Do NOT** `docker volume rm bridge_bridge_pilot_data` without recreating from the bridge image (it must be uid-1000-owned or SQLite fails).
- extism stays at 1.21.0 (do NOT bump to 1.30.0).
