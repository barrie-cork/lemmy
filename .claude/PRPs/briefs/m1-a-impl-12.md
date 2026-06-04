# Brief: m1-a impl Task 12 — room provisioning + soft-pause

## 1. Role + dispatch line

```
[role:impl-task] m1-a-task-12-provision-softpause — see .claude/PRPs/briefs/m1-a-impl-12.md
```

Model: `claude-sonnet-4-6` | Effort: medium

## 2. Scope

**Task 12** from `m1.plan.md §13`: add the manual room-provisioning command surface and the soft-pause drain logic to the `services/bridge/` crate.

### CREATES:
- `services/bridge/src/provision.rs` — CLI-style admin HTTP endpoint (or standalone function) to manually create a Matrix community room; M1 has NO governance-triggered rooms — manual only.
- `services/bridge/src/soft_pause.rs` — poll `messaging_enabled` (HTTP GET `brehon_read_url`); on `false` → set a shared `Arc<AtomicBool>` flag to drain in-flight relays to idle + stop accepting new; on `true` → resume. Poll interval: 10s.

### MODIFIES:
- `services/bridge/src/main.rs` — replace the TODO stub poller (lines 40–49) with a real spawn of `soft_pause::run_poller(config_arc, relay_enabled)`. Add `mod provision;` and `mod soft_pause;`. Wire the `relay_enabled` flag into the AppState so `handle_transactions` skips relay when paused.

### Does NOT touch:
- `services/bridge/src/relay.rs` — read it for context, do NOT modify.
- `services/bridge/src/puppet.rs` — read only.
- `services/bridge/src/appservice.rs` — modify ONLY to add `relay_enabled: Arc<AtomicBool>` to `AppState` + pass it to `handle_transactions`.
- `services/bridge/src/config.rs` — read only; no new fields needed.
- Anything under `crates/` — R8 hard boundary: Tree A is workspace-EXCLUDED.
- `Cargo.toml` — no new dependencies needed (tokio, reqwest, serde_json, anyhow, tracing all already present).

### What to PRODUCE:

**`soft_pause.rs`**:
```rust
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use anyhow::Result;
use crate::config::BridgeConfig;

/// Runs forever: polls Brehon's read endpoint for messaging_enabled.
/// Sets `relay_enabled` accordingly. Never panics; logs errors and continues.
pub async fn run_poller(config: Arc<BridgeConfig>, relay_enabled: Arc<AtomicBool>) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
    loop {
        interval.tick().await;
        match poll_once(&client, &config.brehon_read_url).await {
            Ok(enabled) => {
                let prev = relay_enabled.swap(enabled, Ordering::Relaxed);
                if prev != enabled {
                    tracing::info!(messaging_enabled = enabled, "soft-pause state changed");
                }
            }
            Err(e) => {
                tracing::warn!(err = %e, "soft-pause poll failed; keeping current state");
            }
        }
    }
}

async fn poll_once(client: &reqwest::Client, url: &str) -> Result<bool> {
    let resp = client.get(url).send().await?.error_for_status()?;
    let body: serde_json::Value = resp.json().await?;
    Ok(body.get("messaging_enabled").and_then(|v| v.as_bool()).unwrap_or(false))
}
```

**`provision.rs`**:
- A single public function `pub async fn create_community_room(config: &BridgeConfig, room_alias: &str) -> Result<String>` that sends a Matrix `_matrix/client/v3/createRoom` request (via `reqwest`) using the AS master token; returns the created room ID.
- M1: no routing surface (admin HTTP route NOT added to the router in this task — that is Task 13's wiring). Provision.rs is a module with the function only; Task 13 will expose it.

**`main.rs` changes**:
- Add `mod provision;` and `mod soft_pause;` at top.
- Create `let relay_enabled = Arc::new(AtomicBool::new(true));` after puppet_map.
- Add `relay_enabled: Arc::clone(&relay_enabled)` to `AppState` construction.
- Replace the TODO stub poller spawn with:
  ```rust
  let _poller = tokio::spawn(soft_pause::run_poller(Arc::clone(&config_arc), Arc::clone(&relay_enabled)));
  ```

**`appservice.rs` changes**:
- Add `pub relay_enabled: Arc<std::sync::atomic::AtomicBool>` to `AppState`.
- In `handle_transactions`: before calling `relay::handle_inbound(...)`, check `state.relay_enabled.load(std::sync::atomic::Ordering::Relaxed)` and return early (HTTP 200, empty body) if false, logging "relay paused — messaging_enabled=false".

### VALIDATION (write-then-stop):
Write the `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "commands": ["cd services/bridge && cargo check"],
  "phase_task": 12
}
```
Then commit + push. Do NOT run `cargo check` yourself on the daemon.

## 3. Required reading (read BEFORE writing any code)

1. `.claude/PRPs/plans/m1.plan.md` §13 Task 12 (the full IMPLEMENT + GOTCHA block) — primary spec.
2. `services/bridge/src/main.rs` on `phase-m1-a` — this is where the TODO stub lives that you replace.
3. `services/bridge/src/appservice.rs` on `phase-m1-a` — `AppState` struct and `handle_transactions` handler.
4. `services/bridge/src/relay.rs` on `phase-m1-a` — context only; do NOT modify.
5. `services/bridge/src/config.rs` on `phase-m1-a` — `BridgeConfig` fields (`brehon_read_url` is the poll URL).
6. `services/bridge/Cargo.toml` on `phase-m1-a` — confirm no new deps needed (tokio, reqwest, serde_json, anyhow, tracing present).
7. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ then STOP, do not run cargo on daemon.
8. `.claude/lessons/feedback_read_canonical_before_writing_spec.md` — read sibling modules before authoring new ones (Tree A R8).

### R8 reminder (non-negotiable):
- This crate uses `anyhow` errors, NOT `LemmyError` or `LemmyResult`.
- No Diesel, no `lemmy_*` imports, no `--features full`.
- Follow external Rust crate conventions, NOT Lemmy workspace conventions.
- Cargo runs on the laptop (validate-pending-laptop), NOT on the EliteDesk daemon.

## 4. Constraints

1. **R8 boundary (catch-fire):** NEVER import from `crates/`, `lemmy_*`, `LemmyResult`, `LemmyError`, or Diesel. If a Lemmy-workspace lesson applies, it does NOT apply here.
2. **Workspace exclusion:** `services/bridge/` is in `exclude` in root `Cargo.toml`. Do NOT add it to `members`. Do NOT run `cargo check --workspace` or `cargo check --features full`.
3. **validate-pending-laptop — write then stop:** Write the DQ entry with `commands: ["cd services/bridge && cargo check"]`, commit + push the DQ entry, then stop. Do NOT run cargo on the daemon.
4. **AtomicBool for soft-pause:** The relay_enabled flag must be `Arc<AtomicBool>`, not a Mutex — the soft-pause poller is `tokio::spawn`'d and must not hold a lock across .await.
5. **Reversible without restart (GOTCHA from plan):** The process, puppet_map, and AS server must stay alive during soft-pause. Only relay acceptance/drains. Do NOT kill threads or drop state.
6. **provision.rs is a function module only in Task 12:** No HTTP route registration in this task. The `create_community_room` function is pub but not yet wired to the router.
7. **DQ mid-task push:** After writing the validate-pending-laptop DQ entry, commit and push immediately: `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised DQ <id> — task-12 validate-pending-laptop" && git push origin <current-branch>`.
8. **Attribution:** NEVER write `answered_by: "advisor"`. If self-resolving a blocker, use `answered_by: "impl-self-resolved"`. NEVER write `approved_by` from this session.
9. **base_branch is `phase-m1-a`** — branch from `phase-m1-a` at `9b67b1dd6`. All commits land on your worker branch (Junior will merge to phase-m1-a on finalize).
10. **No `use std::atomic` bare import:** Use `use std::sync::atomic::{AtomicBool, Ordering}` (the full path). Rust 2021 edition; follows patterns in main.rs (explicit `std::` qualified refs).
