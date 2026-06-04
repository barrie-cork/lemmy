# m1-a fix-impl-3 — CR timeout + puppet + relay fixes (cr-012, cr-013, cr-014, cr-015, cr-018)

## 1. Role + dispatch line

`[role:impl-task] m1-a fix-impl-3 — CR timeout+puppet+relay fixes cr-012 cr-013 cr-014 cr-015 cr-018 — see .claude/PRPs/briefs/m1-a-fix-impl-3.md`

## 2. Scope

Fix five CR findings across four files: `provision.rs`, `puppet.rs`, `relay.rs`, `soft_pause.rs`.

**R8 reminder:** `services/bridge/` is workspace-EXCLUDED. Uses `anyhow::Result`, `reqwest`, `axum`, `tokio`. No Diesel, no `lemmy_*`.

### cr-012 — Add HTTP timeout to provision.rs and relay.rs (major)

Files: `services/bridge/src/provision.rs` AND `services/bridge/src/relay.rs`

Both files create `reqwest::Client::new()` without a timeout. Add explicit timeouts using `reqwest::ClientBuilder`.

In `provision.rs` (wherever `Client::new()` or `reqwest::get` is used for the Matrix createRoom call):
```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()
    .context("build HTTP client")?;
```

In `relay.rs` (for the outbound POST to `brehon_notify_url`):
Same pattern — replace `Client::new()` or bare client with a builder that sets a 10-second timeout.

Read both files before editing to find the exact client construction sites.

### cr-013 — Sanitise brehon_user for Matrix MXID localpart (major)

File: `services/bridge/src/puppet.rs`

The `ensure_puppet` function derives a Matrix MXID localpart by prepending `_brehon_` to `brehon_user`. Matrix localparts may only contain: `[a-z0-9._\-=/+]` (per Matrix spec). A Lemmy username containing uppercase letters, spaces, or special characters will produce an invalid MXID.

Add escape/unescape helpers:
```rust
fn localpart_escape(s: &str) -> String {
    // Lowercase + replace any char not in [a-z0-9._\-=] with =XX hex encoding
    s.chars().flat_map(|c| {
        let lc = c.to_lowercase().next().unwrap_or(c);
        if lc.is_ascii_alphanumeric() || matches!(lc, '.' | '_' | '-') {
            vec![lc]
        } else {
            format!("={:02x}", lc as u32).chars().collect()
        }
    }).collect()
}
```

Use `localpart_escape(brehon_user)` when constructing the MXID localpart. This ensures the encoding is bijective (each unique brehon_user maps to a unique localpart) as long as Lemmy usernames don't use `=` (which they don't per Lemmy constraints).

### cr-014 — Fix stale MXID caching in ensure_puppet (major)

File: `services/bridge/src/puppet.rs`

Read `ensure_puppet` carefully. Currently it may insert into the puppet map before confirming the registration succeeded. Fix: only insert into the map on success (`M_USER_IN_USE` or 2xx registration). If the Matrix registration request fails for any other reason, return the error without caching.

Pattern:
```rust
// Only cache on confirmed-exists or newly-created
match register_result {
    Ok(_) | Err(MatrixError::UserInUse) => {
        map.insert(brehon_user.to_owned(), mxid.clone());
        Ok(mxid)
    }
    Err(e) => Err(e.into()),
}
```

Adapt to the actual code structure — read the file first.

### cr-015 — Return error when relay fails instead of ACKing (major)

File: `services/bridge/src/relay.rs`

The `handle_inbound` function (or the transaction handler) currently returns `Ok(())` even when the relay POST to `brehon_notify_url` fails, which causes Tuwunel to ACK the transaction and never retry.

Find where the relay failure is currently swallowed. Change it to propagate the error so the transaction is NOT ACKed and Tuwunel will retry:
- If the function currently ignores the `.context(...)` result or uses `let _ = ...`, change to `?` or explicit error return
- Return an appropriate axum error status (e.g. `StatusCode::BAD_GATEWAY`) so Tuwunel knows delivery failed

### cr-018 — Add timeout to soft_pause.rs client (major)

File: `services/bridge/src/soft_pause.rs`

Replace `Client::new()` with a builder that sets a timeout shorter than the poll interval. If the poll interval is 30 seconds (or whatever is configured), use a 10-second timeout:
```rust
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(10))
    .build()
    .context("build soft_pause HTTP client")?;
```

Read the file to find the current client construction site.

## 3. Required reading

- `services/bridge/src/provision.rs` — read before editing (find client construction)
- `services/bridge/src/puppet.rs` — read before editing (find ensure_puppet, localpart construction, map insert sites)
- `services/bridge/src/relay.rs` — read before editing (find client construction + handle_inbound error handling)
- `services/bridge/src/soft_pause.rs` — read before editing (find client construction)

## 4. Constraints

- **ONLY touch**: `services/bridge/src/provision.rs`, `services/bridge/src/puppet.rs`, `services/bridge/src/relay.rs`, `services/bridge/src/soft_pause.rs`
- **Do NOT touch**: `appservice.rs`, `main.rs`, `Cargo.toml`, `registration.yaml`, `docker-compose.yml`, `crates/`, `migrations/`, `tests/`
- **No `--workspace` or `--features full`** — workspace-excluded crate
- **Use `anyhow::Result`** throughout. No `LemmyResult`.
- **Commit**: subject `fix(bridge): CR cr-012/cr-013/cr-014/cr-015/cr-018 — timeouts + MXID sanitise + puppet cache + relay error propagation`
- **Write `validate-pending-laptop` DQ entry** after committing
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
  "phase_task": "fix-impl-3"
}
```

Commit + push. Stop. Laptop advisor runs cargo check.
