---
role: impl-task
phase: m2-rooms-a
task_number: "2"
base_branch: phase-m2-rooms-a
requires: ["1", "1w"]
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # write DQ + STOP; no cargo on daemon
  # NOTE: feedback_features_full_workspace_only.md does NOT apply — T2 is bridge-only (services/bridge/,
  # workspace-EXCLUDED). DoD is `cd services/bridge && cargo check`, NOT --workspace --features full.
  # Per plan R8: bridge and workspace toolchains are orthogonal; never mix them.
---

# [role:impl-task] m2-rooms-a task-2 — room_provisioner.rs jury room core path

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-2 room provisioner — see .claude/PRPs/briefs/m2-rooms-a-impl-2.md`

Implement the C2.1 jury room provisioning path: create `room_provisioner.rs`,
wire a `POST /brehon/room-event` route in `appservice.rs`, add `mod room_provisioner`
to `main.rs`. All files under `services/bridge/` (bridge toolchain — NOT workspace).

Requires T1 (BridgeConfig + bridge_room store) and T1w (CaseTransitionEvent.juror_pseudonyms
on the wire).

## §2 Scope

**Produce:**
1. `services/bridge/src/room_provisioner.rs` — new file
2. `services/bridge/src/appservice.rs` — add `POST /brehon/room-event` route
3. `services/bridge/src/main.rs` — add `mod room_provisioner;`

**Do NOT:**
- Touch any `crates/**` files (workspace boundary — T2 is bridge-only)
- Block the HTTP ACK on any Matrix call (R3: must `tokio::spawn` the provisioner)
- Look up juror identities from a DB — consume `event.juror_pseudonyms` as provided (ADR-015)
- Add a jury room if `bridge_room::lookup(case_id, "jury")` already returns a row (idempotency)

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md` — write DQ + STOP; no cargo on daemon
2. **R8 toolchain boundary:** this task is `services/bridge/` only. Validate with
   `cd services/bridge && cargo check`. Never `--workspace --features full` for bridge code.

**MIRROR refs — read before writing:**
- `services/bridge/src/appservice.rs` lines 144–191 (`handle_provision_room`, soft-pause gate,
  `tokio::spawn` fire-and-forget pattern) — copy the spawn shape verbatim
- `services/bridge/src/provision.rs` (`create_community_room` signature)
- `services/bridge/src/puppet.rs` (`PuppetMap::ensure_puppet` signature)
- `services/bridge/src/relay.rs` lines 28–36 (payload-carries-identities consume pattern —
  `payload.brehon_sender`/`brehon_recipient`; T2's juror loop is the same shape over
  `event.juror_pseudonyms`)
- `services/bridge/src/bridge_room.rs` (`lookup` + `upsert` signatures — created by T1)
- `crates/api/api_common/src/governance.rs` (`CaseTransitionEvent` struct — read the
  `juror_pseudonyms: Vec<String>` field you'll be deserializing; also `BridgeNotifyPayload`
  enum so you deserialize the right variant)

## §4 IMPLEMENT

### File 1 (new): `services/bridge/src/room_provisioner.rs`

Implement `pub async fn handle_transition(state: AppState, event: CaseTransitionEvent)`.

C2.1 jury path (triggered when `event.new_status == CaseStatus::JurySelection` or when
`event.juror_pseudonyms` is non-empty — use whichever the plan's §10.2 says; default to
non-empty check as the signal):

```
(a) Check relay_enabled soft-pause gate — skip if false (§10.4)
(b) bridge_room::lookup(case_id, "jury") — if row exists, return (idempotency)
(c) provision::create_community_room(...) for the jury room
(d) for each pseudonym in event.juror_pseudonyms:
      puppet = PuppetMap::ensure_puppet(&pseudonym)
      invite puppet as "Juror-<suffix>" (suffix = last segment of pseudonym, or pseudonym itself)
    if event.juror_pseudonyms is empty: log warn + skip invite loop (non-fatal, ADR-012)
(e) bridge_room::upsert new room state
```

Error handling: log + swallow all Matrix transport errors (ADR-012, fire-and-forget).
NO real usernames/emails — `event.juror_pseudonyms` are already `actor_pseudonym.pseudonym`
values (ADR-015).

Idempotency key is `(case_id, "jury")` — a case may later get an appeal room too.

### File 2: `services/bridge/src/appservice.rs`

Add `POST /brehon/room-event` route. Mirror the existing `POST /brehon/notify` or
`POST /brehon/provision-room` handler shape from lines 144–191:

```rust
async fn handle_room_event(
    State(state): State<AppState>,
    Json(payload): Json<BridgeNotifyPayload>,
) -> impl IntoResponse {
    if let BridgeNotifyPayload::CaseTransition(event) = payload {
        tokio::spawn(room_provisioner::handle_transition(state, event));
    }
    Json(serde_json::json!({}))  // 200 immediately — R3
}
```

Wire into `router()` with `hs_token_auth` middleware (same as `/brehon/notify` —
the binary sends `hs_token` on this call too, matching the existing auth pattern).

### File 3: `services/bridge/src/main.rs`

Add `mod room_provisioner;` near the other `mod` declarations.

### Validate-pending-laptop DQ entry (write + STOP)

After committing all 3 files, append a `validate-pending-laptop` entry via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does bridge cargo check pass after adding room_provisioner.rs?",
  "options": ["pass", "fail"],
  "context": "T2 complete: room_provisioner.rs created (handle_transition, jury path), POST /brehon/room-event route added to appservice.rs, mod room_provisioner in main.rs. Bridge toolchain only.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["cd services/bridge && cargo check"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "2",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-2`
Push to `origin/phase-m2-rooms-a`. Then **STOP** — do NOT run cargo yourself.

## §5 Constraints

- Attribution: `answered_by: "impl"` on any DQ entries; never `"advisor"`
- Commit subject: `feat(bridge): add room_provisioner.rs jury room core path (task 2)`
- Bridge toolchain ONLY: `cd services/bridge && cargo check`. NEVER `--workspace --features full`.
- `tokio::spawn` the provisioner — never block the HTTP response on Matrix calls (R3)
- Consume `event.juror_pseudonyms` as-is — never look up real identities (ADR-015)
- Idempotency: check `bridge_room::lookup` before provisioning

## §6 HANDOVER (on completion)

Write `.claude/PRPs/handovers/m2-rooms-a-t2-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation that all 3 files were committed
- any surprises encountered (missing imports, API mismatches, etc.)
