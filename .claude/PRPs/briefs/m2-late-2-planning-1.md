# Brief: m2-late-2 planning — bridge sanction enforcement + atomic publish audit

**Created:** 2026-06-10  
**Status:** planning brief, not an implementation plan  
**Target plan:** `.claude/PRPs/plans/m2-late-2.plan.md`  
**Predecessors:** `.claude/PRPs/reports/m2-late-1-retro.md`, `.claude/PRPs/handovers/m2-late-2-bootstrap.md`, `.claude/PRPs/plans/m2-late.plan.md`  
**Mode:** planning/docs only; no Rust or migration edits in this brief.

---

## 1. Goal

Plan `m2-late-2`, the follow-up to m2-late-1 B-publish sanction propagation. It closes the two deliberate m2-late-1 carry-forwards and adds one operational verification gate:

1. **CR-A atomicity fix:** `enqueue_sanction_event` currently inserts `sanction_event` and appends `sanction_published` / `sanction_event_delivery_failed` to `governance_log` on separate pool connections. Plan must make those two writes atomic in a fresh transaction inside `enqueue_sanction_event` without touching the vote transaction.
2. **Bridge power-level enforcement:** `services/bridge/src/sanction_handler.rs` currently authenticates and ACKs sanction events but returns `applied: false`; plan must make the Matrix bridge apply a power-level change to provisioned rooms.
3. **Pilot verification:** confirm the pilot deployment seeds an active `sanction_subscriber` from `BRIDGE_SANCTION_CALLBACK_URL` and can reach the bridge callback.

**Out of scope:** B-actor portable-ID linkage / OAuth link flow / `actor_app_link` table. User-confirmed out of scope 2026-06-07; OQ-ADR016-03 remains deferred.

---

## 2. Mandatory context already checked

- ADR-016: Brehon is the cross-app governance backplane; apps retain sovereignty; B-publish is webhook publish/subscribe.
- OQ-ADR016-02: resolved 2026-06-07. Bridge subscriber uses `BRIDGE_CALLBACK_SECRET`; future link-claim auth is deferred.
- OQ-ADR016-04: resolved 2026-06-07. Matrix-specific reference translation exists, but bridge implementation currently has only a stub.
- m2-late-1 retro carry-forwards:
  - CR-A atomicity gap.
  - Power-level enforcement stub.
  - Pilot subscriber-row verification.
- Code exploration 2026-06-10:
  - `crates/api/api/src/governance/sanction_publisher.rs` already has `enqueue_sanction_event`, payload construction, subscriber POST, event insert, and governance-log append.
  - `governance_log::append` takes `&mut DbPool<'_>`, but existing call sites use `&mut (&mut *conn).into()` to call it within a transaction/savepoint. CR-A is fixable WITHOUT a signature change — verified: `append` (db_schema/.../governance_log.rs:278) runs its own inner `run_transaction` (line 309) and its doc comment (lines 294-302) confirms diesel-async promotes that inner tx to a SAVEPOINT when called via the reborrow inside a caller's outer tx. **Canonical live exemplar: `admin_assign_jury.rs:218`** (`append(&mut (&mut *conn).into(), ...)`). NOTE: the bootstrap's `federation_outbox.rs:187` cite is stale — use `admin_assign_jury.rs:218`. Resolved clarify-DQ `a3d0e9941441-061`.
  - `services/bridge/src/sanction_handler.rs` receives the event and maps static sanction-kind values but does not apply Matrix state.
  - `services/bridge/src/bridge_room.rs` indexes rooms by `(case_id, room_type)`, not by pseudonym.
  - `SanctionEventPayload` currently carries `sanction_kind`, `subject_actor_pseudonym`, `effective_from`, `effective_until`, and `governance_log_entry_hash`; it does **not** carry `case_id`.

---

## 3. Planning decisions to make before final plan

### D1 — How should the bridge find affected rooms?

**Recommended:** add `case_id` to `SanctionEventPayload` as a Matrix-bridge extension field, then have the bridge look up all `bridge_room` rows for that case.

Rationale:
- B-actor is out of scope, so a durable pseudonym→Matrix-account/room index is not available.
- `bridge_room` already has `(case_id, room_type, matrix_room_id)` and is the m2-rooms-a durable state.
- The current payload's `governance_log_entry_hash` is not enough for the bridge to resolve `case_id` unless it calls back into Brehon or gains DB access, both larger than this follow-up.
- Adding `case_id` is a backward-compatible payload extension for the in-house bridge subscriber. It does not contradict OQ-ADR016-02's universal fields; those remain present.

**Alternative:** bridge calls a Brehon lookup endpoint by `governance_log_entry_hash` to retrieve `case_id`. Rejected for m2-late-2 unless user wants a new HTTP route/auth surface.

### D2 — What Matrix enforcement level is acceptable for m2-late-2?

**Recommended:** implement case-room power-level enforcement only, not historical-message redaction.

Minimum shape:
- Resolve subject puppet MXID using existing `state.puppet_map.ensure_puppet(subject_actor_pseudonym)`.
- For each `bridge_room` row for `case_id`, fetch current `m.room.power_levels` state and write an updated `users[subject_mxid]` override.
- `prevent_post`, `mute`, `ban`: set below the room's message threshold / `events_default`.
- `mute_voice`: set below the MatrixRTC voice event threshold when present, otherwise below `events_default` as a conservative fallback.
- `restrict_reach`: Matrix has no native reach primitive; reduce to the same low posting level and return a reason saying it was translated to room power-level reduction.
- `hide_content`: no safe historical redaction without a message index; apply the same low posting level and return reason `redaction_not_available_in_m2_late_2`.

If full `hide_content` redaction is required now, that should be a separate phase because the bridge needs a subject→event index or room-history scan.

### D3 — Should CR-A introduce a new append helper?

**Recommended:** no new public helper unless the implementation proves `&mut (&mut *conn).into()` does not type-check in `sanction_publisher.rs`.

Expected implementation approach:
- Keep vote transaction untouched (R8).
- Inside `enqueue_sanction_event`, after delivery attempts determine `kind`, open one fresh connection.
- Run `conn.run_transaction(async |conn| { insert sanction_event; governance_log::append(&mut (&mut *conn).into(), kind, payload, Some(subject.clone())).await; Ok(()) })`.
- If this does not compile, stop and re-plan a minimal conn-accepting append variant; do not silently widen `governance_log::append`.

---

## 4. Proposed m2-late-2 task breakdown

### T0 — Pre-phase audit and baseline proof

Read required files, confirm branch/worktree, and run scoped no-edit probes:
- `rg "applied: false|sanction_kind_to_power_level|bridge_room" services/bridge/src`
- `rg "enqueue_sanction_event|SanctionEventPayload|sanction_event" crates/api/api/src/governance crates/db_schema/src/source/governance`
- cargo gates to be run through wrappers/logs per project policy:
  - `scripts/brehon/cargo-check.sh -p lemmy_api --features full` is **invalid**; use workspace for `--features full` only.
  - `scripts/brehon/cargo-check.sh --workspace --features full` for workspace proof.
  - `cd services/bridge && cargo check` for bridge proof.

### T1 — Payload + event-row case_id extension

Files likely affected:
- `crates/api/api/src/governance/sanction_publisher.rs`
- `services/bridge/src/sanction_handler.rs`
- possibly `crates/db_schema/src/source/governance/sanction_event.rs` and migration **only if** persistence of `case_id` in `sanction_event` is required.

Recommendation: include `case_id` in the webhook payload first. Persisting it in `sanction_event` is optional; prefer no migration unless e2e or audit requirements need queryability.

Validation:
- Existing `sanction_event_delivered_to_subscriber` e2e updated to assert `case_id` in JSON payload.
- Bridge handler unit/integration test accepts payload with `case_id`.

### T2 — CR-A atomicity fix in `enqueue_sanction_event`

Files likely affected:
- `crates/api/api/src/governance/sanction_publisher.rs`
- test file only if adding a focused regression test.

Acceptance:
- `sanction_event` insert and `governance_log::append(kind=...)` happen in the same fresh transaction/savepoint.
- No use of the submit-vote transaction connection.
- If `governance_log::append` fails, no `sanction_event` row remains.

### T3 — Bridge room lookup by case_id

Files likely affected:
- `services/bridge/src/bridge_room.rs`
- `services/bridge/src/sanction_handler.rs`

Add helper(s):
- `lookup_by_case(conn, case_id) -> Vec<(room_type, matrix_room_id)>` or equivalent.

Acceptance:
- No new bridge DB table/migration.
- Empty room list returns `applied: false` with an explicit reason; not a 500.

### T4 — Matrix power-level state update

Files likely affected:
- `services/bridge/src/sanction_handler.rs`

Implementation shape:
- Use `state.http_client` + `state.config.as_token`.
- GET current `m.room.power_levels` for each room.
- Update / insert `users[subject_mxid]` to the computed power level.
- PUT state back to Matrix.
- Continue best-effort across rooms: one room failure should not prevent attempts on other rooms.

Acceptance:
- Response reports `applied: true` only if at least one room update succeeded.
- Response reason includes counts: rooms_found, rooms_applied, rooms_failed.
- Unauthorized remains 401 before any side-effect.

### T5 — Tests

Workspace-side:
- Extend m2-late e2e for `case_id` payload and CR-A behaviour if practical.
- Keep full e2e scoped through wrapper/log capture.

Bridge-side:
- Add handler tests with mock Matrix endpoints:
  - bad bearer → 401, no Matrix calls.
  - no rooms for case → 200 `applied=false`.
  - one room for case → GET+PUT power-level calls and `applied=true`.

### T6 — Pilot verification

Manual/operator gate:
- Verify `BRIDGE_SANCTION_CALLBACK_URL` is set in pilot environment.
- Verify `sanction_subscriber` has exactly one active bridge row.
- Smoke POST to bridge `/brehon/sanction-event` with Bearer secret in a non-production-safe test path, or run a controlled governance sanction if pilot state permits.

Do not read secrets into chat. Record only presence/absence and redacted URL host/path.

---

## 5. Risks / tripwires

- **B-actor creep:** any plan task adding user link flow, portable IDs, or `actor_app_link` is out of scope.
- **Payload schema drift:** adding `case_id` is recommended; removing `governance_log_entry_hash` or `subject_actor_pseudonym` is forbidden.
- **Matrix semantics overclaim:** do not claim `hide_content` redacts historical content unless the implementation actually redacts events.
- **Atomicity partial fix:** moving `governance_log::append` closer to the insert but still outside the same transaction does not close CR-A.
- **Bridge cargo isolation:** `services/bridge` is workspace-excluded; workspace cargo does not validate it.
- **Secrets:** do not print `BRIDGE_CALLBACK_SECRET`, `.env`, or bearer values.

---

## 6. User-confirmed planning decisions (clarify pass 2026-06-12)

All three §6 questions resolved via `/brehon-clarify --mode user-relay` 2026-06-12. The planner MUST honour these:

1. **D1 — `case_id` in `SanctionEventPayload`: YES** (clarify-DQ `a3d0e9941441-058`). Add it as a backward-compatible Matrix-bridge extension field; bridge resolves rooms via a new `bridge_room.lookup_by_case(case_id)` returning all `(room_type, matrix_room_id)` rows. No migration, no new HTTP route. Keep `governance_log_entry_hash` + `subject_actor_pseudonym`. Persisting `case_id` in the `sanction_event` table is OPTIONAL — prefer no migration unless audit/e2e queryability needs it (T1).
2. **D2 — Power-levels only, NO historical redaction** (clarify-DQ `a3d0e9941441-059`). `ban`/`mute`/`prevent_post`/`mute_voice` → below post threshold; `hide_content`/`restrict_reach` → reduced posting level with reason `redaction_not_available_in_m2_late_2`. Full `hide_content` redaction is a separate phase (needs a subject→event index).
3. **D3 — Pilot verification IS plan task T6** (clarify-DQ `a3d0e9941441-060`). Confirm `BRIDGE_SANCTION_CALLBACK_URL` set, exactly one active `sanction_subscriber` row, bridge callback reachable. Record presence/absence + redacted host/path only — never read `BRIDGE_CALLBACK_SECRET` or `.env` into chat.
4. **D3-append — CR-A is mechanical, no new helper** (clarify-DQ `a3d0e9941441-061`). Use the `&mut (&mut *conn).into()` reborrow to call `append` inside one fresh `conn.run_transaction()` in `enqueue_sanction_event`, opened AFTER the HTTP POSTs. Canonical exemplar: `admin_assign_jury.rs:218`.

**Planning gate: CLEAR.** All clarify-DQ entries (`a3d0e9941441-058` … `-061`) resolved.
