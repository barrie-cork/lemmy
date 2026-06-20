# Brief: m3-core-e2e-pilot Task 5 [P] — recording.rs LIVE (recording-lands + clean-posture-off + participant-floor)

## §1 Role + dispatch line

`[role:impl-task] m3-core-e2e-pilot-task5-recording-live-e2e — see .claude/PRPs/briefs/m3-core-e2e-pilot-impl-5.md`

You are the **impl-task** subagent (Sonnet 4.6). Turn the 3 `recording.rs` integration scenarios from `#[ignore] todo!()` into LIVE bodies against the e2e harness + Task 2's live LiveSink. Bridge-src tests, **Linux-validated**. Part of Cohort A (with Task 3 `stage_mode.rs` — DISJOINT file).

## §2 Scope

**Plan task: `.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 5** (read it — full ACTION/IMPLEMENT/MIRROR/GOTCHA/VALIDATE) + §16a Stories 3/4/5 (criteria 143/144/145).

**Produces (1 file modified, ONE commit):**

- **`services/bridge/tests/recording.rs`** — turn the 3 scenarios from `todo!()` into LIVE tests (keep `#[tokio::test] #[ignore = "requires docker-compose stack"]`):
  1. **`recording_lands_with_hash_on_chain`** (criterion 143): provision a `record_town_halls=true` room; trigger Egress (Task 2's live LiveSink); assert the MP4 object EXISTS in MinIO (S3 object-exists, endpoint from config — R-S3ENDPOINT); assert a `governance_log` `room_recording_uploaded` row carries `{media_url, content_sha256, duration_s, speakers, attendance_count}` where `content_sha256 == compute_content_sha256(mp4_bytes)` AND the row has a valid signature/prev-hash link (**R-CHAIN** — the hash RODE `append_room_event`, NOT a MinIO-metadata-only check); assert `speakers` + actor are PSEUDONYMS (ADR-015).
  2. **`clean_posture_no_recording_when_disabled`** (criterion 145, NEGATIVE invariant): provision a `record_town_halls=false` room; run the session; assert NO MinIO object, NO `room_recording_uploaded` chain row, NO Egress call (R7 — the integration-level clean-posture invariant; FAILS if the `maybe_record` flag-gate is removed).
  3. **`participant_floor_fetch`** (criterion 144, cr-2/cr-3): non-participant pseudonym fetch → 403; participant pseudonym fetch → 200 + `media_url` (exercises Task 2's cr-2/cr-3 live fetch).

**Do NOT:**
- Touch any file other than `services/bridge/tests/recording.rs` (Task 3 owns `stage_mode.rs` — Cohort A is file-disjoint; DO NOT edit stage_mode.rs/recording.rs-SRC/appservice.rs/etc — only the TEST file `tests/recording.rs`).
- Touch the harness composes/smoke (Task 1 — GREEN), the recording SRC (`src/recording.rs` — Task 2 landed it), any `crates/**`, migration, const, registry, or plan.
- Add identity-shaped data — pseudonyms only (ADR-015).
- Add a dependency.

**Branch:** forks from `phase-m3-core-e2e-pilot` (current tip `5abf83328`).

## §3 Required reading

- **`.claude/PRPs/plans/m3-core-e2e-pilot.plan.md` §13 Task 5** (~lines 497-520) + §16a Stories 3/4/5 (~671-691).
- **`services/bridge/tests/recording.rs`** (the scaffold — 3 `todo!()` bodies at `:14,33,47`; anchor on the ENCLOSING fn signature line first since there are 3 todo!()s in the file, then the todo!() line).
- **`services/bridge/src/recording.rs`** (Task 2's landed live LiveSink + `is_participant` + `compute_content_sha256` + `Recorder`) — the seams the test EXERCISES.
- **`services/bridge/src/appservice.rs:238-284`** (Task 2's live `handle_recording_fetch` with cr-2/cr-3) — the fetch the participant-floor test hits.
- **`services/bridge/src/stage.rs:373-410`** (`record_uploaded` — the EmitIntent → chain path).
- **`services/bridge/tests/room_provisioning.rs`** — the `AsyncPgConnection::establish` + `governance_log` query idiom to MIRROR.
- **Lessons (mandatory):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo via cargo-linux.sh; you run NO cargo.
  - `feedback_authz_state_machine_test_asserts_negative.md` — clean-posture is a NEGATIVE invariant (FAILS if the gate is deleted).
  - `feedback_build_what_tests_exercise.md` + `pattern_test_against_reality_not_syntax.md` — assert OBSERVABLE state (real MinIO object + real chain row), not config.
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate each `todo!()` anchor (3 in this file → anchor on enclosing fn sig).
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP.

## §4 Constraints

- **R-CHAIN (CATCH-FIRE on bypass):** `recording_lands_with_hash_on_chain` MUST assert the `content_sha256` appears in the `governance_log` chain row with a valid signature/prev-hash link — NOT merely in the MinIO object's metadata. The hash rode `append_room_event` (scrub_json + ed25519 chain + signing). A MinIO-metadata-only assertion is insufficient.
- **R7 (clean-posture NEGATIVE invariant — load-bearing):** `clean_posture_no_recording_when_disabled` MUST assert NO MinIO object + NO chain row + NO Egress call, and MUST fail if the `if !enabled { return }` flag-gate is removed. NOT a trivial always-skip. **Why:** `record_town_halls=false` is the default clean posture; a recording firing under the false flag is silent data capture.
- **R-ANCHOR (pre-locate anchors):** `grep -c 'todo!("implement against live docker-compose stack (Phase-6 pilot grade)")' services/bridge/tests/recording.rs` is `3` — anchor each edit on its ENCLOSING fn signature line first (the 3 todo!()s are not individually unique), then the todo!() line.
- **ADR-015 (pseudonymity):** `speakers`/actor/participants/requester are PSEUDONYMS. **DoD:** `rg -i 'person_id|username|@.*:' services/bridge/tests/recording.rs` returns nothing identity-shaped.
- **R-S3ENDPOINT:** the MinIO object-exists check reads the endpoint from config/test-harness env, not a hardcoded literal.
- **Keep `#[ignore]`:** all 3 stay `#[tokio::test] #[ignore = "requires docker-compose stack"]`.
- **NO daemon cargo.** Write TWO DQs (`bash scripts/brehon/dq-v3-new-entry.sh` for ids, `dq-v3-append-fragment.sh <frag>.json --pending`):
  1. `validate-pending-laptop-linux` (worker COMPILE gate):
     ```
     kind: "validate-pending-laptop-linux"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 5
     commands: [
       "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
       "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
       "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording --no-run"
     ]
     result: null
     log_slice: null
     failed_commands: null
     ```
  2. `validate-pending-laptop-e2e` (LIVE run — gate-4=LOCAL):
     ```
     kind: "validate-pending-laptop-e2e"
     branch: "phase-m3-core-e2e-pilot"
     phase_task: 5
     commands: ["scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording -- --ignored"]
     e2e_filter: null
     result: null
     log_slice: null
     failed_commands: null
     ```
  Commit + push BOTH on the worker branch, then **STOP**.
- **ONE commit** — `feat(rtc): recording.rs live e2e — recording-lands-on-chain + clean-posture-off + participant-floor (task 5)`. `LESSON:` trailer if durable.
- **No stray temp artifacts:** delete any DQ fragment file after appending (do NOT commit it — Task 2 left a stray; don't repeat).
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- **Handover:** inline (no handover file).

## §3a Handover from prior tasks

- Task 1 — e2e harness GREEN (MinIO + LiveKit + 2nd instance; reach-smoke OK). Your `-e2e` live run executes against this.
- Task 2 (`5abf83328`) — live LiveSink (async Egress + S3 PUT) + cr-2/cr-3 (`handle_recording_fetch` real participant set, 403 empty pseudonym). YOUR participant-floor test hits this live fetch; YOUR recording-lands test triggers the live LiveSink. Phase tip `5abf83328`.
- Cohort A: you (Task 5 recording.rs) run alongside Task 3 (stage_mode.rs) — DISJOINT files.

## §5 HANDOVER (worker fills at task end — return inline)

```yaml
HANDOVER:
  task: m3-core-e2e-pilot-task5
  filesModified: [services/bridge/tests/recording.rs]
  keyDecisions:
    - "recording_lands_with_hash_on_chain: MinIO object exists + governance_log room_recording_uploaded row w/ content_sha256 matching + valid signature (R-CHAIN)"
    - "clean_posture_no_recording_when_disabled: NO object/chain-row/Egress (R7 negative invariant)"
    - "participant_floor_fetch: non-participant 403, participant 200+media_url (cr-2/cr-3 live)"
    - "all 3 kept #[ignore] (need live stack)"
  validate_dq_linux: <id>
  validate_dq_e2e: <id>
  notes: "<confirm 3 todo!() anchors located via enclosing fn sig; confirm R-CHAIN asserts the chain row not just MinIO metadata; confirm pseudonym-only; confirm only tests/recording.rs edited; confirm no stray dq-frag committed>"
```
