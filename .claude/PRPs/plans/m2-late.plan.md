# Plan: m2-late — B-publish Sanction Propagation

## 1. Summary

m2-late (M2 Phase 6) adds **B-publish sanction propagation**: when a Brehon
jury reaches quorum and a `sanction_created` governance-log entry fires in
`submit_jury_vote.rs`, the binary delivers a structured, schema-correct
sanction event over an HTTP webhook to every registered subscriber. In
m2-late scope the only subscriber is the Matrix bridge. The deliverable is
(a) a `sanction_event` + `sanction_subscriber` schema with a `sanction_kind`
PG enum, (b) an `enqueue_sanction_event` publisher that fires-and-forgets a
Bearer-authed POST **outside** the vote transaction, (c) a bridge ingest
endpoint that translates the event to Matrix power-level changes and ACKs
200, and (d) two new governance-log entry kinds recording publish success /
failure. **Headline acceptance:** a quorum vote that writes a sanction row
results in a schema-correct `POST` to the seeded bridge callback URL with
`Authorization: Bearer <BRIDGE_CALLBACK_SECRET>`, the subject identified
**only** by `actor_pseudonym.pseudonym` (ADR-015), without blocking or
extending the vote transaction.

## 2. Source

- `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` §"Implementation Phases" (Phase 6) — complete scope reference.
- `.claude/PRPs/briefs/m2-late-planning-1.md` — the authoring brief (WP-1..WP-7, scope tripwires §4, lesson injections §6, task-shape guidance §7).
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` @ `5d05f18a0` — OQ-ADR016-02 (webhook transport + universal schema + at-least-once + Bearer auth) and OQ-ADR016-04 (fixed `SanctionKind` enum + Matrix power-level translation + `not_applicable` ack) resolved 2026-06-07.
- ADR-015 @ `5d05f18a0` — pseudonymity: subject identifier MUST be `actor_pseudonym.pseudonym`, never `person.name` / `local_user.email` / `local_user.actor_id`.
- ADR-016 @ `5d05f18a0` — B-publish sanction propagation contract.
- Lessons that bind decisions in this plan:
  - `feedback_validate_pending_laptop_write_then_stop.md` — workers write `validate-pending-laptop` DQ and STOP; never run workspace cargo on the daemon.
  - `feedback_lemmy_migration_runner.md` — migration runner is `cargo run -p lemmy_diesel_utils --features full -- migration run`, NOT `diesel migration run`.
  - `feedback_multi_write_handlers_need_transactions.md` — the spawn is OUTSIDE the transaction; transaction integrity is unchanged.
  - `feedback_lemmy_error_no_std_error.md` + `feedback_async_pool_test_pattern.md` + `feedback_fix_impl_pre_locate_e2e_anchors.md` — e2e task (T8) discipline.
  - `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — newtype lands in the single file `crates/db_schema/src/newtypes.rs` (brief path corrected — see §19).
  - `feedback_features_full_workspace_only.md` — `--features full` is workspace-only.

## 3. Problem statement

Before m2-late, a quorum decision writes a `sanction` row + a `sanction_created`
governance-log entry and stops. Nothing propagates the decision to the Matrix
bridge, so a sanctioned actor's room power levels are never adjusted. Six gaps:

1. No `sanction_kind` vocabulary exists in the DB or Rust type system → §10.1, T1.
2. No `sanction_event` table to record what was published, to whom, with what pseudonymous subject → §10.4, T2.
3. No `sanction_subscriber` registry (the bridge's callback URL has nowhere to live) → §10.4, T2 + §10.8, T6.
4. No `SanctionAction → SanctionKind` mapping; the 8 `SanctionAction` variants must map exhaustively (no wildcard) → §10.2, T3.
5. No publisher: nothing reads subscribers, builds the payload, or POSTs it; and no emit hook fires after `sanction_created` → §10.3, T4 + T5.
6. The bridge has no ingest route to receive the event and translate it to power levels → §10.7, T7.

## 4. Solution statement

```
process_vote (submit_jury_vote.rs)                    [unchanged transaction body]
  └─ tx { … insert sanction; governance_log::append("sanction_created") }  ← T-existing
  └─ (after tx commit) re-query the just-written sanction by case_id  ← T5
        └─ Some(sanction with target_person_id) → tokio::spawn(            ← T5 (ONE spawn site)
              enqueue_sanction_event(sanction)  ← T4 (api/governance/sanction_publisher.rs)
                ├─ map SanctionAction → Option<SanctionKind>  ← T3 (sanction_kind_map.rs)
                │     (None ⇒ skip delivery, tracing::debug, return)
                ├─ resolve subject = actor_pseudonym.pseudonym (ADR-015)
                ├─ read active rows from sanction_subscriber  ← T2 table
                ├─ build SanctionEventPayload, POST Bearer <SECRET> to each url
                └─ governance_log::append("sanction_published" | "sanction_event_delivery_failed")  ← T3 consts
           )
        └─ None (appeal-only / no-sanction / non-person target) → no spawn

bridge: POST /brehon/sanction-event (axum)  ← T7 (services/bridge/src/sanction_handler.rs)
  ├─ verify Authorization: Bearer <BRIDGE_CALLBACK_SECRET>
  ├─ deserialise SanctionEventPayload (LOCAL re-declaration — no api-crate dep)
  ├─ bridge_room store lookup: subject → matrix_room_id rows
  ├─ translate sanction_kind → Matrix power-level change per room
  └─ 200 { applied, reason, applied_at }

startup (crates/server/src/lib.rs async region)  ← T6
  └─ if BRIDGE_SANCTION_CALLBACK_URL set: idempotent INSERT … ON CONFLICT DO NOTHING into sanction_subscriber
```

The reader can predict §11 from this diagram: db_schema (enum + 2 models + schema.rs + newtype), api (2 consts + shim + map + publisher + 1-line spawn in submit_jury_vote), crates/server (startup seed), services/bridge (handler + route), one combined migration dir, one e2e test file + wiring.

## 5. Metadata

- **Phase:** `m2-late`
- **Branch:** `phase-m2-late` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 10 (Task 0 pre-flight + T1–T8 impl + T9 retro)
- **Estimated cargo budget:** `~6 GB peak` (workspace check with `--features full`; pre-Shape-G, runs on laptop via `validate-pending-laptop`)
- **Forbidden-window applicability:** standard (per advisor-orchestrator.md table) — binding because this is a pre-Shape-G plan (workspace cargo runs on the laptop).
- **Complexity score:** `13/10` — see breakdown below. **Exceeds the Sonnet threshold of 8 → split-DQ filed (§19).**

### 5.1 Complexity factor breakdown

Per `feedback_complexity_score_pre_split.md`. Target is `sonnet-4-6`, so the split-DQ threshold is `score > 8`.

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 3 | 8 impl tasks (T1–T8); 8 − 5 = 3 |
| Migrations touched | +2 each | 2 | 1 combined migration dir (enum + 2 tables) — counted once |
| Crates touched | +1 each | 5 | `db_schema_file`, `db_schema`, `api/api`, `crates/server`, `services/bridge` |
| `crates/server/tests/e2e/*.rs` edits | +3 each | 3 | T8 — one new `e2e/m2_late.rs` + `include!` in `e2e.rs` |
| New ADR-affecting decisions | +2 each | 0 | OQ-ADR016-02 + OQ-ADR016-04 already resolved @ `5d05f18a0`; this plan implements, supersedes nothing |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Peak ~6 GB, not above |
| **Total** | — | **13** | Threshold for split-DQ: `>8` (Sonnet) → **fires** |

Migration counted as 2 (one combined dir, still a schema migration with round-trip + rollback cost). Score 13 > 8 → the planner files a `pending` `from: "planner"`, `kind: "blocker"` split-DQ (§19) proposing **m2-late-1** (T0–T4: additive machinery — publisher compiles but is never called; behaviour unchanged) + **m2-late-2** (T5–T9: wire it live + e2e + retro). Sub-scores: m2-late-1 ≈ 5, m2-late-2 ≈ 6 — both under threshold.

### 5.2 Per-task complexity ceiling (Sonnet target: ≤4 files / ≤2 crates)

| Task | union(creates, modifies) files | distinct crate prefixes | OK? |
|---|---|---|---|
| T1 | 4 (migration up/down, enums.rs, schema.rs) | 1 (`db_schema_file` + root `migrations/`; schema.rs is `db_schema_file`) | ✅ |
| T2 | 4 (sanction_event.rs, sanction_subscriber.rs, source/governance/mod.rs, newtypes.rs) | 1 (`db_schema`) | ✅ |
| T3 | 4 (db_schema governance_log.rs, api shim governance_log.rs, sanction_kind_map.rs, api governance/mod.rs) | 2 (`db_schema`, `api/api`) | ✅ |
| T4 | 2 (sanction_publisher.rs, api governance/mod.rs) | 1 (`api/api`) | ✅ |
| T5 | 1 (submit_jury_vote.rs) | 1 (`api/api`) | ✅ |
| T6 | 1 (crates/server/src/lib.rs) | 1 (`crates/server`) | ✅ |
| T7 | 3 (sanction_handler.rs, appservice.rs, main.rs) | 1 (`services/bridge`) | ✅ |
| T8 | 2 (e2e/m2_late.rs, e2e.rs) | 1 (`crates/server`) | ✅ |

All tasks satisfy the Sonnet ceiling (≤4 files / ≤2 crates). T8 puts the e2e edit in its own dedicated task (no non-test logic bundled).

## 6. Relationship to other m2 sub-phases

- **Depends on m2-core-hook** — the `governance_log` const-registry pattern + the 10 room kinds shipped there; this plan adds consts 66–67. The registry stands at 65 consts (per `governance-log-entry-kind-registry.md` acceptance invariant) and rises to **67** with `sanction_published` + `sanction_event_delivery_failed`.
- **Depends on m2-rooms-a** — `services/bridge/src/bridge_room.rs` (`BridgeRoomStore` SQLite lookup, `case_id → matrix_room_id`) and `services/bridge/Cargo.toml`'s `rusqlite` dep; T7's handler queries the store for the subject's rooms.
- **Followed by Phase 7 (B-actor portable-ID linkage)** — **OUT OF SCOPE**, user-confirmed 2026-06-07 (OQ-ADR016-03 deferred). See §12.

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`).
- **R2:** all clippy invocations use `--no-deps` uniformly (avoid upstream `lemmy_*` lint debt masking own code).
- **R3:** all clippy/check invocations on governance crates use `--features full` (governance code is behind `#[cfg(feature = "full")]`).
- **R4:** Task 0 enumerates ALL probes explicitly; do NOT inherit implicitly.
- **R5:** migrations live at the **repository ROOT** `migrations/<YYYY-MM-DD-HHMMSS-0000_name>/{up,down}.sql` (brief said `crates/db_schema/migrations/` — corrected; see §19). The `sanction_kind` PG enum and the `sanction_event` table that references it must exist in dependency order within the migration; Diesel schema is regenerated via the runner before any Rust model references the new tables.
- **R6:** the `SanctionEventId` newtype lands in the single file `crates/db_schema/src/newtypes.rs` (brief implied a `newtypes/` directory — corrected; see §19).
- **R7:** workers WRITE the `validate-pending-laptop` DQ entry and STOP; they do NOT run workspace cargo on the daemon (per `feedback_validate_pending_laptop_write_then_stop.md`). Bridge cargo (`cd services/bridge && cargo check`) is the one in-task exception (workspace-excluded crate).
- **R8:** the spawn is OUTSIDE `conn.run_transaction()`; the governance-log append inside the spawned task uses a NEW pool connection, never the transaction's `conn`.
- **R9:** `services/bridge` stays in the root `Cargo.toml` `exclude` array — never added to `members`. The zero-Matrix-deps gate (`cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` ⇒ 0) is in every bridge-task DoD.

## 8. Flow design

**Before (m2-core-hook HEAD):**

```
process_vote: tx { insert sanction; append "sanction_created" }  →  (returns; nothing propagates)
```

**After (m2-late):**

```
process_vote
  tx { insert sanction; append "sanction_created" }            [T-existing, unchanged]
  └─ after commit:
       re-query sanction WHERE case_id = data.case_id          [T5, mirrors :163-178 re-query]
       match { Some(s) if s.target_person_id.is_some() =>
                 tokio::spawn(enqueue_sanction_event(s, context)) [T5 — ONE spawn site]
               _ => /* skip: appeal-only, no-sanction, or non-person target */ }

enqueue_sanction_event(sanction, ctx)                          [T4, sanction_publisher.rs]
  ├─ map_sanction_action(sanction.action) -> Option<SanctionKind> [T3, exhaustive]
  │     None ⇒ tracing::debug + return                          (FederationQuarantineRecommendation, Restoration)
  ├─ subject = actor_pseudonym::lookup(sanction.target_person_id) [ADR-015]
  ├─ subscribers = sanction_subscriber WHERE active            [T2]
  ├─ payload = SanctionEventPayload { sanction_kind, subject_actor_pseudonym,
  │             effective_from, effective_until, governance_log_entry_hash }
  ├─ for each: reqwest POST Bearer <BRIDGE_CALLBACK_SECRET>
  └─ append "sanction_published" (≥1 ok) | "sanction_event_delivery_failed" [T3 consts, new pool conn]

bridge POST /brehon/sanction-event                             [T7, axum]
  verify Bearer → deserialise → bridge_room lookup → PL change per room → 200 ack
```

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **Schema/type definitions:**
  - `crates/db_schema_file/src/enums.rs:531-577` — `SanctionScope` (DbEnum mirror) + `SanctionAction` (8 variants — the mapping source).
  - `crates/db_schema_file/src/schema.rs:132-138` — `sql_types::SanctionAction` / `SanctionScope` struct pattern (mirror for `SanctionKind`).
  - `crates/db_schema/src/source/governance/sanction.rs:1-48` — `Sanction` + `SanctionInsertForm` (model + insert-form mirror; key fields `target_person_id: Option<PersonId>`, `starts_at`, `ends_at: Option`).
  - `crates/db_schema/src/source/governance/governance_log.rs:112-140` — `ENTRY_KIND_*` const block; `ENTRY_KIND_SANCTION_CREATED` at line 117 (sibling for the 2 new consts); `entry_hash` is `Vec<u8>` (model field `String`, payload field `hex::encode`-d).
- **Existing patterns:**
  - `crates/api/api/src/governance/submit_jury_vote.rs:452-480` — step-8 sanction emission (the `map_decision_to_sanction` block, the `sanction_created` append at 470-480).
  - `crates/api/api/src/governance/submit_jury_vote.rs:163-178` — `governance_case_after_transition` post-transaction re-query (the MIRROR for T5's re-query).
  - `crates/api/api/src/governance/submit_jury_vote.rs:1059-1061` — "No sanction row, no federation outbound" (`process_appeal_vote` writes no sanction → no spawn there).
  - `crates/api/api/src/governance/bridge_auth.rs:6-18` — `verify_bridge_secret` (Bearer-auth mirror).
  - `crates/server/src/lib.rs:142,340-410` — `start_lemmy_server` async fn + the `schedule_governance_jobs` composition-root invocation at line 352 (mirror for T6's startup seed placement, but seed goes in the ASYNC startup region, after migrations).
  - `services/bridge/src/room_provisioner.rs` + `services/bridge/src/appservice.rs` (`router()` + `AppState`) + `services/bridge/src/bridge_room.rs` (`BridgeRoomStore`).
- **Adjacent test fixtures:**
  - `crates/server/tests/e2e.rs:124-148` — `mod common; use common::governance_fixtures;` + the `include!("e2e/<name>.rs")` wiring pattern (governance at :133).
  - Sibling e2e modules `crates/server/tests/e2e/{governance,jury_mechanics,sponsor_liability}.rs` — mirror the LemmyResult error-shape case verbatim (per `feedback_lemmy_error_no_std_error.md`).
- **Lessons:** all listed in §2 + §6 (the brief's per-task injections — applied per-task in §13).

## 10. Patterns to mirror

### 10.1 `SanctionKind` DbEnum + `sql_types` struct

**Mirror:** `crates/db_schema_file/src/enums.rs:531-547` (`SanctionScope`) + `crates/db_schema_file/src/schema.rs:136-138` (`sql_types::SanctionScope`).

```rust
// crates/db_schema_file/src/enums.rs — new enum, mirror SanctionScope verbatim
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(feature = "full", ExistingTypePath = "crate::schema::sql_types::SanctionKind")]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The platform-neutral kind of a published sanction (B-publish, ADR-016).
pub enum SanctionKind {
  #[default]
  PreventPost,
  MuteVoice,
  HideContent,
  RestrictReach,
}
```

```rust
// crates/db_schema_file/src/schema.rs — sql_types block, mirror SanctionScope
#[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
#[diesel(postgres_type(name = "sanction_kind"))]
pub struct SanctionKind;
```

`schema.rs` is normally regenerated by `print-schema`; the impl agent runs the migration runner (per `feedback_lemmy_migration_runner.md`) so the regenerated `schema.rs` carries this `sql_types::SanctionKind` block + the two new tables. The hand-written `enums.rs` enum carries the `ExistingTypePath` binding.

### 10.2 Exhaustive `SanctionAction → Option<SanctionKind>` map (no wildcard)

**Mirror:** `crates/db_schema_file/src/enums.rs:560-577` (the 8 `SanctionAction` variants).

```rust
// crates/api/api/src/governance/sanction_kind_map.rs — new module
use lemmy_db_schema_file::enums::{SanctionAction, SanctionKind};

/// Canonical v0 mapping from the legal SanctionAction taxonomy to the
/// platform-neutral SanctionKind published over B-publish. Exhaustive match
/// (no `_ =>`) so a future SanctionAction variant forces an explicit decision.
/// `None` ⇒ no local Matrix primitive ⇒ skip delivery (not an error).
pub fn map_sanction_action(action: SanctionAction) -> Option<SanctionKind> {
  match action {
    SanctionAction::Label => Some(SanctionKind::RestrictReach),
    SanctionAction::VisibilityReduction => Some(SanctionKind::RestrictReach),
    SanctionAction::TemporaryRestriction => Some(SanctionKind::PreventPost),
    SanctionAction::ContentRemoval => Some(SanctionKind::HideContent),
    SanctionAction::CommunityExclusion => Some(SanctionKind::PreventPost),
    SanctionAction::InstanceSuspension => Some(SanctionKind::PreventPost),
    SanctionAction::FederationQuarantineRecommendation => None,
    SanctionAction::Restoration => None,
  }
}
```

`MuteVoice` is a valid `SanctionKind` variant but is **not produced** by any v0 `SanctionAction` (intentional — no `SanctionAction` carries voice-only semantics in v0; see §19). It exists for the bridge's translation table and for future actions.

### 10.3 Post-transaction re-query + single spawn site

**Mirror:** `crates/api/api/src/governance/submit_jury_vote.rs:163-178` (`governance_case_after_transition` re-query) + `:452-480` (the sanction emission inside the tx).

```rust
// In process_vote, AFTER conn.run_transaction(...) returns Ok — NOT inside the tx.
// Re-query the sanction the transaction just wrote (filter by case_id). Returns
// None for appeal-only / no-sanction / NoAction cases, so the spawn skips naturally.
let published: Option<Sanction> = sanction::table
  .filter(sanction::case_id.eq(data.case_id))
  .filter(sanction::active.eq(true))
  .first::<Sanction>(&mut context.pool().get().await?)
  .await
  .optional()?;

if let Some(sanction) = published {
  if sanction.target_person_id.is_some() {
    let ctx = context.clone();          // SanctionContext = pool handle + env (secret, urls)
    tokio::spawn(async move {
      if let Err(e) = enqueue_sanction_event(sanction, ctx).await {
        tracing::warn!("sanction publish failed: {e}");
      }
    });
  }
}
```

**Design rationale (LOCKED, see §19):** re-query is chosen over threading the
`(scope, action, sanction_id)` tuple through `process_vote`'s 5 return sites
({244, 308, 336, 434, 860}); threading touches the control flow of the riskiest
governance file. The re-query mirrors an existing pattern in the same file.

### 10.4 Combined migration (enum before table)

**Mirror:** existing PG-enum migrations under ROOT `migrations/` (e.g. the `sanction_action` / `sanction_scope` enum migration from Phase 4).

`migrations/2026-06-07-000000-0000_add_sanction_event/up.sql`:

```sql
CREATE TYPE sanction_kind AS ENUM ('prevent_post', 'mute_voice', 'hide_content', 'restrict_reach');

CREATE TABLE sanction_event (
  id SERIAL PRIMARY KEY,
  sanction_id INTEGER NOT NULL REFERENCES sanction(id) ON DELETE CASCADE,
  sanction_kind sanction_kind NOT NULL,
  subject_actor_pseudonym TEXT NOT NULL,          -- actor_pseudonym.pseudonym ONLY (ADR-015)
  effective_from TIMESTAMPTZ NOT NULL,
  effective_until TIMESTAMPTZ NULL,
  governance_log_entry_hash TEXT NOT NULL          -- hex-encoded governance_log.entry_hash
);

CREATE TABLE sanction_subscriber (
  id SERIAL PRIMARY KEY,
  callback_url TEXT NOT NULL UNIQUE,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

`down.sql` drops in reverse order: `DROP TABLE sanction_subscriber; DROP TABLE sanction_event; DROP TYPE sanction_kind;`. Combining into one migration dir is permitted by WP-3 ("Alternatively, combine into one migration file") and keeps the enum strictly before the table that references it.

### 10.5 Diesel models + insert forms

**Mirror:** `crates/db_schema/src/source/governance/sanction.rs:1-48`.

`sanction_event.rs` — `SanctionEvent` (`#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]`, `diesel(table_name = sanction_event)`, ts-rs cfg_attr) + `SanctionEventInsertForm`. `id: SanctionEventId` newtype; `governance_log_entry_hash: String`. `sanction_subscriber.rs` — `SanctionSubscriber` + `SanctionSubscriberInsertForm` (`callback_url: String`, `active: bool`).

### 10.6 governance-log consts + api shim

**Mirror:** `crates/db_schema/src/source/governance/governance_log.rs:117` (`ENTRY_KIND_SANCTION_CREATED`) + the api shim re-export pattern in `crates/api/api/src/governance/governance_log.rs`.

```rust
// crates/db_schema/src/source/governance/governance_log.rs — append to const block
pub const ENTRY_KIND_SANCTION_PUBLISHED: &str = "sanction_published";
pub const ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED: &str = "sanction_event_delivery_failed";
```

Matching `pub use` lines (alphabetical) in the api shim. Registry section in `.claude/rules/governance-log-entry-kind-registry.md` is **advisor meta-work** (WP-7) — Junior writes ONLY the 2 `crates/` files; the registry rises 65 → 67.

### 10.7 Bridge handler + route

**Mirror:** `services/bridge/src/room_provisioner.rs` (axum + reqwest + `BRIDGE_CALLBACK_SECRET`) + `services/bridge/src/appservice.rs` (`router()` + `AppState`) + `services/bridge/src/bridge_room.rs` (`BridgeRoomStore`).

`services/bridge/src/sanction_handler.rs` (new): a LOCAL `SanctionEventPayload` struct (serde; **no api-crate dependency** — bridge is workspace-excluded and matrix-free); `handle_sanction_event(State, headers, Json<payload>) -> impl IntoResponse` verifies `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` (first line, mirroring `room_event_handler`'s bridge_auth-first shape), looks up rooms via `BridgeRoomStore`, translates `sanction_kind` → power-level change, returns `200 { applied, reason, applied_at }`. Route `POST /brehon/sanction-event` wired in `appservice.rs::router()`; `mod sanction_handler;` in `main.rs`. Translation: `prevent_post`/`hide_content`/`restrict_reach` → PL below `events_default`; `mute_voice` → PL below voice threshold (or unchanged if no voice rooms).

### 10.8 Idempotent startup subscriber seed

**Mirror:** `crates/server/src/lib.rs:340-410` (`start_lemmy_server` async region; `schedule_governance_jobs` at :352).

In the ASYNC startup region (after migrations), if `BRIDGE_SANCTION_CALLBACK_URL` is set: call `sanction_publisher::seed_sanction_subscriber(&url, pool)` which runs `INSERT INTO sanction_subscriber (callback_url, active) VALUES ($1, true) ON CONFLICT (callback_url) DO NOTHING`. Absent env var ⇒ no-op (clean unconfigured posture). The seed LOGIC lives in the api crate (`sanction_publisher.rs`, T4); the INVOCATION lives in `lemmy_server` startup (T6) — see the seed-placement log-DQ in §19.

## 11. Files to change

**`crates/db_schema_file/`** (crate 1):
- `migrations/2026-06-07-000000-0000_add_sanction_event/up.sql` + `down.sql` — `sanction_kind` enum + `sanction_event` + `sanction_subscriber` (Task 1). *(ROOT `migrations/`, not under the crate — listed here for crate-grouping of the schema change.)*
- `crates/db_schema_file/src/enums.rs` — new `SanctionKind` enum (Task 1).
- `crates/db_schema_file/src/schema.rs` — regenerated `sql_types::SanctionKind` + `sanction_event` + `sanction_subscriber` tables (Task 1, via migration runner).

**`crates/db_schema/`** (crate 2):
- `crates/db_schema/src/source/governance/sanction_event.rs` — `SanctionEvent` model + insert form (Task 2).
- `crates/db_schema/src/source/governance/sanction_subscriber.rs` — `SanctionSubscriber` model + insert form (Task 2).
- `crates/db_schema/src/source/governance/mod.rs` — `pub mod sanction_event; pub mod sanction_subscriber;` (Task 2).
- `crates/db_schema/src/newtypes.rs` — `SanctionEventId` newtype (Task 2). *(single file — see §19 brief-correction.)*
- `crates/db_schema/src/source/governance/governance_log.rs` — 2 new `ENTRY_KIND_*` consts (Task 3).

**`crates/api/api/`** (crate 3):
- `crates/api/api/src/governance/governance_log.rs` — shim `pub use` for the 2 consts (Task 3).
- `crates/api/api/src/governance/sanction_kind_map.rs` — exhaustive map (Task 3, new).
- `crates/api/api/src/governance/mod.rs` — `pub mod sanction_kind_map; pub mod sanction_publisher;` (Task 3, Task 4).
- `crates/api/api/src/governance/sanction_publisher.rs` — `enqueue_sanction_event`, `seed_sanction_subscriber`, `SanctionEventPayload`, `SanctionContext` (Task 4, new; additive — not wired).
- `crates/api/api/src/governance/submit_jury_vote.rs` — post-tx re-query + single `tokio::spawn` site (Task 5).

**`crates/server/`** (crate 4):
- `crates/server/src/lib.rs` — startup seed invocation in the async region (Task 6).
- `crates/server/tests/e2e/m2_late.rs` — new e2e test (Task 8, new).
- `crates/server/tests/e2e.rs` — `include!("e2e/m2_late.rs")` (Task 8).

**`services/bridge/`** (crate 5, workspace-excluded):
- `services/bridge/src/sanction_handler.rs` — handler + LOCAL payload (Task 7, new).
- `services/bridge/src/appservice.rs` — `POST /brehon/sanction-event` route (Task 7).
- `services/bridge/src/main.rs` — `mod sanction_handler;` (Task 7).

### Struct-field add: enumerate all callsites

No task adds a field to an existing **public** struct. The new structs (`SanctionEvent`, `SanctionSubscriber`, `SanctionEventPayload`, `SanctionContext`) are introduced whole, so no callsite enumeration is required. `SanctionKind` is a new enum (new variants only, no field add to an existing type). The exhaustive map in §10.2 is the only place `SanctionAction` is re-matched; it is authored complete.

## 12. NOT building in m2-late

- **Phase 7 (B-actor portable-ID linkage)** — `actor_app_link` table, OAuth redirect, dual-signed claim; OUT OF SCOPE, user-confirmed 2026-06-07 (OQ-ADR016-03 deferred). STOP-and-ask tripwire (brief §4).
- **Subscriber-registration HTTP endpoint** (`POST /governance/sanction-subscriber`) — deferred; the bridge is the only subscriber, seeded via env var + idempotent INSERT. STOP tripwire.
- **Retry queue / scheduler** (`sanction_event_retry` table, background re-delivery) — at-least-once via fire-and-forget is the v0 contract; retry is M3/future. STOP tripwire.
- **ACK storage** — the bridge POSTs an ack back, but Brehon does not store per-event per-app status; the governance-log entry is the v0 audit trail (per-app status table is M3).
- **Non-Matrix subscribers / generic subscriber management** — only the bridge.
- **Any `e2e.rs` edit in Tasks 1–7** — all e2e touches are isolated to Task 8 (WP-6).
- **Blocking the vote transaction on webhook I/O** — the spawn is outside the tx (WP-2). STOP tripwire.
- **A second spawn site in `process_appeal_vote`** — it writes no sanction row (`submit_jury_vote.rs:1059-1061`). STOP tripwire.

---

## 13. Step-by-step tasks

> **Cohort dispatch (advisor-side):** consecutive `[P]` tasks form a cohort, dispatched simultaneously each on its own worktree, degraded to serial above the EliteDesk cargo budget / shared-`.git/index.lock` hazard (cohort ≥3) / the cross-lane cap of 2. T2+T3 are a `[P]` pair (file-disjoint, both `requires: T1`). T5+T6+T7 are `[P]` (file-disjoint, all `requires: T4`) but the advisor degrades the 3-member cohort to serial per the index.lock hazard + cap-of-2.
>
> **Pre-Shape-G:** every workspace-cargo task writes a `validate-pending-laptop` DQ entry and STOPS (per `feedback_validate_pending_laptop_write_then_stop.md`). Bridge cargo (`cd services/bridge && cargo check`) runs in-task (workspace-excluded). One commit per task.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `m2-late`; confirm branch is `phase-m2-late`; confirm prior deliverables intact; confirm clippy baseline clean.

**Probes (R4 — enumerate ALL explicitly; Linux/EliteDesk wrapper form):**

```bash
# Probe 0 — Docker daemon (needed by T8 e2e testcontainers)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }
# Probe 1 — branch
git branch --show-current   # EXPECT: phase-m2-late
# Probe 2 — wrapper honors -p (no flag-discard)
./scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/PRPs/debug/m2-late-audit-p.log 2>&1; echo "exit: $?"
# Probe 3 — wrapper honors --features full
./scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full > .claude/PRPs/debug/m2-late-audit-features.log 2>&1; echo "exit: $?"
# Probe 4 — negative: wrapper propagates non-zero on bogus feature
./scripts/brehon/cargo-check.sh -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m2-late-audit-neg.log 2>&1; echo "exit (EXPECT non-zero): $?"
# Probe 5 — bridge crate compiles standalone (m2-rooms-a baseline)
cd services/bridge && cargo check > /tmp/m2-late-audit-bridge.log 2>&1; echo "exit: $?"; cd -
# Probe 6 — zero-Matrix-deps gate baseline
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'   # EXPECT: 0
# Probe 7 — no sanction_event table yet (T1 adds it)
grep -c 'sanction_event' crates/db_schema_file/src/schema.rs   # EXPECT: 0
# Probe 8 — bridge_room.rs present (m2-rooms-a) + no sanction route yet
ls services/bridge/src/bridge_room.rs && ! grep -rq 'brehon/sanction-event' services/bridge/src/ && echo "ROUTE ABSENT OK"
# Probe 9 — SanctionAction has 8 variants (mapping source)
grep -cE '^  (Label|VisibilityReduction|TemporaryRestriction|ContentRemoval|CommunityExclusion|InstanceSuspension|FederationQuarantineRecommendation|Restoration),' crates/db_schema_file/src/enums.rs   # EXPECT: 8
```

**EXPECT block:** Probes 0–3,5–9 exit 0 / match; Probe 4 exits NON-ZERO. **No commit at Task 0.**

### Task 1: Migration + `SanctionKind` enum + schema regen

**ACTION:** create the combined migration dir, add the `SanctionKind` Rust enum, regenerate `schema.rs`.

```yaml
creates:
  - migrations/2026-06-07-000000-0000_add_sanction_event/up.sql
  - migrations/2026-06-07-000000-0000_add_sanction_event/down.sql
modifies:
  - crates/db_schema_file/src/enums.rs       # add SanctionKind enum (mirror SanctionScope)
  - crates/db_schema_file/src/schema.rs      # regenerated: sql_types::SanctionKind + 2 tables
```

**IMPLEMENT (1/4):** `up.sql` per §10.4 (enum strictly before tables). **(2/4):** `down.sql` drops in reverse. **(3/4):** `enums.rs` `SanctionKind` per §10.1 (verbatim `SanctionScope` mirror). **(4/4):** run the migration runner so `schema.rs` regenerates.

**MIRROR:** `enums.rs:531-547`; `schema.rs:136-138`; the Phase-4 `sanction_action` enum migration.

**GOTCHA:** migrations live at ROOT `migrations/` (R5), NOT `crates/db_schema/migrations/`. Migration runner is `cargo run -p lemmy_diesel_utils --features full -- migration run`, NOT `diesel migration run` (`feedback_lemmy_migration_runner.md`).

**VALIDATE (validate-pending-laptop DQ, then STOP):**
```json
{ "kind": "validate-pending-laptop", "commands": ["cargo run -p lemmy_diesel_utils --features full -- migration run", "./scripts/brehon/cargo-check.sh --workspace --features full"], "branch": "phase-m2-late", "phase_task": 1 }
```

### Task 2 [P]: Diesel models + newtype

**ACTION:** add `SanctionEvent` / `SanctionSubscriber` models + insert forms + `SanctionEventId` newtype. `requires: T1`.

```yaml
creates:
  - crates/db_schema/src/source/governance/sanction_event.rs
  - crates/db_schema/src/source/governance/sanction_subscriber.rs
modifies:
  - crates/db_schema/src/source/governance/mod.rs   # pub mod sanction_event; pub mod sanction_subscriber;
  - crates/db_schema/src/newtypes.rs                # add SanctionEventId
requires:
  - task: 1
    reason: models reference schema.rs sanction_event/sanction_subscriber tables + sql_types::SanctionKind created by T1
```

**IMPLEMENT:** mirror `sanction.rs:1-48` for both models (Identifiable/Queryable/Selectable under `feature="full"`, `diesel(table_name=...)`, ts-rs cfg_attr). `governance_log_entry_hash: String`.

**MIRROR:** `crates/db_schema/src/source/governance/sanction.rs:1-48`; existing newtype in `crates/db_schema/src/newtypes.rs`.

**GOTCHA:** `[P]` with T3 — disjoint files (T2 = models + db_schema mod.rs + newtypes.rs; T3 = governance_log.rs + api files). Newtype is the single file `newtypes.rs` (R6), NOT a `newtypes/` dir.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 2`, then STOP.

### Task 3 [P]: governance-log consts + shim + exhaustive map

**ACTION:** add 2 `ENTRY_KIND_*` consts + shim re-exports + the `sanction_kind_map` module. `requires: T1`.

```yaml
creates:
  - crates/api/api/src/governance/sanction_kind_map.rs
modifies:
  - crates/db_schema/src/source/governance/governance_log.rs   # 2 new consts
  - crates/api/api/src/governance/governance_log.rs            # 2 new pub use (alphabetical)
  - crates/api/api/src/governance/mod.rs                       # pub mod sanction_kind_map;
requires:
  - task: 1
    reason: sanction_kind_map references SanctionKind enum created by T1
```

**IMPLEMENT:** consts per §10.6; shim `pub use`; `sanction_kind_map.rs` per §10.2 (exhaustive, no wildcard).

**MIRROR:** `governance_log.rs:117` (`ENTRY_KIND_SANCTION_CREATED`) + api shim re-export block.

**GOTCHA:** registry update in `.claude/rules/governance-log-entry-kind-registry.md` is advisor meta-work (WP-7) — Junior does NOT touch it; leave a NOTE to the advisor. Map MUST be exhaustive so a future `SanctionAction` variant fails to compile until mapped. `[P]` with T2 (disjoint).

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 3`, then STOP.

### Task 4: Publisher module (additive, not wired)

**ACTION:** add `sanction_publisher.rs` (`enqueue_sanction_event`, `seed_sanction_subscriber`, `SanctionEventPayload`, `SanctionContext`). `requires: T2, T3`. Additive — compiles but no caller yet.

```yaml
creates:
  - crates/api/api/src/governance/sanction_publisher.rs
modifies:
  - crates/api/api/src/governance/mod.rs   # pub mod sanction_publisher;
requires:
  - task: 2
    reason: reads sanction_subscriber + writes sanction_event via T2 models
  - task: 3
    reason: uses map_sanction_action (T3) + ENTRY_KIND_SANCTION_PUBLISHED consts (T3)
```

**IMPLEMENT:** `enqueue_sanction_event(sanction, ctx)` per §10.3 body — map (None ⇒ debug+return), resolve `actor_pseudonym.pseudonym` (ADR-015), read active subscribers, build payload, reqwest POST Bearer `<BRIDGE_CALLBACK_SECRET>`, append `sanction_published` (≥1 ok) | `sanction_event_delivery_failed` via a NEW pool connection (R8). `seed_sanction_subscriber(url, pool)` per §10.8 (idempotent ON CONFLICT). `SanctionEventPayload { sanction_kind, subject_actor_pseudonym, effective_from, effective_until, governance_log_entry_hash }`.

**MIRROR:** `bridge_auth.rs:6-18` (Bearer header shape); `governance_log.rs` append signature; `submit_jury_vote.rs:163-178` (pool re-query).

**GOTCHA:** `entry_hash` is `Vec<u8>` on the row → `hex::encode` for the `String` payload field. Non-person target ⇒ caller skips (handled at the T5 spawn guard); `enqueue` itself also guards `None` map.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 4`, then STOP.

### Task 5 [P]: Wire the spawn site in `submit_jury_vote.rs`

**ACTION:** add the post-transaction re-query + single `tokio::spawn(enqueue_sanction_event(...))` in `process_vote` only. `requires: T4`.

```yaml
modifies:
  - crates/api/api/src/governance/submit_jury_vote.rs   # post-tx re-query + ONE spawn site
requires:
  - task: 4
    reason: calls enqueue_sanction_event + SanctionContext from T4
```

**IMPLEMENT:** per §10.3 — after `conn.run_transaction(...)` returns Ok in `process_vote`, re-query the sanction by `case_id`, guard `target_person_id.is_some()`, spawn. NOT inside the tx (R8/WP-2). Exactly ONE spawn site; do NOT wire `process_appeal_vote` (`:1059-1061`).

**MIRROR:** `submit_jury_vote.rs:163-178` (re-query) + `:452-480` (sanction emission context).

**GOTCHA:** the riskiest governance file — change is additive (a re-query + guarded spawn after commit), it does NOT alter the 5 return sites or the transaction body. `[P]` with T6/T7 (disjoint files) but advisor degrades to serial (index.lock/cap-of-2).

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 5`, then STOP.

### Task 6 [P]: Startup subscriber seed

**ACTION:** invoke `seed_sanction_subscriber` from `lemmy_server` startup async region when `BRIDGE_SANCTION_CALLBACK_URL` is set. `requires: T4`.

```yaml
modifies:
  - crates/server/src/lib.rs   # idempotent seed in async startup region, after migrations
requires:
  - task: 4
    reason: calls seed_sanction_subscriber from T4
```

**IMPLEMENT:** per §10.8 — read `BRIDGE_SANCTION_CALLBACK_URL`; if set, `sanction_publisher::seed_sanction_subscriber(&url, pool).await`; absent ⇒ no-op.

**MIRROR:** `crates/server/src/lib.rs:340-410` (`start_lemmy_server` async region; `schedule_governance_jobs` at :352 for placement — but seed goes in the ASYNC region, after migrations, NOT in sync `create_http_server`).

**GOTCHA:** seed LOGIC is in the api crate (T4); only the INVOCATION is here (the seed-placement split is a deliberate log-DQ — §19). `[P]` with T5/T7 (disjoint), serial per advisor degrade.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 6`, then STOP.

### Task 7 [P]: Bridge ingest endpoint

**ACTION:** add `services/bridge/src/sanction_handler.rs` + `POST /brehon/sanction-event` route + `mod` decl. `requires: T4` (event schema). Bridge cargo runs IN-TASK (workspace-excluded).

```yaml
creates:
  - services/bridge/src/sanction_handler.rs
modifies:
  - services/bridge/src/appservice.rs   # wire POST /brehon/sanction-event in router()
  - services/bridge/src/main.rs         # mod sanction_handler;
requires:
  - task: 4
    reason: payload shape must match T4's SanctionEventPayload (re-declared LOCALLY, no api dep)
```

**IMPLEMENT:** per §10.7 — LOCAL `SanctionEventPayload` (serde; no api-crate dep), Bearer verify first line, `BridgeRoomStore` lookup, `sanction_kind` → power-level translation, `200 { applied, reason, applied_at }`.

**MIRROR:** `room_provisioner.rs` (axum+reqwest+secret); `appservice.rs::router()`; `bridge_room.rs::BridgeRoomStore`; `room_event_handler.rs` (bridge_auth-first shape).

**GOTCHA:** uses only `axum`/`reqwest`/`serde`/`serde_json`/`tracing`/`anyhow` — no new workspace deps, no matrix-sdk/ruma (WP-5/R9). Bridge stays in `exclude`.

**VALIDATE (in-task — bridge is workspace-excluded):**
```bash
cd services/bridge && cargo check > /tmp/m2-late-t7-bridge.log 2>&1; echo "exit: $?"
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'   # EXPECT: 0
```
Commit in-task (no validate-pending-laptop DQ for the bridge check; the zero-Matrix-deps gate is workspace-scoped and runs here too).

### Task 8: e2e test

**ACTION:** add `crates/server/tests/e2e/m2_late.rs` (quorum vote → `sanction_created` → mock subscriber captures the POST → assert payload shape) + `include!` wiring. `requires: T5, T6`.

```yaml
creates:
  - crates/server/tests/e2e/m2_late.rs
modifies:
  - crates/server/tests/e2e.rs   # include!("e2e/m2_late.rs")
requires:
  - task: 5
    reason: the spawn site (T5) is what fires the POST the test asserts on
  - task: 6
    reason: the startup seed (T6) registers the mock subscriber URL
```

**IMPLEMENT:** mock HTTP server (httpmock or equivalent) seeded as the subscriber; drive a quorum vote that writes a sanction; assert the captured POST body matches `SanctionEventPayload` + Bearer header present + subject is a pseudonym (not a username/email). Mirror a sibling fixtures module's LemmyResult error-shape case verbatim (`feedback_lemmy_error_no_std_error.md`); pre-locate verbatim `include!`/`old_string` anchors before editing `e2e.rs` (`feedback_fix_impl_pre_locate_e2e_anchors.md`); `AsyncPgConnection::establish` pattern (`feedback_async_pool_test_pattern.md`).

**MIRROR:** `crates/server/tests/e2e.rs:124-148` (mod common + include! wiring); sibling `e2e/governance.rs` / `e2e/sponsor_liability.rs`.

**GOTCHA:** e2e is a DIRECTORY; wire via `include!("e2e/m2_late.rs")` (the dominant pattern), NOT `mod`. Uniqueness: confirm the `include!` anchor in `e2e.rs` is unique before editing. Docker must be up (Probe 0).

**VALIDATE (Task-8 e2e gate — Windows bat wrapper, advisor-run):**
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > %LOCALAPPDATA%\\Temp\\m2-late-e2e.log 2>&1 && echo E2E_EXIT_0 >> %LOCALAPPDATA%\\Temp\\m2-late-e2e.log || echo E2E_EXIT_NONZERO >> %LOCALAPPDATA%\\Temp\\m2-late-e2e.log"
```
Worker writes `validate-pending-laptop-e2e` DQ `["<the bat line above>"]`, `phase_task: 8`, then STOPS (per Phase-2 e2e gate — advisor picks local-vs-dispatch). Never bare `cargo test`; never `-p lemmy_server --features full`.

### Task 9: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit.

---

## 14. Testing strategy

- **Unit (compile-time):** `./scripts/brehon/cargo-check.sh --workspace --features full` (every workspace task, via validate-pending-laptop).
- **Lint:** `./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings` (R2/R3).
- **Migration round-trip:** `cargo run -p lemmy_diesel_utils --features full -- migration run` (T1).
- **Bridge compile + zero-Matrix-deps:** `cd services/bridge && cargo check` + `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` ⇒ 0 (T7).
- **e2e execution:** `cargo test --test e2e` via the Windows bat wrapper, `--workspace --features full` (T8) — adds 1 new test; pre-existing tests still pass.

---

## 15. Validation commands (DoD)

### 15.1 Static analysis (per workspace task)

```bash
./scripts/brehon/cargo-check.sh --workspace --features full
# EXPECT: exit 0
```

### 15.2 Lint (per workspace task — R2/R3)

```bash
./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings
# EXPECT: exit 0
```

### 15.3 Migration runner (T1)

```bash
cargo run -p lemmy_diesel_utils --features full -- migration run
# EXPECT: exit 0  (per feedback_lemmy_migration_runner.md — NOT `diesel migration run`)
```

### 15.4 Bridge gate (T7)

```bash
cd services/bridge && cargo check                       # EXPECT: exit 0
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'   # EXPECT: 0
```

### 15.5 e2e (T8)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > %LOCALAPPDATA%\\Temp\\m2-late-e2e.log 2>&1 && echo E2E_EXIT_0 >> %LOCALAPPDATA%\\Temp\\m2-late-e2e.log || echo E2E_EXIT_NONZERO >> %LOCALAPPDATA%\\Temp\\m2-late-e2e.log"
# EXPECT: tail shows E2E_EXIT_0
```

### 15.6 Cross-cutting verification

- [ ] R5: migrations live at ROOT `migrations/`, NOT `crates/db_schema/migrations/`.
- [ ] R6: `SanctionEventId` in the single file `crates/db_schema/src/newtypes.rs`.
- [ ] WP-1: `map_sanction_action` is an exhaustive match (no `_ =>`); subject identifier is `actor_pseudonym.pseudonym` only (no `person.name`/`local_user.email`/`actor_id`).
- [ ] WP-2/R8: the spawn is OUTSIDE `conn.run_transaction()`; the in-spawn governance-log append uses a NEW pool connection.
- [ ] One spawn site only (`process_vote`); `process_appeal_vote` untouched.
- [ ] WP-5/R9: `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` ⇒ 0; `services/bridge` not in `members`.
- [ ] WP-7: registry section in `.claude/rules/governance-log-entry-kind-registry.md` is advisor-authored (count 65 → 67); Junior wrote only the 2 `crates/` files.
- [ ] `governance_log_entry_hash` is `hex::encode`-d into the `String` payload field.
- [ ] No `e2e.rs` edit in Tasks 1–7.

---

## 16. Acceptance criteria

- [ ] All 10 tasks completed in dependency order.
- [ ] §15.1 (cargo check) exit 0 after every workspace task.
- [ ] §15.2 (clippy `--no-deps -- -D warnings`) exit 0 after every workspace task.
- [ ] §15.3 (migration runner) exit 0 (T1).
- [ ] §15.4 (bridge check + zero-Matrix-deps ⇒ 0) (T7).
- [ ] §15.5 (e2e) — 1 new test passes; pre-existing tests still pass.
- [ ] §15.6 (cross-cutting) — all boxes ticked.
- [ ] §16a stories — all `[done]`.
- [ ] No edits to files outside §11.
- [ ] Retro committed per §13 Task 9.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Sanction event schema + vocabulary exist

- **Composing tasks:** Task 1, Task 2 (Task 2 `[P]`; Task 1 is the barrier it `requires:`)
- **Checkpoint command:** `./scripts/brehon/cargo-check.sh --workspace --features full`
- **Expected output:** exit 0
- **Brief-Scope outputs to verify:**
  - `crates/db_schema_file/src/enums.rs` contains `pub enum SanctionKind`
  - `crates/db_schema_file/src/schema.rs` contains `sanction_event` + `sanction_subscriber` tables + `sql_types::SanctionKind`
  - `crates/db_schema/src/source/governance/sanction_event.rs` + `sanction_subscriber.rs` exist + non-empty
  - `crates/db_schema/src/newtypes.rs` contains `SanctionEventId`

### Story 2: Publisher machinery compiles (additive, not yet wired)

- **Composing tasks:** Task 3, Task 4
- **Checkpoint command:** `./scripts/brehon/cargo-check.sh --workspace --features full`
- **Expected output:** exit 0
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/sanction_kind_map.rs` contains an exhaustive `match action {` with no `_ =>`
  - `crates/db_schema/src/source/governance/governance_log.rs` contains `ENTRY_KIND_SANCTION_PUBLISHED` + `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED`
  - `crates/api/api/src/governance/governance_log.rs` re-exports both consts
  - `crates/api/api/src/governance/sanction_publisher.rs` contains `enqueue_sanction_event` + `seed_sanction_subscriber` + `SanctionEventPayload`

### Story 3: Quorum vote publishes a schema-correct event to the bridge

- **Composing tasks:** Task 5, Task 6, Task 7, Task 8
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full ..."` (the §15.5 line)
- **Expected output:** `E2E_EXIT_0` (new m2_late test passes)
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/submit_jury_vote.rs` contains exactly one `tokio::spawn` of `enqueue_sanction_event` (in `process_vote`, after the transaction)
  - `crates/server/src/lib.rs` contains the `seed_sanction_subscriber` invocation guarded by `BRIDGE_SANCTION_CALLBACK_URL`
  - `services/bridge/src/sanction_handler.rs` exists; `services/bridge/src/appservice.rs` wires `POST /brehon/sanction-event`
  - `crates/server/tests/e2e/m2_late.rs` exists; `crates/server/tests/e2e.rs` contains `include!("e2e/m2_late.rs")`

> **Verification mapping:** `/brehon-verify` iterates these stories, runs each checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms trigger catch-fire.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (probes 0–9 confirmed).
- [ ] Tasks 1–8 committed (one commit each).
- [ ] §15 validation green at every gate.
- [ ] §16a stories all `[done]`.
- [ ] Retro committed (Task 9).
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete + findings triaged.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/m2-late-verify.md` shows all stories ✓.
- [ ] Registry section (65 → 67) authored by advisor (WP-7).

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Spawn accidentally inside the tx → unbounded vote latency | MED | HIGH | §10.3 re-query is AFTER `conn.run_transaction()`; §15.6 + Story-3 verify checkpoint asserts placement; WP-2/R8 |
| Non-exhaustive map (wildcard) hides a future `SanctionAction` | MED | MED | §10.2 mandates exhaustive match; clippy + compile fail on a new variant; §15.6 box |
| Pseudonymity leak (real name/email in payload) | LOW | HIGH | §10.3/§15.6 assert subject = `actor_pseudonym.pseudonym`; T8 e2e asserts payload subject is a pseudonym (ADR-015) |
| Editing the riskiest governance file (`submit_jury_vote.rs`) breaks the vote path | MED | HIGH | T5 is additive (post-commit re-query + guarded spawn), no return-site/tx-body change; re-query mirrors `:163-178` |
| Migration order (enum after table) → CREATE TABLE fails | LOW | MED | §10.4 single dir, enum strictly first; migration runner round-trip in T1 DoD |
| Bridge pulls a Matrix/workspace dep | LOW | MED | WP-5/R9 zero-Matrix-deps gate in T7 DoD + §15.4; LOCAL payload re-declaration (no api dep) |
| e2e worker-hang on `e2e.rs` edit | MED | MED | T8 isolated; pre-locate unique `include!` anchor; `feedback_fix_impl_pre_locate_e2e_anchors.md` |

---

## 19. Notes

**Split-DQ pre-seed (complexity 13 > 8 — MUST file at plan finalize).** `from: "planner"`, `kind: "blocker"`, pending. Question: "m2-late complexity 13 exceeds Sonnet threshold 8 (target sonnet-4-6) — split into m2-late-1 (T0–T4: additive machinery, publisher compiles but is never called, behaviour unchanged, sub-score ≈5) + m2-late-2 (T5–T9: wire live + e2e + retro, sub-score ≈6), or proceed-as-one?" Options: `["split", "proceed-as-one"]`. Context: the seam is clean — T0–T4 leave behaviour unchanged (no spawn site, no startup seed, no bridge route); T5–T9 wire it live and add the e2e. File via `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`; commit+push for advisor visibility.

**Seed-placement log-DQ.** `from: "planner"`, `kind: "log"`, `answered_by: "planner"`, resolved[]. WP-4 says the binary reads `BRIDGE_SANCTION_CALLBACK_URL` "at app startup … to seed the `sanction_subscriber` table", while brief §7's task table puts the seed in T3's `sanction_publisher.rs`. Resolution: seed **LOGIC** lives in `sanction_publisher::seed_sanction_subscriber` (api crate, T4 — additive, testable); seed **INVOCATION** lives in `lemmy_server` startup `crates/server/src/lib.rs` async region (T6, after migrations). This keeps the api crate free of a startup dependency and lets T4 stay pure-additive.

**Brief path correction #1 (migrations dir).** Brief §1/§7 + §6 say `crates/db_schema/migrations/<ts>_*.sql`. Actual: migrations live at the repository **ROOT** `migrations/<YYYY-MM-DD-HHMMSS-0000_name>/{up,down}.sql` (Diesel dated-dir convention; schema regenerated by `print-schema` via `lemmy_diesel_utils`). The plan uses the ROOT path (R5). Single combined dir `2026-06-07-000000-0000_add_sanction_event/`.

**Brief path correction #2 (newtypes location).** Brief §6/§7 imply a `crates/db_schema/src/newtypes/` directory. Actual: newtypes live in the single file `crates/db_schema/src/newtypes.rs` (per `feedback_newtype_locations_lemmy_db_schema_vs_file.md`). `SanctionEventId` is added there (R6).

**`MuteVoice` is intentionally unmapped by the v0 map.** It is a valid `SanctionKind` variant (OQ-ADR016-04 fixed the 4-variant enum) but no v0 `SanctionAction` produces it — none carries voice-only semantics. It exists for the bridge's translation table (`mute_voice` → PL below voice threshold) and for future `SanctionAction` additions. This is NOT a mapping bug; the exhaustive map is over `SanctionAction` (the input), and `MuteVoice` is a possible output not currently selected.

**Registry count.** `governance-log-entry-kind-registry.md` acceptance invariant stands at **65** consts (post m2-core-hook). m2-late adds 2 (`sanction_published`, `sanction_event_delivery_failed`) → **67**. The registry section is advisor meta-work (WP-7); Junior writes only the 2 `crates/` `governance_log.rs` files (const + shim).

**Combined-migration rationale.** WP-3 permits combining the enum + 2 tables into one migration dir; chosen to keep the `CREATE TYPE` strictly before the `CREATE TABLE` that references it within a single atomic migration (avoids a 3-file ordering footgun and a partial-apply state where the type exists but the table doesn't).

**`process_vote` 5 return sites (re-query rationale).** `process_vote` returns at {244, 308, 336, 434, 860}; threading the sanction tuple through all five touches the control flow of the most complex governance file. The post-commit re-query (§10.3) reads the just-written row once, filtered by `case_id`, returning `None` for appeal-only / NoAction / no-sanction cases so the spawn skips naturally. LOCKED.

---

## 20. Confidence score

- **Plan correctness:** 8/10 — anchors verified live (enums.rs:531-577, schema.rs:132-138, submit_jury_vote.rs:452-480 + :163-178, sanction.rs:1-48, governance_log.rs:117, e2e.rs:124-148, lib.rs:352). Residual risk: exact `SanctionContext` shape (pool + env handle) is impl-discretion within §10.3.
- **Cargo budget:** 8/10 — ~6 GB peak; pre-Shape-G laptop validation; standard forbidden-window applies.
- **Test coverage:** 7/10 — one e2e covers the golden path (quorum → POST captured → payload asserted); failure-path (`sanction_event_delivery_failed`) is unit-shaped, not e2e-covered in v0.
