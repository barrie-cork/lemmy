## 1. Role + dispatch line

`[role:impl-task] m3-core-recording Task 5 — participant-floor recording-fetch endpoint (ADR-015)`

Dispatch string:
```
[role:impl-task] m3-core-recording-task5-participant-floor-fetch — see .claude/PRPs/briefs/m3-core-recording-impl-5.md
```

## 2. Scope

Add the recording-fetch HTTP route + handler to the bridge that **rejects a non-participant request before serving** and accepts a participant request — the ADR-015 participant-floor (D5 Option C access bar). Exactly **two files**, **one crate** (`brehon-bridge`, workspace-excluded, Linux-validated).

**File 1 of 2 — `services/bridge/src/recording.rs`** (modify):
Add the pure gate function per §10.7:
```rust
/// ADR-015 participant-floor: a recording-fetch requester MUST be a participant
/// of the room at recording time.  Cannot be zero under always_pseudonym (else a
/// pseudonymous town hall's recording leaks).  Both args are PSEUDONYMS.
pub fn is_participant(requester_pseudonym: &str, participants: &[String]) -> bool {
    participants.iter().any(|p| p == requester_pseudonym)
}
```
Add a `#[cfg(test)]` unit test `is_participant_floor` (BARE fn name per R13) asserting:
- a participant pseudonym in the set → `true`
- a non-participant pseudonym → `false`
- an **empty** participant set → `false` (the floor cannot be zero — ADR-015)

**File 2 of 2 — `services/bridge/src/appservice.rs`** (modify):
- Add `.route("/brehon/recording/{id}", get(handle_recording_fetch))` to `router()` (the route block is at `:231`). Add it **AFTER `.route_layer(...)`** alongside the other `/brehon/*` routes (`/brehon/room-event`, `/brehon/sanction-event`, `/brehon/link-claim`) so it does **NOT** inherit `hs_token_auth` — these are Brehon→bridge calls authenticated inline with `BRIDGE_CALLBACK_SECRET`, mirroring `handle_room_event`'s Bearer shape (`:199-226`).
- Add `async fn handle_recording_fetch(...)` per §10.7: mirror `handle_room_event`'s `State`/`HeaderMap` + Bearer-`BRIDGE_CALLBACK_SECRET` auth shape; resolve the recording's participant set (scaffold-grade this phase — see GOTCHA); extract the requester pseudonym (scaffold-grade this phase — Phase-6 wires live session-auth); then call `recording::is_participant(&requester_pseudonym, &participants)` **BEFORE** serving — non-participant → `StatusCode::FORBIDDEN`, participant → `StatusCode::OK` + the `media_url`.

### 2.1 Boundaries (do NOT)

- Do **NOT** build presigned URLs, retention, tombstone, or any strict-ACL machinery — **DEFERRED, D5 Option C** (scope error → catch-fire per bootstrap §7).
- Do **NOT** add a new dependency, a migration, a new column, a new const, or a registry bump (R8/R14). Task 5 touches no deps.
- Do **NOT** touch any file other than the two named above.
- Do **NOT** run cargo on the daemon (R3). Write the `validate-pending-laptop-linux` DQ and **stop** (see §4).

## 3. Required reading

Read these BEFORE writing code:

- `.claude/PRPs/plans/m3-core-recording.plan.md` §10.7 (the `is_participant` + `handle_recording_fetch` MIRROR spec — copy the function signatures verbatim), §13 Task 5 (FILES/IMPLEMENT/MIRROR/GOTCHA/VALIDATE), R9 (ADR-015 load-bearing), R13 (`--bins` bare fn name in test filters).
- `services/bridge/src/appservice.rs:199-226` (`handle_room_event` — the `State`/`HeaderMap`/Bearer-`BRIDGE_CALLBACK_SECRET` auth shape to mirror) and `:231-259` (`router()` — where to add the route, AFTER `route_layer`).
- `services/bridge/src/recording.rs` (the existing module — read the `compute_content_sha256` + `RecordingSink` + `maybe_record` neighbours so `is_participant` matches in-file style; note the existing `#[cfg(test)] mod tests` block to extend).
- `.claude/lessons/feedback_cheap_model_arm_drops_adr_constraints.md` (the ADR-constraint-must-be-load-bearing discipline — see §4 below).
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` (R1 — bridge cargo is Linux-only via `cargo-linux.sh`; the Windows-local `cd services/bridge && cargo` form fails `ruma-common` E0119).
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` (R3 — write the DQ, do NOT run cargo on the daemon).
- `.claude/lessons/feedback_forward_declared_items_need_allow_until_consumer.md` — **read it, but note it does NOT fire this task:** `is_participant` is consumed by `handle_recording_fetch` in the **non-test build SAME commit**, and `handle_recording_fetch` is wired into `router()` SAME commit, so neither is forward-declared-until-later — no `#[allow(dead_code)]` is needed. (If you find yourself reaching for `#[allow(dead_code)]`, the wiring is incomplete — wire the callsite instead.)

## 4. Constraints

### 4.1 ADR-015 participant-floor is LOAD-BEARING (§2.4a — not merely named)

The participant-floor check is the entire point of this task — a recording served at anyone-with-the-URL under `always_pseudonym` leaks a pseudonymous town hall's audio/video to non-participants. It **cannot be deferred**: it is the D5 Option C access bar (the strict presigned ACL is the additive upgrade, NOT this gate).

- **Name the gate + callsite:** `recording::is_participant(&requester_pseudonym, &participants)` is called **BEFORE** any serve/redirect in `handle_recording_fetch`. The non-participant branch returns `StatusCode::FORBIDDEN` and the function returns early — the serve path is unreachable for a non-participant.
- **DoD (grep, both must hit):**
  - `grep "fn is_participant" services/bridge/src/recording.rs` returns the **definition**.
  - `grep is_participant services/bridge/src/appservice.rs` returns the **callsite** in the fetch path.
- **Pseudonyms only (ADR-015):** the participant set and the requester are **pseudonyms** — never `person_id`, username, or MXID. `grep -nE "person_id|username|mxid|@.*:.*" services/bridge/src/appservice.rs` over your new handler returns nothing identity-bearing.

### 4.2 R13 — bare fn name in test filters

Every test-filter command (in the DQ `commands` + your VALIDATE block) uses the **BARE** fn name `is_participant_floor`, NOT `recording::tests::is_participant_floor`.

### 4.3 Validate-pending-laptop-linux DQ then STOP (R2 + R3)

After committing + pushing the worker branch, write a `validate-pending-laptop-linux` DQ entry (use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`) with:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop-linux",
  "question": "Run cargo-linux.sh check + clippy + is_participant_floor test for bridge task 5",
  "options": ["pass", "fail"],
  "context": "m3-core-recording Task 5 — participant-floor recording-fetch endpoint (ADR-015). Bridge Linux compile + clippy -D warnings + named test.",
  "commands": [
    "scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml is_participant_floor"
  ],
  "branch": "<your worker branch>",
  "phase_task": "m3-core-recording Task 5",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```
Commit the DQ write (`chore(decision-queue): impl raised validate-pending-laptop-linux <id> — m3-core-recording task 5`), push, then **STOP**. Do **NOT** run cargo yourself — the laptop advisor is the runner (R3).

### 4.4 Clippy arity awareness (this-phase recurrence)

Task 4's `maybe_record` tripped `clippy::too_many_arguments` (8 args / 7 cap) → needed `#[allow(clippy::too_many_arguments)]`. If `handle_recording_fetch` ends up with >7 params (e.g. State + HeaderMap + Path + several resolved values), pair it with `#[allow(clippy::too_many_arguments)]` in the SAME commit and a one-line comment. The validate-pending-linux clippy `-D warnings` gate WILL catch it otherwise — saving you a fix-impl cycle.

### 4.5 Mid-task blocker discipline

If anything is ambiguous (e.g. the participant-set resolution source is unclear, or the requester-pseudonym extraction needs a real session type that doesn't exist yet), do **NOT** guess. The scaffold-grade allowance covers stubbing the participant-set + requester-pseudonym resolution (Phase-6 wires live session-auth) — the LOAD-BEARING part is the `is_participant`-before-serve gate compiling + the unit test passing. If the scaffold can't compile, raise a `kind: "blocker"` DQ (`from: "impl"`), commit + push it, and stop.

## 5. LESSON trailer

End your final commit body with a `LESSON:` line if you hit anything durable (e.g. an axum 0.8 route-param quirk, a Bearer-auth mirror gotcha).
