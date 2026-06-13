# Plan: M2-late B-actor — Portable Actor-ID Linkage (Phase 7)

## Summary

This plan delivers **B-actor** (ADR-016 component 3): a portable Brehon actor ID that an
external app links to its native user identity via an OAuth-style redirect, sealed by a
**dual-signed link-claim** (Brehon ed25519 + app countersignature). Brehon holds the canonical
`actor_app_link` mapping table (`brehon_actor_id ↔ app_id ↔ app_local_id`, with `revoked_at`);
the app (the M1/M2 Matrix bridge daemon for this reference integration) stores the inverse in a
local SQLite cache. Link and unlink each emit an append-only governance-log entry (ADR-008). The
mapping keys off `actor_pseudonym` (never raw `person_id`), preserving ADR-015 pseudonymity. The
flow reuses the existing `BRIDGE_CALLBACK_SECRET` transport for the bridge-callback reference and
the existing JWT `LocalUserView` extraction for user authentication — no new auth subsystem, no
OIDC IdP role. This is the **first reference implementation** of the B-actor seam; future
non-Matrix apps cite it.

## Source

- [v2-messaging-rtc.prd.md](../prds/v2-messaging-rtc.prd.md) (umbrella) → M2 track
- [m2-governance-triggered-rooms.prd.md](../prds/m2-governance-triggered-rooms.prd.md) §Implementation Phases Phase 7 (B-actor) — **re-scoped into scope 2026-06-13**
- Resolved OQ: **OQ-ADR016-03** (B-actor link-flow UX + mapping integrity) — resolved 2026-06-13, lean adopted as-written (see [99 §OQ-ADR016-03](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md))
- Relevant [04](../../docs/brehon-law-inspired-network/04-data-model-and-api.md) sections: §2 (`governance_log` TEXT-`entry_kind` hash chain), §3 (`actor_pseudonym`)
- Governing ADRs from [99](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md): **ADR-016** (cross-app backplane / B-actor), **ADR-008** (append-only signed log), **ADR-015** (pseudonymity), ADR-004 (plane separation), ADR-011 (AGPLv3)

## Problem Statement

Brehon's value proposition is *one* governance system applied to misbehaviour wherever it
happens — one jury pool, one reputation graph, one rule set, across every integrated app
(ADR-016). That requires a **portable actor ID**: the same Brehon-issued identity must resolve to
"`@alice:matrix.example`" in the Matrix bridge, "alice-on-peertube" in PeerTube, etc., so
reputation and sanctions roll up coherently. Today (post-m2-late-2) the Matrix bridge uses
**bridge-local puppet IDs** (M1's `@_brehon_*` namespace) — there is no canonical Brehon-side
mapping from a Brehon actor to an app's native user, and no tamper-resistant link a sanction
subscriber can trust. Without B-actor, every app is a parallel governance silo (one of the
explicitly-rejected ADR-016 alternatives), and a bad actor evades sanctions by switching apps.

This plan builds the **canonical mapping + the dual-signed link-flow** that makes the actor ID
portable and the mapping re-point-resistant.

## Solution Statement

Three pieces, mirroring shipped m2-late-1 (B-publish) patterns end-to-end:

1. **Brehon-side canonical table + governance writes.** A new `actor_app_link` table (mirror of
   `endorsement`/`surety`: SERIAL PK, composite `UNIQUE (brehon_actor_id, app_id, app_local_id)`,
   nullable `revoked_at`), FK to `actor_pseudonym(id)`. Two new entry kinds
   (`actor_app_link_created`, `actor_app_link_revoked`) on the hash chain via the unchanged
   `append()` writer. A `pub fn sign_link_claim(...)` wrapper in `db_schema` exposes the existing
   ed25519 governance key for the claim signature.

2. **Brehon-side link handlers.** A GET `/api/v4/governance/link` redirect-style endpoint that
   reads the authenticated `LocalUserView` (cookie-or-Bearer JWT, already supported), resolves the
   actor's pseudonym, mints a one-time short-TTL dual-signable claim, and POSTs it to the app's
   registered callback (reusing the `BRIDGE_CALLBACK_SECRET` transport). A POST
   `/api/v4/governance/link/revoke` that marks the row revoked (prospective) and logs it. An
   inbound POST `/api/v4/governance/link/confirm` that the app calls back (bearer-authed via
   `verify_bridge_secret`) carrying its countersignature, which finalises the `actor_app_link` row.

3. **Bridge-side inverse cache + callback handler.** An `app_actor_link` SQLite store
   (rusqlite, inline `CREATE TABLE IF NOT EXISTS`, mirroring `bridge_room.rs`) holding
   `app_local_id → brehon_actor_id`. A `/brehon/link-claim` axum handler (bearer-authed, mirroring
   `sanction_handler.rs`) that receives the Brehon claim, countersigns it, stores the inverse, and
   POSTs the countersignature back to Brehon's `/link/confirm`.

The claim binds `(brehon_actor_id, app_id, app_local_id, nonce, expires_at)`; both signatures are
verified before the canonical row is written. Revocation is prospective (`revoked_at` set; future
sanctions targeting the link no-op) and **never rewrites the chain** — historical attributions
retain the actor ID recorded at the time (ADR-008).

## Metadata

| Field | Value |
|---|---|
| Type | SCHEMA + HANDLER + FEDERATION (cross-app) |
| Complexity | **HIGH** |
| Crates Affected | `lemmy_db_schema`, `lemmy_api`, `lemmy_api_common`, `lemmy_routes`, `services/bridge` (out-of-workspace), `crates/server/tests` (e2e) |
| v0 Step | N/A — M-track (post-v1), ADR-016 reference integration |
| Dependencies | M1 (chat infra, shipped), m2-core (rooms, shipped), m2-late-1/-2 (B-publish, shipped); OQ-ADR016-03 resolved |
| Estimated Tasks | 13 |

---

## Flow Design

### Before State

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                       ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                                 ║
║  Brehon actor  ──(no canonical mapping)──>  bridge-local puppet @_brehon_<id>   ║
║                                                                                 ║
║  Each app keys sanctions/reputation off its OWN local puppet map. No portable   ║
║  actor ID. No tamper-resistant link. Switching apps escapes sanctions.          ║
║                                                                                 ║
║  DATA_FLOW: sanction → bridge → @_brehon_<puppet> (per-app silo)                ║
║  PAIN_POINT: reputation never rolls up across apps; no re-point resistance      ║
║                                                                                 ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After State

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                       ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                                 ║
║  app sends user → GET /governance/link (JWT cookie)                             ║
║    → Brehon resolves actor_pseudonym, mints claim                              ║
║      (brehon_actor_id, app_id, app_local_id, nonce, expires_at), SIGNS (ed25519)║
║    → POST claim to app callback  (Bearer BRIDGE_CALLBACK_SECRET)               ║
║      → app COUNTERSIGNS, stores inverse (app_local_id → brehon_actor_id)       ║
║      → POST /governance/link/confirm (Bearer) with app signature              ║
║        → Brehon verifies BOTH sigs → INSERT actor_app_link                     ║
║          → append(ENTRY_KIND_ACTOR_APP_LINK_CREATED, {pseudonym, app_id})      ║
║                                                                                 ║
║  unlink: POST /governance/link/revoke → UPDATE revoked_at=now() (prospective)  ║
║          → append(ENTRY_KIND_ACTOR_APP_LINK_REVOKED, ...)  [chain not rewritten]║
║                                                                                 ║
║  DATA_FLOW: link → dual-signed claim → canonical actor_app_link + chain entry  ║
║  VALUE_ADD: one portable actor ID; reputation/sanctions roll up; re-point-proof ║
║                                                                                 ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Endpoint Changes

| Endpoint | Before | After |
|---|---|---|
| `GET /api/v4/governance/link?app_id=&app_local_id=` | didn't exist | authed (JWT) link initiation; mints + signs claim; POSTs to app callback; 302/JSON ack |
| `POST /api/v4/governance/link/confirm` | didn't exist | app callback (Bearer); verifies app countersignature; writes canonical `actor_app_link` row + `actor_app_link_created` log |
| `POST /api/v4/governance/link/revoke` | didn't exist | authed (JWT) prospective unlink; `revoked_at=now()` + `actor_app_link_revoked` log |
| (bridge) `POST /brehon/link-claim` | didn't exist | Brehon → bridge claim delivery (Bearer); bridge countersigns, caches inverse, calls `/link/confirm` |

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/db_schema/src/source/governance/endorsement.rs` | all | **Primary model mirror** — composite-key + nullable `revoked_at` + `skip_serializing_none` shape; copy verbatim, swap field names |
| P0 | `crates/db_schema/src/source/governance/actor_pseudonym.rs` | all | The table you FK to; `ActorPseudonymId` newtype; the `pseudonym TEXT` (UUID-string) convention |
| P0 | `migrations/2026-04-15-100300-0000_add_reputation_and_surety/up.sql` | 1-19 | **Migration mirror** — `surety`/`endorsement` composite `UNIQUE` + `revoked_at TIMESTAMPTZ` |
| P0 | `migrations/2026-06-07-000000-0000_add_sanction_event/{up,down}.sql` | all | Most-recent governance migration; naming, FK, `down.sql` reverse-order |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 112-256, 277-357 | `append()` contract; ENTRY_KIND const block (m2-late-1 block at 250-256); `load_signing_key`/`sign` at 319-357 |
| P0 | `crates/api/api/src/governance/sanction_publisher.rs` | 36-47, 86, 139-215 | **Closest behavioural mirror** — scrubbed-payload `append()` callsite, SAVEPOINT reborrow, `Bearer {secret}` POST out, `SanctionEventPayload` wire struct (hash-in-payload precedent) |
| P0 | `crates/api/api/src/governance/bridge_auth.rs` | all (19) | `verify_bridge_secret(&req)?` — the inbound bearer check, first line of `/link/confirm` |
| P0 | `crates/api/api_crud/src/governance/request_appeal.rs` | 53-93 | **Handler mirror** — `(Json(data), context, local_user_view: LocalUserView)` arg shape; `local_user_view.person.id`; `get_or_create` |
| P0 | `crates/server/tests/e2e/m2_late.rs` | all (390) | **E2E mirror** — boot sequence, `spawn_mock_subscriber`, `BRIDGE_CALLBACK_SECRET` guard, governance_log `.filter(entry_kind.eq).count()` assert, ADR-015 pseudonym assert |
| P0 | `services/bridge/src/bridge_room.rs` | all (73) | **Bridge SQLite store mirror** — `open`/`upsert`/`lookup` for `app_actor_link` inverse cache |
| P0 | `services/bridge/src/sanction_handler.rs` | 59-72, 90-163, 327-476 | **Bridge inbound mirror** — axum `Json<>` extractor, inline Bearer check, `(StatusCode, Json).into_response()`, 3-test pattern |
| P1 | `crates/api/routes/src/lib.rs` | 176, 480-499, 31-55 | Route wiring under `scope("/governance")`; handler import blocks |
| P1 | `crates/api/api_common/src/governance.rs` | 1-19, 259-277, 846-883 | DTO conventions (Convention A ts-rs user-facing; Convention B bridge-wire); the 11-endpoint carve-out gotcha |
| P1 | `crates/db_schema/src/newtypes.rs` | 205-209, 289-293 | Governance newtype block; `ActorPseudonymId` macro to clone for `ActorAppLinkId` |
| P1 | `.claude/rules/governance-log-entry-kind-registry.md` | 270+, m2-late-1 section | Registry add procedure; acceptance invariant total (67→69); collision check |
| P1 | `services/bridge/src/appservice.rs` | 231-259 | Route registration AFTER `.route_layer(hs_token_auth)` so `/brehon/*` self-auth with `BRIDGE_CALLBACK_SECRET` |
| P1 | `crates/api/api_utils/src/bridge_notify.rs` | all (156) | `BRIDGE_CALLBACK_SECRET` + URL env resolution; fire-and-forget POST contract |
| P2 | `crates/api/api/src/governance/actor_pseudonym_helper.rs` | 41-80 | `get_or_create(pool, person_id)` + unique-violation race handling (claim-mint uses it) |

**External Documentation:**

| Source | Version | Section | Why |
|---|---|---|---|
| [ed25519-dalek](https://docs.rs/ed25519-dalek) | match `Cargo.toml` | `Signer::sign`, `VerifyingKey::verify_strict` | Brehon already uses `signing_key.sign(bytes).to_bytes().to_vec()`; the **app countersignature verification** on `/link/confirm` needs `VerifyingKey::verify_strict` — confirm the exact API + that the app's pubkey is configured (new env var) |
| [rusqlite](https://docs.rs/rusqlite) | match `services/bridge/Cargo.toml` | `Connection::execute_batch`, `ON CONFLICT` | bridge inverse cache; already used in `bridge_room.rs` — no new dep |

---

## Patterns to Mirror

**DIESEL_MODEL (copy `endorsement.rs` verbatim, swap field names):**
```rust
// SOURCE: crates/db_schema/src/source/governance/endorsement.rs (full)
// COPY THIS PATTERN for actor_app_link.rs:
use crate::newtypes::ActorAppLinkId;          // NEW newtype (see NEWTYPE task)
use crate::newtypes::ActorPseudonymId;        // FK target
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::actor_app_link;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]                       // present because revoked_at is Option
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = actor_app_link))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A portable Brehon actor ↔ external-app-user mapping (B-actor, ADR-016).
/// `brehon_actor_id` references actor_pseudonym(id); never stores raw person_id (ADR-015).
pub struct ActorAppLink {
  pub id: ActorAppLinkId,
  pub brehon_actor_id: ActorPseudonymId,
  pub app_id: String,
  pub app_local_id: String,
  pub created_at: DateTime<Utc>,
  pub revoked_at: Option<DateTime<Utc>>,       // LAST field; excluded from InsertForm
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = actor_app_link))]
pub struct ActorAppLinkInsertForm {
  pub brehon_actor_id: ActorPseudonymId,
  pub app_id: String,
  pub app_local_id: String,
  // id / created_at / revoked_at omitted — DB default / set-later
}
```

**MIGRATION (mirror `surety`/`endorsement` + `sanction_event`):**
```sql
-- SOURCE: migrations/2026-04-15-100300-0000_add_reputation_and_surety/up.sql:1-19
-- COPY for migrations/2026-06-13-000000-0000_add_actor_app_link/up.sql:
CREATE TABLE actor_app_link (
    id SERIAL PRIMARY KEY,
    brehon_actor_id INTEGER NOT NULL REFERENCES actor_pseudonym (id) ON DELETE CASCADE,
    app_id TEXT NOT NULL,
    app_local_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    UNIQUE (brehon_actor_id, app_id, app_local_id)
);
-- down.sql:
-- DROP TABLE actor_app_link;
```

**ENTRY_KIND consts (mirror m2-late-1 block):**
```rust
// SOURCE: crates/db_schema/src/source/governance/governance_log.rs:255-256
// ADD after the m2-late-1 block:
// --- m2-late-b-actor entry kinds (2) ---
pub const ENTRY_KIND_ACTOR_APP_LINK_CREATED: &str = "actor_app_link_created";
pub const ENTRY_KIND_ACTOR_APP_LINK_REVOKED: &str = "actor_app_link_revoked";
```

**SCRUBBED-PAYLOAD append() CALLSITE (mirror `sanction_publisher.rs:197-208`):**
```rust
// SOURCE: crates/api/api/src/governance/sanction_publisher.rs:197-208
// COPY THIS PATTERN at link-confirm:
let subject = actor_pseudonym_helper::get_or_create(&mut context.pool(), person_id).await?;
governance_log::append(
  &mut context.pool(),                                  // standalone pool (admin_close_case shape)
  ENTRY_KIND_ACTOR_APP_LINK_CREATED,
  serde_json::json!({
    "brehon_actor_pseudonym": &subject,                 // pseudonym string, survives scrub
    "app_id": &app_id,                                  // free text, scrubbed in place
    "link_id": link_row.id.0,                           // integer FK, survives scrub
  }),
  Some(subject.clone()),                                // actor_pseudonym arg
)
.await?;
```

**HANDLER reading authed user (mirror `request_appeal.rs:53-93`):**
```rust
// SOURCE: crates/api/api_crud/src/governance/request_appeal.rs:53-93
// COPY for link_actor (GET):
pub async fn link_actor(
  Query(params): Query<LinkActorRequest>,               // GET query params, not Json
  context: Data<LemmyContext>,
  local_user_view: LocalUserView,                       // resolves from cookie OR Bearer JWT
) -> LemmyResult<HttpResponse> {                         // HttpResponse for 302 redirect / JSON ack
  check_local_user_valid(&local_user_view)?;
  let caller_id = local_user_view.person.id;
  let caller_pseudonym = actor_pseudonym_helper::get_or_create(&mut context.pool(), caller_id).await?;
  // mint claim, sign, POST to app callback ...
}
```

**INBOUND BEARER-AUTH (mirror `bridge_auth.rs` + `room_event_handler.rs`):**
```rust
// SOURCE: crates/api/api/src/governance/room_event_handler.rs:26-36
// COPY for link_confirm (POST from app):
pub async fn link_confirm(
  req: HttpRequest,
  body: Json<LinkConfirmRequest>,
  context: Data<LemmyContext>,
) -> LemmyResult<Json<serde_json::Value>> {
  bridge_auth::verify_bridge_secret(&req)?;             // FIRST line — service-principal auth
  // verify app countersignature, INSERT actor_app_link, append() ...
}
```

**ed25519 SIGN wrapper (expose `load_signing_key` via a pub wrapper in db_schema):**
```rust
// SOURCE: crates/db_schema/src/source/governance/governance_log.rs:319, 342
// ADD a pub wrapper (load_signing_key stays private):
#[cfg(feature = "full")]
pub fn sign_link_claim(claim_bytes: &[u8]) -> LemmyResult<Vec<u8>> {
  let signing_key = load_signing_key()?;
  Ok(signing_key.sign(claim_bytes).to_bytes().to_vec())
}
```

**BRIDGE SQLite STORE (mirror `bridge_room.rs` open/upsert/lookup):**
```rust
// SOURCE: services/bridge/src/bridge_room.rs (full)
// COPY for services/bridge/src/app_actor_link.rs:
pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_actor_link (
            app_local_id   TEXT NOT NULL PRIMARY KEY,
            brehon_actor_id TEXT NOT NULL,
            linked_at      INTEGER,
            revoked_at     INTEGER
        );"
    )?;
    Ok(conn)
}
// upsert() with ON CONFLICT(app_local_id) DO UPDATE SET ... = excluded.*  (params![...])
// lookup(conn, app_local_id) -> Result<Option<String>>  (the brehon_actor_id)
```

**BRIDGE INBOUND HANDLER (mirror `sanction_handler.rs:90-163`):**
```rust
// SOURCE: services/bridge/src/sanction_handler.rs:90-115
// COPY for handle_link_claim:
pub async fn handle_link_claim(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(payload): Json<LinkClaimPayload>,
) -> Response {
    let expected = &state.config.bridge_callback_secret;
    let provided = headers.get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok()).and_then(|s| s.strip_prefix("Bearer ")).map(str::to_owned);
    if provided.as_deref() != Some(expected.as_str()) {
        return (StatusCode::UNAUTHORIZED, Json(serde_json::json!({"error":"unauthorized"}))).into_response();
    }
    // verify Brehon sig, countersign, app_actor_link::upsert, POST /governance/link/confirm ...
}
```

---

## Files to Change

| File | Action | Justification |
|---|---|---|
| `migrations/2026-06-13-000000-0000_add_actor_app_link/up.sql` | CREATE | `actor_app_link` table per OQ-ADR016-03(b) |
| `migrations/2026-06-13-000000-0000_add_actor_app_link/down.sql` | CREATE | `DROP TABLE actor_app_link;` |
| `crates/db_schema_file/src/schema.rs` | UPDATE | Regen via `diesel print-schema`; new `table!` + `joinable!` + `allow_tables_…` entries |
| `crates/db_schema/src/newtypes.rs` | UPDATE | Add `ActorAppLinkId(pub i32)` |
| `crates/db_schema/src/source/governance/actor_app_link.rs` | CREATE | Diesel model + InsertForm (mirror `endorsement.rs`) |
| `crates/db_schema/src/source/governance/mod.rs` | UPDATE | `pub mod actor_app_link;` (alphabetical) |
| `crates/db_schema/src/source/governance/governance_log.rs` | UPDATE | 2 new ENTRY_KIND consts + `pub fn sign_link_claim` wrapper |
| `crates/api/api/src/governance/governance_log.rs` | UPDATE | Shim re-export of the 2 new consts (alphabetical, sort first) |
| `crates/api/api_common/src/governance.rs` | UPDATE | `LinkActorRequest`, `LinkClaimPayload`, `LinkConfirmRequest` DTOs (+ carve-out comment) |
| `crates/api/api/src/governance/actor_app_link.rs` | CREATE | `link_actor`, `link_confirm`, `revoke_link` handlers + claim mint/verify + CRUD |
| `crates/api/api/src/governance/mod.rs` | UPDATE | `pub mod actor_app_link;` + handler exports |
| `crates/api/routes/src/lib.rs` | UPDATE | Wire 3 routes under `scope("/governance")`; add handler imports |
| `.claude/rules/governance-log-entry-kind-registry.md` | UPDATE | New section (2 kinds) + total 67→69 + collision check |
| `services/bridge/src/app_actor_link.rs` | CREATE | SQLite inverse cache (mirror `bridge_room.rs`) |
| `services/bridge/src/link_handler.rs` | CREATE | `/brehon/link-claim` axum handler + countersign + callback |
| `services/bridge/src/appservice.rs` | UPDATE | Register `/brehon/link-claim` AFTER `route_layer(hs_token_auth)` |
| `services/bridge/src/config.rs` | UPDATE | Add `brehon_signing_pubkey` (verify Brehon sig) + `bridge_signing_key` (countersign) env config |
| `services/bridge/src/main.rs` | UPDATE | Wire `app_actor_link::open` + `link_handler` module |
| `crates/server/tests/e2e/actor_app_link.rs` | CREATE | E2E tests (mirror `m2_late.rs`) |
| `crates/server/tests/e2e.rs` | UPDATE | `include!("e2e/actor_app_link.rs");` after line 153 |

---

## NOT Building (v0/M-scope limits)

- **QR-code pairing and "paste-actor-ID" link entrypoints** — OQ-ADR016-03(a) deferred them as
  optional alternatives; v1 ships the OAuth-style redirect (bridge-callback reference flow) only.
- **A non-bridge subscriber link-claim auth mechanism** — the reference integration uses the
  bridge's `BRIDGE_CALLBACK_SECRET`; signed-link-claim auth for arbitrary non-bridge apps
  first-blocks the first non-Matrix app integration ADR (OQ-ADR016-03 `Blocks:` line).
- **Reputation/sanction re-keying onto the portable ID** — this plan builds the *mapping*; rewiring
  the sanction subscriber and reputation roll-up to key off `actor_app_link` instead of the puppet
  map is a **follow-up** (it depends on this table existing). Out of scope here to keep the plan
  atomic; flagged in §Risks.
- **OIDC IdP role for Brehon** — explicitly rejected by ADR-016 ("Brehon is *not* an identity
  provider"). The link-flow authenticates against Brehon's existing JWT session; Brehon never
  becomes the app's login authority.
- **Mutual app↔app linking or transitive links** — one Brehon actor ↔ one app-local-id per app;
  no app-to-app links.
- **Unlink consent negotiation / app-side veto** — OQ-ADR016-03(d) resolved to *unilateral
  prospective unlink from the Brehon side*; no app-consent handshake.

---

## Step-by-Step Tasks

Execute in dependency order. One commit per task. Each task has a MIRROR reference, an exact file
path, and a validation command. Tasks 1-9 are Brehon-workspace (cargo); tasks 10-12 are
`services/bridge` (Linux-only cargo via `cargo-linux.sh`); task 13 is e2e.

> **Skill triggers:** Task 4 (newtype) + Task 8 (shim re-export, repeat-pattern alphabetical
> insert) are `/edit-mechanical` candidates (rg-enumerate-first). Task 13 (new e2e file) is a
> `/test-write` candidate (LemmyResult harness + pseudonymisation discipline). All cargo VALIDATE
> lines run through `/cargo-validate`.

### Task 1: CREATE migration `migrations/2026-06-13-000000-0000_add_actor_app_link/{up,down}.sql`
- **ACTION**: `up.sql` creates `actor_app_link`; `down.sql` drops it
- **IMPLEMENT**: exactly the table in §Patterns/MIGRATION — SERIAL PK, FK to `actor_pseudonym(id) ON DELETE CASCADE`, `app_id TEXT`, `app_local_id TEXT`, `created_at TIMESTAMPTZ NOT NULL DEFAULT now()`, `revoked_at TIMESTAMPTZ`, `UNIQUE (brehon_actor_id, app_id, app_local_id)`
- **MIRROR**: `migrations/2026-04-15-100300-0000_add_reputation_and_surety/up.sql:1-19` (surety/endorsement composite-UNIQUE + revoked_at); `migrations/2026-06-07-000000-0000_add_sanction_event/down.sql` (reverse-order drop)
- **GOTCHA**: dir name format is `YYYY-MM-DD-HHMMSS-0000_snake_case`; the `0000` suffix is literal; no `metadata.toml`/`README`. Must sort AFTER `2026-06-07-…_add_sanction_event`
- **GOTCHA**: no standalone `CREATE INDEX` — the `UNIQUE` + FK + PK supply the indexes (governance-migration convention)
- **VALIDATE**: `diesel migration run && diesel migration redo` (both directions clean) — run via the Docker test DB

### Task 2: REGEN `crates/db_schema_file/src/schema.rs` (table! + joinable! + allow_tables)
- **ACTION**: run `diesel print-schema` against the migrated DB; confirm the generated `actor_app_link` `table!` block has `revoked_at -> Nullable<Timestamptz>`, `app_id -> Text`, `app_local_id -> Text`, `brehon_actor_id -> Int4`
- **IMPLEMENT**: ensure `diesel::joinable!(actor_app_link -> actor_pseudonym (brehon_actor_id));` lands alphabetically (right after `actor_pseudonym -> person` ~schema.rs:1503) and `actor_app_link,` is in the `allow_tables_to_appear_in_same_query!` block (alphabetical, near top ~:1629)
- **MIRROR**: existing `actor_pseudonym` `table!` (schema.rs:157-164) + its `joinable!`/`allow_tables` entries
- **GOTCHA**: `schema.rs` lives at `crates/db_schema_file/src/schema.rs` (NOT `crates/db_schema/`); it is **generated** — `diesel.toml` has `patch_file`; verify the patch still applies cleanly (per `feedback_diesel_print_schema_auto_applies_patch`)
- **VALIDATE**: `cargo check -p lemmy_db_schema_file`

### Task 3: CREATE Diesel model `crates/db_schema/src/source/governance/actor_app_link.rs`
- **ACTION**: write `ActorAppLink` (read) + `ActorAppLinkInsertForm`
- **IMPLEMENT**: the struct in §Patterns/DIESEL_MODEL exactly — `#[skip_serializing_none]` (present, `revoked_at` is `Option`), full derive stack, `id: ActorAppLinkId`, `brehon_actor_id: ActorPseudonymId`, `revoked_at: Option<DateTime<Utc>>` as last field; InsertForm omits `id`/`created_at`/`revoked_at`
- **MIRROR**: `crates/db_schema/src/source/governance/endorsement.rs` (full — verbatim structure)
- **GOTCHA**: `#[diesel(table_name = actor_app_link)]` must match `schema.rs`; FK field typed with the **referenced** newtype (`ActorPseudonymId`), not raw `i32`
- **GOTCHA**: import `actor_app_link` from `lemmy_db_schema_file::schema` under `#[cfg(feature = "full")]`
- **VALIDATE**: `cargo check -p lemmy_db_schema`

### Task 4: ADD newtype `ActorAppLinkId` to `crates/db_schema/src/newtypes.rs`
- **ACTION**: add the newtype struct in a new "M2 / ADR-016 B-actor" comment block at the end of the governance block
- **IMPLEMENT**: `pub struct ActorAppLinkId(pub i32);` with the standard derive stack (`Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize` + `#[cfg_attr(feature = "full", derive(DieselNewType))]` + the two ts-rs cfg_attrs)
- **MIRROR**: `crates/db_schema/src/newtypes.rs:289-293` (`ActorPseudonymId`)
- **GOTCHA**: `DieselNewType` is capital (diesel-derive-newtype macro), gated `feature = "full"`; no `Display` impl (governance ids omit it)
- **VALIDATE**: `cargo check -p lemmy_db_schema`

### Task 5: EXPORT model from `crates/db_schema/src/source/governance/mod.rs`
- **ACTION**: add `pub mod actor_app_link;` alphabetically (between `actor_pseudonym` line 3 and `appeal` line 4)
- **MIRROR**: the existing flat alphabetical `pub mod` list in mod.rs
- **GOTCHA**: plain `pub mod` (NOT `#[cfg(feature = "full")]`-gated — only `redaction` is gated; feature-gating happens per-derive inside the file)
- **VALIDATE**: `cargo check -p lemmy_db_schema`

### Task 6: ADD 2 ENTRY_KIND consts + `sign_link_claim` wrapper to `governance_log.rs`
- **ACTION**: append `ENTRY_KIND_ACTOR_APP_LINK_CREATED` + `_REVOKED` in a new commented block after m2-late-1 (after line 256); add `pub fn sign_link_claim(claim_bytes: &[u8]) -> LemmyResult<Vec<u8>>` that calls the private `load_signing_key()` then `signing_key.sign(claim_bytes).to_bytes().to_vec()`
- **IMPLEMENT**: const + wrapper exactly per §Patterns; wrapper gated `#[cfg(feature = "full")]`
- **MIRROR**: const block at governance_log.rs:255-256; sign call at :319; key load at :342
- **GOTCHA**: `load_signing_key()` stays **private** — only the new `pub fn sign_link_claim` is exposed. Do NOT make `load_signing_key` pub
- **GOTCHA**: ENTRY_KIND values are `lower_snake` matching the const tail (`actor_app_link_created`); the define-side count goes 67→69
- **VALIDATE**: `cargo check -p lemmy_db_schema`

### Task 7: SHIM re-export the 2 consts in `crates/api/api/src/governance/governance_log.rs`
- **ACTION**: add `ENTRY_KIND_ACTOR_APP_LINK_CREATED, ENTRY_KIND_ACTOR_APP_LINK_REVOKED,` to the `pub use` block — they sort FIRST (`ACTOR_` < `ADMIN_`, before line 40's `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED`)
- **MIRROR**: the alphabetical `pub use` block at governance_log.rs:39-71
- **GOTCHA**: parity invariant — shim identifier count must equal db_schema define count (both +2)
- **VALIDATE**: `cargo check -p lemmy_api`

### Task 8: UPDATE registry doc `.claude/rules/governance-log-entry-kind-registry.md`
- **ACTION**: add `## m2-late-b-actor entry kinds (2, this sub-phase)` section with a 2-row table (`| Rust const | &str value | Source | Emitting handler | Semantic |`); update the acceptance-invariant total from **67** to **69** and append `+ 2 m2-late-b-actor` to the count breakdown
- **IMPLEMENT**: provenance paragraph (advisor-owned `.claude/**` per DQ #235 — this is an advisor edit, not Junior); emitting handler = `crates/api/api/src/governance/actor_app_link.rs::link_confirm` / `::revoke_link`
- **MIRROR**: the m2-late-1 section structure
- **GOTCHA**: this is a `.claude/**` file — **advisor-authored direct on governance-v0** (BM/Junior file-ownership boundary; not in any impl-task brief's commit set)
- **VALIDATE**: collision check — `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns 69; no duplicate `&str` values

### Task 9: ADD DTOs + handlers `crates/api/api/src/governance/actor_app_link.rs` + DTOs in `api_common`
- **ACTION**: (a) in `crates/api/api_common/src/governance.rs` add `LinkActorRequest` (GET query: `app_id`, `app_local_id`), `LinkClaimPayload` (bridge-wire: `brehon_actor_id`, `app_id`, `app_local_id`, `nonce`, `expires_at`, `brehon_signature`), `LinkConfirmRequest` (`nonce`, `app_signature`, `app_local_id`); (b) in `crates/api/api/src/governance/actor_app_link.rs` implement `link_actor` (GET, authed), `link_confirm` (POST, bearer), `revoke_link` (POST, authed) + the claim mint/verify + CRUD (`insert_link`, `revoke_link_row` with `revoked_at.is_null()` active-filter)
- **IMPLEMENT**:
  - `link_actor`: `check_local_user_valid` → `get_or_create` pseudonym → build claim `(brehon_actor_id, app_id, app_local_id, nonce, expires_at)` → `governance_log::sign_link_claim(&claim_bytes)` → POST `LinkClaimPayload` to the app callback URL (env `BRIDGE_LINK_CLAIM_URL`, `Authorization: Bearer {BRIDGE_CALLBACK_SECRET}`, fire-and-forget log-and-swallow per ADR-012) → return `HttpResponse` ack
  - `link_confirm`: `bridge_auth::verify_bridge_secret(&req)?` FIRST → verify app countersignature against the configured app pubkey (`ed25519_dalek::VerifyingKey::verify_strict`) → verify nonce unexpired + unconsumed → INSERT `actor_app_link` (catch `UniqueViolation` → idempotent re-read, mirror `actor_pseudonym_helper`) → `append(ENTRY_KIND_ACTOR_APP_LINK_CREATED, scrubbed payload, Some(pseudonym))`
  - `revoke_link`: `check_local_user_valid` → `UPDATE actor_app_link SET revoked_at = now() WHERE brehon_actor_id = ? AND app_id = ? AND revoked_at IS NULL` → `append(ENTRY_KIND_ACTOR_APP_LINK_REVOKED, ...)`
- **MIRROR**: `request_appeal.rs:53-93` (authed handler); `bridge_auth.rs` + `room_event_handler.rs:26-36` (inbound bearer); `sanction_publisher.rs:139-208` (Bearer POST out + scrubbed append); `actor_pseudonym_helper.rs:41-80` (unique-violation idempotent insert)
- **GOTCHA (ADR-015 load-bearing)**: the claim + every log payload carries `actor_pseudonym` (UUID string), **never** `person_id`, username, or email. DoD: `grep -n 'person\.id\|local_user.email\|\.name' crates/api/api/src/governance/actor_app_link.rs` shows `person.id` ONLY as input to `get_or_create`, never in a payload or claim. WHY: a raw person_id in a cross-app payload defeats the pseudonymity the whole governance log depends on (ADR-015) and leaks Lemmy identity to every connected app
- **GOTCHA (ADR-008 load-bearing)**: `link_confirm` and `revoke_link` MUST call `governance_log::append(...)` before returning success. WHY: ADR-008 — every write to mapping state is tamper-evidence-relevant; an unlogged link is an unauditable cross-app identity binding. DoD: `grep -c 'governance_log::append' crates/api/api/src/governance/actor_app_link.rs` ≥ 2
- **GOTCHA**: dual-signature is the re-point defence (OQ-ADR016-03(c)) — `link_confirm` MUST reject if EITHER signature fails `verify_strict`. A single-signed or unsigned claim is a hard reject, not a warning
- **GOTCHA**: DTO carve-out — `crates/api/api_common/src/governance.rs:1-19` enforces an "11 user-facing endpoints" guideline; `LinkActorRequest`/`revoke` are new user-facing surfaces → add a carve-out comment entry. `LinkClaimPayload` is bridge-wire (Convention B, no ts-rs) → exempt
- **GOTCHA**: `mod.rs` for `crates/api/api/src/governance/` needs `pub mod actor_app_link;` + the 3 handler fns exported
- **VALIDATE**: `cargo check -p lemmy_api_common && cargo check -p lemmy_api`

### Task 10: WIRE routes in `crates/api/routes/src/lib.rs`
- **ACTION**: add under `scope("/governance")` (~line 499): `.route("/link", get().to(link_actor))`, `.route("/link/confirm", post().to(link_confirm))`, `.route("/link/revoke", post().to(revoke_link))`; add the 3 handler fns to the `use lemmy_api::{ governance::{...} }` import block (~line 31-55)
- **MIRROR**: `crates/api/routes/src/lib.rs:480-499` (governance scope wiring); `handle_room_event` registration at :483 (bearer-authed inbound route under governance scope)
- **GOTCHA**: `/link/confirm` is bearer-authed (not JWT) but still sits under the rate-limited governance scope — that's fine (matches `handle_room_event`). It does NOT use the session middleware's `LocalUserView` (uses `verify_bridge_secret` instead)
- **GOTCHA**: outer scope is `scope("/api/v4")` → final paths are `/api/v4/governance/link{,/confirm,/revoke}`
- **VALIDATE**: `cargo check -p lemmy_routes`

### Task 11: CREATE bridge SQLite cache `services/bridge/src/app_actor_link.rs`
- **ACTION**: `open(path)` with inline `CREATE TABLE IF NOT EXISTS app_actor_link (app_local_id TEXT PRIMARY KEY, brehon_actor_id TEXT NOT NULL, linked_at INTEGER, revoked_at INTEGER)`; `upsert(conn, app_local_id, brehon_actor_id)` with `ON CONFLICT(app_local_id) DO UPDATE SET ... = excluded.*`; `lookup(conn, app_local_id) -> Option<String>`
- **MIRROR**: `services/bridge/src/bridge_room.rs` (full — verbatim open/upsert/lookup shape)
- **GOTCHA**: `services/bridge` is **workspace-excluded** — compiles on Linux only. Use `cargo-linux.sh`, NOT Windows-local cargo (per `feedback_bridge_validates_on_linux_not_windows`)
- **GOTCHA**: rusqlite is already in `services/bridge/Cargo.toml` (no new dep); positional `params![...]` binds
- **VALIDATE**: `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`

### Task 12: CREATE bridge link handler `services/bridge/src/link_handler.rs` + wire in `appservice.rs`/`config.rs`/`main.rs`
- **ACTION**: `handle_link_claim` axum handler (bearer check → verify Brehon sig with `brehon_signing_pubkey` → countersign with `bridge_signing_key` → `app_actor_link::upsert` → POST `LinkConfirmRequest` to `BREHON_LINK_CONFIRM_URL` with `Bearer {bridge_callback_secret}`); register `/brehon/link-claim` in `appservice.rs` AFTER `.route_layer(hs_token_auth)`; add `brehon_signing_pubkey` + `bridge_signing_key` to `config.rs` (env vars); wire module in `main.rs`
- **IMPLEMENT**: per §Patterns/BRIDGE INBOUND HANDLER; `(StatusCode, Json).into_response()` tuples; 500 on infra error, 401 on bad bearer, 200 on success
- **MIRROR**: `services/bridge/src/sanction_handler.rs:90-163` (handler + bearer); `appservice.rs:231-259` (route AFTER route_layer); `config.rs:29-30,58-59` (env-var config field)
- **GOTCHA**: the `/brehon/link-claim` route MUST be added AFTER `.route_layer(...hs_token_auth...)` so it self-auths with `BRIDGE_CALLBACK_SECRET` and does NOT inherit the Matrix hs_token middleware
- **GOTCHA (ADR-015)**: the bridge stores `app_local_id ↔ brehon_actor_id` where `brehon_actor_id` is the **pseudonym UUID string** from the claim — never a Lemmy person_id/username
- **GOTCHA**: countersignature uses a NEW bridge ed25519 key (`bridge_signing_key` env, 32-byte hex seed mirroring `GOVERNANCE_LOG_SIGNING_KEY` shape); Brehon must be configured with the matching `bridge_signing_pubkey` to verify it on `/link/confirm`
- **VALIDATE**: `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml && scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml -- -D warnings`

### Task 13: ADD e2e tests `crates/server/tests/e2e/actor_app_link.rs` + `include!` in `e2e.rs`
- **ACTION**: new test file + `include!("e2e/actor_app_link.rs");` after e2e.rs:153
- **IMPLEMENT** (each `#[tokio::test] async fn … -> LemmyResult<()>`):
  - `link_actor_creates_dual_signed_claim_and_logs` — mock app subscriber (`spawn_mock_subscriber` pattern) captures the POSTed `LinkClaimPayload`; assert it carries a non-empty `brehon_signature` and a pseudonym (NOT the seeded username); drive `/link/confirm` with a valid app countersignature; assert one `actor_app_link` row + one `actor_app_link_created` governance_log entry (`.filter(entry_kind.eq("actor_app_link_created")).count() == 1`)
  - `link_confirm_rejects_bad_app_signature` — POST `/link/confirm` with an invalid countersignature; assert 4xx and ZERO `actor_app_link` rows
  - `link_confirm_rejects_bad_bearer` — POST without `BRIDGE_CALLBACK_SECRET` bearer; assert 401, no DB/network side-effect (mirror `test_bad_bearer_returns_401`)
  - `revoke_link_is_prospective_and_logged` — create a link, revoke it; assert `revoked_at IS NOT NULL`, the `actor_app_link_created` entry is STILL present (chain not rewritten), and one new `actor_app_link_revoked` entry exists
  - `pseudonym_not_raw_identity_in_payload` — assert `subject`/`brehon_actor_pseudonym` in the claim ≠ the seeded `person.name` (ADR-015 lock, mirror m2_late.rs:333-341)
- **MIRROR**: `crates/server/tests/e2e/m2_late.rs` (full — boot sequence, `spawn_mock_subscriber`, `BRIDGE_CALLBACK_SECRET` EnvVarGuard, governance_log assert); fixtures from `common/mod.rs` (`bootstrap`, `seed_user`, `SIGNING_SEED_HEX`)
- **GOTCHA**: tests run `--test-threads=1`; use `EnvVarGuard` for `GOVERNANCE_LOG_SIGNING_KEY`, `BRIDGE_CALLBACK_SECRET`, and the app pubkey env; no `wiremock`/`httpmock` — roll the mock subscriber via `TcpListener::bind("127.0.0.1:0")` + oneshot
- **GOTCHA**: the app countersignature in-test is produced by a fixed test ed25519 keypair whose pubkey is set in the app-pubkey env var — the test owns both halves of the dual signature
- **VALIDATE**: `cargo test --test e2e actor_app_link` (run via the laptop advisor per the no-cargo-on-EliteDesk rule)

---

## Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md):
integration-only for the M-track; all governance tests live in `crates/server/tests/e2e/`. The
m2-late B-publish test (`m2_late.rs`) is the exact mirror — it already crosses the
Brehon→external-service boundary this feature crosses.

### Tests to Add

| Test Name | What It Validates |
|---|---|
| `link_actor_creates_dual_signed_claim_and_logs` | Happy path — claim minted, signed, logged; `actor_app_link` row + `actor_app_link_created` entry |
| `link_confirm_rejects_bad_app_signature` | Dual-signature defence — bad countersignature → reject, no row |
| `link_confirm_rejects_bad_bearer` | Service-principal auth — no `BRIDGE_CALLBACK_SECRET` → 401, no side-effect |
| `revoke_link_is_prospective_and_logged` | Prospective unlink; chain NOT rewritten; `actor_app_link_revoked` entry |
| `pseudonym_not_raw_identity_in_payload` | ADR-015 — pseudonym (not username) in claim + log payload |

### Edge Cases

- [ ] Hash-chain integrity holds after `actor_app_link_created` + `_revoked` writes (prev_hash links)
- [ ] `actor_pseudonym` is resolved via `get_or_create` (idempotent; reused on repeat link)
- [ ] Dual-signature: BOTH Brehon and app sigs verified; either failure → hard reject
- [ ] Revocation is prospective — `revoked_at` set, historical `_created` entry retained
- [ ] Idempotent re-link on the same `(brehon_actor_id, app_id, app_local_id)` (UniqueViolation → re-read)
- [ ] Nonce single-use + TTL — expired/replayed claim rejected on `/link/confirm`
- [ ] Rollback migration (`diesel migration redo`) works
- [ ] `messaging_enabled = false` or bridge down → `link_actor` POST log-and-swallows (no governance-path failure, ADR-012)

---

## Validation Commands

Use these exact commands — Rust project, do NOT substitute npm/pnpm. Cargo runs through
`/cargo-validate` (preserves exit code, keeps tails out of context).

> **Two-runner discipline (per `feedback_four_tool_review_split` + `feedback_bridge_validates_on_linux_not_windows`):**
> Brehon-workspace crates validate locally on Windows (Tasks 1-10, 13); `services/bridge` (Tasks
> 11-12) validates on **Linux only** via `scripts/brehon/cargo-linux.sh` (Docker `rust:1.95`) — it
> does NOT compile on the Windows host (`ruma-common` E0119 vs `time`). A `services/bridge` cargo
> command in Windows form is itself a DoD issue.

### Level 1: STATIC_ANALYSIS (workspace)

```bash
cargo check --workspace --features full
cargo clippy --workspace --features full -- -D warnings
```
**EXPECT**: Exit 0, zero errors, zero warnings.

### Level 1b: STATIC_ANALYSIS (bridge — Linux)

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml -- -D warnings
```
**EXPECT**: Exit 0; no NEW warnings beyond the 18 pre-existing bridge baseline.

### Level 2: INTEGRATION_TESTS

```bash
cargo test --test e2e actor_app_link
```
**EXPECT**: All 5 tests pass. First run pulls `postgres:16`.

### Level 3: FULL_BUILD

```bash
cargo build --workspace --features full
```
**EXPECT**: Exit 0.

### Level 4: MIGRATION_VALIDATION

```bash
diesel migration run
diesel migration redo
psql -h localhost -U lemmy -d lemmy_test -c '\d actor_app_link'
```
**EXPECT**: Round-trip clean; table has the composite UNIQUE, FK to `actor_pseudonym`, nullable `revoked_at`.

### Level 5: CROSS_CUTTING_VERIFICATION

- [ ] `grep -c 'governance_log::append' crates/api/api/src/governance/actor_app_link.rs` ≥ 2 (link + revoke both log — ADR-008)
- [ ] No raw `person_id`/username/email in any claim or log payload (ADR-015): `person.id` appears ONLY as input to `get_or_create`
- [ ] `link_confirm` rejects on either ed25519 signature failing `verify_strict` (re-point defence)
- [ ] ENTRY_KIND parity: define-side count (`rg -c '^pub const ENTRY_KIND_' …governance_log.rs`) == 69 == shim identifier count
- [ ] registry doc total updated 67→69; collision check clean
- [ ] hash-chain test still passes (the `governance_log` chain-integrity e2e)

### Level 6: MANUAL_VALIDATION (pilot stack)

1. With the pilot Tuwunel+bridge stack up, `curl` `GET /api/v4/governance/link?app_id=matrix&app_local_id=@alice:…` with a valid Brehon JWT cookie.
2. Confirm the bridge receives the claim (bridge logs), countersigns, calls back `/link/confirm`.
3. `psql … -c "SELECT * FROM actor_app_link;"` shows the row; `… WHERE entry_kind = 'actor_app_link_created'` shows the chain entry.
4. `POST /api/v4/governance/link/revoke`; confirm `revoked_at` set + `actor_app_link_revoked` entry; `_created` entry still present.

---

## Acceptance Criteria

- [ ] `actor_app_link` table created with composite UNIQUE + FK to `actor_pseudonym` + nullable `revoked_at`
- [ ] Link flow: authed user → signed claim → app countersign → canonical row + `actor_app_link_created` log
- [ ] Dual-signature enforced: either signature failing → hard reject, no row written
- [ ] Revocation prospective: `revoked_at` set, `actor_app_link_revoked` logged, chain NOT rewritten (ADR-008)
- [ ] ADR-015: pseudonym (never raw identity) in every claim + payload
- [ ] Level 1 (+1b bridge) / 2 / 3 pass with exit 0; no new clippy warnings
- [ ] All 5 e2e tests pass
- [ ] No contradictions with ADR-016 / ADR-008 / ADR-015 / ADR-004
- [ ] OQ-ADR016-03 resolution honoured in all four sub-answers (a/b/c/d)
- [ ] Bridge `services/bridge` compiles + clippy-clean on Linux (`cargo-linux.sh`)

---

## Completion Checklist

- [ ] All 13 tasks completed in dependency order
- [ ] Each task validated immediately (Level 1 / 1b after every change)
- [ ] Level 1: `cargo check --workspace --features full` + clippy pass
- [ ] Level 1b: `cargo-linux.sh check + clippy` on `services/bridge` pass
- [ ] Level 2: 5 e2e tests pass
- [ ] Level 3: `cargo build --workspace --features full` succeeds
- [ ] Level 4: migration round-trip works
- [ ] Level 5: cross-cutting verification passes (ADR-008 + ADR-015 + parity + collision)
- [ ] All acceptance criteria met

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| App countersignature verification wrong (ed25519 `verify_strict` API misuse) | MED | HIGH | The dual-signature is the entire re-point defence (OQ-ADR016-03c). Task 13 includes `link_confirm_rejects_bad_app_signature` as a first-class test; verify `verify_strict` (not `verify`) against the configured app pubkey; the test owns both keypair halves |
| `services/bridge` Windows-compile attempt wastes a cycle | MED | LOW | Tasks 11-12 VALIDATE lines are `cargo-linux.sh` only; the two-runner discipline note is explicit. Per `feedback_bridge_validates_on_linux_not_windows` |
| `load_signing_key` accidentally made `pub` (key-exposure surface) | LOW | HIGH | Task 6 GOTCHA: only `sign_link_claim` is pub; `load_signing_key` stays private. Level-5 check: `grep 'pub fn load_signing_key'` returns nothing |
| Reputation/sanction roll-up still keys off puppet map (mapping built but unused) | MED | MED | Explicitly out of scope (§NOT Building); this plan ships the *mapping + flow*; re-keying the subscriber is a tracked follow-up plan. Note in retro |
| New bridge ed25519 keypair adds key-management surface (`bridge_signing_key` + `brehon_signing_pubkey` env) | MED | MED | Mirror the `GOVERNANCE_LOG_SIGNING_KEY` 32-byte-hex-seed posture (v0 env-var key, external signer is v2/ADR-008). Document both env vars in the bridge config + `.env.example` |
| Nonce replay / TTL not enforced → claim reuse | LOW | HIGH | `link_confirm` checks nonce single-use + `expires_at`; edge-case test covers expired/replayed. Store consumed nonces (in-memory LRU or a small table) for the TTL window |
| Doc 04/06 drift (no `actor_app_link` / B-actor plane row) | LOW | LOW | Doc-drift follow-up: 04 §3 (new table) + 06 §2.2/§7 (B-actor plane boundary + threat row). CODE WINS; flag at plan-ship |

---

## Notes

- **This reverses the 2026-06-07 Phase-7-out-of-scope decision** (user-confirmed 2026-06-13). The
  PRD footer + MEMORY.md + `m2_late_2` workflow-state still record Phase 7 as deferred — update
  those at ship time.
- **OQ-ADR016-03 was resolved 2026-06-13** at this plan's pre-planning gate (lean adopted
  as-written). The four sub-answers are the spec source: (a) OAuth redirect / bridge-callback
  reference, (b) Brehon canonical table + app inverse cache, (c) dual-signed claim, (d) prospective
  unilateral unlink. See [99 §OQ-ADR016-03](../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md).
- **The bridge is the reference "app".** For a real second app (PeerTube, etc.) the link-claim
  auth would shift from `BRIDGE_CALLBACK_SECRET` to a per-app signed registration — that
  first-blocks the first non-Matrix integration ADR (OQ-ADR016-03 `Blocks:` line), out of scope here.
- **`#16a` story-mapping**: this plan predates §16a-story-retrofit for M-plans (a known carry-forward
  from the m2-core-hook retro). If `/brehon-verify` requires §16a stories, retrofit them at brief
  time from the 5 acceptance criteria.
- **The link endpoint is the one governance route combining JWT-in + secret-signed-callback-out** —
  no existing single handler does both; Task 9 composes `request_appeal`'s authed-handler shape with
  `sanction_publisher`'s Bearer-POST-out transport.
</content>
</invoke>
