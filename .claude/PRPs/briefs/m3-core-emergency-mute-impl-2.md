# Brief: m3-core-emergency-mute impl-2 (Task 2)

## §1 Role + dispatch line

`[role:impl-task] m3-core-emergency-mute-task2-mute-handler-power-levels — see .claude/PRPs/briefs/m3-core-emergency-mute-impl-2.md`

## §2 Scope

**Task 2 of plan `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` (lines 402–435).** Add the bridge **cross-instance mute power-level path**: a new `services/bridge/src/mute_handler.rs` (the `compute_mute_all_override` pure fn + the async per-room `mute_all_power_levels` GET→mutate→PUT), widen `sanction_handler`'s power-level GET/PUT to `pub(crate)`, and register the module in `main.rs`. This is the cross-instance authority for the mute (Matrix `m.room.power_levels`, OQ-V2-06 — one PUT federates).

**Produces (exactly 3 file edits, ONE commit):**
1. `services/bridge/src/mute_handler.rs` (NEW) — `compute_mute_all_override(content) -> serde_json::Value` (pure) + `#[allow(dead_code)] pub(crate) async fn mute_all_power_levels(state, case_id) -> Result<usize>` + `#[cfg(test)]` tests for the pure fn.
2. `services/bridge/src/sanction_handler.rs` — change `async fn get_power_levels` (`:281`) and `async fn put_power_levels` (`:304`) to `pub(crate) async fn` (bodies UNCHANGED — visibility only).
3. `services/bridge/src/main.rs` — add `mod mute_handler;` (alphabetical, after `mod livekit_jwt;` / before `mod provision;` per the existing block `:12-25`).

**The plan's Task 2 (lines 402–435) is the contract — follow its IMPLEMENT (files 1/2/3) / MIRROR / GOTCHA exactly.**

**`compute_mute_all_override` semantics (plan §10.2/§10.3):** take the room's current power-levels `content`, **raise** the publish/voice power requirement above `users_default` for `events["m.call.member"]` AND the MSC3401 alias `events["org.matrix.msc3401.call.member"]` (and the message threshold), so EVERY non-elevated participant loses publish in ONE PUT. Always return the full merged content. The chair retains publish via the room-admin power-level seated at provisioning — this fn raises the **bar**, it does NOT touch `users[chair]`.

**`mute_all_power_levels` semantics:** open `bridge_room`, `lookup_by_case` (`:59`), per-room `get_power_levels` → `compute_mute_all_override` → `put_power_levels`, **best-effort skip-on-error** (one room's failure must NOT abort the others — mirror `sanction_handler.rs:188-228`), return the count applied. Mark `#[allow(dead_code)]` (Phase-6 wires the live trigger; reachable from the Task-4 `#[ignore]` test).

**The `#[cfg(test)]` test for the pure fn (the deterministic part):**
- Feed `{users_default: 0, events: {}}` → assert returned `events["m.call.member"] == 1` (> users_default) AND `events["org.matrix.msc3401.call.member"] == 1`.
- Feed `{users_default: 50}` → assert `events["m.call.member"] == 51`.
- (The live GET/PUT path is the docker-gated Task-4 `#[ignore]` test — mirror `sanction_handler.rs:474-482`; NOT in this task.)

**Do NOT touch:** any `crates/**` file; any migration; `Cargo.toml` / `Cargo.lock` (NO new dependency — reuse `reqwest`, `serde_json`, `rusqlite`, `tokio`, `anyhow`, `time`); `stage.rs` / `room_event_client.rs` (those are Task 3); any file outside the 3 above. Do NOT call `mint_access_token` here (the mute reaches cross-instance via the power-level PUT, NOT via LiveKit grants — that's Task 3's local sweep).

**Branch:** forks from `phase-m3-core-emergency-mute` (current tip `8ffcef9ee`).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` — Task 2 (402–435), §10.2 (`compute_mute_all_override` shape), §10.3 (`mute_all_power_levels` per-room loop), §4(b) (mechanism: power-levels = cross-instance authority), §19 (1) (gate-1-ratified mechanism decision).
- `services/bridge/src/sanction_handler.rs:188-228` — the per-room best-effort loop (MIRROR the structure), `:250-277` (the `compute_power_override` test shape — MIRROR for your `compute_mute_all_override` test), `:281-325` (the GET/PUT fns you widen to `pub(crate)`), `:437-472` / `:474-482` (compute test + `#[ignore]` live test shapes).
- `services/bridge/src/bridge_room.rs:59` — `lookup_by_case` (the per-case room iterator).
- `services/bridge/src/main.rs:12-25` — the `mod` block (insert `mod mute_handler;` alphabetically).
- **Lessons (mandatory, §2.4 file-class injection — bridge code):**
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); NEVER Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — this task writes a `validate-pending-laptop-linux` DQ; `bm-pr` gates on `result:pass`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; the laptop advisor runs all cargo/Docker.
  - `feedback_clippy_test_style.md` — the `#[cfg(test)]` test uses the bridge's `anyhow::Result<()>` / `?` idiom (mirror the sibling sanction test), no `unwrap`/`expect` under `-D warnings`.

## §4 Constraints

- **ONE commit, 3 files** — `feat(rtc): bridge cross-instance mute power-level path (mute_handler.rs) (task 2)`.
- **Mechanism = Matrix power-levels, NOT LiveKit grants** (gate-1 ratified, planner DQ `78c6ad4bfb41-001`). Do NOT call `mint_access_token` / any LiveKit grant in `mute_handler.rs` — the local LiveKit sweep is Task 3's `Stage::mute_all`. This file is the cross-instance hammer only.
- **ADR-016 (metadata-only):** this is mute ENACTMENT (power-level mutation) — no content is read, hashed, or emitted here. The `room_mute_all` chain emission is Task 3.
- **`compute_mute_all_override` raises the BAR, never touches `users[chair]`** — the chair keeps publish via the room-admin power-level seated at provisioning. If you find yourself special-casing the chair's `users` entry, STOP — that's the wrong mechanism (re-read §4(b)).
- **Best-effort per-room** — one room's GET/PUT failure must NOT abort the loop (mirror `sanction_handler.rs:188-228`). Return the count of rooms successfully muted.
- **NO new dependency** — `Cargo.toml`/`Cargo.lock` unchanged. If you need a new dep, STOP and raise a `kind: "blocker"` DQ (the plan asserts no new dep; a dep add is a scope surprise — watchpoint).
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml mute_handler"]`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself (laptop advisor validates). Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — blocked by the sensitive-file guard in the Junior worktree context per PMD #857/#867).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort

(none — first cohort)

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-emergency-mute-task2
  filesCreated: [services/bridge/src/mute_handler.rs]
  filesModified: [services/bridge/src/sanction_handler.rs, services/bridge/src/main.rs]
  keyDecisions:
    - "compute_mute_all_override raises events[m.call.member] + msc3401 alias above users_default; chair untouched"
    - "mute_all_power_levels best-effort per-room (mirror sanction loop); #[allow(dead_code)] until Phase-6 trigger"
    - "get_power_levels/put_power_levels widened to pub(crate) (bodies unchanged)"
  validate_dq: <composite-id of the validate-pending-laptop-linux DQ>
  notes: "<exact pub(crate) signatures Task 3 can reuse + the compute fn name + mute_all_power_levels arg shape>"
```
