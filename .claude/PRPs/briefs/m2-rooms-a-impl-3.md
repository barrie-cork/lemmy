---
role: impl-task
phase: m2-rooms-a
task_number: "3"
base_branch: phase-m2-rooms-a
requires: ["2"]
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md
  # Bridge-only task: services/bridge/ — DoD is `cd services/bridge && cargo check`
  # NOT --workspace --features full (R8 toolchain boundary)
---

# [role:impl-task] m2-rooms-a task-3 — C2.2–C2.6 room scenarios + OQ-009 + always_pseudonym

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-3 room scenarios — see .claude/PRPs/briefs/m2-rooms-a-impl-3.md`

Extend `room_provisioner.rs` with the remaining 5 room scenarios (C2.2–C2.6),
OQ-009 graduated-reveal logic, and bridge-side `always_pseudonym` enforcement.
**Bridge toolchain only** (`cd services/bridge && cargo check`).

## §2 Scope

**Produce:**
- `services/bridge/src/room_provisioner.rs` — extend `handle_transition` with C2.2–C2.6

**Do NOT touch:**
- Any `crates/**` workspace files
- `appservice.rs` or `main.rs` (those are T2-complete)
- Any migration files

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md`
2. **R8:** bridge toolchain only — `cd services/bridge && cargo check`. Never `--workspace --features full`.

**MIRROR refs — read before writing:**
- `services/bridge/src/room_provisioner.rs` (current T2 jury path — all new match arms follow the same shape)
- `services/bridge/src/bridge_room.rs` (`lookup`, `upsert`, `BridgeRoom` struct — for per-room reveal state storage)
- `crates/api/api/src/governance/messaging_config.rs` lines 69–83 (binary-side `always_pseudonym` validator — mirror the semantics bridge-side; read-only reference, do NOT import from workspace)
- `services/bridge/src/provision.rs` (`create_community_room` — how community rooms are provisioned; emergency room uses same fn + member additions)
- `services/bridge/src/config.rs` (`BridgeConfig` — `legal_contact_mxid` field for C2.5 emergency rooms)

## §4 IMPLEMENT

### File 1: `services/bridge/src/room_provisioner.rs` (extend)

Extend `handle_transition` to cover remaining cases. All arms follow the T2 jury path shape:
soft-pause gate → idempotency check → provision → members → upsert.

**C2.2 community-event room:** triggered by a community-scoped transition (check
`event.community_id.is_some()`). Provision a community discussion room; no pseudonym
requirement; idempotency key `(case_id, "community")`.

**C2.3 spin-out room:** triggered when a case spins out from community to instance scope
(consult plan §9 for the status transitions that trigger spin-out). Same provisioning
shape; idempotency key `(case_id, "spinout")`.

**C2.4 appeal room:** triggered on `CaseStatus::Appealed`. Provision appeal room; invite
only the appeals panel jurors from `event.juror_pseudonyms` (same consume pattern as
C2.1 jury); `always_pseudonym` enforcement applies (see below). Idempotency key
`(case_id, "appeal")`.

**C2.5 emergency room (latency-critical, ADR-013):**
- Triggered on `CaseStatus::EmergencyRemove`
- Provision FIRST, defer any chain-emission to async callback (target <2s)
- Members: admins + `config.legal_contact_mxid`; **reported party ABSENT** (ADR-013)
- Idempotency key `(case_id, "emergency")`

**C2.6 membership-mirror room:** tracks community membership changes. Provision on the
relevant transition; idempotency key `(case_id, "membership")`.

**OQ-009 graduated-reveal logic:**
For `jury*` and `appeal*` rooms, before rendering member display names:
1. Query the Matrix room event count: `GET /_matrix/client/v3/rooms/{roomId}/messages?limit=1`
   (use the `matrix_sdk` client already available in state)
2. Fetch `oq009_reveal_threshold` from bridge config (T4b wires this; for now use a
   default of `1` if the config field is absent)
3. If event_count >= threshold: use `Juror-<suffix>` as display name
4. Else: use `Juror-pending` (opaque display name)
5. Persist the per-room reveal state in `bridge_room` (add a field or metadata column
   if the struct doesn't have one; keep it simple — a bool `reveal_applied`)

**`always_pseudonym` enforcement:**
For `jury*` and `appeal*` rooms, before provisioning:
- Check that `event.juror_pseudonyms` is non-empty (the producer sets this for pseudonym-gated transitions)
- If empty on a jury/appeal transition: log warn + skip provision (non-fatal, ADR-012)
- Do NOT reach into workspace code for this check — the bridge enforces it locally
  using the `event.juror_pseudonyms` field as the signal

### Validate-pending-laptop DQ entry (write + STOP)

After committing, append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does bridge cargo check pass after C2.2-C2.6 + OQ-009 + always_pseudonym?",
  "options": ["pass", "fail"],
  "context": "T3 complete: extended room_provisioner.rs with 5 room scenarios, OQ-009 reveal, always_pseudonym enforcement.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["cd services/bridge && cargo check"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "3",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-3`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Commit subject: `feat(bridge): extend room_provisioner with C2.2-C2.6 + OQ-009 + always_pseudonym (task 3)`
- Bridge toolchain ONLY: `cd services/bridge && cargo check`
- Emergency room (C2.5): provision first, defer chain-emission, <2s target (ADR-013)
- `always_pseudonym`: signal is `event.juror_pseudonyms` emptiness — no workspace import
- NO real identities in any room (ADR-015 across all scenarios)

## §6 HANDOVER

Write `.claude/PRPs/handovers/m2-rooms-a-t3-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation of which C2.* scenarios were implemented vs any that needed simplification
