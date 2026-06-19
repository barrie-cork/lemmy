# Plan: m3-core-recording — optional evidentiary-store recording, first `room_recording_uploaded` chain emission

## 1. Summary

This sub-phase gives the M3 town-hall bridge an **optional evidentiary-store recording** path: when `record_town_halls = true` for a town-hall room, the bridge triggers **LiveKit Egress** → an MP4 lands in a **generic-S3 object store** (MinIO reference backend) → the bridge computes the MP4's **`content_sha256`** → and pushes the **first emission** of the `room_recording_uploaded` hash-chain entry through the existing bridge→binary callback seam. **Headline acceptance:** a deterministic unit test fires the recording controller with the flag ON, asserts exactly one `room_recording_uploaded` EmitIntent carrying the Success-Criteria schema `{media_url, content_sha256, duration_s, speakers, attendance_count}` (speakers as pseudonyms, ADR-015) whose `content_sha256` rides `append_room_event` (never a bypass digest); a **`record_town_halls = false` clean-posture** deterministic test asserts the recording controller produces **ZERO** side-effects (no Egress trigger, no S3 PUT, no EmitIntent — the test breaks if the flag-gate is deleted); and a **participant-floor fetch** test asserts a non-participant request is rejected and a participant request accepted (ADR-015 floor — can't be zero under `always_pseudonym`). The `record_town_halls` flag is a knob in the **existing `bridge_room.recording_config TEXT` column** (m3-core-infra shipped it — no new migration). The `ENTRY_KIND_ROOM_RECORDING_UPLOADED` const is **already registered** (M2; in `ROOM_KINDS` at count 13, registry 72) — this phase **emits only**, via the existing `EmitIntent` → `pending_emits` → `drain_emits` → `post_room_event` → `append_room_event` seam. The **one** new bridge dependency is a generic-S3 client; the Egress trigger reuses the existing `reqwest` + `livekit_jwt`. The in-scope `crates/**` touch is a trivial optional-field DTO extension (5 recording-metadata fields on `RoomEventPayload`, mirroring emergency-mute's `federated` add). **No strict presigned-URL ACL / retention / tombstone / GDPR machinery (D5 Option C defers it); no cross-instance recording federation (instance-local, OQ-V2-07); no transcript.**

## 2. Source

- `.claude/PRPs/prds/m3-town-halls-rtc.prd.md` §"Phase 5: M3-core recording" (245–248), §Technical Approach (174–203, esp. recording-store-ops risk row 200 + `always_pseudonym` Egress-leak risk row 201), §Success Criteria recording rows (143 [recording lands + schema], 144 [fetch authz], 145 [clean-posture]), §Cross-Cutting Impact (155–158), §Users & Context success-state (168), §Decisions Log D4/D5/D6 @ `3987f975b`.
- `.claude/PRPs/briefs/m3-core-recording-planning-1.md` (the authorising brief) @ `3987f975b`.
- `.claude/PRPs/handovers/m3-core-recording-bootstrap.md` §1 (scope+DoD), §4 watchlist (6 items, #6 OQ-V2-04 now satisfied), §7 catch-fire, §"Stop-and-ask tripwires" @ `3987f975b`.
- `docs/brehon-law-inspired-network/99-decisions-and-open-questions.md` **OQ-V2-04 RESOLVED block** (line 655, the D5 = Option C decision: dedicated S3-compatible store, MinIO reference backend, generic S3 API for operator swap, evidentiary store with replay-grade access, participant-floor authz, strict ACL deferred, plane-separated from pict-rs, recording optional via `record_town_halls`, instance-local) @ `3987f975b`. **This is the canonical design source for Tasks 2–5.**
- `.claude/PRPs/reports/m3-recording-storage-considerations-2026-06-15.md` — the A/B/C option trade-offs behind OQ-V2-04 (Option C basis).
- `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` — the SIBLING bridge plan (canonical-schema-first gate, `feedback_read_canonical_before_writing_spec.md`): §5 complexity breakdown, §7 R1–R11 guardrails, §10 patterns, §15 three-surface DoD, §16a story shape (incl. the negative-invariant marquee + ADR-015-pin story). Immediately-preceding bridge plan; this plan matches its section conventions + its `EmitIntent`/`drain_emits` emission pattern + its trivial-DTO `federated` add (the canonical precedent for the recording-fields add).
- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — the plan that BUILT the bridge→binary callback client + the `EmitIntent`/`pending_emits`/`drain_emits` seam. The emit seam this phase REUSES (do not rebuild).
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` — canonical schema; `RoomEventPayload` shape + `ROOM_KINDS` array + the `POST /api/v4/governance/room-event` route contract; recording entry schema (doc-drift row 203 — CODE WINS).
- ADRs (`docs/brehon-law-inspired-network/99-decisions-and-open-questions.md`): **ADR-015** (pseudonymised actor IDs) — load-bearing for the participant-floor fetch authz + the `speakers`/`chair_pseudonym` payload pin; **ADR-016** (cross-app backplane; content never hashed) — only metadata + `content_sha256` reach the chain; **ADR-008** (append-only signed log) — the chain `content_sha256` rides via `append_room_event`; **ADR-004** (plane separation — MinIO ≠ content-plane pict-rs); **ADR-011** (AGPLv3 inherited — MinIO AGPL-3.0 + LiveKit Egress Apache-2.0 licence-clean).
- Lessons that bind decisions:
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); never Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — every bridge-touching task writes a `validate-pending-laptop-linux` DQ; `bm-pr` gates on `result:pass`. **Doubly load-bearing this phase** — Task 3 changes `services/bridge/Cargo.toml`/`Cargo.lock` (the S3 dep).
  - `feedback_authz_state_machine_test_asserts_negative.md` — the `record_town_halls = false` clean-posture test is a NEGATIVE invariant (no Egress / no S3 PUT / no EmitIntent); the test MUST fail if the flag-gate is deleted.
  - `feedback_build_what_tests_exercise.md` — the recording test must EXERCISE the real flow (an actual Egress trigger + S3 write + the real `room_recording_uploaded` EmitIntent drained through `append_room_event`), not assert on config/struct shape.
  - `pattern_test_against_reality_not_syntax.md` — assert on observable S3 object existence + the actual `room_recording_uploaded` chain entry row (with the real `content_sha256`), not config.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — the ADR-015 participant-floor + `speakers`-pseudonym pins (§4) are made LOAD-BEARING, not named.
  - `feedback_entry_kind_runtime_allowlist_check.md` — `room_recording_uploaded` is already in `ROOM_KINDS` (verified present at `governance_log.rs:227`, count 13); this phase emits, never re-registers.
  - `feedback_multi_write_handlers_need_transactions.md` — if the recording-callback handler does 2+ DB writes, use a transaction.
  - `feedback_governance_type_state_handlers.md` — the recording emission guards on room/recording state before appending (type-state pattern).
  - `feedback_plan_dod_dry_run_at_write.md` — every §15 command is written in the exact form the advisor runs at gate 1.
  - `feedback_validate_pending_laptop_write_then_stop.md` — workers write the `validate-pending-laptop[-linux]` DQ and STOP; the laptop advisor runs all cargo.

## 3. Problem statement

m3-core-infra shipped the RTC stack (`rtc_enabled`, LiveKit JWT minting, identity→pseudonym at issue, the `bridge_room.recording_config TEXT` column). m3-core-stage-mode shipped the chair-controlled stage + the `EmitIntent` → `pending_emits` → `drain_emits` → `post_room_event` callback seam. m3-core-entry-kinds registered `ENTRY_KIND_ROOM_RECORDING_UPLOADED` (`governance_log.rs:243`, in `ROOM_KINDS` at the api shim `:227`, count 13/registry 72). m3-core-emergency-mute shipped the `federated` trivial-DTO precedent + the `mute_all` EmitIntent. But **nothing records a town hall**:

- There is **zero S3 / Egress / MinIO surface in the bridge** (`grep -rn "egress\|minio\|s3\|aws_sdk" services/bridge/src/` returns nothing but `bridge_room.rs` comments). No way to trigger LiveKit Egress, no S3 client to land the MP4, no `content_sha256` computation. **This is the net-new dep work + the headline `validate-pending-laptop-linux` risk** (Task 3).
- There is **no `record_town_halls` flag-gate.** The `recording_config TEXT` column exists but nothing reads a recording knob from it; nothing gates the recording path on the flag. A `record_town_halls = false` town hall has no defined zero-side-effect posture (Task 2 + Task 4).
- `ENTRY_KIND_ROOM_RECORDING_UPLOADED` has **zero emitters.** Phase 5 is its first. The `RoomEventPayload` DTO (binary `governance_log.rs` + bridge mirror `room_event_client.rs:6-25`) carries chair-action + `federated` fields but **no recording-metadata fields** — the Success-Criteria `room_recording_uploaded { media_url, content_sha256, duration_s, speakers, attendance_count }` shape (line 143) cannot be carried (Task 1).
- There is **no recording-fetch endpoint** and **no participant-floor authorization.** A pseudonymous town hall's recording, served at anyone-with-the-URL, would leak (ADR-015 / OQ-V2-04 floor). The bridge has the participant set but no authz gate on fetch (Task 5).

## 4. Solution statement

The change has **two architectural surfaces**, mirroring the emergency-mute split:

**(a) Binary DTO extension (`crates/**`, Windows-validated) — trivial, the one in-scope crates touch.** The binary's `RoomEventPayload` (`crates/api/api/src/governance/governance_log.rs`) gains **five optional** recording-metadata fields — `media_url: Option<String>`, `content_sha256: Option<String>`, `duration_s: Option<i64>`, `speakers: Option<Vec<String>>`, `attendance_count: Option<i32>` — each `#[serde(default, skip_serializing_if = "Option::is_none")]`, so the `room_recording_uploaded` chain entry carries the Success-Criteria schema while every existing room kind's hashed JSON stays byte-identical (no `"media_url":null` on m2/m3 emissions → no hash regression). The binary constructs **zero** non-test `RoomEventPayload { … }` literals (it only deserialises the bridge POST + serialises into `append`), so the optional fields are non-breaking. This is the **exact** trivial-DTO class emergency-mute shipped for `federated` (one field → five fields, same pattern). The recording payload is **NOT a new `room_provisioner::RoomEventPayload` enum variant** (§19 (1)): that enum is the **inbound** appservice-handler dispatch type (`PrivateMessage` / `CaseTransition`); the recording emission rides the **outbound** `room_event_client::RoomEventPayload` struct — the same struct `mute_all`/`chair_override` use. `speakers` is `Vec<String>` of **pseudonyms** (ADR-015); the chair/uploader actor rides the existing top-level `actor_pseudonym` arg.

**(b) Bridge recording enactment (`services/bridge/**`, Linux-validated) — the bulk of the phase.**

- **Config + flag-gate (Task 2).** `config.rs` gains the operator-level generic-S3 settings (`s3_endpoint`, `s3_bucket`, `s3_access_key`, `s3_secret_key`, all `Option<String>` — mirroring the existing `livekit_*` optionals from m3-core-infra; **the endpoint comes from config/env, NEVER a hardcoded `minio:9000`**). `bridge_room.rs` gains `read_recording_config`/`write_recording_config` helpers (mirroring `read_chair_id`/`write_chair_id` — INSERT…ON CONFLICT keyed by `(case_id, room_type)`) and a pure `record_town_halls_enabled(recording_config: &str) -> bool` parse of the per-room `recording_config` JSON knob (clarify DQ `a3d0e9941441-073`: the flag is a knob in the EXISTING column, **no new column/migration**).
- **Recording primitives + dep-add (Task 3).** A new `recording.rs` module: `compute_content_sha256(bytes: &[u8]) -> String` (pure, `sha2::Sha256` hex — **fine to compute the hash with `sha2`; the CONSTRAINT is it must reach the chain via `append_room_event`, never a side-channel write**); a `RecordingSink` trait (mirroring `stage.rs`'s `GrantSink`) with `trigger_egress` + `upload` methods, a **real impl** (LiveKit Egress via the existing `reqwest` + a `livekit_jwt`-minted token; S3 PUT via the one new generic-S3 client) and a `Recorder` **spy** for deterministic tests. The **one** new dep is the generic-S3 client (`rust-s3` — §19 (2)); the Egress trigger reuses `reqwest` + `livekit_jwt` (no heavy LiveKit SDK). `Cargo.lock` regenerates in the **same commit** as `Cargo.toml`.
- **Flag-gated emission + clean-posture (Task 4).** `recording::maybe_record(enabled, sink, stage, media_url, content_sha256, duration_s, speakers, attendance_count)` — the flag-gated controller: when `!enabled` it returns **immediately with zero side-effects** (no `sink.trigger_egress`, no `sink.upload`, no EmitIntent); when `enabled` it drives the sink and pushes the `room_recording_uploaded` EmitIntent. `Stage::record_uploaded(...)` pushes the EmitIntent to `pending_emits` (mirroring `mute_all`'s push), drained by the existing `drain_emits` → `post_room_event` → `append_room_event`. The bridge `room_event_client::RoomEventPayload` mirror gains the 5 recording fields (existing literals get `..None`). The **clean-posture negative-invariant** test: `enabled = false` → the `Recorder` spy observed zero Egress/upload calls **and** `stage.pending_emits` is empty; **delete-the-gate check** — removing the `if !enabled { return }` guard makes the test FAIL.
- **Participant-floor fetch (Task 5).** A new bridge HTTP handler (route added to `appservice.rs::router`, mirroring `handle_room_event`'s shape) serves a recording (or its URL) only after a participant-floor check: a pure `is_participant(requester_pseudonym, participants: &[String]) -> bool` gate, called **before** serving — a non-participant request → 403, a participant request → 200 (Success Criteria line 144). The participant set + the requester are **pseudonyms** (ADR-015). The strict presigned-URL ACL / retention / tombstone / GDPR machinery is **out of scope** (D5 Option C — additive post-pilot).

A reader can predict §11 from this: `RoomEventPayload` + 5 fields (binary DTO + bridge mirror); `config.rs` (S3 settings) + `bridge_room.rs` (recording_config helpers + flag parse); `recording.rs` (new — sha256 + `RecordingSink` + `maybe_record` + fetch authz) + `stage.rs` (`record_uploaded` emit) + `main.rs` (`mod recording;`) + `appservice.rs` (fetch route); one bridge integration-test file. **One new dependency** (the S3 client); **no migration, no new const, no registry bump, no new sidecar code in-binary.**

## 5. Metadata

- **Phase:** `m3-core-recording` (M3 Phase 5)
- **Branch:** `phase-m3-core-recording` (cut by `bm-cut` before Task 1)
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 8 (Task 0 pre-flight + 6 impl + retro)
- **Estimated cargo budget:** N/A on daemon — **no cargo runs on EliteDesk** (validate-pending-laptop / -linux discipline; the laptop is the runner). Bridge Linux container first-run is COLD (~10–20 min); subsequent warm. The S3-dep add forces one COLD re-resolve at Task 3.
- **Forbidden-window applicability:** binding for the **local** laptop cargo (crates Windows + bridge Linux Docker); non-binding for daemon impl-task throughput (no daemon cargo). Shape G RESIDUAL-ONLY (local cargo-linux + CR/Copilot = full internal coverage).
- **Complexity score:** `3/10` — see breakdown. Well under the Sonnet split threshold (8). **Proceed-as-one.** (Score undercounts the net-new-dep + S3/Egress surface; that risk is captured as the §18 headline-risk row + the Task-3 `validate-pending-laptop-linux` gate, not the mechanical score.)

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 1 | 6 impl tasks (1–6); one above the 5 baseline |
| Migrations touched | +2 each | 0 | **No new migration** — `recording_config` column shipped by m3-core-infra; flag is a JSON knob in it |
| Crates touched | +1 each | 2 | `lemmy_api` (Task 1 DTO) + `brehon-bridge` (workspace-excluded; the bridge surface) |
| `crates/server/tests/e2e.rs` edits | +3 each | 0 | Bridge tests are separate small files (`tests/recording.rs`), NOT the 18k-line `crates/server/tests/e2e.rs` (no worker-hang risk) |
| New ADR-affecting decisions | +2 each | 0 | Consumes ADR-004/008/011/015/016 + OQ-V2-04 (D5=Option C, already resolved); supersedes none |
| Cargo budget peak above 6 GB | +1 per GB | 0 | No daemon cargo (Shape-G residual) |
| **Total** | — | **3** | Threshold for split-DQ: `>8` (Sonnet) |

**Split decision:** 3 ≤ 8 — proceed-as-one, no split-DQ. The phase has more files than emergency-mute (the S3/Egress/fetch surface is genuinely net-new), but each task respects the Sonnet ≤4-file/≤2-crate ceiling and the dep-add is isolated to one task. The mechanical score (3) is honest per the rubric; the real risk is the Task-3 Linux Cargo.lock resolution (§18 row 1), gated by `validate-pending-laptop-linux`.

### 5.2 Per-task complexity ceiling (Sonnet target ≤4 files / ≤2 crates)

All tasks satisfy the Sonnet ceiling. Closest: **Task 3** hand-edits 4 files (`Cargo.toml`, `recording.rs` new, `config.rs` if not fully covered by Task 2, `main.rs`) — `Cargo.lock` is auto-regenerated (not a hand-edited file), so cognitive load = 3 substantive edits (the new `recording.rs`, the `Cargo.toml` dep line, the `main.rs` mod line) in 1 crate. **Task 2** edits 2 files (`config.rs`, `bridge_room.rs`) — 1 crate. **Task 4** edits 3 files (`stage.rs`, `room_event_client.rs`, `recording.rs`) — 1 crate. **Task 5** edits 2 files (`appservice.rs`, `recording.rs`) — 1 crate. **Task 1** edits 1 file (binary `governance_log.rs`) — 1 crate. No e2e file appears in any `modifies:`.

## 6. Relationship to other M3 sub-phases

- **Depends on:** m3-core-infra (Phase 1, shipped) — `rtc_enabled`, LiveKit JWT minting + identity→pseudonym at issue, the `bridge_room.recording_config` column, the `livekit_*` config. m3-core-entry-kinds (Phase 2, shipped) — `ENTRY_KIND_ROOM_RECORDING_UPLOADED` + `ROOM_KINDS` (count 13, registry 72). m3-core-stage-mode (Phase 3, shipped) — the `EmitIntent`/`pending_emits`/`drain_emits`/`post_room_event` callback seam + `Stage`. m3-core-emergency-mute (Phase 4, shipped) — the `federated` trivial-DTO precedent + the `Stage::mute_all` EmitIntent-push pattern this plan mirrors for `record_uploaded`.
- **Followed by:** Phase 6 (e2e + clean-posture + pilot) wires the live recording trigger (today the Egress/upload path is `#[allow(dead_code)]`-scaffolded, reachable from the docker-gated `#[ignore]` test, exactly as stage-mode/emergency-mute scaffolded their live triggers) + runs the real pilot incl. the live Egress→MinIO→hash flow + the `record_town_halls=false` / `rtc_enabled=false` clean-posture acceptance.
- **Reuses:** the bridge→binary `room_event_client` emit seam (stage-mode built it) + the `bridge_room` helper pattern (m3-core-infra/stage-mode) + the `livekit_jwt` token mint (m3-core-infra) + the `federated` trivial-DTO add (emergency-mute). This phase builds NO new emitter, NO new callback client, NO new const, NO new migration.

## 7. Preflight guardrails inherited from prior phases

- **R1 (bridge-Linux):** every `services/bridge` cargo command runs via `scripts/brehon/cargo-linux.sh … --manifest-path services/bridge/Cargo.toml`; the Windows-local `cd services/bridge && cargo` form fails with `ruma-common` E0119 and is reference-only (`feedback_bridge_validates_on_linux_not_windows.md`).
- **R2 (Linux-compile gate — MANDATORY this phase):** every bridge-touching task (2–6) writes a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass` (`feedback_linux_compile_proof_is_a_gate.md`). **Task 3 is the headline gate** — it adds a new S3 dep to `services/bridge/Cargo.toml`/`Cargo.lock`; Cargo.lock resolution may surface a licence/version conflict not visible from the dep line alone. NOT optional, NOT skippable on a green check elsewhere.
- **R3 (no daemon cargo):** workers write the appropriate `validate-pending-laptop[-linux]` DQ and **stop**; the laptop advisor runs all cargo (`feedback_validate_pending_laptop_write_then_stop.md`).
- **R4 (cargo capture):** every cargo invocation captures to a log file and reads `tail -20` + `echo exit: $?`; never pipe cargo through `tail`/`grep` (`cargo-output-capture.md` + `no-cargo-output-paste.md`).
- **R5 (Task 0 enumerates all probes explicitly):** see Task 0.
- **R6 (clippy uniform):** all clippy invocations use `--no-deps -- -D warnings`; crates clippy adds `--features full`; bridge clippy runs via `cargo-linux.sh`.
- **R7 (the negative invariant is TESTED, not assumed):** the `record_town_halls=false` clean-posture invariant is a §16a story with a green deterministic checkpoint whose test FAILS when the flag-gate is deleted (`feedback_authz_state_machine_test_asserts_negative.md` + `feedback_build_what_tests_exercise.md`); a happy-path "recording fired" assert does NOT satisfy the DoD.
- **R8 (no schema migration / no new column):** Phase 5 USES the existing `recording_config` column + `lookup_by_case`; a new `services/bridge` embedded-schema column or a `crates/db_schema/migrations/**` migration is a scope violation → STOP and raise a `kind: "blocker"` DQ (bootstrap tripwire).
- **R9 (ADR-015 load-bearing):** (a) the recording-fetch handler rejects a non-participant **before** serving — the participant-floor check is a callsite in the fetch path, not a named constant; (b) the `room_recording_uploaded` payload's `speakers` + the actor `actor_pseudonym` are **pseudonyms only** — never `person_id`/username/MXID. Grep DoD in §15.5.
- **R10 (ADR-016 metadata-only):** only recording METADATA (`{media_url, content_sha256, duration_s, speakers, attendance_count}` + top-level `actor_pseudonym`) reaches `append_room_event`; the **MP4 bytes are stored in S3, NEVER sent to the chain**. `content_sha256` is the ONE hash of the bytes, computed bridge-side, carried as metadata. §15.5 asserts the emit payload carries only metadata.
- **R11 (`content_sha256` rides `append_room_event`, never a bypass — CATCH-FIRE):** the recording's `content_sha256` is a tamper-evidence hash on the governance chain (OQ-V2-04: "tamper-evidenced by design"). Computing it with `sha2::Sha256` over the MP4 bytes is fine; the constraint is it MUST flow `recording.rs → EmitIntent → drain_emits → post_room_event → append_room_event` (which runs `scrub_json` + the ed25519 hash-chain + signing), **never** a raw `sha2` write that bypasses the chain (an ADR-008/ADR-016 violation — bootstrap §7 catch-fire). §16a Story 1 asserts the `content_sha256` appears in a `governance_log` chain row with a valid signature/prev-hash link, not merely in a local variable.
- **R12 (generic S3, no hardcoded endpoint):** the bridge speaks a generic S3 API so an operator can swap real S3/R2 (D5). The S3 endpoint comes from `config.rs`/env, NOT a hardcoded `minio:9000` in bridge source. Grep DoD (§15.5): `grep -rn "minio\." services/bridge/src/` returns nothing after impl (or only comments). A task hardcoding the endpoint is a scope violation — catch-fire (bootstrap §7).
- **R13 (`--bins` bare fn name for bridge test filters — promoted from emergency-mute retro L1 recurrence-2):** every §15/§16a test-filter command for a bridge test uses the BARE fn name (e.g. `recording_lands_with_hash`), NOT `module::tests::recording_lands_with_hash`.
- **R14 (Cargo.lock sync on dep add):** the S3-client task (Task 3) adds a `services/bridge/Cargo.toml` dep; the regenerated `Cargo.lock` MUST land in the SAME commit. This is the ONLY task that should touch deps — if any other §13 task changes `Cargo.toml`, flag it.

## 8. Flow design

```
BEFORE (m3-core-infra/entry-kinds/stage-mode/emergency-mute shipped):
  binary  RoomEventPayload { case_id, matrix_room_id, lifecycle_stage, member_count,
                             action?, target_pseudonym?, from_pseudonym?, to_pseudonym?, at?, federated? }
                             (no recording fields)
  binary  ROOM_KINDS ⊇ { room_recording_uploaded }   (registered count 13; ZERO emitters)
  bridge  bridge_room.recording_config TEXT  (column exists; NO reader/writer/flag-parse)
  bridge  config:: livekit_url/api_key/api_secret  (no S3 settings)
  bridge  NO recording.rs ; NO S3 client ; NO Egress trigger ; NO content_sha256
  bridge  stage::Stage { pending_emits } ; mute_all/chair_* push EmitIntents
  bridge  room_event_client:: post_room_event / drain_emits  (the callback seam)
  bridge  appservice::router  (no recording-fetch route)

AFTER (m3-core-recording):
  binary  RoomEventPayload { …, media_url?, content_sha256?, duration_s?,
                             speakers?, attendance_count? }  (opt, serde default+skip)        [Task 1]
  bridge  config:: s3_endpoint? / s3_bucket? / s3_access_key? / s3_secret_key?                [Task 2]
  bridge  bridge_room:: read_recording_config / write_recording_config (mirror chair_id)       [Task 2]
          bridge_room:: record_town_halls_enabled(recording_config) -> bool  (pure parse)      [Task 2]
  bridge  recording.rs (NEW):                                                                  [Task 3]
            compute_content_sha256(bytes) -> hex   (sha2; NEVER a chain bypass)
            trait RecordingSink { trigger_egress(room) ; upload(bytes) -> media_url }
              (mirror GrantSink) ; real impl = reqwest+livekit_jwt Egress + rust-s3 PUT
              ; Recorder spy for tests
          Cargo.toml += rust-s3  (+ Cargo.lock, SAME commit)  ; main.rs += mod recording        [Task 3]
  bridge  recording:: maybe_record(enabled, sink, stage, media_url, sha, dur, speakers, count)  [Task 4]
            if !enabled { return }                       <-- the load-bearing flag-gate
            sink.trigger_egress(room) ; let url = sink.upload(bytes)
            stage.record_uploaded(url, sha, dur, speakers, count)
  bridge  stage::Stage::record_uploaded(...)  -> pending_emits.push(EmitIntent {               [Task 4]
              entry_kind: "room_recording_uploaded",
              payload: RoomEventPayload { …, media_url, content_sha256, duration_s,
                                          speakers, attendance_count },
              actor_pseudonym: self.chair.clone() })   (uploader/chair pseudonym, ADR-015)
  bridge  room_event_client:: RoomEventPayload { …, +5 recording fields }   (mirror; ..None)    [Task 4]
            --drain_emits--> post_room_event -> POST /api/v4/governance/room-event (Bearer)
            binary append_room_event(pool, "room_recording_uploaded", payload, actor) -> chain
            (scrub_json + ed25519 hash-chain + signing; content_sha256 rides HERE — R11)
  bridge  appservice::router += .route("/brehon/recording/{id}", get(handle_recording_fetch))   [Task 5]
            recording:: is_participant(requester_pseudonym, participants) -> bool  (ADR-015 floor)
            handle_recording_fetch: if !is_participant { 403 } else { 200 + media_url }
  bridge  tests/recording.rs  #[ignore] docker-gated real Egress->MinIO->sha256->append + fetch  [Task 6]
```

The live recording **trigger** (a town-hall start path calling `maybe_record`) is **Phase 6** — like stage-mode/emergency-mute, the Egress/upload real-impl carries `#[allow(dead_code)]` until Phase 6 wires the live town-hall-start call; the deterministic gate (`maybe_record` flag-gate + `record_uploaded` EmitIntent + sha256 + `is_participant`) runs now without docker.

## 9. Mandatory reading

Files the impl-task subagent MUST Read before its first edit:

- **Binary room-event seam** — `crates/api/api/src/governance/governance_log.rs:92-114` (`RoomEventPayload` struct — add the 5 recording fields after `federated` at `:113`), `:116-209` (the `#[cfg(test)]` `federated` roundtrip test to mirror for a `room_recording_uploaded` case), `:215-229` (`ROOM_KINDS` — `room_recording_uploaded` present at `:227`, do NOT re-touch), `:238-250` (`append_room_event`). [Task 1]
- **Bridge config** — `services/bridge/src/config.rs:21-90` (`BridgeConfig` struct + `from_env`; the `livekit_url`/`livekit_api_key`/`livekit_api_secret` `Option<String>` fields at `:54-60` are the MIRROR shape for the S3 settings). [Task 2]
- **Bridge room state helpers to MIRROR** — `services/bridge/src/bridge_room.rs:15` (`recording_config TEXT` column), `:138-160` (`write_chair_id`/`read_chair_id` — the INSERT…ON CONFLICT…DO UPDATE pattern to mirror for `write_recording_config`/`read_recording_config`), `:104-136` (`write_queue_state`/`read_queue_state` — the JSON-text read/write shape), `:59` (`lookup_by_case`), `:188,240` (column-existence test pattern). [Task 2]
- **Stage state machine + EmitIntent push pattern** — `services/bridge/src/stage.rs:20-35` (`GrantCmd`/`GrantSink` trait + `Recorder` — the MIRROR for `RecordingSink`), `:44-69` (`EmitIntent` + `Stage` fields incl. `pending_emits`), `:316-348` (`mute_all` — the EmitIntent-push + `actor_pseudonym` shape to mirror for `record_uploaded`), `:349-720` (the `#[cfg(test)]` module incl. the `Recorder` `GrantSink` + the negative-assertion `mute_all` test idioms to mirror for the clean-posture + emit-shape tests). [Tasks 3, 4]
- **Bridge→binary outbound client + drain** — `services/bridge/src/room_event_client.rs:1-25` (`RoomEventPayload` mirror — add the 5 recording fields), `:38-55` (`post_room_event`), `:64-78` (`drain_emits`), `:80-160` (the JSON-shape test idiom to mirror for `room_recording_uploaded`). [Task 4]
- **Bridge HTTP handler shape + router** — `services/bridge/src/appservice.rs:198-230` (`handle_room_event` — the `State`/`Json` handler + Bearer-auth shape to mirror), `:231-259` (`router()` — add the recording-fetch `.route(...)`). [Task 5]
- **Bridge module list** — `services/bridge/src/main.rs:12-26` (add `mod recording;` alphabetically — after `mod provision;`/before `mod relay;`). [Task 3]
- **Integration-test harness (MIRROR)** — `services/bridge/tests/stage_mode.rs` + `services/bridge/tests/emergency_mute.rs` (`#[tokio::test] #[ignore = "requires docker-compose stack"]` + the step-comment stub convention). [Task 6]
- **Registry (do NOT re-touch)** — `.claude/rules/governance-log-entry-kind-registry.md` "M2 room kinds" + "Acceptance invariants" (`room_recording_uploaded` is SHIPPED; Phase 5 emits, never re-declares; count stays 72). [Tasks 1, 4]
- **Lessons** — `feedback_bridge_validates_on_linux_not_windows.md`, `feedback_linux_compile_proof_is_a_gate.md`, `feedback_authz_state_machine_test_asserts_negative.md`, `feedback_build_what_tests_exercise.md`, `pattern_test_against_reality_not_syntax.md`, `feedback_entry_kind_runtime_allowlist_check.md`, `feedback_cheap_model_arm_drops_adr_constraints.md`. [Tasks 1–6]

## 10. Patterns to mirror

### 10.1 Binary `RoomEventPayload` recording-fields extension (trivial DTO — mirror the `federated` add)

**Mirror:** `crates/api/api/src/governance/governance_log.rs` `RoomEventPayload` (struct at `:92`, `federated` field at `:113`, its `#[cfg(test)]` roundtrip test at `:116-209`).

```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomEventPayload {
  pub case_id: i32,
  // … existing fields (matrix_room_id / lifecycle_stage / member_count /
  //    chair-action fields / federated) …
  /// M3 recording: the stored MP4's object URL in the generic-S3 store.
  /// Only `room_recording_uploaded` sets it. ADR-016: metadata only (the
  /// MP4 BYTES live in S3, never on the chain).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub media_url: Option<String>,
  /// M3 recording: hex SHA-256 of the MP4 bytes — the tamper-evidence hash
  /// that rides the chain via append_room_event (ADR-008). NEVER a bypass digest.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub content_sha256: Option<String>,
  /// M3 recording: recording duration in seconds.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub duration_s: Option<i64>,
  /// M3 recording: PSEUDONYMS of speakers in the recording (ADR-015 — never person_id/MXID).
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub speakers: Option<Vec<String>>,
  /// M3 recording: attendance headcount at recording time.
  #[serde(default, skip_serializing_if = "Option::is_none")]
  pub attendance_count: Option<i32>,
}
```

`skip_serializing_if = "Option::is_none"` keeps existing room kinds' hashed JSON byte-identical (no `"media_url":null` on m2/m3 emissions → no hash regression). `#[serde(default)]` keeps deserialisation back-compatible. The uploader/chair actor rides the top-level `actor_pseudonym` (§19 (1)) — **not** a payload field.

### 10.2 Bridge `record_town_halls_enabled` flag parse + recording_config helpers

**Mirror:** `services/bridge/src/bridge_room.rs:138-160` (`write_chair_id`/`read_chair_id` INSERT…ON CONFLICT) + `:104-136` (`write_queue_state`/`read_queue_state` JSON-text shape).

```rust
// services/bridge/src/bridge_room.rs
/// Read the per-room recording_config JSON knob (mirror read_chair_id).
pub fn read_recording_config(conn: &Connection, case_id: i64, room_type: &str) -> Result<Option<String>> { /* SELECT recording_config … */ }

/// Write the per-room recording_config JSON knob (mirror write_chair_id;
/// INSERT … ON CONFLICT(case_id, room_type) DO UPDATE SET recording_config = excluded.recording_config).
pub fn write_recording_config(conn: &Connection, case_id: i64, room_type: &str, config: &str) -> Result<()> { /* … */ }

/// Pure parse of the per-room recording_config knob.  `record_town_halls`
/// defaults FALSE (absent / unparseable / false → false).  Clarify DQ
/// a3d0e9941441-073: the flag lives in the EXISTING recording_config column,
/// NOT a new column.
pub fn record_town_halls_enabled(recording_config: &str) -> bool {
    serde_json::from_str::<serde_json::Value>(recording_config)
        .ok()
        .and_then(|v| v.get("record_town_halls").and_then(|b| b.as_bool()))
        .unwrap_or(false)
}
```

The deterministic test (§16a Story 2) feeds `{"record_town_halls": true}` → `true`, `{"record_town_halls": false}` / `{}` / `"garbage"` → `false`.

### 10.3 `recording.rs` — `compute_content_sha256` + `RecordingSink` trait (mirror `GrantSink`)

**Mirror:** `services/bridge/src/stage.rs:24-37` (`GrantCmd`/`GrantSink` trait + the `Recorder` spy convention at `:349-720`).

```rust
// services/bridge/src/recording.rs  (NEW)
use sha2::{Digest, Sha256};

/// Hex SHA-256 of the MP4 bytes.  This hash RIDES the governance chain via
/// append_room_event (R11) — computing it here with sha2 is correct; writing
/// it anywhere that BYPASSES append_room_event is an ADR-008/016 violation.
pub fn compute_content_sha256(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    hex::encode(h.finalize())
}

/// Applied by the recording controller; the real impl triggers LiveKit Egress
/// (reqwest + livekit_jwt token) and PUTs the MP4 to the generic-S3 store
/// (endpoint from config — R12, NEVER a hardcoded minio:9000).  Tests use the
/// `Recorder` spy.  Mirror of stage::GrantSink.
pub trait RecordingSink {
    /// Trigger LiveKit Egress for `room_id`.  Pseudonymous overlay only (ADR-015).
    fn trigger_egress(&mut self, room_id: &str) -> anyhow::Result<()>;
    /// Upload the MP4 bytes to the generic-S3 store; return the object URL.
    fn upload(&mut self, key: &str, bytes: &[u8]) -> anyhow::Result<String>;
}

/// The real production sink — `#[allow(dead_code)]` until Phase 6 wires the
/// live town-hall-start trigger (the stage-mode/emergency-mute scaffold flow).
#[allow(dead_code)]
pub struct LiveSink<'a> { /* &reqwest::Client, &BridgeConfig (s3_endpoint/bucket/creds, livekit_*) */ }
```

### 10.4 `recording::maybe_record` — the flag-gated controller (the clean-posture negative invariant)

**Mirror:** `services/bridge/src/stage.rs:316-348` (`mute_all` EmitIntent push + `actor_pseudonym`).

```rust
// services/bridge/src/recording.rs
/// Flag-gated recording controller.  When `enabled` is false, returns with
/// ZERO side-effects (no Egress, no upload, no EmitIntent) — the clean-posture
/// invariant (Success Criteria line 145).  When true, drives the sink and
/// pushes the room_recording_uploaded EmitIntent (drained by drain_emits →
/// append_room_event).  `speakers` are PSEUDONYMS (ADR-015).
#[allow(dead_code)] // live town-hall-start trigger lands Phase 6; reachable from the #[ignore] test now.
pub fn maybe_record(
    enabled: bool,
    sink: &mut dyn RecordingSink,
    stage: &mut crate::stage::Stage,
    room_id: &str,
    mp4_bytes: &[u8],
    duration_s: i64,
    speakers: Vec<String>,
    attendance_count: i32,
) -> anyhow::Result<()> {
    if !enabled {
        return Ok(()); // <-- the load-bearing flag-gate (delete this → clean-posture test FAILS)
    }
    sink.trigger_egress(room_id)?;
    let content_sha256 = compute_content_sha256(mp4_bytes);
    let media_url = sink.upload(&format!("{room_id}.mp4"), mp4_bytes)?;
    stage.record_uploaded(media_url, content_sha256, duration_s, speakers, attendance_count);
    Ok(())
}
```

**The clean-posture test (§16a Story 3) — the negative invariant:** call `maybe_record(false, &mut recorder, &mut stage, …)`, assert the `Recorder` spy observed **zero** `trigger_egress`/`upload` calls **and** `stage.pending_emits.is_empty()`. **Mechanical delete-the-gate check:** removing the `if !enabled { return }` guard makes the false-case see Egress + upload + one EmitIntent → the test FAILS. A clean-posture test that is a trivial always-skip (asserts nothing under `enabled=true`) is invalid (R7).

### 10.5 `Stage::record_uploaded` — push the `room_recording_uploaded` EmitIntent

**Mirror:** `services/bridge/src/stage.rs:316-348` (`mute_all` push).

```rust
// services/bridge/src/stage.rs
/// First emission of room_recording_uploaded.  Push the EmitIntent carrying
/// the Success-Criteria schema; drained by the existing drain_emits →
/// post_room_event → append_room_event (the content_sha256 rides THERE — R11).
/// `speakers` are PSEUDONYMS (ADR-015); the actor is the chair pseudonym.
pub fn record_uploaded(&mut self, media_url: String, content_sha256: String,
                       duration_s: i64, speakers: Vec<String>, attendance_count: i32) {
    self.pending_emits.push(EmitIntent {
        entry_kind: "room_recording_uploaded",
        payload: RoomEventPayload {
            case_id: self.case_id as i32,
            matrix_room_id: None,
            lifecycle_stage: self.room_type.clone(),
            member_count: None,
            action: None, target_pseudonym: None,
            from_pseudonym: None, to_pseudonym: None, at: None,
            federated: None,
            media_url: Some(media_url),
            content_sha256: Some(content_sha256),
            duration_s: Some(duration_s),
            speakers: Some(speakers),
            attendance_count: Some(attendance_count),
        },
        actor_pseudonym: self.chair.clone(), // uploader/chair pseudonym (ADR-015 pin)
    });
}
```

The emit-shape test (§16a Story 1) asserts exactly one `room_recording_uploaded` EmitIntent with the 5 recording fields populated + `actor_pseudonym` = a pseudonym, OMITTING the chair-action fields (`action`/`from_pseudonym`/`to_pseudonym`/`federated`) via `skip_serializing_if`.

### 10.6 Bridge `RoomEventPayload` mirror + `room_recording_uploaded` JSON-shape test

**Mirror:** `services/bridge/src/room_event_client.rs:5-25` (the Serialize mirror — the `federated` field) + `:80-160` (the JSON-shape test idiom).

Add the 5 `#[serde(skip_serializing_if = "Option::is_none")]` recording fields to the bridge `RoomEventPayload` (mirror the binary struct). Every existing constructor literal of this struct then requires the 5 fields as `None` (enumerated in §11). The new `room_recording_uploaded` test asserts the serialised `RoomEventRequest` has `entry_kind: "room_recording_uploaded"`, all 5 recording fields present + correct, `actor_pseudonym` a pseudonym, and OMITS the chair-action/`federated` fields.

### 10.7 Participant-floor fetch authz (ADR-015 — LOAD-BEARING)

**Mirror:** `services/bridge/src/appservice.rs:198-230` (the `State`/`Json` handler + Bearer shape) + `:231-259` (`router()`).

```rust
// services/bridge/src/recording.rs
/// ADR-015 participant-floor: a recording-fetch requester MUST be a participant
/// of the room at recording time.  Cannot be zero under always_pseudonym (else a
/// pseudonymous town hall's recording leaks).  Both args are PSEUDONYMS.
pub fn is_participant(requester_pseudonym: &str, participants: &[String]) -> bool {
    participants.iter().any(|p| p == requester_pseudonym)
}

// services/bridge/src/appservice.rs  — new route + handler
async fn handle_recording_fetch(/* State, requester pseudonym from session, recording id */) -> Response {
    // … resolve the recording's participant set (bridge_room / room membership) …
    if !recording::is_participant(&requester_pseudonym, &participants) {
        return (StatusCode::FORBIDDEN, "not a participant").into_response(); // ADR-015 floor
    }
    // 200 + media_url (or the recording bytes/redirect).  Strict presigned ACL DEFERRED (D5 Option C).
}
```

`is_participant` is called **before** serving — the §2.4a grep DoD: `grep is_participant services/bridge/src/appservice.rs` returns the callsite AND `grep "fn is_participant" services/bridge/src/recording.rs` returns the definition. **Why it can't be deferred:** a recording served at anyone-with-the-URL under `always_pseudonym` leaks the pseudonymous event's audio/video to non-participants — the participant floor is the D5 Option C access bar (the strict presigned ACL is the additive upgrade, not this gate).

### 10.8 `always_pseudonym` Egress-leak guarantee (inherited, named not silently assumed)

Per PRD risk row 201: the identity→pseudonym mapping is at **JWT-issue time** (m3-core-infra), so the overlay is on the LiveKit-rendered stream and **Egress captures the PSEUDONYM overlay** — the LiveKit server never receives the real identity. This recording path **inherits** that guarantee (it triggers Egress on the already-pseudonymised stream; it adds no new identity surface). The §16a recording story asserts `speakers`/actor in the chain entry are pseudonyms (the recording-path observable); the "LiveKit never receives the real identity" guarantee is the **m3-core-infra JWT-issue-time test's** responsibility (cited, not re-tested here — the recording path adds no JWT-issue surface). The Task-6 `#[ignore]` integration step-comments name this inheritance explicitly.

## 11. Files to change

**`crates/api` (lemmy_api), Windows-validated:**
- `crates/api/api/src/governance/governance_log.rs` — 5 optional recording fields on `RoomEventPayload` + extend the `#[cfg(test)]` roundtrip test with a `room_recording_uploaded` case (Task 1)

**`services/bridge` (brehon-bridge, workspace-excluded), Linux-validated:**
- `services/bridge/src/config.rs` — 4 optional S3 settings (`s3_endpoint`/`s3_bucket`/`s3_access_key`/`s3_secret_key`) on `BridgeConfig` + `from_env` (Task 2)
- `services/bridge/src/bridge_room.rs` — `read_recording_config`/`write_recording_config` helpers + `record_town_halls_enabled` pure parse + a `#[cfg(test)]` flag-parse test (Task 2)
- `services/bridge/Cargo.toml` + `services/bridge/Cargo.lock` — the one new generic-S3 dep (`rust-s3`), regenerated lock in the SAME commit (Task 3)
- `services/bridge/src/recording.rs` — NEW: `compute_content_sha256` (pure) + `RecordingSink` trait + `LiveSink` real impl (`#[allow(dead_code)]`) + `Recorder` spy + `maybe_record` flag-gated controller + `is_participant` authz (Task 3 creates the file w/ sha256 + sink; Task 4 adds `maybe_record`; Task 5 adds `is_participant`)
- `services/bridge/src/main.rs` — `mod recording;` (alphabetical) (Task 3)
- `services/bridge/src/stage.rs` — `Stage::record_uploaded` (the `room_recording_uploaded` EmitIntent push) + the emit-shape + clean-posture tests; add the 5 recording fields as `None` to the 4 existing `RoomEventPayload` literals (`:?` chair_override ×2, transfer, mute_all) (Task 4)
- `services/bridge/src/room_event_client.rs` — 5 recording mirror fields on `RoomEventPayload`; add `..None` to the existing test literals; add the `room_recording_uploaded` JSON-shape test (Task 4)
- `services/bridge/src/appservice.rs` — the recording-fetch `.route(...)` + `handle_recording_fetch` (participant-floor) (Task 5)

**Tests (brehon-bridge integration), Linux-validated:**
- `services/bridge/tests/recording.rs` — NEW: docker-gated `#[ignore]` real Egress→MinIO→sha256→append + participant-floor fetch (Task 6)

### Struct-field add: enumerate all callsites (mandatory)

- **`RoomEventPayload` (binary, Task 1):** `rg "RoomEventPayload\s*\{" crates/` returns **only** the `#[cfg(test)]` literals in `governance_log.rs` (the binary never constructs it in non-test code — it deserialises the bridge POST + serialises into `append`). New fields are `Option` + `#[serde(default)]` → non-breaking; the test literals get the 5 fields as `None` (or `Some(..)` in the new recording case), same file/task.
- **`RoomEventPayload` (bridge mirror, Task 4):** `rg "RoomEventPayload\s*\{" services/bridge/` returns the literals in `stage.rs` (chair_override ×2, transfer, mute_all = 4) + `room_event_client.rs` (the JSON-shape tests) + the NEW `record_uploaded` literal. Adding the 5 fields to the bridge struct (no `Default`) requires the 5 fields on every existing literal in the SAME commit (else `brehon-bridge` fails to compile). The impl-task runs `rg "RoomEventPayload\s*\{" services/bridge/` FIRST and adds `media_url: None, content_sha256: None, duration_s: None, speakers: None, attendance_count: None` to each existing literal. Caller note tagged "Caller files (compiles-only-after-Task-4)" below.
- **`compute_content_sha256` / `RecordingSink` / `maybe_record` / `is_participant` (Tasks 3–5):** new symbols, zero existing callers; the real `LiveSink` + `maybe_record` are reachable only from the Task-6 `#[ignore]` test until Phase-6 wires the live trigger (hence `#[allow(dead_code)]`).

> **Task 4 note (Caller files — compiles-only-after-Task-4):** `stage.rs` (4 existing literals + 1 new) and `room_event_client.rs` (test literals) BOTH construct the bridge `RoomEventPayload`; both MUST get the 5 recording fields in Task 4's single commit, else `brehon-bridge` fails to compile. Both are in Task 4's `modifies:`. 3 files, 1 crate — within the ≤4/≤2 Sonnet ceiling.

## 12. NOT building in m3-core-recording

- **Strict presigned-URL ACL / retention / tombstone / GDPR-erasure machinery** — DEFERRED to a post-pilot additive sub-phase (D5 = Option C; OQ-V2-04 resolution). Phase 5 ships `content_sha256`-on-chain integrity + the participant-floor authz only. Reason: the strict ACL is "presigned URLs in front of an existing bucket" — additive, not a migration; the pilot decides if it's needed.
- **Cross-instance recording-store federation** — recordings stay **instance-local** (OQ-V2-07). No replicating S3 objects across instances. A §13 task that federates the recording store is a scope error.
- **Transcript / `room_transcript_ready`** — that const exists (`governance_log.rs:228`) but is OUT of scope (a separate future surface). Phase 5 emits `room_recording_uploaded` only.
- **A new entry-kind const / `ROOM_KINDS` add / registry-count bump** — `ENTRY_KIND_ROOM_RECORDING_UPLOADED` is SHIPPED (M2; const at `governance_log.rs:243`, in `ROOM_KINDS` at the shim `:227`, count 13 / registry 72). Phase 5 EMITS via `append_room_event`, never re-declares (bootstrap tripwire → STOP + `kind: "blocker"` DQ).
- **A new `bridge_room` column or any migration** — Phase 5 USES the existing `recording_config` column (clarify DQ `a3d0e9941441-073`). A new column/migration is a scope violation (bootstrap tripwire → STOP + `kind: "blocker"` DQ).
- **A hardcoded MinIO endpoint** — the S3 endpoint comes from `config.rs`/env (R12, D5 operator-swap). A `minio:9000` literal in bridge source is a scope violation (catch-fire).
- **A `content_sha256` chain bypass** — the hash is computed with `sha2` bridge-side but MUST ride `append_room_event` (R11). A raw-digest write that bypasses the chain is an ADR-008/016 violation (catch-fire).
- **The live town-hall-start recording trigger** — deferred to **Phase 6** (the live RTC-session-start path that calls `maybe_record`). Phase 5 lands `maybe_record` + the real `LiveSink` with `#[allow(dead_code)]`; reachable from the Task-6 `#[ignore]` test (the stage-mode/emergency-mute dead_code-scaffold flow).
- **A second new fetched dependency** — the ONLY dep with new Linux resolution risk is the generic-S3 client (`rust-s3`, Task 3). `sha2` is promoted transitive→direct (already in the lock, zero resolution change); `hex` is already a direct dep; the Egress trigger reuses `reqwest` + `livekit_jwt`. If any task needs another newly-fetched dep, flag it (R14).
- **Anonymous-town-hall pseudonym-overlay work** — Phase 6 / inherited; the identity→pseudonym mapping is at JWT-issue time (m3-core-infra). Phase 5 only consumes pseudonyms the Stage already holds + asserts they reach the chain entry as pseudonyms (§10.8).

---

## 13. Step-by-step tasks

Execute in dependency order. **One commit per task.** Task 0 is non-`[P]` (barrier).

> **Cohort note:** Task 1 (`crates/api`, Windows) and Task 2 (`services/bridge` config/flag, Linux) are file-disjoint across different crates and different validation runners → genuine `[P]` (Cohort A). Bridge Tasks 3–6 share the bridge crate + cold Linux builds (and Task 3 adds a dep, forcing a cold Cargo.lock re-resolve), so they run **serial** (3 → 4 → 5 → 6) to avoid concurrent cold-build / `Cargo.lock` contention (the stage-mode/emergency-mute bridge-serial precedent). Per the cross-lane cap, at most 2 Junior workers run concurrently.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment + branch + base state before Task 1; confirm the recording seams the phase REUSES are present + the const/column it must NOT re-create already exist.

**Probes (R5 — enumerate ALL explicitly):**

```bash
# Probe 0 — Docker daemon (cargo-linux.sh Linux build + bridge docker-gated tests)
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 — Docker in LINUX-container mode (cargo-linux.sh requires it)
docker info --format '{{.OSType}}'   # EXPECT: linux

# Probe 2 — on the phase branch
git branch --show-current            # EXPECT: phase-m3-core-recording

# Probe 3 — room_recording_uploaded const is SHIPPED on base (Phase 5 must NOT re-declare)
rg -c '^pub const ENTRY_KIND_ROOM_RECORDING_UPLOADED' \
  crates/db_schema/src/source/governance/governance_log.rs   # EXPECT: 1

# Probe 4 — ROOM_KINDS runtime allowlist already admits room_recording_uploaded (emit path ready)
rg -c 'ENTRY_KIND_ROOM_RECORDING_UPLOADED' crates/api/api/src/governance/governance_log.rs  # EXPECT: >=2 (shim pub use + ROOM_KINDS)

# Probe 5 — entry-kind registry count is UNCHANGED at 72 (Phase 5 adds NO const)
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs  # EXPECT: 72

# Probe 6 — the recording_config column + helper pattern to MIRROR is present (NO new migration)
rg -c 'recording_config' services/bridge/src/bridge_room.rs                       # EXPECT: >=3 (column + tests)
rg -c 'fn write_chair_id|fn read_chair_id|fn lookup_by_case' services/bridge/src/bridge_room.rs  # EXPECT: 3

# Probe 7 — the emit seam is present (Phase 5 reuses, never rebuilds)
rg -c 'pub struct EmitIntent|pub pending_emits' services/bridge/src/stage.rs        # EXPECT: 2
rg -c 'pub async fn post_room_event|pub async fn drain_emits' services/bridge/src/room_event_client.rs  # EXPECT: 2

# Probe 8 — zero S3/Egress/MinIO surface today (Task 3 is genuinely net-new)
rg -c 'aws_sdk|rust_s3|s3::|egress|minio' services/bridge/src/   # EXPECT: 0 (no matches → net-new)

# Probe 9 — livekit config + jwt mint present (Egress reuses them, no new LiveKit SDK)
rg -c 'livekit_url|livekit_api_key|livekit_api_secret' services/bridge/src/config.rs   # EXPECT: 3
rg -c 'mod livekit_jwt' services/bridge/src/main.rs   # EXPECT: 1

# Probe 10 — sha2/hex availability for content_sha256 (hex is a direct dep; sha2 may be transitive)
rg -c '^hex ' services/bridge/Cargo.toml   # EXPECT: 1 (hex present)

# Probe 11 — bridge baseline COMPILES on Linux (cold ~10-20 min; warm after)
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-rec-task0-bridge-check.log 2>&1
echo "bridge baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task0-bridge-check.log  # EXPECT: exit 0

# Probe 12 — crates baseline compiles (Windows)
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-rec-task0-check.log 2>&1"
echo "crates baseline exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task0-check.log  # EXPECT: exit 0

# Probe 13 (negative) — wrapper propagates failure
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/m3-rec-task0-neg.log 2>&1"
echo "negative exit: $?"   # EXPECT: NON-ZERO
```

**EXPECT:** Probes 0–12 succeed per their inline expectations (Probe 8 EXPECT 0 = the net-new confirmation); Probe 13 exits non-zero. **No commit at Task 0.**

### Task 1 [P]: Binary `RoomEventPayload` recording fields

**ACTION:** add 5 optional recording-metadata fields to the binary room-event DTO so the `room_recording_uploaded` chain entry can carry the Success-Criteria schema.

**FILES:**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/governance_log.rs   # media_url/content_sha256/duration_s/speakers/attendance_count on RoomEventPayload + #[cfg(test)] room_recording_uploaded case
```

**IMPLEMENT:** `governance_log.rs` — add the 5 `#[serde(default, skip_serializing_if = "Option::is_none")]` recording fields per §10.1 (place after `federated` at `:113`). Extend the existing `#[cfg(test)]` roundtrip test (the `federated` test) with a `room_recording_uploaded` case: build a payload `{ case_id, lifecycle_stage: "room_recording_uploaded", media_url: Some(..), content_sha256: Some(..), duration_s: Some(..), speakers: Some(vec![..pseudonyms..]), attendance_count: Some(..), .. all chair/federated fields None }`, `serde_json::to_value`, assert each of the 5 fields is present + correct, the chair-action + `federated` fields are OMITTED, and that a payload with all-None recording fields OMITS them (skip_serializing_if → byte-identical to existing room kinds).

**MIRROR:** §10.1; `crates/api/api/src/governance/governance_log.rs` `RoomEventPayload` struct + the `federated` roundtrip test (`:160-209`).

**GOTCHA:** `skip_serializing_if = "Option::is_none"` is load-bearing — without it, existing room emissions gain `"media_url":null` (etc), changing hashed JSON. **No non-test `RoomEventPayload { … }` constructor exists** in `crates/` (grep returns only test literals), so the `Option` fields break nothing. `room_recording_uploaded` is already in `ROOM_KINDS` (`:227`) — do NOT touch `ROOM_KINDS` or any const. ADR-015: `speakers` carries pseudonyms; ADR-016: metadata only (the MP4 bytes are NOT in this payload).

**VALIDATE (Windows):**

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-rec-task1-check.log 2>&1"
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task1-check.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop` DQ with `commands: ["cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"", "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\""]` (`e2e_filter: null` — the `governance_log` unit test runs under check); commit + push, then **stop**. (No `-linux` DQ — Task 1 touches no bridge/Cargo.toml/migration/cfg code.)

### Task 2 [P]: Bridge S3 config + `record_town_halls` flag-gate helpers

**ACTION:** add the generic-S3 operator settings to `BridgeConfig` and the `recording_config` read/write helpers + the `record_town_halls_enabled` pure flag-parse to `bridge_room.rs`.

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/config.rs       # s3_endpoint/s3_bucket/s3_access_key/s3_secret_key Option<String> on BridgeConfig + from_env
  - services/bridge/src/bridge_room.rs  # read_recording_config/write_recording_config + record_town_halls_enabled + flag-parse test
```

**IMPLEMENT (file 1 of 2):** `config.rs` — add `pub s3_endpoint: Option<String>`, `pub s3_bucket: Option<String>`, `pub s3_access_key: Option<String>`, `pub s3_secret_key: Option<String>` to `BridgeConfig` (mirror the `livekit_*` optionals at `:54-60`); populate them in `from_env` from `S3_ENDPOINT`/`S3_BUCKET`/`S3_ACCESS_KEY`/`S3_SECRET_KEY` via `std::env::var(..).ok()`. **R12: the endpoint is config/env — NO hardcoded `minio:9000`.**
**IMPLEMENT (file 2 of 2):** `bridge_room.rs` — add `read_recording_config`/`write_recording_config` (mirror `read_chair_id`/`write_chair_id` at `:138-160`, INSERT…ON CONFLICT(case_id, room_type) DO UPDATE SET recording_config) + `record_town_halls_enabled(recording_config: &str) -> bool` per §10.2 (default FALSE on absent/unparseable/false). Add a `#[cfg(test)]` test `record_town_halls_flag_parse`: `{"record_town_halls": true}` → true; `{"record_town_halls": false}` / `{}` / `"garbage"` → false.

**MIRROR:** §10.2; `services/bridge/src/bridge_room.rs:104-160` (write/read helpers), `:188,240` (column-existence test); `services/bridge/src/config.rs:54-90` (`livekit_*` optionals + `from_env`).

**GOTCHA:** the flag lives in the EXISTING `recording_config` column (clarify DQ `a3d0e9941441-073`) — do NOT add a column or a migration (R8 → STOP if tempted). Default-false is load-bearing (a missing/garbage knob must NOT enable recording — clean-posture). Bridge compiles on **Linux only** (R1).

**VALIDATE (Linux):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-rec-task2-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task2-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml record_town_halls_flag_parse \
  > .claude/PRPs/debug/m3-rec-task2-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task2-bridge-test.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (`commands` = `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test … record_town_halls_flag_parse`), commit + push, then **stop**. (No Cargo.toml change in Task 2 — no dep, so the `-linux` gate here is the compile proof, not a lock-resolution proof.)

### Task 3: Bridge recording primitives + the one new S3 dependency

**ACTION:** create `services/bridge/src/recording.rs` with `compute_content_sha256` (pure) + the `RecordingSink` trait + the `#[allow(dead_code)]` `LiveSink` real impl (LiveKit Egress via reqwest+livekit_jwt + generic-S3 PUT) + the `Recorder` spy; add the one new generic-S3 dep to `Cargo.toml` (regenerate `Cargo.lock` in the SAME commit); register the module.

**FILES:**

```yaml
creates:
  - services/bridge/src/recording.rs
modifies:
  - services/bridge/Cargo.toml          # generic-S3 client dep (rust-s3, the one with resolution risk) + sha2 promoted transitive->direct — §19 (2)
  - services/bridge/Cargo.lock          # regenerated, SAME commit (R14)
  - services/bridge/src/main.rs         # mod recording; (alphabetical)
requires:
  - task: 2
    reason: LiveSink reads s3_endpoint/s3_bucket/s3_access_key/s3_secret_key from BridgeConfig (Task 2)
```

**IMPLEMENT (file 1 of 3):** `recording.rs` per §10.3 — `compute_content_sha256(bytes: &[u8]) -> String` (sha2::Sha256 hex via `hex::encode`), the `RecordingSink` trait (`trigger_egress` + `upload`), the `#[allow(dead_code)] LiveSink` real impl (Egress: build the LiveKit Egress start request, mint a token via the existing `livekit_jwt`, POST via the existing `reqwest::Client` to `config.livekit_url`; S3 PUT: the new generic-S3 client, endpoint/bucket/creds from `config` — **R12 no hardcoded endpoint**), and the `Recorder` spy (records `trigger_egress`/`upload` calls, mirror `stage.rs`'s `Recorder` `GrantSink`). Add `#[cfg(test)]` tests: (a) `content_sha256_is_stable` — `compute_content_sha256(b"abc")` equals the known SHA-256 hex of `abc` (deterministic, exercises the real hash); (b) the `Recorder` spy compiles + records calls.
**IMPLEMENT (file 2 of 3):** `Cargo.toml` — add the generic-S3 dep (`rust-s3` — §19 (2); this is the ONE dep with new Linux Cargo.lock resolution risk) + promote `sha2` from transitive to a **direct** dep (`sha2 = "0.10"` — it is currently transitive-only: present in `Cargo.lock` but NOT in `Cargo.toml`, so using it directly requires a direct dep line; this is zero new resolution risk since it is already locked). **Regenerate `Cargo.lock` in the SAME commit.**
**IMPLEMENT (file 3 of 3):** `main.rs` — `mod recording;` (alphabetical, after `mod provision;`/before `mod relay;` per `:12-26`).

**MIRROR:** §10.3; `services/bridge/src/stage.rs:24-37` (`GrantSink` trait) + `:349-720` (`Recorder` spy); `services/bridge/src/livekit_jwt.rs` (token mint); `services/bridge/src/config.rs` (S3 settings from Task 2).

**GOTCHA (R11 — content_sha256 rides the chain):** `compute_content_sha256` computes the hash with `sha2`; it is the bridge-side hash of the MP4 bytes — it MUST later reach `append_room_event` via the EmitIntent (Task 4), NEVER a raw write here. recording.rs does NOT write `governance_log` directly. **GOTCHA (R12 — generic S3):** the S3 endpoint/bucket/creds come from `config` — NO hardcoded `minio:9000`; `grep -rn "minio\." services/bridge/src/recording.rs` returns nothing. **GOTCHA (R14 — Cargo.lock):** the new dep's `Cargo.lock` lands in THIS commit or `brehon-bridge` won't build reproducibly; the `validate-pending-laptop-linux` gate is the lock-resolution proof (a licence/version conflict surfaces HERE, not from the dep line). `LiveSink`/`maybe_record` are `#[allow(dead_code)]` until Phase-6 wires the live trigger. Bridge compiles on **Linux only**.

**VALIDATE (Linux):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-rec-task3-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task3-bridge-check.log   # EXPECT: exit 0 (cold re-resolve on the new dep)
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml content_sha256_is_stable \
  > .claude/PRPs/debug/m3-rec-task3-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task3-bridge-test.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (`commands` = `cargo-linux.sh check` + `clippy --no-deps -- -D warnings` + `test … content_sha256_is_stable`), commit + push, then **stop**. **This is the headline gate — the new dep's Linux Cargo.lock resolution is proven HERE.**

### Task 4: Flag-gated emission (`maybe_record` + `Stage::record_uploaded`) + clean-posture invariant + bridge DTO mirror

**ACTION:** add `recording::maybe_record` (the flag-gated controller, the clean-posture negative invariant) + `Stage::record_uploaded` (the `room_recording_uploaded` EmitIntent push) + the 5 recording mirror fields on the bridge `RoomEventPayload` (updating the existing literals).

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/recording.rs        # maybe_record flag-gated controller + clean-posture test
  - services/bridge/src/stage.rs            # Stage::record_uploaded EmitIntent push + emit-shape test; 5 recording fields = None on the 4 existing RoomEventPayload literals
  - services/bridge/src/room_event_client.rs # 5 recording fields on RoomEventPayload mirror; ..None on existing test literals; room_recording_uploaded JSON-shape test
requires:
  - task: 1
    reason: the bridge room_recording_uploaded POST carries the 5 recording fields; the binary RoomEventPayload must have the matching #[serde(default)] fields (Task 1) for the round-trip to deserialise
  - task: 3
    reason: maybe_record calls compute_content_sha256 + RecordingSink (Task 3)
```

**IMPLEMENT (file 1 of 3):** `room_event_client.rs` — add the 5 `#[serde(skip_serializing_if = "Option::is_none")]` recording fields to the bridge `RoomEventPayload` (§10.6); add the 5 fields as `None` to the existing test literals; add a `room_recording_uploaded_request_json_shape` test: build a `RoomEventRequest { entry_kind: "room_recording_uploaded", payload: { …, media_url: Some(..), content_sha256: Some(..), duration_s: Some(..), speakers: Some(vec![..]), attendance_count: Some(..) }, actor_pseudonym: Some("chair-pseudonym") }`, `to_value`, assert `entry_kind`, all 5 recording fields present + correct, `actor_pseudonym` a pseudonym, and that `action`/`from_pseudonym`/`to_pseudonym`/`federated` are ABSENT.
**IMPLEMENT (file 2 of 3):** `stage.rs` — add the 5 recording fields as `None` to the 4 existing `RoomEventPayload` literals (chair_override ×2, transfer, mute_all — `rg "RoomEventPayload\s*\{" services/bridge/src/stage.rs` FIRST to enumerate); add `pub fn record_uploaded(&mut self, media_url, content_sha256, duration_s, speakers, attendance_count)` per §10.5 (push the `room_recording_uploaded` EmitIntent, `actor_pseudonym = self.chair.clone()`, all chair/federated fields None). Add an emit-shape unit test `record_uploaded_emits_recording_intent`: seat a chair, call `record_uploaded(..)`, assert exactly one EmitIntent with `entry_kind == "room_recording_uploaded"`, the 5 recording fields populated, `actor_pseudonym == Some(chair)`.
**IMPLEMENT (file 3 of 3):** `recording.rs` — add `maybe_record` per §10.4 (the `if !enabled { return }` flag-gate → drive the sink + `stage.record_uploaded`). Add the **clean-posture negative-invariant** test `clean_posture_no_side_effects_when_disabled`: a `Recorder` spy + a seated `Stage`; call `maybe_record(false, &mut recorder, &mut stage, …)`; assert the spy recorded **zero** `trigger_egress`/`upload` calls AND `stage.pending_emits.is_empty()`. Add a comment documenting the **delete-the-gate check** (removing `if !enabled { return }` makes the false-case fire side-effects → the test fails). Add a positive companion: `maybe_record(true, &mut recorder, &mut stage, …)` records ≥1 sink call + exactly one EmitIntent (so the false-case asserts the GATE, not an always-skip — R7).

**MIRROR:** §10.4, §10.5, §10.6; `services/bridge/src/stage.rs:316-348` (`mute_all` push + `actor_pseudonym`), `:349-720` (`Recorder` + negative-assertion idioms); `services/bridge/src/room_event_client.rs:80-160` (JSON-shape test).

**GOTCHA (R7 — the clean-posture negative invariant):** the clean-posture test MUST fail if the flag-gate is deleted; pair it with a positive `enabled=true` case so it asserts the GATE, not a trivial always-skip. **GOTCHA (R11):** `record_uploaded`'s `content_sha256` rides `append_room_event` (via the EmitIntent → drain_emits → post_room_event); it is NOT written to `governance_log` from the bridge. **GOTCHA (ADR-015):** `speakers` + `actor_pseudonym` are pseudonyms; `rg -i 'person_id|username|@.*:' services/bridge/src/recording.rs services/bridge/src/stage.rs` returns nothing identity-shaped. **GOTCHA (Cargo-lock):** adding the 5 fields to the bridge struct requires the 5 fields on ALL existing literals in the SAME commit or `brehon-bridge` fails to compile. Bridge compiles on **Linux only**.

**VALIDATE (Linux; feeds §16a Stories 1+3):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-rec-task4-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task4-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml record_uploaded_emits_recording_intent \
  > .claude/PRPs/debug/m3-rec-task4-bridge-emit-test.log 2>&1
echo "emit test exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task4-bridge-emit-test.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled \
  > .claude/PRPs/debug/m3-rec-task4-bridge-clean-test.log 2>&1
echo "clean test exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task4-bridge-clean-test.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test … record_uploaded_emits_recording_intent` + `test … clean_posture_no_side_effects_when_disabled`), commit + push, then **stop**.

### Task 5: Participant-floor recording-fetch endpoint (ADR-015)

**ACTION:** add the recording-fetch HTTP route + handler that rejects a non-participant request before serving and accepts a participant request (the ADR-015 participant-floor — D5 Option C access bar).

**FILES:**

```yaml
creates: []
modifies:
  - services/bridge/src/recording.rs   # is_participant(requester_pseudonym, participants) -> bool + unit test
  - services/bridge/src/appservice.rs  # .route("/brehon/recording/{id}", get(handle_recording_fetch)) + handler
requires:
  - task: 3
    reason: the fetch handler serves the recording's S3 media_url (the LiveSink/S3 surface from Task 3)
```

**IMPLEMENT (file 1 of 2):** `recording.rs` — add `pub fn is_participant(requester_pseudonym: &str, participants: &[String]) -> bool` per §10.7 (membership check; both args pseudonyms). Add a `#[cfg(test)]` test `is_participant_floor`: a participant pseudonym in the set → true; a non-participant pseudonym → false; an empty set → false (the floor cannot be zero — ADR-015).
**IMPLEMENT (file 2 of 2):** `appservice.rs` — add `.route("/brehon/recording/{id}", get(handle_recording_fetch))` to `router()` (`:231-259`) + `async fn handle_recording_fetch(...)` per §10.7: resolve the recording's participant set (from `bridge_room`/room membership), call `recording::is_participant(&requester_pseudonym, &participants)` **before** serving — non-participant → `StatusCode::FORBIDDEN`; participant → `200` + the `media_url`. The requester-pseudonym extraction (from the validated session) is scaffold-grade this phase (Phase-6 wires the live session-auth); the LOAD-BEARING part is the `is_participant`-before-serve gate.

**MIRROR:** §10.7; `services/bridge/src/appservice.rs:198-230` (`handle_room_event` State/Json/auth shape), `:231-259` (`router()`).

**GOTCHA (ADR-015 — §2.4a load-bearing):** `is_participant` is called BEFORE serving — `grep is_participant services/bridge/src/appservice.rs` returns the callsite AND `grep "fn is_participant" services/bridge/src/recording.rs` returns the definition. The participant set + requester are pseudonyms (never `person_id`/MXID). The **strict presigned-URL ACL is DEFERRED** (D5 Option C) — do NOT build presigned URLs / retention / tombstone here (scope error). Bridge compiles on **Linux only**.

**VALIDATE (Linux; feeds §16a Story 4):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-rec-task5-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task5-bridge-check.log   # EXPECT: exit 0
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml is_participant_floor \
  > .claude/PRPs/debug/m3-rec-task5-bridge-test.log 2>&1
echo "test exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task5-bridge-test.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test … is_participant_floor`), commit + push, then **stop**.

### Task 6: Docker-gated end-to-end recording test (`recording.rs` integration)

**ACTION:** add the docker-gated `#[ignore]` integration test exercising the real Egress→MinIO→`content_sha256`→`append_room_event` flow + the `room_recording_uploaded` chain-row assertion + the participant-floor fetch.

**FILES:**

```yaml
creates:
  - services/bridge/tests/recording.rs
modifies: []
requires:
  - task: 3
    reason: exercises compute_content_sha256 + LiveSink (Egress + S3)
  - task: 4
    reason: exercises maybe_record + Stage::record_uploaded (the room_recording_uploaded emission)
  - task: 5
    reason: exercises the participant-floor fetch endpoint
```

**IMPLEMENT:** `recording.rs` per §10 / mirror `services/bridge/tests/stage_mode.rs` + `emergency_mute.rs` — `#[tokio::test] #[ignore = "requires docker-compose stack"]` tests with step comments:
(1) `recording_lands_with_hash_on_chain` — provision a `record_town_halls=true` town-hall room; trigger Egress; assert the MP4 object exists in MinIO; assert a `governance_log` `room_recording_uploaded` row carries the schema `{media_url, content_sha256, duration_s, speakers, attendance_count}` with the real `content_sha256` matching `compute_content_sha256(mp4_bytes)` AND a valid signature/prev-hash link (the hash RODE `append_room_event` — R11, not a bypass); assert `speakers` + actor are pseudonyms (ADR-015).
(2) `clean_posture_no_recording_when_disabled` — provision a `record_town_halls=false` town-hall room; run the session; assert NO MinIO object, NO `room_recording_uploaded` chain row, NO Egress call (the integration-level clean-posture — the deterministic unit is Task 4's `clean_posture_no_side_effects_when_disabled`).
(3) `participant_floor_fetch` — a non-participant fetch → 403; a participant fetch → 200.
`todo!()`-stub the bodies (pilot-grade; the live stack is Phase-6). The deterministic gates (Stories 1–4) run under `cargo-linux.sh test` without docker; these are the Phase-6-pilot-grade signals.

**MIRROR:** `services/bridge/tests/stage_mode.rs` + `services/bridge/tests/emergency_mute.rs` (ignore-stub shape).

**GOTCHA:** the `#[ignore]` tests do NOT run under `cargo-linux.sh test` (they need the live docker stack); the DoD is that they **compile** (`--no-run`). The `content_sha256`-rides-the-chain assertion (R11) is the marquee integration check — the chain row must carry the hash with a valid signature, not a local variable. Bridge compiles on **Linux only**.

**VALIDATE (Linux):**

```bash
scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml \
  > .claude/PRPs/debug/m3-rec-task6-bridge-check.log 2>&1
echo "exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task6-bridge-check.log   # EXPECT: exit 0
# recording.rs is #[ignore]'d — assert it COMPILES (not runs) via --no-run:
scripts/brehon/cargo-linux.sh test  --manifest-path services/bridge/Cargo.toml --test recording --no-run \
  > .claude/PRPs/debug/m3-rec-task6-bridge-itc.log 2>&1
echo "test-compile exit: $?"; tail -20 .claude/PRPs/debug/m3-rec-task6-bridge-itc.log   # EXPECT: exit 0
```

Write a `validate-pending-laptop-linux` DQ (check + clippy + `test --test recording --no-run`), commit + push, then **stop**.

### Task 7: Retro

**Goal:** author the retro per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` — one H2 per role (Advisor / Planning / Impl / BM) with signals + lessons + per-task complexity scores (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`). Promote any new lessons to `.claude/lessons/feedback_*.md` in the retro commit. Specific signals: did the new S3 dep (`rust-s3`) resolve clean on Linux first-try, or surface a Cargo.lock licence/version conflict (the headline-risk row)?; did the `RecordingSink`-trait + `Recorder`-spy mirror of `GrantSink` give a clean deterministic clean-posture test (did the delete-the-gate check actually break the test)?; did the 5-field DTO add propagate across the bridge literals + binary literals compile first-try?; did the Egress-via-reqwest (no LiveKit SDK) decision hold, or did it want the SDK?; did the participant-floor fetch land within scope (no presigned-ACL creep)?

---

## 14. Testing strategy

- **crates static (Windows):** `cargo check --workspace --features full`; `cargo clippy --workspace --features full --no-deps -- -D warnings` (Task 1).
- **crates unit (Windows):** the `governance_log.rs` `#[cfg(test)]` `room_recording_uploaded` serde case (Task 1) runs under check/test.
- **bridge static (Linux):** `cargo-linux.sh check/clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings` (Tasks 2–6).
- **bridge unit (Linux, no docker):** `cargo-linux.sh test --manifest-path services/bridge/Cargo.toml` — `record_town_halls_flag_parse` (Task 2), `content_sha256_is_stable` (Task 3), `record_uploaded_emits_recording_intent` + **`clean_posture_no_side_effects_when_disabled`** (Task 4, the marquee negative invariant), `is_participant_floor` (Task 5). **These ARE the marquee DoD** (deterministic; exercise the real sha256, the real EmitIntent, the real flag-gate, the real authz).
- **bridge integration (Linux, docker-gated, `#[ignore]`):** `tests/recording.rs` — compiles under `--no-run` in CI; runs only against a live docker stack (Phase-6 pilot grade; the real Egress→MinIO→`content_sha256`→`append_room_event` + fetch).
- **No migration round-trip** (no new migration). **No deploy-smoke for the bridge binary** (the MinIO sidecar is m3-core-infra/ops; this phase adds the bridge-side client + emit + fetch only).

## 15. Validation commands (DoD)

> Every command below is written in the exact form the advisor runs at gate 1; all dry-run clean against `phase-m3-core-recording` HEAD. Bridge cargo uses `cargo-linux.sh --manifest-path` (NEVER Windows-local — `ruma-common` E0119). No `rg`-absent / line-count greps in a DoD gate.

### 15.1 crates static analysis (Task 1 — Windows)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-rec-check.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.2 crates lint (Task 1 — Windows, uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/m3-rec-clippy.log 2>&1"
echo "exit: $?"   # EXPECT: 0
```

### 15.3 bridge static + lint + unit (Tasks 2–6 — Linux, Docker rust:1.95)

```bash
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-rec-bridge-check.log 2>&1;  echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings > .claude/PRPs/debug/m3-rec-bridge-clippy.log 2>&1; echo "exit: $?"  # EXPECT: 0
scripts/brehon/cargo-linux.sh test   --manifest-path services/bridge/Cargo.toml > .claude/PRPs/debug/m3-rec-bridge-test.log 2>&1; echo "exit: $?"  # EXPECT: 0 (flag-parse + sha256 + record_uploaded emit + clean-posture + is_participant unit tests pass; #[ignore] integration skipped)
```

### 15.4 bridge integration-test compile (Task 6 — Linux; #[ignore] body not run)

```bash
scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording --no-run > .claude/PRPs/debug/m3-rec-bridge-itc.log 2>&1
echo "exit: $?"   # EXPECT: 0 (the docker-gated test compiles; live run is Phase-6 pilot)
```

### 15.5 Cross-cutting verification (planner asserts at end-of-phase)

- [ ] R8: NO new migration / no new `bridge_room` column; `git diff --stat governance-v0..HEAD -- migrations/ crates/db_schema/migrations/` is empty AND `git diff governance-v0..HEAD -- services/bridge/src/bridge_room.rs` shows ONLY helper-fn additions (no `ALTER TABLE`/new column in the embedded schema).
- [ ] No new const / no registry bump: `git diff governance-v0..HEAD -- crates/db_schema/src/source/governance/governance_log.rs` is EMPTY (Phase 5 adds no const); `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **72** (unchanged).
- [ ] FIRST emitter: `rg 'room_recording_uploaded' services/bridge/src/stage.rs` returns the EmitIntent push site (a `pending_emits.push` with `entry_kind: "room_recording_uploaded"`), NOT a new POST client.
- [ ] Reuse the emit seam: `rg -c 'post_room_event|drain_emits' services/bridge/src/room_event_client.rs` shows the EXISTING client is reused — no new bridge→binary POST client was built.
- [ ] R11 (`content_sha256` rides the chain): `rg 'compute_content_sha256' services/bridge/src/recording.rs` present; `rg -n 'append|governance_log|INSERT' services/bridge/src/recording.rs` shows NO direct chain write (the hash reaches the chain ONLY via the EmitIntent → drain_emits → post_room_event → append_room_event).
- [ ] R12 (generic S3, no hardcoded endpoint): `rg -n 'minio\.' services/bridge/src/` returns nothing (or only comments); the S3 endpoint is read from `config` (`rg 's3_endpoint' services/bridge/src/recording.rs` present).
- [ ] R9 (ADR-015): `rg -i 'person_id|username|@.*:' services/bridge/src/recording.rs services/bridge/src/stage.rs` returns nothing identity-shaped (only pseudonym strings); the fetch handler calls `is_participant` BEFORE serving (`rg is_participant services/bridge/src/appservice.rs` returns the callsite).
- [ ] R10 (ADR-016): the `room_recording_uploaded` EmitIntent payload carries only `{media_url, content_sha256, duration_s, speakers, attendance_count}` + top-level `actor_pseudonym` — the MP4 bytes are NOT in the payload (they go to S3); `append_room_event` receives metadata only.
- [ ] Clean-posture negative invariant: the `recording::clean_posture_no_side_effects_when_disabled` test asserts zero Egress/upload/EmitIntent under `enabled=false` AND FAILS if the flag-gate is deleted (the delete-the-gate mechanical check), with a positive `enabled=true` companion so it asserts the GATE not an always-skip.
- [ ] R14 (deps confined to Task 3, lock synced): `git diff governance-v0..HEAD -- services/bridge/Cargo.toml` shows the new `rust-s3` dep (+ `sha2` promoted transitive→direct, already in the lock); `services/bridge/Cargo.lock` changed in the SAME commit (Task 3); no OTHER task touched `Cargo.toml`.

## 16. Acceptance criteria

- [ ] Tasks 0–7 completed in dependency order
- [ ] §15.1/15.2 (crates check + clippy) exit 0 (Task 1)
- [ ] §15.3 (bridge check + clippy + unit tests) exit 0 (Tasks 2–6)
- [ ] §15.4 (integration-test compiles `--no-run`) exit 0 (Task 6)
- [ ] §15.5 cross-cutting boxes all ticked
- [ ] §16a Stories 1–5 all `[done]`
- [ ] No edits outside §11; entry-kind registry + consts untouched (count stays 72); no migration; exactly one new dep
- [ ] Retro committed (Task 7)
- [ ] **Linux-compile gate:** `validate-pending-laptop-linux` DQ at `result:pass` (Tasks 2–6 touch `services/bridge/**`; Task 3 touches `Cargo.toml`/`Cargo.lock`) before `bm-pr`
- [ ] PR opens against `governance-v0` (NOT `main`) with `--repo barrie-cork/lemmy`

## 16a. Stories

### Story 1: Recording lands in S3 with its `content_sha256` chain entry — the marquee evidentiary record (the hash RIDES `append_room_event`)

- **Composing tasks:** Task 1 (binary recording fields), Task 3 (sha256 + S3 sink), Task 4 (`record_uploaded` EmitIntent)
- **Checkpoint command (deterministic):** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml record_uploaded_emits_recording_intent`
- **Expected output:** test passes — `record_uploaded(..)` pushes exactly one `room_recording_uploaded` EmitIntent carrying the schema `{media_url, content_sha256, duration_s, speakers, attendance_count}` with `actor_pseudonym` = a pseudonym, OMITTING the chair-action/`federated` fields. **Mechanical check (must hold):** the `content_sha256` flows ONLY via the EmitIntent → `drain_emits` → `post_room_event` → `append_room_event` (R11) — `recording.rs` writes no `governance_log` directly. The docker-gated `tests/recording.rs::recording_lands_with_hash_on_chain` (Phase-6) asserts the real chain row carries the hash with a valid signature.
- **Brief-Scope outputs to verify:** `services/bridge/src/stage.rs` contains `pub fn record_uploaded` pushing the `room_recording_uploaded` EmitIntent; `crates/api/api/src/governance/governance_log.rs` `RoomEventPayload` + bridge `room_event_client.rs` `RoomEventPayload` carry the 5 recording fields.

### Story 2: `record_town_halls` reads from the existing `recording_config` column (no new migration)

- **Composing tasks:** Task 2 (config + flag helpers)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml record_town_halls_flag_parse`
- **Expected output:** test passes — `record_town_halls_enabled` returns true for `{"record_town_halls": true}` and false for `{"record_town_halls": false}` / `{}` / garbage (default-false). `read_recording_config`/`write_recording_config` mirror the `chair_id` helpers (INSERT…ON CONFLICT).
- **Brief-Scope outputs to verify:** `services/bridge/src/bridge_room.rs` has `read_recording_config`/`write_recording_config`/`record_town_halls_enabled`; `services/bridge/src/config.rs` has the 4 S3 settings; NO new `bridge_room` column / migration.

### Story 3: `record_town_halls = false` clean-posture — ZERO recording side-effects (the negative invariant, marquee)

- **Composing tasks:** Task 4 (`maybe_record` flag-gate)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml clean_posture_no_side_effects_when_disabled`
- **Expected output:** test passes — `maybe_record(false, ..)` produces zero `trigger_egress`/`upload` calls (observed via the `Recorder` spy) AND `stage.pending_emits.is_empty()`. **Mechanical check (must hold):** deleting the `if !enabled { return }` flag-gate makes this test FAIL (the false-case fires side-effects); a positive `enabled=true` companion proves the test asserts the GATE, not an always-skip (R7).
- **Brief-Scope outputs to verify:** `services/bridge/src/recording.rs` contains `maybe_record` with the `if !enabled { return }` gate before any side-effect.

### Story 4: Participant-floor recording fetch — non-participant rejected, participant accepted (ADR-015)

- **Composing tasks:** Task 5 (fetch endpoint + `is_participant`)
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml is_participant_floor`
- **Expected output:** test passes — `is_participant` returns true for a participant pseudonym, false for a non-participant, false for an empty set (the floor cannot be zero). The handler calls `is_participant` BEFORE serving (403 on false, 200 on true). The docker-gated `tests/recording.rs::participant_floor_fetch` (Phase-6) asserts the live HTTP 403/200.
- **Brief-Scope outputs to verify:** `services/bridge/src/recording.rs` has `pub fn is_participant`; `services/bridge/src/appservice.rs` has the `/brehon/recording/{id}` route + `handle_recording_fetch` calling `is_participant` before serving.

### Story 5: The full recording flow compiles end-to-end (Phase-6-pilot-grade docker test)

- **Composing tasks:** Task 6 (docker-gated integration test), Tasks 3/4/5
- **Checkpoint command:** `scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording --no-run` (compile-gate; live run is Phase-6 pilot)
- **Expected output:** `tests/recording.rs` compiles; its `#[ignore]` step comments assert (1) a `record_town_halls=true` room → MP4 in MinIO + a `room_recording_uploaded` chain row with the real `content_sha256` + a valid signature + pseudonym `speakers`/actor; (2) a `record_town_halls=false` room → zero recording side-effects; (3) non-participant fetch 403 / participant 200.
- **Brief-Scope outputs to verify:** `services/bridge/tests/recording.rs` exists with the `#[ignore]` scenarios asserting the chain-row `content_sha256` (riding `append_room_event`), the clean-posture, and the participant floor.

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Story's checkpoint against the worktree branch, and confirms each Brief-Scope output exists + matches its structural pattern. Phantoms (task complete but output absent/empty) trigger the catch-fire procedure.

## 17. Completion checklist

- [ ] Task 0 audit complete (Probes 0–13 per expectations)
- [ ] Tasks 1–6 committed (one commit each)
- [ ] §15 validation green at every gate (crates Windows / bridge Linux unit / integration-compile)
- [ ] §16a Stories 1–5 all `[done]`
- [ ] Retro committed (Task 7)
- [ ] PR opened by BM against `governance-v0` with `--repo barrie-cork/lemmy`
- [ ] CodeRabbit review triaged per `feedback_pr_review_triage_pattern.md`
- [ ] `/brehon-verify` report shows all stories ✓
- [ ] **Linux-compile gate:** `validate-pending-laptop-linux` DQ at `result:pass` (Tasks 2–6; Task 3 = the new-dep lock-resolution proof) before `bm-pr`
- [ ] Post-merge phase branch retained for retro reads

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| **New S3 dep (`rust-s3`) fails Linux Cargo.lock resolution** — licence/version conflict not visible from the dep line (bootstrap §2 flagged "verify at Cargo.lock resolution time") | **MED** | HIGH | Task 3 isolates the dep-add to ONE task gated by `validate-pending-laptop-linux` (R2/R14); the headline gate proves resolution on Linux BEFORE Tasks 4–6 build on it. If `rust-s3` conflicts, the §19 (2) DQ's fallback (`aws-sdk-s3` with `endpoint_url`, heavier but battle-tested) is the documented alternative — surface as a DQ, do NOT silently swap |
| Marquee test asserts "a recording fired", not the clean-posture zero-side-effect invariant (the negative-invariant trap) | MED | HIGH | §16a Story 3 + R7 + Task 4 GOTCHA: `maybe_record` takes the explicit `enabled` flag; the test asserts zero Egress/upload/EmitIntent under false + the delete-the-gate check + a positive `enabled=true` companion; advisor §3.5 rejects a clean-posture story that is a trivial always-skip |
| `content_sha256` written via a bypass digest instead of riding `append_room_event` (ADR-008/016 violation) | LOW | HIGH | R11 + §15.5 grep: `recording.rs` has NO direct `governance_log`/`append`/`INSERT`; the hash reaches the chain ONLY via the EmitIntent → drain_emits → post_room_event → append_room_event; §16a Story 1 + the docker test assert the chain row carries the hash with a valid signature; bootstrap §7 catch-fire on a bypass write |
| Hardcoded `minio:9000` endpoint breaks the D5 operator-swap story | LOW | MED | R12 + §15.5 grep (`rg minio\. services/bridge/src/` empty); the endpoint reads from `config.s3_endpoint` (Task 2); catch-fire on a hardcoded literal |
| ADR-015 leak — a `person_id`/MXID reaches the `room_recording_uploaded` chain entry (`speakers`/actor) or the fetch serves a non-participant | LOW | HIGH | R9: `speakers` + `actor_pseudonym` are pseudonyms (§15.5 grep); the fetch handler calls `is_participant` BEFORE serving (§2.4a grep DoD); §16a Story 4 asserts the floor |
| 5-field DTO propagation leaves `brehon-bridge` non-compiling (multiple literals) | LOW | MED | §11 caller note: all existing bridge literals + the binary literals get the 5 fields in their owning task's single commit (Task 1 binary, Task 4 bridge); `rg "RoomEventPayload\s*\{"` enumerate-first; §15.1/15.3 catch it |
| Plan re-registers `room_recording_uploaded` / bumps the registry count / adds a migration (scope error) | LOW | HIGH | §12 + Task 0 Probes 3/4/5 (const present, count 72) + R8; §15.5 asserts `db_schema/governance_log.rs` diff empty + no migration; bootstrap tripwire → STOP + `kind: "blocker"` DQ |
| Egress-via-reqwest (no LiveKit SDK) proves insufficient for the Egress trigger | LOW | MED | The Egress trigger is `#[allow(dead_code)]` scaffold this phase (live wiring is Phase-6); §19 (2) documents the reqwest+livekit_jwt path; if Phase-6 needs the SDK it's an additive dep then, not a Phase-5 blocker |
| Bridge Linux cold build (~10–20 min, +new-dep re-resolve at Task 3) stalls serial Tasks 3–6 | MED | LOW | Task 0 Probe 11 warms the registry volume; bridge tasks serial avoids concurrent cold builds; the dep-add is isolated to Task 3 (one cold re-resolve) |

## 19. Notes

**Two scope decisions, both pre-seeded as resolved planner DQs for advisor gate-1 ratification:**

**(1) Recording payload = 5 NEW optional fields on the OUTBOUND `RoomEventPayload` struct, NOT a new inbound enum variant. (resolved planner DQ `1ed255d85572-001`.)** The brief §2/§3 hypothesised "a NEW `RoomEventPayload` variant (today only `PrivateMessage` + `CaseTransition` — `room_provisioner.rs:43`)". That conflates two distinct types: `room_provisioner::RoomEventPayload` is the **inbound** appservice-handler dispatch enum (the binary→bridge `handle_room_event` payload); the recording emission rides the **outbound** `room_event_client::RoomEventPayload` struct (the bridge→binary emit payload — the same struct `mute_all`/`chair_override` use, mirrored on the binary side at `governance_log.rs`). The recording schema `{media_url, content_sha256, duration_s, speakers, attendance_count}` therefore goes as 5 NEW `#[serde(default, skip_serializing_if="Option::is_none")]` fields on the OUTBOUND struct (binary + bridge mirror), **exactly mirroring emergency-mute's `federated` add** (one field → five fields, identical pattern) — the canonical-schema-first precedent. The binary constructs zero non-test `RoomEventPayload {…}` literals (`rg` empty), so the add is non-breaking; `skip_serializing_if` keeps existing room emissions' hashed JSON byte-identical. This is the one in-scope `crates/**` touch (the bootstrap trivial-DTO carve-out — NOT a const/migration/registry change). The uploader/chair actor rides the existing top-level `actor_pseudonym` (as `mute_all`/`chair_*` do), not a payload field. **Advisor: ratify the 5-field outbound-struct add at gate-1.** Resolved (planner): outbound struct, 5 optional fields, mirror `federated`.

**(2) The one new dep = a generic-S3 client (`rust-s3`); the Egress trigger reuses `reqwest` + `livekit_jwt` (no LiveKit SDK). (resolved planner DQ `1ed255d85572-002`; the brief's "most likely DQ".)** OQ-V2-04 (D5) mandates a **generic S3 API** so an operator can swap real S3/R2 (not MinIO-specific). Options for the S3 client:

| Crate | For | Against |
|---|---|---|
| **`rust-s3`** (crate `rust-s3`, lib `s3`) ✅ recommended | Purpose-built for generic S3-compatible stores (MinIO/R2/Backblaze); custom-endpoint-first (the D5 swap story); lighter dep tree → lower Linux Cargo.lock resolution risk; async/tokio | Smaller maintainer base than the AWS SDK |
| `aws-sdk-s3` (fallback) | Battle-tested, official; supports custom `endpoint_url` | Heavy dep tree (aws-config/aws-smithy/hyper) → higher Linux-resolution risk + longer cold builds; AWS-shaped API for a generic-S3 goal |

**Recommendation: `rust-s3`** — it matches the D5 "generic S3, operator-swappable" intent directly and minimises the headline Linux Cargo.lock resolution risk (one lighter dep). For the **Egress trigger**, reuse the existing `reqwest::Client` + a `livekit_jwt`-minted token to POST the LiveKit Egress start request (the Egress API is JSON-over-HTTP reachable with the existing stack) — this avoids a second heavy new dep (a LiveKit Rust SDK pulls tonic/protobuf), keeping the ONLY new dep the S3 client (R14). The `sha2`+`hex` for `content_sha256`: `hex` is already a direct dep; `sha2` is currently **transitive-only** (in `Cargo.lock`, not `Cargo.toml`) — Task 3 promotes it to a direct dep (`sha2 = "0.10"`), which is zero new resolution risk since it is already locked. So `rust-s3` is the ONLY newly-fetched dep. **If `rust-s3` fails Linux Cargo.lock resolution (the §18 row-1 risk), surface a DQ and fall back to `aws-sdk-s3` (endpoint_url) — do NOT silently swap.** **Advisor: ratify `rust-s3` + Egress-via-reqwest at gate-1.** Resolved (planner): one new dep (`rust-s3`); Egress via reqwest+livekit_jwt.

**Marquee DoD is two deterministic unit tests, not the docker stack.** Per `feedback_build_what_tests_exercise` + `feedback_authz_state_machine_test_asserts_negative` + `pattern_test_against_reality_not_syntax`: (a) the `room_recording_uploaded` emission shape (`record_uploaded_emits_recording_intent`) over the real `Stage` + a real EmitIntent, and (b) the **clean-posture zero-side-effect negative invariant** (`clean_posture_no_side_effects_when_disabled`) over the real `maybe_record` flag-gate + a `Recorder` `RecordingSink` spy (zero Egress/upload/EmitIntent under false; the delete-the-gate check breaks it). The real Egress→MinIO→`content_sha256`→`append_room_event` + the live fetch 403/200 are the docker-gated `#[ignore]` `tests/recording.rs` (Phase-6 pilot grade). This split is honest: the deterministic tests prove the gate + the emission + the authz; the pilot proves the live Egress/S3/fetch.

**No new migration, no new sidecar code in-binary, no new const, exactly one new dep.** Phase 5 composes m2's `room_recording_uploaded` const, stage-mode's emit seam (`EmitIntent`/`pending_emits`/`drain_emits`/`post_room_event`), m3-core-infra's `recording_config` column + `livekit_jwt` + `livekit_*` config, and emergency-mute's `federated` trivial-DTO precedent. The ONLY net-new surface is the generic-S3 client + `recording.rs` (Egress trigger + sha256 + sink + fetch authz). Complexity 3/10. If any task is tempted to add a const, a column, a migration, a second dep, or a new emitter, STOP (bootstrap tripwire / §7 catch-fire).

## 20. Confidence score

- **Plan correctness:** 8/10 — the two scope decisions (outbound-struct add vs inbound-enum-variant; `rust-s3` + Egress-via-reqwest) are reconciled against the live code (the outbound/inbound `RoomEventPayload` distinction verified at `room_event_client.rs:6` vs `room_provisioner.rs:43`; the `recording_config` column + helper pattern + emit seam + const all verified present). The headline judgment call is the S3-client choice (defended above + the `aws-sdk-s3` fallback + the gate-1 ratify + the Task-3 Linux gate).
- **Cargo budget:** 8/10 — no daemon cargo; the only time cost is the bridge cold build + the one new-dep re-resolve at Task 3, mitigated by Task-0 warm-up + serial bridge tasks.
- **Test coverage:** 8/10 — the clean-posture zero-side-effect negative invariant (the cr-class application site), the `room_recording_uploaded` emission shape, the `content_sha256`-rides-the-chain constraint, and the participant-floor authz are all §16a stories with green deterministic checkpoints; the live Egress→MinIO→fetch flow is a compile-gated `#[ignore]` test deferred to the Phase-6 pilot by scope.
