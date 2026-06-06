---
role: impl-task
phase: m2-rooms-a
task_number: "5"
base_branch: phase-m2-rooms-a
requires: ["4a", "4b"]
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md   # write DQ + STOP; no cargo on daemon
  # Bridge-only task: services/bridge/ — DoD is `cd services/bridge && cargo check`
  # NOT --workspace --features full (R8 toolchain boundary)
---

# [role:impl-task] m2-rooms-a task-5 — bridge callback wiring + soft_pause bearer auth + OQ-009 threshold

## §1 Role + dispatch

`[role:impl-task] m2-rooms-a task-5 bridge callback wiring — see .claude/PRPs/briefs/m2-rooms-a-impl-5.md`

Wire the bridge side to use the T4a/T4b binary endpoints:
1. `soft_pause.rs` — add `Authorization: Bearer <secret>` header to the poll request; parse `oq009_reveal_threshold` from response; update `Arc<AtomicI64>` in AppState
2. `appservice.rs` — add `oq009_reveal_threshold: Arc<AtomicI64>` to `AppState`
3. `main.rs` — initialize `Arc::new(AtomicI64::new(1))`, wire into `AppState` + `run_poller`
4. `room_provisioner.rs` — replace `oq009_threshold()` stub with `state.oq009_reveal_threshold.load(Ordering::Relaxed) as usize`

**Bridge toolchain only** (`cd services/bridge && cargo check`).

## §2 Scope

**Produce:**
1. `services/bridge/src/soft_pause.rs` — add bearer header; parse `oq009_reveal_threshold`; accept `Arc<AtomicI64>` param
2. `services/bridge/src/appservice.rs` — add `oq009_reveal_threshold: Arc<AtomicI64>` to `AppState`
3. `services/bridge/src/main.rs` — init `Arc<AtomicI64>`, wire into `AppState` + `run_poller` call
4. `services/bridge/src/room_provisioner.rs` — replace `oq009_threshold()` call with `state.oq009_reveal_threshold.load(Ordering::Relaxed) as usize`

**Do NOT:**
- Touch any `crates/**` files (workspace boundary)
- Modify `bridge_read.rs`, `bridge_auth.rs`, or `room_event_handler.rs`
- Change `BridgeConfig` fields (config already has `bridge_callback_secret` and `brehon_read_url`)
- Delete the `oq009_threshold()` helper fn — replace its call site(s) but optionally keep it as dead code (or remove if clippy would complain)

## §3 Required reading

1. `feedback_validate_pending_laptop_write_then_stop.md` — write DQ + STOP; no cargo on daemon
2. **R8 toolchain boundary:** bridge-only. Validate with `cd services/bridge && cargo check`.

**MIRROR refs — read before writing:**
- `services/bridge/src/soft_pause.rs` (full file — current `run_poller` + `poll_once`; add bearer header; extend signature to accept `Arc<AtomicI64>`)
- `services/bridge/src/main.rs` (full file — `run_poller` call site at line 55; `AppState` construction at line 40; `AtomicBool` init at line 35 — mirror for `AtomicI64`)
- `services/bridge/src/appservice.rs` lines 33–40 (`AppState` struct fields — add `oq009_reveal_threshold` after `relay_enabled`)
- `services/bridge/src/room_provisioner.rs` lines 520–535 (`oq009_threshold()` stub + call site; `query_room_event_count` fn — the stub returns `1`; replace the `oq009_threshold()` call at lines ~125 and ~328 with `state.oq009_reveal_threshold.load(Ordering::Relaxed) as usize`)
- `services/bridge/src/config.rs` (full file — `bridge_callback_secret` field already exists; `brehon_read_url` is the URL the poller hits; no new fields needed)

## §4 IMPLEMENT

### File 1 (modify): `services/bridge/src/soft_pause.rs`

**Changes:**
1. Add `oq009_reveal_threshold: Arc<AtomicI64>` parameter to `run_poller`
2. Thread `bridge_callback_secret` from config into `poll_once`
3. In `poll_once`: add `Authorization: Bearer <secret>` header
4. In `poll_once`: also parse `oq009_reveal_threshold` from response (default 1); update the atomic

Updated shapes:

```rust
use std::sync::{
    atomic::{AtomicBool, AtomicI64, Ordering},
    Arc,
};

use anyhow::Result;
use crate::config::BridgeConfig;

pub async fn run_poller(
    config: Arc<BridgeConfig>,
    relay_enabled: Arc<AtomicBool>,
    oq009_reveal_threshold: Arc<AtomicI64>,
) {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .expect("build soft_pause HTTP client");
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        match poll_once(&client, &config.brehon_read_url, &config.bridge_callback_secret).await {
            Ok((enabled, threshold)) => {
                let prev = relay_enabled.swap(enabled, Ordering::Relaxed);
                if prev != enabled {
                    tracing::info!(messaging_enabled = enabled, "soft-pause state changed");
                }
                oq009_reveal_threshold.store(threshold, Ordering::Relaxed);
            }
            Err(e) => {
                tracing::warn!(err = %e, "soft-pause poll failed; keeping current state");
            }
        }
    }
}

async fn poll_once(client: &reqwest::Client, url: &str, secret: &str) -> Result<(bool, i64)> {
    let resp = client
        .get(url)
        .header("Authorization", format!("Bearer {}", secret))
        .send()
        .await?
        .error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    let enabled = body
        .get("messaging_enabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let threshold = body
        .get("oq009_reveal_threshold")
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    Ok((enabled, threshold))
}
```

### File 2 (modify): `services/bridge/src/appservice.rs`

Add one field to `AppState` after `relay_enabled`:

```rust
pub struct AppState {
    pub config: Arc<BridgeConfig>,
    pub puppet_map: Arc<crate::puppet::PuppetMap>,
    pub http_client: reqwest::Client,
    pub relay_enabled: Arc<std::sync::atomic::AtomicBool>,
    pub oq009_reveal_threshold: Arc<std::sync::atomic::AtomicI64>,
    /// Path to the SQLite database file for bridge_room state.
    pub bridge_db_path: String,
}
```

No other changes to `appservice.rs`.

### File 3 (modify): `services/bridge/src/main.rs`

Two changes:

1. Add `AtomicI64` to the existing `AtomicBool` import:
```rust
use std::sync::atomic::{AtomicBool, AtomicI64};
```

2. After `let relay_enabled = Arc::new(AtomicBool::new(true));`, add:
```rust
let oq009_reveal_threshold = Arc::new(AtomicI64::new(1));
```

3. Add `oq009_reveal_threshold` to `AppState` construction:
```rust
let app = appservice::router(Arc::new(AppState {
    config: Arc::clone(&config_arc),
    puppet_map,
    http_client: reqwest::Client::new(),
    relay_enabled: Arc::clone(&relay_enabled),
    oq009_reveal_threshold: Arc::clone(&oq009_reveal_threshold),
    bridge_db_path,
}));
```

4. Add `Arc::clone(&oq009_reveal_threshold)` to `run_poller` call:
```rust
let poller_handle = tokio::spawn(soft_pause::run_poller(
    Arc::clone(&config_arc),
    Arc::clone(&relay_enabled),
    Arc::clone(&oq009_reveal_threshold),
));
```

### File 4 (modify): `services/bridge/src/room_provisioner.rs`

Replace the two `oq009_threshold()` call sites (lines ~125 and ~328) with reads from `state`:

```rust
// Before:
let threshold = oq009_threshold();

// After:
let threshold = state.oq009_reveal_threshold.load(std::sync::atomic::Ordering::Relaxed) as usize;
```

The `oq009_threshold()` helper fn at line ~523 and `query_room_event_count` fn at ~527 are unchanged.
If removing `oq009_threshold()` to fix a dead_code warning: that's acceptable but not required.

### Validate-pending-laptop DQ entry (write + STOP)

After committing all 4 files, append via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "Does bridge cargo check pass after T5 callback wiring + bearer auth + OQ-009 threshold?",
  "options": ["pass", "fail"],
  "context": "T5 complete: soft_pause bearer header + oq009_reveal_threshold Arc<AtomicI64> wired into AppState + run_poller; room_provisioner reads from state. Bridge toolchain only.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["cd services/bridge && cargo check"],
  "branch": "phase-m2-rooms-a",
  "phase_task": "5",
  "workflow_run_id": null,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "failed_commands": null
}
```

Commit: `chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-5`
Push to `origin/phase-m2-rooms-a`. Then **STOP**.

## §5 Constraints

- Commit subject: `feat(bridge): wire soft_pause bearer auth + oq009_reveal_threshold (task 5)`
- Bridge toolchain ONLY: `cd services/bridge && cargo check`
- `AtomicI64` for `oq009_reveal_threshold` — matches `value_int: Option<i64>` in binary; `as usize` cast at call site is safe (threshold is always a small positive integer)
- The `brehon_read_url` now hits `GET /governance/bridge/messaging-status` (the T4b endpoint). No `BridgeConfig` field rename needed — operators update the env var
- Keep `query_room_event_count` unchanged
- `run_poller` signature change is backwards-incompatible — all call sites are in `main.rs` only (one call site total)

## §6 HANDOVER

Write `.claude/PRPs/handovers/m2-rooms-a-t5-done.md` with:
- last commit SHA on phase-m2-rooms-a
- DQ entry id for the new validate-pending-laptop
- confirmation all 4 files modified
- any `AtomicI64` import or ordering issues
