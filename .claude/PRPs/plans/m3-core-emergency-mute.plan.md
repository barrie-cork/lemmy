# Plan: m3-core-emergency-mute — federation-wide emergency mute-all, first `room_mute_all` chain emission

## 1. Summary

This sub-phase gives the M3 town-hall bridge a **chair-triggered, federation-wide emergency mute-all**: a chair (or moderator holding room-admin authority) drops **every non-chair publisher** in a town-hall room — including participants whose home instance is a different federated bridge — and the action lands the **first emission** of the `room_mute_all` hash-chain entry. **Headline acceptance:** a deterministic unit test enumerates a multi-publisher set, fires mute-all, and asserts **EVERY** non-chair publisher's publish right is revoked (the zero-holder negative invariant — the test breaks if the revoke is deleted); the `room_mute_all` chain entry carries **`chair_pseudonym`** (pseudonyms only, ADR-015) + **`federated: true`** metadata only (ADR-016); and a docker-gated `#[ignore]` cross-instance integration test carries the real **<500ms-at-the-publisher-client** signal (Phase-6-pilot-grade). The **cross-instance authority is Matrix `m.room.power_levels`** (OQ-V2-06) — mirroring `sanction_handler.rs`'s GET→mutate→PUT machinery generalised to drop all non-chair publishers in one PUT — paired with a **local LiveKit `RevokePublish` sweep** of locally-known publishers for instant in-instance effect (option (b), belt-and-suspenders; §4). The `room_mute_all` const is **already registered** (m3-core-entry-kinds, Phase 2) — this phase **emits only**, via the existing `EmitIntent` → `pending_emits` → `drain_emits` → `post_room_event` seam stage-mode built. No new entry-kind const, no registry-count bump, no migration, no new dependency; the one in-scope `crates/**` touch is a trivial optional-field DTO extension (`federated: Option<bool>`).

## 2. Source

- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 4: M3-core emergency-mute" (240–243), §Technical Approach (174–203, esp. the federation-wide-mute risk row at 198), §Success Criteria emergency-mute row (141), §Cross-Cutting Impact (155–159), OQ-V2-06 (70, 242) @ `5a58686d7`.
- `.claude/PRPs/briefs/m3-core-emergency-mute-planning-1.md` (the authorising brief) @ `5a58686d7`.
- `.claude/PRPs/handovers/m3-core-emergency-mute-bootstrap.md` §1 (scope+DoD), §4 watchlist (6 items + the cr-4 negative-invariant item 7), §7 catch-fire, §"Stop-and-ask tripwires" @ `5a58686d7`.
- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — the SIBLING bridge plan (canonical-schema-first gate): §5 complexity breakdown, §7 R1–R10 guardrails, §10 patterns, §15 three-surface DoD, §16a story shape @ `5a58686d7`.
- `docs/brehon-law-inspired-network/V2/messaging.md` §3.4.3 C3.7 (line 188 — mute-all UX + "re-granting publish permissions" line, reconciled in §4) + §3.7 OQ-V2-06 (line 233 — room-global, federation-wide, Matrix power-level semantics) @ `5a58686d7`.
- ADRs (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`): **ADR-015** (pseudonymised actor IDs) — load-bearing for the `room_mute_all` `chair_pseudonym`; **ADR-016** (cross-app backplane; content never hashed) — metadata-only emission; **ADR-004** (plane separation; room metadata may cross to the chain, content may not); **ADR-011** (AGPLv3 inherited — no new component this phase); **ADR-014** (federation interop — RTC plane is Brehon↔Brehon only, OQ-V2-08). OQ-V2-06 RESOLVED 2026-04-17.
- Lessons that bind decisions:
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); never Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — every bridge-touching task writes a `validate-pending-laptop-linux` DQ; `bm-pr` gates on `result:pass`.
  - `feedback_build_what_tests_exercise.md` — the marquee test must EXERCISE the real flow (an actual publisher set revoked + the real `room_mute_all` EmitIntent queued with `federated: true`), not assert on config/struct shape.
  - `feedback_authz_state_machine_test_asserts_negative.md` — **cr-4 from stage-mode PR #202**; `room_mute_all` is a zero-holder invariant — the marquee test asserts the NEGATIVE side (no publisher retains publish after mute) and breaks when the revoke is deleted.
  - `pattern_test_against_reality_not_syntax.md` — assert on observable LiveKit grant transitions + the actual `room_mute_all` chain-entry payload, not config/struct shape.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — the ADR-015 pseudonym pin (§4) is made load-bearing, not named.
  - `feedback_entry_kind_runtime_allowlist_check.md` — `room_mute_all` is already in `ROOM_KINDS` (verified present at `governance_log.rs:174`); this phase emits, never re-registers.
  - `feedback_plan_dod_dry_run_at_write.md` — every §15 command is written in the exact form the advisor runs at gate 1 (dry-run mentally before commit).
  - `feedback_validate_pending_laptop_write_then_stop.md` — workers write the `validate-pending-laptop[-linux]` DQ and STOP; the laptop advisor runs all cargo.

## 3. Problem statement

m3-core-stage-mode shipped the chair-controlled stage (`stage.rs`: chair seat, FIFO queue, mic-passing, chair override/transfer), the `EmitIntent` → `pending_emits` → `room_event_client::drain_emits` → `post_room_event` callback seam (first emitter), and the LiveKit publish-grant mic mechanism (`GrantSink`/`GrantCmd`). m3-core-entry-kinds registered `ENTRY_KIND_ROOM_MUTE_ALL` (`governance_log.rs:255`, in `ROOM_KINDS` at the api shim `:174`). But **nothing enacts an emergency mute**:

- There is **no cross-instance mute mechanism.** The chair cannot drop all non-chair publishers. The Matrix power-level machinery that could (`sanction_handler.rs` `get_power_levels` `:281` / `put_power_levels` `:304` / `compute_power_override` `:250`) is wired for **per-user** sanctions (drop ONE subject below a threshold), not the **mute-all** generalisation (raise the publish bar so EVERY non-chair publisher loses publish in one PUT).
- There is **no local instant mute.** `stage.rs` revokes a single publisher on handoff (`promote_next` revokes the prior holder); it has no "revoke ALL current publishers" sweep.
- `ENTRY_KIND_ROOM_MUTE_ALL` has **zero emitters.** Phase 4 is its first (`stage.rs` `chair_override`/`transfer_chair` already push `room_chair_*` EmitIntents; mute-all has no analogue).
- The binary's `RoomEventPayload` (`governance_log.rs:91-110`) and the bridge mirror (`room_event_client.rs:6-23`) have the chair-action fields but **no `federated` field** — the Success-Criteria `room_mute_all { chair_pseudonym, federated: true|false }` shape (line 141) cannot be carried.

## 4. Solution statement

The change has **two architectural surfaces**, each with its own validation toolchain, exactly mirroring the stage-mode split:

**(a) Binary DTO extension (`crates/**`, Windows-validated) — trivial, the one in-scope crates touch.** The binary's `RoomEventPayload` gains **one optional** field `federated: Option<bool>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`) so the `room_mute_all` chain entry can carry the `federated: true|false` metadata. The binary constructs **zero** non-test `RoomEventPayload { … }` literals (it only deserialises the bridge POST + serialises into `append`), so the optional field is non-breaking; `skip_serializing_if` keeps every existing room kind's hashed JSON byte-identical. **`chair_pseudonym` is NOT a new payload field** — it is the existing top-level `actor_pseudonym` (the `EmitIntent.actor_pseudonym` → `RoomEventRequest.actor_pseudonym` → `append_room_event(…, actor_pseudonym)` arg), exactly as `room_chair_override`/`room_chair_transferred` carry the acting chair (§19 (2)). The chain entry's actor IS the `chair_pseudonym` (ADR-015 pin).

**(b) Bridge mute enactment (`services/bridge/**`, Linux-validated) — the bulk of the phase, option (b).** Mechanism decision (§19 (1), pre-seeded resolved planner DQ for gate-1 ratify):

- **Cross-instance authority = Matrix `m.room.power_levels`** (OQ-V2-06). A new `mute_handler.rs` module mirrors `sanction_handler.rs` (`get_power_levels`/`put_power_levels` reused via `pub(crate)`) but generalises the mutate step: `compute_mute_all_override` **raises the publish/voice power requirement** (`events["m.call.member"]` + the MSC3401 alias `org.matrix.msc3401.call.member`, and the message threshold) **above `users_default`**, so EVERY non-elevated participant loses publish in **one PUT** — which propagates cross-instance via Matrix federation (the room is the federated room; OQ-V2-06 chair-authority-is-room-global). The chair retains publish by virtue of the room-admin power-level seated at town-hall provisioning. A per-room loop over `bridge_room::lookup_by_case` (`:59`) applies it to every room for the case, best-effort, mirroring `sanction_handler.rs:188-228`. This is the **cross-instance hammer** — the <500ms cross-instance propagation is the documented risk (PRD line 198 fallback).
- **Local instant effect = LiveKit `RevokePublish` sweep** (option (b) belt-and-suspenders). A new `Stage::mute_all(publishers, sink)` method takes the explicit set of locally-known publishers and emits `GrantCmd::RevokePublish(p)` for **each** via the existing `GrantSink` (the same mechanism stage-mode's mic-passing uses for instant local revoke), clears the seat, and pushes the `room_mute_all` EmitIntent `{ federated }` with `actor_pseudonym = chair`. This guarantees the **in-instance <500ms** bar deterministically (PRD fallback line 198) and makes the **zero-holder negative invariant** a clean deterministic test (the `Recorder` `GrantSink` observes every publisher's revoke; deleting the sweep loop makes the test fail).

A reader can predict §11 from this: `RoomEventPayload.federated` (binary DTO); `mute_handler.rs` (new, cross-instance power-level path) + `sanction_handler.rs` (expose GET/PUT `pub(crate)`) + `stage.rs` (`mute_all` sweep+emit) + `room_event_client.rs` (`federated` mirror field) + `main.rs` (`mod mute_handler;`); and one bridge integration-test module. **No migration, no new sidecar, no new dependency** (reuses `reqwest`, `serde_json`, `rusqlite`, `tokio`, `anyhow`, `time`).

## 5. Metadata

- **Phase:** `m3-core-emergency-mute` (M3 Phase 4)
- **Branch:** `phase-m3-core-emergency-mute` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 6 (Task 0 pre-flight + 4 impl + retro)
- **Estimated cargo budget:** N/A on daemon — **no cargo runs on EliteDesk** (validate-pending-laptop / -linux discipline; the laptop is the runner). Bridge Linux container first-run is COLD (~10–20 min); subsequent warm.
- **Forbidden-window applicability:** binding for the **local** laptop cargo (crates Windows + bridge Linux Docker); non-binding for daemon impl-task throughput (no daemon cargo). Shape G RESIDUAL-ONLY (local cargo-linux + CR/Copilot = full internal coverage; public green-check only).
- **Complexity score:** `2/10` — see breakdown. Well under the Sonnet split threshold (8). **Proceed-as-one.**

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 4 impl tasks; none above the 5 baseline |
| Migrations touched | +2 each | 0 | **No new migration** — mute-logic + chain-emission only |
| Crates touched | +1 each | 1 (+1) | `lemmy_api` (Task 1 DTO); `brehon-bridge` is workspace-excluded — counted as the bridge surface = the +1 already in tasks |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | Bridge tests are separate small files (`tests/emergency_mute.rs`), NOT the 18k-line `crates/server/tests/e2e.rs` (no worker-hang risk) |
| New ADR-affecting decisions | +2 each | 0 | Consumes ADR-015/016/004/011/014 + OQ-V2-06; supersedes none |
| Cargo budget peak above 6 GB | +1 per GB | 0 | No daemon cargo (Shape-G residual) |
| **Total** | — | **2** | Threshold for split-DQ: `>8` (Sonnet) |

**Split decision:** 2 ≤ 8 — proceed-as-one, no split-DQ. The phase is logic-focused but file-light (the const exists, the emit seam exists, the power-level machinery exists — this phase composes them).

### 5.2 Per-task complexity ceiling (Sonnet target ≤4 files / ≤2 crates)

All tasks satisfy the Sonnet ceiling. Closest: **Task 2** hand-edits 3 files (`mute_handler.rs` new, `sanction_handler.rs`, `main.rs`) — 1 crate (`brehon-bridge`). **Task 3** edits 2 files (`stage.rs`, `room_event_client.rs`) — 1 crate. **Task 1** edits 1 file (binary `governance_log.rs`) — 1 crate. `Cargo.lock` does not change (no new dep). No e2e file appears in any `modifies:`.

## 6. Relationship to other M3 sub-phases

- **Depends on:** m3-core-infra (Phase 1, shipped) — `bridge_room` RTC columns, the `rtc_enabled` gate, the puppet/pseudonym allocator. m3-core-entry-kinds (Phase 2, shipped) — `ENTRY_KIND_ROOM_MUTE_ALL` + `ROOM_KINDS` 72. m3-core-stage-mode (Phase 3, shipped PR #202) — `stage.rs` (`Stage`/`GrantSink`/`GrantCmd`/`EmitIntent`/`pending_emits`), the `room_event_client.rs` callback seam + `drain_emits`, the `room_provisioner.rs` town-hall stage-mode provisioning path.
- **Followed by:** Phase 5 (recording / MinIO, flag-gated) emits the M2-registered `room_recording_uploaded`. Phase 6 (e2e + pilot) wires the live chair-action HTTP endpoints (the trigger for `transfer_chair`/`chair_override`/`mute_all` — today they push EmitIntents drained at provisioning time, zero-loop) + runs the live pilot incl. the real cross-instance <500ms measurement.
- **Reuses:** the bridge→binary `room_event_client` (stage-mode built it) + the `sanction_handler` Matrix power-level machinery (m2-late-2 built it). This phase builds NO new emitter and NO new power-level HTTP plumbing — it composes both.

## 7. Preflight guardrails inherited from prior phases

- **R1 (bridge-Linux):** every `services/bridge` cargo command runs via `scripts/brehon/cargo-linux.sh … --manifest-path services/bridge/Cargo.toml`; the Windows-local `cd services/bridge && cargo` form fails with `ruma-common` E0119 and is reference-only (`feedback_bridge_validates_on_linux_not_windows.md`).
- **R2 (Linux-compile gate):** every bridge-touching task (2–4) writes a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass` (`feedback_linux_compile_proof_is_a_gate.md`).
- **R3 (no daemon cargo):** workers write the appropriate `validate-pending-laptop[-linux]` DQ and **stop**; the laptop advisor runs all cargo (`feedback_validate_pending_laptop_write_then_stop.md`).
- **R4 (cargo capture):** every cargo invocation captures to a log file and reads `tail -20` + `echo exit: $?`; never pipe cargo through `tail`/`grep` (`cargo-output-capture.md` + `no-cargo-output-paste.md`).
- **R5 (Task 0 enumerates all probes explicitly):** see Task 0.
- **R6 (clippy uniform):** all clippy invocations use `--no-deps -- -D warnings`; crates clippy adds `--features full`; bridge clippy runs via `cargo-linux.sh`.
- **R7 (the negative invariant is TESTED, not assumed):** the zero-holder mute invariant is a §16a story with a green deterministic checkpoint whose test FAILS when the revoke sweep is deleted (`feedback_authz_state_machine_test_asserts_negative.md` + `feedback_build_what_tests_exercise.md`); a happy-path "a mute command fired" assert does NOT satisfy the DoD.
- **R8 (no schema migration):** Phase 4 is mute-logic + chain-emission; a new `services/bridge` embedded-schema column or a `crates/db_schema/migrations/**` migration is a scope violation → STOP (handover tripwire).
- **R9 (ADR-015 load-bearing):** the `room_mute_all` chain entry's actor (`actor_pseudonym` = `chair_pseudonym`) carries **a pseudonym only** — never `person_id`/username/MXID. Grep DoD in §15.5.
- **R10 (ADR-016 metadata-only):** only mute METADATA (`{chair_pseudonym, federated, room context}`) reaches `append_room_event`; speech/video/Q&A-text bytes are NEVER hashed or sent. §15.5 asserts the emit payload carries only metadata.
- **R11 (power-levels ≠ LiveKit grants for the cross-instance path):** the cross-instance authority is Matrix `m.room.power_levels` (`mute_handler.rs`, mirror `sanction_handler.rs`); the LiveKit `RevokePublish` sweep is the option-(b) LOCAL belt-and-suspenders only, never the cross-instance mechanism (watchlist #3). §15.5 asserts `mute_handler.rs` issues a power-level PUT and `stage.rs::mute_all`'s cross-instance reach is NOT claimed via `mint_access_token`.

## 8. Flow design

```
BEFORE (m3-core-stage-mode + entry-kinds shipped):
  binary  RoomEventPayload { case_id, matrix_room_id, lifecycle_stage, member_count,
                             action?, target_pseudonym?, from_pseudonym?, to_pseudonym?, at? }   (no `federated`)
  binary  ROOM_KINDS ⊇ { room_mute_all }   (registered; ZERO emitters)
  bridge  sanction_handler:: get_power_levels / put_power_levels (private) ; compute_power_override (per-USER drop)
  bridge  stage::Stage { chair, fifo, current, pending_emits } ; promote_next revokes ONE prior holder
  bridge  room_event_client:: post_room_event / drain_emits  (the callback seam; drained at provisioning)
  ENTRY_KIND_ROOM_MUTE_ALL : registered, ZERO emitters

AFTER (m3-core-emergency-mute):
  binary  RoomEventPayload { …, federated? }   (opt, serde default, skip_serializing_if)               [Task 1]
  bridge  sanction_handler:: get_power_levels / put_power_levels  -> pub(crate) (reused, unchanged body) [Task 2]
  bridge  mute_handler:: compute_mute_all_override(content) -> raises call-member/voice + msg power      [Task 2]
              requirement ABOVE users_default (all non-chair lose publish; chair stays via room-admin)
          mute_handler:: mute_all_power_levels(state, case_id)                                           [Task 2]
              --per room (lookup_by_case)--> GET power_levels -> compute_mute_all_override -> PUT (full)
              ==Matrix federation propagates the one PUT cross-instance== (OQ-V2-06 cross-instance hammer)
  bridge  stage::Stage::mute_all(publishers: &[String], sink)                                            [Task 3]
              for p in publishers: sink.apply(RevokePublish(p))   (LOCAL instant effect — option b)
              self.current = None
              pending_emits.push(EmitIntent { entry_kind: "room_mute_all",
                                              payload: RoomEventPayload { …, federated: Some(true) },
                                              actor_pseudonym: self.chair.clone() })   (chair_pseudonym)
  bridge  room_event_client:: RoomEventPayload { …, federated? }   (mirror; existing literals += None)   [Task 3]
              --drain_emits--> post_room_event -> POST /api/v4/governance/room-event (Bearer)
              binary append_room_event(pool, "room_mute_all", payload, actor_pseudonym=chair) -> chain
  bridge  tests/emergency_mute.rs  #[ignore] docker-gated cross-instance <500ms + chain-row assertion    [Task 4]
```

The live HTTP **trigger** for `mute_all` (a chair clicking "mute all") is **Phase-6** — like stage-mode's transfer/override, the EmitIntent is pushed by the sync method and drained by the existing `drain_emits`; `mute_all_power_levels` carries `#[allow(dead_code)]`/module-level allow until the Phase-6 live mute endpoint calls it (the dead_code-scaffold-then-remove flow stage-mode used 3×, bootstrap §3).

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **Binary room-event seam** — `crates/api/api/src/governance/governance_log.rs:91-110` (`RoomEventPayload` struct — add `federated`), `:112-158` (the `#[cfg(test)]` skip-serializing test to extend), `:160-177` (`ROOM_KINDS` — `room_mute_all` present at `:174`, do NOT re-touch), `:186-198` (`append_room_event`). [Task 1]
- **Matrix power-level machinery to MIRROR** — `services/bridge/src/sanction_handler.rs:188-228` (the per-room GET→mutate→PUT loop), `:250-277` (`compute_power_override` — the pure-function shape to mirror for `compute_mute_all_override`), `:281-300` (`get_power_levels`), `:304-325` (`put_power_levels`), `:437-472` (`compute_power_override` test shape), `:474-482` (`#[ignore]` live test convention). [Task 2]
- **Bridge room lookup** — `services/bridge/src/bridge_room.rs:59` (`lookup_by_case(conn, case_id) -> Vec<(room_type, room_id)>` — the per-room loop source). [Task 2]
- **Bridge config** — `services/bridge/src/config.rs:23` (`tuwunel_url`), `:25` (`as_token`), `:37` (`brehon_room_event_url`), `:39` (`bridge_callback_secret`). [Tasks 2, 3]
- **Stage state machine + EmitIntent push pattern** — `services/bridge/src/stage.rs:26-35` (`GrantCmd`/`GrantSink`), `:47-51` (`EmitIntent`), `:57-69` (`Stage` fields), `:202-271` (`chair_override` — the EmitIntent push pattern to mirror), `:276-311` (`transfer_chair` — EmitIntent + `actor_pseudonym` shape), `:313-318` (`flush_queue`), `:321-632` (the `#[cfg(test)]` module incl. the `Recorder` `GrantSink` + the negative-assertion test idioms). [Task 3]
- **Bridge→binary outbound client + drain** — `services/bridge/src/room_event_client.rs:5-23` (`RoomEventPayload` mirror — add `federated`), `:36-53` (`post_room_event`), `:62-76` (`drain_emits`), `:85-142` (the JSON-shape test idiom to mirror for `room_mute_all`). [Task 3]
- **Town-hall provisioning + drain callsite** — `services/bridge/src/room_provisioner.rs:201` (`provision_townhall_stage_room`), `:267-299` (chair seat — the chair pseudonym source), `:325-356` (the `drain_emits` callsite). [reference, Tasks 3, 4]
- **Bridge module list** — `services/bridge/src/main.rs:12-25` (add `mod mute_handler;` alphabetically). [Task 2]
- **Integration-test harness (MIRROR)** — `services/bridge/tests/stage_mode.rs:1-23` (`#[tokio::test] #[ignore = "requires docker-compose stack"]` + the step-comment stub convention). [Task 4]
- **Registry (do NOT re-touch)** — `.claude/rules/governance-log-entry-kind-registry.md` §"m3-core-entry-kinds entry kinds (3, this sub-phase)" (`room_mute_all` is SHIPPED; Phase 4 emits, never re-declares; count stays 72). [Tasks 1, 3]
- **Lessons** — `feedback_bridge_validates_on_linux_not_windows.md`, `feedback_linux_compile_proof_is_a_gate.md`, `feedback_build_what_tests_exercise.md`, `feedback_authz_state_machine_test_asserts_negative.md`, `pattern_test_against_reality_not_syntax.md`, `feedback_entry_kind_runtime_allowlist_check.md`. [Tasks 1–4]

## 10. Patterns to mirror

### 10.1 Binary `RoomEventPayload.federated` extension (trivial DTO)

**Mirror:** `crates/api/api/src/governance/governance_log.rs:91-110`

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomEventPayload {
  pub case_id: i32,
  pub matrix_room_id: Option<String>,
  pub lifecycle_stage: String,
  pub member_count: Option<i32>,
  // … existing chair-action fields (action / target_pseudonym / from_pseudonym / to_pseudonym / at) …
  /// M3 emergency-mute: true when the mute reached cross-instance publishers
  /// (Matrix power-levels federate the PUT), false for an in-instance-only mute.
  /// Only `room_mute_all` sets it. ADR-016: metadata only.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub federated: Option<bool>,
}
```

`skip_serializing_if = "Option::is_none"` keeps existing room kinds' JSON byte-identical (no `"federated":null` on m2/m3 emissions) → no hash regression. `#[serde(default)]` keeps deserialisation back-compatible. **`chair_pseudonym` is NOT added here** — it rides the top-level `actor_pseudonym` (§19 (2)).

### 10.2 `compute_mute_all_override` — the mute-all power-level mutate (pure fn)

**Mirror:** `services/bridge/src/sanction_handler.rs:250-277` (`compute_power_override`) + the GET→merge→PUT shape at `:204-228`.

```rust
// services/bridge/src/mute_handler.rs  (NEW)
/// Mute-ALL generalisation of compute_power_override: instead of dropping ONE
/// subject below a threshold, RAISE the publish/voice power requirement above
/// users_default so EVERY non-elevated participant loses publish in one PUT.
/// The chair keeps publish via the room-admin power-level seated at provisioning.
/// Returns the mutated full content (always PUT the whole object — never fragment).
fn compute_mute_all_override(content: &serde_json::Value) -> serde_json::Value {
    let mut out = content.clone();
    let users_default = content.get("users_default").and_then(|v| v.as_i64()).unwrap_or(0);
    // One above users_default: non-elevated users (watchers) lose publish; the
    // chair (room-admin, e.g. 100) stays above it.
    let mute_bar = users_default + 1;
    if let Some(obj) = out.as_object_mut() {
        let events = obj.entry("events").or_insert_with(|| serde_json::json!({}));
        if let Some(ev) = events.as_object_mut() {
            ev.insert("m.call.member".to_string(), serde_json::json!(mute_bar));
            ev.insert("org.matrix.msc3401.call.member".to_string(), serde_json::json!(mute_bar));
        }
    }
    out
}
```

The deterministic test (§16a Story 3) mirrors `sanction_handler.rs:437-472`: feed a known `content`, assert the returned content's `events["m.call.member"]` is `> users_default`.

### 10.3 `mute_all_power_levels` — the cross-instance authority (async, per-room)

**Mirror:** `services/bridge/src/sanction_handler.rs:188-228` (the per-room best-effort GET→mutate→PUT loop) + `:281-325` (GET/PUT helpers, reused via `pub(crate)`).

```rust
// services/bridge/src/mute_handler.rs  (NEW)
/// Drop ALL non-chair publishers in every room for `case_id` via Matrix
/// power-levels. ONE PUT per room; Matrix federation propagates it
/// cross-instance (OQ-V2-06 — chair authority is room-global). Best-effort:
/// one room's failure MUST NOT abort the others (mirror sanction loop).
#[allow(dead_code)] // live HTTP trigger lands Phase 6; reachable from the #[ignore] test now.
pub(crate) async fn mute_all_power_levels(state: &AppState, case_id: i64) -> anyhow::Result<usize> {
    let conn = bridge_room::open(&state.bridge_db_path)?;
    let rooms = bridge_room::lookup_by_case(&conn, case_id)?;
    let mut applied = 0usize;
    for (_room_type, room_id) in &rooms {
        let content = match crate::sanction_handler::get_power_levels(state, room_id).await {
            Ok(c) => c,
            Err(e) => { tracing::warn!(err = %e, room_id = %room_id, "mute_all: get_power_levels failed — skipping room"); continue; }
        };
        let muted = compute_mute_all_override(&content);
        match crate::sanction_handler::put_power_levels(state, room_id, &muted).await {
            Ok(()) => applied += 1,
            Err(e) => tracing::warn!(err = %e, room_id = %room_id, "mute_all: put_power_levels failed — skipping room"),
        }
    }
    Ok(applied)
}
```

**GOTCHA:** `get_power_levels`/`put_power_levels` are `private` in `sanction_handler.rs` today — Task 2 widens them to `pub(crate)` (body unchanged) so `mute_handler` reuses the single canonical Matrix-power-level URL/Bearer shape (no duplication). This is the ONLY edit to `sanction_handler.rs`.

### 10.4 `Stage::mute_all` — local LiveKit sweep + `room_mute_all` emission (the negative invariant)

**Mirror:** `services/bridge/src/stage.rs:202-271` (`chair_override` EmitIntent push) + `:276-311` (`transfer_chair` `actor_pseudonym` shape).

```rust
// services/bridge/src/stage.rs
/// Emergency mute-all: revoke publish for EVERY locally-known publisher
/// (instant in-instance effect — option (b)) and push the room_mute_all
/// EmitIntent. The cross-instance authority is mute_handler::mute_all_power_levels
/// (Matrix power-levels); this is the LOCAL belt-and-suspenders.
///
/// `publishers` is the explicit locally-known publisher set (the caller enumerates
/// it from LiveKit room state). ADR-015: every string is a pseudonym.
/// Zero-holder invariant: after this returns, NO listed publisher retains a grant.
pub fn mute_all(&mut self, publishers: &[String], federated: bool, sink: &mut dyn GrantSink) {
    for p in publishers {
        sink.apply(GrantCmd::RevokePublish(p.clone()));   // the load-bearing sweep
    }
    self.current = None;
    self.pending_emits.push(EmitIntent {
        entry_kind: "room_mute_all",
        payload: RoomEventPayload {
            case_id: self.case_id as i32,
            matrix_room_id: None,
            lifecycle_stage: self.room_type.clone(),
            member_count: None,
            action: None, target_pseudonym: None,
            from_pseudonym: None, to_pseudonym: None, at: None,
            federated: Some(federated),
        },
        actor_pseudonym: self.chair.clone(),   // chair_pseudonym (ADR-015 pin)
    });
}
```

**The marquee test (§16a Story 1) — the cr-4 zero-holder negative invariant:** enumerate `["P1","P2","P3","P4"]`, call `mute_all`, assert the `Recorder` `GrantSink` recorded `RevokePublish` for **EVERY** one of the four (a `for`/set-equality assertion, not "≥1 revoke fired"). **Mechanical delete-the-revoke check:** if the `for p in publishers { sink.apply(RevokePublish) }` loop is deleted, the test sees zero revokes and FAILS — proving it asserts the invariant, not the happy path. Taking the explicit `publishers` slice (not deriving from `self.current`, which is single-presenter) is what makes the test exercise N>1 — the exact failure mode cr-4 named.

### 10.5 Bridge `RoomEventPayload.federated` mirror + `room_mute_all` JSON-shape test

**Mirror:** `services/bridge/src/room_event_client.rs:5-23` (the Serialize mirror) + `:85-142` (the JSON-shape test idiom).

Add `#[serde(skip_serializing_if = "Option::is_none")] pub federated: Option<bool>` to the bridge `RoomEventPayload`. Every existing constructor literal of this struct then requires `federated: None` (enumerated in §11). New `room_mute_all` test asserts the serialised `RoomEventRequest` has `entry_kind: "room_mute_all"`, `payload.federated == true`, `actor_pseudonym` = a pseudonym string, and OMITS the chair-action fields (`action`/`from_pseudonym`/`to_pseudonym` absent via `skip_serializing_if`).

### 10.6 ADR-015 pseudonym pin (LOAD-BEARING)

The `room_mute_all` actor — `EmitIntent.actor_pseudonym = self.chair.clone()` — is **a pseudonym** by the `stage.rs` contract (`:55` "all stored strings are pseudonyms, opaque, never person_id or username"). **Why it can't be deferred:** a real identity in a hash-chained entry is permanent (append-only, unscrubable) — a leak breaks `always_pseudonym` for the whole M3 cluster, unrecoverably. **DoD (§15.5):** the JSON-shape test asserts `actor_pseudonym` is the chair pseudonym (never numeric `person_id`/`@user:domain`); `rg -i 'person_id|username|@.*:' services/bridge/src/mute_handler.rs services/bridge/src/stage.rs` returns nothing identity-shaped (only pseudonym strings + the `m.call.member`/`org.matrix.msc3401.call.member` Matrix event keys, which are not identities).

### 10.7 ADR-016 metadata-only

`mute_all`'s EmitIntent payload carries only `{ case_id, lifecycle_stage, federated }` + the top-level `actor_pseudonym` — mute METADATA, never speech/video/Q&A-text bytes. `mute_handler` touches only `m.room.power_levels` state (a Matrix permission object), never room content. No content is hashed or POSTed.

## 11. Files to change

**`crates/api` (lemmy_api), Windows-validated:**
- `crates/api/api/src/governance/governance_log.rs` — one optional `federated: Option<bool>` field on `RoomEventPayload` + extend the `#[cfg(test)]` roundtrip test with a `room_mute_all` case (Task 1)

**`services/bridge` (brehon-bridge, workspace-excluded), Linux-validated:**
- `services/bridge/src/mute_handler.rs` — NEW: `compute_mute_all_override` (pure) + `mute_all_power_levels` (async, per-room GET→mutate→PUT) (Task 2)
- `services/bridge/src/sanction_handler.rs` — widen `get_power_levels` + `put_power_levels` from private to `pub(crate)` (bodies unchanged) so `mute_handler` reuses them (Task 2)
- `services/bridge/src/main.rs` — `mod mute_handler;` (alphabetical) (Task 2)
- `services/bridge/src/stage.rs` — `Stage::mute_all(publishers, federated, sink)` + the deterministic negative-invariant test; add `federated: None` to the 3 existing `RoomEventPayload` literals (Task 3)
- `services/bridge/src/room_event_client.rs` — `federated: Option<bool>` mirror field; add `federated: None` to the 2 existing test literals; add the `room_mute_all` JSON-shape test (Task 3)

**Tests (brehon-bridge integration), Linux-validated:**
- `services/bridge/tests/emergency_mute.rs` — NEW: docker-gated `#[ignore]` cross-instance end-to-end (Task 4)

### Struct-field add: enumerate all callsites (mandatory)

- **`RoomEventPayload` (binary, Task 1):** `rg "RoomEventPayload\s*\{" crates/` returns **only** the two `#[cfg(test)]` literals in `governance_log.rs:119-128,138-148` (the binary never constructs it in non-test code — it deserialises the bridge POST + serialises into `append`). New field is `Option` + `#[serde(default)]` → non-breaking; the two test literals get `federated: None` (or a `Some(true)` in the new mute case), same file/task.
- **`RoomEventPayload` (bridge mirror, Task 3):** `rg "RoomEventPayload\s*\{" services/bridge/` returns **5** literals — `stage.rs:219` (override force_demote), `:255` (override force_promote), `:297` (transfer), `room_event_client.rs:89` + `:123` (the two JSON-shape tests). Adding `federated: Option<bool>` to the bridge struct (no `Default`) requires **`federated: None`** on all 5; the new `mute_all` literal (`stage.rs`) sets `federated: Some(...)`. Task 3 `modifies:` covers `stage.rs` + `room_event_client.rs` — both crate-local, both in the SAME commit (else `brehon-bridge` fails to compile). Caller note tagged "Caller files (compiles-only-after-Task-3)" below.
- **`compute_mute_all_override` / `mute_all_power_levels` (Task 2):** new symbols, zero existing callers; reachable only from the Task-4 `#[ignore]` test until Phase-6 wires the live trigger (hence `#[allow(dead_code)]` on `mute_all_power_levels`).

> **Task 3 note (Caller files — compiles-only-after-Task-3):** `stage.rs` (3 literals) and `room_event_client.rs` (2 test literals) BOTH construct the bridge `RoomEventPayload`; both MUST get `federated: None` in Task 3's single commit, else `brehon-bridge` fails to compile. Both are in Task 3's `modifies:`. 2 files, 1 crate — within the ≤4/≤2 Sonnet ceiling.

## 12. NOT building in m3-core-emergency-mute

- **Live HTTP trigger for the chair's "mute all" click** — deferred to **Phase 6** (the live stage-action endpoints). Phase 4 lands the sync `Stage::mute_all` (LiveKit sweep + EmitIntent) + the async `mute_all_power_levels`; the EmitIntent is drained by the existing `drain_emits`, and `mute_all_power_levels` carries `#[allow(dead_code)]` until the Phase-6 endpoint calls it (the stage-mode dead_code-scaffold flow). Reason: the live endpoint is pilot-phase work; wiring it now expands into the Phase-6 trigger surface.
- **Un-mute / re-grant publish** — out of scope (`messaging.md:188`: "Unmute is by re-granting publish permissions individually — no 'unmute all', deliberate friction"). Phase 4 mutes; the existing `promote_next`/grant path re-grants individually. No `unmute_all` is built.
- **Per-publisher power-level enumeration** — the cross-instance mute RAISES the publish threshold (O(1), one PUT, federation-propagated), it does NOT enumerate each publisher's `users[mxid]` (that is the per-user `sanction_handler` shape, wrong for mute-all). The chair stays publishing via their existing room-admin level.
- **Re-registering `ENTRY_KIND_ROOM_MUTE_ALL` / bumping the registry count** — SHIPPED (Phase 2; const at `governance_log.rs:255`, in `ROOM_KINDS` at the shim `:174`, count 72). Phase 4 EMITS via `append_room_event`, never re-declares (handover tripwire → STOP).
- **A new `bridge_room` column or any migration** — Phase 4 USES `lookup_by_case` + the existing RTC columns; a new column/migration is a scope violation (handover tripwire → STOP).
- **A new bridge dependency** — reuses `reqwest`, `serde_json`, `rusqlite`, `tokio`, `anyhow`, `time` (all present). If a task needs a new dep, the regenerated `Cargo.lock` lands in the SAME commit (R8-adjacent watchpoint).
- **Anonymous-town-hall pseudonym-overlay work** — Phase 6 / inherited; the identity→pseudonym mapping is at JWT-issue time (m3-core-infra). Phase 4 only consumes pseudonyms the Stage already holds.

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Task 0 is non-`[P]` (barrier).

> **Cohort note:** Task 1 (`crates/api`, Windows) and Task 2 (`services/bridge` `mute_handler`, Linux) are file-disjoint across different crates and different validation runners → genuine `[P]` (Cohort A). Bridge Tasks 3–4 share the bridge crate + cold Linux builds, so they run **serial** (Task 3 → 4) even though file-disjoint from Task 2, to avoid concurrent cold-build / `Cargo.lock` contention (the stage-mode bridge-serial precedent). Per the cross-lane cap, at most 2 Junior workers run concurrently.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment + branch + base state before Task 1.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — Docker daemon (cargo-linux.sh Linux build + bridge docker-gated tests)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — Docker in LINUX-container mode (cargo-linux.sh requires it)
docker info --format '{{.OSType}}'   # EXPECT: linux

# Probe 2 — on the phase branch
git branch --show-current            # EXPECT: phase-m3-core-emergency-mute

# Probe 3 — room_mute_all const is SHIPPED on base (Phase 4 must NOT re-declare)
rg -c '^pub const ENTRY_KIND_ROOM_MUTE_ALL' \
  crates/db_schema/src/source/governance/governance_log.rs   # EXPECT: 1

# Probe 4 — ROOM_KINDS runtime allowlist already admits room_mute_all (emit path ready)
rg -c 'ENTRY_KIND_ROOM_MUTE_ALL' crates/api/api/src/governance/governance_log.rs  # EXPECT: >=2 (const ref in ROOM_KINDS)

# Probe 5 — entry-kind registry count is UNCHANGED at 72 (Phase 4 adds NO const)
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs  # EXPECT: 72

# Probe 6 — the Matrix power-level machinery to MIRROR is present
rg -c 'async fn get_power_levels|async fn put_power_levels|fn compute_power_override' \
  services/bridge/src/sanction_handler.rs                                          # EXPECT: 3
rg -c 'pub fn lookup_by_case' services/bridge/src/bridge_room.rs                    # EXPECT: 1

# Probe 7 — the emit seam is present (Phase 4 reuses, never rebuilds)
rg -c 'pub struct EmitIntent|pub pending_emits' services/bridge/src/stage.rs        # EXPECT: 2
rg -c 'pub async fn post_room_event|pub async fn drain_emits' services/bridge/src/room_event_client.rs  # EXPECT: 2

# Probe 8 — bridge baseline COMPILES on Linux (cold ~10-20 min; warm after)
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-mute-task0-bridge-check.log 2>&1
echo "bridge baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task0-bridge-check.log  # EXPECT: exit 0

# Probe 9 — crates baseline compiles (Windows)
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-mute-task0-check.log 2>&1"
echo "crates baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task0-check.log  # EXPECT: exit 0

# Probe 10 (negative) — wrapper propagates failure
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m3-mute-task0-neg.log 2>&1"
echo "negative exit: $?"   # EXPECT: NON-ZERO
```

**EXPECT:** Probes 0–9 succeed per their inline expectations; Probe 10 exits non-zero. **No commit at Task 0.**

### Task 1 [P]: Binary `RoomEventPayload.federated` field

**ACTION:** add one optional `federated: Option<bool>` field to the binary room-event DTO so the `room_mute_all` chain entry can carry the Success-Criteria `federated` metadata.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/governance_log.rs   # federated: Option<bool> on RoomEventPayload + #[cfg(test)] room_mute_all case
```

**IMPLEMENT:** `governance_log.rs` — add `#[serde(default, skip_serializing_if = "Option::is_none")] pub federated: Option<bool>` to `RoomEventPayload` per §10.1 (place after the chair-action fields). Extend the existing `#[cfg(test)]` test (`:116-157`) with a `room_mute_all` case: build a payload `{ case_id, lifecycle_stage: "room_mute_all", federated: Some(true), .. all chair fields None }`, `serde_json::to_value`, assert `v.get("federated").is_some()` and (`v["federated"] == true`) and assert the chair-action fields are OMITTED; also assert that a payload with `federated: None` OMITS `federated` (skip_serializing_if).

**MIRROR:** §10.1; `crates/api/api/src/governance/governance_log.rs:91-158` (struct + existing test).

**GOTCHA:** `skip_serializing_if = "Option::is_none"` is load-bearing — without it, existing room emissions gain `"federated":null`, changing hashed JSON. **No non-test `RoomEventPayload { … }` constructor exists** in `crates/` (grep returns only the two test literals), so the `Option` field breaks nothing. `room_mute_all` is already in `ROOM_KINDS` (`:174`) — do NOT touch `ROOM_KINDS` or any const. ADR-015: this DTO carries pseudonyms (via `actor_pseudonym`, a separate arg); ADR-016: metadata only.

**VALIDATE (Windows):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-mute-task1-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task1-check.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop` DQ with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""]` (`e2e_filter: null` — the `governance_log` unit test runs under check); commit + push, then **stop**. (No `-linux` DQ — Task 1 touches no bridge/Cargo.toml/migration/cfg code.)

### Task 2 [P]: Bridge cross-instance mute power-level path (`mute_handler.rs`)

**ACTION:** add `services/bridge/src/mute_handler.rs` (the `compute_mute_all_override` pure fn + the async per-room `mute_all_power_levels` GET→mutate→PUT), widen `sanction_handler`'s power-level GET/PUT to `pub(crate)`, and register the module.

**FILES:**

```yaml
creates:
  - services/bridge/src/mute_handler.rs
modifies:
  - services/bridge/src/sanction_handler.rs   # get_power_levels + put_power_levels: private -> pub(crate) (bodies unchanged)
  - services/bridge/src/main.rs               # mod mute_handler; (alphabetical)
```

**IMPLEMENT (file 1 of 3):** `mute_handler.rs` per §10.2 + §10.3 — `compute_mute_all_override(content) -> serde_json::Value` (raise `events["m.call.member"]` + `events["org.matrix.msc3401.call.member"]` above `users_default`; always return the full merged content) + `#[allow(dead_code)] pub(crate) async fn mute_all_power_levels(state, case_id) -> Result<usize>` (open `bridge_room`, `lookup_by_case`, per-room `get_power_levels` → `compute_mute_all_override` → `put_power_levels`, best-effort skip-on-error, return count applied). Add `#[cfg(test)]` tests: (a) `compute_mute_all_override_cases` — feed `{users_default: 0, events: {}}`, assert returned `events["m.call.member"] == 1` (> users_default) and the MSC3401 alias too; feed `{users_default: 50}`, assert `== 51`; (b) the live GET/PUT path is the docker-gated Task-4 `#[ignore]` test (mirror `sanction_handler.rs:474-482`).
**IMPLEMENT (file 2 of 3):** `sanction_handler.rs` — change `async fn get_power_levels` (`:281`) and `async fn put_power_levels` (`:304`) to `pub(crate) async fn` (bodies unchanged). No other edit.
**IMPLEMENT (file 3 of 3):** `main.rs` — add `mod mute_handler;` (alphabetical, after `mod livekit_jwt;`/before `mod provision;` per the existing block `:12-25`).

**MIRROR:** §10.2, §10.3; `services/bridge/src/sanction_handler.rs:188-228` (per-room loop), `:250-277` (compute test shape), `:281-325` (GET/PUT), `:437-472` (compute test); `services/bridge/src/bridge_room.rs:59` (`lookup_by_case`).

**GOTCHA (R11 — power-levels are the cross-instance authority):** the mute reaches cross-instance publishers via the Matrix power-level PUT (one PUT federates), NOT via LiveKit grants. Do NOT call `mint_access_token` here. The chair retains publish via the room-admin power-level seated at provisioning — `compute_mute_all_override` raises the BAR, it does not touch `users[chair]`. `mute_all_power_levels` is `#[allow(dead_code)]` until Phase-6 wires the live trigger (reachable from the Task-4 `#[ignore]` test). Best-effort per-room (one room's failure must not abort others — mirror the sanction loop). Bridge compiles on **Linux only**.

**VALIDATE (Linux):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-mute-task2-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task2-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml mute_handler \
  > .claude/PRPs/debug/m3-mute-task2-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task2-bridge-test.log   # EXPECT: exit 0 (compute_mute_all_override test passes)
```

Write a `validate-pending-laptop-linux` DQ (`commands` = `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test … mute_handler`), commit + push, then **stop**.

### Task 3: `Stage::mute_all` (local LiveKit sweep + `room_mute_all` emission) + bridge `federated` mirror

**ACTION:** add `Stage::mute_all` (the LiveKit `RevokePublish` sweep + the `room_mute_all` EmitIntent) and the `federated` mirror field on the bridge `RoomEventPayload`, updating the 5 existing literals.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/stage.rs              # Stage::mute_all + negative-invariant test; federated: None on 3 existing RoomEventPayload literals
  - services/bridge/src/room_event_client.rs  # federated: Option<bool> mirror field; federated: None on 2 test literals; room_mute_all JSON-shape test
requires:
  - task: 1
    reason: the bridge room_mute_all POST carries `federated`; the binary RoomEventPayload must have the `#[serde(default)] federated` field (Task 1) for the round-trip to deserialise
```

**IMPLEMENT (file 1 of 2):** `room_event_client.rs` — add `#[serde(skip_serializing_if = "Option::is_none")] pub federated: Option<bool>` to the bridge `RoomEventPayload` (§10.5); add `federated: None` to the 2 existing test literals (`:89`, `:123`); add a `mute_all_request_json_shape` test: build a `RoomEventRequest { entry_kind: "room_mute_all", payload: { …, federated: Some(true) }, actor_pseudonym: Some("chair-pseudonym") }`, `to_value`, assert `entry_kind == "room_mute_all"`, `payload.federated == true`, `actor_pseudonym == "chair-pseudonym"`, and that `action`/`from_pseudonym`/`to_pseudonym` are ABSENT (skip_serializing_if).
**IMPLEMENT (file 2 of 2):** `stage.rs` — add `federated: None` to the 3 existing `RoomEventPayload` literals (`:219`, `:255`, `:297`); add `pub fn mute_all(&mut self, publishers: &[String], federated: bool, sink: &mut dyn GrantSink)` per §10.4 (revoke EVERY listed publisher, clear `current`, push the `room_mute_all` EmitIntent with `actor_pseudonym = self.chair.clone()`, `federated: Some(federated)`). **Negative-invariant unit test** `mute_all_revokes_all_publishers` (§16a Story 1): seat a chair, `mute_all(&["P1","P2","P3","P4"], true, &mut recorder)`, collect all `RevokePublish` pseudonyms from the `Recorder`, assert the set equals `{P1,P2,P3,P4}` (every publisher revoked), assert exactly one `room_mute_all` EmitIntent with `payload.federated == Some(true)` + `actor_pseudonym == Some(chair)`. Add a second assertion comment documenting the **delete-the-revoke check** (deleting the sweep loop makes the set-equality assert fail).

**MIRROR:** §10.4, §10.5; `services/bridge/src/stage.rs:202-311` (EmitIntent push + actor_pseudonym), `:321-632` (the `Recorder` `GrantSink` + negative-assertion test idioms); `services/bridge/src/room_event_client.rs:85-142` (JSON-shape test).

**GOTCHA (R7 — the zero-holder negative invariant):** `mute_all` takes the EXPLICIT `publishers` slice (NOT derived from `self.current`, which is single-presenter) so the test exercises N>1 — the cr-4 failure mode. The test asserts EVERY publisher revoked (set equality), not "a revoke fired"; it MUST fail if the revoke loop is deleted. **GOTCHA (ADR-015):** `actor_pseudonym = self.chair` is a pseudonym (`stage.rs` contract); `rg -i 'person_id|username|@.*:' services/bridge/src/stage.rs` returns nothing identity-shaped. **GOTCHA (Cargo-lock):** adding `federated` to the bridge struct requires `federated: None` on ALL 5 existing literals in the SAME commit or `brehon-bridge` fails to compile. Bridge compiles on **Linux only**.

**VALIDATE (Linux; feeds §16a Stories 1+2):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-mute-task3-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task3-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml stage::mute_all \
  > .claude/PRPs/debug/m3-mute-task3-bridge-stage-test.log 2>&1
echo "stage test exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task3-bridge-stage-test.log   # EXPECT: exit 0 (mute_all_revokes_all_publishers passes)
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml room_event_client \
  > .claude/PRPs/debug/m3-mute-task3-bridge-client-test.log 2>&1
echo "client test exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task3-bridge-client-test.log   # EXPECT: exit 0 (mute_all JSON-shape passes)
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test … stage` + `test … room_event_client`), commit + push, then **stop**.

### Task 4: Docker-gated cross-instance end-to-end test (`emergency_mute.rs`)

**ACTION:** add the docker-gated `#[ignore]` integration test that carries the real cross-instance <500ms-at-the-publisher-client signal + the `room_mute_all` chain-row assertion.

**FILES:**

```yaml
creates:
  - services/bridge/tests/emergency_mute.rs
modifies: []
requires:
  - task: 2
    reason: the test exercises mute_all_power_levels (the cross-instance Matrix power-level path)
  - task: 3
    reason: the test exercises Stage::mute_all (the local sweep + room_mute_all emission)
```

**IMPLEMENT:** `emergency_mute.rs` per §10 / mirror `services/bridge/tests/stage_mode.rs:1-23` — one `#[tokio::test] #[ignore = "requires docker-compose stack"]` fn `mute_all_drops_all_publishers_cross_instance_under_500ms` with step comments: (1) provision a federated town-hall stage room across two bridge instances (chair + N publishers); (2) record a per-publisher publish-active baseline at the publisher client; (3) fire mute-all (`mute_all_power_levels` PUT + the local `Stage::mute_all` sweep); (4) assert EVERY non-chair publisher's publish right is revoked **measured at the publisher client** within **500ms** (the zero-holder negative invariant, cross-instance); (5) query `governance_log`: assert a `room_mute_all` row with `actor_pseudonym` = the chair PSEUDONYM (never `person_id`/`@user:domain`) + `payload.federated == true`. `todo!()`-stub the body (pilot-grade; the live stack is Phase-6). The deterministic gate (Stories 1–3) runs under `cargo-linux.sh test` without docker; this test is the Phase-6-pilot-grade cross-instance signal.

**MIRROR:** `services/bridge/tests/stage_mode.rs:1-23` (ignore-stub shape); `services/bridge/src/sanction_handler.rs:474-482` (the `#[ignore]` live-test convention).

**GOTCHA:** the `#[ignore]` test does NOT run under `cargo-linux.sh test` (it needs the live two-instance docker stack); the DoD is that it **compiles** (`--no-run`). The measurement is at the **publisher client**, not the server (Success Criteria line 141 + PRD line 198). If cross-instance <500ms proves infeasible at the Phase-6 pilot, the step comments document the PRD fallback (best-effort cross-instance SLA + in-instance <500ms guarantee via the `Stage::mute_all` LiveKit sweep) — a DQ surfaces before declaring the criterion failed; it is NOT silently dropped. Bridge compiles on **Linux only**.

**VALIDATE (Linux):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-mute-task4-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task4-bridge-check.log   # EXPECT: exit 0
# emergency_mute.rs is #[ignore]'d — assert it COMPILES (not runs) via --no-run:
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml --test emergency_mute --no-run \
  > .claude/PRPs/debug/m3-mute-task4-bridge-itc.log 2>&1
echo "test-compile exit: $?"; tail -20 .claude/PRPs/debug/m3-mute-task4-bridge-itc.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test --test emergency_mute --no-run`), commit + push, then **stop**.

### Task 5: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + lessons + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lessons to `.claude/lessons/feedback_*.md` in the retro commit. Specific signals: did the option-(b) two-surface mechanism (power-levels cross-instance + LiveKit local sweep) survive CR review?; did the cr-4 zero-holder negative-invariant test pattern hold (did the marquee test break when the revoke was deleted — was the delete-the-revoke check actually exercised)?; did the `sanction_handler` `pub(crate)` widening introduce any clippy/visibility friction?; did the `federated`-field propagation across 5 bridge literals + 2 binary literals compile first-try?

---

## 14. Testing strategy

- **crates static (Windows):** `cargo check --workspace --features full`; `cargo clippy --workspace --features full --no-deps -- -D warnings` (Task 1).
- **crates unit (Windows):** the `governance_log.rs` `#[cfg(test)]` `room_mute_all` serde case (Task 1) runs under check/test.
- **bridge static (Linux):** `cargo-linux.sh check/clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings` (Tasks 2–4).
- **bridge unit (Linux, no docker):** `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml` — `mute_handler` (`compute_mute_all_override`), `stage` (the **zero-holder negative-invariant** `mute_all_revokes_all_publishers`), `room_event_client` (the `room_mute_all` JSON shape). **These ARE the marquee DoD** (deterministic, exercise the real revoke sweep + emission).
- **bridge integration (Linux, docker-gated, `#[ignore]`):** `emergency_mute.rs` — compiles under `--no-run` in CI; runs only against a live two-instance docker stack (Phase-6 pilot grade; the real cross-instance <500ms measurement).
- **No migration round-trip** (no new migration). **No deploy-smoke** (no new sidecar — reuses m3-core-infra's `profiles: ["rtc"]` stack).

## 15. Validation commands (DoD)

> Every command below is written in the exact form the advisor runs at gate 1; all dry-run clean against `phase-m3-core-emergency-mute` HEAD. Bridge cargo uses `cargo-linux.sh --manifest-path` (NEVER Windows-local — `ruma-common` E0119). No `rg`-absent / line-count greps.

### 15.1 crates static analysis (Task 1 — Windows)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-mute-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.2 crates lint (Task 1 — Windows, uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/m3-mute-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.3 bridge static + lint + unit (Tasks 2–4 — Linux, Docker rust:1.95)

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-mute-bridge-check.log 2>&1;  echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings > .claude/PRPs/debug/m3-mute-bridge-clippy.log 2>&1; echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-mute-bridge-test.log 2>&1; echo "exit: $?"  # EXPECT: 0 (mute_handler + stage::mute_all + room_event_client unit tests pass; #[ignore] integration skipped)
```

### 15.4 bridge integration-test compile (Task 4 — Linux; #[ignore] body not run)

```bash
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute --no-run > .claude/PRPs/debug/m3-mute-bridge-itc.log 2>&1
echo "exit: $?"   # EXPECT: 0 (the docker-gated test compiles; live cross-instance run is Phase-6 pilot)
```

### 15.5 Cross-cutting verification (planner asserts at end-of-phase)

- [ ] R8: NO new migration / no new `bridge_room` column; `git diff --stat governance-v0..HEAD -- migrations/ crates/db_schema/migrations/` is empty.
- [ ] No new const / no registry bump: `git diff governance-v0..HEAD -- crates/db_schema/src/source/governance/governance_log.rs` is EMPTY (Phase 4 adds no const); `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **72** (unchanged).
- [ ] FIRST emitter: `rg 'room_mute_all' services/bridge/src/stage.rs` returns the EmitIntent push site (a `pending_emits.push` with `entry_kind: "room_mute_all"`), NOT a new POST client.
- [ ] Reuse the emit seam: `rg -c 'post_room_event|drain_emits' services/bridge/src/room_event_client.rs` shows the EXISTING client is reused — no new bridge→binary POST client was built.
- [ ] R11 (power-levels = cross-instance authority): `rg 'put_power_levels' services/bridge/src/mute_handler.rs` present; `rg -i 'mint_access_token|power.?level' services/bridge/src/mute_handler.rs` shows power-level usage and NO `mint_access_token` (the cross-instance path is Matrix power-levels, not LiveKit grants).
- [ ] R9 (ADR-015): `rg -i 'person_id|username|@.*:' services/bridge/src/mute_handler.rs services/bridge/src/stage.rs` returns nothing identity-shaped (only pseudonym strings + the Matrix `m.call.member`/`org.matrix.msc3401.call.member` event keys).
- [ ] R10 (ADR-016): the `room_mute_all` EmitIntent payload carries only `{case_id, lifecycle_stage, federated}` + top-level `actor_pseudonym` — no speech/video/Q&A bytes; `mute_handler` touches only `m.room.power_levels`.
- [ ] Negative invariant: the `stage::mute_all_revokes_all_publishers` test asserts EVERY publisher revoked (set equality) and FAILS if the revoke sweep is deleted (the delete-the-revoke mechanical check).
- [ ] No new dep: `git diff governance-v0..HEAD -- services/bridge/Cargo.toml services/bridge/Cargo.lock` is empty (or, if non-empty, BOTH change in the same commit).

## 16. Acceptance criteria

- [ ] Tasks 0–5 completed in dependency order
- [ ] §15.1/15.2 (crates check + clippy) exit 0 (Task 1)
- [ ] §15.3 (bridge check + clippy + unit test) exit 0 (Tasks 2–4)
- [ ] §15.4 (integration-test compiles `--no-run`) exit 0 (Task 4)
- [ ] §15.5 cross-cutting boxes all ticked
- [ ] §16a Stories 1–4 all `[done]`
- [ ] No edits outside §11; entry-kind registry + consts untouched (count stays 72); no migration
- [ ] Retro committed (Task 5)
- [ ] **Linux-compile gate:** `validate-pending-laptop-linux` DQ at `result:pass` (Tasks 2–4 touch `services/bridge/**`) before `bm-pr`
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

## 16a. Stories

### Story 1: Emergency mute-all drops EVERY non-chair publisher — the zero-holder negative invariant (marquee DoD)

- **Composing tasks:** Task 3 (`Stage::mute_all` sweep + emission)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage::mute_all_revokes_all_publishers`
- **Expected output:** test passes — for `publishers = [P1,P2,P3,P4]`, the `Recorder` `GrantSink` recorded `RevokePublish` for **every** one (set equality), `current` cleared, exactly one `room_mute_all` EmitIntent. **Mechanical check (must hold):** deleting the `for p in publishers { sink.apply(RevokePublish) }` loop makes this test FAIL (it asserts the invariant, not the happy path).
- **Brief-Scope outputs to verify:** `services/bridge/src/stage.rs` contains `pub fn mute_all` revoking every listed publisher + pushing the `room_mute_all` EmitIntent.

### Story 2: `room_mute_all` emits with `chair_pseudonym` (actor) + `federated: true`, metadata only

- **Composing tasks:** Task 1 (binary `federated` field), Task 3 (the EmitIntent + bridge mirror)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml room_event_client::mute_all_request_json_shape`
- **Expected output:** test passes — the serialised `RoomEventRequest` has `entry_kind: "room_mute_all"`, `payload.federated == true`, `actor_pseudonym` = a pseudonym string (never `person_id`/`@user:domain`), and OMITS the chair-action fields (`action`/`from_pseudonym`/`to_pseudonym`). The binary `governance_log.rs` `#[cfg(test)]` `room_mute_all` case (Task 1) asserts `federated` present + omitted-when-None.
- **Brief-Scope outputs to verify:** `room_event_client.rs` `RoomEventPayload` has `federated`; binary `governance_log.rs` `RoomEventPayload` has `federated`; `stage.rs::mute_all` sets `actor_pseudonym = self.chair`.

### Story 3: The power-level mute raises the publish threshold for a federated room (cross-instance authority)

- **Composing tasks:** Task 2 (`mute_handler`)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml mute_handler::compute_mute_all_override_cases`
- **Expected output:** test passes — `compute_mute_all_override` raises `events["m.call.member"]` (+ the MSC3401 alias) ABOVE `users_default` (mutes all non-elevated), preserving the full content for a whole-object PUT.
- **Brief-Scope outputs to verify:** `services/bridge/src/mute_handler.rs` exists with `compute_mute_all_override` + `mute_all_power_levels` (the per-room `lookup_by_case` → GET → mutate → PUT, mirror `sanction_handler`); `sanction_handler.rs` exposes `get_power_levels`/`put_power_levels` as `pub(crate)`.

### Story 4: Cross-instance mute drops all publishers <500ms at the publisher client, `room_mute_all` lands on the chain (Phase-6-pilot-grade)

- **Composing tasks:** Task 4 (docker-gated integration test), Tasks 2/3
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test emergency_mute --no-run` (compile-gate; live run is Phase-6 pilot)
- **Expected output:** the docker-gated `emergency_mute.rs` test compiles; its step comments assert all publishers dropped <500ms at the publisher client across two federated instances + a `room_mute_all` chain row with chair pseudonym + `federated: true`.
- **Brief-Scope outputs to verify:** `services/bridge/tests/emergency_mute.rs` exists with the `#[ignore]` cross-instance scenario asserting the zero-holder invariant + pseudonym + `federated` payload.

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Story's checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent/empty) trigger the catch-fire procedure.

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–10 per expectations)
- [ ] Tasks 1–4 committed (one commit each)
- [ ] §15 validation green at every gate (crates Windows / bridge Linux unit / integration-compile)
- [ ] §16a Stories 1–4 all `[done]`
- [ ] Retro committed (Task 5)
- [ ] PR opened by BM against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report shows all stories ✓
- [ ] **Linux-compile gate:** `validate-pending-laptop-linux` DQ at `result:pass` (Tasks 2–4) before `bm-pr`
- [ ] Post-merge phase branch retained for retro reads

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Marquee test asserts a single mute command, not the zero-holder invariant (the cr-4 trap) | MED | HIGH | §16a Story 1 + R7 + Task 3 GOTCHA: `mute_all` takes an EXPLICIT N-publisher slice; the test asserts set-equality of revokes + the delete-the-revoke mechanical check; advisor §3.5 rejects a story that asserts only "a mute fired" |
| Cross-instance <500ms infeasible (Matrix power-level federation latency) | HIGH | MED | Option (b): the LiveKit `RevokePublish` sweep guarantees in-instance <500ms deterministically; the power-level PUT is the cross-instance best-effort authority; PRD line-198 fallback (documented SLA + DQ before declaring failed) encoded in Task 4 GOTCHA |
| Mute path conflates power-levels with LiveKit grants for the cross-instance reach (mechanism error) | LOW | HIGH | R11 + §15.5 grep: `mute_handler.rs` issues `put_power_levels`, NO `mint_access_token`; the LiveKit sweep is explicitly LOCAL-only; bootstrap §7 catch-fire on a GrantSink-only cross-instance path |
| ADR-015 leak — a `person_id`/MXID reaches the `room_mute_all` chain entry | LOW | HIGH | `actor_pseudonym = self.chair` (a pseudonym by the `stage.rs` contract); §15.5 grep DoD; JSON-shape test asserts pseudonym shape; ADR-016 metadata-only |
| `federated` propagation leaves `brehon-bridge` non-compiling (5 literals) | LOW | MED | §11 caller note: all 5 bridge literals + the 2 binary literals get `federated: None` in their owning task's single commit; §15.1/15.3 catch it |
| Plan re-registers `room_mute_all` or bumps the registry count (scope error) | LOW | HIGH | §12 + Task 0 Probe 5 (count stays 72) + Probe 3 (const present on base); §15.5 asserts `db_schema/governance_log.rs` diff empty; handover tripwire → STOP |
| `pub(crate)` widening of `sanction_handler` GET/PUT introduces clippy/visibility churn | LOW | LOW | bodies unchanged; the only edit is the `async fn` → `pub(crate) async fn` keyword; §15.3 clippy `--no-deps` catches any issue |
| Bridge Linux cold build (~10–20 min) stalls serial Tasks 3–4 | MED | LOW | Task 0 Probe 8 warms the registry volume; bridge tasks serial avoids concurrent cold builds; no Cargo.lock change |

## 19. Notes

**Two scope decisions, both pre-seeded as resolved planner DQs for advisor gate-1 ratification:**

**(1) Mechanism — option (b): Matrix power-levels (cross-instance authority) + LiveKit `RevokePublish` sweep (local instant). (resolved planner DQ `78c6ad4bfb41-001`.)** The PRD §Phase 4 (line 242) says "Matrix power-levels across instances"; `messaging.md:188` says "All non-chair LiveKit publishers are muted … including participants from federated instances … matching Matrix power-level semantics … Unmute is by re-granting publish permissions individually." These are NOT two competing mechanisms — they are the two halves of option (b): **power-levels = the cross-instance authority** (one PUT to `m.room.power_levels` federates via Matrix to all instances — OQ-V2-06 chair-authority-is-room-global, `99-...md:233`), and the **LiveKit grant** = the local instant effect + the unmute path (`messaging.md:188`'s "re-granting publish permissions"). Option (a) (power-levels only) would rely on federation propagation even for local effect — the <500ms risk would bite in-instance too. Option (b) guarantees the in-instance <500ms deterministically (the `Stage::mute_all` sweep) while keeping power-levels as the cross-instance authority (per OQ-V2-06; a GrantSink-only cross-instance path would be a mechanism error, bootstrap §7 catch-fire). **The cross-instance authority is power-levels** (`mute_handler.rs` `put_power_levels`, mirror `sanction_handler.rs:304`); the LiveKit sweep is the LOCAL belt-and-suspenders only. **Advisor: ratify option (b) at gate-1.** Resolved (planner): option (b), cites `prds/m3-town-halls-rtc.prd.md:242` + `V2/messaging.md:188` + `99-...md:233`.

**(2) `room_mute_all` payload = the existing top-level `actor_pseudonym` + ONE new `federated` field. (resolved planner DQ `78c6ad4bfb41-002`.)** Success Criteria line 141 writes the entry as `{chair_pseudonym, federated: true|false}`. **`chair_pseudonym` is NOT a new payload field** — it is the existing top-level `actor_pseudonym` (the `EmitIntent.actor_pseudonym` → `RoomEventRequest.actor_pseudonym` → `append_room_event(…, actor_pseudonym)` arg), exactly as `room_chair_override`/`room_chair_transferred` carry the acting chair via `actor_pseudonym` (the Success-Criteria lines 139–140 likewise name only the payload-specific fields, with the actor on the top-level field). Duplicating it as a payload field would create two sources of truth for the same value. The ONLY DTO change is adding `federated: Option<bool>` to `RoomEventPayload` (binary `governance_log.rs:91-110` + the bridge mirror `room_event_client.rs:6-23`), `#[serde(default, skip_serializing_if="Option::is_none")]` — the same trivial-DTO class stage-mode shipped (its 5 chair fields). The binary constructs zero non-test `RoomEventPayload {…}` literals (`rg` empty), so the add is non-breaking; `skip_serializing_if` keeps existing room emissions' hashed JSON byte-identical. **This is the one in-scope `crates/**` touch** — within the bootstrap tripwire's trivial-DTO carve-out (NOT a const/migration/registry change). **Advisor: ratify the trivial-DTO crates touch + the `chair_pseudonym = actor_pseudonym` mapping at gate-1.** Resolved (planner): add `federated` only; `chair_pseudonym` rides `actor_pseudonym`.

**Marquee DoD is a deterministic unit test, not the docker stack.** Per `feedback_build_what_tests_exercise` + `feedback_authz_state_machine_test_asserts_negative` + `pattern_test_against_reality_not_syntax`: the zero-holder negative invariant (mute-all revokes EVERY publisher) is the load-bearing logic, captured as a deterministic `cargo-linux.sh test` unit test over the real `Stage::mute_all` + a `Recorder` `GrantSink` (set-equality of revokes; the delete-the-revoke check breaks it). The cross-instance LiveKit + real-`append()` + <500ms-at-the-publisher-client flow is the docker-gated `#[ignore]` `emergency_mute.rs` (Phase-6 pilot grade). This split is honest: the deterministic test proves the invariant; the pilot proves the cross-instance latency.

**No new migration, no new sidecar, no new dependency, no new const.** Phase 4 composes m3-core-entry-kinds' `room_mute_all` const, stage-mode's emit seam (`EmitIntent`/`pending_emits`/`drain_emits`/`post_room_event`), and m2-late-2's Matrix power-level machinery (`sanction_handler` GET/PUT). Complexity 2/10. If any task is tempted to add a const, a column, a dep, or a new emitter, STOP (handover tripwire / bootstrap §7 catch-fire).

## 20. Confidence score

- **Plan correctness:** 8/10 — the mechanism decision (option (b)) reconciles the PRD + messaging.md cites; both surfaces are fully reconnoitred (`sanction_handler` power-level machinery exists; the emit seam exists; the const exists). The one judgment call is the deterministic-unit-test-vs-docker split for the cross-instance <500ms (defended above + PRD line-198 fallback).
- **Cargo budget:** 9/10 — no daemon cargo; bridge cold-build is the only time cost, mitigated by Task-0 warm-up + serial bridge tasks.
- **Test coverage:** 8/10 — the zero-holder negative invariant (the cr-4 application site), the `room_mute_all` emission shape, and the power-level mutate are all §16a stories with green deterministic checkpoints; the live cross-instance <500ms is a compile-gated `#[ignore]` test deferred to the Phase-6 pilot by scope.
