# Brief: m3-core-emergency-mute impl-3 (Task 3)

## §1 Role + dispatch line

`[role:impl-task] m3-core-emergency-mute-task3-stage-mute-all-negative-invariant — see .claude/PRPs/briefs/m3-core-emergency-mute-impl-3.md`

## §2 Scope

**Task 3 of plan `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` (lines 437–474).** Add the **local LiveKit revoke sweep + `room_mute_all` emission** (`Stage::mute_all`) and the bridge-side `federated` mirror field. This is the MARQUEE task — it carries the **cr-4 zero-holder negative invariant** test (the test that FAILS if the revoke sweep is deleted). `requires: 1` — Task 1's `federated` field is now on the phase branch (it is — phase tip `09f44e7e6`).

**Produces (exactly 2 file edits, ONE commit):**
1. `services/bridge/src/room_event_client.rs` — add `#[serde(skip_serializing_if = "Option::is_none")] pub federated: Option<bool>` to the bridge `RoomEventPayload` mirror (place AFTER the existing chair-action fields, mirror the binary struct §10.1); add `federated: None` to the **2 existing test literals** (`:89`, `:123`); add a `mute_all_request_json_shape` test (per §10.5).
2. `services/bridge/src/stage.rs` — add `pub fn mute_all(&mut self, publishers: &[String], federated: bool, sink: &mut dyn GrantSink)` (per §10.4 — copy the body verbatim from the plan); add `federated: None` to the **3 existing `RoomEventPayload` literals** (`:219`, `:255`, `:297`); add the **negative-invariant unit test** `mute_all_revokes_all_publishers` (§16a Story 1).

**The plan's Task 3 (lines 437–474) is the contract — follow its IMPLEMENT (file 1/file 2) / MIRROR / GOTCHA exactly. The `Stage::mute_all` body at §10.4 (plan lines 246–264) is copy-paste-exact; do not re-derive it.**

**The marquee test `mute_all_revokes_all_publishers` (the cr-4 zero-holder negative invariant — this is the whole point of the task):**
- Seat a chair (mirror the sibling `chair_override`/`transfer_chair` test setup in `stage.rs:321-632`).
- Call `mute_all(&["P1","P2","P3","P4"], true, &mut recorder)` with an EXPLICIT 4-publisher slice (NOT derived from `self.current`, which is single-presenter — taking the explicit slice is what makes the test exercise N>1, the exact cr-4 failure mode).
- Collect ALL `RevokePublish` pseudonyms from the `Recorder` `GrantSink`; **assert the set equals `{P1,P2,P3,P4}`** (set-equality / every-publisher-revoked — NOT "≥1 revoke fired").
- Assert exactly ONE `room_mute_all` EmitIntent with `payload.federated == Some(true)` AND `actor_pseudonym == Some(chair)`.
- **Add a comment documenting the delete-the-revoke check:** "deleting the `for p in publishers { sink.apply(RevokePublish) }` loop makes the set-equality assert fail — this proves the test asserts the zero-holder invariant, not the happy path."

**The `room_event_client.rs` JSON-shape test `mute_all_request_json_shape` (§10.5):** build a `RoomEventRequest { entry_kind: "room_mute_all", payload: RoomEventPayload { …, federated: Some(true) }, actor_pseudonym: Some("chair-pseudonym") }`, `serde_json::to_value`, assert `entry_kind == "room_mute_all"`, `payload.federated == true`, `actor_pseudonym == "chair-pseudonym"`, and that `action` / `from_pseudonym` / `to_pseudonym` are ABSENT (proves `skip_serializing_if` omits them). Mirror `room_event_client.rs:85-142`.

**Do NOT touch:** any `crates/**` file (Task 1 is shipped); `mute_handler.rs` / `sanction_handler.rs` / `main.rs` (Task 2, shipped); any migration; `Cargo.toml` / `Cargo.lock` (NO new dependency — reuse what `stage.rs` already imports); `services/bridge/tests/emergency_mute.rs` (Task 4); any file outside the 2 above. Do NOT re-register `ENTRY_KIND_ROOM_MUTE_ALL` or bump any registry count (it is SHIPPED — emit-only; §12 handover tripwire → STOP if you find yourself adding a const).

**Branch:** forks from `phase-m3-core-emergency-mute` (current tip `09f44e7e6` — Cohort A + the clippy-debt fix are landed).

## §3 Required reading

- `.claude/PRPs/plans/m3-core-emergency-mute.plan.md` — Task 3 (437–474), §10.4 (the `Stage::mute_all` body — copy verbatim, lines 246–264), §10.5 (`room_event_client` mirror + JSON-shape test), §10.6 (ADR-015 pseudonym pin — LOAD-BEARING), §11 (the 5-literal enumeration — all 5 get `federated`), §16a Story 1 (the marquee negative-invariant DoD).
- `services/bridge/src/stage.rs:202-271` (`chair_override` EmitIntent push — MIRROR the `pending_emits.push(EmitIntent{...})` shape), `:276-311` (`transfer_chair` `actor_pseudonym` shape), `:219`/`:255`/`:297` (the 3 existing `RoomEventPayload` literals you add `federated: None` to), `:321-632` (the `Recorder` `GrantSink` + the negative-assertion test idioms — MIRROR for `mute_all_revokes_all_publishers`).
- `services/bridge/src/room_event_client.rs:5-23` (the Serialize struct — add the mirror field), `:85-142` (the JSON-shape test idiom — MIRROR), `:89`/`:123` (the 2 test literals you add `federated: None` to).
- **Lessons (mandatory, §2.4 file-class injection — bridge code + negative-invariant test):**
  - `feedback_authz_state_machine_test_asserts_negative.md` — the cr-4 zero-holder pattern: a single-X/zero-X state machine's marquee test MUST assert the negative invariant (no holder remains), with a delete-the-mechanism mechanical check. THIS IS THE TASK'S CORE.
  - `feedback_build_what_tests_exercise.md` — the test must exercise the real revoke sweep, not a stub.
  - `feedback_bridge_validates_on_linux_not_windows.md` — all `services/bridge` cargo via `scripts/brehon/cargo-linux.sh --manifest-path services/bridge/Cargo.toml` (Docker `rust:1.95`); NEVER Windows-local (`ruma-common` E0119).
  - `feedback_linux_compile_proof_is_a_gate.md` — this task writes a `validate-pending-laptop-linux` DQ; `bm-pr` gates on `result:pass`.
  - `feedback_validate_pending_laptop_write_then_stop.md` — write the DQ and STOP; the laptop advisor runs all cargo/Docker.
  - `feedback_clippy_test_style.md` — the `#[cfg(test)]` tests use the bridge's `anyhow::Result<()>` / `?` idiom (mirror the sibling stage tests), no `unwrap`/`expect` under `-D warnings`.

## §4 Constraints

- **ONE commit, 2 files** — `feat(rtc): Stage::mute_all local sweep + room_mute_all emission + federated mirror (task 3)`.
- **cr-4 ZERO-HOLDER NEGATIVE INVARIANT (load-bearing — the reason this task exists):** the marquee test `mute_all_revokes_all_publishers` MUST assert set-equality of revokes (`{P1,P2,P3,P4}`), NOT "a revoke fired". **Why it can't be deferred or weakened:** cr-4 (single-presenter gap) was caught in m3-core-stage-mode by CR precisely because a happy-path "command fired" assert passes even when the state machine leaves a holder behind — for an emergency-mute, a surviving publisher is a federation-wide safety failure. **DoD: the test MUST fail if the `for p in publishers { sink.apply(RevokePublish(p)) }` loop is deleted** (the delete-the-revoke check) — document this in a test comment. Per `feedback_authz_state_machine_test_asserts_negative.md`.
- **`mute_all` takes the EXPLICIT `publishers` slice** (NOT `self.current`) — `self.current` is single-presenter, so deriving from it would make the test exercise N=1 and miss the cr-4 N>1 failure mode. The plan §10.4 body is exact: copy it.
- **ADR-015 (pseudonyms, load-bearing):** `actor_pseudonym = self.chair.clone()` is a PSEUDONYM (the `stage.rs` contract — all stored strings are pseudonyms). A real identity in a hash-chained `room_mute_all` entry is permanent and unscrubable — it breaks `always_pseudonym` for the whole M3 cluster. **DoD: `rg -i 'person_id|username|@.*:' services/bridge/src/stage.rs` returns nothing identity-shaped** (only pseudonym strings + Matrix event keys). Per §10.6.
- **ADR-016 (metadata-only):** the `room_mute_all` payload carries `{ federated, actor_pseudonym }` — metadata only. No content/speech/video field. `federated: Some(bool)` is a metadata flag (was the mute federation-wide).
- **ALL 5 bridge literals get `federated`** — `stage.rs:219`, `:255`, `:297` get `federated: None`; `room_event_client.rs:89`, `:123` get `federated: None`; the new `mute_all` literal sets `federated: Some(federated)`. **Adding the struct field without updating all 5 → `brehon-bridge` fails to compile.** Both files are in this ONE commit (else compile breaks). Per §11.
- **registry count stays 72** — do NOT add or re-register any `ENTRY_KIND_*`; `room_mute_all` is shipped (Phase 2). EMIT only.
- **NO new dependency** — `Cargo.toml`/`Cargo.lock` unchanged. If you need one, STOP and raise a `kind: "blocker"` DQ.
- **Bridge compiles on Linux ONLY.** Write a `validate-pending-laptop-linux` DQ with `commands: ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml", "scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --bins stage::mute_all", "scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml --bins room_event_client"]`. **NOTE the `--bins` flag:** `brehon-bridge` is a binary crate (no lib target); the `#[cfg(test)]` unit tests in `src/` run under `--bins`, NOT a bare `test <filter>` (a bare filter hits the integration target and matches 0 unit tests — confirmed in Task 2 validation). Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; `bash scripts/brehon/dq-v3-append-fragment.sh <frag>.json --pending` to append. Commit + push the DQ on the worker branch, then **STOP** — do NOT run cargo/Docker yourself. Per `feedback_validate_pending_laptop_write_then_stop.md`.
- **DQ mid-task discipline:** any blocker → `kind: "blocker"` DQ, commit + push on the worker branch, stop.
- **Handover:** deliver the HANDOVER block inline in your task output (do NOT write a `.claude/PRPs/handovers/` file — blocked by the sensitive-file guard in the Junior worktree context).
- End the commit body with a `LESSON:` trailer if you find anything durable.

## §3a Handover from prior cohort (Cohort A — Tasks 1+2, shipped)

- **Task 1 (#726, `governance_log.rs`):** binary `RoomEventPayload.federated: Option<bool>` field added with `#[serde(default, skip_serializing_if = "Option::is_none")]` (load-bearing — keeps existing room emissions byte-identical). registry count 72 (unchanged). The bridge POST you emit in `room_mute_all` round-trips into this field.
- **Task 2 (#727, `mute_handler.rs`):** `compute_mute_all_override` (pure) + `mute_all_power_levels` (async, `#[allow(dead_code)]`) + `sanction_handler` `get_power_levels`/`put_power_levels` widened to `pub(crate)`. **That is the CROSS-INSTANCE authority (Matrix power-levels). YOUR `Stage::mute_all` is the LOCAL belt-and-suspenders (LiveKit revoke sweep) — the two are complementary, not duplicates.** Do NOT call `mute_all_power_levels` or any power-level fn from `stage.rs`; the local sweep is grant-only.
- **Validation env note (from Cohort A):** bridge unit tests need `--bins <filter>` not bare `test <filter>`. The `validate-pending-laptop-linux` command list above reflects this.

## §5 HANDOVER (worker fills at task end — return inline in task output)

```yaml
HANDOVER:
  task: m3-core-emergency-mute-task3
  filesModified: [services/bridge/src/stage.rs, services/bridge/src/room_event_client.rs]
  keyDecisions:
    - "Stage::mute_all takes explicit publishers slice; revokes EVERY one; pushes single room_mute_all EmitIntent with actor_pseudonym=chair, federated=Some(bool)"
    - "marquee test mute_all_revokes_all_publishers asserts set-equality {P1..P4}; delete-the-revoke check documented"
    - "federated mirror field added to bridge RoomEventPayload; all 5 existing literals get federated: None; same commit"
    - "registry count UNCHANGED 72; room_mute_all emit-only"
  validate_dq: <composite-id of the validate-pending-laptop-linux DQ>
  notes: "<exact mute_all signature + the GrantSink/Recorder test idiom Task 4's #[ignore] test reuses + whether the 5-literal compile held first-try>"
```
