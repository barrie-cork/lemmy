# Brief: m3-core-stage-mode impl-6 (Task 6 — stage-mode provisioning + Q&A sidebar + drain-emits + docker-gated e2e)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-task6-stage-mode-provisioning-drain-emits-e2e — see .claude/PRPs/briefs/m3-core-stage-mode-impl-6.md`

## §2 Scope

**Task 6 of plan `.claude/PRPs/plans/m3-core-stage-mode.plan.md` (lines 602–639).** The LAST impl task. Extend the bridge town-hall provisioning to open in **stage mode** (chair presenter, watchers muted), seat the dual-sourced chair into `chair_id`, attach the Q&A sidebar (= Matrix text timeline), **wire the async controller drain of `Stage::pending_emits → post_room_event`** (the user-decision extension — see §2.1), and add the docker-gated `#[ignore]` end-to-end test.

`requires: task 2` (presenter/watcher token mint — landed) + `task 5` (`Stage::pending_emits` + `post_room_event` + `room_event_client` — landed).

**Produces (1 new file + 2 modified, ONE commit):**

1. **MODIFY** `services/bridge/src/room_provisioner.rs`:
   - Add `#[serde(default)] pub chair_pseudonym: Option<String>` to the bridge `CaseTransitionEvent` mirror (`:19-33` — the struct with `case_id`/`juror_pseudonyms`/`new_status`/`old_status`/`community_id`). Mirror the existing `#[serde(default)]` idiom. This field is READ here (Phase-3); the binary-side population is Phase-6.
   - Add a town-hall stage-mode provisioning path (extend `provision_community_event_room` `:187` or add a sibling `provision_townhall_stage_room`). Mirror the `:183-237` shape: `bridge_room::open` → `bridge_room::lookup` idempotent-skip → `provision::create_community_room` → `bridge_room::upsert`. **Additionally**, on `state.config.rtc_enabled`:
     - Seat `chair_id` from `event.chair_pseudonym` ?? `event.juror_pseudonyms.first()` (foreperson fallback, OQ-V2-05); `bridge_room::write_chair_id`.
     - Initialise `queue_state` to `[]` (`bridge_room::write_queue_state(conn, case_id, room_type, "[]")`).
     - Mint the chair a **presenter token** (`mint_access_token(..., can_publish = true)`) and watchers **watcher tokens** (`can_publish = false`). (Token plumbing is the Task-2 `can_publish` param; if there is no participant list at provisioning time, mint only the chair presenter token + leave watcher minting to join-time — note which in the commit body.)
   - The Q&A sidebar is the Matrix room's **native text timeline** — NO new room type, NO content field, NO hashing (ADR-016). Provisioning the Matrix room IS provisioning the Q&A sidebar; add a one-line comment saying so.

2. **MODIFY** `services/bridge/src/room_event_client.rs` — **wire the drain (the user-decision extension, §2.1) + REMOVE the 3 dead_code allows:**
   - Add `pub async fn drain_emits(stage: &mut crate::stage::Stage, client: &reqwest::Client, url: &str, secret: &str)` that loops `for intent in stage.pending_emits.drain(..) { if let Err(e) = post_room_event(client, url, secret, intent.entry_kind, intent.payload, intent.actor_pseudonym).await { tracing::warn!("room-event POST failed (non-fatal — best-effort chain entry): {e}"); } }`. **Fire-and-forget warn-on-error** per `bridge_notify.rs:62-72`.
   - **REMOVE** `#[allow(dead_code)]` from `post_room_event` (`:38`), from `RoomEventRequest` (`:28`), and (in `config.rs`) from `brehon_room_event_url` (`:37`) — `drain_emits` is the real production caller; `post_room_event` constructs `RoomEventRequest`; the provisioning path reads `config.brehon_room_event_url`. Once `drain_emits` is reachable from a non-test caller, all three are live and clippy `-D warnings` passes WITHOUT the allows.
   - **DO NOT touch** `RoomEventPayload` or its fields (used) or any `room_event_client` test.

3. **CREATE** `services/bridge/tests/stage_mode.rs` per §10.9 — the docker-gated `#[ignore]` end-to-end test (`four_mic_pass_then_grace_boundary_emits_chair_entries`). Mirror `services/bridge/tests/room_provisioning.rs:12-21,36-42` (the `#[tokio::test] #[ignore = "requires docker-compose stack"]` + step-comment + `todo!(...)` convention). Steps (comments only, `todo!()` body — pilot-grade, live run is Phase-6):
   - provision a stage room (chair + 4 watchers); drive 4 raise-hand + promote/activate passes; drive a 5th promote with no activation → assert auto-revoke + next-promote at 30s; drive a chair transfer + override; query `governance_log` → assert `room_chair_transferred {from_pseudonym, to_pseudonym, at}` + `room_chair_override {action, target_pseudonym}` rows exist with **PSEUDONYM** payload fields (never `person_id` / `@user:domain` MXID).

### §2.1 The drain wiring — WHY it's load-bearing (user decision 2026-06-19)

Task 5 used the **emit-intent seam**: `transfer_chair`/`chair_override`/`promote_next` stay synchronous and push an `EmitIntent { entry_kind, payload, actor_pseudonym }` to `Stage::pending_emits: Vec<EmitIntent>` (keeping the 11 stage tests green). The actual async POST to the binary was deferred — which left `post_room_event` + `RoomEventRequest` + `brehon_room_event_url` with **no production caller**, flagged `dead_code` and carried under `#[allow(dead_code)]` (fix-impl-5).

**The user decided (2026-06-19):** Task 6 wires the async controller drain (`drain_emits`) AND removes the 3 allows now that the caller exists. **Why it can't be deferred again:** leaving the allows means the emit path is never exercised by production code — `pending_emits` would accumulate forever with nothing draining them, and the FIRST-emission DoD (`room_chair_transferred`/`room_chair_override` actually POST) would be a phantom. The drain is the seam closure that makes the emit-intent design real.

**Scope-honest note:** there is currently NO production HTTP endpoint that calls `Stage::transfer_chair`/`chair_override` (only `#[cfg(test)]` callers). So at provisioning time `pending_emits` is empty and `drain_emits` loops zero times — `post_room_event` is **reachable** (clippy dead_code satisfied, allows removable) but not yet *driven* by a live stage-action. The live stage-action HTTP endpoint that populates `pending_emits` and calls `drain_emits` per-action is **Phase-6** (the binary-side chair-action trigger). Task 6 ships the drain function + makes it reachable from the provisioning path (even an empty drain after seat-chair counts as a real caller). Do NOT build the stage-action HTTP endpoint — that is Phase-6 (§12). If `drain_emits` cannot be made reachable from a non-test caller without building the Phase-6 endpoint, raise a `kind: "blocker"` DQ rather than guessing.

**Do NOT:**
- Build the Phase-6 stage-action HTTP endpoint (the live transfer/override/promote trigger). Task 6 ships the drain function + provisioning + e2e stub only.
- Add content hashing / a Q&A content field / a new room type — the Q&A sidebar IS the Matrix text timeline (ADR-016).
- Run the `#[ignore]` test body (it needs the live docker stack); it must COMPILE under `--no-run` only.
- Touch any `crates/**`, any migration, `Cargo.toml`/`Cargo.lock` (no new dep; reqwest/serde/anyhow already bridge deps).
- Break any of the 11 `stage` tests or the 2 `room_event_client` tests.
- Re-add a new `ENTRY_KIND_*` const or touch the entry-kind registry (`ROOM_KINDS` admits the kinds already — M3 Phase 2).

**Branch:** forks from `phase-m3-core-stage-mode` (tip AFTER Task 5 + fix-impl-5 finalize-merge — the advisor will state the exact SHA in the dispatch; it has Tasks 1–5 + the 3 dead_code allows that THIS task removes).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — Task 6 (602–639), **§10.8** (provisioning, 300–304), **§10.9** (test harness skeleton, 306–324), §10.10 (ADR-015 pin), §10.11 (ADR-016 metadata-only), §16a Story 5 (743–748).
- `services/bridge/src/room_provisioner.rs:19-33` — the `CaseTransitionEvent` mirror (add `chair_pseudonym`). `:183-237` — `provision_community_event_room` (the provisioning MIRROR: open/lookup/create/upsert idempotent shape). `:573-591` — shared reqwest client handle shape.
- `services/bridge/src/room_event_client.rs` (WHOLE file) — `post_room_event` (`:39`), `RoomEventRequest` (`:29`), `RoomEventPayload` (`:6`, do NOT touch), the 3 `#[allow(dead_code)]` to remove (`:28`, `:38`, + config.rs `:37`). `EmitIntent` is in `stage.rs:47`.
- `services/bridge/src/stage.rs:47-69` — `EmitIntent { entry_kind: &'static str, payload: RoomEventPayload, actor_pseudonym: Option<String> }` + `Stage::pending_emits` field (`:68`). The drain consumes these directly (field names match `post_room_event` args).
- `services/bridge/src/main.rs` — `AppState` (`http_client: reqwest::Client` at `:62`; `config: Arc<BridgeConfig>`). The drain takes `&state.http_client`, `&config.brehon_room_event_url`, `&config.bridge_callback_secret`.
- `crates/api/api_utils/src/bridge_notify.rs:62-72` — **the fire-and-forget warn-on-error MIRROR** for the drain callsite (`if let Err(e) = ...post(...).send().await { tracing::warn!("... non-fatal: {e}"); }`).
- `services/bridge/tests/room_provisioning.rs:12-42` — the `#[ignore]`-stub + step-comment + `todo!()` convention to mirror for `stage_mode.rs`.
- **Lessons (mandatory):**
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — the ADR-015 pseudonym gate (§4) is **load-bearing**, not a nit.
  - `feedback_build_what_tests_exercise.md` — provisioning seats chair + mints tokens; the `#[ignore]` e2e asserts REAL `governance_log` rows (not a struct read).
  - `feedback_governance_type_state_handlers.md` — provision only on the valid transition; idempotent-skip via `bridge_room::lookup`.
  - **Bridge file-class (§2.4 — `services/bridge/**`):**
    - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
    - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.
    - `feedback_clippy_test_style.md` — no `unwrap`/`expect`/`unwrap_or_default`; tests return `Result<()>` + `?` + `.context()`.

## §4 Constraints

- **ONE commit, ≤3 files** (`room_provisioner.rs` + `room_event_client.rs` + new `tests/stage_mode.rs`; config.rs allow-removal is part of the same commit if the allow is there — count it as a 4th touch only if needed) — `feat(rtc): stage-mode provisioning + Q&A sidebar + room-event drain + e2e stub (task 6)`.
- **REMOVE the 3 `#[allow(dead_code)]`** (`post_room_event`, `RoomEventRequest` in room_event_client.rs; `brehon_room_event_url` in config.rs). **DoD: `grep -rn 'allow(dead_code)' services/bridge/src/room_event_client.rs` returns NOTHING; `grep -n 'allow(dead_code)' services/bridge/src/config.rs` returns ONLY the 3 livekit fields (lines ~54/57/60), NOT brehon_room_event_url.** State both grep results in the commit body.
- **`drain_emits` MUST be reachable from a non-test caller** (the provisioning path) — that is what removes the dead_code. **DoD: `grep -n 'drain_emits' services/bridge/src/room_provisioner.rs` returns a callsite** (or another non-test prod path). If you cannot wire a reachable caller without building the Phase-6 endpoint → blocker-DQ.
- **ADR-015 (pseudonyms-only) — LOAD-BEARING:** `chair_id` is seated from `chair_pseudonym` ?? `juror_pseudonyms.first()` — both **pseudonyms**. The e2e asserts every chair-payload field is a pseudonym string. **Why it can't be deferred:** a real identity in a hash-chained chair entry is permanent + unscrubable (GDPR violation in the immutable log). **DoD: `rg -i 'person_id|username|mxid|@.*:' services/bridge/src/room_provisioner.rs services/bridge/tests/stage_mode.rs` returns NOTHING identity-shaped.** State the grep result in the commit body.
- **ADR-016 (metadata-only):** the Q&A sidebar is the Matrix text timeline — NEVER hashed or POSTed to the chain. No content field on any payload. No new room type.
- **Watcher tokens `can_publish=false`, chair `can_publish=true`** — stage mode = one presenter slot at open.
- **No `unwrap`/`expect`/`unwrap_or_default`** (clippy `-D warnings`). Propagate with `?` + `.context()`.
- **NO new dependency.** **NO new migration** (no `bridge_room` column add — `chair_id`/`queue_state` already exist from Task 3).
- **The `#[ignore]` e2e must COMPILE under `--no-run`, not run.**
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --test stage_mode --no-run", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- End the commit body with a `LESSON:` trailer if you find anything durable (the emit-intent → drain seam closure is a candidate).

## §3a Handover from prior cohort

Tasks 1–5 + fix-impl-5 landed on `phase-m3-core-stage-mode` (advisor states exact SHA at dispatch):
- **Task 1** — `RoomEventPayload` 5 optional chair-action fields + `CaseTransitionEvent.chair_pseudonym` binary-side.
- **Task 2** — `mint_access_token(can_publish)` (presenter/watcher mint).
- **Task 3** — `stage.rs` `Stage`/FIFO/`transfer_chair`/`chair_override` + 6 tests.
- **Task 4** — 30s grace (`run_grace`/`on_grace_expired`/`GRACE_SECS`) + 2 grace tests; separate async `run_grace` (sync methods unchanged).
- **Task 5** — `room_event_client.rs` (`post_room_event` + `RoomEventPayload`/`RoomEventRequest` + 2 JSON-shape tests) + config URL fix (`/api/v4`) + `main.rs mod` + the **emit-intent seam** (`EmitIntent` + `Stage::pending_emits`; `transfer_chair`/`chair_override`/`promote_next` push intents; 11 stage tests). The async POST was deferred to THIS task.
- **fix-impl-5** — `#[allow(dead_code)]` on `post_room_event`/`RoomEventRequest`/`brehon_room_event_url` (scaffold-ahead-of-caller). **THIS task removes all 3** by wiring `drain_emits` (the production caller).

Task 6 closes the seam: provisioning opens stage mode + seats the chair + drains emits; the e2e stub asserts the FIRST `room_chair_*` chain entries land with pseudonyms. After Task 6 validates + merges, the phase is impl-complete (6/6) → bm-pr → CR triage (gate 3).
