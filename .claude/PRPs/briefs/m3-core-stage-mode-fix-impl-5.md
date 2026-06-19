# Brief: m3-core-stage-mode fix-impl-5 (Task 5 dead_code — scaffold-ahead-of-caller)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-fix-impl5-task5-deadcode-allow — see .claude/PRPs/briefs/m3-core-stage-mode-fix-impl-5.md`

## §2 Scope

**Fix the 3 clippy `dead_code` errors that failed Task 5's `validate-pending-laptop-linux` validation** (DQ `7677417db9fb-001`). `cargo check` PASSES, the 2 `room_event_client` JSON-shape tests PASS, and the 11 `stage` tests PASS; the ONLY blocker is clippy `-D warnings` flagging 3 scaffold-ahead-of-caller items.

**The failing lints (verbatim):**
```
error: field `brehon_room_event_url` is never read
  --> src/config.rs:37:9
error: struct `RoomEventRequest` is never constructed
  --> src/room_event_client.rs:27:8
error: function `post_room_event` is never used
  --> src/room_event_client.rs:36:14
```

**Why dead (not a defect):** Task 5 used the emit-intent seam (`Stage::pending_emits: Vec<EmitIntent>`) so `transfer_chair`/`chair_override` stay synchronous (keeping the 11 stage tests green). The actual *drain-and-POST* — the async bridge controller that drains `pending_emits` and calls `post_room_event(state.http_client, config.brehon_room_event_url, ...)` — is **Task 6** scope. So `post_room_event`, `RoomEventRequest`, and the consumed `brehon_room_event_url` field have no caller **yet**. This is the SAME scaffold-ahead-of-caller situation as Task 2's `mint_access_token`/`VideoGrant` (which carried `#[allow(dead_code)]` until Task 3 referenced them).

**USER DECISION (2026-06-19):** `#[allow(dead_code)]` the 3 items now (scaffold-ahead-of-caller), AND Task 6 will wire the async-controller drain + remove these allows when the caller lands. (The advisor will extend the Task 6 brief with the explicit drain requirement.)

**Produces (exactly 1 commit, ≤2 files):**
1. `services/bridge/src/room_event_client.rs` — add `#[allow(dead_code)]`:
   - above `struct RoomEventRequest<'a>` (currently line ~26, above `#[derive(serde::Serialize)]` or directly above `struct` — place it so it applies to the struct):
     ```rust
     // ponytail: scaffold-ahead-of-caller — Task 6 wires the async controller drain that constructs this.
     #[allow(dead_code)]
     struct RoomEventRequest<'a> { ... }
     ```
     (Place `#[allow(dead_code)]` directly above the item; keep the existing `#[derive(serde::Serialize)]`.)
   - above `pub async fn post_room_event(` (line ~36):
     ```rust
     #[allow(dead_code)] // Task 6: the async bridge controller drains Stage::pending_emits and calls this.
     pub async fn post_room_event( ... )
     ```
2. `services/bridge/src/config.rs` — add `#[allow(dead_code)]` above `pub brehon_room_event_url: String,` (line ~37):
   ```rust
   /// URL for POST /api/v4/governance/room-event (binary callback). Read from
   /// BREHON_ROOM_EVENT_URL; consumed by the room-event emitters (Task 6 controller drain).
   #[allow(dead_code)] // Task 6: removed when the controller reads this to call post_room_event.
   pub brehon_room_event_url: String,
   ```

**Do NOT:**
- Touch `RoomEventPayload` (it IS used — by the tests + `EmitIntent`), the `EmitIntent` struct, `pending_emits`, `stage.rs`, `main.rs`, or any test.
- Remove the `#[allow(dead_code)]` on the 3 livekit config fields (still unconsumed).
- Wire a real caller / async controller — that is Task 6 (the allows are deliberate placeholders).
- Touch any `crates/**`, any migration, `Cargo.toml`/`Cargo.lock`.

**Branch:** forks from the **Task 5 #717 worker branch** `junior/role-impl-task-m3-core-stage-mode-task5-room-event-client-emitters-see-claude-prps-briefs-m3-core-stage-mode-impl-5-717` (tip `efb9c9d9d`) — so the fix lands on Task 5's lineage.

## §3 Required reading

- `services/bridge/src/room_event_client.rs:26-50` — `RoomEventRequest` (`:27`) + `post_room_event` (`:36`); confirm `RoomEventPayload` (`:6`) is NOT touched.
- `services/bridge/src/config.rs:35-40` — the `brehon_room_event_url` field (`:37`).
- **Lessons (mandatory, §2.4 file-class — `services/bridge/**` + config):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml`.
  - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.

## §4 Constraints

- **ONE commit, ≤2 files** (room_event_client.rs + config.rs) — `fix(rtc): allow(dead_code) on room-event emitter scaffold pending Task 6 drain (fix-impl 5)`.
- **`#[allow(dead_code)]` only on the 3 named items** — `RoomEventRequest`, `post_room_event`, `brehon_room_event_url`. NOT on `RoomEventPayload` (used).
- **Each allow carries a `// Task 6:` comment** noting it's removed when the controller drain lands — so the next reader knows it's intentional scaffold, not debt.
- **No caller wiring, no test change** — pure suppression-until-consumer.
- **Resolve DQ `7677417db9fb-001`** in the same commit (move pending→resolved, `answered_by: "advisor"`, `answer` = "dead_code allowed per user 2026-06-19 — scaffold-ahead-of-caller; Task 6 controller drain removes the allows").
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml room_event_client"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push, stop.

## §3a Handover from prior cohort

Task 5 (#717 @ `efb9c9d9d`) landed `room_event_client.rs` (post_room_event + RoomEventPayload/RoomEventRequest + 2 JSON-shape tests), the config URL fix (`/api/v4`), `main.rs mod`, and the `stage.rs` emit-intent wiring (`EmitIntent` + `pending_emits`, transfer_chair/chair_override push intents; 11 stage tests pass). `cargo check` PASSES. The ONLY validation blocker is 3 `dead_code` clippy errors — the emitter scaffold has no caller until Task 6's async controller drains `pending_emits`. This fix-impl suppresses the 3 with `#[allow(dead_code)]` (Task-2 precedent) per the user's decision; Task 6 wires the drain + removes the allows.
