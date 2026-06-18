# Plan: m3-core-infra — deployable, fully-optional RTC stack + LiveKit JWT seam

## 1. Summary

This sub-phase makes the M3 town-hall **real-time-communication stack deployable and fully optional**, and gives the bridge the ability to mint LiveKit access tokens whose participant identity is a **pseudonym** (ADR-015). It delivers: a new `rtc_enabled` config row (default **`false`**) seeded into the existing `governance_messaging_config` typed-KV table plus the read consumer that surfaces it to the bridge; RTC-state columns (`chair_id`, `queue_state`, `recording_config`) added to the bridge-local SQLite `bridge_room` table via the embedded-schema convention; a bridge-side LiveKit HS256 JWT mint function; a Brehon-side `actor-pseudonym` resolution endpoint that calls the canonical `actor_pseudonym_helper::get_or_create` allocator (the load-bearing ADR-015 callsite for the whole M3 cluster); the LiveKit / lk-jwt-service / Element Call Docker sidecars gated behind a compose profile so a governance-only instance runs none of them; and the AGPL-NOTICE rows for those sidecars. **Headline acceptance:** `rtc_enabled=false` runs a clean governance-only instance with the entire RTC stack absent and zero side-effects; `rtc_enabled=true` mints a valid LiveKit JWT carrying a pseudonym identity for an `always_pseudonym` room. **Phase 1 mints a JWT and stands up the stack; it does NOT run a town hall** (stage mode, chair, queue, mute, recording are Phases 3–5).

## 2. Source

- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 1: M3-core infra" (lines 225–228), §Technical Approach (174–203), §Success Criteria (132–151), §ADR table (48–60), §Decisions Log D4/D5/D6 (271–273) @ `488e1d075`.
- `.claude/PRPs/briefs/m3-core-infra-planning-1.md` (the authorising brief) @ `7b0042c1c`.
- `.claude/PRPs/handovers/m3-core-infra-bootstrap.md` §4 watchlist + §7 catch-fire + §"Stop-and-ask tripwires" @ `7b0042c1c`.
- Clarify DQs resolved by user 2026-06-18: `a3d0e9941441-063` (LiveKit creds from `.env`), `a3d0e9941441-064` (deploy-smoke DoD), `a3d0e9941441-065` (Phase 1 does NOT touch `governance_log.rs`).
- ADRs (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`): **ADR-015** (pseudonymised actor IDs; line 257) — load-bearing; **ADR-011** (AGPLv3 inherited; line 199) — source-disclosure; **ADR-004** (plane separation; line 59) — RTC sidecars out-of-binary.
- Lessons that bind decisions:
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); never Windows-local.
  - `feedback_linux_compile_proof_is_a_gate.md` — every bridge-touching task gates on a `validate-pending-laptop-linux` DQ.
  - `feedback_plan_dod_dry_run_at_write.md` — §15 commands must dry-run clean (m3-core-entry-kinds used absent `rg` + line-count greps; this plan avoids both).
  - PMD retro id 907 (m2-late-1 task-5) — governance handlers use the federation `Data<LemmyContext>`, NOT `actix_web::web::Data`; the bridge-read handlers use `actix_web::web::Data` (§4 watchpoint).

## 3. Problem statement

A governance instance today has the M2 text-room machinery (bridge daemon, `bridge_room` table, `governance_messaging_config` KV, pseudonym allocator) but **no real-time floor and no way to gate one on/off**. There is no `rtc_enabled` flag, no LiveKit token minting, no RTC state on `bridge_room`, and the RTC sidecars are not in any compose stack. Concretely, before this sub-phase ships:

- An operator cannot turn the RTC layer on or off — there is no config row (Task 1).
- The bridge has no row-level state to track a room's chair / queue / recording config (Task 2).
- The bridge cannot produce a LiveKit join token (Task 3), and there is no ADR-015-safe path to resolve a `person_id` to the pseudonym that token must carry (Task 4).
- LiveKit / lk-jwt-service / Element Call are not deployable alongside the bridge, and a governance-only instance has no way to run "none of them" (Task 5).
- The source-disclosure obligation for the new sidecars is unmet (Task 5, AGPL-NOTICE).
- There is no tested proof that a governance-only instance (`rtc_enabled=false`) runs unchanged with the RTC stack absent (Task 6).

## 4. Solution statement

The change has **three architectural surfaces**, each with its own validation toolchain:

**(a) Postgres config + binary read path (`crates/**`, Windows-validated).** A new repo-root migration seeds one `rtc_enabled=false` instance-scoped row into `governance_messaging_config` — a **data seed, not a schema change** (no `CREATE`/`ALTER`; mirrors the M1 `messaging_enabled` seed exactly). The bridge-read surface (`crates/api/api/src/governance/bridge_read.rs`) gains an `rtc_enabled` field on `BridgeStatus`, read via the existing `GovernanceMessagingConfig::read_current` idiom, so the bridge can later gate RTC provisioning on it. The same file gains a second bridge-read endpoint — `get_bridge_actor_pseudonym` — that calls `actor_pseudonym_helper::get_or_create(pool, person_id)` and returns the opaque pseudonym. **This is the ADR-015 allocator callsite** (the binary owns the `DbPool`; the bridge does not).

**(b) Bridge daemon (`services/bridge/**`, Linux-validated).** The SQLite `bridge_room` table gains three nullable `TEXT` columns (`chair_id`, `queue_state`, `recording_config`) via the bridge's **embedded-schema** convention — extend the `CREATE TABLE IF NOT EXISTS` block in `bridge_room.rs::open()` AND add idempotent `ALTER TABLE … ADD COLUMN` guards for already-provisioned DBs (the bridge has no `.sql` migration runner; the embedded-schema extension *is* the bridge schema change). A new `livekit_jwt.rs` module mints a LiveKit access token: an **HS256 JWT** over the documented LiveKit claims `{iss: api_key, sub: identity, nbf, exp, video: {room, roomJoin}}`, signed with `LIVEKIT_API_SECRET` (a single small dep — `jsonwebtoken` — NOT the LiveKit SDK). The token `sub`/`identity` is **always a pseudonym** by the function's contract; no `person_id`/username/email type ever flows into it. `BridgeConfig` gains `livekit_api_key`/`livekit_api_secret`/`livekit_url` read from env — **optional at startup** (so a governance-only bridge with no LiveKit creds starts cleanly; the mint fn errors only when actually called without creds).

**(c) Docker sidecars + licence (deploy-smoke + meta).** LiveKit Server (Apache-2.0), lk-jwt-service (Apache-2.0), Element Call (AGPL-3.0) are appended to `services/bridge/docker-compose.yml` under a **`profiles: ["rtc"]`** gate, so `docker compose up` (no profile) starts none of them and `docker compose --profile rtc up` starts the RTC stack. `docker/docker-compose.yml` stays Lemmy-only. AGPL-NOTICE rows are added for all three. **MinIO is NOT in this phase** (Phase 5, gated by `record_town_halls`).

The ADR-015 seam is split across the HTTP boundary by necessity (see §19): the **allocator call** lands in the Brehon endpoint (Task 4), the **JWT `sub`-assignment** lands in the bridge mint fn (Task 3); the bridge→endpoint fetch that connects them end-to-end is **Phase 3** (town-hall provisioning). Both halves are present and grep-able in Phase 1 per the handover §4.3 DoD.

A reader can predict §11 from this: a migration pair, `bridge_read.rs`, `bridge_room.rs`, a new `livekit_jwt.rs`, `config.rs`, `main.rs`, `Cargo.toml`/`.lock`, `routes/src/lib.rs`, `docker-compose.yml`, `AGPL-NOTICE.md`, and one e2e module.

## 5. Metadata

- **Phase:** `m3-core-infra` (M3 Phase 1)
- **Branch:** `phase-m3-core-infra` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 8 (Task 0 pre-flight + 6 impl + retro)
- **Estimated cargo budget:** N/A on daemon — **no cargo runs on EliteDesk** (validate-pending-laptop / -linux discipline; the laptop is the runner). Bridge Linux container first-run is COLD (~10–20 min).
- **Forbidden-window applicability:** binding for the **local** laptop cargo + the deploy-smoke (Docker image pulls); non-binding for daemon impl-task throughput (no daemon cargo).
- **Complexity score:** `13/10` — see breakdown below. Exceeds the Sonnet split threshold (8); **proceed-as-one per the explicit user decision 2026-06-18** (brief §2 — M3 Phase 1 is the natural deploy-unit). See DQ pre-seed in §19 and §5.1 note.

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 1 | 6 impl tasks; 1 above the 5 baseline |
| Migrations touched | +2 each | 1 (+2) | The `rtc_enabled` **row-seed** migration (data, not schema — minimal round-trip risk) |
| Crates touched | +1 each | 4 (+4) | `lemmy_api`, `lemmy_routes`, `lemmy_server` (e2e), `brehon-bridge` (workspace-excluded) |
| `crates/server/tests/e2e/*.rs` edits | +3 each | 2 (+6) | Task 4 (pseudonym endpoint e2e) + Task 6 (clean-posture e2e) — **small additive tests**, not 8000-line refactors |
| New ADR-affecting decisions | +2 each | 0 | Consumes ADR-015/011/004; supersedes none |
| Cargo budget peak above 6 GB | +1 per GB | 0 | No daemon cargo (validate-pending-laptop) |
| **Total** | — | **13** | Threshold for split-DQ: `>8` (Sonnet) |

**Split decision:** the score exceeds 8, which mechanically triggers a split-DQ. The user **pre-decided one-plan** on 2026-06-18 (brief §2: "Plan all of M3-core-infra Phase 1 in one plan — the natural deploy-unit, matches PRD Phase 1 boundary"). For a Sonnet target the planner may answer proceed (not the forbidden non-Sonnet precedent override). The score is inflated by the e2e `+3` weight applied to **two small additive tests** (a clean-posture assertion and a pseudonym-endpoint assertion), not the >8000-line-file edit-hang risk that factor targets. The cohort discipline (§13 `[P]` markers) keeps individual Junior dispatches small. **Proceed-as-one.** Pre-seeded as a resolved planner DQ (§19).

### 5.2 Per-task complexity ceiling (Sonnet target ≤4 files / ≤2 crates)

All tasks satisfy the Sonnet ceiling. Closest: **Task 3** hand-edits 4 files (`livekit_jwt.rs` new, `config.rs`, `Cargo.toml`, `main.rs`) — `Cargo.lock` is a **generated artifact** regenerated by `cargo-linux.sh`, not a hand-edit, so the hand-edited count is 4 ≤ 4, 1 crate. No e2e file appears in any `modifies:` bundled with non-test logic.

## 6. Relationship to other M3 sub-phases

- **Depends on:** M2 (shipped) — bridge daemon, `bridge_room` table, `governance_messaging_config` KV, `actor_pseudonym` allocator. m3-core-entry-kinds (Phase 2, shipped `a5fc60a2d`) — the 3 chair/mute consts exist; this phase emits ZERO of them.
- **Followed by:** Phase 3 (stage-mode) consumes this phase's mint fn + pseudonym endpoint (wires the bridge→endpoint fetch + RTC provisioning gated on `rtc_enabled`). Phase 4 (emergency-mute), Phase 5 (recording / MinIO), Phase 6 (e2e + pilot).
- **Does NOT touch** `governance_log.rs` (clarify DQ `a3d0e9941441-065`): chair-action chain emission is Phase 3/4.

## 7. Preflight guardrails inherited from prior phases

- **R1 (bridge-Linux):** every `services/bridge` cargo command runs via `scripts/brehon/cargo-linux.sh … --manifest-path services/bridge/Cargo.toml`; the Windows-local `cd services/bridge && cargo` form fails with `ruma-common` E0119 and is reference-only (`feedback_bridge_validates_on_linux_not_windows.md`).
- **R2 (Linux-compile gate):** every bridge-touching task (2, 3) writes a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` is gated on `result:pass` (`feedback_linux_compile_proof_is_a_gate.md`).
- **R3 (no daemon cargo):** workers write the appropriate `validate-pending-laptop[-linux]` DQ and **stop**; the laptop advisor runs all cargo (`feedback_validate_pending_laptop_write_then_stop.md`).
- **R4 (cargo capture):** every cargo invocation captures to a log file and reads `tail -20` + `echo exit: $?`; never pipe cargo through `tail`/`grep` (`cargo-output-capture.md` + `no-cargo-output-paste.md`).
- **R5 (Task 0 enumerates all probes explicitly):** see Task 0.
- **R6 (clippy uniform):** all clippy invocations use `--no-deps -- -D warnings`; crates clippy adds `--features full`; bridge clippy runs via `cargo-linux.sh`.
- **R7 (off-path is tested, not assumed):** the `rtc_enabled=false` clean posture is a §16a story with a green e2e checkpoint, not an untested assumption (the "build what tests exercise" discipline).
- **R8 (no Postgres schema migration):** Task 1 is a **row-seed** under repo-root `migrations/`; a `CREATE`/`ALTER` (a real schema migration) under `crates/db_schema/migrations/**` is a scope violation → STOP (handover tripwire).

## 8. Flow design

```
BEFORE (M2):
  governance_messaging_config: { messaging_enabled, identity_policy, oq009_reveal_threshold }
  bridge soft-pause poller --GET--> /bridge/messaging-status --> BridgeStatus { messaging_enabled, oq009_reveal_threshold }
  bridge_room (SQLite): { case_id, room_type, matrix_room_id, lifecycle_state, last_seen_..., reveal_state }
  services/bridge/docker-compose.yml: [ tuwunel ]
  bridge: no LiveKit token minting; no LiveKit creds

AFTER (m3-core-infra):
  governance_messaging_config: { …, rtc_enabled=false (NEW seed row) }
  /bridge/messaging-status --> BridgeStatus { messaging_enabled, oq009_reveal_threshold, rtc_enabled }   [Task 1]
  /bridge/actor-pseudonym?person_id=N --> { pseudonym }                                                   [Task 4]
        └─ calls actor_pseudonym_helper::get_or_create(pool, person_id)   ← ADR-015 allocator callsite
  bridge_room (SQLite): { …, chair_id, queue_state, recording_config (NEW, nullable TEXT) }               [Task 2]
  bridge livekit_jwt::mint_access_token(api_key, api_secret, room, identity=<pseudonym>, ttl, grants)     [Task 3]
        └─ HS256 JWT, sub = pseudonym (NEVER person_id/username)          ← ADR-015 sub-assignment
  services/bridge/docker-compose.yml: [ tuwunel, (profile rtc) livekit, lk-jwt-service, element-call ]    [Task 5]
  AGPL-NOTICE.md: + LiveKit + lk-jwt-service + Element Call rows                                           [Task 5]

  rtc_enabled=false  → BridgeStatus.rtc_enabled=false; `docker compose up` (no profile) starts no RTC; governance flow unchanged  [Task 6]
  rtc_enabled=true   → bridge can mint a pseudonym-JWT; `docker compose --profile rtc up` starts the stack
```

The bridge→`/bridge/actor-pseudonym` fetch + the RTC-provisioning gate on `rtc_enabled` are **Phase 3** (dashed seam); Phase 1 lands both endpoints + the mint fn standalone.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **Config seed pattern** — `migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql` (the `messaging_enabled` seed: `INSERT … ON CONFLICT (scope,key,valid_from) DO NOTHING` with a pinned `valid_from` literal). [Task 1]
- **Read path** — `crates/db_schema/src/source/governance/governance_messaging_config.rs:75-91` (`read_current`); `crates/api/api/src/governance/bridge_read.rs:1-46` (`get_bridge_messaging_status` — the mirror for both the `rtc_enabled` field and the new pseudonym endpoint). [Tasks 1, 4]
- **Pseudonym allocator** — `crates/api/api/src/governance/actor_pseudonym_helper.rs:41-80` (`get_or_create(pool, person_id) -> LemmyResult<String>`). [Task 4]
- **Route registration** — `crates/api/routes/src/lib.rs:46` (import) + `:484-485` (`.route("/bridge/messaging-status", …)`). [Tasks 1, 4]
- **Bridge embedded schema** — `services/bridge/src/bridge_room.rs:3-17` (`open()` CREATE TABLE). [Task 2]
- **Bridge config** — `services/bridge/src/config.rs:45-77` (`from_env`; note required vs optional env idiom: `.context()?` = required, `.unwrap_or_else(…)` = optional). [Task 3]
- **Bridge entrypoint** — `services/bridge/src/main.rs:12-22` (module decls), `:46-63` (config + AppState wiring). [Task 3]
- **Docker sidecar stanza** — `services/bridge/docker-compose.yml:16-72` (tuwunel stanza, networks, volumes); `services/bridge/docker-compose.pilot.yml` (real-image stanza shape + `extra_hosts` pattern). [Task 5]
- **AGPL-NOTICE format** — `AGPL-NOTICE.md:28-43` (the `### services/bridge/` + `### Tuwunel` rows — the "what it is / licence / §13 applicability" three-part pattern). [Task 5]
- **Clean-posture e2e mirror** — `crates/server/tests/e2e/governance.rs:~5355-5390` (`messaging_enabled` absent → `false` → no-op governance transition). [Task 6]
- **Lessons** — `feedback_bridge_validates_on_linux_not_windows.md`, `feedback_linux_compile_proof_is_a_gate.md`, `feedback_validate_pending_laptop_write_then_stop.md`. [Tasks 2, 3]

## 10. Patterns to mirror

### 10.1 Config-row seed (Postgres, data not schema)

**Mirror:** `migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql` (seed block, ~line 52)

```sql
-- valid_from pinned to a STABLE literal so reruns hit the same row under the
-- unique index (scope,key,valid_from) and ON CONFLICT DO NOTHING is a true no-op.
INSERT INTO governance_messaging_config (scope, key, value_type, value_int, value_bool, value_text, valid_from) VALUES
    ('instance', 'rtc_enabled', 'bool', NULL, false, NULL, '2026-06-18T00:00:00Z'::timestamptz)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
```

`down.sql`: `DELETE FROM governance_messaging_config WHERE scope='instance' AND key='rtc_enabled' AND valid_from='2026-06-18T00:00:00Z'::timestamptz;` (deletes only the seeded row, never admin edits at other `valid_from`).

### 10.2 Bridge-read endpoint reading a config row

**Mirror:** `crates/api/api/src/governance/bridge_read.rs:15-46`

```rust
#[cfg(feature = "full")]
#[derive(Debug, Serialize)]
pub struct BridgeStatus {
  pub messaging_enabled: bool,
  pub oq009_reveal_threshold: i64,
  // ADD: pub rtc_enabled: bool,
}

// inside get_bridge_messaging_status, after oq009_reveal_threshold:
let rtc_enabled =
  GovernanceMessagingConfig::read_current(pool, "instance", "rtc_enabled")
    .await?
    .and_then(|r| r.value_bool)
    .unwrap_or(false);   // absent → false (clean-posture default)
```

### 10.3 `read_current`

**Mirror:** `crates/db_schema/src/source/governance/governance_messaging_config.rs:75-91` — `read_current(pool, scope, key) -> LemmyResult<Option<Self>>`.

### 10.4 Pseudonym allocator (ADR-015 callsite)

**Mirror:** `crates/api/api/src/governance/actor_pseudonym_helper.rs:41-80`

```rust
// Task 4: in bridge_read.rs, a Bearer-secret-authed GET handler:
pub async fn get_bridge_actor_pseudonym(
  req: HttpRequest,
  Query(params): Query<BridgeActorPseudonymQuery>,   // { person_id: i32 }
  context: Data<LemmyContext>,
) -> LemmyResult<Json<BridgeActorPseudonym>> {
  bridge_auth::verify_bridge_secret(&req)?;
  let pool = &mut context.pool();
  let pseudonym =
    actor_pseudonym_helper::get_or_create(pool, PersonId(params.person_id)).await?;
  Ok(Json(BridgeActorPseudonym { pseudonym }))   // opaque UUID; never username/email
}
```

`Data` here is `actix_web::web::Data<LemmyContext>` (mirror `bridge_read.rs:3-7`), **NOT** `activitypub_federation::config::Data` — see §4 watchpoint.

### 10.5 Bridge embedded schema + idempotent ALTER

**Mirror:** `services/bridge/src/bridge_room.rs:3-17`

```rust
pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS bridge_room (
            case_id   INTEGER NOT NULL,
            room_type TEXT    NOT NULL,
            matrix_room_id TEXT,
            lifecycle_state TEXT,
            last_seen_governance_log_row_id INTEGER,
            reveal_state TEXT,
            chair_id        TEXT,   -- ADD (M3): current chair pseudonym
            queue_state     TEXT,   -- ADD (M3): FIFO raised-hand queue (JSON text)
            recording_config TEXT,  -- ADD (M3): recording knobs (JSON text)
            PRIMARY KEY (case_id, room_type)
        );"
    )?;
    // Idempotent guards for DBs created before M3 (CREATE TABLE IF NOT EXISTS
    // does NOT add columns to an existing table). SQLite has no ADD COLUMN IF
    // NOT EXISTS — swallow the "duplicate column name" error per column.
    for col in ["chair_id", "queue_state", "recording_config"] {
        let stmt = format!("ALTER TABLE bridge_room ADD COLUMN {col} TEXT");
        match conn.execute(&stmt, []) {
            Ok(_) => {}
            Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
                if msg.contains("duplicate column name") => {}
            Err(e) => return Err(e),
        }
    }
    Ok(conn)
}
```

### 10.6 Bridge config env (optional-at-startup idiom)

**Mirror:** `services/bridge/src/config.rs:45-77`. LiveKit fields are **optional** (RTC may be off):

```rust
// BridgeConfig new fields:
pub livekit_url: Option<String>,
pub livekit_api_key: Option<String>,
pub livekit_api_secret: Option<String>,
// in from_env(): use .ok() (NOT .context()?) so a governance-only bridge starts
// without LiveKit creds. The mint fn errors if called while any is None.
livekit_url: env::var("LIVEKIT_URL").ok(),
livekit_api_key: env::var("LIVEKIT_API_KEY").ok(),
livekit_api_secret: env::var("LIVEKIT_API_SECRET").ok(),
```

### 10.7 Route registration

**Mirror:** `crates/api/routes/src/lib.rs:485` —
`.route("/bridge/actor-pseudonym", get().to(get_bridge_actor_pseudonym))` directly after the `messaging-status` route; add `bridge_read::get_bridge_actor_pseudonym` to the import at line 46.

### 10.8 Docker sidecar stanza, profile-gated

**Mirror:** `services/bridge/docker-compose.yml:18-47` (tuwunel) + `docker-compose.pilot.yml` (real-image + `extra_hosts`). New stanzas carry `profiles: ["rtc"]` so default `up` starts none:

```yaml
  livekit:
    image: livekit/livekit-server:v1.8   # Apache-2.0; pin exact tag at impl
    profiles: ["rtc"]
    command: ["--config", "/etc/livekit.yaml"]
    ports: ["7880:7880", "7881:7881"]
    networks: [bridge-net]
  lk-jwt-service:
    image: ghcr.io/element-hq/lk-jwt-service:0.3.0   # Apache-2.0; pin at impl
    profiles: ["rtc"]
    environment:
      LIVEKIT_URL: "ws://livekit:7880"
      LIVEKIT_KEY: "devkey"        # dev-only; real key via .env at deploy
      LIVEKIT_SECRET: "devsecret"
    ports: ["8085:8080"]           # host 8085 — avoids the bridge's 8080
    networks: [bridge-net]
  element-call:
    image: ghcr.io/element-hq/element-call:0.6.0     # AGPL-3.0; pin at impl
    profiles: ["rtc"]
    ports: ["8086:8080"]
    networks: [bridge-net]
```

(Exact image tags + the lk-jwt `/healthz` port are confirmed at impl time against the pulled images; the deploy-smoke asserts liveness.)

### 10.9 AGPL-NOTICE row

**Mirror:** `AGPL-NOTICE.md:28-43` — add a `## Additional components — M3 RTC stack` section with three `### <component>` rows, each stating *what it is / licence (Apache-2.0 or AGPL-3.0) / §13 applicability*, modelled on the Tuwunel row (Apache pattern) and the bridge row (AGPL pattern).

### 10.10 LiveKit access-token claims (HS256)

The mint fn produces a standard LiveKit access token (no SDK):

```rust
// services/bridge/src/livekit_jwt.rs
#[derive(serde::Serialize)]
struct VideoGrant { room: String, #[serde(rename = "roomJoin")] room_join: bool }
#[derive(serde::Serialize)]
struct Claims<'a> {
    iss: &'a str,            // LIVEKIT_API_KEY
    sub: &'a str,            // identity == PSEUDONYM (ADR-015) — never person_id/username
    nbf: u64, exp: u64,
    video: VideoGrant,
}
// jsonwebtoken: Header::new(Algorithm::HS256), EncodingKey::from_secret(api_secret.as_bytes())
pub fn mint_access_token(
    api_key: &str, api_secret: &str, room: &str, identity: &str, ttl_secs: u64,
) -> anyhow::Result<String> { /* … */ }
```

`identity: &str` is a pseudonym by contract (the only string the caller passes as `sub`). There is no overload taking `person_id`.

### 10.11 Clean-posture e2e

**Mirror:** `crates/server/tests/e2e/governance.rs:~5355-5390` (the `messaging_enabled`-absent → no-op governance transition). Task 6's test asserts a governance transition completes with `rtc_enabled` absent/false and zero RTC side-effects.

## 11. Files to change

**Migration (repo-root `migrations/`, NOT `crates/db_schema/migrations/`):**
- `migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/up.sql` — CREATE: INSERT `rtc_enabled=false` row (Task 1)
- `migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/down.sql` — CREATE: DELETE the seeded row (Task 1)
- **No `schema.rs` regen** — a data seed changes no table shape.

**`crates/api` (lemmy_api + lemmy_routes), Windows-validated:**
- `crates/api/api/src/governance/bridge_read.rs` — add `rtc_enabled` to `BridgeStatus` + read (Task 1); add `get_bridge_actor_pseudonym` endpoint + `BridgeActorPseudonymQuery`/`BridgeActorPseudonym` types (Task 4)
- `crates/api/routes/src/lib.rs` — wire `/bridge/actor-pseudonym` route (Task 4)

**`services/bridge` (brehon-bridge, workspace-excluded), Linux-validated:**
- `services/bridge/src/bridge_room.rs` — RTC-state columns + idempotent ALTER guards (Task 2)
- `services/bridge/src/livekit_jwt.rs` — NEW: `mint_access_token` (Task 3)
- `services/bridge/src/config.rs` — `livekit_url`/`livekit_api_key`/`livekit_api_secret` (optional) (Task 3)
- `services/bridge/src/main.rs` — `mod livekit_jwt;` (Task 3)
- `services/bridge/Cargo.toml` — add `jsonwebtoken` dep (Task 3)
- `services/bridge/Cargo.lock` — regenerated by `cargo-linux.sh` (generated artifact, committed) (Task 3)

**Deploy + meta:**
- `services/bridge/docker-compose.yml` — LiveKit + lk-jwt-service + Element Call stanzas, `profiles: ["rtc"]` (Task 5)
- `AGPL-NOTICE.md` — M3 RTC stack rows (Task 5)

**Tests (lemmy_server e2e), Windows-validated:**
- `crates/server/tests/e2e/governance.rs` — pseudonym-endpoint test (Task 4) + `rtc_enabled=false` clean-posture test (Task 6)

### Struct-field add: enumerate all callsites

`BridgeStatus` (Task 1) is constructed at exactly one site — `bridge_read.rs:42` (`Ok(Json(BridgeStatus { … }))`). `rg "BridgeStatus" crates/` returns only the definition + that one constructor (the bridge consumes the JSON, not the Rust struct). No cross-crate constructor enumeration needed. `BridgeActorPseudonym` (Task 4) is a new type with one constructor. No `Default`-shape callers to chase.

## 12. NOT building in m3-core-infra

- **Bridge→`/bridge/actor-pseudonym` fetch + RTC-provisioning gate on `rtc_enabled`** — deferred to Phase 3 (stage-mode provisioning); Phase 1 lands both seam halves standalone. Reason: "Phase 1 does NOT run a town hall" (brief §2).
- **Stage mode / dual-sourced chair / FIFO queue / mic-passing / chair override / Q&A sidebar** — Phase 3.
- **Federation-wide emergency mute (`room_mute_all` emission)** — Phase 4.
- **MinIO / LiveKit Egress / recording / `record_town_halls` flag / `Room::RecordingUploaded` emission** — Phase 5 (gated by `record_town_halls`).
- **The 3 chair/mute entry-kind consts** — ALREADY SHIPPED (Phase 2, `a5fc60a2d`); Phase 1 must NOT re-touch `governance_log.rs` (clarify DQ `a3d0e9941441-065`).
- **Any new Postgres schema migration** (CREATE/ALTER under `crates/db_schema/migrations/**`) — Task 1 is a row-seed only (handover tripwire).
- **The LiveKit Rust SDK / `livekit-api` crate** — a single `jsonwebtoken` dep mints the HS256 token (brief task 3 decision).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Task 0 is non-`[P]` (barrier).

> **Cohort note:** Task 1 (`crates/api`, Windows) and Task 2 (`services/bridge`, Linux) are file-disjoint across different crates and different validation runners → genuine `[P]`. Bridge Tasks 2 and 3 share the bridge crate + `Cargo.lock` surface, so Task 3 is **serial after** Task 2 (avoids lock races + concurrent cold Linux builds). Tasks 4–6 are barriers (file overlap or e2e). Per the cross-lane cap, at most 2 Junior workers run concurrently.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify the environment + branch + base state before Task 1.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — Docker daemon (needed for e2e testcontainers AND the deploy-smoke + bridge Linux build)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — Docker is in LINUX-container mode (cargo-linux.sh + sidecars require it)
docker info --format '{{.OSType}}'   # EXPECT: linux

# Probe 2 — on the phase branch
git branch --show-current            # EXPECT: phase-m3-core-infra

# Probe 3 — Phase-2 deliverable intact on base (the 3 chair/mute consts are present; Phase 1 must NOT re-touch)
rg -c '^pub const ENTRY_KIND_ROOM_(CHAIR_TRANSFERRED|CHAIR_OVERRIDE|MUTE_ALL)' \
  crates/db_schema/src/source/governance/governance_log.rs   # EXPECT: 3

# Probe 4 — config table + read path present
rg -c 'pub async fn read_current' crates/db_schema/src/source/governance/governance_messaging_config.rs  # EXPECT: 1

# Probe 5 — pseudonym allocator present
rg -c 'pub async fn get_or_create' crates/api/api/src/governance/actor_pseudonym_helper.rs  # EXPECT: 1

# Probe 6 — bridge baseline COMPILES on Linux (cold ~10-20 min; warm after)
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-core-infra-task0-bridge-check.log 2>&1
echo "bridge baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task0-bridge-check.log  # EXPECT: exit 0

# Probe 7 — crates baseline compiles (Windows)
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-core-infra-task0-check.log 2>&1"
echo "crates baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task0-check.log  # EXPECT: exit 0

# Probe 8 (negative) — wrapper propagates failure
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m3-core-infra-task0-neg.log 2>&1"
echo "negative exit: $?"   # EXPECT: NON-ZERO
```

**EXPECT:** Probes 0–7 succeed per their inline expectations; Probe 8 exits non-zero. **No commit at Task 0.**

### Task 1 [P]: `rtc_enabled` config-row seed + `BridgeStatus.rtc_enabled` read

**ACTION:** seed an `rtc_enabled=false` instance row into `governance_messaging_config` (data, not schema) and expose it on the bridge-read `BridgeStatus`.

**FILES:**

```yaml
creates:
  - migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/up.sql
  - migrations/2026-06-18-000000-0000_seed_rtc_enabled_config/down.sql
modifies:
  - crates/api/api/src/governance/bridge_read.rs   # add rtc_enabled to BridgeStatus + read_current
```

**IMPLEMENT (file 1 of 3):** `up.sql` per §10.1 — single `INSERT … ON CONFLICT (scope,key,valid_from) DO NOTHING`, `value_type='bool'`, `value_bool=false`, `valid_from='2026-06-18T00:00:00Z'`. No `CREATE`/`ALTER`.
**IMPLEMENT (file 2 of 3):** `down.sql` — `DELETE … WHERE scope='instance' AND key='rtc_enabled' AND valid_from='2026-06-18T00:00:00Z'`.
**IMPLEMENT (file 3 of 3):** `bridge_read.rs` — add `pub rtc_enabled: bool` to `BridgeStatus`; read via `read_current(pool, "instance", "rtc_enabled")` → `.and_then(|r| r.value_bool).unwrap_or(false)`; set the field in the `BridgeStatus { … }` constructor.

**MIRROR:** §10.1 (seed), §10.2 (BridgeStatus read), §10.3 (read_current).

**GOTCHA:** this is a **row-seed migration**, NOT a schema change — do not regen `schema.rs`, do not `CREATE`/`ALTER`. Pin `valid_from` to the stable literal so `diesel migration redo` is idempotent. Default-on-absent is `false` (clean-posture).

**VALIDATE (Windows; story feeds §16a Story 1):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-core-infra-task1-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task1-check.log   # EXPECT: exit 0
bash scripts/brehon/migrate-roundtrip.sh 2026-06-18-000000-0000_seed_rtc_enabled_config
echo "roundtrip exit: $?"   # EXPECT: exit 0 (up + down + redo clean)
```

Write a `validate-pending-laptop` DQ with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""]`, commit + push, then **stop** (do not run cargo on the daemon).

### Task 2 [P]: `bridge_room` RTC-state columns + idempotent ALTER guards

**ACTION:** extend the embedded SQLite schema in `bridge_room.rs::open()` with `chair_id`/`queue_state`/`recording_config` (nullable TEXT) + idempotent ALTER guards for already-provisioned DBs.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/bridge_room.rs   # CREATE TABLE cols + ALTER guards in open()
```

**IMPLEMENT:** per §10.5 — add the 3 columns to the `CREATE TABLE IF NOT EXISTS` block AND the post-create `ALTER TABLE … ADD COLUMN` loop that swallows SQLite's "duplicate column name" error per column. Add a `#[cfg(test)]` test: open a fresh DB → assert the 3 columns exist (`PRAGMA table_info(bridge_room)`); open a DB pre-seeded with the OLD 6-column shape → assert `open()` adds the 3 columns and is idempotent on a second call.

**MIRROR:** `services/bridge/src/bridge_room.rs:3-17` + the `rusqlite::Error::SqliteFailure` match shape in §10.5.

**GOTCHA:** `CREATE TABLE IF NOT EXISTS` does NOT alter an existing table — the ALTER guards are load-bearing for DBs created before M3. SQLite has no `ADD COLUMN IF NOT EXISTS`; match-and-swallow the duplicate-column error, do NOT blindly ignore all errors. Bridge compiles on **Linux only**.

**VALIDATE (Linux; story feeds §16a Story 3):**

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-core-infra-task2-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task2-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml bridge_room \
  > .claude/PRPs/debug/m3-core-infra-task2-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task2-bridge-test.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (`commands` = the `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test` lines above), commit + push, then **stop**.

### Task 3: LiveKit JWT mint fn + bridge LiveKit config + `jsonwebtoken` dep

**ACTION:** add a bridge-side `mint_access_token` (HS256 LiveKit token, `sub`=pseudonym) + the optional LiveKit env config + the `jsonwebtoken` dependency.

**FILES:**

```yaml
creates:
  - services/bridge/src/livekit_jwt.rs
modifies:
  - services/bridge/src/config.rs    # optional livekit_url/api_key/api_secret
  - services/bridge/src/main.rs      # mod livekit_jwt;
  - services/bridge/Cargo.toml       # jsonwebtoken dep
  # services/bridge/Cargo.lock regenerated by cargo-linux.sh (generated artifact, commit it)
requires:
  - task: 2
    reason: serial bridge ownership — Task 3 shares services/bridge/Cargo.lock; run after Task 2 to avoid lock races + concurrent cold Linux builds
```

**IMPLEMENT (file 1 of 4):** `livekit_jwt.rs` per §10.10 — `mint_access_token(api_key, api_secret, room, identity, ttl_secs) -> anyhow::Result<String>`; `Claims { iss: api_key, sub: identity, nbf, exp, video: VideoGrant { room, room_join: true } }`; sign HS256 via `jsonwebtoken::encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(api_secret.as_bytes()))`. Add a `#[cfg(test)]` test: mint with a sample pseudonym → decode (validation off or with the same secret) → assert `sub == "<pseudonym>"`, `iss == api_key`, `video.room == room`, `exp > nbf`.
**IMPLEMENT (file 2 of 4):** `config.rs` — 3 **optional** fields per §10.6 (`.ok()`, not `.context()?`).
**IMPLEMENT (file 3 of 4):** `main.rs` — add `mod livekit_jwt;` to the module block (alphabetical: after `link_handler`).
**IMPLEMENT (file 4 of 4):** `Cargo.toml` — `jsonwebtoken = "9"` (already present transitively in the workspace lock; known-good). Regenerate `Cargo.lock` via `cargo-linux.sh` and commit it.

**MIRROR:** §10.10 (claims), §10.6 (config), `services/bridge/src/config.rs:9-43` (field doc-comment style).

**GOTCHA (ADR-015 — load-bearing):** `mint_access_token`'s `identity`/`sub` is a **pseudonym** — the signature takes `identity: &str` and there is NO variant that accepts a `person_id`/username/email. **Why it can't be deferred:** a real identity in the LiveKit `sub` reaches the LiveKit server, breaking the `always_pseudonym` anonymity guarantee — the load-bearing ADR-015 seam for the whole M3 cluster. **DoD:** `rg 'sub:' services/bridge/src/livekit_jwt.rs` shows `sub: identity` (the pseudonym param) AND `rg -i 'person_id|username|email' services/bridge/src/livekit_jwt.rs` returns nothing. **GOTCHA (clean-posture):** LiveKit env is OPTIONAL — a governance-only bridge must start without LiveKit creds; the mint fn returns an error only when called with creds absent. Bridge compiles on **Linux only**.

**VALIDATE (Linux; story feeds §16a Story 2):**

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-core-infra-task3-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task3-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml livekit_jwt \
  > .claude/PRPs/debug/m3-core-infra-task3-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task3-bridge-test.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (the 3 `cargo-linux.sh` lines), commit + push, then **stop**.

### Task 4: Brehon-side `actor-pseudonym` resolution endpoint (ADR-015 allocator callsite) + route

**ACTION:** add the Bearer-secret-authed `GET /api/v4/governance/bridge/actor-pseudonym` endpoint that calls `actor_pseudonym_helper::get_or_create`, plus its route.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/bridge_read.rs   # get_bridge_actor_pseudonym + query/response types
  - crates/api/routes/src/lib.rs                   # /bridge/actor-pseudonym route
  - crates/server/tests/e2e/governance.rs          # endpoint e2e test (small additive)
requires:
  - task: 1
    reason: Task 1 also edits bridge_read.rs (BridgeStatus.rtc_enabled); Task 4 extends the same file — sequence after Task 1
```

**IMPLEMENT (file 1 of 3):** `bridge_read.rs` — `get_bridge_actor_pseudonym` per §10.4 (`verify_bridge_secret` → `get_or_create(pool, PersonId(person_id))` → `Json(BridgeActorPseudonym { pseudonym })`); add `BridgeActorPseudonymQuery { person_id: i32 }` (Deserialize) + `BridgeActorPseudonym { pseudonym: String }` (Serialize). Use `actix_web::web::Data<LemmyContext>` (mirror the file's existing handler).
**IMPLEMENT (file 2 of 3):** `routes/src/lib.rs` — add `get_bridge_actor_pseudonym` to the `bridge_read::` import (line 46) + `.route("/bridge/actor-pseudonym", get().to(get_bridge_actor_pseudonym))` after the messaging-status route (line 485).
**IMPLEMENT (file 3 of 3):** `governance.rs` e2e — seed a `person`, call `get_or_create` (or the endpoint), assert the returned pseudonym is a non-empty UUID-shaped string and equals a second call's result (idempotent), and is NOT the person's name/email.

**MIRROR:** §10.4 (handler), §10.7 (route), `crates/api/api/src/governance/bridge_read.rs:22-46` (the existing Bearer-authed handler).

**GOTCHA (ADR-015 — load-bearing):** this endpoint is the **allocator callsite** — `get_or_create` materialises the opaque per-instance pseudonym the bridge puts in the LiveKit `sub`. The endpoint returns ONLY the pseudonym, never `person_id`/username/email. **Why here:** the bridge has no `DbPool` and no `lemmy_*` deps; the allocator can only run binary-side, so the seam is HTTP. **DoD:** `rg 'get_or_create' crates/api/api/src/governance/bridge_read.rs` returns the callsite. **GOTCHA (Data type):** use `actix_web::web::Data<LemmyContext>` (this file's idiom), NOT `activitypub_federation::config::Data` — mixing them is the latent additive-alias mismatch from PMD retro 907.

**VALIDATE (Windows):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-core-infra-task4-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task4-check.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop` DQ with `commands` = the workspace `cargo-check` + `cargo-clippy --no-deps -- -D warnings` lines AND an `e2e_filter` scoping to the new pseudonym test; commit + push, then **stop**.

### Task 5: RTC Docker sidecars (profile-gated) + AGPL-NOTICE rows

**ACTION:** append LiveKit / lk-jwt-service / Element Call to `services/bridge/docker-compose.yml` under `profiles: ["rtc"]`; add the three AGPL-NOTICE rows.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/docker-compose.yml   # 3 RTC sidecar stanzas, profiles: ["rtc"]
  - AGPL-NOTICE.md                        # M3 RTC stack rows (ADR-011)
```

**IMPLEMENT (file 1 of 2):** `docker-compose.yml` per §10.8 — three stanzas after the `tuwunel` stanza, each with `profiles: ["rtc"]`, on `bridge-net`, with pinned image tags (confirm exact tags against the pulled images at impl). `docker/docker-compose.yml` stays Lemmy-only (do not touch it). lk-jwt host port avoids the bridge's 8080.
**IMPLEMENT (file 2 of 2):** `AGPL-NOTICE.md` per §10.9 — a new `## Additional components — M3 RTC stack` section with `### LiveKit Server` (Apache-2.0), `### lk-jwt-service` (Apache-2.0), `### Element Call` (AGPL-3.0), each in the what-it-is / licence / §13-applicability three-part shape.

**MIRROR:** §10.8 (tuwunel stanza + pilot compose), §10.9 (AGPL rows).

**GOTCHA:** `profiles: ["rtc"]` is the clean-posture mechanism — `docker compose up` with no profile starts NONE of the RTC sidecars (governance-only instance). Element Call is AGPL-3.0 (same licence as us — clean); LiveKit + lk-jwt are Apache-2.0. **MinIO is NOT added here** (Phase 5). Skipping the AGPL rows is a stop-and-ask tripwire (ADR-011).

**VALIDATE (deploy-smoke — NOT a cargo gate; story feeds §16a Story 4):**

```bash
# Default up starts NO RTC sidecars (clean posture):
docker compose -f services/bridge/docker-compose.yml config --services      # EXPECT: rtc services NOT listed
docker compose -f services/bridge/docker-compose.yml --profile rtc config --services  # EXPECT: livekit, lk-jwt-service, element-call listed
# Boot the RTC profile, assert liveness, tear down:
docker compose -f services/bridge/docker-compose.yml --profile rtc up -d livekit lk-jwt-service element-call \
  > .claude/PRPs/debug/m3-core-infra-task5-deploy.log 2>&1
sleep 20
curl -fsS http://localhost:7880/  >/dev/null 2>&1 && echo "LIVEKIT_OK"   # LiveKit :7880 responds
curl -fsS http://localhost:8085/healthz 2>&1 | head -1 && echo "LKJWT_OK"  # lk-jwt /healthz (host 8085 -> container 8080)
docker compose -f services/bridge/docker-compose.yml --profile rtc down
echo "deploy-smoke done"
```

This LEVEL runs **locally** (Docker + image pulls, several min) — keep out of forbidden windows. A cargo command cannot prove a compose stack boots; this is the honest gate. No `validate-pending-laptop` cargo DQ for this task (the deploy-smoke is the gate); the advisor runs it inline at the §16a Story 4 checkpoint.

### Task 6: `rtc_enabled=false` clean-posture e2e story

**ACTION:** add an e2e test proving a governance flow runs unchanged with `rtc_enabled` absent/false and zero RTC side-effects.

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e/governance.rs   # rtc_enabled=false clean-posture test (small additive)
requires:
  - task: 1
    reason: asserts BridgeStatus.rtc_enabled defaults false + governance transition unaffected
```

**IMPLEMENT:** mirror the `messaging_enabled`-absent no-op test (§10.11): bootstrap fixtures, drive a governance case transition with NO `rtc_enabled` row (→ `read_current` returns `None` → `false`), assert the transition completes (`LemmyResult` Ok) and `get_bridge_messaging_status` reports `rtc_enabled=false`. No LiveKit calls are made (the binary has no LiveKit client — assert by construction that the governance path compiles + passes with the RTC stack absent).

**MIRROR:** `crates/server/tests/e2e/governance.rs:~5355-5390`.

**GOTCHA:** R7 — the off-path must be a TESTED signal, not an assumption. The test asserts the governance transition succeeds AND `rtc_enabled` reads `false` by default. Pre-locate the verbatim `old_string` anchor before editing `governance.rs` (it is a large file); confirm the anchor is unique (`grep -c`).

**VALIDATE (Windows e2e; story feeds §16a Story 1):**

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full <clean_posture_test_fn> > .claude/PRPs/debug/m3-core-infra-task6-e2e.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-core-infra-task6-e2e.log   # EXPECT: exit 0, test passes
```

Write a `validate-pending-laptop-e2e` DQ with `commands` = the e2e line above and `e2e_filter` scoping to the clean-posture test; commit + push, then **stop**.

### Task 7: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + lessons + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lessons to `.claude/lessons/feedback_*.md` in the retro commit. Specific signals to capture: bridge Linux cold-build cost, the ADR-015 split-seam design (did it survive review?), the deploy-smoke as a non-cargo gate (new pattern this phase).

---

## 14. Testing strategy

- **crates static (Windows):** `cargo check --workspace --features full`; `cargo clippy --workspace --features full --no-deps -- -D warnings`.
- **bridge static (Linux):** `cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`; `… clippy … --no-deps -- -D warnings`.
- **bridge unit (Linux):** `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml` — `bridge_room` ALTER-idempotency + `livekit_jwt` mint→decode→`sub==pseudonym`.
- **e2e (Windows):** `cargo test --workspace --test e2e --features full <fn>` — pseudonym endpoint (Task 4) + clean-posture (Task 6).
- **migration round-trip:** `bash scripts/brehon/migrate-roundtrip.sh 2026-06-18-000000-0000_seed_rtc_enabled_config`.
- **deploy-smoke (local Docker, NOT cargo):** `docker compose --profile rtc up` → liveness → `down` (Task 5).

## 15. Validation commands (DoD)

> Every command below is written in the exact form the advisor runs at gate 1; all dry-run clean against `phase-m3-core-infra` HEAD. Bridge cargo uses `cargo-linux.sh --manifest-path` (NEVER Windows-local — `ruma-common` E0119). No `rg`-absent / line-count greps (the m3-core-entry-kinds DoD footgun).

### 15.1 crates static analysis (Tasks 1, 4 — Windows)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-core-infra-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.2 crates lint (Tasks 1, 4 — Windows, uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/m3-core-infra-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.3 bridge static + lint + unit (Tasks 2, 3 — Linux, Docker rust:1.95)

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-bridge-check.log 2>&1;  echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings > .claude/PRPs/debug/m3-bridge-clippy.log 2>&1; echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-bridge-test.log 2>&1; echo "exit: $?"  # EXPECT: 0
```

### 15.4 e2e (Tasks 4, 6 — Windows)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/m3-e2e.log 2>&1"
echo "exit: $?"   # EXPECT: 0 (new tests pass; pre-existing unaffected)
```

### 15.5 migration round-trip (Task 1)

```bash
bash scripts/brehon/migrate-roundtrip.sh 2026-06-18-000000-0000_seed_rtc_enabled_config > .claude/PRPs/debug/m3-migrate.log 2>&1
echo "exit: $?"   # EXPECT: 0 (up + redo idempotent; ON CONFLICT DO NOTHING is a no-op on rerun)
```

### 15.6 deploy-smoke (Task 5 — local Docker, NOT a cargo gate)

```bash
docker compose -f services/bridge/docker-compose.yml --profile rtc config --services   # EXPECT: livekit + lk-jwt-service + element-call listed
docker compose -f services/bridge/docker-compose.yml config --services                 # EXPECT: NONE of the rtc services listed (clean posture)
docker compose -f services/bridge/docker-compose.yml --profile rtc up -d livekit lk-jwt-service element-call && sleep 20
curl -fsS http://localhost:7880/ >/dev/null && echo "LIVEKIT_OK"
curl -fsS http://localhost:8085/healthz >/dev/null && echo "LKJWT_OK"
docker compose -f services/bridge/docker-compose.yml --profile rtc down
```

### 15.7 Cross-cutting verification (planner asserts at end-of-phase)

- [ ] R8: Task 1 added NO `CREATE`/`ALTER` (a row-seed under repo-root `migrations/`, not `crates/db_schema/migrations/**`); `schema.rs` unchanged.
- [ ] ADR-015: `rg 'get_or_create' crates/api/api/src/governance/bridge_read.rs` returns the Task-4 callsite; `rg 'sub: identity' services/bridge/src/livekit_jwt.rs` returns the mint `sub`-assignment; `rg -i 'person_id|username|email' services/bridge/src/livekit_jwt.rs` returns nothing.
- [ ] `rtc_enabled` defaults `false` (seed + `.unwrap_or(false)`).
- [ ] AGPL-NOTICE has LiveKit + lk-jwt-service + Element Call rows (ADR-011).
- [ ] `governance_log.rs` UNCHANGED (clarify DQ `a3d0e9941441-065`); zero chain entries emitted this phase.
- [ ] LiveKit env optional — bridge starts without creds (clean posture).
- [ ] `profiles: ["rtc"]` gates all 3 sidecars; `docker/docker-compose.yml` untouched.

## 16. Acceptance criteria

- [ ] Tasks 0–7 completed in dependency order
- [ ] §15.1/15.2 (crates check + clippy) exit 0 (Tasks 1, 4)
- [ ] §15.3 (bridge check + clippy + test) exit 0 (Tasks 2, 3)
- [ ] §15.4 (e2e) — Task 4 + Task 6 tests pass; pre-existing e2e unaffected
- [ ] §15.5 (migration round-trip) exit 0
- [ ] §15.6 (deploy-smoke) — RTC profile boots + tears down; default up starts no RTC
- [ ] §15.7 cross-cutting boxes all ticked
- [ ] §16a stories all `[done]`
- [ ] No edits outside §11; `governance_log.rs` untouched
- [ ] Retro committed (Task 7)
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

## 16a. Stories

### Story 1: A governance-only instance (`rtc_enabled=false`) runs clean with the RTC stack absent

- **Composing tasks:** Task 1, Task 6
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full <clean_posture_test_fn>"`
- **Expected output:** `1 passed; 0 failed`
- **Brief-Scope outputs to verify:** `bridge_read.rs` `BridgeStatus` has `rtc_enabled`; `migrations/2026-06-18-…_seed_rtc_enabled_config/up.sql` seeds `false`; the clean-posture test exists in `governance.rs`.

### Story 2: `rtc_enabled=true` mints a valid pseudonym-JWT for an `always_pseudonym` room

- **Composing tasks:** Task 3 (mint fn), Task 4 (pseudonym endpoint)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml livekit_jwt` AND the Task-4 e2e
- **Expected output:** bridge `livekit_jwt` test passes (`sub == pseudonym`, `iss == api_key`); endpoint e2e passes (returns UUID pseudonym, idempotent, not username)
- **Brief-Scope outputs to verify:** `services/bridge/src/livekit_jwt.rs` exists with `mint_access_token` (`sub: identity`, no `person_id`); `bridge_read.rs` has `get_bridge_actor_pseudonym` calling `get_or_create`; route wired in `routes/src/lib.rs`.

### Story 3: `bridge_room` carries RTC state, provisionable + idempotent on old DBs

- **Composing tasks:** Task 2
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml bridge_room`
- **Expected output:** test passes — fresh DB has 3 new columns; old-shape DB gets ALTERed idempotently
- **Brief-Scope outputs to verify:** `bridge_room.rs::open()` CREATE TABLE has `chair_id`/`queue_state`/`recording_config` + the ALTER-guard loop.

### Story 4: The RTC sidecars deploy under a profile and a no-profile `up` starts none

- **Composing tasks:** Task 5
- **Checkpoint command:** §15.6 deploy-smoke
- **Expected output:** `LIVEKIT_OK` + `LKJWT_OK`; default `config --services` omits the RTC services; `--profile rtc config --services` lists all 3
- **Brief-Scope outputs to verify:** `docker-compose.yml` has 3 `profiles: ["rtc"]` stanzas; `AGPL-NOTICE.md` has the 3 rows; `docker/docker-compose.yml` untouched.

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–8 per expectations)
- [ ] Tasks 1–6 committed (one commit each)
- [ ] §15 validation green at every gate (crates Windows / bridge Linux / e2e / migration / deploy-smoke)
- [ ] §16a Stories 1–4 all `[done]`
- [ ] Retro committed (Task 7)
- [ ] PR opened by BM against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report shows all stories ✓
- [ ] **Linux-compile gate:** `validate-pending-laptop-linux` DQ at `result:pass` (Tasks 2, 3 touch `services/bridge/**` + `Cargo.toml`/`Cargo.lock`) before `bm-pr`
- [ ] Post-merge phase branch retained for retro reads

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| ADR-015 leak — a `person_id`/username reaches the LiveKit `sub` | LOW | HIGH | Split seam: allocator callsite binary-side (Task 4), `sub`=pseudonym param bridge-side (Task 3); §15.7 grep DoD; mint fn has no `person_id` overload |
| Bridge Linux cold build (~10–20 min) stalls the cohort | MED | MED | Warm the `brehon-cargo-registry` volume before the `/auto-phase` loop (Task 0 Probe 6 is the warm-up); bridge tasks serial |
| `rtc_enabled=false` bridge hard-fails on missing LiveKit creds | MED | HIGH | LiveKit env is `Option` (`.ok()`, not `.context()?`); mint fn errors only when called; clean-posture story tests the off-path (R7) |
| Migration treated as a schema change (regen `schema.rs` / `ALTER`) | LOW | MED | §10.1 + Task 1 GOTCHA + R8 + handover tripwire: row-seed only, no `schema.rs` regen |
| Sidecar image tags / lk-jwt health port wrong | MED | LOW | Pin exact tags at impl against pulled images; deploy-smoke asserts liveness, not config-by-faith |
| `actix_web::web::Data` vs federation `Data` mismatch on the new endpoint | LOW | MED | §10.4 + Task 4 GOTCHA name the correct type (mirror existing `bridge_read.rs` handler); PMD retro 907 |
| 8080 port collision (bridge vs lk-jwt) | LOW | LOW | lk-jwt host port 8085 in §10.8; documented |

## 19. Notes

**The ADR-015 seam is split across the HTTP boundary by architectural necessity.** The brief §4.1 says "the JWT mint fn MUST call `actor_pseudonym_helper::get_or_create(pool, person_id)`." Mechanically this cannot live in one bridge-side fn: (a) `get_or_create` takes a `DbPool` and lives in `crates/api/api` — the bridge is a separate process (`brehon-bridge`) with no `DbPool` and no `lemmy_*` deps (`services/bridge/Cargo.toml` has only matrix-sdk/axum/rusqlite/ed25519); (b) the JWT is signed with `LIVEKIT_API_SECRET`, which the brief (task 3 + clarify `a3d0e9941441-063`) puts **bridge-side** in `BridgeConfig` — so the mint/sign step is bridge-side. Therefore the seam splits: the **allocator call** lands in the Brehon `get_bridge_actor_pseudonym` endpoint (Task 4); the **JWT `sub`-assignment** lands in the bridge `mint_access_token` (Task 3). Both are present + grep-able in Phase 1 (handover §4.3 DoD satisfied). The bridge→endpoint fetch that connects them end-to-end is **Phase 3** (town-hall provisioning) — Phase 1 "does NOT run a town hall." This mirrors the project's pre-landed-const exemption pattern (a seam half landed ahead of its caller). **Pre-seeded as a resolved planner DQ for advisor gate-1 review.**

**Complexity 13 > 8 Sonnet threshold; proceed-as-one per user decision 2026-06-18** (brief §2). Inflation is the e2e `+3` weight on two small additive tests, not 8000-line-file hang risk. Pre-seeded as a resolved planner DQ.

**Three validation surfaces, three toolchains** — crates (Windows bat wrappers + `--features full`), bridge (`cargo-linux.sh --manifest-path`, Linux-only), sidecars (deploy-smoke, NOT cargo). The §15 deploy-smoke is a NEW gate shape for this project — a compose `up`/liveness/`down` that no cargo command can substitute.

## 20. Confidence score

- **Plan correctness:** 8/10 — the seam-split is the one judgment call (surfaced as a DQ); everything else mirrors shipped patterns.
- **Cargo budget:** 9/10 — no daemon cargo; bridge cold-build is the only time cost, mitigated by warm-up.
- **Test coverage:** 8/10 — both PRD Phase-1 success signals (`rtc_enabled=false` clean + `rtc_enabled=true` mints pseudonym-JWT) are §16a stories with green checkpoints; the end-to-end resolve→mint flow is deferred to Phase 3 by scope.
