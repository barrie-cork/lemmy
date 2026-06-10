# Plan: m2-late-2 — Bridge Sanction Enforcement + CR-A Atomicity Fix

## 1. Summary

m2-late-2 closes the two deliberate m2-late-1 stubs plus one operational verification gate:

1. **CR-A atomicity fix.** `enqueue_sanction_event` (`crates/api/api/src/governance/sanction_publisher.rs:178,196`) currently inserts `sanction_event` and appends `sanction_published` / `sanction_event_delivery_failed` to `governance_log` on **two separate** `get_conn` calls. CR-A from the m2-late-1 retro at line 78 deferred this to m2-late-2. The fix wraps both writes in one `conn.run_transaction(...)` while keeping the outer vote transaction untouched (R8).
2. **Bridge power-level enforcement.** `services/bridge/src/sanction_handler.rs:122-125` currently ACKs but returns `applied: false`. The bridge now looks up all `bridge_room` rows for the case and PUTs a `m.room.power_levels` state event for each room with a per-`SanctionKind` level for the subject puppet. `hide_content` returns `applied: true` with a reason explaining historical redaction is deferred (no subject→event index in this phase).
3. **Pilot verification.** Confirm `BRIDGE_SANCTION_CALLBACK_URL` is set in the pilot env, `sanction_subscriber` has one active row, and a controlled smoke-POST to `/brehon/sanction-event` reaches the bridge.

**Out of scope:** B-actor portable-ID linkage / OAuth link flow / `actor_app_link` table. User-confirmed out of scope 2026-06-07 (OQ-ADR016-03 deferred).

## 2. Source

- `.claude/PRPs/briefs/m2-late-2-planning-1.md` — the brief (this plan's user decisions: case_id in payload, partial hide_content, in-plan pilot gate).
- `.claude/PRPs/reports/m2-late-1-retro.md` — CR-A + power-level stub + pilot-seed carry-forwards (§"What to carry forward").
- `.claude/PRPs/handovers/m2-late-2-bootstrap.md` — advisor bootstrap; decisions baked in (B-actor out; pilot verification in scope; bridge cargo isolation rule).
- `.claude/PRPs/plans/m2-late.plan.md` — predecessor plan (T4 publisher, T6 seed, T7 handler, T8 e2e). The spawn site at `submit_jury_vote.rs:198` shipped with m2-late-1.
- ADRs: ADR-008 (append-only audit), ADR-015 (pseudonymity), ADR-016 (B-publish contract).
- OQs: OQ-ADR016-02 (resolved 2026-06-07; webhook transport, universal event schema, at-least-once), OQ-ADR016-04 (resolved 2026-06-07; `SanctionKind` enum + per-app translation + `not_applicable` ack).
- Lessons that bind:
  - `feedback_multi_write_handlers_need_transactions.md` — the CR-A fix IS this pattern; the impl brief cites it explicitly.
  - `feedback_validate_pending_laptop_write_then_stop.md` — workers write the DQ and stop.
  - `feedback_features_full_workspace_only.md` — `--features full` is workspace-scope only.
  - `feedback_lemmy_error_no_std_error.md` — LemmyResult error-shape.
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate unique `include!`/`mod` anchors before editing `crates/server/tests/e2e.rs`.
  - `feedback_junior_task_updated_at_unreliable.md` — SSH log tail first diagnostic, not daemon `updatedAt`.
  - Bridge `Cargo.toml`/`Cargo.lock` change → raise `validate-pending-laptop-linux` (per `feedback_linux_compile_proof_is_a_gate.md` Option-2 trigger).

## 3. Problem statement

After m2-late-1, the B-publish flow has three gaps:

1. **CR-A atomicity gap.** A crash or DB error between the `sanction_event` insert and the `governance_log::append` leaves a `sanction_event` row without a paired audit-log entry, violating the ADR-008 invariant "every governance write emits an audit log entry". Two separate pool connections with no transaction.
2. **Bridge is a no-op.** The bridge receives the event, authenticates, and ACKs `applied: false` with no Matrix state change. The cross-app enforcement the contract promises is unimplemented.
3. **No end-to-end confidence in pilot.** The startup seed in `crates/server/src/lib.rs:252-260` runs on every boot, but the pilot has not been checked for an active subscriber row or a reachable bridge callback URL.

## 4. Solution statement

```
[pre-m2-late-2: enqueue_sanction_event does 2 separate get_conn calls]

enqueue_sanction_event (sanction, ctx)
  ├─ map_sanction_action → Option<SanctionKind>            [unchanged]
  ├─ subject = actor_pseudonym::get_or_create(...)         [unchanged]
  ├─ subscribers = SELECT * FROM sanction_subscriber ...   [unchanged]
  ├─ entry_hash lookup (sql_query)                          [unchanged]
  ├─ POST to each subscriber                                [unchanged]
  ├─ conn1 = get_conn(); INSERT INTO sanction_event ...    [CR-A split point]
  ├─ conn2 = get_conn(); governance_log::append(...)        [CR-A split point]
  └─ Ok(())

[m2-late-2:]

enqueue_sanction_event (sanction, ctx)
  ├─ (steps 1-5 unchanged — single conn reused for both writes)
  └─ conn = get_conn();
       conn.run_transaction(|conn| {
         INSERT INTO sanction_event ...;                   [T2, atomic]
         governance_log::append(&mut (&mut *conn).into(), kind, payload, Some(subject)).await?;  [T2]
         Ok(())
       })
       ↑ SAVEPOINT semantics when called from inside a parent tx (per federation_outbox.rs:59 pattern);
         here we're outside the vote tx, so this is a top-level transaction.

[pre-m2-late-2: bridge handler]

POST /brehon/sanction-event
  ├─ Bearer verify
  ├─ log
  ├─ _ = sanction_kind_to_power_level(...);               [stub]
  └─ 200 { applied: false, reason: "acknowledged..." }

[m2late-2:]

POST /brehon/sanction-event
  ├─ Bearer verify
  ├─ parse SanctionEventPayload { ..., case_id: i32 }      [T1, new field]
  ├─ subject_mxid = state.puppet_map.ensure_puppet(subject_actor_pseudonym).await?
  ├─ rooms = bridge_room::lookup_by_case(conn, case_id)    [T3, new helper]
  ├─ for each room:
  │    GET /_matrix/client/v3/rooms/{id}/state/m.room.power_levels
  │    power_level = power_level_for(sanction_kind)        [T4, per-kind mapping]
  │    PUT /_matrix/client/v3/rooms/{id}/state/m.room.power_levels (with users[subject_mxid]=level)
  └─ 200 { applied, reason: "applied:N/total:M", applied_at }
       applied = true iff at least one room updated successfully
       reason enumerates rooms_found / rooms_applied / rooms_failed
```

`m2-late-2` does **not** change the spawn site (shipped with m2-late-1 at `submit_jury_vote.rs:198`), the subscriber seed (shipped at `crates/server/src/lib.rs:252-260`), or the `BRIDGE_CALLBACK_SECRET` auth (shipped).

## 5. Metadata

| Field | Value |
|---|---|
| Type | CROSS_CUTTING (publisher atomicity + bridge enforcement) |
| Complexity | LOW–MEDIUM |
| Crates Affected | `lemmy_api` (sanction_publisher), `services/bridge` (sanction_handler + bridge_room), `crates/server` (e2e), `services/bridge` tests (new) |
| v0/M Step | m2-late-2 (post-v1, ADR-016 first reference integration follow-on) |
| Dependencies | m2-late-1 (shipped: enqueue_sanction_event + SanctionEventPayload + sanction_event/sanction_subscriber models + bearer auth + spawn site + seed) |
| Estimated tasks | 8 (Task 0 pre-flight + T1–T5 impl + T6 pilot gate + T7 retro) |
| Target impl-task model | `sonnet-4-6` |
| Complexity score | 7/8 — see §5.1. **Under the Sonnet threshold → no split-DQ required.** |
| Forbidden-window applicability | Standard — workspace cargo on laptop, validate-pending-laptop DQ (pre-Shape-G). |
| Estimated cargo budget | ~6 GB peak (workspace `check --features full` + bridge `cargo check`); no new deps in the workspace. |

### 5.1 Complexity score breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 5 impl tasks (T1–T5); 5 − 5 = 0 |
| Migrations touched | +2 each | 0 | No schema change. `case_id` lives in the webhook DTO, not in the `sanction_event` table. |
| Crates touched | +1 each | 3 | `crates/api/api`, `services/bridge` (src + tests), `crates/server` (e2e) |
| `crates/server/tests/e2e/*.rs` edits | +3 each | 3 | T5 — extends `m2_late.rs` + 1 `include!`-adjacent `e2e.rs` edit |
| N-callsite (T1 adds `case_id` to existing public struct) | +1 | 1 | `SanctionEventPayload` (publisher) + bridge mirror struct (sanction_handler) — pre-locate per `feedback_struct_field_add_enumerate_all_callsites.md` |
| New ADR-affecting decisions | +2 each | 0 | Brief decisions are scope clarifications, not new ADRs |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Peak ~6 GB |
| **Total** | — | **7** | Threshold 8 → **does NOT fire split-DQ** |

## 6. Relationship to other m2 sub-phases

- **Depends on m2-late-1 (PR #192, shipped 2026-06-08)** — the `enqueue_sanction_event` machinery, the `SanctionEventPayload` DTO, the `sanction_event` + `sanction_subscriber` Diesel models, the bearer auth pattern, the spawn site at `submit_jury_vote.rs:198`, and the startup seed at `crates/server/src/lib.rs:252-260`.
- **Followed by M3 (town halls / MatrixRTC)** — m2-late-2 is the second-and-last m2-late sub-phase; M3 is its own phase lane, scoped under `v2-messaging-rtc.prd.md` §V2c / `.claude/PRPs/prds/` (not yet authored per roadmap).
- **B-actor (portable-ID linkage) deferred** — out of scope for m2-late-2; OQ-ADR016-03 remains parked.
- **M2-late Phase 7 deferred to a future post-m2-late-2 lane** — or absorbed into M3 if M3 introduces a second app subscriber.

## 7. Preflight guardrails inherited from prior phases

- **R1:** `i32 ↔ i64` comparisons use `i64::from(...)`, never `as` cast.
- **R2:** clippy invocations use `--no-deps` uniformly.
- **R3:** clippy/check invocations on governance crates use `--features full` (governance code is `#[cfg(feature = "full")]`).
- **R4:** Task 0 enumerates ALL probes explicitly.
- **R7:** workers WRITE the `validate-pending-laptop` DQ entry and STOP; they do NOT run workspace cargo on the daemon.
- **R8 (CRITICAL):** the spawn in `submit_jury_vote.rs:198` is OUTSIDE the vote transaction. The CR-A fix uses a FRESH `conn.run_transaction(...)` inside `enqueue_sanction_event` — never touches the vote transaction's `conn`.
- **R9 (CRITICAL):** `services/bridge` stays in the root `Cargo.toml` `exclude` array. The zero-Matrix-deps gate (`cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` ⇒ 0) is in every bridge-touching DoD.
- **R10 (NEW):** any change to `services/bridge/Cargo.toml` or `services/bridge/Cargo.lock` raises a `validate-pending-laptop-linux` DQ per `feedback_linux_compile_proof_is_a_gate.md` Option-2 trigger.

## 8. Flow design

### 8.1 Before (m2-late-1 HEAD)

```
process_vote (submit_jury_vote.rs:198)
  tx { insert sanction; append "sanction_created" }  [commit]
  └─ (after commit) tokio::spawn(enqueue_sanction_event) [R8]
        ├─ map → sanction_kind
        ├─ subject = actor_pseudonym::get_or_create
        ├─ subscribers = SELECT active
        ├─ entry_hash = SELECT entry_hash FROM governance_log
        ├─ for sub: reqwest POST → Bearer
        ├─ conn1 = get_conn(); INSERT INTO sanction_event      ← CR-A split
        └─ conn2 = get_conn(); governance_log::append          ← CR-A split

POST /brehon/sanction-event (services/bridge/src/sanction_handler.rs)
  ├─ Bearer verify
  ├─ log
  ├─ _ = sanction_kind_to_power_level(...)   ← stub
  └─ 200 { applied: false }                   ← never applied
```

### 8.2 After (m2-late-2)

```
process_vote (unchanged)
  tx { insert sanction; append "sanction_created" }  [commit]
  └─ (after commit) tokio::spawn(enqueue_sanction_event)  [R8 — same as m2-late-1]
        ├─ (steps 1-5 unchanged)
        └─ conn = get_conn();
              conn.run_transaction(|conn| {
                INSERT INTO sanction_event
                governance_log::append(&mut (&mut *conn).into(), kind, payload, Some(subject))
              })  ← single fresh transaction, atomic per CR-A

POST /brehon/sanction-event
  ├─ Bearer verify
  ├─ parse { ..., case_id: i32 }                          [T1]
  ├─ subject_mxid = puppet_map.ensure_puppet(pseudonym)   [T4]
  ├─ rooms = bridge_room::lookup_by_case(case_id)         [T3]
  ├─ for each room:
  │    GET  /_matrix/client/v3/rooms/{id}/state/m.room.power_levels
  │    users[subject_mxid] = power_level_for(sanction_kind)  [T4]
  │    PUT  /_matrix/client/v3/rooms/{id}/state/m.room.power_levels
  └─ 200 { applied: true|false, reason: "applied:N/total:M", applied_at }
```

## 9. Mandatory reading (impl-task subagent MUST read before first edit)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `crates/api/api/src/governance/sanction_publisher.rs` | 1-210 | The current publisher; CR-A fix is at lines 178-203. |
| P0 | `crates/api/api/src/governance/federation_outbox.rs` | 55-146 | The `&mut (&mut *conn).into()` reborrow idiom — MIRROR for T2's nested `governance_log::append` call. |
| P0 | `crates/db_schema/src/source/governance/governance_log.rs` | 280-339 | `governance_log::append(pool, kind, payload, pseudonym)` signature; verify the conn reborrow type-checks for T2. |
| P0 | `services/bridge/src/sanction_handler.rs` | 1-160 | Current handler; T1 changes the struct; T4 replaces the stub. |
| P0 | `services/bridge/src/bridge_room.rs` | 1-55 | Current shape; T3 adds `lookup_by_case`. |
| P0 | `services/bridge/src/provision.rs` | 1-40 | `reqwest` + `bearer_auth(&as_token)` + `error_for_status` — MIRROR for T4's GET/PUT. |
| P0 | `services/bridge/src/puppet.rs` | 60-115 | `puppet_map.ensure_puppet(pseudonym)` — T4 uses this to map subject pseudonym → MXID. |
| P0 | `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` | all | The CR-A fix IS this pattern. |
| P0 | `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` | all | R7 enforcement. |
| P0 | `.claude/lessons/feedback_linux_compile_proof_is_a_gate.md` | all | R10 (bridge Cargo.toml/lock changes raise linux DQ). |
| P1 | `services/bridge/src/appservice.rs` | 209-236 | Router wiring + `route_layer` placement (sanction-event route already exists; T4 changes only the handler body). |
| P1 | `services/bridge/src/config.rs` | 1-60 | `as_token`, `tuwunel_url`, `bridge_callback_secret` — T4 reads `as_token` + `tuwunel_url`. |
| P1 | `services/bridge/src/sanction_handler.rs` | 130-159 | `sanction_kind_to_power_level` already exists; T4 replaces the stub but should keep the per-kind mapping table nearby. |
| P1 | `crates/api/api/src/governance/governance_log.rs` | 39-66 | The `pub use` shim that re-exports the `ENTRY_KIND_*` consts. |
| P1 | `crates/server/tests/e2e/m2_late.rs` | 1-400 | The existing m2-late-1 e2e; T5 extends it for the new payload field. |
| P1 | `crates/server/tests/e2e.rs` | 124-148 | `mod common; use common::governance_fixtures;` + `include!` wiring. |
| P1 | `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` | all | T5 pre-locate the unique `include!` anchor before editing `e2e.rs`. |
| P2 | `services/bridge/tests/room_provisioning.rs` | 1-65 | The bridge `#[ignore]`-gated test pattern; T5 mirrors it. |
| P2 | `services/bridge/Cargo.toml` | 1-25 | `axum 0.8`, `reqwest 0.12`, no `[dev-dependencies]`; T5 may need `tokio` `test-util` feature or wire a manual mock — decide during T5. |

## 10. Patterns to mirror

### 10.1 Conn-reborrow inside `run_transaction` (CR-A fix)

```rust
// SOURCE: crates/api/api/src/governance/federation_outbox.rs:140-175
// (adapted to the publisher case)
let mut pool = ctx.pool();
let conn = &mut get_conn(&mut pool).await?;
conn.run_transaction(async |conn| {
  insert_into(sanction_event_dsl::table)
    .values(&event_form)
    .execute(&mut *conn)  // pass the conn reborrowed; diesel-async's
                           // AsyncPgConnection implements diesel::Connection
    .await?;
  governance_log::append(
    &mut (&mut *conn).into(),  // canonical reborrow for `&mut DbPool<'_>`
    kind,
    payload,
    Some(subject.clone()),
  )
  .await?;
  Ok(())
})
.await
```

`governance_log::append` opens its OWN inner `run_transaction` for the INSERT+signature UPDATE atomicity (line 308-330 of `governance_log.rs`). Inside the CR-A outer transaction, diesel-async promotes that inner `run_transaction` to a SAVEPOINT — preserving the existing ADR-008 atomicity invariant. The `enqueue_sanction_event` outer `conn.run_transaction` is the new layer; both writes are atomic across the failure boundary.

### 10.2 Bridge Matrix state-event GET/PUT (T4)

```rust
// SOURCE: services/bridge/src/provision.rs:23-35 (createRoom POST shape)
// MIRROR: same client, different endpoint, GET first then PUT back

// GET current power levels
let encoded_room_id = room_id.replace(':', "%3A");
let get_url = format!(
  "{}/_matrix/client/v3/rooms/{}/state/m.room.power_levels",
  state.config.tuwunel_url, encoded_room_id
);
let current_pl: serde_json::Value = state
  .http_client
  .get(&get_url)
  .bearer_auth(&state.config.as_token)
  .send().await?
  .error_for_status()?
  .json().await?;

// Mutate users[subject_mxid] to the per-kind level; preserve all other keys
let mut pl = current_pl;
let users = pl.get_mut("users").cloned().unwrap_or_else(|| serde_json::json!({}));
// ... insert users[subject_mxid] = level
// PUT
let put_url = format!(
  "{}/_matrix/client/v3/rooms/{}/state/m.room.power_levels",
  state.config.tuwunel_url, encoded_room_id
);
let resp = state.http_client
  .put(&put_url)
  .bearer_auth(&state.config.as_token)
  .json(&pl)
  .send().await?;
resp.error_for_status()?;
```

### 10.3 Power-level mapping (T4)

```rust
// SOURCE: services/bridge/src/sanction_handler.rs:152-159 (existing fn — replace
// the body or extend to return a richer struct)

fn sanction_kind_to_power_level(sanction_kind: &str) -> (i32, &'static str) {
  match sanction_kind {
    // Cannot post, cannot speak. Posting, joining call, etc. all blocked.
    "ban" | "mute" | "prevent_post" => (0, "events_default_blocked"),
    // Voice: blocks matrixrtc voice events; the rooms without matrixrtc
    // fall back to a posting block as conservative translation.
    "mute_voice" => (-1, "voice_threshold_blocked"),
    // Matrix has no native reach primitive; reduce posting level.
    "restrict_reach" => (25, "reach_reduction_via_posting_level"),
    // Historical redaction deferred (no subject→event index in m2-late-2).
    "hide_content" => (0, "redaction_not_available_in_m2_late_2_posting_block_only"),
    _ => (50, "no_change_default_member"),
  }
}
```

`hide_content` returns `(0, "redaction_not_available_in_m2_late_2_posting_block_only")` so the bridge applies a posting block AND reports the reason honestly — the audit trail makes the partial coverage visible (OQ-ADR016-04 §d).

### 10.4 Bridge room lookup by case_id (T3)

```rust
// SOURCE: services/bridge/src/bridge_room.rs:19-29 (existing lookup, single-key)
// MIRROR: new helper, multi-row

/// Return all Matrix room IDs provisioned for the given case, regardless of room_type.
pub fn lookup_by_case(conn: &Connection, case_id: i64) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT room_type, matrix_room_id FROM bridge_room WHERE case_id = ?1 \
         AND matrix_room_id IS NOT NULL"
    )?;
    let rows = stmt.query_map(params![case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect()
}
```

The PRIMARY KEY is `(case_id, room_type)` (line 13), so this returns one row per room_type provisioned for the case (jury, appeal, emergency, etc. — depending on which `provision_*` paths fired).

### 10.5 SanctionEventPayload field addition (T1)

```rust
// SOURCE: crates/api/api/src/governance/sanction_publisher.rs:38-46
// MIRROR: same field shape on the bridge mirror struct.

#[derive(Debug, Serialize, Deserialize)]
pub struct SanctionEventPayload {
    pub sanction_kind: SanctionKind,
    pub subject_actor_pseudonym: String,
    pub effective_from: DateTime<Utc>,
    pub effective_until: Option<DateTime<Utc>>,
    pub governance_log_entry_hash: String,
    // NEW (T1, m2-late-2):
    pub case_id: i32,    // i32 mirrors CaseTransitionEvent; bridge maps to i64 for bridge_room
}

// services/bridge/src/sanction_handler.rs:55-65 (mirror struct):
#[derive(Debug, Deserialize)]
pub struct SanctionEventPayload {
    pub sanction_kind: String,
    pub subject_actor_pseudonym: String,
    pub effective_from: String,
    pub effective_until: Option<String>,
    pub governance_log_entry_hash: String,
    pub case_id: i32,    // NEW (T1)
}
```

**N-callsite enumeration (T1):** pre-locate ALL construction sites:
- `crates/api/api/src/governance/sanction_publisher.rs:130-138` — the only `SanctionEventPayload { ... }` literal (publisher). Add `case_id: sanction.case_id.0` (SanctionCaseId is i32; the `Sanction` struct's `case_id` field).
- `services/bridge/src/sanction_handler.rs:55-65` — the bridge mirror struct definition only; deserialised from JSON, no construction site. Adding the field is automatic.
- e2e: the existing `m2_late.rs` test constructs the mock subscriber URL but does NOT construct a payload directly (the publisher builds the payload); the test asserts on the JSON. T5's payload-field assertion will need to read `case_id` from the captured JSON.

## 11. Files to change

| File | Action | Tasks | Justification |
|---|---|---|---|
| `crates/api/api/src/governance/sanction_publisher.rs` | UPDATE | T1, T2 | T1 adds `case_id` to `SanctionEventPayload`; T2 wraps the two writes in a single `conn.run_transaction`. |
| `services/bridge/src/sanction_handler.rs` | UPDATE | T1, T4 | T1 mirrors the `case_id` field in the local struct; T4 replaces the stub with the room-lookup + GET+PUT power-level enforcement. |
| `services/bridge/src/bridge_room.rs` | UPDATE | T3 | New `lookup_by_case` helper. |
| `crates/server/tests/e2e/m2_late.rs` | UPDATE | T5 | Extend the existing e2e to assert `case_id` in the captured payload; assert governance_log entry matches expected kind under both success and delivery-failure paths. |
| `services/bridge/tests/sanction_enforcement.rs` | CREATE | T5 | `#[ignore]`-gated bridge integration test (mirror of `room_provisioning.rs`); in-process mock Matrix server. |
| `services/bridge/Cargo.toml` | UPDATE (maybe) | T5 | Add `tokio` `test-util` feature only if the mock server needs `tokio::io::duplex` or similar. Decision deferred to T5. |
| `.claude/PRPs/reports/m2-late-2-retro.md` | CREATE | T7 | Per-role retro. |
| `.claude/PRPs/reports/m2-late-2-verify.md` | CREATE | post-T7 | `/brehon-verify` output. |

## 12. NOT building in m2-late-2

- **B-actor portable-ID linkage** — out of scope, user-confirmed 2026-06-07 (OQ-ADR016-03 deferred). STOP-and-ask tripwire.
- **Historical message redaction for `hide_content`** — Matrix has no native primitive; building a subject→event index in this phase would expand scope. The reason string says so explicitly. STOP-and-ask if user wants it.
- **A new `sanction_event` table column for `case_id`** — `case_id` lives in the webhook DTO only; the `sanction_event` row is recoverable from `sanction_id` → `sanction.case_id`. No migration in m2-late-2.
- **A conn-accepting variant of `governance_log::append`** — the federation_outbox pattern (`&mut (&mut *conn).into()` reborrow) type-checks. STOP-and-ask if the reborrow fails to compile (would be a signature change with downstream impact).
- **A retry queue for delivery failure** — at-least-once via fire-and-forget is the v0 contract; retry is M3 or later. The `sanction_event_delivery_failed` audit row is the v0 fallback.
- **Subscribing non-bridge apps** — the bridge is the only subscriber; the env-var seed remains the only registration path.
- **Editing `submit_jury_vote.rs`** — the spawn site at `:198` shipped with m2-late-1 and is unchanged.
- **Editing `crates/server/src/lib.rs`** — the seed at `:252-260` shipped with m2-late-1 and is unchanged.
- **A migration** — none in this phase. STOP tripwire.

## 13. Step-by-step tasks

> **Pre-Shape-G:** every workspace-cargo task writes a `validate-pending-laptop` DQ entry and STOPS (R7). Bridge cargo (`cd services/bridge && cargo check`) runs in-task (workspace-excluded).
> **One commit per task.** Cohort dispatch: tasks in this plan are small (≤4 files each) and `[P]`-cohorts are not used because T2 reads T1's struct change and T4 reads T3's helper. The plan is strictly serial. Skip cohort framing.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `m2-late-2`; confirm branch is `phase-m2-late-2`; confirm m2-late-1 deliverables are intact on the base branch.

**Probes (R4 — enumerate ALL explicitly):**

```bash
# Probe 0 — Docker daemon (for e2e)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }
# Probe 1 — branch
git branch --show-current   # EXPECT: phase-m2-late-2 (cut by BM-task before T1)
# Probe 2 — wrapper honors -p
./scripts/brehon/cargo-check.sh -p lemmy_utils > .claude/PRPs/debug/m2-late-2-audit-p.log 2>&1; echo "exit: $?"
# Probe 3 — wrapper honors --features full
./scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full > .claude/PRPs/debug/m2-late-2-audit-features.log 2>&1; echo "exit: $?"
# Probe 4 — negative: wrapper propagates non-zero on bogus feature
./scripts/brehon/cargo-check.sh -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m2-late-2-audit-neg.log 2>&1; echo "exit (EXPECT non-zero): $?"
# Probe 5 — bridge crate compiles standalone (m2-late-1 baseline)
cd services/bridge && cargo check > /tmp/m2-late-2-audit-bridge.log 2>&1; echo "exit: $?"; cd -
# Probe 6 — zero-Matrix-deps gate baseline
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'   # EXPECT: 0
# Probe 7 — sanction_publisher.rs has TWO get_conn calls (CR-A split point)
rg -n "get_conn" crates/api/api/src/governance/sanction_publisher.rs   # EXPECT: ≥2 hits
# Probe 8 — sanction_handler.rs has the stub
rg -n 'applied: false' services/bridge/src/sanction_handler.rs   # EXPECT: 1 hit
# Probe 9 — bridge_room.rs lookup is single-key
rg -n "pub fn lookup" services/bridge/src/bridge_room.rs   # EXPECT: 1 hit
# Probe 10 — spawn site intact (m2-late-1)
rg -n "tokio::spawn.*enqueue_sanction_event" crates/api/api/src/governance/submit_jury_vote.rs   # EXPECT: 1 hit
# Probe 11 — startup seed intact (m2-late-1)
rg -n "seed_sanction_subscriber" crates/server/src/lib.rs   # EXPECT: 1 hit
# Probe 12 — SanctionEventPayload has 5 fields (not 6)
rg -n "pub (effective|governance_log|sanction_kind|subject)" crates/api/api/src/governance/sanction_publisher.rs   # EXPECT: 5 hits
```

**EXPECT block:** Probes 0, 1, 2, 3, 5, 6, 10, 11 confirm m2-late-1 base intact. Probes 7, 8, 9, 12 confirm the CR-A + stub + new-field targets exist. Probe 4 exits non-zero. **No commit at Task 0.**

### Task 1: `case_id` in `SanctionEventPayload` + bridge mirror

**ACTION:** add `case_id: i32` field to both `SanctionEventPayload` structs (publisher + bridge mirror); populate the field at the publisher's construction site. `requires: -`.

```yaml
modifies:
  - crates/api/api/src/governance/sanction_publisher.rs   # add case_id to struct + populate at construction
  - services/bridge/src/sanction_handler.rs                # add case_id to mirror struct
requires:
  - task: 0
    reason: probes confirmed
```

**IMPLEMENT:**
- `crates/api/api/src/governance/sanction_publisher.rs`:
  - Add `pub case_id: i32,` to `SanctionEventPayload` (alphabetical insertion is the convention here — currently the order is: `sanction_kind`, `subject_actor_pseudonym`, `effective_from`, `effective_until`, `governance_log_entry_hash`; insert `case_id` between `effective_until` and `governance_log_entry_hash` to keep the new field grouped with the time fields, OR keep alphabetical strict and add at the very top. Pick the former — group by related concept.)
  - At the `SanctionEventPayload { ... }` construction site (line 130-138), add `case_id: sanction.case_id.0`. The `Sanction` struct's `case_id` is `SanctionCaseId` (a newtype around `i32`); `.0` unwraps.
- `services/bridge/src/sanction_handler.rs`:
  - Add `pub case_id: i32,` to the local `SanctionEventPayload` struct (line 55-65). This is `Deserialize` only — no construction site to update.

**MIRROR:** see §10.5.

**GOTCHA (N-callsite):** the publisher struct has 2 call patterns (definition + construction). The bridge mirror is `Deserialize` only. The pre-locate is in §10.5.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 1`, then STOP.

### Task 2: CR-A atomicity fix in `enqueue_sanction_event`

**ACTION:** wrap the `sanction_event` INSERT and `governance_log::append` in a single `conn.run_transaction(...)`. `requires: T1`.

```yaml
modifies:
  - crates/api/api/src/governance/sanction_publisher.rs   # wrap lines 178-203 in conn.run_transaction
requires:
  - task: 1
    reason: struct unchanged in shape from m2-late-1; T1 only added a field
```

**IMPLEMENT:** per §10.1 — the canonical reborrow pattern. The current code is two `get_conn` calls and two write operations. Replace with one `get_conn` + one `run_transaction` wrapping both.

**Specific edits:**
- Keep the existing `let mut pool = ctx.pool(); let conn = &mut get_conn(&mut pool).await?;` but rename / restructure to a single conn scoped to the closure.
- Move the `event_form` construction INSIDE the closure (it borrows `payload.sanction_kind` and `subject`; both are available inside the closure scope).
- Move the `governance_log::append(...)` call INSIDE the closure, using `&mut (&mut *conn).into()` for the `&mut DbPool<'_>` argument.
- Remove the second `let mut pool = ctx.pool(); let conn = &mut get_conn(&mut pool).await?;` block.

**MIRROR:** `crates/api/api/src/governance/federation_outbox.rs:140-175` — the conn reborrow + nested `run_transaction` pattern.

**GOTCHA:** `governance_log::append` opens its OWN `run_transaction` for INSERT+signature UPDATE atomicity (line 308-330 of `governance_log.rs`). Inside our outer `run_transaction`, diesel-async promotes that inner one to a SAVEPOINT. Behaviour unchanged for the inner writes. The new outer `run_transaction` is the layer that closes the CR-A gap.

**VALIDATE:** `validate-pending-laptop` `["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `phase_task: 2`, then STOP.

### Task 3: `bridge_room::lookup_by_case` helper

**ACTION:** add a new helper to `services/bridge/src/bridge_room.rs` that returns all `bridge_room` rows for a `case_id`. `requires: -` (parallel to T1).

```yaml
modifies:
  - services/bridge/src/bridge_room.rs   # add lookup_by_case
requires: []  # no upstream task; can run parallel to T1
```

**IMPLEMENT:** per §10.4. The function returns `Vec<(String /* room_type */, String /* matrix_room_id */)>` so T4 can label failures by room_type in the response reason.

**MIRROR:** `services/bridge/src/bridge_room.rs:19-29` (existing single-key `lookup`).

**VALIDATE (bridge is workspace-excluded, in-task):**
```bash
cd services/bridge && cargo check > /tmp/m2-late-2-t3-bridge.log 2>&1; echo "exit: $?"
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'   # EXPECT: 0
```

### Task 4: Bridge power-level enforcement

**ACTION:** replace the `applied: false` stub in `services/bridge/src/sanction_handler.rs` with the room-lookup + per-room GET/PUT power-level enforcement. `requires: T1, T3`.

```yaml
modifies:
  - services/bridge/src/sanction_handler.rs   # replace stub with enforcement
requires:
  - task: 1
    reason: T4 reads payload.case_id
  - task: 3
    reason: T4 calls bridge_room::lookup_by_case
```

**IMPLEMENT:** per §10.2 + §10.3. Pseudocode:

1. Bearer verify (unchanged).
2. Parse `SanctionEventPayload` (now has `case_id`).
3. `let subject_mxid = state.puppet_map.ensure_puppet(&payload.subject_actor_pseudonym).await?;` — T4 uses the existing M1 `puppet.rs` map; if the puppet registration fails, log warn and continue with `subject_mxid = format!("@_brehon_<escape>:...")` (the same shape the map would produce) so the power-level PUT still attempts.
4. `let rooms = bridge_room::lookup_by_case(conn, payload.case_id as i64)?;`
5. If `rooms.is_empty()` → return 200 `{ applied: false, reason: "no_rooms_for_case", applied_at }`.
6. For each `(room_type, matrix_room_id)`:
   - GET `m.room.power_levels` (per §10.2).
   - Set `users[subject_mxid] = power_level` from the per-kind mapping.
   - PUT `m.room.power_levels` (per §10.2).
   - Track applied/failed counts.
7. Return 200 `{ applied: rooms_applied > 0, reason: format!("applied:{rooms_applied}/total:{rooms_found}"), applied_at }`. Best-effort: one room failure does not abort the loop.

**MIRROR:** `services/bridge/src/provision.rs:23-35` (reqwest+as_token), `services/bridge/src/room_provisioner.rs:560-578` (URL percent-encoding of room ID).

**GOTCHA:** room IDs contain `:` which must be percent-encoded as `%3A` in the path. URL format mirrors `room_provisioner.rs:560-567`. Error handling: `error_for_status()?` on GET (skip the room, count failed); `error_for_status()?` on PUT (skip the room, count failed). Continue to next room — never abort the loop.

**VALIDATE (bridge in-task, workspace gate in-task):**
```bash
cd services/bridge && cargo check > /tmp/m2-late-2-t4-bridge.log 2>&1; echo "exit: $?"
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'   # EXPECT: 0
```

If `services/bridge/Cargo.toml` or `Cargo.lock` change is required (e.g. adding a `tokio` test-util feature), also raise `validate-pending-laptop-linux` per R10 / `feedback_linux_compile_proof_is_a_gate.md`. T4 itself should NOT change `Cargo.toml` (uses existing `reqwest` + `serde_json` + `anyhow`); the T5 test may.

### Task 5: Tests

**ACTION:** extend the existing `crates/server/tests/e2e/m2_late.rs` to assert `case_id` in the captured payload + the CR-A atomicity property. Add a `services/bridge/tests/sanction_enforcement.rs` `#[ignore]`-gated bridge integration test. `requires: T1, T2, T4`.

```yaml
modifies:
  - crates/server/tests/e2e/m2_late.rs   # extend existing test
creates:
  - services/bridge/tests/sanction_enforcement.rs   # mirror room_provisioning.rs pattern
requires:
  - task: 1
    reason: payload has case_id; test asserts on it
  - task: 2
    reason: CR-A test verifies the new transaction shape
  - task: 4
    reason: bridge enforcement test exercises the new handler
```

**IMPLEMENT — workspace e2e (`crates/server/tests/e2e/m2_late.rs`):**

Locate the assertion block that captures the mock-subscriber's POST body. Add a JSON parse + assert `case_id > 0` and matches the seeded `case_id`. Also add a new test function (mirror the `sanction_event_delivered_to_subscriber` pattern):
- New test: `cr_a_atomicity_holds_under_subscriber_failure`. Drive a quorum vote with a mock subscriber that returns 500. After the spawn runs:
  - Assert EXACTLY one row in `sanction_event` for the case.
  - Assert EXACTLY one row in `governance_log` with `entry_kind = "sanction_event_delivery_failed"`.
  - This proves both writes happened atomically (neither missing, both present). The opposite case (success subscriber) already exists in the m2-late-1 test.
- Pre-locate the unique `include!`/`mod` anchor in `crates/server/tests/e2e.rs` per `feedback_fix_impl_pre_locate_e2e_anchors.md` before editing.

**IMPLEMENT — bridge test (`services/bridge/tests/sanction_enforcement.rs`):**

Mirror `services/bridge/tests/room_provisioning.rs`:
- `#[tokio::test] #[ignore = "requires live bridge crate binary; no docker required"]` for a unit-shaped handler test that:
  - Builds an `AppState` with a stub `BridgeConfig` (dummy tokens) and an in-memory `PuppetMap`.
  - Constructs a `SanctionEventPayload` with a unique `case_id`.
  - Spawns the bridge AS transaction server bound to a random port (use `tokio::net::TcpListener::bind("127.0.0.1:0")`).
  - POSTs to the `/brehon/sanction-event` endpoint with the correct `BRIDGE_CALLBACK_SECRET`.
  - Asserts the response: `{ applied: false, reason: "no_rooms_for_case" }` (no bridge_room rows for the case).
- If time permits: add a second test that seeds `bridge_room` rows in a temp SQLite and asserts the `applied: true` path — but this requires a mock Matrix HTTP server (out of scope for m2-late-2, deferred).

**VALIDATE (workspace):** `validate-pending-laptop-e2e` `["cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full > %LOCALAPPDATA%\\\\Temp\\\\m2-late-2-e2e.log 2>&1 && echo E2E_EXIT_0 >> %LOCALAPPDATA%\\\\Temp\\\\m2-late-2-e2e.log || echo E2E_EXIT_NONZERO >> %LOCALAPPDATA%\\\\Temp\\\\m2-late-2-e2e.log\""]`, `phase_task: 5`, then STOP.

**VALIDATE (bridge in-task, optional):**
```bash
cd services/bridge && cargo test --no-run   # confirm compile only (the #[ignore] prevents run)
```

If `services/bridge/Cargo.toml` was changed in T5 (e.g. test-util feature), also raise `validate-pending-laptop-linux` (R10).

### Task 6: Pilot verification

**ACTION:** manual/operator gate. Verify pilot env, the active `sanction_subscriber` row, the bridge callback reachability, and (optionally) a controlled smoke-POST. `requires: T1, T2, T3, T4, T5`.

```yaml
# NO file changes. NO commit. NO cargo.
requires:
  - task: 1
    reason: payload case_id used in the smoke POST
  - task: 2
    reason: CR-A verified at the binary level; pilot confirms it runs against a real DB
  - task: 3
    reason: bridge_room::lookup_by_case consulted by T4
  - task: 4
    reason: bridge handler is the smoke-POST target
  - task: 5
    reason: e2e covers the binary side; pilot covers the operator side
```

**Probes (manual, NOT cargo):**

```bash
# Gate A — pilot env has the env var set (do NOT print the value)
test -n "$BRIDGE_SANCTION_CALLBACK_URL" && echo "GATE_A: env var set, host=$(echo $BRIDGE_SANCTION_CALLBACK_URL | sed -E 's|https?://||; s|/.*||')"

# Gate B — bridge HTTP port reachable
curl -sS -o /dev/null -w "%{http_code}\n" "$BRIDGE_SANCTION_CALLBACK_URL" || true
# EXPECT: 401 (unauthorized) or 405 (method not allowed) — proves the route exists
# A 502/503/connection-refused means the bridge is not running

# Gate C — Brehon PG has the seeded subscriber row
# (run via psql with credentials; do NOT print secrets)
psql "$BREHON_DATABASE_URL" -c \
  "SELECT id, substring(callback_url for 60) AS url_prefix, active, created_at \
   FROM sanction_subscriber WHERE active = true;"
# EXPECT: exactly 1 row, active = true, url_prefix matches the bridge host

# Gate D — controlled smoke POST (optional, only if a safe non-production
# sanction can be authored). The smoke body uses the existing m2_late test
# pattern. Record only HTTP status + response body, no secrets.
```

**EXPECT block:**
- Gate A: env var set.
- Gate B: HTTP 401 or 405 from `/brehon/sanction-event` (proves the route is registered).
- Gate C: exactly 1 active `sanction_subscriber` row.
- Gate D (optional): smoke POST returns 200; response reason contains `applied:N/total:M` or `no_rooms_for_case`.

**NO commit at Task 6** (operator gate; recorded in the verify report by the advisor, not in the codebase).

If any gate fails, surface to the user before proceeding to T7.

### Task 7: Retro

**Goal:** author retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lessons to `.claude/lessons/feedback_*.md` in the same retro commit.

```yaml
creates:
  - .claude/PRPs/reports/m2-late-2-retro.md
```

**REQUIRED retro sections:**
- Advisor — T6 outcome (pilot gates), any user decisions, any catch-fires.
- Planning — score accuracy (planned 7, actual?), brief usability.
- Impl — T2 (CR-A reborrow) and T4 (URL percent-encoding) — the two riskiest tasks.
- BM — branch cut, merge.

## 14. Testing strategy

- **Unit (compile-time):** `./scripts/brehon/cargo-check.sh --workspace --features full` (T1, T2, via validate-pending-laptop).
- **Lint:** `./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings` (R2/R3).
- **Bridge compile + zero-Matrix-deps:** `cd services/bridge && cargo check` + `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` ⇒ 0 (T3, T4).
- **Bridge test compile:** `cd services/bridge && cargo test --no-run` (T5).
- **e2e execution:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full ..."` (T5) — adds 1 new test; pre-existing m2_late tests still pass.
- **Pilot verification:** T6 (manual operator gate, no cargo).

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

### 15.3 Bridge gate (T3, T4, T5 if Cargo.toml touched)

```bash
cd services/bridge && cargo check                       # EXPECT: exit 0
cargo tree --workspace 2>/dev/null | grep -cE 'matrix-sdk|ruma'   # EXPECT: 0
```

If `services/bridge/Cargo.toml` or `Cargo.lock` change is required in T5, also run:
```bash
# Linux compile proof (R10 / feedback_linux_compile_proof_is_a_gate.md)
ssh 100.81.145.58 "cd ~/brehon-fork && cargo check -p brehon-bridge --manifest-path services/bridge/Cargo.toml"
# or whatever the project's validate-pending-laptop-linux pattern is
```

### 15.4 e2e (T5)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > %LOCALAPPDATA%\\Temp\\m2-late-2-e2e.log 2>&1 && echo E2E_EXIT_0 >> %LOCALAPPDATA%\\Temp\\m2-late-2-e2e.log || echo E2E_EXIT_NONZERO >> %LOCALAPPDATA%\\Temp\\m2-late-2-e2e.log"
# EXPECT: tail shows E2E_EXIT_0
```

### 15.5 Pilot verification (T6)

- Gate A: `BRIDGE_SANCTION_CALLBACK_URL` set.
- Gate B: `curl` to the URL returns 401 or 405 (route registered).
- Gate C: `SELECT FROM sanction_subscriber WHERE active = true` returns exactly 1 row.
- Gate D (optional): smoke POST returns 200 with reason containing `applied:N/total:M` or `no_rooms_for_case`.

### 15.6 Cross-cutting verification

- [ ] R7: every workspace-cargo task wrote a `validate-pending-laptop` DQ and STOPPED.
- [ ] R8: the spawn in `submit_jury_vote.rs:198` is unchanged; the CR-A fix is inside `enqueue_sanction_event`'s fresh `conn.run_transaction`.
- [ ] R9: `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` ⇒ 0; `services/bridge` not in `members`.
- [ ] R10: if `services/bridge/Cargo.toml` changed, `validate-pending-laptop-linux` raised.
- [ ] T1: pre-located both `SanctionEventPayload` construction/definition sites; the new `case_id` field is present in both publisher and bridge mirror structs.
- [ ] T2: the conn reborrow `&mut (&mut *conn).into()` type-checks for `governance_log::append` (no signature change).
- [ ] T4: URL percent-encoding for room IDs (`replace(':', "%3A")`) mirrors `room_provisioner.rs:560-567`.
- [ ] T4: `hide_content` returns `(0, "redaction_not_available_in_m2_late_2_posting_block_only")` and the response reason says so.
- [ ] T5: e2e test added; no `e2e.rs` edit outside the unique `include!` anchor.
- [ ] T6: pilot gates A, B, C passed (D optional); recorded in the verify report by the advisor.

## 16. Acceptance criteria

- [ ] All 8 tasks completed in dependency order.
- [ ] §15.1 (cargo check) exit 0 after T1, T2.
- [ ] §15.2 (clippy `--no-deps -- -D warnings`) exit 0 after T1, T2.
- [ ] §15.3 (bridge check + zero-Matrix-deps ⇒ 0) after T3, T4, T5.
- [ ] §15.4 (e2e) — 1 new test passes; pre-existing m2_late tests still pass.
- [ ] §15.5 (pilot gates A, B, C passed).
- [ ] §15.6 (cross-cutting) — all boxes ticked.
- [ ] §16a stories — all `[done]`.
- [ ] No edits to files outside §11.
- [ ] Retro committed per §13 Task 7.
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`.

## 16a. Stories (independently-testable behaviour units)

### Story 1: CR-A atomic publish audit

- **Composing tasks:** Task 1, Task 2
- **Checkpoint command:** `./scripts/brehon/cargo-check.sh --workspace --features full`
- **Expected output:** exit 0
- **Brief-Scope outputs to verify:**
  - `crates/api/api/src/governance/sanction_publisher.rs` contains `case_id: i32` in `SanctionEventPayload` AND has exactly one `conn.run_transaction(` wrapping both `INSERT INTO sanction_event` and `governance_log::append(`
  - `services/bridge/src/sanction_handler.rs` contains `case_id: i32` in its local `SanctionEventPayload`

### Story 2: Bridge enforces Matrix power levels

- **Composing tasks:** Task 3, Task 4
- **Checkpoint command:** `cd services/bridge && cargo check` + `cargo tree --workspace | grep -cE 'matrix-sdk|ruma'` ⇒ 0
- **Expected output:** exit 0 + zero
- **Brief-Scope outputs to verify:**
  - `services/bridge/src/bridge_room.rs` contains `pub fn lookup_by_case`
  - `services/bridge/src/sanction_handler.rs` contains a function body that calls `state.http_client.get(...).bearer_auth(&state.config.as_token)...` for `m.room.power_levels` AND a matching `put` call
  - `services/bridge/src/sanction_handler.rs::sanction_kind_to_power_level` returns a `hide_content` branch whose reason contains `redaction_not_available_in_m2_late_2`

### Story 3: End-to-end test + pilot verification

- **Composing tasks:** Task 5, Task 6
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full ..."` (the §15.4 line)
- **Expected output:** `E2E_EXIT_0` (new m2_late test passes; pre-existing m2_late tests still pass)
- **Brief-Scope outputs to verify:**
  - `crates/server/tests/e2e/m2_late.rs` contains a test function with `cr_a_atomicity` in its name that asserts both `sanction_event` row + `governance_log` row present under subscriber-failure
  - `services/bridge/tests/sanction_enforcement.rs` exists + has at least one `#[ignore]`-gated `#[tokio::test]`
  - The verify report records pilot gates A, B, C passed (D optional)

> **Verification mapping:** `/brehon-verify` iterates these stories, runs each checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms trigger catch-fire.

## 17. Completion checklist

- [ ] Task 0 audit complete (probes 0–12 confirmed).
- [ ] Tasks 1–5 committed (one commit each).
- [ ] §15 validation green at every gate.
- [ ] §16a stories all `[done]`.
- [ ] Task 6 pilot gates A, B, C passed (recorded in verify report, not codebase).
- [ ] Retro committed (Task 7).
- [ ] PR opened by BM session against `governance-v0`.
- [ ] CodeRabbit review complete + findings triaged.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/m2-late-2-verify.md` shows all stories ✓.

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `governance_log::append` signature change required to accept a transactional `&mut AsyncPgConnection` | LOW | HIGH | The federation_outbox.rs:140-175 pattern shows the reborrow works; if compile fails, STOP-and-ask (per §12). |
| Bridge handler hits a different auth header / wrong matrix URL | MED | MED | T4 mirrors `provision.rs:23` and `room_provisioner.rs:560` exactly; T5 test catches it without docker. |
| `hide_content` overclaim — the response reason lies about redaction | LOW | MED | The reason string in §10.3 is explicit; T5 e2e asserts the reason contains `redaction_not_available_in_m2_late_2`. |
| Pilot `BRIDGE_SANCTION_CALLBACK_URL` not set or bridge not running | MED | LOW | T6 surfaces to the user before T7; do not silently pass T6. |
| Bridge `Cargo.toml` change in T5 trips the linux compile gate | LOW | LOW | R10 + the lesson are explicit; raising the DQ is the correct response, not a bug. |
| T2 conn-reborrow fails to compile (signatures drift) | LOW | HIGH | §12 STOP-and-ask; do not silently widen `governance_log::append`. |
| Pilot e2e under T5 hits cycle-count §5.3 HARD REFUSAL on `e2e.rs` edit | MED | MED | Pre-locate unique `include!` anchor; T5 isolated; `feedback_fix_impl_pre_locate_e2e_anchors.md`. |

## 19. Notes

**No migration in m2-late-2.** `case_id` lives in the webhook DTO only. The `sanction_event` table is recoverable from `sanction_id → sanction.case_id` (FK). Adding a `case_id` column to `sanction_event` is deferred — it would simplify some audit queries but is not required for the CR-A fix or the bridge enforcement.

**`hide_content` reason string is load-bearing.** It is the only thing in the audit trail that tells future readers the bridge applied a posting block but did NOT redact past messages. Changing the string in a follow-up phase (e.g. "now redacts via subject→event index") is fine, but the literal `redaction_not_available_in_m2_late_2` must remain until that follow-up ships. Future retrofit work should grep the audit logs for this string to find historical partial applications.

**Bridge has no matrix-sdk use in T4.** Power-level enforcement uses raw `reqwest` against the client-server API, mirroring the existing `provision.rs` and `room_provisioner.rs` patterns. The `matrix-sdk = "0.18"` dep in `services/bridge/Cargo.toml` is used by `puppet.rs` for registration; T4 calls into `PuppetMap::ensure_puppet` (the public API) and does not add new matrix-sdk surface. R9 zero-Matrix-deps-in-workspace invariant preserved.

**m2-late-1's `SanctionEventPayload` field order was alphabetical. m2-late-2 adds `case_id`.** The brief §10.5 discussion noted the option of grouping by related concept (between `effective_until` and `governance_log_entry_hash`) vs strict alphabetical. Plan picks the grouping choice (`case_id` follows the time fields) because the time/correlation fields read as a unit; alphabetical ordering is for the registry (where `case_id` would sort to the top — but the registry is for `entry_kind`, not for payload field order).

**T6 is an operator gate, not a code task.** It is the only task without a commit. The verify report records the gate outcomes; the codebase is unchanged. If Gate A, B, or C fails, surface to the user before T7 (a retro without a successful pilot is misleading).

**T4 is the most invasive bridge edit.** The handler body changes from a 3-line stub to a 30+ line enforcement loop. The risk is highest in URL percent-encoding and in the per-room error handling (one room failure must not abort the loop). T5's bridge test catches the no-rooms-for-case path; the multi-room-with-failure path is left as a `todo!` in the test for a future phase.

**B-actor out-of-scope tripwire.** Any impl task that creates an `actor_app_link` table, a sign-claim token format, or an OAuth redirect is a §12 STOP-and-ask tripwire. User-confirmed 2026-06-07 that B-actor is deferred.

## 20. Confidence score

- **Plan correctness:** 8/10 — anchors verified live (sanction_publisher.rs:178-203 + :196, sanction_handler.rs:122-125, bridge_room.rs:13-29, federation_outbox.rs:55-175, governance_log.rs:280-339, submit_jury_vote.rs:198). Residual risk: T4 URL percent-encoding + per-room error handling is a 30-line code body that is easier to write than the stub; the bridge test exercises only the empty-rooms path.
- **Cargo budget:** 9/10 — ~6 GB peak, no new workspace deps. Bridge stays in `exclude`.
- **Test coverage:** 7/10 — workspace e2e covers CR-A success + failure paths; bridge unit test covers the no-rooms path; multi-room-with-failure path is `todo!` deferred.
- **Pilot verification:** 8/10 — three concrete gates (env var, route reachable, row present); smoke-POST is optional. Cannot fully exercise end-to-end without a real pilot sanction.
