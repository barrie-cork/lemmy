---
role: impl-task
phase: m2-rooms-a
task_number: "1w-fix"
base_branch: phase-m2-rooms-a
requires: []
mandatory_lessons_fired:
  - feedback_features_full_workspace_only.md  # workspace --features full; -p invalid
  - feedback_validate_pending_laptop_write_then_stop.md  # write DQ + STOP; no cargo on daemon
---

# [role:impl-task] m2-rooms-a task-1w-fix — add diesel deps to api_utils Cargo.toml

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-1w cargo-fix — see .claude/PRPs/briefs/m2-rooms-a-fix-impl-1w-cargo.md`

Fix-impl for DQ `6ad0b18d9dac-001` (result:fail). Cargo check returned 6 errors in
`lemmy_api_utils`: `diesel` and `diesel_async` are not declared as dependencies in
`crates/api/api_utils/Cargo.toml`. The `use` imports in `bridge_notify.rs` are correct —
they just need the crates present.

## §2 Scope

**Produce:**
1. `crates/api/api_utils/Cargo.toml` — add `diesel` and `diesel-async` as workspace
   dependencies (non-optional, unconditional)
2. Verify `crates/api/api_utils/src/bridge_notify.rs` use block is correct (read-only
   verify; do NOT edit unless the use block is wrong after adding deps)

**Do NOT:**
- Edit any file outside `crates/api/api_utils/`
- Add `#[cfg(feature = "full")]` gates (bridge_notify.rs is unconditionally compiled)
- Change the query logic in `fetch_juror_pseudonyms`
- Touch migrations, tests, or bridge (`services/`) files

## §3 Required reading

1. `feedback_features_full_workspace_only.md` — workspace `--features full`; never `-p lemmy_api_utils --features full`
2. `feedback_validate_pending_laptop_write_then_stop.md` — write `validate-pending-laptop` DQ entry + STOP; no cargo on daemon

**MIRROR refs — read before editing:**
- `crates/api/api_utils/Cargo.toml` lines 80–90 (existing non-optional workspace deps pattern: `lemmy_diesel_utils = { workspace = true }`, `diesel_ltree = { workspace = true }`)
- `Cargo.toml` (workspace root) lines containing `diesel =` and `diesel-async =` (verify workspace keys exist)

## §4 IMPLEMENT

### File 1: `crates/api/api_utils/Cargo.toml`

After line 87 (`diesel_ltree = { workspace = true }`), add:

```toml
diesel = { workspace = true }
diesel-async = { workspace = true }
```

These match the unconditional (non-optional) pattern of the existing `diesel_ltree` dep on line 87. The workspace root already declares both keys (`diesel = { version = "=2.3.10", ... }` and `diesel-async = "0.9.1"`).

### File 2 (verify only): `crates/api/api_utils/src/bridge_notify.rs`

Read lines 1–10. Confirm the use block includes:
```rust
use diesel::{ExpressionMethods, QueryDsl};
use diesel_async::RunQueryDsl;
```
If these lines are present and correct, NO edit needed — the deps being present will resolve all 6 errors.

If they are absent or wrong, add/fix them.

### Validate-pending-laptop DQ entry (write + STOP)

After committing the Cargo.toml fix, write this DQ fragment to `.claude/decision-queue.json`
using `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does workspace cargo check pass after adding diesel deps to api_utils?",
  "options": ["pass", "fail"],
  "context": "Fix-impl for DQ 6ad0b18d9dac-001: added diesel + diesel-async workspace deps to crates/api/api_utils/Cargo.toml. Commit on phase-m2-rooms-a.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "1w-fix",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-1w-fix`
Push to `origin/phase-m2-rooms-a`. Then **STOP** — do not run cargo yourself.

## §5 Constraints

- Attribution: `answered_by: "impl"` on any DQ entries you raise; never `"advisor"`
- Commit subject: `fix(api): add diesel deps to api_utils for juror-pseudonym query (task 1w-fix)`
- NO cargo on the daemon — write the DQ entry and stop
- `--workspace --features full` only; never `-p lemmy_api_utils`

## §6 HANDOVER (on completion)

Write `.claude/PRPs/handovers/m2-rooms-a-1w-fix-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation that bridge_notify.rs use block was verified correct (or what was changed)
