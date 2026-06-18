# Plan: m3-core-stage-mode — chair-controlled stage mode, FIFO mic-passing, first chair-action chain emission

## 1. Summary

This sub-phase gives the M3 town-hall bridge a **chair-controlled stage** on top of the deployable RTC stack m3-core-infra shipped. It delivers: a **dual-sourced chair seat** (governance-assigned initial chair persisted in `bridge_room.chair_id`, delegable mid-session via a Matrix-native transfer the bridge mirrors); a **persisted FIFO raised-hand queue** in `bridge_room.queue_state`; **mic-passing with a 30s grace** (promote a watcher → grant LiveKit publish for 30s; on no-activate **auto-revoke** + **promote the next** queued watcher); **chair override** (force-demote / force-promote out of FIFO order); a **Q&A text sidebar** (the town-hall Matrix room's text timeline); and the **FIRST emission** of the `room_chair_transferred` + `room_chair_override` hash-chain entries — wiring the bridge→binary room-event callback that no code exercises today. **Headline acceptance:** the bridge drives **4 mic-passes in sequence** with no manual intervention and the **30s no-activate boundary** auto-revokes + next-promotes, both as deterministic tests; chair transfer/override each emit their chain entry carrying **pseudonyms only** (ADR-015), **metadata only** (ADR-016). Mic-grant is a **LiveKit publish-grant** (re-mint), never Matrix power-levels (those are Phase-4 emergency-mute). No new migration; no new sidecar; bridge-side logic + one trivial binary DTO extension.

## 2. Source

- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 3: M3-core stage-mode" (235–238), §Technical Approach (174–203), §Success Criteria stage-mode rows (136–140), §Cross-Cutting Impact (155–159), §ADR table (48–69, esp. OQ-V2-05 @ 69), Decisions Log D1/D3/D7 @ `307747138`.
- `.claude/PRPs/briefs/m3-core-stage-mode-planning-1.md` (the authorising brief) @ `307747138`.
- `.claude/PRPs/handovers/m3-core-stage-mode-bootstrap.md` §1 (scope+DoD), §4 watchlist (6 items), §7 catch-fire, §"Stop-and-ask tripwires" @ `307747138`.
- `.claude/PRPs/plans/m3-core-infra.plan.md` — the SIBLING bridge plan (canonical-schema-first gate): §5 complexity breakdown, §7 R1–R8 guardrails, §10 patterns, §15 three-surface DoD, §16a story shape @ `307747138`.
- Clarify DQs resolved by advisor 2026-06-18: `a3d0e9941441-066` (build the bridge→binary callback client — option (a)); `a3d0e9941441-067` (mic-grant = LiveKit publish-grant re-mint, NOT Matrix power-levels).
- ADRs (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`): **ADR-015** (pseudonymised actor IDs; line 257) — load-bearing for every chair payload; **ADR-016** (cross-app backplane; content never hashed) — metadata-only emission; **ADR-011** (AGPLv3 inherited; line 199) — no new component this phase. OQ-V2-05 (chair dual-sourced, RESOLVED 2026-04-17).
- Lessons that bind decisions:
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); never Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — every bridge-touching task writes a `validate-pending-laptop-linux` DQ; `bm-pr` gates on `result:pass`.
  - `feedback_build_what_tests_exercise.md` — the marquee tests must EXERCISE the real flows (the 4-mic-pass FIFO sequence; the 30s no-activate boundary; the real chair-entry emission), not assert on config/struct shape.
  - `feedback_governance_type_state_handlers.md` — the chair-seat / mic-grant transitions are a type-state candidate (phantom-typed `Watcher`/`Promoted`/`Speaking`); applied in §10.6.
  - `pattern_test_against_reality_not_syntax.md` — assert on observable grant transitions + the actual emitted `RoomEventRequest` payload + (docker-gated) the real `governance_log` rows.
  - `feedback_plan_dod_dry_run_at_write.md` — every §15 command is written in the exact form the advisor runs at gate 1 (dry-run mentally before commit).

## 3. Problem statement

m3-core-infra made the RTC stack deployable + optional (`rtc_enabled`), gave the bridge LiveKit JWT minting (`livekit_jwt::mint_access_token`), and added the `bridge_room` RTC-state columns (`chair_id`/`queue_state`/`recording_config`). m3-core-entry-kinds registered the 3 chair/mute consts. But **nothing exercises any of it as a live town hall**:

- The bridge has no chair-seat logic — `bridge_room.chair_id` is an unread column (Task 3 + Task 6).
- There is no raised-hand queue — `bridge_room.queue_state` is an unread column (Task 3).
- There is no mic-passing and no 30s grace timer — the load-bearing stage behaviour (Tasks 3, 4).
- `mint_access_token` grants only `roomJoin` — it cannot express the presenter-vs-watcher distinction (`canPublish`) stage mode needs (Task 2).
- **There is no bridge→binary room-event callback client.** `config.rs:35-40` `brehon_room_event_url` is a `#[allow(dead_code)]` config surface only; `room_provisioner.rs:391,439,463` explicitly defer chain-emission. The binary seam (`handle_room_event` → `append_room_event`, route `/api/v4/governance/room-event`, Bearer) **exists and is mounted** — but no bridge code POSTs to it (Task 5).
- The binary's `RoomEventPayload` `{case_id, matrix_room_id, lifecycle_stage, member_count}` **has no field** for the chair-action metadata `{action, target_pseudonym, from_pseudonym, to_pseudonym, at}` the Success Criteria require (Task 1).
- The `room_chair_transferred` / `room_chair_override` consts exist but have **zero emitters** — Phase 3 is their first (Task 5).

## 4. Solution statement

The change has **two architectural surfaces**, each with its own validation toolchain:

**(a) Binary DTO extension (`crates/**`, Windows-validated) — trivial, the one in-scope crates touch.** The binary's `RoomEventPayload` gains **five optional** chair-action fields (`action`, `target_pseudonym`, `from_pseudonym`, `to_pseudonym`, `at`), all `#[serde(default, skip_serializing_if = "Option::is_none")]` — so the chain entry the bridge POSTs can carry the Success-Criteria payload shapes (`room_chair_override` → `{action, target_pseudonym}`; `room_chair_transferred` → `{from_pseudonym, to_pseudonym, at}`). The binary constructs **zero** `RoomEventPayload { … }` literals (it only deserialises the bridge POST + serialises into `append`), so optional `#[serde(default)]` fields are non-breaking — no callsite enumeration breaks. `CaseTransitionEvent` (the binary→bridge wire mirror) gains one optional `chair_pseudonym: Option<String>` field as the **named governance→bridge channel** for the initial chair (mirrors the existing `juror_pseudonyms` shape exactly). Binary-side **population** of `chair_pseudonym` is **deferred to the Phase-6 governance trigger** (§12) — Phase 3 lands the field + the bridge-side read, with a `juror_pseudonyms[0]` foreperson fallback for jury-adjacent town halls (OQ-V2-05).

**(b) Bridge stage controller (`services/bridge/**`, Linux-validated) — the bulk of the phase.** A new `stage.rs` module holds the **dual-sourced chair seat** + the **persisted FIFO queue** + the **mic-pass state machine** (phantom-typed `Watcher`/`Promoted`/`Speaking`, per `feedback_governance_type_state_handlers`), emitting grant/revoke **commands** to a `GrantSink` trait so the 4-mic-pass sequence + chair-override reordering are deterministic unit tests. The **30s grace** uses `tokio::time` (deterministic via `tokio::time::pause()` + `advance`) — a no-activate promoted speaker auto-revokes + next-promotes at the boundary. `livekit_jwt::mint_access_token` gains a `can_publish: bool` so a presenter token grants publish and a watcher token does not (the mic-grant mechanism, clarify `a3d0e9941441-067`). A new `room_event_client.rs` POSTs `RoomEventRequest { entry_kind, payload, actor_pseudonym }` to `brehon_room_event_url` with the `bridge_callback_secret` Bearer (mirrors `bridge_notify.rs`) — the chair-action emitters call it on transfer (`room_chair_transferred`) and override (`room_chair_override`). The town-hall provisioning path opens the room in **stage mode** (chair presenter grant, watchers muted), seats the dual-sourced chair into `chair_id`, and uses the Matrix room's text timeline as the **Q&A sidebar**. The end-to-end LiveKit + real-`append()` flow is a docker-compose-gated `#[ignore]` integration test mirroring `room_provisioning.rs`.

A reader can predict §11 from this: `RoomEventPayload` + `CaseTransitionEvent` (binary DTO); `livekit_jwt.rs`, `stage.rs` (new), `room_event_client.rs` (new), `config.rs`, `main.rs`, `room_provisioner.rs` (bridge); and one bridge integration-test module. **No migration, no new sidecar, no new dependency expected.**

## 5. Metadata

- **Phase:** `m3-core-stage-mode` (M3 Phase 3)
- **Branch:** `phase-m3-core-stage-mode` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 8 (Task 0 pre-flight + 6 impl + retro)
- **Estimated cargo budget:** N/A on daemon — **no cargo runs on EliteDesk** (validate-pending-laptop / -linux discipline; the laptop is the runner). Bridge Linux container first-run is COLD (~10–20 min); subsequent warm.
- **Forbidden-window applicability:** binding for the **local** laptop cargo (crates Windows + bridge Linux Docker); non-binding for daemon impl-task throughput (no daemon cargo).
- **Complexity score:** `3/10` — see breakdown. Well under the Sonnet split threshold (8). **Proceed-as-one.**

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 1 | 6 impl tasks; 1 above the 5 baseline |
| Migrations touched | +2 each | 0 | **No new migration** — uses the `bridge_room` columns m3-core-infra shipped |
| Crates touched | +1 each | 2 (+2) | `lemmy_api` + `lemmy_api_common` (Task 1 DTO); `brehon-bridge` is workspace-excluded so not double-counted with the binary crates — counted as the bridge surface = the +1 already in tasks |
| `crates/server/tests/e2e/*.rs` edits | +3 each | 0 | Task-1 binary assertion is a `#[cfg(test)]` unit test in `governance_log.rs`, NOT a `crates/server/tests/e2e.rs` edit (no 18k-line-file hang risk); bridge tests are separate small files |
| New ADR-affecting decisions | +2 each | 0 | Consumes ADR-015/016/011 + OQ-V2-05; supersedes none |
| Cargo budget peak above 6 GB | +1 per GB | 0 | No daemon cargo |
| **Total** | — | **3** | Threshold for split-DQ: `>8` (Sonnet) |

**Split decision:** 3 ≤ 8 — proceed-as-one, no split-DQ. The phase is logic-heavy but file-light (the marquee complexity is the FIFO + 30s state machine, captured in one `stage.rs` with deterministic unit tests, not breadth of files).

### 5.2 Per-task complexity ceiling (Sonnet target ≤4 files / ≤2 crates)

All tasks satisfy the Sonnet ceiling. Closest: **Task 5** hand-edits 3 files (`room_event_client.rs` new, `config.rs`, `stage.rs`) — 1 crate (`brehon-bridge`). **Task 1** edits 2 files across 2 crates (`lemmy_api`, `lemmy_api_common`) — ≤4 / ≤2 ✓. `Cargo.lock` does not change (no new dep). No e2e file appears in any `modifies:`.

## 6. Relationship to other M3 sub-phases

- **Depends on:** m3-core-infra (Phase 1, shipped) — `livekit_jwt::mint_access_token`, `bridge_room` RTC columns, the `rtc_enabled` gate, the `/bridge/actor-pseudonym` allocator endpoint. m3-core-entry-kinds (Phase 2, shipped `a5fc60a2d`) — the 3 chair/mute consts + `ROOM_KINDS` 10→13.
- **Followed by:** Phase 4 (emergency-mute) emits `room_chair`-sibling `room_mute_all` via Matrix power-levels (the cross-instance <500ms problem) — Phase 3 deliberately does NOT touch power-levels. Phase 5 (recording / MinIO). Phase 6 (e2e + pilot) populates the binary-side `chair_pseudonym` governance trigger + runs the live pilot.
- **First emitter:** Phase 3 is the FIRST phase to exercise the bridge→binary `append_room_event` callback (every prior phase deferred it). The `room_event_client.rs` it builds is reused by Phase 4 (mute-all) and Phase 5 (recording-uploaded) emissions.

## 7. Preflight guardrails inherited from prior phases

- **R1 (bridge-Linux):** every `services/bridge` cargo command runs via `scripts/brehon/cargo-linux.sh … --manifest-path services/bridge/Cargo.toml`; the Windows-local `cd services/bridge && cargo` form fails with `ruma-common` E0119 and is reference-only (`feedback_bridge_validates_on_linux_not_windows.md`).
- **R2 (Linux-compile gate):** every bridge-touching task (2–6) writes a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass` (`feedback_linux_compile_proof_is_a_gate.md`).
- **R3 (no daemon cargo):** workers write the appropriate `validate-pending-laptop[-linux]` DQ and **stop**; the laptop advisor runs all cargo (`feedback_validate_pending_laptop_write_then_stop.md`).
- **R4 (cargo capture):** every cargo invocation captures to a log file and reads `tail -20` + `echo exit: $?`; never pipe cargo through `tail`/`grep` (`cargo-output-capture.md` + `no-cargo-output-paste.md`).
- **R5 (Task 0 enumerates all probes explicitly):** see Task 0.
- **R6 (clippy uniform):** all clippy invocations use `--no-deps -- -D warnings`; crates clippy adds `--features full`; bridge clippy runs via `cargo-linux.sh`.
- **R7 (off-path / boundary is tested, not assumed):** the 30s no-activate boundary is a §16a story with a green deterministic checkpoint (tokio virtual time), not an untested happy-path assert (`feedback_build_what_tests_exercise.md`).
- **R8 (no schema migration):** Phase 3 uses the `bridge_room` columns already shipped; a new `services/bridge` embedded-schema column or a `crates/db_schema/migrations/**` migration is a scope violation → STOP (handover tripwire).
- **R9 (ADR-015 load-bearing):** every chair-action chain payload carries **pseudonyms only** — never `person_id`/username/MXID. Grep DoD in §15.7.
- **R10 (ADR-016 metadata-only):** only chair-action METADATA reaches `append_room_event`; speech/video/Q&A-text bytes are NEVER hashed or sent. §15.7 asserts the client carries only `{action, target_pseudonym, from/to_pseudonym, at, room context}`.

## 8. Flow design

```
BEFORE (m3-core-infra + entry-kinds):
  livekit_jwt::mint_access_token(key, secret, room, identity=<pseudonym>, ttl)  -> JWT { video: {room, roomJoin} }    (no canPublish)
  bridge_room (SQLite): { …, chair_id (unread), queue_state (unread), recording_config }
  binary  RoomEventPayload { case_id, matrix_room_id, lifecycle_stage, member_count }   (no chair fields)
  binary  handle_room_event(req, RoomEventRequest{entry_kind, payload, actor_pseudonym}) -> append_room_event(...)      MOUNTED @ /api/v4/governance/room-event (Bearer)
  bridge  config.brehon_room_event_url : #[allow(dead_code)] config-only — NO POSTer
  ENTRY_KIND_ROOM_CHAIR_TRANSFERRED / _CHAIR_OVERRIDE : registered, ZERO emitters

AFTER (m3-core-stage-mode):
  livekit_jwt::mint_access_token(…, can_publish: bool) -> video: { room, roomJoin, canPublish }                         [Task 2]
        presenter token: can_publish=true ; watcher token: can_publish=false
  binary  RoomEventPayload { …, action?, target_pseudonym?, from_pseudonym?, to_pseudonym?, at? }   (opt, serde default) [Task 1]
  binary  CaseTransitionEvent { …, chair_pseudonym? }   (opt; populated Phase 6)                                        [Task 1]
  bridge  stage::Stage { chair_id, fifo_queue(persisted queue_state), seat-state-machine }                              [Task 3]
        promote(watcher) -> GrantSink::grant_publish ; 30s grace timer                                                  [Task 4]
        on no-activate@30s -> GrantSink::revoke_publish + promote(next)                                                 [Task 4]
        chair override(force_demote|force_promote, target) -> reorder + grant/revoke                                    [Task 3]
  bridge  room_event_client::post_room_event(entry_kind, payload, actor_pseudonym)                                      [Task 5]
        --POST--> brehon_room_event_url (/api/v4/governance/room-event) + Bearer bridge_callback_secret
        chair transfer -> room_chair_transferred { from_pseudonym, to_pseudonym, at }                                   [Task 5]
        chair override  -> room_chair_override   { action, target_pseudonym }                                           [Task 5]
  bridge  town-hall provision: stage mode (chair presenter grant, watchers muted), seat chair_id, Q&A=Matrix text       [Task 6]
        chair_id <- event.chair_pseudonym ?? juror_pseudonyms[0] (foreperson, jury-adjacent)                            [Task 6]
```

The binary-side `chair_pseudonym` population (the governance trigger that says "this case-transition opens a town hall with chair X") is the **dashed seam** — Phase 6. Phase 3 lands the field + the bridge read + the foreperson fallback.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **Binary room-event seam** — `crates/api/api/src/governance/governance_log.rs:80-137` (`RoomEventPayload` struct + `ROOM_KINDS` + `append_room_event`); `crates/api/api/src/governance/room_event_handler.rs:17-36` (`RoomEventRequest` + `handle_room_event` Bearer + `append_room_event` call). [Tasks 1, 5]
- **Binary wire mirror** — `crates/api/api_common/src/governance.rs:885-902` (`CaseTransitionEvent` + the `juror_pseudonyms` `#[serde(default)]` shape to mirror for `chair_pseudonym`). [Task 1]
- **Outbound-POST + Bearer pattern** — `crates/api/api_utils/src/bridge_notify.rs:23-72` (`bridge_room_event_url()` + the `.post(...).header("Authorization", Bearer …).json(...).send()` shape — MIRROR for the bridge's outbound client). [Task 5]
- **LiveKit mint** — `services/bridge/src/livekit_jwt.rs:5-53` (`VideoGrant`/`Claims`/`mint_access_token`; extend `VideoGrant` with `canPublish`). [Task 2]
- **Bridge config** — `services/bridge/src/config.rs:35-40,55-60,80-96` (`brehon_room_event_url` config-only field + the optional `livekit_*` idiom + the `optional_non_empty` helper). [Tasks 5, 6]
- **Bridge room state** — `services/bridge/src/bridge_room.rs:3-100` (`open()` embedded schema with `chair_id`/`queue_state`; `lookup`/`upsert`/`set_watermark` accessor shape to mirror for a `queue_state` reader/writer). [Tasks 3, 6]
- **Bridge provisioning** — `services/bridge/src/room_provisioner.rs:35-79` (the `RoomEventPayload` enum mirror + `handle_transition` dispatch), `:183-237` (`provision_community_event_room` — the town-hall extension point), `:539-591` (`query_room_event_count` + `invite_to_room` — reqwest+`state.http_client` shape). [Tasks 5, 6]
- **Bridge inbound auth mirror** — `services/bridge/src/appservice.rs:196-235` (`handle_room_event` Bearer `bridge_callback_secret` check — the reverse direction, for symmetry). [Task 5]
- **Bridge integration-test harness** — `services/bridge/tests/room_provisioning.rs:1-65` (`#[tokio::test] #[ignore = "requires docker-compose stack"]` + POST-to-`/brehon/room-event` + `bridge_room::lookup` poll + `governance_log` query). [Task 6]
- **Registry (do NOT re-touch)** — `.claude/rules/governance-log-entry-kind-registry.md` §"m3-core-entry-kinds entry kinds (3, this sub-phase)" (the 3 consts are SHIPPED; Phase 3 emits, never re-declares). [Tasks 1, 5]
- **Lessons** — `feedback_bridge_validates_on_linux_not_windows.md`, `feedback_linux_compile_proof_is_a_gate.md`, `feedback_build_what_tests_exercise.md`, `feedback_governance_type_state_handlers.md`, `pattern_test_against_reality_not_syntax.md`. [Tasks 2–6]

## 10. Patterns to mirror

### 10.1 Binary `RoomEventPayload` chair-action extension (trivial DTO)

**Mirror:** `crates/api/api/src/governance/governance_log.rs:91-97`

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomEventPayload {
  pub case_id: i32,
  pub matrix_room_id: Option<String>,
  pub lifecycle_stage: String,
  pub member_count: Option<i32>,
  // M3 stage-mode chair-action metadata (opt; only chair entries set them).
  // ADR-015: pseudonyms only — never person_id/username/MXID.
  // ADR-016: metadata only — never room content.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub action: Option<String>,            // room_chair_override: "force_demote" | "force_promote"
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub target_pseudonym: Option<String>,  // room_chair_override target
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub from_pseudonym: Option<String>,    // room_chair_transferred prior chair
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub to_pseudonym: Option<String>,      // room_chair_transferred new chair
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub at: Option<String>,                // ISO-8601 transfer/override timestamp
}
```

`skip_serializing_if` keeps existing room kinds' JSON byte-identical (no chair fields serialised when None) → no regression to the m2-core-hook room emissions. `#[serde(default)]` keeps deserialisation back-compatible.

### 10.2 `CaseTransitionEvent.chair_pseudonym` (named governance→bridge channel)

**Mirror:** `crates/api/api_common/src/governance.rs:895-901` (the `juror_pseudonyms` `#[serde(default)]` shape)

```rust
  #[serde(default)]
  pub juror_pseudonyms: Vec<String>,
  /// Pre-resolved pseudonymous initial chair for a town-hall event (OQ-V2-05).
  /// `None` for non-town-hall transitions. Populated binary-side by the Phase-6
  /// governance trigger; Phase-3 bridge falls back to juror_pseudonyms[0]
  /// (foreperson) for jury-adjacent town halls. ADR-015: pseudonym ONLY.
  #[serde(default)]
  pub chair_pseudonym: Option<String>,
```

### 10.3 Bridge→binary outbound client (the FIRST emitter)

**Mirror:** `crates/api/api_utils/src/bridge_notify.rs:62-72` (Bearer POST) + `services/bridge/src/room_provisioner.rs:573-591` (`invite_to_room` reqwest via `state.http_client`).

```rust
// services/bridge/src/room_event_client.rs  (NEW)
#[derive(serde::Serialize)]
pub struct RoomEventPayload {          // bridge-side Serialize mirror of the binary struct
    pub case_id: i32,
    #[serde(skip_serializing_if = "Option::is_none")] pub matrix_room_id: Option<String>,
    pub lifecycle_stage: String,
    #[serde(skip_serializing_if = "Option::is_none")] pub member_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")] pub action: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub target_pseudonym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub from_pseudonym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub to_pseudonym: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")] pub at: Option<String>,
}
#[derive(serde::Serialize)]
struct RoomEventRequest<'a> {          // mirrors room_event_handler.rs:19-23
    entry_kind: &'a str,
    payload: RoomEventPayload,
    actor_pseudonym: Option<String>,
}
/// Fire-and-forget POST to the binary's /api/v4/governance/room-event.
/// Bearer = bridge_callback_secret. Transport errors logged + swallowed
/// (the chain entry is best-effort; never blocks the stage loop). ADR-016:
/// `payload` carries METADATA only — never room content.
pub async fn post_room_event(
    client: &reqwest::Client, url: &str, secret: &str,
    entry_kind: &str, payload: RoomEventPayload, actor_pseudonym: Option<String>,
) -> anyhow::Result<()> {
    let req = RoomEventRequest { entry_kind, payload, actor_pseudonym };
    client.post(url)
        .header("Authorization", format!("Bearer {secret}"))
        .json(&req).send().await?.error_for_status()?;
    Ok(())
}
```

**GOTCHA — URL path:** the binary route is `/api/v4/governance/room-event`, but the `config.rs` default `brehon_room_event_url` is `http://localhost:8536/governance/room-event` (missing `/api/v4`). Task 5 fixes the default to include `/api/v4` (deployments still override via `BREHON_ROOM_EVENT_URL`). Grep DoD: `rg 'api/v4/governance/room-event' services/bridge/src/config.rs`.

### 10.4 Publish-grant mint extension (the mic-grant mechanism, clarify `a3d0e9941441-067`)

**Mirror:** `services/bridge/src/livekit_jwt.rs:5-53`

```rust
#[derive(serde::Serialize, serde::Deserialize)]
struct VideoGrant {
    room: String,
    #[serde(rename = "roomJoin")] room_join: bool,
    #[serde(rename = "canPublish")] can_publish: bool,   // ADD
}
// mint_access_token gains `can_publish: bool`; presenter = true, watcher = false.
pub fn mint_access_token(
    api_key: &str, api_secret: &str, room: &str, identity: &str,
    ttl_secs: u64, can_publish: bool,                     // ADD
) -> Result<String> { /* set video.can_publish = can_publish */ }
```

`identity` stays a pseudonym by contract (ADR-015) — no `person_id`/username overload. **Stage mode = promote re-mints with `can_publish=true`; demote/revoke re-mints with `can_publish=false`.** Matrix power-levels are NOT used (those are Phase-4 emergency-mute). Existing call sites pass `can_publish` explicitly (enumerate in §11).

### 10.5 FIFO queue persistence in `bridge_room.queue_state`

**Mirror:** `services/bridge/src/bridge_room.rs:69-100` (`upsert`/`set_watermark` accessor shape)

```rust
// services/bridge/src/bridge_room.rs  (new accessors)
/// Serialise the FIFO queue (Vec<pseudonym>) to bridge_room.queue_state (JSON TEXT).
pub fn write_queue_state(conn, case_id, room_type, queue_json: &str) -> Result<()> { /* UPDATE … SET queue_state = ?1 */ }
pub fn read_queue_state(conn, case_id, room_type) -> Result<Option<String>> { /* SELECT queue_state … */ }
pub fn write_chair_id(conn, case_id, room_type, chair_pseudonym: &str) -> Result<()> { /* UPDATE … SET chair_id = ?1 */ }
pub fn read_chair_id(conn, case_id, room_type) -> Result<Option<String>> { /* SELECT chair_id … */ }
```

The queue is **persisted** (survives bridge restart — the PRD/research model; the in-memory alternative loses the raised-hand order on restart). `stage.rs` owns the in-memory `Vec<Pseudonym>` and flushes to `queue_state` on every mutation; on `Stage::load`, it reads `queue_state` back.

### 10.6 Stage-mode type-state machine (per `feedback_governance_type_state_handlers`)

**Mirror concept:** phantom-typed participant state, single-writer (the chair's bridge controller).

```rust
// services/bridge/src/stage.rs  (NEW)
pub enum SeatState { Watcher, Promoted /* granted, awaiting activation */, Speaking }
/// Commands the state machine emits; the GrantSink adapter applies them to LiveKit.
pub enum GrantCmd { GrantPublish(String /*pseudonym*/), RevokePublish(String) }
pub trait GrantSink { fn apply(&mut self, cmd: GrantCmd); }   // real impl re-mints LiveKit tokens; test impl records cmds
pub struct Stage { chair: Option<String>, fifo: VecDeque<String>, current: Option<(String, SeatState)>, /* … */ }
impl Stage {
    pub fn raise_hand(&mut self, p: &str);                    // append to FIFO (no dup)
    pub fn promote_next(&mut self, sink: &mut dyn GrantSink); // pop FIFO head, GrantPublish, SeatState::Promoted, start 30s grace
    pub fn on_activate(&mut self, p: &str);                   // Promoted -> Speaking, cancel grace
    pub fn on_grace_expired(&mut self, p: &str, sink: &mut dyn GrantSink); // RevokePublish(p) + promote_next  (Task 4)
    pub fn chair_override(&mut self, action: Override, target: &str, sink: &mut dyn GrantSink); // force_demote/promote out of FIFO order
    pub fn transfer_chair(&mut self, to: &str) -> (String /*from*/, String /*to*/); // delegate; returns the pair for the chain entry
}
```

The 4-mic-pass sequence (§16a Story 1) drives `raise_hand`×N + `promote_next`/`on_activate` and asserts the `GrantCmd` order recorded by a test `GrantSink` — **observable grant transitions, not config**. `feedback_governance_type_state_handlers`: invalid transitions (promote with empty FIFO, activate a non-promoted participant) are unrepresentable / return an error.

### 10.7 30s grace via deterministic virtual time (the load-bearing boundary)

**Mirror:** tokio's test-time control — `#[tokio::test(start_paused = true)]` (or `tokio::time::pause()` + `tokio::time::advance(Duration::from_secs(30))`).

```rust
#[tokio::test(start_paused = true)]
async fn grace_no_activate_auto_revokes_and_promotes_next() {
    // chair promotes W1 (granted, awaiting); W2 already queued
    // advance virtual clock 30s WITHOUT W1 activating
    tokio::time::advance(Duration::from_secs(30)).await;
    // assert: GrantSink saw RevokePublish(W1) THEN GrantPublish(W2)  (auto-revoke + next-promote)
}
```

The grace timer is `tokio::time::sleep(30s)` inside a `tokio::select!` against an activation signal — **never a wall-clock sleep**, so the test fires the boundary deterministically (no 30s real wait). This satisfies R7: the boundary is a TESTED signal, not a happy-path assert. A test that only asserts a successful promote does NOT satisfy the DoD.

### 10.8 Stage-mode provisioning (town-hall room-open)

**Mirror:** `services/bridge/src/room_provisioner.rs:183-237` (`provision_community_event_room`)

The town-hall path provisions the Matrix room (Q&A text timeline) + the LiveKit room (gated on `rtc_enabled`); mints the chair a **presenter token** (`can_publish=true`) and watchers **watcher tokens** (`can_publish=false`); seats `chair_id` from `event.chair_pseudonym ?? juror_pseudonyms.first()` (foreperson fallback, OQ-V2-05); initialises `queue_state` to `[]`. Idempotent skip mirrors the existing `bridge_room::lookup` guard.

### 10.9 Bridge integration-test harness (docker-gated)

**Mirror:** `services/bridge/tests/room_provisioning.rs:12-21,36-42`

```rust
// services/bridge/tests/stage_mode.rs  (NEW)
#[tokio::test] #[ignore = "requires docker-compose stack"]
async fn four_mic_pass_then_grace_boundary_emits_chair_entries() -> anyhow::Result<()> {
    // 1. provision a town-hall stage room (chair + 4 watchers)
    // 2. drive 4 raise-hand + promote/activate passes via the real LiveKit grants
    // 3. drive a 5th promote with no activation; assert auto-revoke + next-promote at 30s
    // 4. drive a chair transfer + a chair override
    // 5. query governance_log: assert room_chair_transferred {from,to,at} + room_chair_override {action,target_pseudonym} rows
    //    assert each payload field is a PSEUDONYM string (never person_id / @user:domain)
    todo!("implement against live docker-compose stack (Phase-6 pilot grade)")
}
```

This is the pilot-grade end-to-end signal. The **deterministic** DoD (Stories 1–4) runs as unit tests under `cargo-linux.sh test` without docker.

### 10.10 ADR-015 pseudonym pin (LOAD-BEARING)

Every chair-action payload field (`from_pseudonym`/`to_pseudonym`/`target_pseudonym`) and the `actor_pseudonym` arg are **pseudonyms** — sourced from the same allocator the JWT mint uses (`actor_pseudonym_helper::get_or_create` binary-side, surfaced via `/bridge/actor-pseudonym`). **Why it can't be deferred:** a real identity in a chain entry is permanent (hash-chained, append-only) — unscrubable, breaking `always_pseudonym` for the whole M3 cluster. **DoD (§15.7):** the integration test asserts every chair-payload field is a pseudonym string, never numeric `person_id` or `@user:domain` MXID; `rg -i 'person_id|username|mxid|@.*:' services/bridge/src/room_event_client.rs services/bridge/src/stage.rs` returns nothing identity-shaped.

### 10.11 ADR-016 metadata-only

`room_event_client::post_room_event` carries only `{action, target_pseudonym, from/to_pseudonym, at, case_id, matrix_room_id, lifecycle_stage}` — chair-action METADATA + room context. Speech/video/Q&A-text bytes are NEVER passed to it. The Q&A sidebar (Matrix text timeline) is content — it is NOT hashed or POSTed to the chain.

## 11. Files to change

**`crates/api` (lemmy_api + lemmy_api_common), Windows-validated:**
- `crates/api/api/src/governance/governance_log.rs` — 5 optional chair-action fields on `RoomEventPayload` + a `#[cfg(test)]` roundtrip unit test (Task 1)
- `crates/api/api_common/src/governance.rs` — optional `chair_pseudonym` on `CaseTransitionEvent` (Task 1)

**`services/bridge` (brehon-bridge, workspace-excluded), Linux-validated:**
- `services/bridge/src/livekit_jwt.rs` — `can_publish` param + `VideoGrant.canPublish` (Task 2)
- `services/bridge/src/stage.rs` — NEW: chair seat + FIFO queue + mic-pass state machine + `GrantSink` (Tasks 3, 4)
- `services/bridge/src/bridge_room.rs` — `read/write_queue_state` + `read/write_chair_id` accessors (Task 3)
- `services/bridge/src/room_event_client.rs` — NEW: bridge→binary outbound POST + chair-action emitters (Task 5)
- `services/bridge/src/config.rs` — fix `brehon_room_event_url` default to `/api/v4/...`; drop `#[allow(dead_code)]` once consumed (Task 5)
- `services/bridge/src/main.rs` — `mod stage;` + `mod room_event_client;` (Tasks 3, 5)
- `services/bridge/src/room_provisioner.rs` — `chair_pseudonym` on the bridge `CaseTransitionEvent` mirror + the town-hall stage-mode provisioning path + Q&A sidebar (Task 6)

**Tests (brehon-bridge integration), Linux-validated:**
- `services/bridge/tests/stage_mode.rs` — NEW: docker-gated `#[ignore]` end-to-end (Task 6)

### Struct-field add: enumerate all callsites

- **`RoomEventPayload` (binary, Task 1):** `rg "RoomEventPayload\s*\{" crates/` returns **zero** constructor literals (the binary only deserialises it in `room_event_handler.rs` and serialises it in `append_room_event`). New fields are `Option` + `#[serde(default)]` → non-breaking. No caller enumeration needed (the template's skip condition is satisfied: zero existing constructors).
- **`CaseTransitionEvent` (binary, Task 1):** constructed at `crates/api/api_utils/src/bridge_notify.rs:136-143` (`governance_case_after_transition`). Adding `chair_pseudonym` requires that one constructor to set `chair_pseudonym: None` (Phase-3 default) — Task 1 `modifies:` includes `bridge_notify.rs`. The bridge mirror (`room_provisioner.rs:19-33`) uses `#[serde(default)]` + silently-ignored unknown fields, so it compiles unchanged; Task 6 adds the field there to READ it.
- **`mint_access_token` (bridge, Task 2):** `rg "mint_access_token" services/bridge/` — call sites are the `#[cfg(test)]` test in `livekit_jwt.rs` (update to pass `can_publish`) + any Task-6 provisioning callsite (passes `true`/`false` explicitly). No `Default` shape; enumerate at impl: every call passes the new bool.
- **`VideoGrant` (bridge, Task 2):** constructed only inside `mint_access_token` — one site.

> **Task 1 note (Caller crates — compiles-only-after-Task-1):** `bridge_notify.rs` (lemmy_api_utils) constructs `CaseTransitionEvent`; it MUST be in Task 1's `modifies:` so the `chair_pseudonym: None` initialiser lands in the same commit (else `lemmy_api_utils` fails to compile). Hence Task 1 touches 3 files across 3 crate dirs (`api`, `api_common`, `api_utils`) — still ≤4 files; crates = {lemmy_api, lemmy_api_common, lemmy_api_utils} = 3. **This exceeds the ≤2-crate Sonnet ceiling by one** — acceptable because all three edits are a single mechanical DTO-field propagation (add field + one `None` initialiser), not independent logic. Flagged for advisor gate-1 awareness; not split (splitting a struct-field-add across tasks would leave a non-compiling intermediate commit).

## 12. NOT building in m3-core-stage-mode

- **Binary-side population of `CaseTransitionEvent.chair_pseudonym`** — deferred to **Phase 6** (the governance trigger that designates a case-transition as a town hall with chair X). Phase 3 lands the field + the bridge read + the `juror_pseudonyms[0]` foreperson fallback. Reason: the governance event-scheduling trigger is pilot-phase work; wiring it now expands into governance decision logic the brief forbids.
- **Federation-wide emergency mute / `room_mute_all`** — Phase 4. The const exists (`governance_log.rs:255`) but stage-mode does NOT emit it. Matrix power-levels (the mute-all mechanism) are NOT touched this phase (clarify `a3d0e9941441-067`).
- **MinIO / LiveKit Egress / recording / `room_recording_uploaded`** — Phase 5 (gated by `record_town_halls`).
- **A new `bridge_room` column or any migration** — Phase 3 USES the `chair_id`/`queue_state` columns m3-core-infra shipped (handover tripwire: a new migration → STOP).
- **A new bridge dependency** — stage-mode reuses `jsonwebtoken`, `rusqlite`, `reqwest`, `serde_json`, `tokio` (all present). If a task needs a new dep, the regenerated `Cargo.lock` lands in the SAME commit (watchpoint #6).
- **Anonymous-town-hall pseudonym-overlay work beyond the existing JWT mint** — Phase 6 / inherited. The `always_pseudonym` identity→pseudonym mapping is already at JWT-issue time (m3-core-infra).
- **Re-touching the entry-kind registry / re-declaring the 3 consts** — SHIPPED (`a5fc60a2d`). Phase 3 emits via `append_room_event`, never re-declares.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Task 0 is non-`[P]` (barrier).

> **Cohort note:** Task 1 (`crates/api`, Windows) and Task 2 (`services/bridge` `livekit_jwt`, Linux) are file-disjoint across different crates and different validation runners → genuine `[P]` (Cohort A). Bridge Tasks 3–6 share the bridge crate + `Cargo.lock` + cold Linux builds, so they run **serial** (Task 3 → 4 → 5 → 6). Per the cross-lane cap, at most 2 Junior workers run concurrently.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment + branch + base state before Task 1.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — Docker daemon (cargo-linux.sh Linux build + bridge docker-gated tests)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — Docker in LINUX-container mode (cargo-linux.sh requires it)
docker info --format '{{.OSType}}'   # EXPECT: linux

# Probe 2 — on the phase branch
git branch --show-current            # EXPECT: phase-m3-core-stage-mode

# Probe 3 — the 3 chair/mute consts are SHIPPED on base (Phase 3 must NOT re-declare)
rg -c '^pub const ENTRY_KIND_ROOM_(CHAIR_TRANSFERRED|CHAIR_OVERRIDE|MUTE_ALL)' \
  crates/db_schema/src/source/governance/governance_log.rs   # EXPECT: 3

# Probe 4 — ROOM_KINDS gate already admits the chair kinds (binary emit path ready)
rg -c 'ENTRY_KIND_ROOM_CHAIR_(OVERRIDE|TRANSFERRED)' crates/api/api/src/governance/governance_log.rs  # EXPECT: 2

# Probe 5 — the binary room-event seam is mounted (bridge will POST to it)
rg -c 'handle_room_event' crates/api/routes/src/lib.rs                                  # EXPECT: >=1
rg -c 'pub async fn append_room_event' crates/api/api/src/governance/governance_log.rs  # EXPECT: 1

# Probe 6 — m3-core-infra deliverables present (mint fn + RTC columns)
rg -c 'pub fn mint_access_token' services/bridge/src/livekit_jwt.rs                     # EXPECT: 1
rg -c 'chair_id|queue_state' services/bridge/src/bridge_room.rs                          # EXPECT: >=2

# Probe 7 — bridge baseline COMPILES on Linux (cold ~10-20 min; warm after)
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-stage-task0-bridge-check.log 2>&1
echo "bridge baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task0-bridge-check.log  # EXPECT: exit 0

# Probe 8 — crates baseline compiles (Windows)
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-stage-task0-check.log 2>&1"
echo "crates baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task0-check.log  # EXPECT: exit 0

# Probe 9 (negative) — wrapper propagates failure
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m3-stage-task0-neg.log 2>&1"
echo "negative exit: $?"   # EXPECT: NON-ZERO
```

**EXPECT:** Probes 0–8 succeed per their inline expectations; Probe 9 exits non-zero. **No commit at Task 0.**

### Task 1 [P]: Binary `RoomEventPayload` chair-action fields + `CaseTransitionEvent.chair_pseudonym`

**ACTION:** extend the binary room-event DTO with the chair-action metadata fields the Success Criteria require, and add the optional `chair_pseudonym` governance→bridge channel.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/governance_log.rs   # 5 optional chair-action fields on RoomEventPayload + #[cfg(test)] roundtrip
  - crates/api/api_common/src/governance.rs           # optional chair_pseudonym on CaseTransitionEvent
  - crates/api/api_utils/src/bridge_notify.rs         # set chair_pseudonym: None in the CaseTransitionEvent constructor
```

**IMPLEMENT (file 1 of 3):** `governance_log.rs` — add the 5 optional fields per §10.1 (`action`, `target_pseudonym`, `from_pseudonym`, `to_pseudonym`, `at`), each `#[serde(default, skip_serializing_if = "Option::is_none")]`. Add a `#[cfg(test)]` unit test: build a `RoomEventPayload` for `room_chair_override` (`action: Some("force_demote")`, `target_pseudonym: Some("pseu_x")`, others None) → `serde_json::to_value` → assert the JSON has `action`+`target_pseudonym` and OMITS `from_pseudonym`/`to_pseudonym`/`at` (skip_serializing_if); build a `room_chair_transferred` payload → assert `from_pseudonym`/`to_pseudonym`/`at` present, `action`/`target_pseudonym` omitted. (Pure serde — no DB, no `feature=full` gating issue since the struct is already `#[cfg(feature="full")]`-adjacent; mirror the file's existing test-module gating.)
**IMPLEMENT (file 2 of 3):** `api_common/src/governance.rs` — add `#[serde(default)] pub chair_pseudonym: Option<String>` to `CaseTransitionEvent` per §10.2 with the doc-comment.
**IMPLEMENT (file 3 of 3):** `bridge_notify.rs` — in `governance_case_after_transition`'s `CaseTransitionEvent { … }` constructor (`:136-143`), add `chair_pseudonym: None` (Phase-3 default; Phase-6 populates).

**MIRROR:** §10.1, §10.2; `crates/api/api/src/governance/room_event_handler.rs:19-23` (the `RoomEventRequest` consumer that deserialises the new fields).

**GOTCHA:** `skip_serializing_if = "Option::is_none"` is load-bearing — without it, existing m2-core-hook room emissions would gain `"action":null` etc., changing the hashed JSON. **No `RoomEventPayload { … }` constructor exists** in `crates/` (grep is empty), so the `Option` fields break nothing; but `CaseTransitionEvent` HAS one constructor (`bridge_notify.rs:136`) — it MUST get `chair_pseudonym: None` in the same commit or `lemmy_api_utils` fails to compile. ADR-015: these fields carry pseudonyms; ADR-016: metadata only.

**VALIDATE (Windows):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-stage-task1-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task1-check.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop` DQ with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""]` and an `e2e_filter` scoping to the new `governance_log` unit test (or null — it's a unit test, runs under check); commit + push, then **stop**. (No `validate-pending-laptop-linux` — Task 1 touches no bridge/Cargo.toml/migration/cfg code.)

### Task 2 [P]: LiveKit publish-grant — `can_publish` on the mint fn

**ACTION:** add a `can_publish: bool` to `mint_access_token` and `canPublish` to `VideoGrant` so stage mode can mint presenter (publish) vs watcher (no-publish) tokens.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/livekit_jwt.rs   # VideoGrant.canPublish + mint_access_token can_publish param + test
```

**IMPLEMENT:** per §10.4 — add `#[serde(rename = "canPublish")] can_publish: bool` to `VideoGrant`; add `can_publish: bool` as the final `mint_access_token` param; set `video.can_publish = can_publish`. Update the existing `mint_pseudonym_claims` test to pass `can_publish: true` and assert `claims.video.can_publish == true`; add a second test minting a watcher token (`can_publish: false`) and asserting `false`.

**MIRROR:** `services/bridge/src/livekit_jwt.rs:5-76` (struct + mint + the existing decode-roundtrip test).

**GOTCHA:** `identity` stays a pseudonym by contract — do NOT add a `person_id` overload (ADR-015). The presenter/watcher distinction is `can_publish`, NOT Matrix power-levels (clarify `a3d0e9941441-067` — power-levels are Phase-4 emergency-mute only). Bridge compiles on **Linux only**.

**VALIDATE (Linux):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-stage-task2-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task2-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml livekit_jwt \
  > .claude/PRPs/debug/m3-stage-task2-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task2-bridge-test.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (`commands` = the `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test` lines), commit + push, then **stop**.

### Task 3: Stage-mode core — chair seat + persisted FIFO queue + mic-pass state machine

**ACTION:** add `services/bridge/src/stage.rs` (the chair-seat + FIFO + type-state mic-pass machine emitting `GrantCmd` to a `GrantSink`) and the `bridge_room` queue/chair accessors.

**FILES:**

```yaml
creates:
  - services/bridge/src/stage.rs
modifies:
  - services/bridge/src/bridge_room.rs   # read/write_queue_state + read/write_chair_id accessors
  - services/bridge/src/main.rs          # mod stage;
requires:
  - task: 2
    reason: the real GrantSink adapter re-mints LiveKit tokens via mint_access_token(can_publish); Task 3 references the can_publish grant concept
```

**IMPLEMENT (file 1 of 3):** `stage.rs` per §10.6 — `SeatState`/`GrantCmd`/`GrantSink`/`Stage`; `raise_hand` (FIFO append, no dup), `promote_next` (pop head → `GrantPublish` → `Promoted`), `on_activate` (`Promoted`→`Speaking`), `chair_override(force_demote|force_promote, target)` (reorder out of FIFO + grant/revoke), `transfer_chair(to)` (returns `(from, to)` for the chain entry). Persist the FIFO + chair to `bridge_room` on every mutation (`write_queue_state`/`write_chair_id`); `Stage::load` reads them back. **Unit tests (deterministic, no docker):** (a) **4-mic-pass-in-sequence** — 4 `raise_hand` + 4 `promote_next`/`on_activate`; assert the test `GrantSink` recorded `GrantPublish` in FIFO order for all 4 (§16a Story 1, marquee DoD); (b) **chair-override reorder** — `force_promote` a watcher out of FIFO order; assert it's granted before the FIFO head; `force_demote` the speaker; assert `RevokePublish`.
**IMPLEMENT (file 2 of 3):** `bridge_room.rs` — `read/write_queue_state` + `read/write_chair_id` per §10.5 (mirror `upsert`/`set_watermark`). Add a `#[cfg(test)]` test: write a queue JSON → read it back equal; write a chair_id → read it back.
**IMPLEMENT (file 3 of 3):** `main.rs` — `mod stage;` (alphabetical).

**MIRROR:** §10.5, §10.6; `services/bridge/src/bridge_room.rs:69-100` (accessor shape); `feedback_governance_type_state_handlers.md` (type-state).

**GOTCHA:** the FIFO is **single-writer** (the chair's bridge controller) — no lock needed, but the queue MUST persist to `queue_state` (survives bridge restart; the PRD model). Invalid transitions (promote empty FIFO, activate a non-promoted participant) return an error / are unrepresentable, not silent no-ops. Bridge compiles on **Linux only**. The 30s grace timer is Task 4 — Task 3's `promote_next` leaves the `Promoted` participant awaiting activation with no timer yet (Task 4 adds it).

**VALIDATE (Linux; feeds §16a Story 1):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-stage-task3-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task3-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml stage \
  > .claude/PRPs/debug/m3-stage-task3-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task3-bridge-test.log   # EXPECT: exit 0 (4-mic-pass + override tests pass)
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test … stage`), commit + push, then **stop**.

### Task 4: 30s grace — auto-revoke + next-promote at the boundary

**ACTION:** wire a 30s grace timer into `stage.rs`: a promoted-but-not-activated speaker is auto-revoked at 30s and the next queued watcher is promoted.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/stage.rs   # 30s grace timer (tokio::time) + on_grace_expired + the boundary test
requires:
  - task: 3
    reason: extends the Stage state machine + GrantSink from Task 3
```

**IMPLEMENT:** per §10.7 — `promote_next` starts a `tokio::time::sleep(Duration::from_secs(GRACE_SECS))` inside a `tokio::select!` raced against an activation signal; on grace-elapsed-without-activation call `on_grace_expired(p, sink)` → `RevokePublish(p)` + `promote_next` (the next FIFO head). `GRACE_SECS = 30` const. **Unit test (deterministic):** `#[tokio::test(start_paused = true)]` — promote W1 (W2 queued), `tokio::time::advance(Duration::from_secs(30))` WITHOUT activating W1, assert the test `GrantSink` recorded `RevokePublish(W1)` THEN `GrantPublish(W2)` (§16a Story 2). Add a contrast test: promote W1, `on_activate(W1)` before 30s, advance 30s, assert NO revoke fired (activation cancels the grace).

**MIRROR:** §10.7 (tokio virtual-time test); the Task-3 `Stage` machine.

**GOTCHA (R7 — the load-bearing boundary):** the grace timer MUST use `tokio::time` (cancellable, virtual-time-controllable), NEVER `std::thread::sleep` or wall-clock. The test fires the boundary with `tokio::time::advance` — a test that only asserts a successful promote does NOT satisfy the DoD (surface as a §3.5 watchpoint failure if §16a Story 2 omits the boundary). Bridge compiles on **Linux only**.

**VALIDATE (Linux; feeds §16a Story 2):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-stage-task4-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task4-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml stage \
  > .claude/PRPs/debug/m3-stage-task4-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task4-bridge-test.log   # EXPECT: exit 0 (grace-boundary test passes)
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test … stage`), commit + push, then **stop**.

### Task 5: Bridge→binary room-event client + chair-action emission (the FIRST emitter)

**ACTION:** add `services/bridge/src/room_event_client.rs` (POST `RoomEventRequest` to the binary's `/api/v4/governance/room-event` with the Bearer secret) and wire the chair-transfer + chair-override emitters.

**FILES:**

```yaml
creates:
  - services/bridge/src/room_event_client.rs
modifies:
  - services/bridge/src/config.rs   # fix brehon_room_event_url default to /api/v4/...; drop #[allow(dead_code)]
  - services/bridge/src/main.rs     # mod room_event_client;
  - services/bridge/src/stage.rs    # call post_room_event on transfer_chair + chair_override
requires:
  - task: 1
    reason: the RoomEventRequest/RoomEventPayload wire shape must match the binary's extended struct (Task 1)
  - task: 4
    reason: the emitters fire from the Stage transfer/override paths (Tasks 3, 4)
```

**IMPLEMENT (file 1 of 4):** `room_event_client.rs` per §10.3 — the bridge-side `RoomEventPayload` Serialize mirror + `RoomEventRequest` + `post_room_event(client, url, secret, entry_kind, payload, actor_pseudonym)` (reqwest `.post().header(Bearer).json().send().error_for_status()`). A `#[cfg(test)]` test: serialise a `room_chair_override` request → assert the JSON has `entry_kind: "room_chair_override"` + `payload.action` + `payload.target_pseudonym` and OMITS the transfer fields; serialise a `room_chair_transferred` request → assert `from_pseudonym`/`to_pseudonym`/`at` present. (Assert on the JSON, not a live POST — the live POST is the docker-gated Task-6 test.)
**IMPLEMENT (file 2 of 4):** `config.rs` — change the `brehon_room_event_url` default from `http://localhost:8536/governance/room-event` to `http://localhost:8536/api/v4/governance/room-event`; remove `#[allow(dead_code)]` (now consumed). Keep the `BREHON_ROOM_EVENT_URL` env override.
**IMPLEMENT (file 3 of 4):** `main.rs` — `mod room_event_client;`.
**IMPLEMENT (file 4 of 4):** `stage.rs` — `transfer_chair` builds `room_chair_transferred` `{from_pseudonym, to_pseudonym, at}` (the acting chair = `actor_pseudonym`) and calls `post_room_event`; `chair_override` builds `room_chair_override` `{action, target_pseudonym}` and calls it. Use the entry-kind consts by their string values (`"room_chair_transferred"`/`"room_chair_override"`) — the bridge does not depend on `lemmy_db_schema`, so it passes the literal strings the binary's `ROOM_KINDS` admits (the binary validates against `ENTRY_KIND_ROOM_*`).

**MIRROR:** §10.3 (client), §10.11 (metadata-only); `crates/api/api_utils/src/bridge_notify.rs:62-72` (Bearer POST); `services/bridge/src/room_provisioner.rs:573-591` (reqwest via `state.http_client`); `services/bridge/src/appservice.rs:204-220` (the Bearer-secret shape, reverse direction).

**GOTCHA (ADR-015 — load-bearing):** the chair payload fields are **pseudonyms** — `from_pseudonym`/`to_pseudonym`/`target_pseudonym`/`actor_pseudonym` are all pseudonym strings the Stage already holds (sourced from the LiveKit-identity pseudonyms). **Why it can't be deferred:** a real identity in a hash-chained chair entry is permanent and unscrubable. **DoD:** `rg -i 'person_id|username|@.*:' services/bridge/src/room_event_client.rs` returns nothing identity-shaped. **GOTCHA (ADR-016):** `post_room_event` carries METADATA only — never Q&A text / speech / video. **GOTCHA (URL):** the binary route is `/api/v4/governance/room-event`; verify `rg 'api/v4/governance/room-event' services/bridge/src/config.rs` after the fix. Transport errors are logged + swallowed (best-effort; never block the stage loop). Bridge compiles on **Linux only**.

**VALIDATE (Linux; feeds §16a Stories 3+4):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-stage-task5-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task5-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml room_event_client \
  > .claude/PRPs/debug/m3-stage-task5-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task5-bridge-test.log   # EXPECT: exit 0 (override + transfer JSON-shape tests pass)
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test … room_event_client`), commit + push, then **stop**.

### Task 6: Stage-mode provisioning + Q&A sidebar + docker-gated end-to-end test

**ACTION:** extend the bridge town-hall provisioning path to open in stage mode (chair presenter grant, watchers muted), seat the dual-sourced chair into `chair_id`, attach the Q&A sidebar, and add the docker-gated end-to-end integration test.

**FILES:**

```yaml
creates:
  - services/bridge/tests/stage_mode.rs
modifies:
  - services/bridge/src/room_provisioner.rs   # chair_pseudonym mirror field + stage-mode provisioning + Q&A sidebar
requires:
  - task: 2
    reason: mints presenter (can_publish=true) / watcher (false) tokens
  - task: 5
    reason: chair-action emission wired through the Stage + room_event_client
```

**IMPLEMENT (file 1 of 2):** `room_provisioner.rs` — add `#[serde(default)] pub chair_pseudonym: Option<String>` to the bridge `CaseTransitionEvent` mirror (`:19-33`). Add a town-hall stage-mode provisioning path (extend `provision_community_event_room` or a sibling): on `rtc_enabled`, provision the Matrix room (its text timeline = the Q&A sidebar) + initialise stage state — seat `chair_id` from `event.chair_pseudonym` else `event.juror_pseudonyms.first()` (foreperson fallback, OQ-V2-05); `write_chair_id`; initialise `queue_state` to `[]`; mint the chair a **presenter token** (`can_publish=true`) and watchers **watcher tokens** (`can_publish=false`). Idempotent skip via the existing `bridge_room::lookup` guard. (The Q&A sidebar is the Matrix room's native text timeline — NO new room type, NO content hashing, ADR-016.)
**IMPLEMENT (file 2 of 2):** `stage_mode.rs` per §10.9 — the docker-gated `#[ignore]` end-to-end test (`four_mic_pass_then_grace_boundary_emits_chair_entries`): provision a stage room, drive 4 real mic-passes, drive the 30s no-activate boundary, drive a chair transfer + override, then query `governance_log` and assert `room_chair_transferred {from_pseudonym, to_pseudonym, at}` + `room_chair_override {action, target_pseudonym}` rows exist with **pseudonym** payload fields (never `person_id`/MXID). `todo!()`-stub the body with the step comments (pilot-grade; the live stack is a Phase-6 concern) — mirror `room_provisioning.rs`'s `#[ignore]` stub convention.

**MIRROR:** §10.8 (provisioning), §10.9 (test harness); `services/bridge/src/room_provisioner.rs:183-237` (`provision_community_event_room`); `services/bridge/tests/room_provisioning.rs:12-42` (ignore-stub shape).

**GOTCHA:** the Q&A sidebar is **content** — it lives in the Matrix room text timeline and is NEVER hashed or POSTed to the chain (ADR-016). The watcher tokens MUST be `can_publish=false` (muted) and only the chair gets `can_publish=true` at open (stage mode = one presenter slot). `chair_id` population is the named OQ-V2-05 mechanism: governance-assigned (`chair_pseudonym`) with the foreperson fallback — the binary-side population of `chair_pseudonym` is Phase-6 (§12). Bridge compiles on **Linux only**; the `#[ignore]` test does NOT run under `cargo-linux.sh test` (it needs the live stack).

**VALIDATE (Linux):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-stage-task6-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task6-bridge-check.log   # EXPECT: exit 0
# stage_mode.rs is #[ignore]'d — assert it COMPILES (not runs) via --no-run:
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run \
  > .claude/PRPs/debug/m3-stage-task6-bridge-testcompile.log 2>&1
echo "test-compile exit: $?"; tail -20 .claude/PRPs/debug/m3-stage-task6-bridge-testcompile.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test --test stage_mode --no-run`), commit + push, then **stop**.

### Task 7: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + lessons + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lessons to `.claude/lessons/feedback_*.md` in the retro commit. Specific signals: the bridge→binary callback design (FIRST emitter — did the seam survive review?); the deterministic 30s-grace virtual-time test pattern (new this phase — candidate lesson); whether the type-state `Stage` machine paid off; the trivial-DTO-across-3-crates Task 1 (did the `chair_pseudonym: None` propagation compile first-try?).

---

## 14. Testing strategy

- **crates static (Windows):** `cargo check --workspace --features full`; `cargo clippy --workspace --features full --no-deps -- -D warnings` (Task 1).
- **crates unit (Windows):** the `governance_log.rs` `#[cfg(test)]` chair-payload serde roundtrip (Task 1) runs under check/test.
- **bridge static (Linux):** `cargo-linux.sh check/clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings` (Tasks 2–6).
- **bridge unit (Linux, no docker):** `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml` — `livekit_jwt` (publish-grant), `stage` (4-mic-pass + override + **30s grace boundary**), `room_event_client` (override/transfer JSON shape), `bridge_room` (queue/chair accessors). **These ARE the marquee DoD** (deterministic, exercise the real logic).
- **bridge integration (Linux, docker-gated, `#[ignore]`):** `stage_mode.rs` — compiles under `--no-run` in CI; runs only against a live LiveKit+Tuwunel stack (Phase-6 pilot grade).
- **No migration round-trip** (no new migration). **No deploy-smoke** (no new sidecar — reuses m3-core-infra's `profiles: ["rtc"]` stack).

## 15. Validation commands (DoD)

> Every command below is written in the exact form the advisor runs at gate 1; all dry-run clean against `phase-m3-core-stage-mode` HEAD. Bridge cargo uses `cargo-linux.sh --manifest-path` (NEVER Windows-local — `ruma-common` E0119). No `rg`-absent / line-count greps.

### 15.1 crates static analysis (Task 1 — Windows)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-stage-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.2 crates lint (Task 1 — Windows, uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/m3-stage-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.3 bridge static + lint + unit (Tasks 2–6 — Linux, Docker rust:1.95)

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-stage-bridge-check.log 2>&1;  echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings > .claude/PRPs/debug/m3-stage-bridge-clippy.log 2>&1; echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-stage-bridge-test.log 2>&1; echo "exit: $?"  # EXPECT: 0 (livekit_jwt + stage + room_event_client + bridge_room unit tests pass; #[ignore] integration skipped)
```

### 15.4 bridge integration-test compile (Task 6 — Linux; #[ignore] body not run)

```bash
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run > .claude/PRPs/debug/m3-stage-bridge-itc.log 2>&1
echo "exit: $?"   # EXPECT: 0 (the docker-gated test compiles; live run is Phase-6 pilot)
```

### 15.5 Cross-cutting verification (planner asserts at end-of-phase)

- [ ] R8: NO new migration (no `crates/db_schema/migrations/**`, no new `bridge_room` embedded column); `git diff --stat governance-v0..HEAD -- migrations/ crates/db_schema/migrations/` is empty.
- [ ] R9 (ADR-015): `rg -i 'person_id|username|@.*:' services/bridge/src/room_event_client.rs services/bridge/src/stage.rs` returns nothing identity-shaped (only pseudonym strings flow to the chain).
- [ ] R10 (ADR-016): `post_room_event` payload carries only chair-action metadata + room context — no Q&A/speech/video bytes; the Q&A sidebar is the Matrix text timeline (never hashed).
- [ ] FIRST emitter: `rg 'room_chair_transferred|room_chair_override' services/bridge/src/` returns emit call sites in `stage.rs` (via `room_event_client`), not just config.
- [ ] URL: `rg 'api/v4/governance/room-event' services/bridge/src/config.rs` returns the fixed default.
- [ ] Mic-grant = publish-grant: `rg 'can_publish' services/bridge/src/livekit_jwt.rs` present; NO Matrix power-level call for mic-passing (`rg -i 'power.?level' services/bridge/src/stage.rs` empty).
- [ ] Registry untouched: `git diff governance-v0..HEAD -- crates/db_schema/src/source/governance/governance_log.rs` shows ONLY the `RoomEventPayload` field additions, NO new `ENTRY_KIND_*` const.
- [ ] No new dep: `git diff governance-v0..HEAD -- services/bridge/Cargo.toml services/bridge/Cargo.lock` is empty (or, if non-empty, BOTH change in the same commit — watchpoint #6).

## 16. Acceptance criteria

- [ ] Tasks 0–7 completed in dependency order
- [ ] §15.1/15.2 (crates check + clippy) exit 0 (Task 1)
- [ ] §15.3 (bridge check + clippy + unit test) exit 0 (Tasks 2–6)
- [ ] §15.4 (integration-test compiles `--no-run`) exit 0 (Task 6)
- [ ] §15.5 cross-cutting boxes all ticked
- [ ] §16a Stories 1–5 all `[done]`
- [ ] No edits outside §11; entry-kind registry + consts untouched; no migration
- [ ] Retro committed (Task 7)
- [ ] **Linux-compile gate:** `validate-pending-laptop-linux` DQ at `result:pass` (Tasks 2–6 touch `services/bridge/**`) before `bm-pr`
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

## 16a. Stories

### Story 1: The chair drives 4 mic-passes in sequence with no manual intervention (marquee DoD)

- **Composing tasks:** Task 3 (FIFO + mic-pass machine), Task 2 (publish-grant token)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage::four_mic_pass_in_sequence`
- **Expected output:** test passes — the `GrantSink` recorded `GrantPublish` for all 4 watchers in FIFO order
- **Brief-Scope outputs to verify:** `services/bridge/src/stage.rs` exists with `Stage::promote_next`/`raise_hand`/`on_activate`; FIFO persisted via `bridge_room::write_queue_state`.

### Story 2: A promoted speaker who doesn't activate within 30s is auto-revoked and the next is promoted (the load-bearing boundary)

- **Composing tasks:** Task 4 (30s grace)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage::grace_no_activate_auto_revokes_and_promotes_next`
- **Expected output:** test passes — after `tokio::time::advance(30s)` with no activation, `GrantSink` saw `RevokePublish(W1)` THEN `GrantPublish(W2)`; the contrast test (activation before 30s) fires NO revoke
- **Brief-Scope outputs to verify:** `stage.rs` `on_grace_expired` + the `GRACE_SECS = 30` const + the `#[tokio::test(start_paused = true)]` boundary test.

### Story 3: Chair override emits `room_chair_override` with a pseudonym target

- **Composing tasks:** Task 1 (payload fields), Task 5 (emitter)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml room_event_client::override_emits_pseudonym_target`
- **Expected output:** test passes — the serialised `RoomEventRequest` has `entry_kind: "room_chair_override"`, `payload.action`, `payload.target_pseudonym` (a pseudonym string), and omits the transfer fields
- **Brief-Scope outputs to verify:** `room_event_client.rs` `post_room_event`; `stage.rs::chair_override` calls it; `RoomEventPayload` has `action`/`target_pseudonym` (binary, Task 1).

### Story 4: Chair transfer (delegate) emits `room_chair_transferred`

- **Composing tasks:** Task 1 (payload fields), Task 5 (emitter)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml room_event_client::transfer_emits_from_to_at`
- **Expected output:** test passes — the serialised request has `entry_kind: "room_chair_transferred"` + `payload.{from_pseudonym, to_pseudonym, at}` (pseudonym strings), and omits `action`/`target_pseudonym`
- **Brief-Scope outputs to verify:** `stage.rs::transfer_chair` returns `(from, to)` + calls `post_room_event`; `RoomEventPayload` has `from_pseudonym`/`to_pseudonym`/`at` (Task 1).

### Story 5: A town-hall opens in stage mode (chair presenter, watchers muted), Q&A live; end-to-end emits the chair entries on a live stack

- **Composing tasks:** Task 6 (provisioning + Q&A + integration test), Tasks 2/5
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run` (compile-gate; live run is Phase-6 pilot)
- **Expected output:** the docker-gated `stage_mode.rs` test compiles; provisioning seats `chair_id` + mints presenter/watcher tokens
- **Brief-Scope outputs to verify:** `room_provisioner.rs` town-hall stage-mode path seats `chair_id` (governance-assigned ?? foreperson) + Q&A=Matrix text timeline; `services/bridge/tests/stage_mode.rs` exists asserting `governance_log` chair rows carry pseudonyms.

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Story's checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent/empty) trigger the catch-fire procedure.

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–9 per expectations)
- [ ] Tasks 1–6 committed (one commit each)
- [ ] §15 validation green at every gate (crates Windows / bridge Linux unit / integration-compile)
- [ ] §16a Stories 1–5 all `[done]`
- [ ] Retro committed (Task 7)
- [ ] PR opened by BM against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report shows all stories ✓
- [ ] **Linux-compile gate:** `validate-pending-laptop-linux` DQ at `result:pass` (Tasks 2–6) before `bm-pr`
- [ ] Post-merge phase branch retained for retro reads

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| ADR-015 leak — a `person_id`/username reaches a chair chain entry | LOW | HIGH | Pseudonyms-only by the `Stage`'s held-state contract; §15.5 grep DoD; `RoomEventPayload` fields named `*_pseudonym`; integration test asserts pseudonym shape |
| 30s grace tested as a happy-path promote, not the boundary | MED | HIGH | §16a Story 2 + R7 + Task 4 GOTCHA: `tokio::time::advance(30s)` deterministic boundary test is the DoD; advisor §3.5 rejects a §16a story that omits it |
| Bridge→binary URL wrong (`/api/v4` missing) → silent 404, entries never land | MED | MED | §10.3 + Task 5 fix + §15.5 grep `api/v4/governance/room-event`; the docker-gated test asserts real `governance_log` rows |
| Mic-grant implemented via Matrix power-levels (Phase-4 mechanism) | LOW | MED | clarify `a3d0e9941441-067` + §10.4 + §15.5: publish-grant re-mint only; `rg power.?level stage.rs` must be empty |
| Task 1 `chair_pseudonym` propagation leaves `lemmy_api_utils` non-compiling | LOW | MED | §11 caller note: `bridge_notify.rs:136` gets `chair_pseudonym: None` in the SAME commit; §15.1 catches it |
| FIFO queue lost on bridge restart (in-memory only) | LOW | MED | §10.5: queue persists to `bridge_room.queue_state`; `Stage::load` reads it back; `bridge_room` accessor unit test |
| Scope creep into Phase-6 governance trigger (binary chair_pseudonym population) | MED | MED | §12 explicit NOT-building; Phase-3 ships the field + bridge read + foreperson fallback only |
| Bridge Linux cold build (~10–20 min) stalls serial Tasks 3–6 | MED | LOW | Task 0 Probe 7 warms the registry volume; tasks serial avoids concurrent cold builds + Cargo.lock races |

## 19. Notes

**Two scope decisions, both pre-seeded as resolved planner DQs for advisor gate-1 ratification:**

**(1) Bridge→binary callback — build the bridge-side client (clarify `a3d0e9941441-066` → option (a); resolved planner DQ `da838b8fc109-001`).** The chair-action emission requires the bridge to POST `RoomEventRequest` to the binary's `/api/v4/governance/room-event`. Reconnaissance confirms the **binary half already exists and is mounted**: `room_event_handler.rs::handle_room_event` (Bearer-authed) → `append_room_event` (gates on `ROOM_KINDS`, runs `scrub_json` + hash-chain + signing), route at `routes/src/lib.rs:484`. The gap is purely the **bridge-side outbound HTTP client** — `config.rs:35-40 brehon_room_event_url` is `#[allow(dead_code)]` config-only and `room_provisioner.rs:391,439,463` defer emission. Task 5 builds `room_event_client.rs` (mirror `bridge_notify.rs:62-72` Bearer POST). **No binary handler change** beyond the Task-1 DTO. Resolved (planner): build the bridge client only.

**(2) `RoomEventPayload` cannot carry chair metadata — extend it (trivial DTO; resolved planner DQ `da838b8fc109-002`, advisor gate-1 ratify).** The binary's `RoomEventPayload {case_id, matrix_room_id, lifecycle_stage, member_count}` (`governance_log.rs:91-97`) has **no field** for `{action, target_pseudonym}` (`room_chair_override`) or `{from_pseudonym, to_pseudonym, at}` (`room_chair_transferred`) — both mandated by Success Criteria lines 139–140. Recommendation (Task 1): add 5 `#[serde(default, skip_serializing_if="Option::is_none")]` fields. The binary constructs **zero** `RoomEventPayload {…}` literals (`rg` empty — it only deserialises from the bridge POST + serialises into `append`), so the add is non-breaking and needs no callsite enumeration. `skip_serializing_if` keeps the m2-core-hook room emissions' hashed JSON byte-identical. This is the **one in-scope `crates/**` touch** — within the bootstrap tripwire's "trivial DTO" carve-out (NOT "Lemmy-crate changes beyond a trivial DTO"). It additionally adds `CaseTransitionEvent.chair_pseudonym` (the named OQ-V2-05 governance→bridge channel; populated Phase-6) — making Task 1 span 3 crate dirs (`api`/`api_common`/`api_utils`) for a single mechanical field propagation. **Advisor: ratify the trivial-DTO crates touch + the 3-crate Task-1 ceiling exception at gate-1.** Resolved (planner): extend; do not split Task 1 (splitting leaves a non-compiling intermediate commit).

**Marquee DoD is deterministic unit tests, not the docker stack.** Per `feedback_build_what_tests_exercise` + `pattern_test_against_reality_not_syntax`: the 4-mic-pass sequence and the 30s no-activate boundary are the load-bearing logic, captured as deterministic `cargo-linux.sh test` unit tests over the real `Stage` state machine (`tokio::time::advance` for the boundary — no wall-clock wait, no flake). The end-to-end LiveKit + real-`append()` flow is the docker-gated `#[ignore]` `stage_mode.rs` (Phase-6 pilot grade). This split is honest: the deterministic tests prove the logic; the pilot proves the integration.

**No new migration, no new sidecar, no new dependency.** Phase 3 is logic on top of m3-core-infra's plumbing — the `bridge_room` RTC columns, the `mint_access_token` fn (extended, not replaced), the `profiles:["rtc"]` stack. Complexity 3/10. If any task is tempted to add a `services/bridge` column or a dep, STOP (handover tripwire / watchpoint #6).

## 20. Confidence score

- **Plan correctness:** 8/10 — the two scope decisions (callback client + DTO extension) are surfaced as planner DQs; the seam is fully reconnoitred (binary half exists, bridge half is the build). The one judgment call is the deterministic-unit-test-vs-docker split for the DoD (defended above).
- **Cargo budget:** 9/10 — no daemon cargo; bridge cold-build is the only time cost, mitigated by Task-0 warm-up + serial bridge tasks.
- **Test coverage:** 8/10 — all four Success-Criteria chair behaviours (4-mic-pass, 30s boundary, override-emits, transfer-emits) are §16a stories with green deterministic checkpoints; the live end-to-end is a compile-gated `#[ignore]` test deferred to the Phase-6 pilot by scope.
