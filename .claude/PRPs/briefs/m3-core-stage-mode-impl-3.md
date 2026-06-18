# Brief: m3-core-stage-mode impl-3 (Task 3 — stage-mode core: chair seat + FIFO + mic-pass machine)

## §1 Role + dispatch line

`[role:impl-task] m3-core-stage-mode-task3-stage-fifo-mic-pass — see .claude/PRPs/briefs/m3-core-stage-mode-impl-3.md`

## §2 Scope

**Task 3 of plan `.claude/PRPs/plans/m3-core-stage-mode.plan.md` (lines 488–524).** Add `services/bridge/src/stage.rs` (NEW) — the chair-seat + **persisted FIFO raised-hand queue** + type-state mic-pass state machine that emits `GrantCmd` to a `GrantSink` — plus the `bridge_room` queue/chair accessors. This is the **marquee of the phase**: the 4-mic-pass-in-sequence test is the §16a Story 1 DoD.

`requires: task 2` (satisfied — Task 2 `mint_access_token(can_publish)` landed on `phase-m3-core-stage-mode` tip `ee630ad71`). Task 3 references the `can_publish` grant *concept* via the `GrantSink` trait; the real LiveKit-re-minting `GrantSink` adapter is wired later — **Task 3's `GrantSink` is a trait + a test recorder impl only.**

**Produces (exactly 1 new file + 2 modified, ONE commit):**
1. **CREATE** `services/bridge/src/stage.rs` — per plan §10.6:
   - `pub enum SeatState { Watcher, Promoted, Speaking }`
   - `pub enum GrantCmd { GrantPublish(String), RevokePublish(String) }`
   - `pub trait GrantSink { fn apply(&mut self, cmd: GrantCmd); }`
   - `pub struct Stage { chair: Option<String>, fifo: VecDeque<String>, current: Option<(String, SeatState)>, /* … */ }`
   - methods: `raise_hand(&mut self, p: &str)` (FIFO append, **no dup**), `promote_next(&mut self, sink: &mut dyn GrantSink)` (pop FIFO head → `GrantPublish` → `SeatState::Promoted`; **no 30s timer yet — Task 4 adds it**), `on_activate(&mut self, p: &str)` (`Promoted`→`Speaking`), `chair_override(&mut self, action: Override, target: &str, sink: &mut dyn GrantSink)` (force_demote/force_promote out of FIFO order + grant/revoke), `transfer_chair(&mut self, to: &str) -> (String, String)` (returns `(from, to)` for the chain entry — Task 5 emits).
   - **Persist** FIFO + chair to `bridge_room` on every mutation (`write_queue_state` / `write_chair_id`); `Stage::load` reads them back.
   - **Unit tests (deterministic, NO docker):** (a) **4-mic-pass-in-sequence** — 4× `raise_hand` + 4× `promote_next`/`on_activate`; assert the test `GrantSink` recorded `GrantPublish` in **FIFO order** for all 4 (§16a Story 1 marquee DoD); (b) **chair-override reorder** — `force_promote` a watcher out of FIFO order, assert it's granted **before** the FIFO head; `force_demote` the speaker, assert `RevokePublish`.
2. **MODIFY** `services/bridge/src/bridge_room.rs` — add `read/write_queue_state` + `read/write_chair_id` per §10.5 (mirror the `upsert`/`set_watermark` shape at `:69-100`). Add a `#[cfg(test)]` test: write a queue JSON → read it back equal; write a chair_id → read it back.
3. **MODIFY** `services/bridge/src/main.rs` — `mod stage;` (alphabetical position).

**Do NOT:**
- Add the **30s grace timer** or `on_grace_expired` — that is **Task 4** (`promote_next` leaves the `Promoted` participant awaiting activation with NO timer; Task 4 adds the `tokio::select!` boundary). Adding it here is scope creep.
- Build the real LiveKit-re-minting `GrantSink` adapter — Task 3 only defines the trait + a test recorder. (The real adapter calling `mint_access_token(can_publish)` comes with later wiring.)
- Build the `room_event_client` / chain emission — that is **Task 5**.
- Read `event.chair_pseudonym` / dual-source the chair seat — that is **Task 6**.
- Touch any `crates/**` file, any migration, `Cargo.toml`/`Cargo.lock` (no new dep — `VecDeque` is std).

**Branch:** forks from `phase-m3-core-stage-mode` (current tip `ee630ad71` — has Task 1 + Task 2 + the dead_code allows).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-stage-mode.plan.md` — Task 3 (488–524), **§10.5** (FIFO persistence in `bridge_room.queue_state`, the `write_queue_state`/`Stage::load` round-trip), **§10.6** (the `stage.rs` `SeatState`/`GrantCmd`/`GrantSink`/`Stage` skeleton — code-block to follow), §8 flow (108–117 the seat-state-machine + FIFO + override rows).
- `services/bridge/src/bridge_room.rs:13-100` — the `chair_id` + `queue_state` columns (already exist, lines 13-14) and the `upsert` (`:69`) / `set_watermark` (`:94`) accessor shapes to MIRROR for `read/write_queue_state` + `read/write_chair_id`.
- `services/bridge/src/main.rs` — the existing `mod` declaration block (add `mod stage;` alphabetically).
- **Lessons (mandatory):**
  - `feedback_governance_type_state_handlers.md` — **the type-state pattern is the load-bearing design** (plan MIRROR cites it). Invalid transitions (promote empty FIFO, activate a non-promoted participant) must be **unrepresentable or return an error**, NOT silent no-ops. Mirror the phantom-typed / `TryFrom`-guard idiom.
  - `feedback_build_what_tests_exercise.md` — the 4-mic-pass test must **EXERCISE the real flow** (drive `raise_hand`×4 + `promote_next`/`on_activate` and assert the `GrantSink`-recorded `GrantPublish` ORDER), NOT assert on struct/config shape. A test that only asserts a single successful promote does NOT satisfy the DoD.
  - **Bridge file-class (§2.4 — `services/bridge/**`):**
    - `feedback_bridge_validates_on_linux_not_windows.md` — bridge cargo runs **Linux ONLY** via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); Windows-local fails `ruma-common` E0119.
    - `feedback_linux_compile_proof_is_a_gate.md` — write a `validate-pending-laptop-linux` DQ; the laptop runs `cargo-linux.sh`; `bm-pr` gates on `result:pass`.
    - `feedback_clippy_test_style.md` — tests use `anyhow::Result<()>` / `LemmyResult`-style with `?`, no `unwrap`/`expect` (clippy denies them under `-D warnings`).

## §4 Constraints

- **ONE commit** — `feat(rtc): stage-mode core — chair seat + persisted FIFO + mic-pass state machine (task 3)`.
- **Type-state load-bearing (per `feedback_governance_type_state_handlers.md`):** `SeatState` transitions are guarded — `promote_next` on an empty FIFO returns an error; `on_activate(p)` where `p` is not the `Promoted` participant returns an error. Do NOT make invalid transitions silent no-ops. DoD: the override-reorder test exercises at least one guarded-error path (or the type makes it unrepresentable).
- **FIFO single-writer, but PERSISTED** (per §10.6 GOTCHA): the chair's bridge controller is the sole writer (no lock needed), BUT the queue MUST flush to `bridge_room.queue_state` on every mutation and `Stage::load` must read it back — the raised-hand order survives bridge restart (the PRD model; the in-memory-only alternative loses order on restart and is wrong).
- **NO 30s timer in Task 3** — `promote_next` ends at `Promoted` awaiting activation; the timer is Task 4. Keep the seam clean (`on_grace_expired` may be declared as a stub/signature per §10.6 but its timer wiring is Task 4).
- **NO new dependency** — `VecDeque` is std; `serde_json` for queue serialisation already in the bridge. If you find yourself wanting a new dep, STOP and raise a `kind: "blocker"` DQ (the plan asserts no new dep — watchpoint).
- **ADR-015 (pseudonyms-only):** the FIFO + chair store **pseudonym strings** (opaque), never `person_id`/username. `queue_state` JSON is `Vec<pseudonym>`; `chair_id` is a pseudonym. No content/speech field touches `stage.rs`.
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml stage"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort (Cohort A)

Cohort A (Tasks 1+2) landed on `phase-m3-core-stage-mode` @ `ee630ad71`:
- **Task 1** added the binary `RoomEventPayload` 5 optional chair-action fields (`action`, `target_pseudonym`, `from_pseudonym`, `to_pseudonym`, `at`, all `skip_serializing_if`) + `CaseTransitionEvent.chair_pseudonym` — the DTO Task 5 will populate when emitting chain entries.
- **Task 2** added `can_publish` to `mint_access_token`/`VideoGrant` — the LiveKit publish-grant mechanism the real `GrantSink` adapter (later) re-mints. Task 2's `mint_access_token`/`VideoGrant`/`Claims` carry `#[allow(dead_code)]` until a caller exists; **your `stage.rs` `GrantSink` trait does NOT call `mint_access_token` yet** (it's a trait + test recorder), so those allows stay until the real adapter lands.
- Both validated Linux-clean (check+clippy+test). Task 3 forks from this tip.
