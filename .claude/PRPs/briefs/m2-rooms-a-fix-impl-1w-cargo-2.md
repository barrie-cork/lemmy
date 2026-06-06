---
role: impl-task
phase: m2-rooms-a
task_number: "1w-fix-2"
base_branch: phase-m2-rooms-a
requires: ["1w-fix"]
mandatory_lessons_fired:
  - feedback_features_full_workspace_only.md
  - feedback_validate_pending_laptop_write_then_stop.md
---

# [role:impl-task] m2-rooms-a task-1w-fix-2 — add JoinOnDsl to bridge_notify.rs use block

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-1w fix-2 — see .claude/PRPs/briefs/m2-rooms-a-fix-impl-1w-cargo-2.md`

Fix-impl-2 for remaining E0599 after fix-impl-1 (DQ `f99a71bc9ed4-001`). After adding
diesel + diesel-async Cargo.toml deps, one error remains:

```
error[E0599]: no method named `on` found
  --> crates/api/api_utils/src/bridge_notify.rs:66:40
  help: trait `JoinOnDsl` which provides `on` is implemented but not in scope
  help: use diesel::JoinOnDsl;
```

The `use diesel::{ExpressionMethods, QueryDsl}` block is missing `JoinOnDsl`.

## §2 Scope

**Produce:** one edit to `crates/api/api_utils/src/bridge_notify.rs`:
- Add `JoinOnDsl` to the existing `use diesel::{ExpressionMethods, QueryDsl}` line

**Do NOT touch anything else.**

## §3 Required reading

1. `feedback_features_full_workspace_only.md`
2. `feedback_validate_pending_laptop_write_then_stop.md`

**MIRROR ref:** `crates/api/api/src/governance/submit_jury_vote.rs` line with
`BoolExpressionMethods, ExpressionMethods, JoinOnDsl, QueryDsl` — this is the canonical
pattern for importing all three together.

## §4 IMPLEMENT

### File 1: `crates/api/api_utils/src/bridge_notify.rs`

Find the line (near top of file):
```rust
use diesel::{ExpressionMethods, QueryDsl};
```

Change to:
```rust
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
```

That is the only change needed.

### Validate-pending-laptop DQ entry (write + STOP)

After committing, append a new `validate-pending-laptop` entry via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does workspace cargo check pass after adding JoinOnDsl to bridge_notify.rs?",
  "options": ["pass", "fail"],
  "context": "Fix-impl-2: added JoinOnDsl to use diesel block in bridge_notify.rs. This was the last remaining E0599 after fix-impl-1 added the Cargo.toml deps.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "1w-fix-2",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-1w-fix-2`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Commit subject: `fix(api): add JoinOnDsl to bridge_notify use block (task 1w-fix-2)`
- NO cargo on the daemon
- `--workspace --features full` only

## §6 HANDOVER

One-line: last commit SHA on phase-m2-rooms-a + new DQ entry id.
