# Plan: m2-rooms-a — Bridge-Side Room Provisioning + Hash-Chain Emission

## 1. Summary

m2-rooms-a is the bridge-side continuation of M2-core. The in-binary slice
(`m2-core-hook`, PR #184) already shipped the notification fire
(`governance_case_after_transition`), the 10 `ENTRY_KIND_ROOM_*` consts, the
`append_room_event` typed hash-chain writer, and the `RoomEventPayload`
struct. This sub-phase delivers the **consumer** of that notification: a
`services/bridge/` provisioning service that receives `CaseTransitionEvent`
payloads, provisions/archives Matrix rooms for the C2.1–C2.6 governance
scenarios, renders jurors as `Juror-<suffix>` pseudonyms under OQ-009
graduated reveal, persists room lifecycle state in a bridge-local rusqlite
store, and writes `Room::*` entries onto the Brehon hash chain via a **new
binary-side HTTP callback route**. It also fixes a latent M1 soft-pause
401 bug by giving the bridge its own service-principal read path.

**Headline acceptance condition:** after one full case lifecycle against a
docker-compose Tuwunel stack, exactly the 5 assigned jurors appear in a
jury room as `Juror-<suffix>` puppets (no reporter/reported/admin), the 10
`Room::*` taxonomy entries land on the hash chain with correct schema, a
mid-case bridge restart produces **no** duplicate `Room::Created`, and with
`messaging_enabled=false` zero rooms are provisioned and zero `Room::*`
entries are written. The zero-Matrix-deps-in-workspace invariant
(`cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` == 0) holds after
every task.

## 2. Source

- `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` §"Implementation
  Phases" (Phases 3–5), §"Success Criteria", §"Decisions Log" — the PRD this
  sub-phase implements.
- `.claude/PRPs/briefs/m2-rooms-a-planning-1.md` @ `governance-v0` — the
  advisor brief (all 4 clarify-DQ resolved: DQ `a3d0e9941441-051`/`-052`/
  `-053`/`-054`).
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` — predecessor/sibling
  (the in-binary slice; PR #184). The `append_room_event` wrapper +
  `RoomEventPayload` + 10 room consts this plan consumes were shipped there.
- `.claude/rules/governance-log-entry-kind-registry.md` §"M2 room kinds (10,
  m2-core-hook)" — the 10 `ENTRY_KIND_ROOM_*` consts + total count **65**.
- Lessons (binding):
  - `feedback_validate_pending_laptop_write_then_stop.md` — workers write the
    `validate-pending-laptop` DQ entry and STOP; cargo runs on the laptop
    advisor, never the daemon (every task in §13).
  - `feedback_cherry_pick_onto_restructured_file_reinjects_content.md` —
    carry-forward from m2-core-hook; re-author across restructure boundaries,
    never cherry-pick.
  - `feedback_features_full_workspace_only.md` — `--features full` is
    workspace-scope only; `-p lemmy_server --features full` is invalid
    (T4a/T4b workspace gates).
  - `feedback_multi_write_handlers_need_transactions.md` — confirms the T4a
    callback handler is single-write (`append_room_event` wraps its own
    transaction at `governance_log.rs`) so no extra `run_transaction`.
  - `feedback_async_pool_test_pattern.md` — for any Phase-5 test that opens a
    Brehon PG connection.
- ADRs: ADR-004 (plane separation — bridge owns its own store), ADR-012
  (fire-and-forget governance hook, bridge-down is non-fatal), ADR-013
  (emergency-remove room <2s), ADR-015 (always_pseudonym, `Juror-<suffix>`),
  ADR-016 line 282 (B-side backplane seam — service-to-service auth is
  orthogonal to the end-user separate-login decision). OQ-009 (graduated
  reveal at ≥1 comment, default threshold 1).

## 3. Problem statement

After m2-core-hook, the Brehon binary **emits** a fire-and-forget
`CaseTransition` notification to `http://localhost:9009/brehon/notify` and
**owns** a typed hash-chain writer (`append_room_event`), but:

1. **No bridge consumer exists for governance transitions.** The bridge's
   `/brehon/notify` path handles inbound Matrix DMs (relay), not governance
   case transitions. Nothing provisions a Matrix room when a case reaches
   jury selection, emergency-remove, appeal, etc. → §13 T2, T3.
2. **`append_room_event` has no HTTP surface.** It is only callable from
   in-process e2e tests. The bridge (a separate process) cannot write
   `Room::*` chain entries without a callback route. → §13 T4a.
3. **No durable bridge-side room state.** Restart-idempotency
   (`Room::Created` fires exactly once per `case_id` even across a bridge
   restart) requires a persistent store; `services/bridge/Cargo.toml` has no
   SQLite dependency. → §13 T1, T5.
4. **Latent M1 soft-pause 401 bug.** `soft_pause.rs:20` polls the
   `is_admin`-gated `admin_get_messaging_config` with a bare unauthenticated
   `GET`; against a live binary this returns 401 → `unwrap_or(false)` → the
   bridge silently never relays. The M1 `soft_pause_enable_disable_cycle`
   test is `todo!()`, so it was never caught. → §13 T4b, T5, T6.
5. **No graduated-reveal logic.** OQ-009 requires jurors to be revealed to
   each other only once a room has ≥1 posted comment (admin-configurable
   threshold, default 1). The bridge needs a service-principal read path to
   fetch the threshold. → §13 T3, T4b.

## 4. Solution statement

Two processes, one new HTTP seam in each direction:

```
Brehon binary (workspace, actix-web)            services/bridge (excluded, axum)
─────────────────────────────────────           ────────────────────────────────
governance_case_after_transition  ──POST /brehon/notify──▶  room-event handler
  (already shipped, m2-core-hook)                              (T2: dispatch →
                                                                tokio::spawn)
                                                                     │
                                                                     ▼
POST /governance/room-event  ◀──bearer BRIDGE_CALLBACK──  room_provisioner
  (T4a: new route + append_room_event)   _SECRET             (T2/T3: createRoom,
                                                              invite Juror-<suffix>
GET /governance/bridge/messaging-status ◀──bearer──         puppets, idempotency)
  (T4b: new route, fixes soft_pause 401)                          │
                                                                  ▼
                                                            bridge_room (T1:
                                                            rusqlite store —
                                                            watermark + reveal
                                                            state, ADR-004)
```

- **Bridge gets a governance consumer** (`room_provisioner.rs`) dispatched
  fire-and-forget (`tokio::spawn`) from the notify handler so the
  `/brehon/notify` ACK returns immediately (scope tripwire: never call the
  Matrix client synchronously inside the handler thread).
- **Binary grows two bridge-callback routes** under `/governance`, both
  authed by `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` via a shared
  `bridge_auth` guard (NOT `is_admin`, NOT JWT) — service-to-service trust,
  least-privilege, mirroring the bridge's existing `hs_token`/`as_token`
  bearer model in *shape* (actix-web, not axum).
- **Bridge owns a rusqlite store** (`bridge_room` table) holding
  `case_id`, `room_type`, `matrix_room_id`, lifecycle state,
  `last_seen_governance_log_row_id` watermark, and per-room OQ-009 reveal
  state — the durability ADR-004 requires for restart-idempotency.

The reader should be able to predict §11 from this diagram: 3 new bridge
source files (config extension, store, provisioner), 1 bridge test file, 1
soft_pause edit, 2 new binary handler files + 1 auth helper + 2 route
registrations.

## 5. Metadata

- **Phase:** `m2-rooms-a`
- **Branch:** `phase-m2-rooms-a` (cut from `governance-v0` @ `f7a9c9c12` by
  BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 9 (Task 0 pre-flight + T1–T6 with T4 split into
  T4a/T4b = 7 impl + retro)
- **Estimated cargo budget:** `~6 GB peak` (workspace `cargo check
  --features full` for T4a/T4b; bridge `cargo check` is small). Runs on the
  laptop advisor via `validate-pending-laptop`, NOT the daemon.
- **Forbidden-window applicability:** standard (per advisor-orchestrator.md
  table). Cargo runs on the laptop, so windows bind only the advisor's local
  validation runs, not worker dispatch.
- **Complexity score:** `5/10` — see breakdown below. Under the Sonnet
  split threshold (`>8`); no split-DQ.

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target model `sonnet-4-6` →
split-DQ threshold `>8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 2 | 7 impl tasks (T1,T2,T3,T4a,T4b,T5,T6); 7−5 = 2 |
| Migrations touched | +2 each | 0 | Zero workspace migrations; `bridge_room` is rusqlite, bridge-local |
| Crates touched | +1 each | 3 | `services/bridge` (excluded), `crates/api/api`, `crates/api/routes` |
| `crates/server/tests/e2e/*.rs` edits | +3 each | 0 | Bridge tests live in `services/bridge/tests/`, not the Lemmy e2e harness |
| New ADR-affecting decisions | +2 each | 0 | Bridge-callback auth is orthogonal to ADR-016 (line 282), supersedes nothing |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Pre-Shape-G; peak ~6 GB, not above |
| **Total** | — | **5** | Threshold for split-DQ: `>8` (Sonnet) → **no split** |

### 5.2 Per-task complexity ceiling

Sonnet target → ceiling `≤ 4` files / `≤ 2` crates per task; e2e edits in
their own task. Every §13 task satisfies this:

- T1: 4 files (bridge_room.rs, config.rs, Cargo.toml, main.rs), 1 crate. ✅
- T1w: 2 files (`api_common/governance.rs`, `api_utils/bridge_notify.rs`) +
  0-1 (`db_schema` read method IFF no existing fetch is reachable — see T1w
  IMPLEMENT file 3), so 2-3 files / 2-3 crates. ✅ (at the ceiling; this is
  the workspace half of the juror-sourcing seam — kept SEPARATE from the
  bridge consumer T2 precisely because the two cross the toolchain boundary:
  T1w is `--workspace --features full`, T2 is `cd services/bridge && cargo
  check`, and §15 / R8 forbid mixing the two validation profiles on one
  task — same split rationale as T4 → T4a/T4b.)
- T2: 3 files (room_provisioner.rs, appservice.rs, main.rs), 1 crate. ✅
- T3: 1 file (room_provisioner.rs extend), 1 crate. ✅
- T4a: 4 files (room_event_handler.rs, bridge_auth.rs, governance/mod.rs,
  routes/lib.rs), 2 crates. ✅ (this is *why* T4 is split — unsplit T4 was
  5 files / 2 crates, over the ceiling)
- T4b: 3 files (bridge_read.rs, governance/mod.rs, routes/lib.rs), 2 crates. ✅
- T5: 2 files (room_provisioner.rs extend, soft_pause.rs), 1 crate. ✅
- T6: 2 files (room_provisioning.rs new test, dm_round_trip.rs un-stub), 1
  crate. ✅

## 6. Relationship to other M2 sub-phases

- **Depends on:** `m2-core-hook` (PR #184, merged to `governance-v0`) — the
  notification fire, `append_room_event`, `RoomEventPayload`, and 10 room
  consts. This plan consumes them; it does NOT re-create them.
- **Followed by:** `m2-late` (gated) — B-publish sanction propagation +
  B-actor portable-ID linkage (OQ-ADR016-02/-04). Out of scope here (§12).
- **Sibling (in-binary):** `m2-core-transition-hook` shipped the binary-side
  half; this is the bridge-side half + the HTTP seam connecting them.

## 7. Preflight guardrails inherited from prior phases

- **R1 (zero-Matrix-deps invariant):** `services/bridge` MUST stay in the
  workspace `exclude` array, never `members`. `cargo tree --workspace |
  grep -cE 'matrix-sdk|ruma'` == 0 after every task (M1 story-6). Probe in
  Task 0 + every task DoD (WP-6).
- **R2 (zero workspace migrations):** the `bridge_room` table is rusqlite,
  bridge-local. No file lands under `crates/db_schema/migrations/**`.
- **R3 (fire-and-forget, non-blocking notify):** the provisioner is
  dispatched via `tokio::spawn` from the `/brehon/notify` handler; the ACK
  returns immediately. No synchronous Matrix-client call inside the handler
  thread (ADR-012 + brief §4 tripwire).
- **R4 (validate-pending-laptop, write-then-stop):** every impl-task writes
  a `validate-pending-laptop` DQ entry, commits, pushes, and STOPS. Workers
  never run cargo on the daemon (`feedback_validate_pending_laptop_write_
  then_stop.md`).
- **R5 (Task 0 enumerates ALL probes explicitly):** no implicit probe
  inheritance (`pre-phase-harness-audit.md`).
- **R6 (clippy uniformity):** workspace clippy invocations (T4a/T4b) use
  `--workspace --features full --no-deps -- -D warnings`; bridge clippy uses
  `cd services/bridge && cargo clippy -- -D warnings` (no `--features full`
  — bridge has no `full` feature, R8).
- **R7 (test-target compile after struct/route change):** `cargo test
  --no-run` on the touched scope after any task that adds a route or a
  public struct.
- **R8 (bridge toolchain isolation):** do NOT apply Lemmy-workspace lessons
  (`feedback_lemmy_error_no_std_error.md`, Diesel lessons, `--features
  full`, `e2e.rs` edit discipline) to `services/bridge/**`. Wrong toolchain
  (bootstrap §5). Bridge uses `anyhow::Result`, axum, matrix-sdk.

## 8. Flow design

**Before (m2-core-hook shipped):**

```
case transition ─▶ governance_case_after_transition ─POST─▶ http://localhost:9009/brehon/notify
                                                                       │
                                                              (no governance consumer —
                                                               bridge only relays DMs)
append_room_event(pool, kind, payload, actor_pseudonym)  ◀── (only callable in-process / e2e)
```

**After (m2-rooms-a):**

```
case transition ─▶ governance_case_after_transition ─POST /brehon/notify─▶ [T2 handler]
                                                                                │ tokio::spawn (R3)
                                                                                ▼
                                                              room_provisioner::handle_transition
                                                                  │  ├─ bridge_room::lookup(case_id, room_type)   [T1 store]
                                                                  │  │     idempotency gate → skip if exists
                                                                  │  ├─ provision::create_*_room               [T2/T3]
                                                                  │  ├─ puppet::ensure_puppet(Juror-<suffix>)   [T2/T3]
                                                                  │  ├─ reveal_threshold ◀─GET /governance/bridge/messaging-status [T4b]
                                                                  │  └─ POST /governance/room-event ─▶ append_room_event [T4a]
                                                                  ▼
                                                              bridge_room::upsert(watermark, reveal_state) [T1/T5]

soft_pause::poll_once ─GET (bearer secret)─▶ /governance/bridge/messaging-status [T4b]  (fixes 401, T5)
```

Each box cites the §13 task that creates/edits it.

## 9. Mandatory reading

The impl-task subagent MUST Read before its first edit:

- **Binary-side shipped primitives (consume, do not re-create):**
  - `crates/api/api/src/governance/governance_log.rs:88-130` —
    `RoomEventPayload` struct + `append_room_event` fn signature (T4a calls
    this).
  - `crates/db_schema/src/source/governance/governance_log.rs:239-248` — the
    10 `ENTRY_KIND_ROOM_*` consts (referenced as string literals from the
    bridge; the binary handler imports the consts).
  - `crates/api/api_utils/src/bridge_notify.rs:50-86` —
    `governance_case_after_transition` + `CaseTransitionEvent` field shape
    (`case_id`, `old_status`, `new_status`, `community_id`, `target_type`).
- **Binary-side patterns to mirror (T4a/T4b):**
  - `crates/api/api/src/governance/messaging_config.rs:160-183` —
    `admin_get_messaging_config` handler shape (extractors, `LemmyResult<Json<T>>`).
  - `crates/api/routes/src/lib.rs:478-509` — the `scope("/governance")` route
    registration block.
- **Bridge-side patterns to mirror (T1/T2/T3/T5):**
  - `services/bridge/src/appservice.rs:33-90` — `AppState` struct +
    `hs_token_auth` middleware shape + `router()` wiring.
  - `services/bridge/src/appservice.rs:144-191` — `handle_provision_room`
    (the `relay_enabled` soft-pause gate pattern).
  - `services/bridge/src/provision.rs` — `create_community_room(config,
    room_alias)` (T2/T3 extend the createRoom path).
  - `services/bridge/src/puppet.rs` — `PuppetMap::ensure_puppet` (juror
    `Juror-<suffix>` puppet resolution).
  - `services/bridge/src/soft_pause.rs:34-41` — `poll_once` (the 401 bug
    site; T5 adds the bearer header).
  - `services/bridge/src/config.rs:9-44` — `BridgeConfig` + `from_env` (T1
    extends).
  - `services/bridge/tests/dm_round_trip.rs` — the `#[ignore = "requires
    docker-compose stack"]` test pattern + the `soft_pause_enable_disable_
    cycle` `todo!()` stub (T6 un-stubs).
- **Lessons:** the five in §2 (binding per the task each gates).

## 10. Patterns to mirror

### 10.1 Binary-side governance handler (T4a/T4b)

**Mirror:** `crates/api/api/src/governance/messaging_config.rs:160-183`

```rust
// actix-web handler convention: extractors, then in-handler auth, then pool.
// Returns LemmyResult<Json<T>>. The auth check is IN-HANDLER (is_admin here),
// NOT a wrap-middleware — the bridge-callback routes follow the same in-handler
// convention but call bridge_auth::verify_bridge_secret(&req)? instead of
// is_admin (service principal, NOT a LocalUserView/JWT).
pub async fn admin_get_messaging_config(
  Query(params): Query<GetMessagingConfigQuery>,
  local_user_view: LocalUserView,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<ConfigValueWithProvenance>> {
  is_admin(&local_user_view)?;
  let pool = &mut context.pool();
  // ...
}
```

### 10.2 Bridge-callback bearer guard (T4a — new `bridge_auth.rs` helper)

**Mirror (shape only):** `services/bridge/src/appservice.rs:52-90`
(`hs_token_auth`) — the bearer-extraction + constant compare. Re-implement
in **actix-web** (NOT axum): extract the `Authorization` header from
`actix_web::HttpRequest`, strip `Bearer `, compare against
`std::env::var("BRIDGE_CALLBACK_SECRET")`, return
`LemmyErrorType` (401-mapped) on mismatch. Both new routes call this guard
as their first line (in-handler, mirroring the `is_admin(&...)?` convention
in 10.1). GOTCHA: actix-web has no `middleware::from_fn_with_state` axum
equivalent in this codebase's idiom — use an in-handler guard fn, not a
`.wrap()` middleware, to match Lemmy's existing in-handler-auth style.

### 10.3 append_room_event call (T4a)

**Mirror:** `crates/api/api/src/governance/governance_log.rs:88-130`

```rust
// ACTUAL shipped signature (CODE WINS — the brief §2 lists the args in a
// slightly different order; use this one):
#[cfg(feature = "full")]
pub async fn append_room_event(
    pool: &mut DbPool<'_>,
    kind: &str,                       // one of the 10 ENTRY_KIND_ROOM_* values
    payload: RoomEventPayload,
    actor_pseudonym: Option<String>,
) -> LemmyResult<GovernanceLog>
```

The T4a handler body deserializes `{ entry_kind: String, payload:
RoomEventPayload }`, validates `entry_kind` is one of the 10 room consts,
and calls `append_room_event(pool, &entry_kind, payload, actor_pseudonym)`.
Single write — `append_room_event` wraps its own transaction internally, so
NO extra `run_transaction` (per `feedback_multi_write_handlers_need_
transactions.md` — this is the single-write case the lesson explicitly
exempts).

### 10.4 Bridge soft-pause gate (T2/T3 provisioning guard)

**Mirror:** `services/bridge/src/appservice.rs:144-158`

```rust
// Refuse provisioning when the bridge is in soft-pause (relay_enabled=false).
// The room_provisioner checks the SAME AtomicBool before any createRoom call,
// so messaging_enabled=false ⇒ zero rooms provisioned (a Phase-5 acceptance).
if !state.relay_enabled.load(std::sync::atomic::Ordering::Relaxed) {
    tracing::info!("provisioning skipped — messaging_enabled=false");
    return; // fire-and-forget; no error propagation (ADR-012)
}
```

### 10.5 Bridge soft-pause poll (T5 — the 401 fix)

**Mirror:** `services/bridge/src/soft_pause.rs:34-41`

```rust
// BEFORE (latent 401 bug): bare GET, no auth header.
async fn poll_once(client: &reqwest::Client, url: &str) -> Result<bool> {
    let resp = client.get(url).send().await?.error_for_status()?;
    // ...
}
// AFTER (T5): add Authorization: Bearer <bridge_callback_secret>; point url
// at the new GET /governance/bridge/messaging-status route (T4b). The
// existing admin_get_messaging_config stays is_admin-gated for the human
// admin panel; the bridge uses its own service-principal read path.
```

## 11. Files to change

**`services/bridge/` (workspace-excluded crate `brehon-bridge`):**

- `services/bridge/Cargo.toml` — add `rusqlite` dependency (T1).
- `services/bridge/src/config.rs` — extend `BridgeConfig` with
  `brehon_room_event_url: String`, `bridge_callback_secret: String`,
  `legal_contact_mxid: String`; repoint `brehon_read_url` env value at the
  new bridge-read route; extend `from_env()` (T1).
- `services/bridge/src/bridge_room.rs` — **NEW**: rusqlite store for the
  `bridge_room` table (`case_id`, `room_type`, `matrix_room_id`, lifecycle
  state, `last_seen_governance_log_row_id` watermark, OQ-009 reveal state);
  `open()`, `lookup(case_id, room_type)`, `upsert(...)`, `watermark(...)`
  (T1, extended T5).
- `services/bridge/src/room_provisioner.rs` — **NEW**: receives
  `CaseTransitionEvent`, provisions/archives rooms (C2.1 jury core path in
  T2; C2.2–C2.6 + OQ-009 reveal + always_pseudonym in T3; binary callback +
  watermark in T5).
- `services/bridge/src/appservice.rs` — add `POST /brehon/room-event` route
  (or extend `/brehon/notify` dispatch) wired to the provisioner via
  `tokio::spawn`; AppState already carries `config`/`relay_enabled` (T2).
- `services/bridge/src/main.rs` — add `mod bridge_room;` (T1) and `mod
  room_provisioner;` (T2) declarations.
- `services/bridge/src/soft_pause.rs` — add bearer header to `poll_once`;
  point at the new read route (T5).
- `services/bridge/tests/room_provisioning.rs` — **NEW**: `#[ignore]` +
  docker-compose integration tests (T6).
- `services/bridge/tests/dm_round_trip.rs` — un-`todo!()` the
  `soft_pause_enable_disable_cycle` test (T6).

**`crates/api/api_common` + `crates/api/api_utils` (workspace) — T1w (DQ -055):**

- `crates/api/api_common/src/governance.rs` — add `pub juror_pseudonyms:
  Vec<String>` to `CaseTransitionEvent` (lines ~860-866) + doc-comment note
  (pre-resolved pseudonyms only, ADR-015); consider `#[serde(default)]` (T1w).
- `crates/api/api_utils/src/bridge_notify.rs` — populate `juror_pseudonyms`
  in `governance_case_after_transition` (lines 53-90) for jury-bound
  `new_status`, from `jury_assignment ⨝ actor_pseudonym`; empty otherwise
  (T1w).
- *(conditional, T1w IMPLEMENT file 3)* a by-case juror-pseudonym read
  method — REUSE `crates/api/api/src/governance/actor_pseudonym_helper.rs` if
  reachable from `api_utils`; else add to the `db_schema` source model
  (`JuryAssignment`). Resolve the `api_utils → api` dep-direction question at
  author time; raise a blocker DQ if no legal path exists.

**`crates/api/api` (workspace):**

- `crates/api/api/src/governance/bridge_auth.rs` — **NEW**:
  `verify_bridge_secret(req: &HttpRequest) -> LemmyResult<()>` bearer guard
  (T4a).
- `crates/api/api/src/governance/room_event_handler.rs` — **NEW**: `POST
  /governance/room-event` handler calling `append_room_event` (T4a).
- `crates/api/api/src/governance/bridge_read.rs` — **NEW**: `GET
  /governance/bridge/messaging-status` handler returning `{messaging_enabled,
  oq009_reveal_threshold}` (T4b).
- `crates/api/api/src/governance/mod.rs` — add `pub mod bridge_auth;`,
  `pub mod room_event_handler;` (T4a); `pub mod bridge_read;` (T4b).

**`crates/api/routes` (workspace):**

- `crates/api/routes/src/lib.rs` — register `.route("/room-event",
  post().to(handle_room_event))` (T4a) and `.route("/bridge/messaging-status",
  get().to(bridge_messaging_status))` (T4b) inside the `scope("/governance")`
  block at lines 478-509.

### Struct-field add: enumerate all callsites

`BridgeConfig` (T1) is constructed at exactly one site: `from_env()` in
`services/bridge/src/config.rs:26`. `rg "BridgeConfig\s*\{" services/bridge/`
confirms a single struct-literal. No external callers (bridge is a single
binary, not a library). Adding three fields requires updating only
`from_env()` — no cross-crate fan-out. (Verified: `BridgeConfig` is
constructed solely via `from_env`; every other reference is `&BridgeConfig`
borrow.)

`CaseTransitionEvent` (T1w, DQ -055) is constructed at exactly one site:
`governance_case_after_transition()` in
`crates/api/api_utils/src/bridge_notify.rs:69`. The planner MUST confirm with
`rg "CaseTransitionEvent\s*\{" crates/` before authoring T1w — if the rg
returns more than the one `bridge_notify.rs` literal, every additional
construction site needs the new field (a struct-literal without
`juror_pseudonyms` won't compile unless `#[serde(default)]` + a `..Default`
is used, which `CaseTransitionEvent` does NOT derive). Consumers are: the
serde round-trip on the bridge (`services/bridge/src/` deserialization in T2)
and any test that builds the event. R7 (test-target compile after struct
change) applies — run `cargo test --no-run` on the touched workspace scope
after T1w. Expected callsite count: 1 producer (`bridge_notify.rs:69`) + N
test fixtures (enumerate at author time).

## 12. NOT building in m2-rooms-a

- **B-publish sanction propagation / B-actor portable-ID linkage** —
  deferred to `m2-late`; gated on OQ-ADR016-02/-04.
- **Recording / transcript / M3 LiveKit/MatrixRTC** — deferred to M3;
  `ENTRY_KIND_ROOM_RECORDING_UPLOADED` + `_TRANSCRIPT_READY` are registered
  but NOT emitted in M2.
- **Cross-instance jury rooms beyond Matrix federation** — OQ-V2-08
  resolved (standard Matrix federation; no AP governance signals).
- **Any change to `CaseStatus` enum or governance decision logic** —
  m2-rooms-a is observer-only; reason: the binary already emits transitions,
  the bridge only consumes them.
- **Tuwunel configuration changes** — pilot Tuwunel is already running.
- **doc-04 drift fix** (`governance_messaging_config` table not reflected in
  `04-data-model-and-api.md`) — deferred to a `chore` commit after merge.
- **CI-runnable bridge tests** — deferred to M3; Phase-5 tests are
  `#[ignore]` + docker-compose (DQ `a3d0e9941441-051`).

---

## 13. Step-by-step tasks

Execute in dependency order. One commit per task. All tasks are non-`[P]`
(serial): T1w→T1, T2→T1w, T3→T2, T4b→T4a, T5→{T2,T4a,T4b}, T6→{T3,T5} form a
tight dependency chain, and T1↔T4a — though file-disjoint — sit either side
of that chain; serial dispatch keeps the daemon at ≤1 running worker and
avoids shared `.git/index.lock` contention for a marginal 1-task parallelism
gain.

> **Amendment 2026-06-06 (DQ `a3d0e9941441-055`, option-a):** Task 1w (NEW,
> workspace-side) was inserted between T1 and T2 to close the juror-sourcing
> gap. The original T2 step (d) ("invite the 5 jurors as `Juror-<suffix>`")
> was unimplementable: `CaseTransitionEvent` carried no juror identities and
> the bridge has no workspace-DB access, so the bridge had no way to learn
> *which* jurors to invite. T1w augments the payload at the binary producer
> (matching the M1 `relay.rs` `brehon_sender`/`brehon_recipient`
> payload-carries-identities precedent); T2 now consumes the new
> `juror_pseudonyms` field. T1w is the ONLY `--workspace --features full`
> task that touches `crates/**` for a data reason (T4a/T4b are the route
> tasks); it must ship before T2 so the field exists to deserialize.

> **Pre-Shape-G validation:** every impl-task writes a `validate-pending-
> laptop` DQ entry, commits, pushes, and STOPS. The laptop advisor runs the
> `commands` per the validate-pending-laptop handler. Workers do NOT run
> cargo on the daemon (R4, `feedback_validate_pending_laptop_write_then_
> stop.md`).

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `m2-rooms-a`; confirm branch is
`phase-m2-rooms-a`; confirm m2-core-hook deliverables are intact on the
base; confirm the zero-Matrix-deps invariant holds at the start.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — branch
git branch --show-current   # EXPECT: phase-m2-rooms-a

# Probe 1 — bridge stays workspace-excluded
grep -A40 '^\[workspace\]' Cargo.toml | grep -c 'services/bridge'   # context only
python -c "import tomllib,sys; d=tomllib.load(open('Cargo.toml','rb')); \
  ex=d['workspace'].get('exclude',[]); me=d['workspace'].get('members',[]); \
  print('EXCLUDE_OK' if any('services/bridge' in e for e in ex) and not any('services/bridge' in m for m in me) else 'EXCLUDE_FAIL')"
# EXPECT: EXCLUDE_OK

# Probe 2 — zero-Matrix-deps invariant (WP-6)
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'
# EXPECT: 0

# Probe 3 — m2-core-hook primitives present on base
grep -c 'pub const ENTRY_KIND_ROOM_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 10
grep -c 'fn append_room_event' crates/api/api/src/governance/governance_log.rs
# EXPECT: 1 (the shim re-export/wrapper)

# Probe 4 — no /governance/room-event route exists yet (T4a adds it)
grep -c 'room-event' crates/api/routes/src/lib.rs
# EXPECT: 0

# Probe 5 — bridge has NO rusqlite yet (T1 adds it)
grep -c 'rusqlite' services/bridge/Cargo.toml
# EXPECT: 0

# Probe 6 — registry total const count (collision check)
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 65
```

**EXPECT block:**
- Probes 0–6 produce the expected values above.
- No cargo invocation at Task 0 (probes are grep/python only — bridge cargo
  is deferred to T1's validate-pending-laptop).

**No commit at Task 0** — verification only.

### Task 1: BridgeConfig extension + rusqlite `bridge_room` store

**ACTION:** add `rusqlite` to the bridge crate; extend `BridgeConfig` with
the three new env-backed fields + repoint `brehon_read_url`; create the
`bridge_room` rusqlite store module.

**FILES:**

```yaml
creates:
  - services/bridge/src/bridge_room.rs
modifies:
  - services/bridge/Cargo.toml          # add rusqlite dependency
  - services/bridge/src/config.rs       # +brehon_room_event_url, +bridge_callback_secret, +legal_contact_mxid; repoint brehon_read_url
  - services/bridge/src/main.rs         # add `mod bridge_room;`
```

**IMPLEMENT (file 1 of 4):** in `services/bridge/Cargo.toml`, add
`rusqlite = { version = "0.32", features = ["bundled"] }` under
`[dependencies]` (DQ `a3d0e9941441-053`, option-a; `bundled` avoids a system
libsqlite dependency). Confirm the chosen version compiles at the
validate-pending-laptop step — pin is advisory.

**IMPLEMENT (file 2 of 4):** in `services/bridge/src/config.rs`, add the
three fields to `BridgeConfig` and read them in `from_env()`
(`BREHON_ROOM_EVENT_URL`, `BRIDGE_CALLBACK_SECRET`, `LEGAL_CONTACT_MXID`).
`brehon_read_url` keeps its field name; its `.env` value is repointed at the
new `GET /governance/bridge/messaging-status` route (the binary-side change
lands in T4b; this is config-only).

**IMPLEMENT (file 3 of 4):** create `services/bridge/src/bridge_room.rs`: a
rusqlite-backed store. Schema (created on `open()` via `CREATE TABLE IF NOT
EXISTS`): `bridge_room(case_id INTEGER, room_type TEXT, matrix_room_id TEXT,
lifecycle_state TEXT, last_seen_governance_log_row_id INTEGER,
reveal_state TEXT, PRIMARY KEY (case_id, room_type))`. Public fns: `open(path)
-> Result<Connection>`, `lookup(&conn, case_id, room_type) -> Result<Option<Row>>`,
`upsert(&conn, ...) -> Result<()>`, `set_watermark(&conn, case_id, row_id)`.
GOTCHA: rusqlite is synchronous — if a write ever lands on the provisioning
hot path, wrap in `tokio::task::spawn_blocking` (DQ -053 note); `bridge_room`
writes are off the hot path, so direct calls are acceptable here.

**IMPLEMENT (file 4 of 4):** in `services/bridge/src/main.rs`, add `mod
bridge_room;` after `mod appservice;`.

**MIRROR:** `services/bridge/src/config.rs:9-44` (`BridgeConfig` +
`from_env` shape).

**GOTCHA:** R8 — bridge toolchain, `anyhow::Result`, no `--features full`,
no Diesel. `BridgeConfig` has a single construction site (`from_env`); no
cross-crate caller fan-out.

**VALIDATE (write-then-stop, R4):** write a `validate-pending-laptop` DQ
entry with `commands: ["cd services/bridge && cargo check"]`, `branch:
"phase-m2-rooms-a"`, `phase_task: 1`; commit + push; STOP. Do NOT run cargo
on the daemon. Laptop advisor confirms exit 0 + the zero-Matrix-deps gate.

### Task 1w: Augment `CaseTransitionEvent` with `juror_pseudonyms` (workspace producer)

> **NEW (DQ `a3d0e9941441-055`, option-a).** Workspace-side task — the ONLY
> `crates/**` data-shape change in this plan. Closes the T2 juror-sourcing
> gap: the bridge cannot learn which jurors to invite because the payload it
> receives carries no identities and the bridge has no workspace-DB access.
> This task makes the binary producer resolve `jury_assignment ⨝
> actor_pseudonym` for jury-bound transitions and push the pseudonyms into
> the event, exactly as the M1 DM-relay pushes `brehon_sender`/
> `brehon_recipient` (`services/bridge/src/relay.rs:28-36`).

**ACTION:** add a `juror_pseudonyms: Vec<String>` field to
`CaseTransitionEvent`; populate it at the binary producer
(`governance_case_after_transition`) for jury-bound transitions from the
existing `jury_assignment ⨝ actor_pseudonym` data; non-jury transitions leave
it empty (no fetch).

**FILES:**

```yaml
modifies:
  - crates/api/api_common/src/governance.rs   # +pub juror_pseudonyms: Vec<String> on CaseTransitionEvent + doc-comment
  - crates/api/api_utils/src/bridge_notify.rs # populate juror_pseudonyms for jury-bound new_status; else empty
creates:
  # 0 or 1 — ONLY if no existing by-case juror-pseudonym fetch is reachable from the producer's crate (see IMPLEMENT file 3)
  # - crates/db_schema/src/source/governance/jury_assignment.rs  (extend with a read method) OR an api-side helper
requires:
  - task: 1
    reason: serial dispatch (file-disjoint from T1 but keeps daemon at ≤1 worker); no logical dep on T1's bridge changes
```

**IMPLEMENT (file 1 of 2):** in `crates/api/api_common/src/governance.rs`,
add `pub juror_pseudonyms: Vec<String>` to `CaseTransitionEvent` (after
`target_type`, the last field, lines ~860-866). Derive-clean on
`Vec<String>` (all of `Debug, Clone, Serialize, Deserialize, PartialEq`
hold). Update the struct doc-comment: the field carries **pre-resolved
pseudonymous handles** (e.g. the `Juror-<suffix>` source pseudonyms), NEVER
usernames/emails/display-names — so the ADR-015 "no real identities cross to
the bridge" invariant is preserved (the binary resolves to *pseudonyms*; the
bridge still owns the `Juror-<suffix>` rendering). For non-jury transitions
the vec is empty. Consider `#[serde(default)]` so older/empty producers
deserialize cleanly.

**IMPLEMENT (file 2 of 2):** in `crates/api/api_utils/src/bridge_notify.rs`
`governance_case_after_transition` (lines 53-90), **conditionally** populate
`juror_pseudonyms`. The function already holds `let pool = &mut
context.pool();` (line 60) and already does an async DB read before
constructing the payload (`GovernanceMessagingConfig::read_current`, lines
60-66) — this fetch mirrors that shape. Gate the fetch on `new_status` being
a jury-bound transition (the C2.1 path — `CaseStatus::JurySelection` and any
status under which a jury room is live per the PRD C2.1 scenario; enumerate
the exact set against the `CaseStatus` enum, do NOT fetch for every
transition). For jury-bound transitions, fetch the pseudonyms for
`case.id` and set them; otherwise `juror_pseudonyms: Vec::new()`.

**IMPLEMENT (file 3 — CONDITIONAL, planner resolves at author time):** the
fetch is `jury_assignment(case_id → person_id)` ⨝ `actor_pseudonym(person_id
→ pseudonym)` → `Vec<String>`. **Before adding any new code, check
`crates/api/api/src/governance/actor_pseudonym_helper.rs` for an existing
by-case juror-pseudonym fetch and reuse it.** CRATE-DIRECTION CAUTION:
`bridge_notify.rs` is in `api_utils`; `actor_pseudonym_helper.rs` is in
`api`. Verify `api_utils → api` is a legal dependency edge (it may not be —
`api` typically depends on `api_utils`, not the reverse). If the helper is
NOT reachable from `api_utils`, put the read on the `db_schema` source model
(a method on `JuryAssignment` returning the joined pseudonyms), which BOTH
crates may call. Resolve this dep-direction question explicitly here; if
neither path is legal without a new dep edge, raise a `kind: blocker` DQ
rather than inventing one.

**MIRROR:** `crates/api/api_utils/src/bridge_notify.rs:60-66` (existing async
DB read before payload construction — the shape to follow for the new fetch);
`services/bridge/src/relay.rs:28-36` (`BridgeNotifyPayload` carries
`brehon_sender`/`brehon_recipient` — the payload-carries-identities
precedent); `crates/db_schema/src/source/governance/jury_assignment.rs:21-24`
+ `actor_pseudonym.rs:20-21` (the join source models).

**GOTCHA:** ADR-015 is load-bearing — resolve to `actor_pseudonym.pseudonym`
ONLY; if any path would put a real username/email/display-name into
`juror_pseudonyms`, STOP and raise a blocker. R3 (non-blocking notify) still
holds: this adds one indexed `case_id` read for jury transitions only — if a
fan-out / N+1 appears, surface it. R1 (zero-Matrix-deps) UNAFFECTED — the
field is pure `Vec<String>`, no Matrix types. R2 (zero workspace migrations)
UNAFFECTED — `jury_assignment` + `actor_pseudonym` already exist (v1-JM-b);
no schema change. This is workspace Rust → validation is `--workspace
--features full` (§15.2), NOT `cd services/bridge && cargo check`.

**VALIDATE (write-then-stop, R4):** write a `validate-pending-laptop` DQ
entry with `commands: ["./scripts/brehon/cargo-check.sh --workspace
--features full"]`, `branch: "phase-m2-rooms-a"`, `phase_task: "1w"`; commit
+ push; STOP. Do NOT run cargo on the daemon. Laptop advisor confirms exit 0
+ the zero-Matrix-deps gate (`cargo tree --workspace | grep -cE
'matrix-sdk|ruma'` == 0).

### Task 2: `room_provisioner.rs` — C2.1 jury room core path + idempotency

**ACTION:** create the provisioner module; wire a `POST /brehon/room-event`
route dispatched fire-and-forget; implement the C2.1 jury-room provisioning
path (createRoom → invite the 5 jurors as `Juror-<suffix>` puppets) with the
`bridge_room` idempotency gate.

**FILES:**

```yaml
creates:
  - services/bridge/src/room_provisioner.rs
modifies:
  - services/bridge/src/appservice.rs   # add POST /brehon/room-event route + tokio::spawn dispatch
  - services/bridge/src/main.rs         # add `mod room_provisioner;`
requires:
  - task: 1
    reason: provisioner uses BridgeConfig new fields + bridge_room::lookup/upsert
  - task: 1w
    reason: deserializes the new CaseTransitionEvent.juror_pseudonyms field; the field must exist on the wire before T2 reads it
```

**IMPLEMENT (file 1 of 3):** create
`services/bridge/src/room_provisioner.rs`:
`handle_transition(state, event: CaseTransitionEvent)`. For the C2.1 jury
path (transition into jury selection): (a) check `relay_enabled` soft-pause
gate (§10.4) — skip if false; (b) `bridge_room::lookup(case_id, "jury")` —
if a row exists, skip (idempotency: `Room::Created` fires exactly once per
`case_id`+`room_type` even across restart); (c) call
`provision::create_community_room` (extended) for the jury room; (d) **for
each pseudonym in `event.juror_pseudonyms`** (populated by the binary
producer in T1w — the bridge does NOT compute these; ADR-015 keeps real
identities binary-side), call `PuppetMap::ensure_puppet(&pseudonym)` and
invite as `Juror-<suffix>`. If `event.juror_pseudonyms` is empty on a
jury-bound transition, log + skip the invite loop (the producer fetch
returned nothing — non-fatal, ADR-012); (e) `bridge_room::upsert` the new
room state. NO reporter/reported/admin in a jury room (ADR-015). Deserialize
the `CaseTransition` variant of the payload the binary sends.

**IMPLEMENT (file 2 of 3):** in `services/bridge/src/appservice.rs`, add a
`POST /brehon/room-event` route (handler `handle_room_event`) that
deserializes the `CaseTransitionEvent`, then dispatches
`tokio::spawn(room_provisioner::handle_transition(state.clone(), event))`
and returns `200 {}` IMMEDIATELY (R3 — never block the ACK on the Matrix
client). Wire it inside `router()` (it inherits the `hs_token_auth`
route_layer — confirm the binary→bridge call carries the hs_token, or add a
dedicated unauthed path if the notify fire is unauthed; mirror how
`/brehon/notify` is currently authed).

**IMPLEMENT (file 3 of 3):** in `services/bridge/src/main.rs`, add `mod
room_provisioner;`.

**MIRROR:** `services/bridge/src/appservice.rs:144-191`
(`handle_provision_room`, soft-pause gate); `services/bridge/src/provision.rs`
(`create_community_room`); `services/bridge/src/puppet.rs`
(`PuppetMap::ensure_puppet`); `services/bridge/src/relay.rs:28-36` +
relay-handler use of `payload.brehon_sender`/`brehon_recipient` (the
payload-carries-identities consume pattern — T2's juror loop is the same
shape over `event.juror_pseudonyms`).

**GOTCHA:** R3 — the notify handler MUST return before any Matrix call. Use
`tokio::spawn`; the provisioner owns its own error handling (log + swallow,
ADR-012). Idempotency key is `(case_id, room_type)`, not `case_id` alone (a
case may have a jury room AND an appeal room).

**VALIDATE (write-then-stop, R4):** `validate-pending-laptop` DQ,
`commands: ["cd services/bridge && cargo check"]`, `phase_task: 2`. Commit +
push + STOP.

### Task 3: C2.2–C2.6 room scenarios + OQ-009 reveal + always_pseudonym

**ACTION:** extend the provisioner to cover the remaining five scenarios
(community-event, spin-out, appeal, emergency, membership-mirror), the
OQ-009 graduated-reveal logic, and bridge-side `always_pseudonym`
enforcement.

**FILES:**

```yaml
modifies:
  - services/bridge/src/room_provisioner.rs   # extend with C2.2-C2.6, OQ-009 reveal, always_pseudonym
requires:
  - task: 2
    reason: extends the same handle_transition dispatch + bridge_room store
```

**IMPLEMENT (file 1 of 1):** in `services/bridge/src/room_provisioner.rs`,
add match arms for C2.2 community-event, C2.3 spin-out, C2.4 appeal, C2.5
emergency (ADR-013: admins + `legal_contact_mxid`, reported party ABSENT,
target <2s), C2.6 membership-mirror. OQ-009 graduated reveal: before
rendering room membership, query the Matrix room event count
(`GET /_matrix/client/v3/rooms/{roomId}/messages?limit=1`); if count ≥
threshold use `Juror-<suffix>`, else an opaque `Juror-pending`/blank display
name. The threshold comes from the bridge-read route (`oq009_reveal_threshold`,
T4b; default 1) — cache it from the soft_pause poll or fetch at room-join.
`always_pseudonym` enforcement: for `jury*`/`appeal*` rooms, refuse to
provision unless the pseudonym config is present; if absent, log + skip
(non-fatal, ADR-012). Persist per-room reveal state in `bridge_room`.

**MIRROR:** `services/bridge/src/room_provisioner.rs` (T2 jury path — same
shape per scenario); `crates/api/api/src/governance/messaging_config.rs:69-83`
(the binary-side identity-policy validator, for the `always_pseudonym`
semantics to mirror bridge-side).

**GOTCHA:** emergency room is the latency-critical path (ADR-013 <2s);
provision it FIRST, defer chain-emission to the async callback. OQ-009
reveal is bridge-observed (the bridge can see Matrix room event count); do
NOT reach into Brehon for comment counts.

**VALIDATE (write-then-stop, R4):** `validate-pending-laptop` DQ,
`commands: ["cd services/bridge && cargo check"]`, `phase_task: 3`. Commit +
push + STOP.

### Task 4a: Binary `POST /governance/room-event` route + bridge bearer guard

**ACTION:** add the binary-side HTTP callback route exposing
`append_room_event`, plus the shared `BRIDGE_CALLBACK_SECRET` bearer guard
both new routes use (DQ `a3d0e9941441-052`, option-a).

**FILES:**

```yaml
creates:
  - crates/api/api/src/governance/bridge_auth.rs
  - crates/api/api/src/governance/room_event_handler.rs
modifies:
  - crates/api/api/src/governance/mod.rs    # pub mod bridge_auth; pub mod room_event_handler;
  - crates/api/routes/src/lib.rs            # .route("/room-event", post().to(handle_room_event))
```

**IMPLEMENT (file 1 of 4):** create
`crates/api/api/src/governance/bridge_auth.rs`:
`verify_bridge_secret(req: &HttpRequest) -> LemmyResult<()>` — extract the
`Authorization` header, strip `Bearer `, compare against
`std::env::var("BRIDGE_CALLBACK_SECRET")`; on mismatch/absence return a
401-mapped `LemmyErrorType`. Service-principal auth — NO `LocalUserView`, NO
`is_admin` (§10.2). Use a length-constant compare if a helper is readily
available; otherwise a plain `==` is acceptable for v0 (note in a comment).

**IMPLEMENT (file 2 of 4):** create
`crates/api/api/src/governance/room_event_handler.rs`: `handle_room_event(req:
HttpRequest, body: Json<RoomEventRequest>, context: Data<LemmyContext>) ->
LemmyResult<Json<...>>`. First line: `bridge_auth::verify_bridge_secret(&req)?`.
Deserialize `RoomEventRequest { entry_kind: String, payload: RoomEventPayload,
actor_pseudonym: Option<String> }`; validate `entry_kind` ∈ the 10
`ENTRY_KIND_ROOM_*` consts; call `append_room_event(&mut context.pool(),
&entry_kind, payload, actor_pseudonym)` (§10.3). Single write — no
`run_transaction` (`feedback_multi_write_handlers_need_transactions.md`
single-write exemption).

**IMPLEMENT (file 3 of 4):** in `crates/api/api/src/governance/mod.rs`, add
`pub mod bridge_auth;` and `pub mod room_event_handler;` (alphabetical).

**IMPLEMENT (file 4 of 4):** in `crates/api/routes/src/lib.rs`, inside the
`scope("/governance")` block (lines 478-509), add
`.route("/room-event", post().to(handle_room_event))`.

**MIRROR:** `crates/api/api/src/governance/messaging_config.rs:160-183`
(handler shape); `services/bridge/src/appservice.rs:52-90` (bearer-extraction
shape — re-implement in actix-web); `crates/api/routes/src/lib.rs:478-509`
(route registration).

**GOTCHA:** the `/governance` scope wraps `rate_limit.post()` — the bridge
callback inherits it; confirm the rate ceiling is acceptable for the
bridge's call volume (one call per provisioning action, low). actix-web
in-handler guard, NOT a `.wrap()` middleware (§10.2 GOTCHA). `RoomEventPayload`
is behind `#[cfg(feature = "full")]` — the handler is too.

**VALIDATE (write-then-stop, R4):** `validate-pending-laptop` DQ,
`commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`,
`phase_task: 4`. Commit + push + STOP. (Workspace gate — this is the only
workspace-touching pair; `--features full` is workspace-scope, NOT
`-p lemmy_server`, per `feedback_features_full_workspace_only.md`.)

### Task 4b: Binary `GET /governance/bridge/messaging-status` bridge-read route

**ACTION:** add the bridge-read route (same bearer secret) returning
`{messaging_enabled, oq009_reveal_threshold}`, fixing the soft_pause 401 bug
class at the binary side (DQ `a3d0e9941441-054`).

**FILES:**

```yaml
creates:
  - crates/api/api/src/governance/bridge_read.rs
modifies:
  - crates/api/api/src/governance/mod.rs    # pub mod bridge_read;
  - crates/api/routes/src/lib.rs            # .route("/bridge/messaging-status", get().to(bridge_messaging_status))
requires:
  - task: 4a
    reason: reuses bridge_auth::verify_bridge_secret guard
```

**IMPLEMENT (file 1 of 3):** create
`crates/api/api/src/governance/bridge_read.rs`: `bridge_messaging_status(req:
HttpRequest, context: Data<LemmyContext>) -> LemmyResult<Json<BridgeStatus>>`.
First line: `bridge_auth::verify_bridge_secret(&req)?`. Read
`GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled")`
and the `(scope="jury_rooms", key="oq009_reveal_threshold")` row (default 1).
Return `BridgeStatus { messaging_enabled: bool, oq009_reveal_threshold: i64 }`.
This is the bridge's service-principal read path; the `is_admin`-gated
`admin_get_messaging_config` stays unchanged for the human admin panel.

**IMPLEMENT (file 2 of 3):** in `crates/api/api/src/governance/mod.rs`, add
`pub mod bridge_read;` (alphabetical).

**IMPLEMENT (file 3 of 3):** in `crates/api/routes/src/lib.rs`, inside
`scope("/governance")`, add `.route("/bridge/messaging-status",
get().to(bridge_messaging_status))`.

**MIRROR:** `crates/api/api/src/governance/messaging_config.rs:160-183`
(`admin_get_messaging_config` — same read shape minus `is_admin`).

**GOTCHA:** the read returns BOTH the messaging flag AND the OQ-009
threshold in one call so the bridge's soft_pause poll (T5) gets the reveal
threshold for free, avoiding a second round-trip.

**VALIDATE (write-then-stop, R4):** `validate-pending-laptop` DQ,
`commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`,
`phase_task: 4`. Commit + push + STOP.

### Task 5: Bridge callback wiring + restart-idempotency + soft_pause fix

**ACTION:** the provisioner POSTs to the binary `room-event` route after
each provisioning action (`Room::*` lands on the chain); restart-idempotency
via the `last_seen_governance_log_row_id` watermark; repoint soft_pause at
the new read route with the bearer header.

**FILES:**

```yaml
modifies:
  - services/bridge/src/room_provisioner.rs   # POST /governance/room-event after each action; watermark
  - services/bridge/src/soft_pause.rs         # add bearer header; point at /governance/bridge/messaging-status
requires:
  - task: 2
    reason: extends the provisioner created in T2
  - task: 4a
    reason: calls POST /governance/room-event (T4a route) with the bearer secret
  - task: 4b
    reason: soft_pause now polls the GET /governance/bridge/messaging-status route (T4b)
```

**IMPLEMENT (file 1 of 2):** in `services/bridge/src/room_provisioner.rs`,
after each provisioning action POST `{entry_kind, payload, actor_pseudonym}`
to `config.brehon_room_event_url` with `Authorization: Bearer
<config.bridge_callback_secret>`; map the scenario onto the correct
`ENTRY_KIND_ROOM_*` value (creation → `room_created`; archive →
`room_archived`; juror add/remove → `room_member_added`/`_removed`; generic
→ `room_lifecycle_event`). Update `bridge_room.last_seen_governance_log_row_id`
from the response. On restart, the idempotency lookup (T2) + watermark
prevents duplicate `Room::Created`.

**IMPLEMENT (file 2 of 2):** in `services/bridge/src/soft_pause.rs`, extend
`poll_once` to send `Authorization: Bearer <bridge_callback_secret>` and
point at the repointed `brehon_read_url` (now the
`/governance/bridge/messaging-status` route). Parse `messaging_enabled` (and
cache `oq009_reveal_threshold` for T3's reveal logic if the poller owns the
cache). This closes the M1 401 silent-no-relay bug.

**MIRROR:** `services/bridge/src/soft_pause.rs:34-41` (§10.5);
`crates/api/api_utils/src/bridge_notify.rs:38-45` (reqwest POST + swallow
pattern).

**GOTCHA:** `cargo-output-capture.md` — chain emission must NOT block the
provisioning latency-critical path (emergency <2s); POST is fire-and-forget
with log-and-continue on transport error (ADR-012). The watermark write is
the idempotency anchor — write it only AFTER a successful chain append.

**VALIDATE (write-then-stop, R4):** `validate-pending-laptop` DQ,
`commands: ["cd services/bridge && cargo check"]`, `phase_task: 5`. Commit +
push + STOP.

### Task 6: Integration tests + un-stub soft_pause test

**ACTION:** add `#[ignore]` + docker-compose integration tests for the
acceptance criteria; un-`todo!()` the M1 `soft_pause_enable_disable_cycle`
test now that the 401 bug is fixed.

**FILES:**

```yaml
creates:
  - services/bridge/tests/room_provisioning.rs
modifies:
  - services/bridge/tests/dm_round_trip.rs    # un-todo!() soft_pause_enable_disable_cycle
requires:
  - task: 3
    reason: tests exercise the full C2.1-C2.6 provisioning surface
  - task: 5
    reason: tests assert chain emission + restart-idempotency + soft_pause fix
```

**IMPLEMENT (file 1 of 2):** create
`services/bridge/tests/room_provisioning.rs` with `#[ignore = "requires
docker-compose stack"]` tests (DQ `a3d0e9941441-051`): (a) jury room
provisions in <5s with exactly the 5 assigned jurors as `Juror-<suffix>`,
no reporter/reported/admin, `always_pseudonym` enforced; (b) `emergency_remove`
→ emergency room in <2s with admins + `legal_contact_mxid`, reported party
absent; (c) 10 `Room::*` entries land on the chain with correct schema after
one full lifecycle; (d) bridge restart mid-case → no duplicate
`Room::Created`; (e) `messaging_enabled=false` → zero rooms provisioned,
zero `Room::*` entries. Reference the docker-compose stack in a module
doc-comment. If any test opens a Brehon PG connection, use the
`AsyncPgConnection::establish` pattern (`feedback_async_pool_test_pattern.md`).

**IMPLEMENT (file 2 of 2):** in `services/bridge/tests/dm_round_trip.rs`,
replace the `todo!()` body of `soft_pause_enable_disable_cycle` with a real
`#[ignore]` test asserting the bearer-authed poll toggles `relay_enabled`
correctly (no 401).

**MIRROR:** `services/bridge/tests/dm_round_trip.rs` (the existing
`#[ignore = "requires docker-compose stack"]` pattern).

**GOTCHA:** R8 — bridge tests use `anyhow`, NOT `LemmyError`; do NOT apply
`feedback_lemmy_error_no_std_error.md`. Timing assertions (<5s/<2s) are
meaningful only against a real Tuwunel; the `#[ignore]` marker keeps them
out of CI (M3 scope).

**VALIDATE (write-then-stop, R4):** `validate-pending-laptop` DQ,
`commands: ["cd services/bridge && cargo test --no-run"]`, `phase_task: 6`.
Commit + push + STOP. (Test COMPILE only; `#[ignore]` execution is manual
against docker-compose.)

### Task 7: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` and
`feedback_four_role_retro_signals.md`. One H2 per role (Advisor / Planning /
Impl / BM) with signals + lessons. Promote any new lessons to
`.claude/lessons/feedback_*.md` in the same retro commit. Candidate
lessons: bridge-side service-principal HTTP-callback auth pattern; rusqlite
idempotency-watermark pattern for fire-and-forget bridge state.

---

## 14. Testing strategy

- **Unit (compile-time), bridge:** `cd services/bridge && cargo check`
  (every bridge task).
- **Unit (compile-time), workspace:** `cargo check --workspace --features
  full` (T4a/T4b).
- **Lint, bridge:** `cd services/bridge && cargo clippy -- -D warnings` (no
  `--features full`, R8).
- **Lint, workspace:** `cargo clippy --workspace --features full --no-deps
  -- -D warnings` (T4a/T4b, R6).
- **Test target compile, bridge:** `cd services/bridge && cargo test
  --no-run` (T6, and any task touching a public struct/route).
- **Integration (manual, docker-compose):** `cd services/bridge && cargo
  test --test room_provisioning -- --ignored` — run against a live
  docker-compose Tuwunel stack; NOT in CI (DQ -051, M3 scope).
- **Zero-Matrix-deps gate (every task):** `cargo tree --workspace 2>/dev/null
  | grep -cE 'matrix-sdk|ruma'` == 0.

---

## 15. Validation commands (DoD)

> **Planner-side discipline:** every command dry-run against current HEAD
> before approval (advisor §3.4). Bridge cargo uses `cd services/bridge &&
> cargo check` (NO `--features full` — bridge has no `full` feature, R8).
> Workspace cargo uses `--workspace --features full` (NOT `-p lemmy_server`,
> per `feedback_features_full_workspace_only.md`).

### 15.1 Bridge static analysis (T1, T2, T3, T5, T6)

```bash
cd services/bridge && cargo check > /tmp/m2-rooms-a-bridge-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### 15.2 Workspace static analysis (T1w, T4a, T4b)

```bash
./scripts/brehon/cargo-check.sh --workspace --features full > /tmp/m2-rooms-a-ws-check.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### 15.3 Zero-Matrix-deps gate (every task — WP-6)

```bash
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'
# EXPECT: 0
```

### 15.4 Lint

```bash
# Bridge (T1,T2,T3,T5,T6):
cd services/bridge && cargo clippy -- -D warnings > /tmp/m2-rooms-a-bridge-clippy.log 2>&1
echo "exit: $?"   # EXPECT: exit 0
# Workspace (T1w,T4a,T4b):
./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > /tmp/m2-rooms-a-ws-clippy.log 2>&1
echo "exit: $?"   # EXPECT: exit 0
```

### 15.5 Test compile (T6)

```bash
cd services/bridge && cargo test --no-run > /tmp/m2-rooms-a-bridge-testcompile.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

### 15.6 Cross-cutting verification

- [ ] R1: `services/bridge` stays in workspace `exclude` (never `members`).
- [ ] R2: zero files under `crates/db_schema/migrations/**`.
- [ ] R3: `/brehon/room-event` handler returns the ACK before any Matrix
      call (`tokio::spawn` dispatch).
- [ ] R4: every impl-task wrote a `validate-pending-laptop` DQ and STOPped;
      no daemon-side cargo.
- [ ] WP-6: `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` == 0 after
      every task.
- [ ] Both new binary routes call `bridge_auth::verify_bridge_secret` (NOT
      `is_admin`, NOT JWT).
- [ ] `soft_pause::poll_once` sends `Authorization: Bearer
      <bridge_callback_secret>` and targets `/governance/bridge/messaging-status`.
- [ ] Idempotency: `bridge_room` keyed on `(case_id, room_type)`; restart
      produces no duplicate `Room::Created`.
- [ ] Registry const count stays 65 (no new `ENTRY_KIND_*` — this plan
      consumes the existing 10, adds none).
- [ ] T1w: `CaseTransitionEvent.juror_pseudonyms` carries ONLY
      `actor_pseudonym.pseudonym` values (ADR-015 — no real
      usernames/emails/display-names reach the bridge); the producer fetch is
      gated to jury-bound `new_status` (empty vec otherwise); no workspace
      migration added (R2 holds); `cargo tree --workspace | grep -cE
      'matrix-sdk|ruma'` still 0 (R1 holds — the field is pure `Vec<String>`).

---

## 16. Acceptance criteria

- [ ] All 10 tasks (T0, T1, T1w, T2, T3, T4a, T4b, T5, T6, retro) completed
      in dependency order.
- [ ] §15.1/15.2 (cargo check) exit 0 after every task (bridge / workspace
      per task).
- [ ] §15.3 (zero-Matrix-deps) == 0 after every task.
- [ ] §15.4 (clippy) exit 0 after every task.
- [ ] §15.5 (test compile) exit 0 after T6.
- [ ] §15.6 cross-cutting — all boxes ticked.
- [ ] §16a stories — all `[done]`.
- [ ] No edits to files outside §11; no workspace migration; `services/bridge`
      stays excluded.
- [ ] Retro committed per §13 Task 7.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo
      barrie-cork/lemmy`.

---

## 16a. Stories

> Bridge runtime behaviour (room provisioning, timing, chain emission) is
> testable end-to-end only against a docker-compose Tuwunel stack
> (`#[ignore]`, DQ -051), which does NOT run in CI. Each story's
> **automatable** checkpoint is therefore the compile + test-compile +
> structural grep; the runtime assertions are the manual docker-compose
> acceptance tests named in §13 T6, run by the operator before merge.

### Story 1: Bridge provisions governance rooms with enforced pseudonymity

- **Composing tasks:** Task 1, Task 1w, Task 2, Task 3
- **Checkpoint command:** `./scripts/brehon/cargo-check.sh --workspace
  --features full && cd services/bridge && cargo check && cargo tree
  --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'`
- **Expected output:** workspace `cargo check` exit 0 (T1w field compiles);
  bridge `cargo check` exit 0; grep count `0`
- **Brief-Scope outputs to verify** (`/brehon-verify`):
  - `crates/api/api_common/src/governance.rs` `CaseTransitionEvent` contains
    `juror_pseudonyms: Vec<String>` (T1w)
  - `crates/api/api_utils/src/bridge_notify.rs`
    `governance_case_after_transition` populates `juror_pseudonyms` for
    jury-bound transitions (T1w)
  - `services/bridge/src/room_provisioner.rs` exists, contains
    `handle_transition` + match arms for the C2.1–C2.6 scenarios, and its
    jury-invite loop sources from `event.juror_pseudonyms`
  - `services/bridge/src/bridge_room.rs` exists, contains `lookup` + `upsert`
  - `services/bridge/Cargo.toml` contains `rusqlite`
  - `services/bridge/src/config.rs` `BridgeConfig` contains
    `bridge_callback_secret`, `brehon_room_event_url`, `legal_contact_mxid`

### Story 2: Binary exposes the bridge-callback HTTP surface (bearer-authed)

- **Composing tasks:** Task 4a, Task 4b
- **Checkpoint command:** `./scripts/brehon/cargo-check.sh --workspace
  --features full && grep -c 'room-event\|bridge/messaging-status'
  crates/api/routes/src/lib.rs`
- **Expected output:** `cargo check` exit 0; grep count `2`
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/bridge_auth.rs` exists, contains
    `verify_bridge_secret`
  - `crates/api/api/src/governance/room_event_handler.rs` exists, calls
    `append_room_event`
  - `crates/api/api/src/governance/bridge_read.rs` exists, returns
    `messaging_enabled` + `oq009_reveal_threshold`
  - `crates/api/routes/src/lib.rs` registers both routes inside
    `scope("/governance")`

### Story 3: End-to-end chain emission, restart-idempotency, soft_pause fix

- **Composing tasks:** Task 5, Task 6
- **Checkpoint command:** `cd services/bridge && cargo test --no-run`
- **Expected output:** exit 0 (test binaries compile)
- **Brief-Scope outputs to verify:**
  - `services/bridge/src/room_provisioner.rs` POSTs to
    `brehon_room_event_url` with the bearer secret + writes the watermark
  - `services/bridge/src/soft_pause.rs` `poll_once` sends the bearer header
  - `services/bridge/tests/room_provisioning.rs` exists with the 5
    `#[ignore]` acceptance tests
  - `services/bridge/tests/dm_round_trip.rs` `soft_pause_enable_disable_cycle`
    is no longer `todo!()`
- **Manual (docker-compose, pre-merge):** `cd services/bridge && cargo test
  --test room_provisioning -- --ignored` against a live stack — jury <5s,
  emergency <2s, 10-entry lifecycle, restart-no-dup, messaging-disabled
  clean.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–6 confirmed)
- [ ] Tasks 1–6 + 4a/4b committed (one commit per task)
- [ ] §15 validation green at every gate
- [ ] §16a stories all `[done]`
- [ ] Retro committed
- [ ] PR opened by BM session against `governance-v0` with `--repo
      barrie-cork/lemmy`
- [ ] CodeRabbit review complete with findings triaged
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/m2-rooms-a-verify.md`
      shows all stories ✓
- [ ] Post-merge phase branch retained for retro reads

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `rusqlite` version pin doesn't compile on the bridge toolchain | LOW | LOW | T1 pin is advisory; validate-pending-laptop confirms; `bundled` feature avoids system libsqlite |
| `tokio::spawn` provisioner panics silently, breaking the ACK invariant | MED | MED | R3 — provisioner owns log-and-swallow error handling (ADR-012); §10.4 soft-pause gate; T6 restart test asserts no dup |
| Bridge-callback route inherits `rate_limit.post()` ceiling, throttling chain emission | LOW | MED | §13 T4a GOTCHA — bridge call volume is low (one per provisioning action); confirm ceiling at review |
| OQ-009 reveal threshold round-trips per room-join, adding latency | MED | LOW | §13 T4b GOTCHA — read route returns threshold alongside `messaging_enabled`; cache in the soft_pause poller |
| Emergency room misses the <2s ADR-013 target due to sync chain emission | MED | HIGH | §13 T3/T5 GOTCHA — provision emergency room FIRST, chain emission is async fire-and-forget |
| A future merge cherry-picks a bridge fix across a restructure | LOW | MED | `feedback_cherry_pick_onto_restructured_file_reinjects_content.md` — re-author across restructure boundaries |

---

## 19. Notes

- **No DQ pre-seeds.** All 4 clarify-DQ entries (`a3d0e9941441-051`/`-052`/
  `-053`/`-054`) were resolved before this plan was authored (brief §9);
  the plan implements their resolutions directly. No forward-looking OQ
  needs a planner pre-seed.
- **CODE-WINS reconciliation:** the brief §2 lists the `append_room_event`
  signature with `&str` and `payload` in a different order than the shipped
  code. §10.3 uses the ACTUAL shipped signature
  (`pool, kind: &str, payload, actor_pseudonym`). The brief also references
  provisioning-scenario const names (`ROOM_EMERGENCY_PROVISIONED`,
  `ROOM_APPEAL_PROVISIONED`) that do NOT exist in the shipped 10-const set;
  this plan maps scenarios onto the 10 ACTUAL consts (`room_created`,
  `room_lifecycle_event`, etc — §13 T5) rather than inventing new kinds (the
  registry const count stays 65).
- **Auth design choice:** §10.2 specs an **in-handler** `bridge_auth` guard
  rather than a `.wrap()` middleware, to match Lemmy's existing in-handler
  auth convention (`is_admin(&...)?`). The brief WP-1 says "middleware"; the
  shape (bearer extraction + compare) is mirrored, the mechanism (in-handler
  fn) follows the codebase idiom. Flagged here for advisor awareness.
- **T4 split rationale:** unsplit T4 was 5 files / 2 crates, over the Sonnet
  per-task ceiling (≤4 files). Split into T4a (callback route + auth helper)
  and T4b (read route) — each within ceiling (§5.2). The brief §7 explicitly
  anticipated this split.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — primitives confirmed shipped; one open design
  choice (in-handler guard vs middleware) flagged in §19.
- **Cargo budget:** 9/10 — bridge check is cheap; workspace check well-
  characterised at ~6 GB; runs on laptop.
- **Test coverage:** 6/10 — runtime acceptance is `#[ignore]`+docker-compose
  (not CI); automatable checkpoints are compile-only. Inherent to the bridge
  M2 scope (DQ -051; CI-runnable bridge tests are M3).
