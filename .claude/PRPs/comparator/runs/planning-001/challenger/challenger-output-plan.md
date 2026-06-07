# Plan: M2-late — B-publish Sanction Propagation

## Summary

M2-late Phase 6 adds the ADR-016 **B-publish** path: when `submit_jury_vote` creates a sanction and appends the existing `sanction_created` governance-log row, the Brehon binary derives a universal `SanctionEventPayload`, records a `sanction_event`, and delivers it by HTTP webhook to active `sanction_subscriber` rows. The only in-scope subscriber is the Matrix bridge at `POST /brehon/sanction-event`, authenticated with the existing `BRIDGE_CALLBACK_SECRET`. Delivery is fire-and-forget and outside the jury-vote transaction; the main quorum path remains bounded by DB work only.

## Source

- Brief: [`.claude/PRPs/briefs/m2-late-planning-1.md`](../briefs/m2-late-planning-1.md) — M2-late Phase 6 only.
- PRD: [`.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md`](../prds/m2-governance-triggered-rooms.prd.md) §Implementation Phases, Phase 6 (B-publish).
- Relevant design docs:
  - [`docs/brehon-law-inspired-network/04-data-model-and-api.md`](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) §§2, 3, 8, 10, 11, 14.
  - [`docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md`](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) §§3-4, 10.
  - [`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) ADR-004, ADR-008, ADR-010, ADR-011, ADR-014, ADR-015, ADR-016, OQ-ADR016-02, OQ-ADR016-04.
  - [`docs/brehon-law-inspired-network/06-security-and-threat-model.md`](../../../docs/brehon-law-inspired-network/06-security-and-threat-model.md) §§2.2.2, 6.1.
  - [`docs/brehon-law-inspired-network/07-operations-and-federation.md`](../../../docs/brehon-law-inspired-network/07-operations-and-federation.md) §5.6.
- Prior M2 plans: [`m2-core-transition-hook.plan.md`](m2-core-transition-hook.plan.md), [`m2-rooms-a.plan.md`](m2-rooms-a.plan.md).

## Problem Statement

Today, a quorum decision may insert a `sanction` row and append `governance_log.entry_kind = "sanction_created"`, but no external app is told. Matrix rooms can be provisioned by M2-core/M2-rooms-a, but a Matrix participant subject to a Brehon sanction is not muted, restricted, or otherwise locally constrained. M2-late must bridge that gap without making the Brehon binary an app super-admin and without blocking the live `submit_jury_vote` transaction on network I/O.

## Solution Statement

Add a narrow publish-subscribe path:

1. **Schema + enum** — add `SanctionKind` (`prevent_post`, `mute_voice`, `hide_content`, `restrict_reach` on the wire; PascalCase PG enum tokens) plus `sanction_event` and `sanction_subscriber` tables under the existing root `migrations/` convention. CODE WINS over the brief's stale `crates/db_schema/migrations/` path.
2. **Publisher** — add `sanction_kind_map.rs` and `sanction_publisher.rs` in `crates/api/api/src/governance/`. The publisher maps `SanctionAction` exhaustively to `Option<SanctionKind>`, resolves `target_person_id` to `actor_pseudonym.pseudonym`, builds the universal payload, inserts `sanction_event`, POSTs to each active subscriber with `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>`, and appends `sanction_published` or `sanction_event_delivery_failed` governance-log entries outside the main transaction.
3. **Hook** — change `submit_jury_vote` internally to capture the inserted `Sanction` row and the returned `sanction_created.entry_hash`, return that as private post-transaction work, then `tokio::spawn` the publisher after `conn.run_transaction(...).await?` commits. Appeal votes remain out of scope: `process_appeal_vote` writes no sanction row and gets no spawn site.
4. **Bridge subscriber** — add `services/bridge/src/sanction_handler.rs` and `POST /brehon/sanction-event`. It verifies `BRIDGE_CALLBACK_SECRET`, deserializes the payload, maps `subject_brehon_actor_id` to the existing puppet MXID, applies Matrix power-level changes across bridge-managed rooms, and returns `{applied, reason, applied_at}`. No `services/bridge` workspace membership; root `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` must stay `0`.

## Metadata

| Field | Value |
|---|---|
| Type | CROSS_CUTTING + SCHEMA + HANDLER |
| Complexity | HIGH |
| Crates Affected | `lemmy_db_schema_file`, `lemmy_db_schema`, `lemmy_api`, `lemmy_server`; workspace-excluded `services/bridge` |
| v0/M Step | Post-v1 M2-late Phase 6 (ADR-016 B-publish); closest v0 analogue is [05 §4 Step 6](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) outbound signals |
| Dependencies | M1 shipped; M2-core hook + m2-rooms-a shipped; OQ-ADR016-02 and OQ-ADR016-04 resolved 2026-06-07 |
| Estimated Tasks | 8 implementation tasks + Task 0 audit |

---

## Flow Design

### Before State

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║ BEFORE                                                                      ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ Juror vote #3 reaches threshold                                             ║
║   → submit_jury_vote transaction                                            ║
║     → INSERT sanction                                                       ║
║     → append governance_log("sanction_created")                            ║
║     → flip case status / public log / reputation / federation AP if needed  ║
║   → COMMIT                                                                  ║
║   → response                                                                ║
║                                                                              ║
║ Matrix bridge has rooms, but no sanction event reaches it.                  ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### After State

```text
╔══════════════════════════════════════════════════════════════════════════════╗
║ AFTER                                                                       ║
╠══════════════════════════════════════════════════════════════════════════════╣
║ Juror vote #3 reaches threshold                                             ║
║   → submit_jury_vote transaction                                            ║
║     → INSERT sanction RETURNING Sanction                                    ║
║     → append governance_log("sanction_created") RETURNING entry_hash        ║
║     → all existing case/status/public-log/reputation side effects           ║
║   → COMMIT                                                                  ║
║   → tokio::spawn(sanction_publisher::publish(seed))                         ║
║       → map SanctionAction → Option<SanctionKind>                           ║
║       → target_person_id? else debug skip                                   ║
║       → actor_pseudonym_helper::get_or_create(target_person_id)             ║
║       → INSERT sanction_event                                               ║
║       → for active sanction_subscriber: POST callback_url                   ║
║          Authorization: Bearer <BRIDGE_CALLBACK_SECRET>                     ║
║       → append sanction_published OR sanction_event_delivery_failed         ║
║                                                                              ║
║ Bridge: POST /brehon/sanction-event → Matrix power-level translation → ACK  ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### Entrypoint Changes

| Endpoint / Entrypoint | Before | After | Impact |
|---|---|---|---|
| `submit_jury_vote` internal quorum path | Inserts sanction/logs decision only | Captures sanction + log hash, spawns publisher after commit | No network inside DB transaction; exactly one spawn site |
| `services/bridge POST /brehon/sanction-event` | Did not exist | Bridge subscriber receives universal sanction event | Matrix app plane enforces Brehon sanction primitive |
| `BRIDGE_SANCTION_CALLBACK_URL` startup env | Did not exist | If set, Brehon seeds one active subscriber row idempotently | No user-facing subscriber-registration endpoint |
| `POST /api/v4/governance/*` routes | Unchanged | Unchanged | No new Brehon user/admin endpoint in M2-late |

### Redaction / Pseudonym Touchpoints

- Event subject is **only** `actor_pseudonym.pseudonym`; never `person.name`, `local_user.email`, or `ap_id` (ADR-015).
- `governance_log::append` still scrubs payload JSON through `scrub_json` before insert.
- `sanction_event.subject_actor_pseudonym` stores an opaque pseudonym string, not a direct identifier.

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `.claude/PRPs/briefs/m2-late-planning-1.md` | all | Authoritative phase boundary and watchpoints |
| P0 | `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` | ADR-016 + OQ-ADR016-02/-04 | B-publish schema, auth, sanction primitives, Phase 7 out-of-scope |
| P0 | `crates/api/api/src/governance/submit_jury_vote.rs` | 126-180, 193-310, 454-485, 778-858, 1049-1090 | Transaction boundary, sanction insert/log point, AP outbound ordering, appeal non-spawn proof, exhaustive decision map |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 117, 239-333 | Entry-kind const style and `append(...)` writer invariants |
| P0 | `crates/api/api/src/governance/governance_log.rs` | 39-129 | Shim re-export style and bridge callback wrapper pattern |
| P0 | `crates/api/api/src/governance/actor_pseudonym_helper.rs` | 1-80 | ADR-015 `get_or_create` allocator; use for event subject |
| P0 | `crates/db_schema_file/src/enums.rs` | 542-577 | `SanctionScope` / `SanctionAction` DbEnum pattern to mirror for `SanctionKind` |
| P0 | `crates/db_schema/src/source/governance/sanction.rs` | 1-47 | Diesel model + InsertForm pattern to mirror for `SanctionEvent` |
| P0 | `services/bridge/src/appservice.rs` | 33-90, 197-221 | `AppState`, auth middleware shape, router wiring; sanction route must not be trapped behind HS token only |
| P0 | `services/bridge/src/provision.rs` | 10-31 | reqwest + Matrix client API call shape to mirror for power-level PUTs |
| P0 | `services/bridge/src/bridge_room.rs` | 1-56 | Bridge-local rusqlite state; add active-room listing helper |
| P1 | `crates/server/src/lib.rs` | 210-247 | Startup context point where subscriber seed helper can run |
| P1 | `.claude/rules/governance-log-entry-kind-registry.md` | all | Advisor-owned registry update rule for new entry kinds |
| P1 | `crates/server/tests/e2e/governance.rs` | 1602-1720, 1870-1935, 4041-4450, 5249-5328 | Existing e2e context, quorum-driving, federation publish, and hash-chain callback test patterns |

**External Documentation:** none required. All needed APIs and crate versions are already in the workspace (`diesel 2.3.10`, `diesel-derive-enum 2.1.0`, `reqwest 0.13.4` workspace / `reqwest 0.12` bridge, `tokio 1.52.3`, `serde_json 1.0.150`).

---

## Codebase Discovery Table

| Category | File:Lines | Pattern | Snippet |
|---|---|---|---|
| MIGRATION_ENUM | `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql:60-68` | PG enum uses PascalCase verbatim tokens | `CREATE TYPE sanction_action AS ENUM ('Label', 'VisibilityReduction', ...);` |
| MIGRATION_TABLE | `migrations/2026-04-15-100100-0000_add_governance_core/up.sql:33-45` | Core table with FKs/defaults | `CREATE TABLE sanction (... action sanction_action NOT NULL, starts_at TIMESTAMPTZ NOT NULL DEFAULT now(), active BOOLEAN NOT NULL DEFAULT TRUE);` |
| SCHEMA_SQL_TYPE | `crates/db_schema_file/src/schema.rs:130-138` | Diesel sql_types struct for PG enum | `#[diesel(postgres_type(name = "sanction_action"))] pub struct SanctionAction;` |
| MODEL | `crates/db_schema/src/source/governance/sanction.rs:20-47` | `Identifiable, Queryable, Selectable` read struct + `Insertable` form | `pub struct Sanction { pub id: SanctionId, ... }` |
| LOG_APPEND | `crates/db_schema/src/source/governance/governance_log.rs:262-333` | `append(pool, kind, payload, actor_pseudonym)` scrubs + signs in tx | `let form = GovernanceLogInsertForm { entry_kind, payload: scrub_json(&payload), actor_pseudonym };` |
| HANDLER_TX | `crates/api/api/src/governance/submit_jury_vote.rs:126-180` | Pre-tx read, `run_transaction`, post-tx hook | `let outcome = conn.run_transaction(...).await?;` |
| SANCTION_INSERT | `crates/api/api/src/governance/submit_jury_vote.rs:454-485` | Sanction insert + `sanction_created` append point | `insert_into(sanction::table).values(&sanction_form).execute(conn).await?;` |
| APPEAL_SKIP | `crates/api/api/src/governance/submit_jury_vote.rs:1049-1067` | Appeal path explicitly no sanction/federation outbound | comment: `No sanction row, no federation outbound.` |
| AUTH | `crates/api/api/src/governance/bridge_auth.rs:6-18` | Existing bearer secret verification | `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` |
| ROUTE | `crates/api/routes/src/lib.rs:481-498` | Governance route registration under `/api/v4/governance` | `.route("/room-event", post().to(handle_room_event))` |
| BRIDGE_ROUTE | `services/bridge/src/appservice.rs:197-221` | axum route + `tokio::spawn` fire-and-forget | `.route("/brehon/room-event", post(handle_room_event))` |
| BRIDGE_HTTP | `services/bridge/src/provision.rs:10-31` | Bridge reqwest client API pattern | `.post(&url).bearer_auth(&config.as_token).json(&body).send().await?` |
| TEST | `crates/server/tests/e2e/governance.rs:4041-4450` | Drive quorum and assert exactly-once side effect | `sanction_notice_round_trip` |

---

## Patterns to Mirror

### Diesel enum + schema pattern

```rust
// SOURCE: crates/db_schema_file/src/enums.rs:542-577
#[derive(EnumString, Display, Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Hash, DbEnum)]
#[ExistingTypePath = "crate::schema::sql_types::SanctionAction"]
#[DbValueStyle = "verbatim"]
pub enum SanctionAction {
  Label,
  VisibilityReduction,
  TemporaryRestriction,
  ContentRemoval,
  CommunityExclusion,
  InstanceSuspension,
  FederationQuarantineRecommendation,
  Restoration,
}
```

`SanctionKind` must mirror the Diesel side (`ExistingTypePath`, `DbValueStyle = "verbatim"`) while adding `#[serde(rename_all = "snake_case")]` for the webhook wire payload.

### Governance log append pattern

```rust
// SOURCE: crates/db_schema/src/source/governance/governance_log.rs:262-333
pub async fn append(
  pool: &mut DbPool<'_>,
  entry_kind: &str,
  payload: serde_json::Value,
  actor_pseudonym: Option<String>,
) -> LemmyResult<GovernanceLog> {
  // reads previous hash, scrubs payload, inserts, signs, returns row
}
```

The publisher appends only after the sanction transaction commits. Delivery failures are non-fatal to `submit_jury_vote`.

### Post-transaction side-effect shape

```rust
// SOURCE: crates/api/api/src/governance/submit_jury_vote.rs:126-180
let outcome = conn
  .run_transaction(|conn| Box::pin(process_vote(..., conn)))
  .await?;
// existing room-transition notification happens after this boundary.
```

The sanction publisher must follow this boundary: no HTTP call and no `tokio::spawn` inside `process_vote`.

### Pseudonym allocation pattern

```rust
// SOURCE: crates/api/api/src/governance/actor_pseudonym_helper.rs:40-80
pub async fn get_or_create(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<String> {
  // reads or inserts UUIDv4 pseudonym, handles unique-race retry
}
```

Use this for the event subject. Do not serialize `PersonId`, usernames, emails, or actor URLs into the event or governance log.

### Bridge route pattern

```rust
// SOURCE: services/bridge/src/appservice.rs:211-221
pub fn router(state: Arc<AppState>) -> Router {
  Router::new()
    .route("/transactions/:txn_id", put(handle_transaction))
    .route("/users/:user_id", get(handle_query_user))
    .route("/rooms/:room_alias", get(handle_query_room))
    .route("/brehon/room-event", post(handle_room_event))
    .route_layer(axum::middleware::from_fn_with_state(..., hs_token_auth))
    .with_state(state)
}
```

Add `/brehon/sanction-event` **after** the HS-token `route_layer` or split routers so this Brehon→bridge route is authenticated by `BRIDGE_CALLBACK_SECRET`, not by the Matrix homeserver token.

---

## Files to Change

| File | Action | Justification |
|---|---|---|
| `migrations/2026-06-07-000000-0000_add_sanction_publish_tables/up.sql` | ADD | `sanction_kind`, `sanction_event`, `sanction_subscriber` |
| `migrations/2026-06-07-000000-0000_add_sanction_publish_tables/down.sql` | ADD | Drop tables then enum in reverse order |
| `crates/db_schema_file/src/enums.rs` | UPDATE | Add Rust `SanctionKind` DbEnum |
| `crates/db_schema_file/src/schema.rs` | UPDATE | Add `sql_types::SanctionKind`, table declarations, joinable/allow-table wiring |
| `crates/db_schema/src/newtypes.rs` | UPDATE | Add `SanctionEventId`, `SanctionSubscriberId` |
| `crates/db_schema/src/source/governance/sanction_event.rs` | ADD | Diesel read/insert structs for event rows |
| `crates/db_schema/src/source/governance/sanction_subscriber.rs` | ADD | Diesel read/insert structs + active-list helper + idempotent seed helper |
| `crates/db_schema/src/source/governance/mod.rs` | UPDATE | Export the two new modules |
| `crates/db_schema/src/source/governance/governance_log.rs` | UPDATE | Add `ENTRY_KIND_SANCTION_PUBLISHED`, `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED` |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Re-export the two new entry-kind consts in alphabetical order |
| `crates/api/api/src/governance/sanction_kind_map.rs` | ADD | Exhaustive `SanctionAction` → `Option<SanctionKind>` mapping |
| `crates/api/api/src/governance/sanction_publisher.rs` | ADD | Build payload, insert event, send webhooks, append success/failure logs |
| `crates/api/api/src/governance/mod.rs` | UPDATE | Add `sanction_kind_map`, `sanction_publisher` modules |
| `crates/api/api/src/governance/submit_jury_vote.rs` | UPDATE | Capture sanction + log hash; enqueue publisher after commit; late votes remain idempotent |
| `crates/server/src/lib.rs` | UPDATE | Startup seed from `BRIDGE_SANCTION_CALLBACK_URL` |
| `services/bridge/src/sanction_handler.rs` | ADD | Bridge subscriber endpoint, ACK, Matrix power-level translation |
| `services/bridge/src/appservice.rs` | UPDATE | Route `/brehon/sanction-event` with correct auth boundary |
| `services/bridge/src/bridge_room.rs` | UPDATE | Add active-room listing helper used by sanction handler |
| `services/bridge/src/main.rs` | UPDATE | `mod sanction_handler;` |
| `services/bridge/docker-compose.yml` | UPDATE | Commented dev env for `BRIDGE_CALLBACK_SECRET`, `BRIDGE_SANCTION_CALLBACK_URL` if bridge service block is kept as docs |
| `crates/server/tests/e2e/governance.rs` | UPDATE | Workspace e2e for publish path and skip/idempotency cases |
| `services/bridge/tests/sanction_event.rs` | ADD | Bridge route auth/deserialization test and ignored Matrix integration smoke |

**Advisor-owned side file:** `.claude/rules/governance-log-entry-kind-registry.md` must be updated for the two new entry kinds, but do not put that edit in a Junior impl task unless the advisor explicitly assigns rule-file ownership.

## Files Explicitly Out of Scope

- No edits to root `CLAUDE.md`.
- No edits to `.claude/decision-queue.json`, `.claude/runlog/`, or Junior daemon state.
- No new user/admin Brehon HTTP endpoint for subscriber registration.
- No Phase 7 / B-actor portable ID table, claim flow, or Matrix-login separation.
- No retry scheduler / durable outbox worker.
- No full OQ-ADR016-04 per-app ACK callback endpoint in the Brehon binary; M2-late handles synchronous HTTP ACKs and delivery success/failure log entries only.

---

## Step-by-Step Tasks

### Task 0: Pre-flight audit — reconcile plan against current HEAD

**Purpose:** catch drift before changing Rust or SQL.

**Actions:**

1. Re-run these searches and paste results into the task notes, not the commit message:
   - `rg -n "SanctionKind|sanction_event|sanction_subscriber|sanction_published|sanction_event_delivery_failed|BRIDGE_SANCTION_CALLBACK_URL|/brehon/sanction-event" crates services migrations .claude docs`
   - `rg -n "insert_into\(sanction::table\)|ENTRY_KIND_SANCTION_CREATED|sanction_created|process_appeal_vote|late vote|already" crates/api/api/src/governance/submit_jury_vote.rs crates/server/tests/e2e/governance.rs`
   - `rg -n "route_layer|/brehon/room-event|BRIDGE_CALLBACK_SECRET|bridge_room" services/bridge/src services/bridge/tests`
2. Confirm live migrations are under root `migrations/`, not `crates/db_schema/migrations/`.
3. Confirm no existing `SanctionKind` or subscriber table exists.
4. Confirm `services/bridge` remains in root workspace `exclude`, not `members`.

**Validation:** no cargo. This is read-only.

**Stop if:** `SanctionKind` or `sanction_event` already exists; re-author this plan from the existing implementation instead of duplicating it.

---

### Task 1: Add schema — `sanction_kind`, `sanction_event`, `sanction_subscriber`

**Files:**

- `migrations/2026-06-07-000000-0000_add_sanction_publish_tables/up.sql`
- `migrations/2026-06-07-000000-0000_add_sanction_publish_tables/down.sql`
- `crates/db_schema_file/src/enums.rs`
- `crates/db_schema_file/src/schema.rs`
- `crates/db_schema/src/newtypes.rs`

**Implementation requirements:**

1. SQL migration:
   - `CREATE TYPE sanction_kind AS ENUM ('PreventPost', 'MuteVoice', 'HideContent', 'RestrictReach');`
   - `CREATE TABLE sanction_event (`
     - `id SERIAL PRIMARY KEY`
     - `sanction_id INTEGER NOT NULL REFERENCES sanction(id) ON DELETE CASCADE`
     - `sanction_kind sanction_kind NOT NULL`
     - `subject_actor_pseudonym TEXT NOT NULL`
     - `effective_from TIMESTAMPTZ NOT NULL`
     - `effective_until TIMESTAMPTZ NULL`
     - `governance_log_entry_hash TEXT NOT NULL UNIQUE`
     - `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
     - `)`
   - `CREATE INDEX sanction_event_sanction_id_idx ON sanction_event (sanction_id);`
   - `CREATE INDEX sanction_event_subject_idx ON sanction_event (subject_actor_pseudonym);`
   - `CREATE TABLE sanction_subscriber (`
     - `id SERIAL PRIMARY KEY`
     - `callback_url TEXT NOT NULL UNIQUE`
     - `active BOOLEAN NOT NULL DEFAULT TRUE`
     - `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`
     - `)`
   - `CREATE INDEX sanction_subscriber_active_idx ON sanction_subscriber (active) WHERE active;`
2. Down migration drops indexes/tables then `DROP TYPE sanction_kind`.
3. `SanctionKind` Rust enum in `crates/db_schema_file/src/enums.rs`:
   - derive the same traits as `SanctionAction`.
   - use `ExistingTypePath = "crate::schema::sql_types::SanctionKind"`.
   - use `DbValueStyle = "verbatim"`.
   - add `#[serde(rename_all = "snake_case")]` so webhook JSON emits `prevent_post`, `mute_voice`, `hide_content`, `restrict_reach`.
4. `schema.rs` must add:
   - `sql_types::SanctionKind` for `sanction_kind`.
   - `diesel::table!` blocks for `sanction_event` and `sanction_subscriber`.
   - `diesel::joinable!(sanction_event -> sanction (sanction_id));`
   - both tables in `allow_tables_to_appear_in_same_query!`.
5. `newtypes.rs` must add `SanctionEventId(pub i32)` and `SanctionSubscriberId(pub i32)` near related governance newtypes.

**Validation commands:**

```bash
scripts/brehon/migrate-roundtrip.sh > .pi/m2-late-migration-roundtrip.log 2>&1
scripts/brehon/cargo-check.sh -p lemmy_db_schema_file > .pi/m2-late-db-schema-file-check.log 2>&1
```

Read only the tail/log errors; do not paste full cargo output.

**Gotchas:**

- Use root `migrations/`. The brief's `crates/db_schema/migrations/` path is stale.
- Keep PG enum tokens PascalCase to match the repo's `DbValueStyle = "verbatim"`; use serde snake_case for webhook strings.
- Do not add a table named only `subscriber`; keep the governance-specific `sanction_subscriber`.

---

### Task 2: Add Diesel models and active-subscriber helper

**Files:**

- `crates/db_schema/src/source/governance/sanction_event.rs`
- `crates/db_schema/src/source/governance/sanction_subscriber.rs`
- `crates/db_schema/src/source/governance/mod.rs`

**Implementation requirements:**

1. `SanctionEvent` mirrors `Sanction`:
   - derives `PartialEq, Serialize, Deserialize, Debug, Clone`.
   - `#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]`.
   - `diesel(table_name = sanction_event)` and `check_for_backend(diesel::pg::Pg)`.
   - fields match Task 1; `sanction_kind: SanctionKind`, `sanction_id: SanctionId`, `id: SanctionEventId`.
2. `SanctionEventInsertForm` derives `Insertable`, no `AsChangeset`.
3. `SanctionSubscriber` mirrors `GovernanceMessagingConfig` helper style:
   - read struct and insert form.
   - `SanctionSubscriber::list_active(pool) -> LemmyResult<Vec<Self>>` using `active.eq(true)`.
   - `SanctionSubscriber::insert_bridge_if_configured(pool, callback_url: &str) -> LemmyResult<Option<Self>>` or equivalent; use `ON CONFLICT (callback_url) DO NOTHING` and return `Ok(None)` on conflict.
4. Export modules from `source/governance/mod.rs`.

**Validation command:**

```bash
scripts/brehon/cargo-check.sh -p lemmy_db_schema > .pi/m2-late-db-schema-check.log 2>&1
```

**Gotchas:**

- Gate Diesel query impls behind `#[cfg(feature = "full")]`, mirroring existing models.
- Use `lemmy_utils::error::{LemmyErrorExt, LemmyErrorType, LemmyResult}` conventions; no `unwrap()`/`expect()` outside tests.

---

### Task 3: Add log entry kinds and sanction-action mapping

**Files:**

- `crates/db_schema/src/source/governance/governance_log.rs`
- `crates/api/api/src/governance/governance_log.rs`
- `crates/api/api/src/governance/sanction_kind_map.rs`
- `crates/api/api/src/governance/mod.rs`

**Implementation requirements:**

1. Add constants:
   - `ENTRY_KIND_SANCTION_PUBLISHED: &str = "sanction_published"`
   - `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED: &str = "sanction_event_delivery_failed"`
2. Re-export both from the API shim in alphabetical position.
3. Add `sanction_kind_map.rs` with an exhaustive match:

| `SanctionAction` | `SanctionKind` / skip | Rationale |
|---|---|---|
| `Label` | `RestrictReach` | Advisory label affects prominence/reach |
| `VisibilityReduction` | `RestrictReach` | Direct reach restriction |
| `TemporaryRestriction` | `PreventPost` | Time-bounded posting restriction |
| `ContentRemoval` | `HideContent` | Hide/remove content in app plane |
| `CommunityExclusion` | `PreventPost` | No posting in governed room/community |
| `InstanceSuspension` | `PreventPost` | Strongest local posting restriction |
| `FederationQuarantineRecommendation` | `None` | Not a local Matrix primitive in m2-late |
| `Restoration` | `None` | No B-publish enforcement; future unpublish/restore is M3+ |

4. Add a unit test or e2e helper assertion that the match mentions all eight variants. Rust exhaustiveness is the primary guard; the test documents the mapping.

**Validation command:**

```bash
scripts/brehon/cargo-check.sh -p lemmy_api > .pi/m2-late-api-map-check.log 2>&1
```

**Advisor follow-up:** update `.claude/rules/governance-log-entry-kind-registry.md` with the two new constants and bump the total count. Per the brief, this is advisor-side ownership, not a Junior impl-task file unless explicitly assigned.

---

### Task 4: Implement publisher + startup subscriber seed

**Files:**

- `crates/api/api/src/governance/sanction_publisher.rs`
- `crates/api/api/src/governance/mod.rs`
- `crates/server/src/lib.rs`
- `services/bridge/docker-compose.yml` (commented dev env only, if bridge service block remains as operator docs)

**Implementation requirements:**

1. Define private publisher seed data containing at least:
   - `sanction: Sanction`
   - `governance_log_entry_hash: String` (hex/text value returned by `governance_log::append`)
2. Define webhook payload:

```json
{
  "sanction_kind": "prevent_post | mute_voice | hide_content | restrict_reach",
  "subject_brehon_actor_id": "<actor_pseudonym.pseudonym>",
  "effective_from": "<RFC3339>",
  "effective_until": "<RFC3339|null>",
  "governance_log_entry_hash": "<hex hash>"
}
```

3. `enqueue_sanction_event(context, seed)` must be fire-and-forget:
   - Clone `LemmyContext` and `tokio::spawn`.
   - Any error logs with `tracing::warn!` and is swallowed.
4. Inner `publish_sanction_event` must:
   - map action to `Option<SanctionKind>`; `None` → `tracing::debug!` skip, no event, no subscriber POST.
   - require `sanction.target_person_id`; missing target → `tracing::debug!` skip.
   - resolve pseudonym with `actor_pseudonym_helper::get_or_create`.
   - insert `sanction_event` once for the given `governance_log_entry_hash`. If a duplicate unique violation occurs, treat as idempotent duplicate and skip duplicate POST/log append.
   - read `SanctionSubscriber::list_active`.
   - if no subscribers, debug-log and return.
   - read `BRIDGE_CALLBACK_SECRET`; if missing/empty, append `sanction_event_delivery_failed` with reason `missing_bridge_callback_secret`, then return.
   - POST the payload to each active `callback_url` with bearer auth.
   - On any 2xx response, append `sanction_published` including `sanction_event_id`, `subscriber_id`, `callback_url` host/path only if safe, `status_code`, and parsed ACK body if JSON.
   - On transport error or non-2xx, append `sanction_event_delivery_failed` with `status_code`/error string.
5. Startup seed in `crates/server/src/lib.rs`:
   - after `LemmyContext` exists and before server bind/listen, read `BRIDGE_SANCTION_CALLBACK_URL`.
   - absent/empty → no-op.
   - present → call idempotent subscriber insert (`ON CONFLICT DO NOTHING`).
   - failure should be loud enough to diagnose (`tracing::warn!` or return error if DB unavailable at startup); do not panic.

**Validation commands:**

```bash
scripts/brehon/cargo-check.sh -p lemmy_api > .pi/m2-late-api-publisher-check.log 2>&1
scripts/brehon/cargo-check.sh -p lemmy_server > .pi/m2-late-server-seed-check.log 2>&1
```

**Gotchas:**

- Never pass a transaction connection into the spawned task.
- Do not use raw `PersonId` or username in any payload/log.
- Treat HTTP 200 with `{ "applied": false, "reason": "not_applicable:..." }` as `sanction_published` from the Brehon delivery perspective; the subscriber accepted and applied its local translation decision.

---

### Task 5: Wire `submit_jury_vote` post-commit enqueue

**Files:**

- `crates/api/api/src/governance/submit_jury_vote.rs`

**Implementation requirements:**

1. Replace the sanction insert `.execute(conn)` at the quorum-created site with `RETURNING Sanction`.
2. Capture the returned `GovernanceLog` from the existing `governance_log::append(... ENTRY_KIND_SANCTION_CREATED ...)` call. Use its `entry_hash` as the event hash.
3. Introduce a private outcome wrapper for `process_vote`, for example:
   - `response: SubmitJuryVoteResponse`
   - `sanction_publish_seed: Option<SanctionPublishSeed>`
4. Keep the public handler response unchanged: `LemmyResult<Json<SubmitJuryVoteResponse>>`.
5. After `run_transaction(...).await?` commits and after any existing post-commit case-transition notification, call `sanction_publisher::enqueue_sanction_event` if the seed is present.
6. Preserve existing idempotency:
   - Late votes after quorum must not create a second sanction.
   - Therefore they must not create a second `sanction_event`, second webhook POST, or second `sanction_published` row.
7. Do **not** add any sanction publisher call in `process_appeal_vote`; line-commented current behavior says appeal votes do not create sanction rows or federation outbound side effects.

**Validation command:**

```bash
scripts/brehon/cargo-check.sh -p lemmy_api > .pi/m2-late-submit-vote-check.log 2>&1
```

**Regression tests to add in Task 7:** quorum publish once; late vote does not publish again; restoration/federation-quarantine action skips publish.

---

### Task 6: Implement bridge subscriber endpoint

**Files:**

- `services/bridge/src/sanction_handler.rs`
- `services/bridge/src/appservice.rs`
- `services/bridge/src/bridge_room.rs`
- `services/bridge/src/main.rs`

**Implementation requirements:**

1. Add `SanctionEventPayload` and `SanctionAck` bridge-side structs matching the wire schema:
   - `sanction_kind` snake_case enum.
   - `subject_brehon_actor_id` string.
   - `effective_from`, `effective_until`, `governance_log_entry_hash`.
   - ACK response: `applied: bool`, `reason: String`, `applied_at: DateTime<Utc>`.
2. Auth:
   - Verify `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` using `state.config.bridge_callback_secret`.
   - Missing/mismatch → `401 Unauthorized`, no Matrix call.
3. Routing:
   - Add `POST /brehon/sanction-event` in `appservice::router` so it is not only protected by the Matrix homeserver `hs_token_auth` layer.
   - The cleanest minimal change is to add this route **after** the existing `.route_layer(...)` call or split Brehon routes into a separate router with per-handler secret verification.
4. Room lookup:
   - Add `bridge_room::list_active(conn) -> Result<Vec<BridgeRoom>>` (or minimal tuple equivalent) to enumerate bridge-managed rooms.
   - Current `bridge_room` schema has no subject column, so m2-late applies sanctions for the subject puppet MXID across all active bridge-managed rooms. Matrix `m.room.power_levels.users[subject_mxid]` is per-user and harmless for rooms where the puppet is absent. If subject-scoped room indexing is required, that is a scope expansion needing a separate bridge-room schema/data-source plan.
5. Translation table:

| `sanction_kind` | Matrix action | ACK behavior |
|---|---|---|
| `prevent_post` | set subject puppet power level below `events_default` / message-send threshold in all active rooms | `applied=true` if at least one room update succeeds; else `applied=false`, `reason="not_applicable:no_rooms"` |
| `restrict_reach` | same power-level downgrade as `prevent_post` for room posting/reaction reach in m2-late | same as above |
| `hide_content` | no generic Matrix delete/redact target exists in the universal payload; return accepted not-applicable | `applied=false`, `reason="not_applicable:no_content_reference"` |
| `mute_voice` | if no voice-room/call primitive exists, accepted not-applicable | `applied=false`, `reason="not_applicable:no_voice_rooms"` |

6. Matrix calls:
   - Use raw reqwest patterns from `provision.rs`; no Matrix SDK dependency.
   - GET current `m.room.power_levels`, mutate `users[subject_mxid]`, PUT updated state.
   - Continue across rooms if one room update fails; return 200 with partial reason if at least one succeeds, or 500 only for a systemic failure (e.g. invalid config / homeserver unreachable before any room attempt).
7. Subject puppet:
   - Use the existing puppet naming/map convention from `services/bridge/src/puppet.rs`; do not invent a new public identifier.

**Validation commands:**

```bash
cd services/bridge && cargo check > ../../.pi/m2-late-bridge-check.log 2>&1
cd services/bridge && cargo clippy -- -D warnings > ../../.pi/m2-late-bridge-clippy.log 2>&1
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma' > .pi/m2-late-zero-matrix-deps.txt
```

The matrix-deps count must be `0`.

**Gotchas:**

- Do not put `services/bridge` into the root workspace `members`.
- Do not reuse `HS_TOKEN`/`AS_TOKEN` for Brehon→bridge sanction webhook auth; use `BRIDGE_CALLBACK_SECRET`.
- Keep handler ACK fast. Long Matrix work may be done inline for m2-late because the Brehon publisher is already fire-and-forget, but do not block unrelated bridge appservice transaction routes.

---

### Task 7: Workspace e2e coverage for publisher behavior

**Files:**

- `crates/server/tests/e2e/governance.rs` (preferred, existing governance include)

**Tests to add:**

1. `m2_late_sanction_event_posts_to_subscriber_once`
   - Bootstrap governance fixture.
   - Insert an active `sanction_subscriber` row pointing to a local mock HTTP server.
   - Set `BRIDGE_CALLBACK_SECRET` for the test process.
   - Drive the existing report/jury/vote golden path until the third vote creates a sanction.
   - Mock server asserts:
     - method `POST`
     - bearer header matches secret
     - JSON has `sanction_kind`, `subject_brehon_actor_id`, `effective_from`, `governance_log_entry_hash`
     - JSON lacks `person_id`, username, email, actor URL.
   - DB asserts:
     - one `sanction_event` row for that `governance_log_entry_hash`
     - one `governance_log` `sanction_published` row.
2. `m2_late_late_vote_does_not_publish_again`
   - After the publish test reaches quorum, submit a late vote if the existing fixture permits it.
   - Assert request count remains one and no second event/log row appears.
3. `m2_late_non_local_sanction_action_skips_publish`
   - Exercise the mapping helper or seed a sanction with `FederationQuarantineRecommendation` / `Restoration` and call the publisher directly against a mock subscriber.
   - Assert zero webhook requests and zero `sanction_event` rows.
4. `m2_late_delivery_failure_is_logged`
   - Active subscriber points at a closed port or returns 500.
   - Assert no handler error reaches the user response and `sanction_event_delivery_failed` is appended.

**Validation command:**

```bash
scripts/brehon/cargo-test.sh --workspace --features full --test e2e m2_late_sanction_event > .pi/m2-late-e2e.log 2>&1
```

If the exact filter cannot match all four names, run the specific test names one by one and record tails in the task notes.

**Gotchas:**

- The publisher is spawned. Tests must wait/poll with a bounded timeout for the mock server request and DB log row.
- Do not paste raw cargo output.
- Keep mock server local-only; no external network.

---

### Task 8: Bridge tests and final gates

**Files:**

- `services/bridge/tests/sanction_event.rs`
- optional minor additions to `services/bridge/tests/room_provisioning.rs` only if reuse is clearer

**Tests to add:**

1. Bridge route auth:
   - Missing bearer → 401.
   - Wrong bearer → 401.
   - Correct bearer with `hide_content` and no content reference → 200 `{ applied:false, reason:"not_applicable:no_content_reference" }`.
2. Deserialization:
   - valid payload round-trips all four `sanction_kind` strings.
   - unknown `sanction_kind` → 400.
3. Ignored docker-compose smoke:
   - With Tuwunel running and at least one room in `bridge_room`, `prevent_post` returns 200 and mutates the room power-level event for the subject puppet.

**Validation commands:**

```bash
cd services/bridge && cargo test --no-run > ../../.pi/m2-late-bridge-testcompile.log 2>&1
cd services/bridge && cargo test sanction_event > ../../.pi/m2-late-bridge-tests.log 2>&1
scripts/brehon/cargo-check.sh --workspace --features full > .pi/m2-late-workspace-check.log 2>&1
scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .pi/m2-late-workspace-clippy.log 2>&1
scripts/brehon/cargo-test.sh --test e2e -p lemmy_server > .pi/m2-late-e2e-full.log 2>&1
```

Run the ignored docker-compose smoke manually only when the bridge stack is available:

```bash
cd services/bridge && cargo test prevent_post_power_level_smoke -- --ignored
```

---

## Validation Gates Summary

| Gate | Command | Required Result |
|---|---|---|
| Migration round-trip | `scripts/brehon/migrate-roundtrip.sh` | exits 0, new migration applied on fresh PG |
| Schema crate | `scripts/brehon/cargo-check.sh -p lemmy_db_schema_file` | exits 0 |
| DB schema crate | `scripts/brehon/cargo-check.sh -p lemmy_db_schema` | exits 0 |
| API crate | `scripts/brehon/cargo-check.sh -p lemmy_api` | exits 0 |
| Server crate | `scripts/brehon/cargo-check.sh -p lemmy_server` | exits 0 |
| Bridge crate | `cd services/bridge && cargo check` | exits 0 |
| Bridge lint | `cd services/bridge && cargo clippy -- -D warnings` | exits 0 |
| Bridge test compile | `cd services/bridge && cargo test --no-run` | exits 0 |
| Workspace full check | `scripts/brehon/cargo-check.sh --workspace --features full` | exits 0 |
| Workspace clippy | `scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings` | exits 0 |
| Server e2e | `scripts/brehon/cargo-test.sh --test e2e -p lemmy_server` | exits 0 |
| Matrix dependency invariant | `cargo tree --workspace 2>/dev/null \| grep -cE 'matrix-sdk\|ruma'` | prints `0` |

---

## Acceptance Criteria

- [ ] `SanctionKind` exists with exactly four variants and snake_case webhook serialization.
- [ ] `sanction_event` and `sanction_subscriber` tables migrate forward and down cleanly.
- [ ] Startup with `BRIDGE_SANCTION_CALLBACK_URL` seeds exactly one subscriber row idempotently.
- [ ] Startup without `BRIDGE_SANCTION_CALLBACK_URL` performs no external publish action and does not fail.
- [ ] Quorum-created sanctions with a mappable `SanctionAction` and `target_person_id` produce one webhook POST after commit.
- [ ] Late votes after quorum do not produce duplicate event rows, duplicate POSTs, or duplicate `sanction_published` entries.
- [ ] `Restoration` and `FederationQuarantineRecommendation` skip publishing in m2-late.
- [ ] Webhook body contains `subject_brehon_actor_id = actor_pseudonym.pseudonym` and no direct user identifiers.
- [ ] Bridge endpoint accepts only `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>`.
- [ ] Bridge returns ACK body `{applied, reason, applied_at}` for all four sanction kinds.
- [ ] Delivery failure is logged with `sanction_event_delivery_failed` and never changes the original jury-vote response.
- [ ] No Matrix SDK / Ruma dependency enters the root workspace.

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---:|---:|---|
| Network I/O accidentally enters `submit_jury_vote` transaction | MED | HIGH | Task 5 requires spawn only after `run_transaction` returns; tests assert response still succeeds on subscriber failure |
| Duplicate publish on late vote | MED | HIGH | Seed produced only at actual sanction insert; unique `governance_log_entry_hash`; late-vote e2e |
| Raw identity leaks in payload/log | MED | HIGH | Use actor pseudonym helper; tests assert no `person_id`/username/email/AP URL; `governance_log::append` scrub remains in path |
| OQ-ADR016-04 full ACK status storage exceeds M2-late | MED | MED | Scope note: M2-late records success/failure delivery log only; full per-app ACK endpoint deferred to M3 by brief |
| Bridge room store lacks subject index | HIGH | MED | Apply subject puppet power-level across all active bridge-managed rooms; subject-scoped index is Phase 7/M3 expansion |
| Process crash between commit and spawn loses delivery | LOW | MED | Accepted v0 simplification: no retry scheduler. `sanction_event` + delivery logs make future replay possible; true durable outbox is M3+ |
| Missing `BRIDGE_CALLBACK_SECRET` causes silent no-op | MED | MED | Publisher appends `sanction_event_delivery_failed` with `missing_bridge_callback_secret`; bridge rejects missing secret loudly |
| Diesel enum wire/DB casing mismatch | MED | HIGH | PascalCase PG tokens + serde snake_case; schema crate check and webhook JSON tests |
| Bridge route accidentally protected by HS token | MED | HIGH | Task 6 routing requirement: sanction route after/split from `hs_token_auth`; auth tests prove 401/200 behavior |

---

## Open Questions / Escalations

No blocking OQs remain for M2-late planning.

Non-blocking scope notes for reviewer awareness:

1. **ACK storage:** OQ-ADR016-04 describes a full acknowledgement endpoint and per-app status as governance-log entries. The M2-late brief narrows this to synchronous bridge ACK plus Brehon delivery success/failure entries; full ACK callback storage is M3 scope. Escalate only if the reviewer wants OQ-ADR016-04 fully implemented in M2-late.
2. **Subject-scoped room lookup:** the brief says the bridge queries `bridge_room` for rooms for the subject, but current `bridge_room` has no subject column and the universal payload has no case/room ID. This plan uses global bridge-managed room enumeration with per-subject power-level entries. Escalate if a subject-specific room index is required before M3/B-actor.
3. **True at-least-once:** fire-and-forget without a retry worker cannot guarantee delivery across process crash. This plan gives idempotent event rows and failure logs, but a durable outbox/replay worker is explicitly future work.

---

## Definition of Done

- Plan tasks above implemented in order, each with a focused commit.
- All validation gates pass with logs under `.pi/` or equivalent task-local paths.
- Registry entry-kind update is completed by advisor or explicitly assigned.
- No `.claude/decision-queue.json` / daemon state mutations by the implementation worker.
- No Rust changes are made outside this plan's file list without updating the plan first.

