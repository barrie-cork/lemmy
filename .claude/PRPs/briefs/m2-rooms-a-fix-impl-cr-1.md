---
role: impl-task
phase: m2-rooms-a
task_number: "cr-fix-1"
base_branch: phase-m2-rooms-a
requires: []
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md   # write DQ + STOP; no cargo on daemon
  # Bridge-only task — DoD is `cd services/bridge && cargo check`
  # R8 toolchain boundary: NEVER mix bridge and workspace cargo
  # Bridge tests use anyhow, NOT LemmyError — do NOT apply feedback_lemmy_error_no_std_error.md
---

# [role:impl-task] m2-rooms-a CR fix-1 — config.rs expect→context, room_provisioner oq009 logic + dead code

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a cr-fix-1 bridge CR findings — see .claude/PRPs/briefs/m2-rooms-a-fix-impl-cr-1.md`

Three CodeRabbit findings in `services/bridge/` (CR triage approved 2026-06-06):
- **cr-5:** `config.rs` line 51 — `.expect()` → `.context()?` for `BRIDGE_CALLBACK_SECRET`
- **cr-6:** `room_provisioner.rs` lines 126 + 329 — OQ-009 `room_has_messages` always false at provision (new room has 0 messages); fix both call sites to `let room_has_messages = false;` and remove the `query_room_event_count` call from the provision path
- **cr-7:** `room_provisioner.rs` line 523 — remove dead `oq009_threshold()` fn (all call sites replaced in T5; only `query_room_event_count` which we're also removing remains)

**Bridge toolchain only.** DoD: `cd services/bridge && cargo check`.

## §2 Scope

**Modify:**
1. `services/bridge/src/config.rs` — 1-line fix (`.expect()` → `.context()?`)
2. `services/bridge/src/room_provisioner.rs` — fix 2 call sites + remove 2 dead fns

**Do NOT:**
- Touch any `crates/**` workspace files
- Modify any other bridge files (`bridge_room.rs`, `soft_pause.rs`, `appservice.rs`, `main.rs`, etc.)
- Run `cargo test` (compile check only)
- Add new Cargo.toml dependencies

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md` — write DQ + STOP; no cargo on daemon
2. **R8:** bridge toolchain only; DoD = `cd services/bridge && cargo check`

**MIRROR refs — read before writing:**
- `services/bridge/src/config.rs` (full file — see `.context(...)? ` pattern on all other env vars; `BridgeConfig::from_env()` return type is `Result<Self>` so `?` propagation works)
- `services/bridge/src/room_provisioner.rs` lines 110–135 (jury path cr-6 fix site), lines 315–335 (appeal path cr-6 fix site), lines 519–555 (dead fns cr-7)

## §4 IMPLEMENT

### Fix 1 (config.rs): Replace `.expect()` with `.context()?`

**File:** `services/bridge/src/config.rs`

Find:
```rust
            bridge_callback_secret: std::env::var("BRIDGE_CALLBACK_SECRET")
                .expect("BRIDGE_CALLBACK_SECRET must be set"),
```

Replace with:
```rust
            bridge_callback_secret: std::env::var("BRIDGE_CALLBACK_SECRET")
                .context("BRIDGE_CALLBACK_SECRET env var required")?,
```

No other changes to config.rs.

### Fix 2 (room_provisioner.rs): OQ-009 provision-time reveal state

**File:** `services/bridge/src/room_provisioner.rs`

**Fix site A (jury path, ~lines 124–127):**

Find:
```rust
    // OQ-009: determine reveal state based on room event count
    let threshold = state.oq009_reveal_threshold.load(std::sync::atomic::Ordering::Relaxed) as usize;
    let room_has_messages = query_room_event_count(&state, &room_id).await >= threshold;
```

Replace with:
```rust
    // OQ-009: newly provisioned rooms have no messages — reveal state is false at creation.
    // The bridge re-checks on subsequent events via the soft-pause poller threshold.
    let room_has_messages = false;
```

**Fix site B (appeal path, ~lines 327–329):**

Find:
```rust
    // OQ-009: determine reveal state for appeal jurors
    let threshold = state.oq009_reveal_threshold.load(std::sync::atomic::Ordering::Relaxed) as usize;
    let room_has_messages = query_room_event_count(&state, &room_id).await >= threshold;
```

Replace with:
```rust
    // OQ-009: newly provisioned rooms have no messages — reveal state is false at creation.
    let room_has_messages = false;
```

### Fix 3 (room_provisioner.rs): Remove dead functions

**File:** `services/bridge/src/room_provisioner.rs`

Remove the entire `oq009_threshold()` function block:

```rust
/// OQ-009: reveal threshold (default 1; T4b wires the real config value from bridge-read route).
fn oq009_threshold() -> usize {
    1
}
```

Remove the entire `query_room_event_count()` function block (it is now unreachable — both call sites replaced in Fix 2):

```rust
/// OQ-009: query the Matrix room event count via GET /messages?limit=1.
/// Returns chunk length (0 on any error — fail-safe).
async fn query_room_event_count(state: &AppState, room_id: &str) -> usize {
    let encoded_room_id = room_id.replace(':', "%3A");
    let url = format!(
        "{}/_matrix/client/v3/rooms/{}/messages?limit=1",
        state.config.tuwunel_url, encoded_room_id
    );
    match state
        .http_client
        .get(&url)
        .bearer_auth(&state.config.as_token)
        .send()
        .await
    {
        Ok(resp) => match resp.json::<serde_json::Value>().await {
            Ok(json) => json
                .get("chunk")
                .and_then(|c| c.as_array())
                .map(|a| a.len())
                .unwrap_or(0),
            Err(_) => 0,
        },
        Err(e) => {
            tracing::warn!(err = %e, "query_room_event_count HTTP error");
            0
        }
    }
}
```

**After removing both fns:** verify no remaining references to `oq009_threshold` or `query_room_event_count` in room_provisioner.rs before committing. Also verify `state.oq009_reveal_threshold` (the T5 AtomicI64 load) is still present at the TWO non-fix-site locations (lines ~125 and ~329 become the `let room_has_messages = false;` lines — verify the `threshold` variable is no longer referenced after removal, so no unused-variable warning).

### Commit

```
fix(bridge): cr-5/6/7 — config expect→context, oq009 provision-always-false, remove dead fns
```

### Validate-pending-laptop DQ entry (write + STOP)

After committing, append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does `cd services/bridge && cargo check` pass after cr-5/6/7 fixes?",
  "options": ["pass", "fail"],
  "context": "CR fix-1: config.rs .expect()->context, room_provisioner oq009 provision-always-false + remove dead oq009_threshold/query_room_event_count fns. Bridge toolchain only.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["cd services/bridge && cargo check"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "cr-fix-1",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null,
  "id": "a3d0e9941441-056"
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a cr-fix-1`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Bridge toolchain ONLY: `cd services/bridge && cargo check`
- No new Cargo.toml dependencies — `.context()` already uses `anyhow::Context` which is in scope
- The `let room_has_messages = false;` replacement removes the `threshold` variable entirely from those two blocks — verify no dangling `threshold` reference remains after fix
- The `query_room_event_count` fn is async — removing it does not require `#[allow(dead_code)]`; it simply disappears
- Do NOT modify the `#[ignore]` tests in `tests/room_provisioning.rs` — they reference `room_has_messages` conceptually but don't call these fns directly

## §6 HANDOVER

Write `.claude/PRPs/handovers/m2-rooms-a-cr-fix-1-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the validate-pending-laptop
- confirmation both files modified (config.rs 1 line, room_provisioner.rs 2 fix sites + 2 dead fn removals)
- any clippy warnings seen (dead_code should drop to ≤4 after removal)
