# [role:impl-task] m1-a task-10 puppet-on-first-contact — see .claude/PRPs/briefs/m1-a-impl-10.md

## 1. Role + dispatch

`[role:impl-task] m1-a task-10 puppet-on-first-contact — see .claude/PRPs/briefs/m1-a-impl-10.md`

Branch from: `phase-m1-a` (current tip `e2397474b`)
Model: Sonnet 4.6

## 2. Scope

**Creates:**
- `services/bridge/src/puppet.rs` — `PuppetMap` + `ensure_puppet()` function

**Modifies:**
- `services/bridge/src/main.rs` — add `mod puppet;`; wire puppet map into `AppState`

**Does NOT create or modify:**
- `services/bridge/src/appservice.rs` — DO NOT touch; that file is Task 9 complete
- `services/bridge/src/relay.rs` — Task 11
- `services/bridge/src/soft_pause.rs` — Task 12
- `services/bridge/src/config.rs` — no new env vars needed for this task
- Any file outside `services/bridge/`
- `Cargo.toml` (no new deps — `matrix-sdk 0.18` already present from Task 8)

**Commit subject (exact):**
```
feat(bridge): puppet-on-first-contact — matrix-sdk AS client + bridge-local map (task 10)
```

## 3. Required reading

- **R8 (mandatory):** This crate is NOT Lemmy-workspace. Use `anyhow::Result` not `LemmyResult`. No Diesel, no `--features full`, no `use lemmy_*`. External conventions only.
- `services/bridge/src/config.rs` — read first: `BridgeConfig` fields (tuwunel_url, as_token, bridge_port, brehon_read_url). `homeserver_url()` below needs `tuwunel_url`.
- `services/bridge/src/appservice.rs` — read first: `AppState { pub config: Arc<BridgeConfig> }` and `pub fn router(state: Arc<AppState>) -> Router`. Task 10 adds `puppet_map` field to AppState.
- `services/bridge/src/main.rs` — read first: current `mod appservice; mod config;` pattern and how `AppState` is constructed.
- **`m1.plan.md` §13 Task 10** — IMPLEMENT + GOTCHA sections; bridge-local map is ephemeral-to-M1.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ, commit, push, STOP. Do NOT run cargo on the daemon.

**NO Lemmy lessons apply.** Do not inject: `feedback_lemmy_error_no_std_error.md`, `feedback_features_full_workspace_only.md`, Diesel/migration lessons, `feedback_async_pool_test_pattern.md`. Tree A is a different toolchain (R8).

## 4. Constraints

### §4.1 What to implement

**`services/bridge/src/puppet.rs`:**

```rust
// Bridge-local puppet account map.
//
// Maintains a simple in-memory map: Brehon user-id → Matrix user-id.
// Ephemeral-to-M1: persisted only for process lifetime. M2 replaces
// this with B-actor-linked portable IDs (ADR-016).
//
// GOTCHA: the puppet namespace MUST match registration.yaml
// (Task 13 wires the actual file). For M1, use a fixed localpart
// prefix "_brehon_" in the AS user namespace. E.g.:
//   @_brehon_alice:tuwunel.local

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::{Context, Result};
use matrix_sdk::{config::SyncSettings, Client};

use crate::config::BridgeConfig;

/// Brehon user-id (opaque string, e.g. username or UUID from Brehon PM).
pub type BrehonUserId = String;
/// Matrix user-id (fully qualified, e.g. @_brehon_alice:tuwunel.local).
pub type MatrixUserId = String;

pub struct PuppetMap {
    inner: Mutex<HashMap<BrehonUserId, MatrixUserId>>,
    config: Arc<BridgeConfig>,
}

impl PuppetMap {
    pub fn new(config: Arc<BridgeConfig>) -> Arc<Self> {
        Arc::new(Self {
            inner: Mutex::new(HashMap::new()),
            config,
        })
    }

    /// Return the Matrix user-id for `brehon_user`, creating a puppet if
    /// this is the first contact. Idempotent: repeated calls for the same
    /// user return the cached id without touching the homeserver.
    pub async fn ensure_puppet(&self, brehon_user: &str) -> Result<MatrixUserId> {
        // Fast-path: already in map.
        {
            let guard = self.inner.lock().unwrap();
            if let Some(mxid) = guard.get(brehon_user) {
                return Ok(mxid.clone());
            }
        }

        // Derive puppet localpart and mxid.
        let localpart = format!("_brehon_{brehon_user}");
        // The homeserver domain is extracted from tuwunel_url.
        // e.g. "http://localhost:8448" → "localhost:8448"
        let server_name = self
            .config
            .tuwunel_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .to_owned();
        let mxid = format!("@{localpart}:{server_name}");

        // Register/login the puppet via the AS admin token.
        // matrix-sdk Client with the AS token (not the puppet's own token —
        // for M1 we use the AS master token to register the puppet via the
        // /_matrix/client/v3/register?kind=guest endpoint or equivalent;
        // the exact API depends on Tuwunel version. For M1 the puppet
        // register call is best-effort: log and continue on error.
        let client = Client::builder()
            .homeserver_url(&self.config.tuwunel_url)
            .build()
            .await
            .context("build puppet matrix-sdk Client")?;

        // Attempt puppet registration. Tuwunel may reject duplicate usernames
        // with M_USER_IN_USE (puppet already exists) — treat as success.
        let register_result = client
            .matrix_auth()
            .register(
                matrix_sdk::ruma::api::client::account::register::v3::Request::new(),
            )
            .await;

        match register_result {
            Ok(_) => {
                tracing::info!(mxid = %mxid, "puppet registered");
            }
            Err(e) => {
                // M_USER_IN_USE → puppet already exists, not an error.
                let err_str = e.to_string();
                if err_str.contains("M_USER_IN_USE") {
                    tracing::debug!(mxid = %mxid, "puppet already exists, reusing");
                } else {
                    tracing::warn!(mxid = %mxid, err = %e, "puppet registration failed; continuing");
                }
            }
        }

        // Cache and return.
        self.inner
            .lock()
            .unwrap()
            .insert(brehon_user.to_owned(), mxid.clone());

        Ok(mxid)
    }
}
```

**`services/bridge/src/main.rs` modifications:**
1. Add `mod puppet;` after `mod appservice;`.
2. Import: `use puppet::PuppetMap;`
3. Create the puppet map before constructing `AppState`:
   ```rust
   let puppet_map = PuppetMap::new(Arc::clone(&config_arc));
   ```
   where `config_arc: Arc<BridgeConfig>`.
4. Add `puppet_map: Arc<PuppetMap>` field to `AppState` (in `appservice.rs`).
5. Pass `puppet_map` into `AppState { config: ..., puppet_map }`.

**`services/bridge/src/appservice.rs` — MINIMAL change only:**
Add `pub puppet_map: Arc<crate::puppet::PuppetMap>` field to `AppState`. That's it. Do NOT change handler logic (relay in Task 11).

### §4.2 Gotchas

- **Bridge-local map is ephemeral-to-M1** — do NOT add persistence (no sqlite file, no serde derive on PuppetMap). The inner `HashMap` is in-memory only. M2 replaces this entirely.
- **Puppet namespace** — localpart prefix `_brehon_` for M1. Registration.yaml (Task 13) will declare `"_brehon_.*"` as the user namespace regex. This must match.
- **matrix-sdk API shape** — `matrix_sdk::Client` uses an async builder. The `matrix_auth().register(...)` call shape may differ from what's above if matrix-sdk 0.18 uses a different path. READ the Cargo.toml `matrix-sdk = "0.18"` version and follow what compiles. Use `anyhow::Context` for all `.await?` calls.
- **No `#[cfg(feature = "full")]`** — R8. No feature gates.
- **No imports from `crate::appservice` in puppet.rs** — `puppet.rs` only imports `crate::config::BridgeConfig`. The dependency flows: `main.rs` → both `appservice` and `puppet`; `appservice.rs` imports `puppet::PuppetMap` via `crate::puppet`.
- **`AppState` lives in `appservice.rs`** — it's `pub struct AppState`. When you add `puppet_map` field, `main.rs` must construct it correctly.

### §4.3 validate-pending-laptop DQ (MANDATORY — write-then-stop)

After committing, write a `kind: "validate-pending-laptop"` entry to `.claude/decision-queue.json`, commit+push it, then **STOP**. Do NOT run `cargo check` yourself.

DQ entry shape:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<current worker branch>",
  "phase_task": 10,
  "commands": ["cd services/bridge && cargo check"],
  "question": "Tree-A Task 10: puppet-on-first-contact compiles inside services/bridge — laptop runs cargo check.",
  "options": ["pass", "fail"],
  "context": "Task 10 created services/bridge/src/puppet.rs (PuppetMap + ensure_puppet via matrix-sdk Client) and added mod puppet + PuppetMap field to AppState. VALIDATE: cd services/bridge && cargo check must be GREEN (exit 0). NOT --workspace. NOT --features full.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "id": "<generate via bash scripts/brehon/dq-v3-new-entry.sh>"
}
```

Generate id: `bash scripts/brehon/dq-v3-new-entry.sh`
Append via: `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`
Commit: `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised DQ <id> — task-10 validate-pending-laptop"`
Push to `origin <current-branch>`. Then STOP — do not run cargo check.

### §4.4 Attribution

Do NOT write `answered_by: "advisor"` in any DQ entry. You are `impl`.
Do NOT commit to `governance-v0` or `main`.
Commit only on the current worker branch.

## 5. Expected outputs

After this task:
- `services/bridge/src/puppet.rs` exists and is non-empty
- `services/bridge/src/main.rs` has `mod puppet;` and `PuppetMap` wired
- `services/bridge/src/appservice.rs` has `puppet_map: Arc<crate::puppet::PuppetMap>` in `AppState`
- A `validate-pending-laptop` DQ entry is in `pending[]` with `phase_task: 10`
- Worker branch pushed to origin
