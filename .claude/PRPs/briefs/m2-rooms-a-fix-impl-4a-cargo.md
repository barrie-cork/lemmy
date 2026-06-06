---
role: impl-task
phase: m2-rooms-a
task_number: "4a-fix"
base_branch: phase-m2-rooms-a
requires: ["4a"]
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md   # write DQ + STOP
  - feedback_features_full_workspace_only.md              # --workspace --features full
---

# [role:impl-task] m2-rooms-a task-4a-fix — add Deserialize to RoomEventPayload

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-4a cargo fix Deserialize — see .claude/PRPs/briefs/m2-rooms-a-fix-impl-4a-cargo.md`

Narrow fix: `RoomEventPayload` in `governance_log.rs` is missing `serde::Deserialize`.
The `room_event_handler.rs` (T4a) deserializes it from JSON; the struct only derived
`Serialize`. **Workspace toolchain.**

§G4 allowlist match: E0277 missing derive on local type → add `Deserialize` to derive.

## §2 Scope

**Produce:**
1. `crates/api/api/src/governance/governance_log.rs` — add `serde::Deserialize` to the
   `#[derive(...)]` on `RoomEventPayload` (line 87, single-character change)

**Do NOT:**
- Touch any other files
- Modify any handler, route, or import
- Add `Deserialize` to `GovernanceLog` or any other struct (only `RoomEventPayload` is affected)

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md` — write DQ + STOP
2. `feedback_features_full_workspace_only.md` — `--workspace --features full`

**MIRROR ref:**
- `crates/api/api/src/governance/governance_log.rs` lines 84–93 (the `RoomEventPayload`
  struct — verify derive line before editing; the current derive is
  `#[derive(Debug, serde::Serialize)]`)

## §4 IMPLEMENT

### File 1 (modify): `crates/api/api/src/governance/governance_log.rs`

Change line 87 from:
```rust
#[derive(Debug, serde::Serialize)]
pub struct RoomEventPayload {
```

To:
```rust
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RoomEventPayload {
```

That is the complete fix. One word added to the derive list.

### Validate-pending-laptop DQ entry (write + STOP)

After committing the single file, append via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does workspace cargo check pass after adding Deserialize to RoomEventPayload?",
  "options": ["pass", "fail"],
  "context": "T4a-fix: added serde::Deserialize to RoomEventPayload derive in governance_log.rs. Fixes E0277 in room_event_handler.rs.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "4a-fix",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-4a-fix`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Commit subject: `fix(governance): add Deserialize to RoomEventPayload (task 4a-fix)`
- Workspace toolchain: `./scripts/brehon/cargo-check.sh --workspace --features full` (laptop only)
- Single file, single-word change — do NOT scope-creep
- `bridge_auth.rs` and `room_event_handler.rs` are NOT modified by this task

## §6 HANDOVER

Write `.claude/PRPs/handovers/m2-rooms-a-t4a-fix-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation that only `governance_log.rs` was modified
