# m1-a fix-impl-2 — CR appservice + main.rs fixes (cr-009, cr-010, cr-011)

## 1. Role + dispatch line

`[role:impl-task] m1-a fix-impl-2 — CR appservice+main fixes cr-009 cr-010 cr-011 — see .claude/PRPs/briefs/m1-a-fix-impl-2.md`

## 2. Scope

Fix three CR findings in `services/bridge/src/appservice.rs` and `services/bridge/src/main.rs`.

**R8 reminder:** `services/bridge/` is workspace-EXCLUDED from the Lemmy Cargo workspace. Use `anyhow::Result` (NOT `LemmyResult`). No Diesel, no actix, no `lemmy_*` imports. Uses axum + tokio.

### cr-009 — Missing capability check before provision (major)

File: `services/bridge/src/appservice.rs`

The `POST /admin/provision-room` handler currently provisions a room for any caller that presents a valid bearer token. Per ADR-014, admin operations require hardcoded capability checks against governance flags. At M1 scope, the bridge reads `messaging_enabled` from `BREHON_READ_URL` already (soft_pause). Add a simple guard: before provisioning, check that `messaging_enabled` is true (soft-pause check as a proxy for "bridge is in operating state"). Return HTTP 403 if messaging is disabled.

The simplest M1 implementation: add a check against `state.relay_enabled` (which is updated by the soft_pause poller from `governance_messaging_config.messaging_enabled`). If `!state.relay_enabled.load(Ordering::Relaxed)` → return 403 `{"error": "bridge is in soft-pause; provisioning disabled"}`.

Read `services/bridge/src/appservice.rs` to find the exact location of the provision handler and the AppState definition (relay_enabled is an `Arc<AtomicBool>`).

### cr-010 — Validate room_alias (low)

File: `services/bridge/src/appservice.rs`

In the same provision handler, the `room_alias` field from the JSON body currently silently defaults to `"brehon-default"` when missing. Return HTTP 400 instead.

Find the JSON body deserialization or the place where `room_alias` is read. If it's `Option<String>`, add: if `room_alias.is_none() || room_alias.as_deref().unwrap_or("").is_empty()` → return 400 `{"error": "room_alias is required"}`.

### cr-011 — Panic-on-serve fix in main.rs (major)

File: `services/bridge/src/main.rs`

Three issues:
1. `axum::serve(...).await.unwrap()` — should propagate error, not panic
2. The soft_pause poller task is spawned but the JoinHandle is dropped (`.await` not tracked), so panics in the poller are silently ignored
3. Use `tokio::select!` for coordinated shutdown so both the axum server and the poller task stop together

Read `services/bridge/src/main.rs` to see the current structure. The fix:

```rust
// Keep the JoinHandle for the soft_pause poller
let poller_handle = tokio::spawn(run_soft_pause_poller(state.clone()));

// Use select! so either shutdown signal stops both
tokio::select! {
    result = axum::serve(listener, app) => {
        if let Err(e) = result {
            tracing::error!("axum serve error: {e:#}");
        }
    }
    result = poller_handle => {
        match result {
            Ok(Ok(())) => tracing::info!("soft_pause poller exited cleanly"),
            Ok(Err(e)) => tracing::error!("soft_pause poller error: {e:#}"),
            Err(e) => tracing::error!("soft_pause poller panicked: {e}"),
        }
    }
}
```

Adapt to the actual current code structure — read the file first.

## 3. Required reading

- `services/bridge/src/appservice.rs` — read completely before editing (find AppState, provision handler, relay_enabled usage)
- `services/bridge/src/main.rs` — read completely before editing (find current axum::serve + spawn pattern)

## 4. Constraints

- **ONLY touch**: `services/bridge/src/appservice.rs` and `services/bridge/src/main.rs`
- **Do NOT touch**: `Cargo.toml`, `registration.yaml`, `docker-compose.yml`, `crates/`, `migrations/`, `tests/`
- **No `--workspace` or `--features full`** — bridge is workspace-excluded; validate with `cd services/bridge && cargo check`
- **Write `validate-pending-laptop` DQ entry** after committing: `commands: ["cd services/bridge && cargo check"]`, `branch: "phase-m1-a"`, `phase_task: "fix-impl-2"`
- Use `anyhow::Result` not `LemmyResult`. Use `std::sync::atomic::Ordering`
- **Commit**: subject `fix(bridge): CR cr-009/cr-010/cr-011 — capability check + room_alias validation + serve error propagation`
- **Push** `phase-m1-a` to origin before stopping
- **Stop after DQ write + push** — do NOT run cargo yourself

## 5. Validation gate

Write `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "commands": ["cd services/bridge && cargo check"],
  "branch": "phase-m1-a",
  "phase_task": "fix-impl-2"
}
```

Commit + push the DQ entry. Stop. Laptop advisor runs cargo check.
