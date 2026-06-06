---
role: impl-task
phase: m2-rooms-a
task_number: "4a"
base_branch: phase-m2-rooms-a
requires: ["3"]
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md   # workspace cargo; write DQ + STOP
  - feedback_features_full_workspace_only.md              # --workspace --features full; NOT -p lemmy_server
  - feedback_governance_type_state_handlers.md            # handler shape in governance crate
---

# [role:impl-task] m2-rooms-a task-4a — binary POST /governance/room-event + bridge bearer guard

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-4a binary room-event route — see .claude/PRPs/briefs/m2-rooms-a-impl-4a.md`

Add the binary-side HTTP callback route exposing `append_room_event`, plus the shared
`BRIDGE_CALLBACK_SECRET` bearer guard that both T4a and T4b use.
**Workspace toolchain** (`./scripts/brehon/cargo-check.sh --workspace --features full`).

## §2 Scope

**Produce:**
1. `crates/api/api/src/governance/bridge_auth.rs` — new file
2. `crates/api/api/src/governance/room_event_handler.rs` — new file
3. `crates/api/api/src/governance/mod.rs` — add `pub mod bridge_auth;` + `pub mod room_event_handler;`
4. `crates/api/routes/src/lib.rs` — add `.route("/room-event", post().to(handle_room_event))` inside `scope("/governance")`

**Do NOT:**
- Touch any `services/bridge/**` files (bridge toolchain boundary — workspace tasks only)
- Use `is_admin` or `LocalUserView` in the new routes (service-principal auth, NOT user auth)
- Touch migrations (none needed for this task)
- Wrap in `run_transaction` (single-write handler, per `feedback_multi_write_handlers_need_transactions.md`)

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md` — write DQ + STOP; no cargo on daemon
2. `feedback_features_full_workspace_only.md` — `--workspace --features full`; NEVER `-p lemmy_server`
3. `feedback_governance_type_state_handlers.md` — handler conventions in this crate

**MIRROR refs — read before writing:**
- `crates/api/api/src/governance/messaging_config.rs` lines 160–183 (handler shape: `HttpRequest`, `Data<LemmyContext>`, `LemmyResult<Json<...>>` return type; how `context.pool()` is used)
- `crates/api/api/src/governance/governance_log.rs` lines 88–140 (`RoomEventPayload` struct fields, `ROOM_KINDS` const array, `append_room_event` full signature: `pool: &mut DbPool<'_>, kind: &str, payload: RoomEventPayload, actor_pseudonym: Option<String>`)
- `services/bridge/src/appservice.rs` lines 52–90 (bearer-extraction shape in axum — re-implement the `Authorization: Bearer <secret>` extract logic in actix-web style for `bridge_auth.rs`)
- `crates/api/routes/src/lib.rs` lines 478–535 (`scope("/governance")` block — add the new route at the TOP of the governance scope, before the `/admin` sub-scope)
- `crates/api/api/src/governance/mod.rs` (full list of `pub mod` declarations — `bridge_auth` goes alphabetically between `appeal_window_expiry` and `case_open_snapshot`; `room_event_handler` goes between `reputation_snapshot` and `sponsor_liability`)

## §4 IMPLEMENT

### File 1 (new): `crates/api/api/src/governance/bridge_auth.rs`

Implement:
```rust
use actix_web::HttpRequest;
use lemmy_api_common::LemmyErrorType;
use lemmy_utils::error::LemmyResult;

/// Verifies the Authorization: Bearer <secret> header for bridge-to-binary callbacks.
/// Service-principal auth — NOT is_admin, NOT JWT. Called as first line of T4a and T4b handlers.
pub fn verify_bridge_secret(req: &HttpRequest) -> LemmyResult<()> {
    let expected = std::env::var("BRIDGE_CALLBACK_SECRET")
        .unwrap_or_default();
    let provided = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .unwrap_or("");
    // Plain == is acceptable for v0 (constant-time compare adds a dep for low-value gain)
    if provided.is_empty() || provided != expected {
        return Err(LemmyErrorType::NotLoggedIn.into());
    }
    Ok(())
}
```

### File 2 (new): `crates/api/api/src/governance/room_event_handler.rs`

Implement:

```rust
#[cfg(feature = "full")]
use actix_web::web::{Data, Json};
use actix_web::HttpRequest;
use lemmy_api_common::context::LemmyContext;
use lemmy_utils::error::LemmyResult;
use serde::Deserialize;

use super::{bridge_auth, governance_log::{append_room_event, RoomEventPayload}};

#[derive(Debug, Deserialize)]
pub struct RoomEventRequest {
    pub entry_kind: String,
    pub payload: RoomEventPayload,
    pub actor_pseudonym: Option<String>,
}
```

Handler:
```rust
#[cfg(feature = "full")]
pub async fn handle_room_event(
    req: HttpRequest,
    body: Json<RoomEventRequest>,
    context: Data<LemmyContext>,
) -> LemmyResult<Json<serde_json::Value>> {
    bridge_auth::verify_bridge_secret(&req)?;
    let pool = &mut context.pool();
    append_room_event(pool, &body.entry_kind, body.into_inner().payload, body.into_inner().actor_pseudonym).await?;
    Ok(Json(serde_json::json!({})))
}
```

Note: `body.into_inner()` is called twice — extract the inner value first to avoid the move:
```rust
let req_body = body.into_inner();
append_room_event(pool, &req_body.entry_kind, req_body.payload, req_body.actor_pseudonym).await?;
```

The `#[cfg(feature = "full")]` gate applies to both the struct and the handler because
`LemmyContext` and `append_room_event` are behind that feature gate.

### File 3 (modify): `crates/api/api/src/governance/mod.rs`

Add two lines in alphabetical position:
- `pub mod bridge_auth;` — between `pub mod appeal_window_expiry;` and `pub mod case_open_snapshot;`
- `pub mod room_event_handler;` — between `pub mod reputation_snapshot;` and `pub mod sponsor_liability;`

### File 4 (modify): `crates/api/routes/src/lib.rs`

Inside the `scope("/governance")` block (around line 479), add the new route at the top
(before the existing `.route("/report", ...)` line or after the `.wrap(rate_limit.post())` line):

```rust
.route("/room-event", post().to(handle_room_event))
```

Add the corresponding import at the top of the file's governance use block:
```rust
use lemmy_api::governance::room_event_handler::handle_room_event;
```

### Validate-pending-laptop DQ entry (write + STOP)

After committing all 4 files, append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does workspace cargo check pass after adding bridge_auth.rs + room_event_handler.rs?",
  "options": ["pass", "fail"],
  "context": "T4a complete: bridge_auth.rs bearer guard + handle_room_event handler + mod.rs + routes wired. Workspace toolchain.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "4a",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-4a`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Commit subject: `feat(governance): add bridge_auth guard + room_event_handler callback route (task 4a)`
- Workspace toolchain: `./scripts/brehon/cargo-check.sh --workspace --features full` (laptop only — NO cargo on daemon)
- `bridge_auth::verify_bridge_secret` is `pub fn` (not async) — called as first line of handler
- Service-principal auth: NO `LocalUserView`, NO `is_admin` in bridge routes
- `handle_room_event` and `RoomEventRequest` struct are `#[cfg(feature = "full")]`
- The `/room-event` route inherits `rate_limit.post()` from the `scope("/governance")` wrapper — acceptable for bridge call volume (one per provisioning action)

## §6 HANDOVER

Write `.claude/PRPs/handovers/m2-rooms-a-t4a-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation that all 4 files were modified/created
- any import resolution issues encountered
