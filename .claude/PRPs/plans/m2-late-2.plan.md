# Plan: m2-late-2 — bridge power-level enforcement + atomic sanction publish + pilot verification

## 1. Summary

m2-late-2 closes the two deliberate stubs carried forward from m2-late-1 and adds one operational gate:

1. **Atomic sanction publish (CR-A):** `enqueue_sanction_event` writes the `sanction_event` row and the `sanction_published` / `sanction_event_delivery_failed` governance-log entry on **two separate pool connections** with no transaction. This plan makes those two writes atomic in a single fresh `run_transaction` inside `enqueue_sanction_event` — a `sanction_event` row never exists without its matching audit-log entry, and a signing failure rolls both back. The vote transaction (R8) is never touched.
2. **Matrix power-level enforcement:** `services/bridge/src/sanction_handler.rs` authenticates, ACKs, and returns `applied: false`. This plan makes the bridge resolve the case's provisioned rooms (new `case_id` payload field + `bridge_room::lookup_by_case`), resolve the subject's puppet MXID, and write a room-relative `m.room.power_levels` override per `SanctionKind`, returning `applied: true` with `{rooms_found, rooms_applied, rooms_failed}` counts.
3. **Pilot verification (T6):** confirm the pilot has `BRIDGE_SANCTION_CALLBACK_URL` set, exactly one active `sanction_subscriber` row, and a reachable bridge callback — recording presence/absence + redacted host/path only.

**Headline acceptance:** the existing `sanction_event_delivered_to_subscriber` e2e passes with a `case_id` assertion added; the bridge handler returns `applied: true` after a GET+PUT power-level write against a provisioned room; CR-A is closed (single transaction). **B-actor portable-ID linkage is out of scope** (user-confirmed 2026-06-07; OQ-ADR016-03 deferred).

## 2. Source

- `.claude/PRPs/briefs/m2-late-2-planning-1.md` — this plan's brief; §6 user-confirmed decisions D1/D2/D3/D3-append (clarify-DQ `a3d0e9941441-058` … `-061`).
- `.claude/PRPs/handovers/m2-late-2-bootstrap.md` — RESUME block, §4 watchlist, §7 catch-fire triggers (R8/R9).
- `.claude/PRPs/reports/m2-late-1-retro.md` — carry-forwards (CR-A, power-level stub, pilot verification); retro change #3 (reqwest timeout default).
- ADR-016 (cross-app governance backplane; B-publish webhook publish/subscribe) — apps retain sovereignty.
- ADR-015 (pseudonymity) — subject identifier is `actor_pseudonym.pseudonym` only.
- ADR-008 (every governance write commits with a signed audit-log entry; INSERT+signature atomic) — the invariant CR-A restores for the sanction-event path.
- Lessons binding decisions: `feedback_multi_write_handlers_need_transactions.md` (CR-A IS this pattern), `feedback_lemmy_error_no_std_error.md` (LemmyResult returns), `feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md` (cargo scope), `feedback_linux_compile_proof_is_a_gate.md` (bridge Cargo.lock gate), `feedback_async_pool_test_pattern.md` + `feedback_fix_impl_pre_locate_e2e_anchors.md` (e2e edit discipline), `feedback_validate_pending_laptop_write_then_stop.md` (validation delegation).

## 3. Problem statement

- **P1 (→ Pattern 10.1, Task 2):** `sanction_publisher.rs:178-207` inserts the `sanction_event` row on one `get_conn` checkout (lines 178-193), then calls `governance_log::append(&mut ctx.pool(), ...)` (lines 196-207) which checks out a **second** connection. A crash or signing failure between the two leaves a `sanction_event` row with no audit-log entry — an ADR-008 violation. CR-A flagged this in m2-late-1; R8 (fire-and-forget outside the vote tx) blocked the fix then.
- **P2 (→ Pattern 10.2/10.3, Tasks 3+4):** the bridge cannot act on a sanction. `handle_sanction_event` returns `applied: false` because (a) the payload carries no `case_id`, so the bridge cannot find the case's rooms (`bridge_room` is keyed by `(case_id, room_type)`); and (b) there is no power-levels GET/PUT helper.
- **P3 (→ Story 5, Task 6):** m2-late-1 wired `seed_sanction_subscriber` at startup but never verified it fires in the pilot. A "delivery succeeded but nobody is subscribed" false-green is possible until the pilot's `sanction_subscriber` row is confirmed.

## 4. Solution statement

Two independent edit surfaces — workspace (`crates/`) and the workspace-excluded bridge (`services/bridge/`):

**Workspace (lemmy_api + lemmy_server e2e):**
- Add `case_id: i64` to `SanctionEventPayload`, populated via `i64::from(sanction.case_id.0)` (R1 — never `as`). The payload is the only carrier; **no migration, `sanction_event` table unchanged** (D1: persistence optional, not needed for audit/e2e).
- Wrap the `sanction_event` INSERT + `governance_log::append` in one fresh `conn.run_transaction(async |conn| { … })`, calling `append` via the `&mut (&mut *conn).into()` reborrow so diesel-async promotes `append`'s inner `run_transaction` to a SAVEPOINT (D3-append; exemplar `admin_assign_jury.rs:218`). The transaction conn is a **fresh `get_conn`**, opened AFTER all subscriber POSTs — never the vote tx conn (R8).

**Bridge (services/bridge):**
- Mirror `case_id: i64` onto the bridge's local `SanctionEventPayload`.
- Add `bridge_room::lookup_by_case(conn, case_id) -> Vec<(room_type, matrix_room_id)>`.
- Rewrite `handle_sanction_event`: auth → lookup rooms by `case_id` → (empty ⇒ `applied:false`, no Matrix call) → resolve subject MXID via `puppet_map.ensure_puppet` → for each room GET `m.room.power_levels`, compute a **room-relative** override per `SanctionKind`, merge `users[mxid]`, PUT back → return `applied = rooms_applied > 0` with counts. Best-effort across rooms; one room's failure does not abort the others. **Power-levels only — no membership ban, no historical redaction** (D2).

`/brehon-verify` predictability: §11 follows directly from §4 — two workspace files, two bridge source files, one bridge test surface.

## 5. Metadata

- **Phase:** `m2-late-2`
- **Branch:** `phase-m2-late-2` (cut by BM-task before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 8 (Task 0 pre-flight + T1–T6 + retro)
- **Estimated cargo budget:** ~6 GB peak (workspace `--features full` check + e2e compile; bridge check is small, ~1 GB)
- **Forbidden-window applicability:** standard (Shape G SUSPENDED — cargo runs on the laptop advisor per `advisor-orchestrator.md` §5.2)
- **Complexity score:** 6/10 — see breakdown

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 5 code tasks (T1–T5); T6 is a manual operator gate, not a Junior impl-task |
| Migrations touched | +2 each | 0 | D1: no migration; `case_id` is payload-only |
| Crates touched | +1 each | 3 | `lemmy_api`, `lemmy_server` (e2e), `services/bridge` |
| `crates/server/tests/e2e/*.rs` edits | +3 each | 3 | 1 file (`m2_late.rs`, ~387 lines) — mechanical worker-hang weight; **actual hang risk LOW** (split module, not the 18k monolith; single assertion add) |
| New ADR-affecting decisions | +2 each | 0 | `case_id` extension is backward-compatible within ADR-016; D2 enforcement is within ADR-016 |
| Cargo budget peak above 6 GB | +1 per GB | 0 | ~6 GB, not above |
| **Total** | — | **6** | Threshold for split-DQ (Sonnet): `>8`. **6 < 8 → no split; proceed as one phase.** |

### 5.2 Per-task complexity ceiling (Sonnet ≤4 files / ≤2 crates)

All §13 tasks satisfy the Sonnet ceiling:

- T1: `sanction_publisher.rs` + `m2_late.rs` = 2 files / 2 crates (`lemmy_api` + `lemmy_server`). ✓
- T2: `sanction_publisher.rs` = 1 file / 1 crate. ✓
- T3: `sanction_handler.rs` + `bridge_room.rs` = 2 files / 1 crate (`services/bridge`). ✓
- T4: `sanction_handler.rs` = 1 file / 1 crate. ✓
- T5: `sanction_handler.rs` (in-module `#[cfg(test)] mod`) = 1 file / 1 crate. ✓

T1 spans 2 crates deliberately (payload field + its e2e assertion are one logical unit and must compile/pass together). It stays at the ≤2-crate ceiling.

## 6. Relationship to other m2 sub-phases

Follows `m2-late-1` (B-publish sanction propagation, PR #192, merge `be8134d0b`). Depends on m2-late-1's `sanction_event` / `sanction_subscriber` tables, `sanction_publisher.rs`, `sanction_kind_map.rs`, the bridge `/brehon/sanction-event` route, and m2-rooms-a's `bridge_room` table. **Followed by** a future B-actor phase (portable-ID linkage, OQ-ADR016-03) and a future `hide_content` redaction phase (needs a subject→event index) — both out of scope here.

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 ↔ i64` conversion/comparison uses `i64::from(...)`, never `as` cast (per `feedback_clippy_test_style.md`). Applies to `case_id: i64 = i64::from(sanction.case_id.0)` in Task 1.
- **R8 (CRITICAL):** `enqueue_sanction_event` runs OUTSIDE any vote transaction. The CR-A fix opens a **fresh `get_conn`** for its `run_transaction`, AFTER the subscriber POSTs — it never reuses the vote tx conn. Catch-fire if Task 2's transaction conn comes from anywhere but a fresh `ctx.pool()` / `get_conn`.
- **R9 (CRITICAL):** `services/bridge` stays in root `Cargo.toml` `exclude`. NEVER `--workspace` for bridge cargo; use `cd services/bridge && cargo <verb>` (no `--features full` — bridge has no such feature). Catch-fire if any bridge validation uses `--workspace`.
- **R-multi-write:** the CR-A atomicity fix IS the `feedback_multi_write_handlers_need_transactions.md` pattern — both writes in one `run_transaction`, the second via the `&mut (&mut *conn).into()` reborrow.
- **R-timeout:** any outbound HTTP from a governance/bridge path uses connect+read timeouts. `sanction_publisher.rs:138-141` already sets `connect_timeout(10s)`/`timeout(30s)`. Task 4's bridge GET/PUT reuse `state.http_client` — confirm it was built with timeouts at AppState construction; if it is a bare `reqwest::Client::new()`, add timeouts there (per m2-late-1 retro change #3).
- **R-bridge-lock:** if Task 3/4/5 add ANY dependency to `services/bridge/Cargo.toml`, `services/bridge/Cargo.lock` changes → raise a `validate-pending-laptop-linux` DQ (Option-2 trigger). **The recommended approach adds NO new bridge dependency** (reuses `reqwest`/`serde_json`/`tokio`/`rusqlite`), so no Linux gate fires.

## 8. Flow design

**Before (m2-late-1):**

```
submit_jury_vote ──spawn──▶ enqueue_sanction_event
                              ├─ POST → subscriber (bridge)
                              ├─ get_conn #1: INSERT sanction_event      ┐ NOT atomic
                              └─ append(&mut pool): get_conn #2 + sign   ┘ (CR-A gap)

bridge /brehon/sanction-event ─▶ handle_sanction_event
                                  ├─ Bearer auth
                                  ├─ log event
                                  └─ return applied:false  (stub)
```

**After (m2-late-2):**

```
enqueue_sanction_event
  ├─ build payload { …, case_id: i64::from(sanction.case_id.0) }   (Task 1)
  ├─ POST → subscriber(s)
  └─ get_conn(fresh).run_transaction(async |conn| {               (Task 2)
        INSERT sanction_event(conn);
        governance_log::append(&mut (&mut *conn).into(), kind, …); // SAVEPOINT
        Ok(())
     })

bridge /brehon/sanction-event ─▶ handle_sanction_event             (Task 4)
  ├─ Bearer auth → 401 if bad (before any side-effect)
  ├─ rooms = bridge_room::lookup_by_case(conn, payload.case_id)    (Task 3)
  ├─ rooms.is_empty() ⇒ 200 applied:false, reason="no rooms…"  (no Matrix, no puppet)
  ├─ mxid = puppet_map.ensure_puppet(subject_actor_pseudonym)
  └─ for (room_type, room_id) in rooms:                            (best-effort)
        content = GET …/state/m.room.power_levels
        (level, reason_code) = compute_power_override(sanction_kind, &content)
        content.users[mxid] = level
        PUT …/state/m.room.power_levels  (content)
     ⇒ 200 applied:(rooms_applied>0), reason="… rooms_found/applied/failed; <reason_code>"
```

## 9. Mandatory reading

Impl-task subagent MUST Read before its first edit:

- **CR-A pattern (Task 2):**
  - `crates/db_schema/src/source/governance/governance_log.rs:277-336` — `append` signature + the SAVEPOINT doc-comment (lines 294-302: reborrow promotes inner `run_transaction` to a savepoint).
  - `crates/api/api/src/governance/admin_assign_jury.rs:204-231` — **canonical exemplar** of `append` called inside a caller's transaction.
  - `crates/api/api/src/governance/sanction_publisher.rs:1-233` — current (non-atomic) writes at 178-207; payload struct at 36-46; build site at 130-136.
- **Bridge enforcement (Tasks 3+4+5):**
  - `services/bridge/src/sanction_handler.rs` (whole file) — current stub.
  - `services/bridge/src/bridge_room.rs:19-29` — `lookup` shape to mirror for `lookup_by_case`.
  - `services/bridge/src/appservice.rs:33-41` — `AppState` (`config`, `puppet_map`, `http_client`, `bridge_db_path`).
  - `services/bridge/src/config.rs:11,13,26` — `tuwunel_url`, `as_token`, `bridge_callback_secret`.
  - `services/bridge/src/puppet.rs:58` — `async fn ensure_puppet(&self, brehon_user: &str) -> Result<MatrixUserId>` (MXID = String).
  - `services/bridge/src/room_provisioner.rs:527-581` — existing authenticated Matrix call pattern: `state.http_client.get(&url).bearer_auth(&state.config.as_token)`, URL-encoded room id, `bridge_room::open(&state.bridge_db_path)` per-request.
- **e2e (Task 1):**
  - `crates/server/tests/e2e/m2_late.rs:316-385` — payload assertions to extend; `case_id` already captured at line 262.
- **Lessons:** `feedback_multi_write_handlers_need_transactions.md`, `feedback_lemmy_error_no_std_error.md`, `feedback_features_full_workspace_only.md`, `feedback_features_full_p_crate_incompatible.md`, `feedback_linux_compile_proof_is_a_gate.md`, `feedback_async_pool_test_pattern.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`.

## 10. Patterns to mirror

### 10.1 Atomic two-write transaction with `append` reborrow

**Mirror:** `crates/api/api/src/governance/admin_assign_jury.rs:218-231` (call shape) + `crates/db_schema/src/source/governance/governance_log.rs:308-335` (inner SAVEPOINT).

```rust
// Task 2 target shape inside enqueue_sanction_event (replaces lines 178-207):
let mut pool = ctx.pool();
let conn = &mut get_conn(&mut pool).await?;          // FRESH conn — NOT the vote tx (R8)
conn
  .run_transaction(async |conn| {
    let event_form = SanctionEventInsertForm {
      sanction_id: sanction.id,
      sanction_kind: payload.sanction_kind,
      subject_actor_pseudonym: subject.clone(),
      effective_from: sanction.starts_at,
      effective_until: sanction.ends_at,
      governance_log_entry_hash: payload.governance_log_entry_hash.clone(),
    };
    insert_into(sanction_event_dsl::table)
      .values(&event_form)
      .execute(conn)
      .await?;

    // Reborrow: `append`'s own run_transaction becomes a SAVEPOINT of THIS tx.
    governance_log::append(
      &mut (&mut *conn).into(),
      kind,
      serde_json::json!({
        "sanction_id": sanction.id.0,
        "sanction_kind": payload.sanction_kind,
        "subscriber_count": subscribers.len(),
        "subject_actor_pseudonym": subject,
      }),
      Some(subject.clone()),
    )
    .await?;

    Ok(())
  })
  .await?;
```

**GOTCHA:** the insert uses `conn` first, then `append` needs it again → the reborrow `(&mut *conn)` is mandatory (a bare `conn.into()` would move the closure's `conn`). `subject` is moved into the `json!` and into `Some(subject)` inside the closure — clone as shown so the outer `Ok(())` path and any later use stay valid. If this does not type-check, **STOP and file a DQ** — do NOT widen `governance_log::append`'s public signature (D3).

### 10.2 bridge_room lookup-all-by-case

**Mirror:** `services/bridge/src/bridge_room.rs:19-29` (`lookup`).

```rust
/// All provisioned rooms for a case: (room_type, matrix_room_id).
/// Rows with a NULL matrix_room_id (not yet provisioned) are skipped.
pub fn lookup_by_case(conn: &Connection, case_id: i64) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT room_type, matrix_room_id FROM bridge_room \
         WHERE case_id = ?1 AND matrix_room_id IS NOT NULL"
    )?;
    let rows = stmt.query_map(params![case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect()
}
```

### 10.3 Authenticated Matrix power-levels GET/PUT (room-relative override)

**Mirror:** `services/bridge/src/room_provisioner.rs:527-581` (bearer_auth + URL-encoded room id + `state.http_client`).

```rust
// m.room.power_levels state event: GET returns the content object directly.
//   GET  {tuwunel_url}/_matrix/client/v3/rooms/{encoded_room_id}/state/m.room.power_levels
//   PUT  same URL, body = the FULL merged content (PUT replaces, so merge first).

/// Compute the room-relative power override per sanction_kind (D2).
/// `content` is the fetched m.room.power_levels object.
/// Returns (target_level, reason_code).
fn compute_power_override(sanction_kind: &str, content: &serde_json::Value) -> (i64, &'static str) {
    let events_default = content.get("events_default").and_then(|v| v.as_i64()).unwrap_or(0);
    let msg_threshold = content
        .get("events").and_then(|e| e.get("m.room.message")).and_then(|v| v.as_i64())
        .unwrap_or(events_default);
    let voice_threshold = content
        .get("events")
        .and_then(|e| e.get("m.call.member").or_else(|| e.get("org.matrix.msc3401.call.member")))
        .and_then(|v| v.as_i64());
    match sanction_kind {
        // Silence on the message channel (one below the send threshold).
        "ban" | "mute" | "prevent_post" => (msg_threshold - 1, "power_level_reduced_below_post_threshold"),
        // Voice channel if a voice threshold exists, else conservative fallback.
        "mute_voice" => (voice_threshold.unwrap_or(events_default) - 1, "voice_power_reduced_fallback_post_threshold"),
        // No native primitive: reduce posting, flag that redaction/reach is not enforced.
        "hide_content" => (msg_threshold - 1, "redaction_not_available_in_m2_late_2"),
        "restrict_reach" => (msg_threshold - 1, "restrict_reach_translated_to_power_level_reduction"),
        _ => (msg_threshold - 1, "unknown_sanction_kind_default_post_reduction"),
    }
}
```

**GOTCHA:** PUT to `/state/{type}/{key}` **replaces the entire content** — GET first, set `content["users"][mxid] = level`, PUT the whole merged object. Do NOT PUT a `{"users": {mxid: level}}` fragment (it would wipe `events_default`, `ban`, etc). Power level `0` is the *default member* level in most rooms — never use a bare `0` to mute; always compute `threshold - 1` from the fetched state (this is why the m2-late-1 static `sanction_kind_to_power_level` is replaced, not reused).

### 10.4 Handler ordering for side-effect-free empty path

**Mirror:** auth-first shape in `services/bridge/src/sanction_handler.rs:90-110`.

Order in `handle_sanction_event`: (1) Bearer auth → 401; (2) `bridge_room::open` + `lookup_by_case`; (3) `rooms.is_empty()` ⇒ `200 applied:false` **before** `ensure_puppet` (so the no-rooms path makes zero Matrix calls — load-bearing for the dep-free unit test); (4) `ensure_puppet`; (5) per-room GET/compute/merge/PUT with `rooms_applied`/`rooms_failed` counters. A `bridge_room::open` error is a genuine infra fault → `500`; an empty result set is `200 applied:false` (T3 acceptance: not a 500).

## 11. Files to change

**`lemmy_api` (`crates/api/api/`):**
- `src/governance/sanction_publisher.rs` — add `case_id: i64` to `SanctionEventPayload` + populate it (Task 1); wrap INSERT + `append` in one `run_transaction` (Task 2).

**`lemmy_server` (`crates/server/`):**
- `tests/e2e/m2_late.rs` — assert `case_id` present + equal to the test's `case_id` in the delivered payload (Task 1).

**`services/bridge` (workspace-excluded, R9):**
- `src/sanction_handler.rs` — add `case_id: i64` to the local `SanctionEventPayload` mirror (Task 3); rewrite `handle_sanction_event` + add `compute_power_override` + GET/PUT helpers; remove the now-superseded `sanction_kind_to_power_level` (Task 4); add `#[cfg(test)] mod tests` (Task 5).
- `src/bridge_room.rs` — add `lookup_by_case` (Task 3).

**No migration. No `sanction_event.rs`/`schema.rs` change. No new bridge dependency.**

### Struct-field add: enumerate all callsites

`SanctionEventPayload` (workspace, `sanction_publisher.rs:38`) is constructed at exactly one site (`sanction_publisher.rs:130-136`) — `rg "SanctionEventPayload" crates/` confirms no other constructor. The bridge mirror (`sanction_handler.rs:58`) is `#[derive(Deserialize)]`-only (never constructed in source except in Task 5 tests). No cross-crate callsite fan-out. Adding `case_id` is a one-site change per struct.

## 12. NOT building in m2-late-2

- **`case_id` persistence in the `sanction_event` table** — deferred; reason: D1 confirms persistence is optional and no audit/e2e query needs it. Avoids a migration + the migration Linux gate.
- **B-actor portable-ID linkage / `actor_app_link` / OAuth link flow** — deferred to a future B-actor phase; reason: user-confirmed out of scope 2026-06-07, OQ-ADR016-03 parked.
- **Actual Matrix room ban (`m.room.member` membership=ban) for `SanctionKind::Ban`** — deferred; reason: D2 scopes m2-late-2 to power-level enforcement only. `ban` reduces posting power; the reason_code records it was a power reduction, not a membership ban. (Avoids the Matrix-semantics-overclaim risk.)
- **Historical-message redaction for `hide_content`** — deferred to a future phase; reason: D2 — needs a subject→event index or room-history scan. Reason_code `redaction_not_available_in_m2_late_2` makes the limit explicit.
- **A pseudonym→rooms index** — not built; the `case_id` payload field + `lookup_by_case` replaces the need (D1).
- **A new Brehon HTTP lookup route** (bridge → Brehon by `governance_log_entry_hash`) — rejected in D1; `case_id` in the payload is sufficient.

---

## 13. Step-by-step tasks

Execute in dependency order. One commit per task.

> **Validation runner:** Shape G is SUSPENDED for m2-late-2. Each impl-task writes a `kind: "validate-pending-laptop"` DQ entry with its `commands`, commits + pushes, then **stops** — the laptop advisor runs cargo (per `feedback_validate_pending_laptop_write_then_stop.md`). Workspace commands use the Windows bat wrappers; bridge commands use plain `cargo` from `services/bridge` (R9 — never `--workspace`, no `--features full`).

### Task 0: Pre-flight harness audit + branch verification

**Goal:** confirm branch is `phase-m2-late-2`; confirm m2-late-1 deliverables intact on base; confirm wrapper + bridge-cargo sanity.

**Probes (R5 — enumerate all):**

```bash
# Probe 0 — Docker daemon (e2e uses testcontainers Postgres)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — branch
git branch --show-current   # EXPECT: phase-m2-late-2

# Probe 2 — m2-late-1 base intact
rg -n "applied: false" services/bridge/src/sanction_handler.rs   # EXPECT: present (stub to replace)
rg -n "enqueue_sanction_event" crates/api/api/src/governance/sanction_publisher.rs   # EXPECT: present

# Probe 3 — workspace wrapper honors --features full
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m2-late-2-task0-ws.log 2>&1"
echo "exit: $?"   # EXPECT: 0 (clean base)
tail -20 .claude/PRPs/debug/m2-late-2-task0-ws.log

# Probe 4 — bridge cargo sanity (R9: from crate dir, NO --workspace)
( cd services/bridge && cargo check > /tmp/m2-late-2-task0-bridge.log 2>&1 ); echo "exit: $?"   # EXPECT: 0
tail -20 /tmp/m2-late-2-task0-bridge.log

# Probe 5 — negative: bogus feature propagates non-zero (exit-code masking guard)
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m2-late-2-task0-neg.log 2>&1"
echo "neg exit: $?"   # EXPECT: NON-ZERO

# Probe 6 — concurrent-PR check on the four target files
gh pr list --repo barrie-cork/lemmy --state open --json number,title,headRefName,files \
  --jq '.[] | select(.files[]?.path | test("sanction_publisher|sanction_handler|bridge_room|e2e/m2_late")) | {number, title, headRefName}'
# EXPECT: empty
```

**No commit at Task 0.**

### Task 1 [P]: `case_id` payload field + e2e assertion

**ACTION:** add `case_id: i64` to the workspace `SanctionEventPayload`, populate it, and assert it in the m2-late e2e.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/sanction_publisher.rs   # add case_id field (struct + build site)
  - crates/server/tests/e2e/m2_late.rs                    # assert case_id in delivered payload
requires: []
```

**IMPLEMENT (file 1 of 2):** in `sanction_publisher.rs`, add `pub case_id: i64,` to `SanctionEventPayload` (after `sanction_kind`, line ~39). In the build site (lines 130-136) add `case_id: i64::from(sanction.case_id.0),` (R1 — never `as`).

**IMPLEMENT (file 2 of 2):** in `m2_late.rs`, after the `governance_log_entry_hash` assertion (line ~359), add:
```rust
let payload_case_id = payload.get("case_id").and_then(|v| v.as_i64())
  .expect("payload missing case_id");
assert_eq!(payload_case_id, i64::from(case_id.0), "payload case_id matches the case");
```
(`case_id` is the `ModerationCaseId` captured at line 262; use `.0` + `i64::from`.)

**MIRROR:** `sanction_publisher.rs:38-46` (struct) + `:130-136` (build); `m2_late.rs:354-359` (assertion shape).

**GOTCHA:** the e2e captures `case_id` as a `ModerationCaseId` newtype — convert with `i64::from(case_id.0)`, not `case_id as i64`. Only the one `.expect` assertion is added; do not restructure the surrounding block (anchor uniqueness — the `governance_log_entry_hash` assertion is a unique anchor).

**VALIDATE:** write a `validate-pending-laptop` DQ entry:
```
commands: ["cmd //c \"scripts\\brehon\\cargo-check.bat --workspace --features full\"",
           "cmd //c \"scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\"",
           "cmd //c \"scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server --features full sanction_event_delivered_to_subscriber\""]
e2e_filter: null
```
EXPECT: all exit 0; e2e test passes with the new `case_id` assertion.

### Task 2: CR-A atomicity fix

**ACTION:** wrap the `sanction_event` INSERT + `governance_log::append` in one fresh `run_transaction` inside `enqueue_sanction_event`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/sanction_publisher.rs   # lines 178-207 → single run_transaction
requires:
  - task: 1
    reason: Task 1 adds case_id to the payload built before this block; the build site must already carry case_id so the transaction body references the final payload shape.
```

**IMPLEMENT:** replace `sanction_publisher.rs:178-207` (the separate INSERT block + the `append(&mut ctx.pool(), …)` call) with the single-`run_transaction` shape in Pattern 10.1. The transaction conn is a **fresh `get_conn(&mut pool)`** opened here (R8 — not the vote tx). `append` is called via `&mut (&mut *conn).into()`.

**MIRROR:** Pattern 10.1; `admin_assign_jury.rs:218-231`; `governance_log.rs:308-335`.

**GOTCHA:** `append` returns `LemmyResult<GovernanceLog>`; inside the closure use `…await?;` and discard the value, then `Ok(())`. The closure must return `LemmyResult<()>`. If the reborrow does not type-check, STOP + file a DQ (D3 — do not widen `append`). Do NOT move the subscriber POST loop inside the transaction (it must stay before, so delivery latency is outside the tx).

**VALIDATE:** `validate-pending-laptop` DQ:
```
commands: ["cmd //c \"scripts\\brehon\\cargo-check.bat --workspace --features full\"",
           "cmd //c \"scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\"",
           "cmd //c \"scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server --features full sanction_event_delivered_to_subscriber\""]
```
EXPECT: exit 0; e2e still asserts `1 sanction_event row` + `1 sanction_published` (now written atomically).

### Task 3 [P]: bridge `case_id` field + `lookup_by_case`

**ACTION:** mirror `case_id` onto the bridge payload and add the all-rooms-by-case query.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/sanction_handler.rs   # add case_id: i64 to local SanctionEventPayload mirror
  - services/bridge/src/bridge_room.rs        # add lookup_by_case
requires: []
```

**IMPLEMENT (file 1 of 2):** in `sanction_handler.rs`, add `pub case_id: i64,` to the local `SanctionEventPayload` (after `sanction_kind`, line ~59). Field order/names must match the workspace struct's JSON keys.

**IMPLEMENT (file 2 of 2):** in `bridge_room.rs`, add `lookup_by_case` per Pattern 10.2.

**MIRROR:** `bridge_room.rs:19-29`; workspace `SanctionEventPayload` JSON keys from Task 1.

**GOTCHA:** the bridge struct has no `#[serde(deny_unknown_fields)]`, but adding `case_id` as a **required** field means a POST without it now fails axum's `Json` extractor with 400 before the handler runs. That is acceptable for the pilot (Brehon + bridge deploy together) — see §18 deploy-ordering risk. Do NOT add a serde default to mask a missing `case_id` (the bridge genuinely needs it to find rooms).

**VALIDATE:** `validate-pending-laptop` DQ (bridge — R9, no `--workspace`):
```
commands: ["bash -c 'cd services/bridge && cargo check'",
           "bash -c 'cd services/bridge && cargo clippy --no-deps -- -D warnings'"]
```
EXPECT: exit 0. (No `Cargo.toml`/`Cargo.lock` change → no `validate-pending-laptop-linux` gate.)

### Task 4: Matrix power-level enforcement

**ACTION:** rewrite `handle_sanction_event` to apply room-relative power-level overrides; add GET/PUT + `compute_power_override` helpers; remove the superseded static mapping.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/sanction_handler.rs   # handler rewrite + helpers; remove sanction_kind_to_power_level
requires:
  - task: 3
    reason: needs the case_id payload field and bridge_room::lookup_by_case from Task 3.
```

**IMPLEMENT:** per Pattern 10.3 + 10.4. New private helpers in `sanction_handler.rs`:
- `async fn get_power_levels(state: &AppState, room_id: &str) -> Result<serde_json::Value, ...>` — GET, return content object.
- `async fn put_power_levels(state: &AppState, room_id: &str, content: &serde_json::Value) -> Result<(), ...>` — PUT merged content.
- `fn compute_power_override(sanction_kind: &str, content: &serde_json::Value) -> (i64, &'static str)`.

Rewrite the handler body (lines 121-144) following the ordering in Pattern 10.4: lookup-by-case → empty-path early return (no Matrix) → `ensure_puppet` → per-room GET/merge `content["users"][mxid] = level`/PUT with `rooms_applied`/`rooms_failed` counters → `applied = rooms_applied > 0`. Build the response `reason` as `format!("rooms_found={rooms_found} rooms_applied={rooms_applied} rooms_failed={rooms_failed}; {reason_code}")`. Remove `sanction_kind_to_power_level` (lines 147-158) — it is fully superseded by `compute_power_override`.

**MIRROR:** `room_provisioner.rs:527-581` (bearer_auth + URL-encoded room id via `state.http_client`); `puppet.rs:58` (`ensure_puppet`); `bridge_room.rs` open-per-request pattern (`room_provisioner.rs:90-96`).

**GOTCHA:** (a) PUT replaces full content — always GET→merge→PUT (Pattern 10.3). (b) URL-encode the room id (contains `!`/`:`) — reuse the existing encode helper used in `room_provisioner.rs:531-533`. (c) Best-effort: a per-room GET/PUT error increments `rooms_failed` and continues — never `?`-propagate out of the loop. (d) 401 stays before any side-effect. (e) `bridge_room::open` error → 500; empty rooms → 200 `applied:false`. (f) Acting as the appservice (room creator → PL 100) is sufficient to write power-levels — no `?user_id=` impersonation.

**VALIDATE:** `validate-pending-laptop` DQ (bridge):
```
commands: ["bash -c 'cd services/bridge && cargo check'",
           "bash -c 'cd services/bridge && cargo clippy --no-deps -- -D warnings'"]
```
EXPECT: exit 0.

### Task 5: bridge handler tests

**ACTION:** add in-module `#[cfg(test)] mod tests` to `sanction_handler.rs` covering the dep-free paths; add an `#[ignore]` integration test for the live GET+PUT path.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/sanction_handler.rs   # #[cfg(test)] mod tests
requires:
  - task: 4
    reason: tests assert the enforced handler behaviour (auth/no-rooms/applied) shipped in Task 4.
```

**IMPLEMENT:** add `#[cfg(test)] mod tests` with `#[tokio::test]` cases:
1. **bad bearer → 401, no Matrix call.** Build `AppState` (temp `bridge_db_path`, `reqwest::Client::new()`, a `PuppetMap` — check its constructor in `puppet.rs`); call `handle_sanction_event` with a wrong/missing `Authorization`. Assert status 401. (Auth returns before lookup/puppet, so no Matrix is contacted.)
2. **no rooms for case → 200 `applied:false`, no Matrix call.** Open a temp bridge DB with the schema but no rows for `case_id`; valid Bearer. Assert 200 and `applied == false` and the reason mentions no rooms. (Per Pattern 10.4 ordering, `ensure_puppet` is never reached.)
3. **`compute_power_override` unit cases** — pure-function asserts for each `SanctionKind` against a sample `m.room.power_levels` content (e.g. `events_default=0`, `events["m.room.message"]=0`): `ban`/`mute`/`prevent_post` ⇒ `-1`; `mute_voice` with no voice threshold ⇒ `events_default-1`; `hide_content` ⇒ reason `redaction_not_available_in_m2_late_2`; `restrict_reach` ⇒ reason `restrict_reach_translated_…`.
4. **`#[ignore = "requires docker-compose stack"]` integration test** — one room provisioned for a case → POST sanction event → assert GET+PUT fired and `applied:true` (mirrors `services/bridge/tests/room_provisioning.rs` convention). This is the third T5 brief bullet, run manually against the stack.

**MIRROR:** `services/bridge/tests/room_provisioning.rs:12-20` (`#[tokio::test]` + `#[ignore]` convention); `feedback_async_pool_test_pattern.md`.

**GOTCHA:** **add NO new dev-dependency** — no `wiremock`/`mockito`/`httpmock` (would change `services/bridge/Cargo.lock` → trigger the Linux gate, R-bridge-lock). The dep-free tests (1–3) are the CI-runnable coverage; the live GET+PUT path (4) is `#[ignore]` per the existing bridge convention. For temp DB paths, reuse `tempfile` only if already a dev-dep; otherwise build a unique path under `std::env::temp_dir()` and remove it at test start. If `PuppetMap` cannot be constructed without a live Matrix client, restructure tests 1–2 so the auth-fail / no-rooms paths are exercised without constructing the puppet map (the no-rooms path returns before `ensure_puppet`).

**VALIDATE:** `validate-pending-laptop` DQ (bridge):
```
commands: ["bash -c 'cd services/bridge && cargo test'",
           "bash -c 'cd services/bridge && cargo clippy --no-deps -- -D warnings'"]
```
EXPECT: exit 0; tests 1–3 pass, test 4 skipped (`#[ignore]`).

### Task 6: Pilot verification (operator gate — no commit)

**ACTION:** confirm the pilot's sanction-delivery wiring. This is a manual advisor/operator step, **not a Junior impl-task** and produces **no code commit**.

Steps (record presence/absence + redacted host/path ONLY — never read `BRIDGE_CALLBACK_SECRET` or `.env` into chat):
- Confirm `BRIDGE_SANCTION_CALLBACK_URL` is set in the pilot environment (presence boolean + redacted host/path).
- `SELECT count(*) FROM sanction_subscriber WHERE active = true;` on the pilot DB — EXPECT exactly 1.
- `SELECT callback_url FROM sanction_subscriber WHERE active = true;` — confirm host/path matches the bridge callback (redact before recording).
- Reachability: a controlled probe to the bridge `/brehon/sanction-event` (e.g. a deliberately bad Bearer to confirm a 401 response without applying anything), OR a governance sanction in a pilot-safe path if state permits.

**Output:** a short note in the retro / runlog: `{callback_url_set: bool, active_subscriber_rows: N, bridge_reachable: bool, redacted_host: "<host/path>"}`. If `active_subscriber_rows != 1` or the callback is unreachable → STOP and surface (do not mark the phase shippable).

### Task 7: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + per-task metrics (`files/commits/runtime-min/max-log-silence-min`). Promote any new lessons (e.g. a `reqwest`-timeout-default lesson if retro change #3 is confirmed; a bridge-power-levels GET-merge-PUT lesson) in the same retro commit.

---

## 14. Testing strategy

- **Workspace compile:** `cargo check --workspace --features full` (Tasks 1, 2).
- **Workspace lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings`.
- **Workspace e2e:** `cargo test --test e2e -p lemmy_server --features full sanction_event_delivered_to_subscriber` — asserts `case_id` in payload + atomic `1 sanction_event` / `1 sanction_published`.
- **Bridge compile/lint (R9 — from `services/bridge`, no `--workspace`, no `--features full`):** `cargo check` / `cargo clippy --no-deps -- -D warnings` (Tasks 3, 4, 5).
- **Bridge tests:** `cargo test` (from `services/bridge`) — dep-free auth/no-rooms/`compute_power_override` cases pass; the live GET+PUT case is `#[ignore]` (run via docker-compose stack manually).
- **Pilot (Task 6):** manual SQL + reachability probe; no automated gate.

## 15. Validation commands (DoD)

> Dry-run each against `phase-m2-late-2` HEAD before plan approval. Bridge commands are R9-scoped (never `--workspace`).

### 15.1 Workspace static (Tasks 1, 2)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m2-late-2-ws-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.2 Workspace lint (Tasks 1, 2)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/m2-late-2-ws-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.3 Workspace e2e (Tasks 1, 2)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server --features full sanction_event_delivered_to_subscriber > .claude/PRPs/debug/m2-late-2-e2e.log 2>&1"
echo "exit: $?"   # EXPECT: 0  (1 passed)
```

### 15.4 Bridge static / lint / test (Tasks 3, 4, 5 — R9)

```bash
( cd services/bridge && cargo check )                          ; echo "check exit: $?"   # EXPECT: 0
( cd services/bridge && cargo clippy --no-deps -- -D warnings ); echo "clippy exit: $?"  # EXPECT: 0
( cd services/bridge && cargo test )                           ; echo "test exit: $?"    # EXPECT: 0 (ignored test skipped)
```

### 15.5 Cross-cutting verification

- [ ] `case_id` added to BOTH `SanctionEventPayload` structs (workspace + bridge); workspace populates via `i64::from` (R1).
- [ ] CR-A: `sanction_event` INSERT + `governance_log::append` are inside ONE `run_transaction` on a FRESH conn (R8); `rg -n "run_transaction" crates/api/api/src/governance/sanction_publisher.rs` returns exactly 1.
- [ ] `append` called via `&mut (&mut *conn).into()` reborrow; `governance_log::append` public signature UNCHANGED (`git diff governance-v0 -- crates/db_schema/src/source/governance/governance_log.rs crates/api/api/src/governance/governance_log.rs` is empty).
- [ ] No migration added (`git diff --name-only governance-v0 -- migrations/` empty).
- [ ] `services/bridge/Cargo.toml` + `Cargo.lock` UNCHANGED (no Linux gate). If changed → raise `validate-pending-laptop-linux`.
- [ ] No bridge cargo command used `--workspace` (R9).
- [ ] `sanction_kind_to_power_level` removed; `compute_power_override` is the only mapping.
- [ ] Power-level write is GET→merge→PUT (full content), not a fragment PUT.
- [ ] No B-actor / `actor_app_link` / pseudonym→account-linkage code (catch-fire if present).

## 16. Acceptance criteria

- [ ] Tasks 0–7 completed in dependency order
- [ ] §15.1/15.2 (workspace check + clippy) exit 0
- [ ] §15.3 (e2e) — `sanction_event_delivered_to_subscriber` passes with `case_id` assertion
- [ ] §15.4 (bridge check/clippy/test) exit 0
- [ ] §15.5 cross-cutting boxes all ticked
- [ ] §16a stories all `[done]`
- [ ] No edits outside §11 list
- [ ] Retro committed (Task 7)
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

## 16a. Stories

### Story 1: Sanction webhook carries the case id

- **Composing tasks:** Task 1 (`[P]`)
- **Checkpoint command:** `cmd //c "scripts\\brehon\\cargo-test.bat --test e2e -p lemmy_server --features full sanction_event_delivered_to_subscriber"`
- **Expected output:** `1 passed`
- **Brief-Scope outputs to verify:** `sanction_publisher.rs` `SanctionEventPayload` contains `case_id`; `m2_late.rs` asserts payload `case_id`.

### Story 2: Sanction publish is atomic (CR-A closed)

- **Composing tasks:** Task 2
- **Checkpoint command:** same e2e as Story 1 (asserts `1 sanction_event` + `1 sanction_published`)
- **Expected output:** `1 passed`
- **Brief-Scope outputs to verify:** `sanction_publisher.rs` has exactly one `run_transaction` wrapping the INSERT + `append`; `governance_log.rs` unchanged.

### Story 3: Bridge resolves a case's rooms

- **Composing tasks:** Task 3 (`[P]`)
- **Checkpoint command:** `cd services/bridge && cargo test no_rooms`
- **Expected output:** no-rooms test passes (`200 applied:false`, no Matrix call)
- **Brief-Scope outputs to verify:** `bridge_room.rs` exports `lookup_by_case`; `sanction_handler.rs` `SanctionEventPayload` has `case_id`.

### Story 4: Bridge applies a Matrix power-level override

- **Composing tasks:** Task 4, Task 5
- **Checkpoint command:** `cd services/bridge && cargo test` (dep-free cases) + `cargo test -- --ignored` (live GET+PUT, docker-compose)
- **Expected output:** auth-401 / no-rooms / `compute_power_override` cases pass; ignored one-room test applies `true` against the stack
- **Brief-Scope outputs to verify:** `sanction_handler.rs` `handle_sanction_event` returns `applied:true` with counts; `compute_power_override` present; `sanction_kind_to_power_level` removed.

### Story 5: Pilot delivery wiring verified

- **Composing tasks:** Task 6 (operator gate)
- **Checkpoint command:** manual — `SELECT count(*) FROM sanction_subscriber WHERE active = true;` on the pilot + a 401 reachability probe
- **Expected output:** exactly 1 active subscriber row; bridge reachable
- **Brief-Scope outputs to verify:** retro/runlog note records `{callback_url_set, active_subscriber_rows, bridge_reachable, redacted_host}` with no secrets.

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–6)
- [ ] Tasks 1–6 committed (Task 6 = operator note, no code commit)
- [ ] §15 validation green at every gate
- [ ] §16a stories all `[done]`
- [ ] Retro committed (Task 7)
- [ ] PR opened by BM against `governance-v0`
- [ ] CodeRabbit review triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/m2-late-2-verify.md` shows all stories ✓
- [ ] Post-merge phase branch retained for retro reads

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| CR-A reborrow `&mut (&mut *conn).into()` fails to type-check | LOW | MED | Pattern 10.1 is byte-aligned with the proven `admin_assign_jury.rs:218` + `governance_log.rs:308` path; if it fails, STOP + DQ (D3) — never widen `append` |
| Bridge `case_id` required → old POST without it returns 400 (deploy ordering) | LOW | LOW | Brehon + bridge deploy together in the pilot; deploy Brehon (sends `case_id`) before/with the bridge. If a rolling deploy is a concern, a follow-up can switch the bridge field to `Option<i64>` (None → `applied:false`, reason "case_id missing") — noted, not built |
| PUT power-levels wipes existing content (fragment PUT) | MED | HIGH | Pattern 10.3 GOTCHA: always GET→merge→PUT full content; §15.5 verifies |
| Matrix-semantics overclaim (`ban`/`hide_content` read as more than a PL change) | MED | MED | reason_codes make the limit explicit (`redaction_not_available_in_m2_late_2`, `restrict_reach_translated_…`); §12 NOT-building lists membership-ban + redaction as deferred |
| Bridge dev-dep added for tests → `Cargo.lock` change → unplanned Linux gate | LOW | LOW | Task 5 GOTCHA forbids new dev-deps; dep-free tests + `#[ignore]` live test |
| `PuppetMap` not constructible in a unit test | LOW | LOW | Task 5 GOTCHA: structure auth-401 / no-rooms tests so the puppet map is never reached (ordering in Pattern 10.4) |
| Pilot has 0 active subscribers (false-green) | MED | MED | Task 6 STOP condition: `active_subscriber_rows != 1` blocks ship |

## 19. Notes

- **D1/D2/D3 are user-confirmed** (brief §6, clarify-DQ `a3d0e9941441-058` … `-061`). No further clarify pass needed.
- **No migration, no new HTTP route, no new bridge dependency** — the smallest surface that closes both carry-forwards.
- The m2-late-1 static `sanction_kind_to_power_level` (absolute 0/25/50) is intentionally **removed**, not reused: absolute power levels ignore the room's actual `events_default` and would not mute a user in a room where `events_default == 0`. The room-relative `compute_power_override` is the correct enforcement.
- A `reqwest`-timeout-default lesson (m2-late-1 retro change #3) was never authored — not cited as a lesson; the timeout discipline is captured as guardrail R-timeout (§7) and should be promoted to a lesson at retro (Task 7) if confirmed.
- Cohort suggestion for the advisor (cap 2): **Cohort 1** = Task 1 `[P]` + Task 3 `[P]` (disjoint: workspace vs bridge); **Cohort 2** = Task 2 `[P]` + Task 4 `[P]` (disjoint files; each `requires:` its Cohort-1 predecessor); then Task 5 (requires Task 4); Task 6 manual.

## 20. Confidence score

- **Plan correctness:** 9/10 — CR-A path is proven (`admin_assign_jury` exemplar read); bridge enforcement shape verified against `room_provisioner.rs` call pattern + confirmed `AppState`/`puppet.rs`/`config.rs` fields.
- **Cargo budget:** 9/10 — workspace `--features full` ~6 GB; bridge small; no migration round-trip.
- **Test coverage:** 7/10 — dep-free bridge unit tests cover auth + no-rooms + `compute_power_override`; the live GET+PUT path is `#[ignore]` (docker-compose), matching the existing bridge convention rather than adding a mock-HTTP dependency.
