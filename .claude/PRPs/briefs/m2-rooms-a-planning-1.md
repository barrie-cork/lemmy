# Planning Brief — m2-rooms-a: Bridge-Side Room Provisioning + Hash-Chain Emission

**Phase:** m2-rooms-a
**Branch:** phase-m2-rooms-a (cut from governance-v0 @ f7a9c9c12 at bm-cut)
**Authored:** 2026-06-06
**Authored by:** advisor (canonical brehon-fork / governance-v0 session)
**PRD:** `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` §"Implementation Phases" (Phases 3–5)
**Plan target:** `.claude/PRPs/plans/m2-rooms-a.plan.md`

---

## 1. What this phase delivers (M2-core Phases 3–5)

m2-rooms-a is the bridge-side continuation of M2-core. m2-core-hook (PR #184, merged) shipped:
- The `governance_case_after_transition` notification function in `crates/api/api_utils/src/bridge_notify.rs` (fire-and-forget POST with `BridgeNotifyPayload::CaseTransition`)
- 11 transition-site wiring calls in `crates/api/api/src/governance/*.rs`
- 10 `ENTRY_KIND_ROOM_*` consts in `crates/db_schema/src/source/governance/governance_log.rs:239-248`
- The `append_room_event(pool, kind, payload, actor_pseudonym)` typed wrapper in `crates/api/api/src/governance/governance_log.rs:118` + `RoomEventPayload` struct at line 88
- e2e tests covering the above (Tasks 6–8 in m2-core-hook plan)

m2-rooms-a delivers (bridge-side, **out-of-workspace**):

**Phase 3: Bridge room provisioning service**
- A new `services/bridge/src/room_provisioner.rs` module: receives `CaseTransitionEvent` payloads from the Brehon binary's HTTP notification, provisions/archives Matrix rooms for C2.1–C2.6 scenarios (jury / community-event / spin-out / appeal / emergency / membership-mirror), pins `always_pseudonym` for jury+appeal rooms, renders jurors as `Juror-<suffix>` via OQ-009 graduated-reveal logic, tracks state in a new `bridge_room` table (bridge-local SQLite store — **not** Brehon workspace; zero workspace migrations). Idempotent by `case_id`+`room_type`: `Room::Created` fires exactly once per `case_id` even on bridge restart.
- The bridge HTTP router gains a new `POST /brehon/room-event` endpoint (or extends `/brehon/notify` dispatch) that accepts `CaseTransitionEvent` and calls into the provisioner.

**Phase 4: Binary-side HTTP endpoint exposing append_room_event**
- A new `POST /governance/room-event` Lemmy route (or `/brehon/room-event` on a separate bridge-callback port) that accepts a `RoomEventPayload` + `entry_kind` and calls `append_room_event(pool, kind, payload, actor_pseudonym)`. **This HTTP endpoint does not yet exist** — `append_room_event` is only called from e2e tests directly; the bridge needs an HTTP surface to call back into it.
- The bridge `room_provisioner.rs` calls this HTTP endpoint (the Brehon binary's `brehon_read_url` or a new `brehon_room_event_url` env var) to write `Room::*` entries onto the hash chain after each provisioning action.
- Restart-idempotency: bridge records `last_seen_governance_log_row_id` per `case_id` in `bridge_room` table; on restart, replays pending provisioning from the last-seen row id without duplicating `Room::Created`.

**Phase 5: Integration tests + clean-posture**
- Integration tests in `services/bridge/tests/` (testcontainers — bridge-local test infra, NOT the Brehon e2e suite):
  - Jury room provisions in <5s with exactly the 5 assigned jurors as `Juror-<suffix>` puppets, no reporter/reported/admin, `always_pseudonym` pin enforced.
  - `emergency_remove` → emergency room in <2s with admins + legal-contact MXID, reported party absent.
  - 10 `Room::*` entries land on the hash chain with correct schema after one full case lifecycle.
  - Bridge restart mid-case → no duplicate `Room::Created` (restart-idempotency acceptance test).
- Clean-posture: `messaging_enabled=false` → zero room provisioning, zero `Room::*` entries (asserted in the binary's e2e test `m2_append_room_event_writes_chain_entry` already ships; bridge integration test adds a messaging-disabled smoke variant).

**Deliverable boundary:** only `services/bridge/` + 1 new Lemmy HTTP route (binary-side HTTP endpoint for bridge callback, in `crates/api/routes/src/lib.rs` + a handler file in `crates/api/api/src/governance/`). No workspace migration. No changes to `crates/db_schema/migrations/**`.

---

## 2. Key file anchors (planner must verify these before authoring tasks)

| File | Anchor | Purpose |
|---|---|---|
| `crates/api/api_utils/src/bridge_notify.rs:48-78` | `governance_case_after_transition` fn | Binary-side notification (already shipped). Bridge receives at `/brehon/notify` POST. Payload: `BridgeNotifyPayload::CaseTransition(CaseTransitionEvent)`. |
| `crates/api/api_utils/src/bridge_notify.rs:13` | `BRIDGE_NOTIFY_URL` = `http://localhost:9009/brehon/notify` | Confirms the notify endpoint the bridge exposes. The room-event callback goes the other way (bridge→binary). |
| `crates/api/api/src/governance/governance_log.rs:88-130` | `RoomEventPayload` struct + `append_room_event` fn | Binary-side hash-chain writer. Bridge calls this via the new HTTP route (Phase 4). Signature: `append_room_event(pool, kind, &str, payload: RoomEventPayload, actor_pseudonym: Option<String>) -> LemmyResult<GovernanceLog>`. |
| `crates/db_schema/src/source/governance/governance_log.rs:239-248` | 10 `ENTRY_KIND_ROOM_*` consts | Already shipped. Bridge crate references as string literals (no cross-crate dep on `db_schema`). |
| `services/bridge/src/appservice.rs` | `AppState` struct + `router()` fn | Bridge HTTP server structure; new provisioner endpoint wired here. `AppState.relay_enabled` gates provisioning (same as DM relay). |
| `services/bridge/src/provision.rs` | `create_community_room` fn | M1 stub: creates a Matrix community room. Phase 3 extends this with room-type-specific provisioning (jury, appeal, emergency, etc.) + idempotency. |
| `services/bridge/src/puppet.rs` | `PuppetMap::ensure_puppet` | Bridge-local puppet map for MXID lookup/creation. Phase 3 provisioner uses this to resolve juror `Juror-<suffix>` pseudonyms to Matrix puppet accounts. |
| `services/bridge/src/soft_pause.rs` | `run_poller` / `poll_once` | Pattern for bridge-side async polling. Provisioner may use similar tokio::interval for retry-on-failure. |
| `services/bridge/src/config.rs` | `BridgeConfig` struct | Must be extended with: `brehon_room_event_url: String` (the binary's `POST /governance/room-event` endpoint) + `legal_contact_mxid: String` (for emergency rooms). |
| `crates/api/routes/src/lib.rs:467` | Governance route block comment | Pattern for adding new governance HTTP routes; new `POST /governance/room-event` lands here. |
| `lemmy_api_common::governance::BridgeNotifyPayload` | `CaseTransition(CaseTransitionEvent)` variant | The payload the binary sends and the bridge receives. Bridge must deserialize this in Phase 3's handler. |

**No workspace migrations.** The `bridge_room` table lives in a bridge-local SQLite file (NOT Diesel/Postgres). Bridge crate uses `rusqlite` (already an M1 dep? — **planner to verify** by reading `services/bridge/Cargo.toml`). If SQLite dep is absent, planner should add `rusqlite` or `sqlx` (SQLite feature) to `services/bridge/Cargo.toml`.

---

## 3. Watchpoints for the planner

**WP-1 (Phase 4 — bridge callback HTTP route):** `append_room_event` has no HTTP surface yet. The plan MUST include a task that: (a) adds a `POST /governance/room-event` handler in `crates/api/api/src/governance/` accepting `{ entry_kind: String, payload: RoomEventPayload }` + hs_token authentication (bridge-only endpoint); (b) registers the route in `crates/api/routes/src/lib.rs`; (c) adds `brehon_room_event_url` to `BridgeConfig` so the bridge knows the callback URL. The `hs_token` auth on this Lemmy-binary endpoint should be a shared secret from `.env` (same pattern as bridge's AS token). Planner must spec the auth mechanism before dispatching this task.

**WP-2 (Phase 3 — `bridge_room` table store choice):** If `services/bridge/Cargo.toml` has no SQLite dep, the planner must choose the store for `bridge_room`: (a) `rusqlite` embedded file, (b) `sqlx` with SQLite feature, or (c) a simple in-memory `HashMap<case_id, BridgeRoom>` (sufficient for Phase 3 if restart-idempotency is handled by replaying from `last_seen_governance_log_row_id` on startup via the binary's API). Option (c) defers the SQLite dep and simplifies Phase 3; options (a)/(b) fully satisfy the restart-idempotency criterion in Phase 5. **Planner decides; document in plan §4 watchpoints.**

**WP-3 (Phase 3 — OQ-009 graduated reveal):** The PRD spec is: `Juror-<suffix>` revealed to fellow jurors only when they enter a room where ≥1 posted comment exists (admin-configurable threshold, default 1). The reveal check is bridge-side (bridge can observe Matrix room event count). Implementation: bridge queries the room event count via the Matrix client API (`GET /_matrix/client/v3/rooms/{roomId}/messages?limit=1`) before rendering room membership; if count ≥ threshold, use `Juror-<suffix>`; otherwise use an opaque `Juror-pending` or blank display name. The threshold is read from `governance_messaging_config` KV table (scope=`jury_rooms`, key=`oq009_reveal_threshold`, default 1) via the bridge polling the Brehon read URL. **Planner must spec the threshold lookup call shape** (a new poll at room-join time, or a cached value from the soft_pause poller).

**WP-4 (Phase 5 — integration test infrastructure) — RESOLVED (DQ a3d0e9941441-053):** Follow the existing `#[ignore]` + docker-compose pattern established in `services/bridge/tests/dm_round_trip.rs`. Phase 5 tests use `#[ignore = "requires docker-compose stack"]` and run manually with `docker compose up`. Timing assertions (<5s / <2s) are meaningful only against a real Tuwunel homeserver. CI-runnable bridge tests are M3 scope. Planner should add Phase 5 test stubs in `services/bridge/tests/room_provisioning.rs` with the `#[ignore]` marker and complete implementations referencing the docker-compose stack.

**WP-5 (Phase 3 — `always_pseudonym` enforcement):** The M1 identity-policy validator (`crates/api/api/src/governance/messaging_config.rs:69-83`) rejects non-pseudonymous `identity_policy` on `jury*`/`appeal*` scopes at the binary level. For bridge-side enforcement, the provisioner must refuse to create a jury/appeal room without the `always_pseudonym` config row being present (check `governance_messaging_config` KV at room-creation time). If the row is absent, the provisioner should log + skip provisioning (non-fatal — bridge-down failure mode per ADR-012 fire-and-forget). **Planner to verify that `GovernanceMessagingConfig::read_current` is callable over HTTP** or that the bridge has an equivalent read path (the `brehon_read_url` endpoint may not expose the full KV table; a new read route may be needed).

**WP-6 (Zero-Matrix-deps-in-workspace gate):** After every bridge-side task, the plan DoD MUST include:
```bash
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'
```
Expected: `0`. This gate fires even if the planner only adds `rusqlite` to `services/bridge/Cargo.toml` (SQLite is workspace-excluded; grep should still return 0). Planner must include this check in Task 0's audit + in every bridge-side task DoD.

---

## 4. Scope constraints (stop-and-ask tripwires for the planner)

- **STOP if:** any task adds `services/bridge` to the workspace `members` array in root `Cargo.toml`. It MUST stay in `exclude`. This breaks the zero-Matrix-deps invariant (M1 story-6).
- **STOP if:** any task adds a workspace migration under `crates/db_schema/migrations/**`. m2-rooms-a has zero workspace migrations; `bridge_room` table lives bridge-local only.
- **STOP if:** the `append_room_event` HTTP callback endpoint requires non-trivial binary-side Diesel schema changes. The endpoint is a thin HTTP wrapper over the existing `append_room_event` fn — no new columns or tables in the Brehon workspace.
- **STOP if:** Phase 3 provisioning logic requires calling the Matrix client synchronously from within the HTTP notification handler (the `POST /brehon/notify` response must return immediately — provisioning is async and must not block the notification ACK). Planner must verify the provisioner is dispatched via `tokio::spawn` (fire-and-forget from the handler thread).
- **Do NOT apply** Lemmy-workspace lessons (`feedback_lemmy_error_no_std_error.md`, Diesel lessons, `--features full`, `e2e.rs` edit discipline) to `services/bridge/**` tasks. Wrong toolchain (R8 from bootstrap §5). Apply bridge-context patterns instead.

---

## 5. DoD gates (cargo + integration)

Per `feedback_plan_dod_dry_run_at_write.md` — these must be executable as written:

**Workspace-level gate (all tasks):**
```bash
cargo check --workspace --features full
```
Expected: exit 0. Validates zero-Matrix-deps invariant holds.

**Bridge-level gate (all bridge tasks):**
```bash
cd services/bridge && cargo check
```
Expected: exit 0. validate-pending-laptop DQ command is `["cd services/bridge && cargo check"]` (pre-Shape-G; bridge toolchain is workspace-excluded per R8).

**Zero-Matrix-deps gate (all bridge tasks):**
```bash
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'
```
Expected: `0`.

**Phase 5 integration test gate:**
```bash
cd services/bridge && cargo test --test integration
```
Expected: exit 0 with all §Success Criteria assertions passing.

**validate-pending-laptop DQ entry shape for bridge tasks:**
```json
{
  "kind": "validate-pending-laptop",
  "commands": ["cd services/bridge && cargo check"],
  "branch": "<phase-branch>",
  "phase_task": <N>
}
```
For workspace-touching tasks (new Lemmy route), add `"./scripts/brehon/cargo-check.sh --workspace --features full"` as a second command.

---

## 6. Lesson injections (mandatory, per advisor-orchestrator §2.4)

**Mandatory for ALL bridge tasks:**
- `feedback_validate_pending_laptop_write_then_stop.md` — workers write DQ validate-pending-laptop and STOP; do NOT run cargo on the daemon.
- `feedback_cherry_pick_onto_restructured_file_reinjects_content.md` — carry-forward from m2-core-hook (cherry-pick caution on restructured files).

**Mandatory for Phase 4 Lemmy route task:**
- `feedback_features_full_workspace_only.md` — `--features full` is workspace-scope; per-crate `-p <crate> --features full` is invalid on `lemmy_server` etc.
- `feedback_multi_write_handlers_need_transactions.md` — the route handler writes only via `append_room_event` (single write); no transaction needed. But confirm the `append` fn wraps its own transaction (it does — governance_log.rs:301).

**Mandatory for Phase 5 integration test task:**
- `feedback_async_pool_test_pattern.md` — if integration tests need a PG connection, follow `AsyncPgConnection::establish` pattern. If using testcontainers for Brehon DB, the pattern applies.

**Do NOT inject** into bridge tasks: `feedback_lemmy_error_no_std_error.md`, `feedback_lemmy_migration_runner.md`, `feedback_postgres_jsonb_canonicalization.md`, `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — these are Lemmy-workspace-specific, not applicable to `services/bridge/**` (R8).

---

## 7. Plan structure guidance

Suggested §13 task breakdown — planner should refine based on WP-2 (store choice) and WP-4 (test infra choice):

| Task # | Deliverable | Primary files | `[P]`? |
|---|---|---|---|
| T0 | Audit — enumerate bridge deps; verify `services/bridge/Cargo.toml`; verify `brehon_read_url` endpoint; run zero-Matrix-deps gate | N/A | No |
| T1 | `BridgeConfig` + env var extensions (`brehon_room_event_url`, `legal_contact_mxid`); `bridge_room` store (WP-2 decision) | `services/bridge/src/config.rs`, `services/bridge/src/bridge_room.rs` (new) | No — blocks T2 |
| T2 | `room_provisioner.rs` module — C2.1 jury room provisioning (core path: JurySelection transition → Matrix createRoom → member invite loop with `Juror-<suffix>` puppets) + idempotency key | `services/bridge/src/room_provisioner.rs` (new), `services/bridge/src/appservice.rs` (route wiring) | No — depends T1 |
| T3 | Full C2.2–C2.6 room scenarios (community-event, spin-out, appeal, emergency, membership-mirror) + OQ-009 graduated-reveal + `always_pseudonym` enforcement | `services/bridge/src/room_provisioner.rs` (extend) | No — depends T2 |
| T4 | Binary-side `POST /governance/room-event` Lemmy route + hs_token auth + `append_room_event` call | `crates/api/api/src/governance/room_event_handler.rs` (new), `crates/api/routes/src/lib.rs` | No — depends T1 (needs `brehon_room_event_url` in config) |
| T5 | Bridge callback — provisioner calls binary `POST /governance/room-event` after each provisioning action; `Room::*` entries land on chain; restart-idempotency via `last_seen_governance_log_row_id` | `services/bridge/src/room_provisioner.rs` (extend) | No — depends T2, T4 |
| T6 | Integration tests — jury <5s, emergency <2s, 10-entry lifecycle, restart-no-dup, messaging-disabled clean | `services/bridge/tests/integration.rs` (new) | No — depends T3, T5 |

**Pre-Shape-G; all cargo runs bridge-local (`cd services/bridge && cargo check`). workspace-check at every task boundary.**

Planner is free to adjust task split based on WP-2 and WP-4 decisions. Task 0 MUST run `cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'` and confirm `0` before any bridge-side edits.

---

## 8. Not in scope for m2-rooms-a

- **B-publish sanction propagation / B-actor portable-ID linkage** — M2-late, gated on OQ-ADR016-02/-04.
- **Recording, transcript, M3 LiveKit/MatrixRTC** — `ENTRY_KIND_ROOM_RECORDING_UPLOADED` is registered but NOT emitted in M2.
- **Cross-instance jury rooms beyond Matrix federation** — OQ-V2-08 resolved: standard Matrix federation for cross-instance; no AP governance signals.
- **Any change to `CaseStatus` enum or governance handler decision logic** — m2-rooms-a is observer-only.
- **Tuwunel configuration changes** — pilot server Tuwunel is already running; bridge just uses it.
- **doc-04 drift fix** — doc `04-data-model-and-api.md` doesn't reflect `governance_messaging_config` table (M1-b); flagged but deferred to a chore commit after m2-rooms-a merges.

---

## 9. Pre-queue checklist (advisor to run before dispatching planning Junior)

- [x] `/brehon-clarify` run on this brief — DQ a3d0e9941441-051 (WP-1 auth, **pending user**), DQ a3d0e9941441-052 (WP-2 store, **pending user**), DQ a3d0e9941441-053 (WP-4 test infra, resolved by advisor). WP-3 gates on WP-1 resolution. Planning NOT queueable until 051+052 resolved.
- [ ] Verify: `grep -n "bridge" C:/Users/barri/Developer/brehon-fork/services/bridge/Cargo.toml` — confirm whether `rusqlite` or `sqlx-sqlite` is already a dep.
- [ ] Verify: `grep -n "room.event\|room_event" C:/Users/barri/Developer/brehon-fork/crates/api/routes/src/lib.rs` — confirm no `/governance/room-event` route exists yet.
- [ ] `mcp__junior-brehon__list_hooks` — confirm Telegram completion hook is registered and active before dispatch.
- [ ] `git -C C:/Users/barri/Developer/brehon-fork show governance-v0:.claude/PRPs/briefs/m2-rooms-a-planning-1.md` — must succeed (brief committed before dispatch).
- [ ] `/precheck` — mandatory before any Junior dispatch.

---

_Authored by advisor. Read-only reference for planning Junior. Do not modify during the planning run._
