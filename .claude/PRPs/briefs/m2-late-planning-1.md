# Planning Brief — m2-late: B-publish Sanction Propagation

**Phase:** m2-late
**Branch:** phase-m2-late (cut from governance-v0 at bm-cut, after plan approval)
**Authored:** 2026-06-07
**Authored by:** advisor (canonical brehon-fork / governance-v0 session)
**PRD:** `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` §"Implementation Phases" (Phase 6)
**Plan target:** `.claude/PRPs/plans/m2-late.plan.md`

---

## 1. What this phase delivers (M2-late Phase 6 only)

m2-late Phase 6 adds **B-publish sanction propagation**. When a Brehon governance decision
reaches quorum (the `sanction_created` log entry fires in `submit_jury_vote.rs`), the binary
delivers a structured sanction event via HTTP webhook to registered subscribers. The only
subscriber in m2-late scope is the bridge (Matrix).

**OQ gate cleared (2026-06-07):**
- OQ-ADR016-02 resolved: webhook transport; universal schema `{ sanction_kind, subject_brehon_actor_id,
  effective_from, effective_until, governance_log_entry_hash }`; at-least-once; bridge uses
  `BRIDGE_CALLBACK_SECRET` bearer auth.
- OQ-ADR016-04 resolved: fixed `SanctionKind` enum (`prevent_post`, `mute_voice`, `hide_content`,
  `restrict_reach`); Matrix power-level translation defined; partial-applicability via
  "not_applicable" ack.
- **Phase 7 (B-actor) is OUT OF SCOPE** — user-confirmed 2026-06-07.

**Deliverables:**

**Phase 6.1 — Sanction event schema + DB table:**
- `SanctionKind` enum in `crates/db_schema_file/src/enums.rs` (four variants:
  `PreventPost`, `MuteVoice`, `HideContent`, `RestrictReach`). Derives `DbEnum` under
  `#[cfg(feature = "full")]`.
- `sanction_event` table migration (new `crates/db_schema/migrations/<ts>_sanction_event.sql`):
  `id` SERIAL PK, `sanction_id` FK→`sanction`, `sanction_kind` (PG enum), `subject_actor_pseudonym`
  TEXT (from `actor_pseudonym.pseudonym` — ADR-015: never real username/email), `effective_from`
  TIMESTAMPTZ NOT NULL, `effective_until` TIMESTAMPTZ NULL, `governance_log_entry_hash` TEXT NOT NULL.
- Diesel model + insert form at `crates/db_schema/src/source/governance/sanction_event.rs`.

**Phase 6.2 — Subscriber registry:**
- `sanction_subscriber` table migration (new migration): `id` SERIAL PK, `callback_url` TEXT
  NOT NULL UNIQUE, `active` BOOLEAN NOT NULL DEFAULT TRUE, `created_at` TIMESTAMPTZ NOT NULL.
  For m2-late, exactly one row is pre-seeded: the bridge's callback URL (from a new env var
  `BRIDGE_SANCTION_CALLBACK_URL`). No HTTP subscriber-registration endpoint in m2-late scope
  (manual via migration seed).
- Diesel model at `crates/db_schema/src/source/governance/sanction_subscriber.rs`.

**Phase 6.3 — Emit hook in submit_jury_vote.rs:**
- After `sanction_created` log entry is appended (line ~472 in
  `crates/api/api/src/governance/submit_jury_vote.rs`), call a new
  `enqueue_sanction_event(sanction, conn)` function (fire-and-forget, `tokio::spawn`).
- `enqueue_sanction_event`: reads all `active` subscribers from `sanction_subscriber`, derives
  `SanctionKind` from `SanctionAction` (mapping in a new `sanction_kind_map.rs`), builds the
  event payload, and delivers to each subscriber URL via `reqwest::Client::post`.
- Auth: `Authorization: Bearer <BRIDGE_CALLBACK_SECRET>` header — same env var used for
  `bridge_auth::verify_bridge_secret` (existing, `521e949e3` pattern).
- At-least-once delivery: a failed POST is logged (`tracing::warn`) and a `sanction_event_delivery_failed`
  governance_log entry is appended (best-effort, outside the main transaction). No retry queue
  in m2-late scope (retry = future work).
- `BRIDGE_SANCTION_CALLBACK_URL` env var read at app startup; if absent, subscriber table is
  not pre-seeded (no-op — clean posture when not configured).

**Phase 6.4 — Bridge subscriber endpoint:**
- New `POST /brehon/sanction-event` route in `services/bridge/src/appservice.rs` (axum).
- Handler in `services/bridge/src/sanction_handler.rs` (new): verifies `Authorization: Bearer
  <BRIDGE_CALLBACK_SECRET>`, deserialises `SanctionEventPayload`, translates `sanction_kind`
  to Matrix power-level changes, and sends the power-level-change request via the Matrix
  client API for each provisioned room for the `subject_actor_pseudonym`.
- ACK: responds HTTP 200 `{ "applied": true|false, "reason": String, "applied_at": ISO-8601 }`.
  The binary logs the ACK (best-effort; no DB write in m2-late scope — ACK logging is M3 scope).
- `SanctionKind` ↔ Matrix power-level translation (from OQ-ADR016-04 resolution):
  - `prevent_post` → set user PL to < `events_default` in all provisioned rooms
  - `mute_voice` → set user PL to < voice event threshold (or leave unchanged if no voice rooms)
  - `hide_content` → power-level drop (same as prevent_post; content redaction is not a Matrix
    primitive the bridge controls on others' content)
  - `restrict_reach` → same as prevent_post (Matrix has no native reach-restriction primitive)
- Bridge queries provisioned rooms via `bridge_room` SQLite store (shipped in m2-rooms-a) to
  find all `case_id` → `matrix_room_id` rows for the subject.

**Phase 6.5 — Governance log entry kind:**
- New `ENTRY_KIND_SANCTION_PUBLISHED = "sanction_published"` in
  `crates/db_schema/src/source/governance/governance_log.rs`.
- Re-export in api shim at `crates/api/api/src/governance/governance_log.rs`.
- Registry section in `.claude/rules/governance-log-entry-kind-registry.md` (advisor-side,
  not Junior authored).
- Emitted by `enqueue_sanction_event` when at least one subscriber POST returns 200.
  On best-effort failure-path it emits `sanction_event_delivery_failed` (separate const,
  same files).

**Deliverable boundary:** `crates/db_schema/migrations/` (2 new migrations) + `crates/db_schema/src/source/governance/` (2 new model files) + `crates/db_schema_file/src/enums.rs` (new enum) + `crates/api/api/src/governance/` (2 new handlers, 1 new module) + `crates/api/routes/src/lib.rs` (no new route; subscriber delivery is outbound-only) + `services/bridge/src/` (1 new handler, 1 new route in appservice.rs). **No changes to `crates/server/tests/e2e.rs` for m2-late Tasks 1–5** (the emit hook fires after the existing `sanction_created` log entry; e2e tests for m2-late are task 6, scope-gated).

---

## 2. Key file anchors (planner must verify before authoring tasks)

| File | Anchor | Purpose |
|---|---|---|
| `crates/api/api/src/governance/submit_jury_vote.rs:453-472` | `if let Some((scope, action)) = map_decision_to_sanction(winning_decision)` block | Emission point — `enqueue_sanction_event` call goes here, after the `"sanction_created"` log append at ~472 |
| `crates/db_schema_file/src/enums.rs:560-577` | `SanctionAction` enum | Source for the `SanctionAction → SanctionKind` mapping. Seven variants: Label, VisibilityReduction, TemporaryRestriction, ContentRemoval, CommunityExclusion, InstanceSuspension, FederationQuarantineRecommendation, Restoration |
| `crates/api/api/src/governance/bridge_auth.rs:6-18` | `verify_bridge_secret` fn | Existing BRIDGE_CALLBACK_SECRET bearer-auth pattern — mirror for new bridge endpoint |
| `crates/api/api/src/governance/room_event_handler.rs` | `handle_room_event` fn | T4a pattern: bridge_auth first line, then body |
| `crates/api/routes/src/lib.rs:481-484` | `scope("/governance")` block | Pattern for new governance route registration; new bridge endpoint goes at POST `/brehon/sanction-event` on the BRIDGE side (axum), NOT here |
| `services/bridge/src/appservice.rs` | `router()` fn + `AppState` | Where the new `/brehon/sanction-event` axum route is wired |
| `services/bridge/src/room_provisioner.rs` | Bridge-side provisioner pattern | Template for the sanction_handler (reqwest, axum, BRIDGE_CALLBACK_SECRET) |
| `services/bridge/src/bridge_room.rs` | `BridgeRoomStore` | SQLite lookup for `case_id` → `matrix_room_id` (used by sanction_handler to find rooms for subject) |
| `crates/db_schema/src/source/governance/governance_log.rs:117` | `ENTRY_KIND_SANCTION_CREATED` const | Sibling pattern for new `ENTRY_KIND_SANCTION_PUBLISHED` const |
| `crates/db_schema/src/source/governance/sanction.rs:20-40` | `Sanction` + `SanctionInsertForm` structs | Source pattern for `SanctionEvent` + `SanctionEventInsertForm`. Key fields: `target_person_id: Option<PersonId>` (nullable), `starts_at` → event `effective_from`, `ends_at` → event `effective_until` |
| `crates/db_schema_file/src/schema.rs:133-138` | `sql_types::SanctionAction` + `sql_types::SanctionScope` struct declarations | Pattern for new `pub struct SanctionKind` in `sql_types` — required for Diesel DbEnum plumbing alongside the PG migration `CREATE TYPE sanction_kind` |
| `crates/api/api/src/governance/submit_jury_vote.rs:1059-1061` | Comment "No sanction row, no federation outbound" | `process_appeal_vote` does NOT write a sanction row — `sanction_created` fires ONLY in `process_vote` (line 472). `enqueue_sanction_event` has exactly ONE spawn site. |
| `.claude/PRPs/prds/m2-governance-triggered-rooms.prd.md` | Phase 6 spec section | Complete scope reference |

**Workspace migration note:** Two new migrations in `crates/db_schema/migrations/`. Both must pass
`cargo run -p lemmy_diesel_utils --features full -- migration run` (per `feedback_lemmy_migration_runner.md`).
Planner must assign migration creation to a task early (T1 or T2) to allow the Diesel schema to be
regenerated before downstream tasks touch the new tables.

---

## 3. Watchpoints for the planner

**WP-1 (SanctionAction → SanctionKind mapping — ADR-015 and partial-applicability):**
Not all seven `SanctionAction` variants map to the four `SanctionKind` variants from OQ-ADR016-04.
The planner must include a `sanction_kind_map.rs` module that:
- Maps `Label` → `restrict_reach` (softest intent)
- Maps `VisibilityReduction` → `restrict_reach`
- Maps `TemporaryRestriction` → `prevent_post`
- Maps `ContentRemoval` → `hide_content`
- Maps `CommunityExclusion` → `prevent_post`
- Maps `InstanceSuspension` → `prevent_post`
- Maps `FederationQuarantineRecommendation` → none (no local Matrix primitive; skip delivery)
- Maps `Restoration` → none (reserved variant; skip delivery)
This mapping is the canonical v0 mapping; it must be an exhaustive match in Rust (not a
`_ => ...` wildcard) so future `SanctionAction` additions force an explicit mapping decision.
**ADR-015 note:** `subject_brehon_actor_id` in the event payload must be `actor_pseudonym.pseudonym`
(the opaque pseudonym), NOT `person.name`, NOT `local_user.email`. The emit hook reads
`actor_pseudonym` from the `sanction` row's `person_id` FK.

**WP-2 (Fire-and-forget delivery + no main-tx pollution):**
The `enqueue_sanction_event` call in `submit_jury_vote.rs` must be a `tokio::spawn(async move {...})`
**outside** the `conn.run_transaction()` block. Rationale: webhook delivery is a network I/O;
blocking the DB transaction on an HTTP POST introduces unbounded latency. Planner: verify the
`sanction_created` log append completes inside the transaction; the spawn happens after the
transaction commits (or at the `conn.run_transaction()` return site, post-commit). The governance
log entry `sanction_published` (or `sanction_event_delivery_failed`) is appended separately
inside the spawned task using a NEW pool connection — not the transaction's `conn`.

**One spawn site only:** `process_appeal_vote` does NOT write a sanction row
(`submit_jury_vote.rs:1059-1061` — "No sanction row, no federation outbound"). `enqueue_sanction_event`
goes only in `process_vote` (standard jury quorum path, after line 472).

**WP-3 (Migration order — `SanctionKind` PG enum must precede `sanction_event` table):**
PG enums in Lemmy use the `diesel-derive-pg` path; the enum must be defined in a migration
BEFORE the table that references it. The planner must order the migrations:
1. `<ts1>_sanction_kind_enum.sql` — `CREATE TYPE sanction_kind AS ENUM (...)`
2. `<ts2>_sanction_event.sql` — `CREATE TABLE sanction_event (... sanction_kind sanction_kind ...)`
3. `<ts3>_sanction_subscriber.sql` — `CREATE TABLE sanction_subscriber (...)`
Alternatively, combine into one migration file if the planner prefers. Either way, Diesel
schema regeneration MUST run before writing Rust model files that reference these tables.
Use `ENUM_KIND_PATTERN.md` if it exists; otherwise follow the `SanctionScope` pattern in
`crates/db_schema_file/src/enums.rs:542-547` + the existing PG enum migration in Phase 4.

**WP-4 (BRIDGE_SANCTION_CALLBACK_URL vs BRIDGE_CALLBACK_SECRET env var discipline):**
Two env vars are in play:
- `BRIDGE_CALLBACK_SECRET` — the shared secret for **both** binary→bridge auth AND bridge→binary
  auth. Already in `.env`. The bridge's new `POST /brehon/sanction-event` verifies this bearer
  token on ingress; the binary sends it as `Authorization: Bearer` on outbound POSTs.
- `BRIDGE_SANCTION_CALLBACK_URL` — the bridge's ingest URL for sanction events (e.g.
  `http://localhost:9009/brehon/sanction-event`). New env var; add to `.env` with a default.
The planner must verify `.env` has both and the binary reads `BRIDGE_SANCTION_CALLBACK_URL`
at startup (not per-request) to seed the `sanction_subscriber` table row on first run or on
each restart (idempotent `INSERT ... ON CONFLICT DO NOTHING`).

**WP-5 (Zero-Matrix-deps-in-workspace gate — still applies):**
All bridge-side code lives in `services/bridge/` (workspace-excluded). The new `sanction_handler.rs`
uses only `axum`, `reqwest`, `serde_json`, `tracing`, and `anyhow` — no new workspace deps.
The planner must include the zero-Matrix-deps gate in every bridge-side task DoD:
```bash
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'
```
Expected: `0`.

**WP-6 (e2e test scope — gate to Task 6, not earlier):**
`crates/server/tests/e2e.rs` is touched only by Task 6 (a new e2e test for
`sanction_created → sanction_published` chain). Tasks 1–5 must NOT touch `e2e.rs`. The e2e
test validates end-to-end: submit a vote that reaches quorum → `sanction_created` fires →
`enqueue_sanction_event` runs → a mock HTTP server (httpmock or similar) captures the POST →
assert event payload matches schema. This is a workspace-level e2e test; all Lemmy e2e
lessons apply (`feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`,
`feedback_fix_impl_pre_locate_e2e_anchors.md`).

**WP-7 (Governance-log registry update — advisor-side, not Junior):**
The two new entry kinds (`sanction_published`, `sanction_event_delivery_failed`) require
a new section in `.claude/rules/governance-log-entry-kind-registry.md`. This file is
advisor-owned meta-work (`.claude/rules/**`). The planner's §13 task that introduces the
consts should include a NOTE to the advisor to update the registry. Junior writes only the
`crates/db_schema/` files (consts + shim re-exports); the registry section is advisor-authored
at plan review time (same pre-landed-const exemption as prior phases).

---

## 4. Scope constraints (stop-and-ask tripwires for the planner)

- **STOP if:** any task adds a subscriber-registration HTTP endpoint (user-facing `POST /governance/sanction-subscriber`). Subscriber management is out of m2-late scope — the bridge is the only subscriber, seeded via env var + idempotent INSERT.
- **STOP if:** any task implements a retry queue (e.g. a `sanction_event_retry` table, a background scheduler for failed deliveries). At-least-once via fire-and-forget is the v0 contract; retry infrastructure is M3/future scope.
- **STOP if:** any task adds the Phase 7 (B-actor) link-flow: `actor_app_link` table, OAuth redirect, dual-signed claim mechanism. Phase 7 is OUT OF SCOPE — user-confirmed 2026-06-07.
- **STOP if:** any task makes `enqueue_sanction_event` block the `conn.run_transaction()` DB transaction. The spawn must be outside the transaction boundary.
- **STOP if:** `enqueue_sanction_event` is also wired inside `process_appeal_vote`. That function writes no sanction row (see `submit_jury_vote.rs:1059-1061` comment "No sanction row, no federation outbound") — wiring there is a scope error. Exactly one spawn site: after line 472 in `process_vote`.
- **Non-person-targeted sanctions:** `Sanction.target_person_id` is nullable. When `target_person_id` is `None` (post/comment/community-targeted sanctions), `enqueue_sanction_event` must skip delivery silently (no subject pseudonym to resolve via `actor_pseudonym_helper`). This is not an error — log at `tracing::debug` and return early.
- **STOP if:** any task exposes real `person.name`, `local_user.email`, or `local_user.actor_id` in the sanction event payload. The subject identifier MUST be `actor_pseudonym.pseudonym` (ADR-015).
- **STOP if:** any task adds `services/bridge` to the workspace `members` array in root `Cargo.toml`. Bridge stays in `exclude`.
- **Do NOT apply** Lemmy workspace lessons to bridge-side tasks for Tasks 1–5 scope; apply only for workspace-touching tasks (T1 migration, T3 enum, T6 e2e). See brief §5 lesson injections for the full per-task breakdown.

---

## 5. DoD gates

Per `feedback_plan_dod_dry_run_at_write.md` — these must be executable as written:

**Workspace gate (migration + enum tasks):**
```bash
./scripts/brehon/cargo-check.sh --workspace --features full
```
Expected: exit 0.

**Migration gate (all migration tasks):**
```bash
cargo run -p lemmy_diesel_utils --features full -- migration run
```
Expected: exit 0. Per `feedback_lemmy_migration_runner.md`.

**Bridge gate (all bridge tasks):**
```bash
cd services/bridge && cargo check
```
Expected: exit 0.

**Zero-Matrix-deps gate (all bridge tasks):**
```bash
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'
```
Expected: `0`.

**Task 6 e2e gate:**
```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > %LOCALAPPDATA%\\Temp\\m2-late-e2e.log 2>&1 && echo E2E_EXIT_0 >> %LOCALAPPDATA%\\Temp\\m2-late-e2e.log || echo E2E_EXIT_NONZERO >> %LOCALAPPDATA%\\Temp\\m2-late-e2e.log"
```
Never bare `cargo test`; never `-p lemmy_server --features full`. Per `feedback_windows_e2e_requires_bat_wrapper.md`.

**validate-pending-laptop DQ shape (bridge tasks):**
```json
{ "kind": "validate-pending-laptop", "commands": ["cd services/bridge && cargo check"], "branch": "<phase-branch>", "phase_task": N }
```
For workspace-touching tasks: add `"./scripts/brehon/cargo-check.sh --workspace --features full"` as a second command. For migration tasks: add `"cargo run -p lemmy_diesel_utils --features full -- migration run"` as a second or third command.

---

## 6. Lesson injections (mandatory, per advisor-orchestrator §2.4)

**Mandatory for ALL tasks:**
- `feedback_validate_pending_laptop_write_then_stop.md` — workers write DQ validate-pending-laptop and STOP; do NOT run cargo on the daemon.

**Mandatory for Tasks 1–2 (migration + Diesel model):**
- `feedback_lemmy_migration_runner.md` — migration runner command (NOT `diesel migration run`).
- `feedback_postgres_jsonb_canonicalization.md` — if any JSONB columns are added (not currently planned; but sanction_event payload column if added later).
- `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — `SanctionEventId` newtype in `crates/db_schema/src/newtypes/`.

**Mandatory for Task 3 (SanctionKind enum in db_schema_file):**
- `feedback_features_full_workspace_only.md` — `--features full` scope.
- `feedback_clippy_test_style.md` — deny unwrap/expect in new code.

**Mandatory for Task 4 (emit hook in submit_jury_vote.rs):**
- `feedback_multi_write_handlers_need_transactions.md` — fire-and-forget spawn is OUTSIDE the transaction; confirm this pattern is respected.
- `feedback_governance_type_state_handlers.md` — the emit hook modifies no `ModerationCase` state; no `GovernanceCase<S>` wrapper needed. But confirm `actor_pseudonym` lookup follows ADR-015 path.

**Mandatory for Task 6 (e2e test):**
- `feedback_lemmy_error_no_std_error.md` — LemmyResult case A/B/C enum; `.map_err` annotated closure.
- `feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` pattern for e2e.
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim `old_string` anchors before editing `e2e.rs`.

**Do NOT inject** into bridge tasks (T5, T7): `feedback_lemmy_error_no_std_error.md`, `feedback_lemmy_migration_runner.md`, `feedback_postgres_jsonb_canonicalization.md`, `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — these are Lemmy-workspace-specific.

---

## 7. Plan structure guidance

Suggested §13 task breakdown:

| Task # | Deliverable | Primary files | `[P]`? |
|---|---|---|---|
| T0 | Audit — verify bridge has `rusqlite` + `bridge_room.rs`; confirm no `/brehon/sanction-event` route; confirm no `sanction_event` table; run zero-Matrix-deps gate; confirm `SanctionAction` enum variants (7) and produce the mapping table for T3 | N/A | No |
| T1 | Migrations: `CREATE TYPE sanction_kind ...` + `CREATE TABLE sanction_event ...` + `CREATE TABLE sanction_subscriber ...`; Diesel model files; newtype `SanctionEventId` | `crates/db_schema/migrations/<ts>_*.sql` (2 or 3 files), `crates/db_schema/src/source/governance/sanction_event.rs`, `crates/db_schema/src/source/governance/sanction_subscriber.rs`, `crates/db_schema/src/newtypes/` | No — blocks T2 |
| T2 | `SanctionKind` enum in `crates/db_schema_file/src/enums.rs`; Diesel `DbEnum` plumbing; `sanction_kind_map.rs` (exhaustive `SanctionAction → Option<SanctionKind>` match) | `crates/db_schema_file/src/enums.rs`, `crates/api/api/src/governance/sanction_kind_map.rs` (new) | No — depends T1 |
| T3 | `ENTRY_KIND_SANCTION_PUBLISHED` + `ENTRY_KIND_SANCTION_EVENT_DELIVERY_FAILED` consts + api shim re-exports; `enqueue_sanction_event` fn in new `sanction_publisher.rs`; idempotent `BRIDGE_SANCTION_CALLBACK_URL` subscriber seed | `crates/db_schema/src/source/governance/governance_log.rs`, `crates/api/api/src/governance/governance_log.rs` (shim), `crates/api/api/src/governance/sanction_publisher.rs` (new) | No — depends T2 |
| T4 | Emit hook in `submit_jury_vote.rs`: `tokio::spawn(enqueue_sanction_event(...))` after `sanction_created` log entry; `actor_pseudonym` lookup for subject | `crates/api/api/src/governance/submit_jury_vote.rs` | No — depends T3 |
| T5 | Bridge: new `services/bridge/src/sanction_handler.rs` + `POST /brehon/sanction-event` route in `appservice.rs`; `BRIDGE_CALLBACK_SECRET` verify; sanction kind → Matrix power-level translation; ACK response | `services/bridge/src/sanction_handler.rs` (new), `services/bridge/src/appservice.rs` | No — depends T4 (needs the event schema) |
| T6 | e2e test: quorum vote → `sanction_created` fires → mock subscriber captures `POST /brehon/sanction-event` → assert payload shape; `#[ignore]` or direct e2e harness per WP-6 | `crates/server/tests/e2e/governance.rs` (or new `crates/server/tests/e2e/m2_late.rs`) | No — depends T4 |

**T4 is the riskiest task** (touches the live `submit_jury_vote.rs` quorum path, which is the most complex file in the governance module). Planner may split T4 into T4a (`actor_pseudonym` lookup + `SanctionEventInsertForm` build) and T4b (`tokio::spawn` hook + post-transaction wiring) if §5 complexity scoring warrants.

**T1 and T2 are the highest cargo complexity** (migrations + new PG enum). Planner must schedule T0's `cargo run -p lemmy_diesel_utils ...` DoD validation before T2's Diesel model files are written.

**Pre-Shape-G; all workspace cargo runs via validate-pending-laptop DQ on the laptop. Bridge cargo runs `cd services/bridge && cargo check`.**

---

## 8. Not in scope for m2-late

- **Phase 7 (B-actor portable-ID linkage)** — OUT OF SCOPE, user-confirmed 2026-06-07. OQ-ADR016-03 deferred.
- **Subscriber-registration HTTP endpoint** — no user-facing `POST /governance/sanction-subscriber`.
- **Retry queue / scheduler for failed deliveries** — best-effort fire-and-forget is the v0 contract.
- **ACK storage** — bridge POSTs ack back, but Brehon does not store per-event per-app status in m2-late (governance_log entry for success/failure is the v0 audit trail; per-app status table is M3).
- **Non-Matrix subscribers** — only the bridge is registered. No generic subscriber management.
- **Any e2e.rs edit in Tasks 1–5** — all `e2e.rs` touches are isolated to Task 6.
- **ADR-016 OQ-ADR016-02(d) link-claim auth** — deferred to first non-Matrix integration ADR.
- **`ENTRY_KIND_SANCTION_PUBLISHED` registry section** — advisor-authored at plan review time (not Junior, same pre-landed-const exemption as prior phases).

---

## 9. Pre-queue checklist (advisor to run before dispatching planning Junior)

- [x] OQ-ADR016-02 + OQ-ADR016-04 resolved in `99-decisions-and-open-questions.md` (commit `5d05f18a0`, 2026-06-07).
- [ ] `/brehon-clarify` run on this brief — no clarify-DQ entries expected (OQs resolved, watchpoints documented); skip only if clarify confirms 0 open entries.
- [ ] Verify `services/bridge/src/bridge_room.rs` exists (m2-rooms-a shipped it — confirms SQLite room lookup is available for T5). **Confirmed: exists** (checked 2026-06-07 `ls services/bridge/src/*.rs`).
- [ ] Verify no `sanction_event` table in DB schema (`crates/db_schema/src/schema.rs` — T1 adds it).
- [ ] Verify `services/bridge/Cargo.toml` has `rusqlite` (m2-rooms-a T1 added it — confirmed present 2026-06-07).
- [ ] `mcp__junior-brehon__list_hooks` — confirm Telegram completion hook is active.
- [ ] `/precheck` — mandatory before Junior dispatch.
- [ ] This brief committed to `governance-v0` and visible at that ref before `create_task`.

---

_Authored by advisor. Read-only reference for planning Junior. Do not modify during the planning run._
