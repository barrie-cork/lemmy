---
phase: pilot-internal (testing session — phase 2: Matrix-side verification)
plan: "(none — ops/testing track)"
phase_branch: "(none — testing work, no code commits from this session)"
lane_mode: B
worktree: C:/Users/barri/Developer/brehon-fork
authored: 2026-06-13T10:3x
authored_by: testing session (canonical brehon-fork / governance-v0)
purpose: Resume brief for the SECOND testing phase — verifying the Matrix side of the governance flow ONCE room provisioning is wired. Phase 1 (DB+API+bridge-receipt) is DONE and verified.
---

# ⏩ RESUME — read this block first

**You are the TESTING session for Brehon pilot-internal, phase 2 (Matrix-side verification).** This is the cross-session two-advisor setup: you are the testing session; the OTHER session is infra/bridge.

**FIRST ACTION every turn:** read `.claude/PRPs/handovers/pilot-internal-SHARED-STATE.md` top-to-bottom. The most-recent §4 entry is authoritative. APPEND your entries; never rewrite theirs. Cross-session requests go in §5. Respect §6 hazards (do NOT restart `docker-lemmy-1`, do NOT touch the bridge/Tuwunel containers or `services/bridge/` — that's infra's lane).

**Surface-first lane line** (required at session start, per advisor-orchestrator surface-first ritual):
`lanes: C:/Users/barri/Developer/brehon-fork:governance-v0 active (TESTING session, Mode B); other-active: infra/bridge session (no separate worktree — shared canonical)`

## What is already DONE (phase 1 — do NOT re-run)

The full governance flow is **verified end-to-end at the DB + API + bridge-receipt layer** (SHARED-STATE §4 entry 09:2x). Specifically, on **case 1**:
- post (post_id=1, testuser) → report (testmod) → case opened → admin assign-jury → case `JurySelection`, 5-juror panel (persons 6–10) → 5× accept → 3× vote `remove_content` → quorum → case `Decided`.
- `sanction_event` row written (`HideContent`, pseudonymized subject, hash-linked). Bridge **received** the sanction POST, authenticated it, returned `200 applied:false reason:"no rooms found for case_id=1"` (correct — no room existed).
- Governance hash chain: 24 entries, cryptographically intact (genesis → unbroken prev_hash linkage).

**Case 1 is spent (status `Decided`).** Phase 2 needs a FRESH case (case 2+) so the room-provisioning hook fires on a NEW `JurySelection` transition.

## ⛔ GATING PRECONDITION — phase 2 cannot start until BOTH are true

Phase 2 is **blocked** until the infra/code session wires room provisioning. Verify before doing anything:

1. **`messaging_enabled = true`** — check:
   ```bash
   ssh homeserver "docker exec docker-postgres-1 psql -U lemmy lemmy -t -A -c \"SELECT value_bool FROM governance_messaging_config WHERE scope='instance' AND key='messaging_enabled' ORDER BY valid_from DESC LIMIT 1;\""
   ```
   Must return `t`. If `f` → STILL BLOCKED; post a note in SHARED-STATE §5 asking infra for status; do NOT proceed.

2. **The Lemmy→bridge room-event path is wired.** The gap (SHARED-STATE §4 09:2x + §5 09:2x): `crates/api/api_utils/src/bridge_notify.rs:19` hardcodes `http://localhost:9009/brehon/notify` and sends a `CaseTransition` payload, but the bridge's room provisioner reads `/brehon/room-event` with a different event union. Infra must (a) add an env-var override pointing at `http://host.docker.internal:8082/brehon/room-event`, (b) match the payload shape, AND (c) rebuild+redeploy the Lemmy binary. Confirm the infra session has logged this as DONE in SHARED-STATE §4 before starting. **A redeploy of `docker-lemmy-1` is infra's action — do not trigger it yourself.**

If either precondition is unmet: write a one-line §5 ask, set a long ScheduleWakeup (1800s) to re-check, and STOP. Do not attempt the Matrix flow against unwired infra.

## Phase 2 test plan (run ONLY after the gate clears)

Goal: prove a live case transition **provisions a Matrix jury room** and a subsequent sanction **applies real `m.room.power_levels`**.

All actor JWTs are stored on homeserver at `/tmp/brehon-smoke/<user>.jwt` (testuser, testmod, juror1–5). They may have expired (JWTs ~7-day exp, but re-login is cheap). Re-login any actor with:
```bash
ssh homeserver 'curl -s -X POST http://localhost:8536/api/v4/account/auth/login -H "Content-Type: application/json" -d "{\"username_or_email\":\"juror1\",\"password\":\"testpass123\"}"'
```
Admin: `lemmy`/`lemmylemmy`. Creds reference: PMD `reference_pilot_test_accounts.md`.

**Steps:**
1. **Baseline** `bridge_room` count (expect 0 or prior rows):
   ```bash
   ssh homeserver "docker cp brehon-bridge:/data/bridge.db /tmp/br.db && python3 -c \"import sqlite3;print(sqlite3.connect('/tmp/br.db').execute('SELECT count(*) FROM bridge_room').fetchone()[0])\""
   ```
2. **New post** (testuser) in community 2 → **report** (testmod) → capture new `case_id` (will be 2+).
3. **admin assign-jury** on the new case. This is the trigger: case → `JurySelection` fires `governance_case_after_transition` → (now-wired) POST to bridge `/brehon/room-event`.
4. **CRITICAL VERIFY — room provisioned:**
   - `bridge_room` count incremented; row has `case_id=<new>`, `room_type` (jury), non-null `matrix_room_id`.
   - Bridge logs show a room-event received + `create_community_room` success:
     `ssh homeserver "docker logs brehon-bridge --since 3m 2>&1 | grep -iE 'room.event|provision|create_community_room|room_alias'"`
   - The room exists in Tuwunel (query via bridge AS or Tuwunel admin API on `:8448`).
5. **Drive to quorum** (5× accept, 3× vote) → case `Decided` → sanction fires.
6. **CRITICAL VERIFY — power-levels applied:** bridge sanction response should now be `applied:true` with `rooms_found>=1 / applied>=1`. Confirm the jury room's `m.room.power_levels` state event reflects the `SanctionKind` change (query room state in Tuwunel). This is the m2-late-2 deliverable — the whole point of phase 2.
7. **Record** the full result (room_id, power-level before/after, applied counts) in SHARED-STATE §4 with a 10:xx testing-session entry.

## Rate-limit context (already mitigated)

Lemmy rate limits were raised live this session via `PUT /api/v4/site` (NOT `/admin/site`): `register`→500/600s, `post`→500/600s, `message`→2000/60s, `comment`→500/600s. **Login shares the `register` bucket; governance accept/vote use the `post` bucket** (non-obvious — PMD `reference_lemmy_rate_limit_buckets_pilot.md`). If 429s recur (e.g. lemmy was redeployed and reset to defaults 10/3600 + 6/600), re-raise via the same endpoint with admin JWT. The bridge soft-pause poller hits `/governance/bridge/messaging-status` every 10s and shares the IP bucket — infra may have lengthened its interval (check §4).

## Key paths / facts

- Pilot API: `http://100.81.145.58:8536` (Tailscale) / `http://192.168.1.157:8536` (LAN). UI on `:1236`.
- Bridge: host port **8082** → container 8080. `BRIDGE_CALLBACK_SECRET=brehon-bridge-callback-secret-pilot-01`.
- Tuwunel (Matrix): host `:8448` → container 8008, `server_name=localhost`, federation OFF.
- Governance routes: `POST /api/v4/governance/report`, `/admin/assign-jury`, `/jury/accept`, `/jury/vote`. Enums serialize snake_case (`target_type:"post"`, `decision:"remove_content"`).
- jury config: panel_size=5, quorum=3, threshold=ceil(5×0.5001)=3. Small-pool fallback ON (no reputation snapshots needed).
- Postgres exec: `docker exec docker-postgres-1 psql -U lemmy lemmy`. Bridge SQLite: copy out via `docker cp brehon-bridge:/data/bridge.db` then read with python3 sqlite3 (no `sqlite3` CLI in container).
- DEBUG logging is ON in lemmy — `docker logs docker-lemmy-1` is very noisy; grep tightly.

## What NOT to do (phase 2)

- Do NOT re-run phase 1 (case 1 is done; its result is recorded).
- Do NOT restart/recreate `docker-lemmy-1`, `brehon-bridge`, or `brehon-tuwunel` — all infra's lane (SHARED-STATE §6).
- Do NOT edit `crates/**` or `services/bridge/**` — the room-event wiring fix is infra/code's job, not the testing session's.
- Do NOT `docker volume rm bridge_bridge_pilot_data` (must stay uid-1000-owned).
- Do NOT start M3 PRD work — pilot testing is the current track.
- Do NOT bump extism past 1.21.0.

## Git state at handoff

- governance-v0 HEAD: `8ac81ea90` (no commits from testing session — testing makes no code changes).
- Working tree (canonical): `M .claude/runlog/bm-runlog.md`, untracked `brehon-fork-phase-dq/`, `brehon-fork-validate-665/` (residual worktrees, not blocking), `.claude/PRPs/reports/session-retro-2026-06-12-memory-prune.md`. None are this session's; leave them.
- SHARED-STATE.md has uncommitted testing-session appends (§4 09:2x, §5 entries). It is shared coordination state — the infra session also writes it. Per multi-lane discipline, commit only if coordinated; otherwise leave for the live cross-session file exchange.
- DQ pending: 0.
- PMD lessons written this session: `reference_pilot_test_accounts.md`, `reference_lemmy_rate_limit_buckets_pilot.md` (both indexed in MEMORY.md).
