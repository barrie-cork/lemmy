## 1. Role + dispatch line

`[role:impl-task] m3-core-recording Task 6 — docker-gated end-to-end recording integration test (compile-only)`

Dispatch string:
```
[role:impl-task] m3-core-recording-task6-recording-itc — see .claude/PRPs/briefs/m3-core-recording-impl-6.md
```

## 2. Scope

Add ONE new bridge integration-test file with three `#[ignore]`'d end-to-end tests that **compile but do not run** (they need a live docker-compose stack — that's Phase-6). Exactly **one file created, zero modified**, **one crate** (`brehon-bridge`, workspace-excluded, Linux-validated). Serial task — requires Tasks 3, 4, 5 (all shipped on the phase branch).

**File 1 of 1 — `services/bridge/tests/recording.rs`** (CREATE):
Mirror the ignore-stub shape of `services/bridge/tests/stage_mode.rs` + `emergency_mute.rs` EXACTLY:
- A file-level doc-comment block: title line, a `Run with:` block (`cd services/bridge` → `docker compose up -d` → `cargo test --test recording -- --ignored` → `docker compose down`), and the `All test functions are #[ignore]'d so bare cargo test … skips this suite` note.
- Three `#[tokio::test] #[ignore = "requires docker-compose stack"] async fn … -> anyhow::Result<()>` functions, each with numbered `//` step comments per §13 Task 6 IMPLEMENT, body `todo!("implement against live docker-compose stack (Phase-6 pilot grade)")`:
  1. **`recording_lands_with_hash_on_chain`** — step comments: provision a `record_town_halls=true` town-hall room; trigger Egress; assert the MP4 object exists in MinIO; assert a `governance_log` `room_recording_uploaded` row carries the schema `{media_url, content_sha256, duration_s, speakers, attendance_count}` with the real `content_sha256` matching `compute_content_sha256(mp4_bytes)` AND a valid signature/prev-hash link (**the hash RODE `append_room_event` — R11, not a bypass digest**); assert `speakers` + actor are pseudonyms (ADR-015).
  2. **`clean_posture_no_recording_when_disabled`** — step comments: provision a `record_town_halls=false` town-hall room; run the session; assert NO MinIO object, NO `room_recording_uploaded` chain row, NO Egress call (integration-level clean-posture; the deterministic unit is Task 4's `clean_posture_no_side_effects_when_disabled`).
  3. **`participant_floor_fetch`** — step comments: a non-participant fetch → 403; a participant fetch → 200.

### 2.1 Boundaries (do NOT)

- Do **NOT** implement the test bodies — `todo!()`-stub them (pilot-grade; the live stack is Phase-6). The DoD is that the file **compiles**, not that the tests run.
- Do **NOT** modify any other file. `modifies: []` — Tasks 3/4/5 already shipped every production symbol these tests reference (compile-only, so the stub bodies need no live symbols).
- Do **NOT** add a dependency, a `docker-compose.yml`, a fixture, or any live-stack scaffolding (R8 — Task 6 adds zero deps).
- Do **NOT** run cargo on the daemon (R3). Write the `validate-pending-laptop-linux` DQ and **stop** (see §4).

## 3. Required reading

Read these BEFORE writing the file:

- `.claude/PRPs/plans/m3-core-recording.plan.md` §13 Task 6 (FILES/IMPLEMENT/MIRROR/GOTCHA/VALIDATE — copy the three test names + step-comment intent verbatim), §14 Testing strategy, R11 (`content_sha256` rides the chain — the marquee integration assertion in test 1's comments).
- `services/bridge/tests/stage_mode.rs` (the canonical ignore-stub shape to mirror — file-level doc block + `#[tokio::test] #[ignore] async fn -> anyhow::Result<()>` + numbered step comments + `todo!(...)` body). **Read it first; match its structure exactly.**
- `services/bridge/tests/emergency_mute.rs` (second ignore-stub sibling — confirms the `anyhow::Result<()>` return + `#[ignore = "requires docker-compose stack"]` attribute string).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` (R1 — bridge cargo is Linux-only via `cargo-linux.sh`; the Windows-local `cd services/bridge && cargo` form fails `ruma-common` E0119).
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` (R3 — write the DQ, do NOT run cargo on the daemon).

**Note — these lessons do NOT mechanically fire (read for negative confirmation only):**
- `feedback_lemmy_error_no_std_error.md` — N/A: bridge tests return `anyhow::Result<()>`, NOT `Result<(), Box<dyn Error>>`, and this is `services/bridge/tests/`, not `crates/*/tests/`.
- `feedback_forward_declared_items_need_allow_until_consumer.md` — N/A: this task creates NO production symbols; it only adds `#[ignore]`'d test fns whose bodies are `todo!()`. No dead-code / no struct-field propagation.
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — N/A: this is NOT `crates/server/tests/e2e.rs`; it's a NEW small bridge test file (Write, not multi-Edit).

## 4. Constraints

### 4.1 Compile-only is the DoD (not run)

The three tests are `#[ignore]`'d because they need the live docker-compose stack (Phase-6). The DoD is **they compile** — proven by `cargo-linux.sh test --test recording --no-run` exiting 0. `todo!()` bodies compile fine (they're `-> !` and satisfy any return type). The deterministic gates (Stories 1–4 unit tests) already run under `cargo-linux.sh test` from Tasks 1–5; these `#[ignore]` tests are the Phase-6-pilot-grade signal.

### 4.2 R11 lives in test 1's step comments (load-bearing intent)

The `content_sha256`-rides-the-chain assertion is the marquee integration check. Even as a `todo!()`-stub, test 1's step comments MUST spell out: the `room_recording_uploaded` chain row carries the hash **with a valid signature/prev-hash link** (it rode `append_room_event`), NOT a local variable / side-channel digest. This is the R11 guardrail made visible in the spec so Phase-6 implements it correctly. Do NOT water this comment down to "assert the row exists."

### 4.3 ADR-015 pseudonyms in test 1's step comments

Test 1's comments MUST note `speakers` + the uploader actor are **pseudonyms** (ADR-015) — never `person_id`, username, or MXID. Mirror how `stage_mode.rs`'s comments call out the PSEUDONYM payload fields.

### 4.4 Validate-pending-laptop-linux DQ then STOP (R2 + R3)

After committing + pushing the worker branch, write a `validate-pending-laptop-linux` DQ entry (use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`) with:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop-linux",
  "question": "Run cargo-linux.sh check + clippy + recording.rs test-compile for bridge task 6",
  "options": ["pass", "fail"],
  "context": "m3-core-recording Task 6 — docker-gated #[ignore] recording integration test (compile-only). Bridge Linux check + clippy -D warnings + test --test recording --no-run (the #[ignore] tests must COMPILE, not run).",
  "commands": [
    "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test recording --no-run"
  ],
  "branch": "<your worker branch>",
  "phase_task": "m3-core-recording Task 6",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```
Commit the DQ write (`chore(decision-queue): impl raised validate-pending-laptop-linux <id> — m3-core-recording task 6`), push, then **STOP**. Do **NOT** run cargo yourself — the laptop advisor is the runner (R3).

### 4.5 Mid-task blocker discipline

This task is mechanically small (one Write of a stub-test file mirroring two siblings). If anything is ambiguous — e.g. a sibling-test attribute string differs from what §13 says, or `anyhow` is unexpectedly absent from the bridge dev-deps — do **NOT** guess. Raise a `kind: "blocker"` DQ (`from: "impl"`), commit + push it, and stop.

## 5. LESSON trailer

End your final commit body with a `LESSON:` line only if you hit anything durable (e.g. a bridge `[[test]]` Cargo.toml requirement for a new test file name, an `anyhow` vs `Result` mirror gotcha).
