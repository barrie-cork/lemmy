# Brief: m3-core-infra fix-impl-cr-bridge (cr-6, cr-7, cr-8)

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-fix-cr-bridge — see .claude/PRPs/briefs/m3-core-infra-fix-impl-cr-bridge.md`

## §2 Scope

Fix 3 CodeRabbit findings (all `major`/`low`, all quick-wins) in `services/bridge/src/`,
approved fix-in-pr at gate-3 (PR #201). Three disjoint files, one commit.

- **cr-6** (`bridge_room.rs`): the `ALTER TABLE bridge_room ADD COLUMN` loop runs on EVERY
  `open()` — repeated schema-write locks risk `database is locked` under concurrency
  (`open()` is called in sanction_handler.rs L126/L408). Gate the ALTER behind a
  `PRAGMA table_info(bridge_room)` existence check so it runs at most once per missing column,
  not on every open.
- **cr-7** (`config.rs`): `env::var("LIVEKIT_*").ok()` returns `Some("")` for an empty-but-set
  env var — RTC then looks configured while `LIVEKIT_API_SECRET` is blank, weakening
  token-signing. Add an `optional_non_empty()` helper that trims + maps empty → `None`, and
  use it for `livekit_url` / `livekit_api_key` / `livekit_api_secret`.
- **cr-8** (`livekit_jwt.rs`): `mint_access_token` uses unchecked `now + ttl_secs` — large TTL
  overflows; `ttl_secs == 0` mints an immediately-invalid token. Add `ensure!(ttl_secs > 0)` +
  `checked_add` for `exp`.

**Produces:** edits to exactly these 3 files, one commit. Do NOT touch any other file.

**Do NOT:** edit `crates/`, `migrations/`, `crates/server/tests/`, `AGPL-NOTICE.md`,
`docker-compose.yml`, `.claude/`. cr-3 is being rebutted (NOT fixed); cr-4/cr-5 are a separate
worker. cr-1/cr-2 already landed advisor-side.

**Branch:** forks from `phase-m3-core-infra` (current tip `bfcf8aac9`).

## §3 Required reading

- `services/bridge/src/bridge_room.rs:19-31` — the ALTER loop (cr-6). Read the whole `open()` fn.
- `services/bridge/src/config.rs:82-85` — the `livekit_*` env reads (cr-7). Read the whole `from_env()`.
- `services/bridge/src/livekit_jwt.rs:25-50` — `mint_access_token` (cr-8). Read the whole fn + its `tests` mod.
- The CR finding bodies are summarized in §2 above (verbatim from `pr-201-findings.yaml`).

## §4 Constraints

### cr-6 — gate the ALTER behind PRAGMA table_info

Replace the unconditional per-column ALTER loop with: query `PRAGMA table_info(bridge_room)`
once, collect existing column names into a set, and ALTER only the columns NOT present. This
turns the per-open write-lock into a no-op when columns already exist (the common path). Keep
the "duplicate column name" error-swallow as a belt-and-braces fallback if you prefer, but the
PRAGMA check is the primary fix. Preserve idempotency + the existing column list
(`chair_id`, `queue_state`, `recording_config`).

### cr-7 — optional_non_empty helper

Add a small helper (module-private fn or inline closure):
```rust
fn optional_non_empty(var: &str) -> Option<String> {
    std::env::var(var).ok().map(|s| s.trim().to_string()).filter(|s| !s.is_empty())
}
```
Use it for `livekit_url`, `livekit_api_key`, `livekit_api_secret` instead of `env::var(...).ok()`.

### cr-8 — TTL guards

In `mint_access_token`, before building claims:
```rust
anyhow::ensure!(ttl_secs > 0, "ttl_secs must be > 0");
let exp = now.checked_add(ttl_secs).context("exp overflow")?;
```
Use `exp` for the claim. (The crate already uses `anyhow::Result` + `?`; mirror its error style —
read the existing `use` + return type first.)

### DoD (verify before committing — read-only)

1. `grep -c 'PRAGMA table_info' services/bridge/src/bridge_room.rs` → ≥1
2. `grep -c 'optional_non_empty' services/bridge/src/config.rs` → ≥2 (def + ≥1 use; ideally 4: def + 3 uses)
3. `grep -c 'checked_add\|ensure!(ttl_secs' services/bridge/src/livekit_jwt.rs` → ≥1

### Validate — Linux compile gate (delegated to laptop)

Write a `validate-pending-laptop-linux` DQ entry with:
```json
{
  "kind": "validate-pending-laptop-linux",
  "commands": ["./scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml"],
  "branch": "phase-m3-core-infra",
  "phase_task": "cr-bridge"
}
```
Commit + push, then **stop**. Do NOT run cargo yourself — the laptop advisor runs the Linux
compile gate (the bridge does not compile on Windows). End the commit body with a `LESSON:`
trailer. Commit subject: `fix(bridge): cr-6 PRAGMA-gated ALTER + cr-7 non-empty env + cr-8 TTL guards (fix-in-pr)`.
