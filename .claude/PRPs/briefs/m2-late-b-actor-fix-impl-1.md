# Brief: m2-late-b-actor fix-impl-1 — CR fix-in-pr (bridge)

## 1. Role + dispatch

`[role:impl-task] m2-late-b-actor-fix-impl-1 — see .claude/PRPs/briefs/m2-late-b-actor-fix-impl-1.md`

## 2. Scope

Fix 3 CodeRabbit findings on PR #197, all in `services/bridge/` (workspace-excluded crate, validates on Linux via `cargo-linux.sh`).

### Fix 1 — cr-6: Cache key `(app_id, app_local_id)` not `app_local_id` alone

**File:** `services/bridge/src/app_actor_link.rs`

The SQLite table uses `app_local_id TEXT PRIMARY KEY` — this means two different apps with the same local user ID collide. Change:

1. Schema: composite PK `(app_id, app_local_id)` instead of `app_local_id` alone
2. `upsert()`: add `app_id` parameter, include in INSERT + ON CONFLICT
3. `lookup()`: add `app_id` parameter, filter by both columns
4. Update all callers in `link_handler.rs` to pass `app_id`

### Fix 2 — cr-8: Redact raw actor identifiers from logs

**File:** `services/bridge/src/link_handler.rs`

Lines logging `brehon_actor_id` (line 77 warn, line 189 info) expose the pseudonym UUID in logs. Per ADR-015, redact to first 8 chars:

- `brehon_actor_id = %payload.brehon_actor_id` → `brehon_actor_id = %&payload.brehon_actor_id[..8]` (or a helper)
- Apply to ALL tracing spans in this file that log `brehon_actor_id`

### Fix 3 — cr-9: Treat non-2xx confirm responses as failures

**File:** `services/bridge/src/link_handler.rs`

Line ~189: after `.send().await`, the current code only checks `Err(e)` (connection failure). A `200`-masked error (4xx/5xx from Brehon) is silently treated as success. Fix:

```rust
let resp = state.http_client
    .post(&state.config.brehon_link_confirm_url)
    .bearer_auth(&state.config.bridge_callback_secret)
    .json(&confirm_body)
    .send()
    .await;

match resp {
    Ok(r) if !r.status().is_success() => {
        tracing::error!(status = %r.status(), "link-claim: Brehon confirm returned non-2xx");
        return (StatusCode::BAD_GATEWAY, Json(serde_json::json!({ "error": "confirm returned non-2xx" }))).into_response();
    }
    Err(e) => {
        tracing::error!(err = %e, "link-claim: POST to brehon_link_confirm_url failed");
        return (StatusCode::INTERNAL_SERVER_ERROR, Json(serde_json::json!({ "error": "confirm POST to Brehon failed" }))).into_response();
    }
    Ok(_) => {}
}
```

### Validation

After all 3 fixes: `./scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`

Write a `validate-pending-laptop-linux` DQ entry with `commands: ["./scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml"]`, commit + push, then **stop**. Do NOT run `cargo-linux.sh` yourself — validation is delegated to the laptop advisor.

**Do NOT:**
- Touch any file outside `services/bridge/src/`
- Touch `crates/`, `migrations/`, `tests/`, `Cargo.lock` (workspace root)
- Run cargo yourself

## 3. Required reading

- `.claude/rules/branch-manager.md` — file-ownership (this task ONLY writes `services/bridge/src/`)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — not directly applicable (bridge uses anyhow), but read for error-shape awareness
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — mandatory: write DQ then STOP

## 4. Constraints

- Branch: `phase-m2-late-b-actor` (the open PR branch)
- Commit style: `fix(bridge): <summary> (cr-N)`
- One commit per finding OR one bundled commit — either is fine for bridge-only changes
- ADR-015 (load-bearing): `brehon_actor_id` in logs must be redacted. DoD: `grep brehon_actor_id services/bridge/src/link_handler.rs` shows NO full-UUID logging
- The bridge crate is workspace-EXCLUDED — it uses `anyhow`, NOT `LemmyResult`/`LemmyError`
- `services/bridge/` validates on Linux Docker only — never Windows-local cargo
