# Brief: m3-core-stage-mode impl-5 (Task 5 — bridge→binary room-event client + chair-action EMITTERS)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-task5-room-event-client-emitters — see .claude/PRPs/briefs/m3-core-stage-mode-impl-5.md`

## §2 Scope

**Task 5 of plan `.claude/PRPs/plans/m3-core-stage-mode.plan.md` (lines 560–600).** Add `services/bridge/src/room_event_client.rs` (NEW) — the bridge→binary outbound POST client — and wire the **chair-transfer + chair-override emitters**. This is the **FIRST emission** of `room_chair_transferred` / `room_chair_override` chain entries from the bridge. The binary side (`room_event_handler.rs`, `ROOM_KINDS`) already accepts these kinds (M3 Phases 1+2).

`requires: task 1` (RoomEventPayload wire shape — landed) + `task 4` (Stage transfer/override paths — landed @ phase `29b16adf0`).

**Produces (1 new file + 3 modified, ONE commit):**

1. **CREATE** `services/bridge/src/room_event_client.rs` — copy the §10.3 skeleton (plan lines 188–221) verbatim as the starting point:
   - `#[derive(serde::Serialize)] pub struct RoomEventPayload` with fields `case_id: i32`, and `Option<String>`/`Option<i32>` fields `matrix_room_id`, `lifecycle_stage` (String, NOT optional), `member_count`, `action`, `target_pseudonym`, `from_pseudonym`, `to_pseudonym`, `at` — each Option field `#[serde(skip_serializing_if = "Option::is_none")]`.
   - `#[derive(serde::Serialize)] struct RoomEventRequest<'a> { entry_kind: &'a str, payload: RoomEventPayload, actor_pseudonym: Option<String> }`.
   - `pub async fn post_room_event(client: &reqwest::Client, url: &str, secret: &str, entry_kind: &str, payload: RoomEventPayload, actor_pseudonym: Option<String>) -> anyhow::Result<()>` — `client.post(url).header("Authorization", format!("Bearer {secret}")).json(&req).send().await?.error_for_status()?; Ok(())`.
   - **`#[cfg(test)]` tests (JSON-shape, NO live POST):**
     - `room_chair_override` request → `serde_json::to_value(&RoomEventRequest{...})` → assert `entry_kind == "room_chair_override"`, `payload.action` present, `payload.target_pseudonym` present, and the transfer fields (`from_pseudonym`/`to_pseudonym`) are ABSENT (skip_serializing_if omits them).
     - `room_chair_transferred` request → assert `from_pseudonym`/`to_pseudonym`/`at` present.
   - (The live POST is the docker-gated Task-6 test, NOT here.)

2. **MODIFY** `services/bridge/src/config.rs`:
   - Change the `brehon_room_event_url` default (currently line ~83-84) from `"http://localhost:8536/governance/room-event"` to `"http://localhost:8536/api/v4/governance/room-event"`. Keep the `BREHON_ROOM_EVENT_URL` env override.
   - **Remove the `#[allow(dead_code)]`** on `brehon_room_event_url` (line ~39) — it's now consumed by the emitter. Do NOT remove the other 3 `#[allow(dead_code)]` (link_confirm/livekit fields — still unconsumed until later).

3. **MODIFY** `services/bridge/src/main.rs` — add `mod room_event_client;` (alphabetical position).

4. **MODIFY** `services/bridge/src/stage.rs` — wire the emitters into `transfer_chair` + `chair_override`. **READ THE SEAM NOTE BELOW FIRST.**

**SEAM NOTE (read carefully — the one judgment call):** `transfer_chair(&mut self, to: &str, conn: &Connection) -> Result<(String, String)>` and `chair_override(&mut self, action, target, sink, conn) -> Result<()>` are **synchronous** and have **existing test callers** (`chair_override_reorder` etc — they must keep passing). `post_room_event` is **async** and needs a `reqwest::Client` + `url` + `secret` the `Stage` does not currently hold. Do NOT force `async` onto the sync `transfer_chair`/`chair_override` signatures if it breaks the existing test callers. The clean approaches (pick whichever fits with least churn):
   - **(a) Emit-intent return (preferred):** `transfer_chair`/`chair_override` stay sync and additionally RETURN the `(entry_kind, RoomEventPayload, actor_pseudonym)` emit-intent (or push it to a `Vec<EmitIntent>` field on `Stage`); the async bridge controller (Task 6 wiring) drains the intent and calls `post_room_event`. The §16a Story 3/4 DoD is satisfied by the JSON-shape test in `room_event_client.rs` + a stage-side test asserting `transfer_chair` produces the right emit-intent. Keep the existing 8 tests green.
   - **(b) Async helper method:** add a separate `async fn emit_chair_transferred(&self, client, url, secret) -> Result<()>` that builds the payload from Stage state + calls `post_room_event`; `transfer_chair` stays sync and only mutates state. The test asserts the payload builder.
   If you cannot wire the emitter without changing a sync signature that has callers, raise a `kind: "blocker"` DQ rather than guessing or breaking tests.

**ENTRY-KIND STRINGS:** the bridge does NOT depend on `lemmy_db_schema`, so pass the literal strings `"room_chair_transferred"` / `"room_chair_override"` (the binary's `ROOM_KINDS` admits them — M3 Phase 2 added them). Do NOT import an entry-kind const from a crate.

**Do NOT:**
- Make a live HTTP POST in any test — JSON-shape assertions only (the live POST is Task 6, docker-gated).
- Build the stage-mode provisioning / Q&A sidebar / `room_provisioner.rs` changes — that is **Task 6**.
- Break any of the 8 existing `stage` tests.
- Touch any `crates/**`, any migration, `Cargo.toml`/`Cargo.lock` (reqwest + serde + serde_json already bridge deps).

**Branch:** forks from `phase-m3-core-stage-mode` (current tip `29b16adf0` — has Tasks 1+2+3+4).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — Task 5 (560–600), **§10.3** (lines 184–224, the `room_event_client.rs` skeleton — copy it), §10.11 (metadata-only).
- `crates/api/api_utils/src/bridge_notify.rs:62-72` — **the canonical Bearer-POST MIRROR**: `client.post(url).header("Authorization", format!("Bearer {}", secret)).json(&payload).send().await` + the **fire-and-forget warn-on-error** idiom (`if let Err(e) = ... { tracing::warn!("... non-fatal: {e}"); }`). The GOTCHA's "transport errors logged + swallowed, never block the loop" = this exact pattern — mirror it at the emitter callsite (NOT inside `post_room_event`, which propagates via `?`; the CALLER swallows + warns).
- `services/bridge/src/room_provisioner.rs:573-591` — reqwest via a shared client (the client-handle shape).
- `services/bridge/src/stage.rs` (the WHOLE file, phase tip `29b16adf0`) — `transfer_chair` (`:216`), `chair_override` (`:183`), `Override` enum (`:37`), the 8 existing tests (mirror their recorder-test idiom).
- `services/bridge/src/config.rs:35-84` — the `brehon_room_event_url` field (`:39-40` allow+field), default (`:83-84`).
- **Lessons (mandatory):**
  - `feedback_governance_type_state_handlers.md` — emit only from the valid transition path; the emit-intent must reflect the state mutation that actually happened.
  - `feedback_build_what_tests_exercise.md` — the JSON-shape test must assert the REAL serialised wire shape (entry_kind + present/absent fields), not a struct-field read.
  - `feedback_cheap_model_arm_drops_adr_constraints.md` — the ADR-015 pseudonym gate below is **load-bearing**, not a nit.
  - **Bridge file-class (§2.4 — `services/bridge/**`):**
    - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
    - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.
    - `feedback_clippy_test_style.md` — no `unwrap`/`expect`/`unwrap_or_default`; tests return `Result<()>` + `?` + `.context()`.

## §4 Constraints

- **ONE commit, ≤4 files** — `feat(rtc): bridge→binary room-event client + chair-action emitters (task 5)`.
- **ADR-015 (pseudonyms-only) — LOAD-BEARING:** every payload field (`from_pseudonym`/`to_pseudonym`/`target_pseudonym`/`actor_pseudonym`) is a **pseudonym string** the `Stage` already holds (sourced from LiveKit-identity pseudonyms). **Why it can't be deferred:** a real identity (person_id / username / `@user:server` MXID) written into a hash-chained chair entry is permanent and unscrubable — a GDPR violation baked into the immutable governance log. **DoD: `rg -i 'person_id|username|@.*:' services/bridge/src/room_event_client.rs` returns NOTHING identity-shaped** (only pseudonym strings). State this grep result in the commit body.
- **ADR-016 (metadata-only):** `post_room_event` payload carries METADATA only (case_id, lifecycle_stage, action, pseudonyms, timestamp) — **never** Q&A text, speech, or video. No content field on `RoomEventPayload`.
- **URL DoD:** after the config fix, `rg 'api/v4/governance/room-event' services/bridge/src/config.rs` MUST match (the missing `/api/v4` was a silent-404 footgun — plan §10.3 GOTCHA + §15.5).
- **Fire-and-forget at the callsite:** `post_room_event` returns `Result` (propagates via `?`); the EMITTER callsite (in stage.rs or the controller) wraps it `if let Err(e) = ... { tracing::warn!(...) }` — a bridge→binary transport failure must NEVER block or panic the stage loop (best-effort chain entry). Mirror `bridge_notify.rs:69-72`.
- **Don't break the 8 stage tests** — reconcile the sync/async seam per the §2 SEAM NOTE; blocker-DQ if it forces a sync-signature break with callers.
- **No `unwrap`/`expect`/`unwrap_or_default`** (clippy `-D warnings`). Propagate with `?` + `.context()`.
- **NO new dependency** — reqwest/serde/serde_json/anyhow already bridge deps.
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml room_event_client"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

Tasks 1–4 landed on `phase-m3-core-stage-mode` @ `29b16adf0`:
- **Task 1** — `RoomEventPayload` 5 optional chair-action fields (`action`/`target_pseudonym`/`from_pseudonym`/`to_pseudonym`/`at`) on the BINARY struct (`crates/api/api_common/src/governance.rs`) + `room_event_handler.rs`. Your bridge-side `RoomEventPayload` (Serialize mirror) MUST match this wire shape (same field names, same skip_serializing_if).
- **Task 2** — `mint_access_token(can_publish)` (the publish-grant mint).
- **Task 3** — `stage.rs` `Stage`/`SeatState`/FIFO/`transfer_chair`/`chair_override` + 6 tests.
- **Task 4** — 30s grace timer (`run_grace`/`on_grace_expired`/`GRACE_SECS`) + 2 grace tests (8 total). Task 4's seam was reconciled with a separate async `run_grace` method (sync `promote_next`/`on_grace_expired` unchanged). **Apply the same discipline to the Task-5 emit seam** — keep `transfer_chair`/`chair_override` sync if making them async would break their callers; use emit-intent return or an async helper per the §2 SEAM NOTE.
