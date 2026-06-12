---
phase: m2-late-2
role: impl-task
n: 4
authored: 2026-06-12
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-late-2
task_number: 4
parallel_group: cohort-2
---

# [role:impl-task] m2-late-2 Task 4 — Matrix power-level enforcement

## 1. Role + dispatch line

```
[role:impl-task] m2-late-2 task-4 bridge power-levels — see .claude/PRPs/briefs/m2-late-2-impl-task-4.md
```

## 2. Scope

Rewrite `handle_sanction_event` to apply room-relative Matrix power-level overrides using
`bridge_room::lookup_by_case` (added in Task 3). Add `get_power_levels`, `put_power_levels`,
and `compute_power_override` private helpers. Remove the now-superseded
`sanction_kind_to_power_level` function.

### IMPLEMENT

| File | Change |
|---|---|
| `services/bridge/src/sanction_handler.rs` | (A) Rewrite `handle_sanction_event` body (lines 121-144) per Pattern 10.4. (B) Add three private helper functions: `get_power_levels`, `put_power_levels`, `compute_power_override`. (C) Remove `sanction_kind_to_power_level` (lines 147-158). |

### Explicit out-of-scope

- Do NOT touch `crates/` — workspace is Tasks 1/2.
- Do NOT add ANY new bridge dependency to `services/bridge/Cargo.toml` (R-bridge-lock —
  would change `Cargo.lock` and trigger the Linux gate; the recommended approach reuses
  `reqwest`/`serde_json`/`tokio`/`rusqlite` already present).
- Do NOT add `services/bridge` to `workspace.members` or use `--workspace` (R9 hard rule).
- Do NOT write tests in this task — that is Task 5.
- Do NOT change `bridge_room.rs` or any file other than `sanction_handler.rs`.

## 3. Implementation patterns (verbatim from plan §10)

### Pattern 10.3 — Authenticated Matrix power-levels GET/PUT (room-relative override)

**Mirror:** `services/bridge/src/room_provisioner.rs:527-581` (bearer_auth + URL-encoded
room id + `state.http_client`); `services/bridge/src/puppet.rs:58` (`ensure_puppet`);
`services/bridge/src/appservice.rs:33-41` (AppState fields: `config`, `puppet_map`,
`http_client`, `bridge_db_path`); `services/bridge/src/config.rs:11,13,26` (`tuwunel_url`,
`as_token`, `bridge_callback_secret`).

Read ALL mirror files before writing any code.

```rust
/// Compute the room-relative power override per sanction_kind.
/// `content` is the fetched m.room.power_levels object.
/// Returns (target_level, reason_code).
fn compute_power_override(sanction_kind: &str, content: &serde_json::Value) -> (i64, &'static str) {
    let events_default = content.get("events_default").and_then(|v| v.as_i64()).unwrap_or(0);
    let msg_threshold = content
        .get("events").and_then(|e| e.get("m.room.message")).and_then(|v| v.as_i64())
        .unwrap_or(events_default);
    let voice_threshold = content
        .get("events")
        .and_then(|e| e.get("m.call.member").or_else(|| e.get("org.matrix.msc3401.call.member")))
        .and_then(|v| v.as_i64());
    match sanction_kind {
        // Silence on the message channel (one below the send threshold).
        "ban" | "mute" | "prevent_post" => (msg_threshold - 1, "power_level_reduced_below_post_threshold"),
        // Voice channel if a voice threshold exists, else conservative fallback.
        "mute_voice" => (voice_threshold.unwrap_or(events_default) - 1, "voice_power_reduced_fallback_post_threshold"),
        // No native primitive: reduce posting, flag that redaction/reach is not enforced.
        "hide_content" => (msg_threshold - 1, "redaction_not_available_in_m2_late_2"),
        "restrict_reach" => (msg_threshold - 1, "restrict_reach_translated_to_power_level_reduction"),
        _ => (msg_threshold - 1, "unknown_sanction_kind_default_post_reduction"),
    }
}
```

**GOTCHA (Pattern 10.3):** PUT to `/state/{type}/{key}` **replaces the entire content** —
never PUT a fragment like `{"users": {mxid: level}}` (it wipes `events_default`, `ban`, etc).
Always GET first, modify `content["users"][mxid] = level`, then PUT the whole merged object.
Power level `0` is the *default member* level — always compute `threshold - 1` from the
fetched state; the old `sanction_kind_to_power_level` static absolute values are wrong for
rooms where `events_default != 0`.

Matrix endpoint pattern (mirror `room_provisioner.rs:527-581`):
```
GET  {tuwunel_url}/_matrix/client/v3/rooms/{encoded_room_id}/state/m.room.power_levels
PUT  same URL, body = merged full content object
```
The room id contains `!` and `:` — URL-encode it (reuse the existing encode helper from
`room_provisioner.rs:531-533`). Use `state.http_client.get(&url).bearer_auth(&state.config.as_token)`.

### Pattern 10.4 — Handler ordering

Order in the rewritten `handle_sanction_event`:
1. **Bearer auth** → 401 if bad (before any side-effect) — mirror current auth-first shape
2. **`bridge_room::open(&state.bridge_db_path)`** → 500 on infra error (never `?`-propagate as 200)
3. **`bridge_room::lookup_by_case(conn, payload.case_id)`** → get `rooms: Vec<(String, String)>`
4. **`rooms.is_empty()`** → return `200 applied:false, reason="no rooms found for case_id"` **before** `ensure_puppet` (the no-rooms path makes zero Matrix calls — load-bearing for the dep-free unit test in Task 5)
5. **`state.puppet_map.ensure_puppet(&payload.subject_actor_pseudonym)`** → `mxid: String`
6. **Per-room loop** (best-effort — one room's failure MUST NOT abort others):
   - `get_power_levels(&state, &room_id)` → `content`
   - `(level, reason_code) = compute_power_override(&payload.sanction_kind, &content)`
   - Set `content["users"][&mxid] = level`
   - `put_power_levels(&state, &room_id, &content)`
   - On success: increment `rooms_applied`
   - On error: increment `rooms_failed`, log the error (do NOT `?`-propagate out of loop)
7. Build response: `applied = rooms_applied > 0`, `reason = format!("rooms_found={rooms_found} rooms_applied={rooms_applied} rooms_failed={rooms_failed}; {reason_code}")`

`rooms_found = rooms.len()`. `reason_code` from the last successful `compute_power_override`
call (or a suitable default if all rooms failed).

**Acting as the appservice (room creator → PL 100) is sufficient to write power-levels — no
`?user_id=` impersonation needed.**

### Helper function signatures

```rust
async fn get_power_levels(
    state: &AppState,
    room_id: &str,
) -> Result<serde_json::Value, /* error type matching existing handler errors */>

async fn put_power_levels(
    state: &AppState,
    room_id: &str,
    content: &serde_json::Value,
) -> Result<(), /* same error type */>
```

Mirror the error-type + `.json()` / `.text()` patterns from `room_provisioner.rs:527-581`.

## 4. Validation gate (validate-pending-laptop DQ — write then STOP)

After committing the change, write a `kind: "validate-pending-laptop"` DQ entry with:

```json
{
  "commands": [
    "bash -c 'cd services/bridge && cargo check'",
    "bash -c 'cd services/bridge && cargo clippy --no-deps -- -D warnings'"
  ],
  "branch": "<current-worktree-branch>",
  "phase_task": 4,
  "e2e_filter": null
}
```

**IMPORTANT — advisor will run these as Linux Docker commands.** The laptop advisor converts
these to:
```
scripts/brehon/cargo-linux.sh check  --manifest-path services/bridge/Cargo.toml
scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings
```
because the bridge does NOT compile on Windows (ruma-common E0119). Write the commands
exactly as above — the advisor handles the platform translation.

Commit the DQ entry + push the branch, then **STOP**.

Generate the DQ id via `bash scripts/brehon/dq-v3-new-entry.sh`.
Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`.

**R-bridge-lock check BEFORE committing:** verify `git diff services/bridge/Cargo.toml` and
`git diff services/bridge/Cargo.lock` are empty. If either changed (you added a new
dependency), STOP and file a `kind: "blocker"` DQ — adding a new bridge dependency triggers
the Linux gate (`validate-pending-laptop-linux`) which is a separate DQ from the
`validate-pending-laptop` entry above. The recommended approach adds NO new dependency.

## 5. Required reading

- `.claude/PRPs/plans/m2-late-2.plan.md` — §13 Task 4 spec, §10 Patterns 10.3+10.4
  (verbatim helper shapes + ordering contract), §7 R9/R-bridge-lock/R-timeout guardrails,
  §9 bridge mirror files list
- `services/bridge/src/sanction_handler.rs` (WHOLE FILE) — current stub to rewrite; auth
  shape at lines 90-110; `sanction_kind_to_power_level` at lines 147-158 (to REMOVE)
- `services/bridge/src/room_provisioner.rs:527-581` — **canonical exemplar** for authenticated
  Matrix call pattern: bearer_auth + URL-encoded room id + `state.http_client`
- `services/bridge/src/room_provisioner.rs:90-96` — `bridge_room::open` per-request pattern
- `services/bridge/src/appservice.rs:33-41` — AppState fields (config, puppet_map,
  http_client, bridge_db_path)
- `services/bridge/src/puppet.rs:58` — `ensure_puppet(&self, brehon_user: &str) -> Result<MatrixUserId>`
- `services/bridge/src/config.rs:11,13,26` — `tuwunel_url`, `as_token`, `bridge_callback_secret`
- `services/bridge/src/bridge_room.rs` — `lookup_by_case` added by Task 3 (verify it is
  present on the base branch before starting)
- `.claude/lessons/feedback_linux_compile_proof_is_a_gate.md` — bridge Cargo.lock change
  triggers the Linux gate; the recommended approach avoids new deps entirely

## 6. Constraints

- **R9 (CRITICAL):** `services/bridge` stays in root `Cargo.toml` `exclude`. NEVER `--workspace`
  for bridge cargo commands. NO `--features full` (bridge has no such feature).
- **NO cargo on EliteDesk (NO-CARGO-ON-ELITEDESK)** — write the validate-pending-laptop DQ
  and STOP. The daemon has no cargo on PATH.
- **NO new bridge dependency** — the fix reuses `reqwest`, `serde_json`, `tokio`, `rusqlite`
  already in `services/bridge/Cargo.toml`. If a new dep is added, STOP + file a DQ (R-bridge-lock).
- **Per-room loop is best-effort** — a per-room GET/PUT error increments `rooms_failed` and
  CONTINUES; never `?`-propagate out of the loop. One room failure must not abort the others.
- **`bridge_room::open` error → 500** (genuine infra fault), NOT 200 applied:false.
- **Empty rooms → 200 applied:false BEFORE ensure_puppet** (Pattern 10.4 ordering; load-bearing
  for Task 5 dep-free tests).
- **GET→merge→PUT full content** (Pattern 10.3 GOTCHA); never fragment PUT.
- **`sanction_kind_to_power_level` MUST be removed** — it is fully superseded by
  `compute_power_override`; leaving it causes dead-code lint failure.
- **DQ v3 id** — generate via `bash scripts/brehon/dq-v3-new-entry.sh`.
- **Mid-task push discipline** — commit DQ entry + push immediately after writing it.
- **Commit prefix** — `feat(bridge): ` subject, with `LESSON:` trailer if any non-obvious
  finding arises.

### Mandatory lesson fires (file-class table check)

| File class | Match? | Lesson injected |
|---|---|---|
| `services/bridge/` edits | ✅ | R9 constraint (§6 above); `feedback_linux_compile_proof_is_a_gate.md` (§5) |
| Any handler doing 2+ DB writes | ❌ bridge uses SQLite read-only in this path | — |
| `crates/server/tests/e2e.rs` edits | ❌ no e2e edits | — |
