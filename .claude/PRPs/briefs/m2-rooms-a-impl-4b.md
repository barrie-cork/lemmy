---
role: impl-task
phase: m2-rooms-a
task_number: "4b"
base_branch: phase-m2-rooms-a
requires: ["4a"]
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md   # write DQ + STOP
  - feedback_features_full_workspace_only.md              # --workspace --features full
  - feedback_governance_type_state_handlers.md            # handler conventions in governance crate
---

# [role:impl-task] m2-rooms-a task-4b — GET /governance/bridge/messaging-status (bridge-read route)

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-4b bridge messaging-status route — see .claude/PRPs/briefs/m2-rooms-a-impl-4b.md`

Add a bearer-authed read-only endpoint that returns `messaging_enabled` and
`oq009_reveal_threshold` — no JWT, no `is_admin`. The bridge's soft_pause poller
currently calls `BREHON_READ_URL` (which requires JWT → 401). T4b adds the correct
service-principal endpoint that fixes the 401 bug. **Workspace toolchain.**

## §2 Scope

**Produce:**
1. `crates/api/api/src/governance/bridge_read.rs` — new file
2. `crates/api/api/src/governance/mod.rs` — add `pub mod bridge_read;`
3. `crates/api/routes/src/lib.rs` — add `.route("/bridge/messaging-status", get().to(get_bridge_messaging_status))` inside `scope("/governance")`

**Do NOT:**
- Touch `services/bridge/**` files (bridge toolchain boundary — T5 wires the bridge side)
- Use `is_admin` or `LocalUserView` (service-principal auth)
- Touch migrations (no schema changes needed)
- Modify `bridge_auth.rs` or `room_event_handler.rs`

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md` — write DQ + STOP; no cargo on daemon
2. `feedback_features_full_workspace_only.md` — `--workspace --features full`; NEVER `-p lemmy_server`
3. `feedback_governance_type_state_handlers.md` — handler conventions in this crate

**MIRROR refs — read before writing:**
- `crates/api/api/src/governance/bridge_auth.rs` (full file — `verify_bridge_secret` signature; call as first line of handler)
- `crates/api/api/src/governance/messaging_config.rs` lines 160–183 (handler shape: `Data<LemmyContext>`, `LemmyResult<Json<...>>` return; how `context.pool()` + `GovernanceMessagingConfig::read_current` is used)
- `crates/db_schema/src/source/governance/governance_messaging_config.rs` lines 29–39 (`GovernanceMessagingConfig` struct — `value_bool: Option<bool>` for `messaging_enabled`; `value_int: Option<i64>` for `oq009_reveal_threshold`)
- `crates/api/api/src/governance/mod.rs` lines 28–32 (`bridge_auth` at line 30, `case_open_snapshot` at line 31 — `bridge_read` inserts between them alphabetically)
- `crates/api/routes/src/lib.rs` lines 479–484 (`scope("/governance")` block — T4a added `/room-event` at the top; add `/bridge/messaging-status` immediately after it)

## §4 IMPLEMENT

### File 1 (new): `crates/api/api/src/governance/bridge_read.rs`

Implement:

```rust
#[cfg(feature = "full")]
use actix_web::web::{Data, Json};
use actix_web::HttpRequest;
use lemmy_api_common::context::LemmyContext;
use lemmy_db_schema::source::governance::governance_messaging_config::GovernanceMessagingConfig;
use lemmy_utils::error::LemmyResult;
use serde::Serialize;

use super::bridge_auth;

#[derive(Debug, Serialize)]
pub struct BridgeStatus {
    pub messaging_enabled: bool,
    pub oq009_reveal_threshold: i64,
}

#[cfg(feature = "full")]
pub async fn get_bridge_messaging_status(
    req: HttpRequest,
    context: Data<LemmyContext>,
) -> LemmyResult<Json<BridgeStatus>> {
    bridge_auth::verify_bridge_secret(&req)?;
    let pool = &mut context.pool();

    let messaging_enabled = GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled")
        .await?
        .and_then(|r| r.value_bool)
        .unwrap_or(false);

    let oq009_reveal_threshold = GovernanceMessagingConfig::read_current(pool, "instance", "oq009_reveal_threshold")
        .await?
        .and_then(|r| r.value_int)
        .unwrap_or(1);

    Ok(Json(BridgeStatus {
        messaging_enabled,
        oq009_reveal_threshold,
    }))
}
```

Key points:
- `BridgeStatus` struct: no `#[cfg(feature = "full")]` needed on the struct itself (it only derives `Serialize`, no Diesel/Lemmy types)
- Handler: `#[cfg(feature = "full")]` required (uses `LemmyContext` + DB)
- No `LocalUserView`, no `is_admin` — service-principal auth via `bridge_auth::verify_bridge_secret`
- `messaging_enabled` default: `false` (safe: no messaging if config absent)
- `oq009_reveal_threshold` default: `1` (matches the current stub in `room_provisioner.rs`)

### File 2 (modify): `crates/api/api/src/governance/mod.rs`

Add one line in alphabetical position between `pub mod bridge_auth;` and `pub mod case_open_snapshot;`:

```
pub mod bridge_read;
```

The current lines 30–31 read:
```rust
pub mod bridge_auth;
pub mod case_open_snapshot;
```

After edit:
```rust
pub mod bridge_auth;
pub mod bridge_read;
pub mod case_open_snapshot;
```

### File 3 (modify): `crates/api/routes/src/lib.rs`

Add route and import.

**Route** — inside `scope("/governance")`, immediately after the existing `/room-event` line:
```rust
.route("/room-event", post().to(handle_room_event))
.route("/bridge/messaging-status", get().to(get_bridge_messaging_status))
```

**Import** — add to the governance use block at the top of the file:
```rust
use lemmy_api::governance::bridge_read::get_bridge_messaging_status;
```

### Validate-pending-laptop DQ entry (write + STOP)

After committing all 3 files, append via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does workspace cargo check pass after adding bridge_read.rs messaging-status route?",
  "options": ["pass", "fail"],
  "context": "T4b complete: bridge_read.rs bearer-authed GET /governance/bridge/messaging-status, BridgeStatus{messaging_enabled,oq009_reveal_threshold}. Workspace toolchain.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "4b",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-4b`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Commit subject: `feat(governance): add bridge_read messaging-status route (task 4b)`
- Workspace toolchain: `./scripts/brehon/cargo-check.sh --workspace --features full` (laptop only)
- No JWT auth, no `is_admin` — bearer secret only (`bridge_auth::verify_bridge_secret`)
- `get_bridge_messaging_status` is `#[cfg(feature = "full")]`; `BridgeStatus` struct needs no feature gate
- Single-read per field (no transaction needed — two independent reads, both read-only)
- The soft_pause poller in `services/bridge/src/soft_pause.rs` expects `{"messaging_enabled": bool}` — our response is a superset of that, so the poller continues to work without changes

## §6 HANDOVER

Write `.claude/PRPs/handovers/m2-rooms-a-t4b-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation that all 3 files were modified/created
- any import issues encountered with `GovernanceMessagingConfig`
