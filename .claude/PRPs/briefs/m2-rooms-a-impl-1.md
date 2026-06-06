---
phase: m2-rooms-a
role: impl-task
n: 1
authored: 2026-06-06
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-rooms-a
task_number: 1
requires: []
minimax_trial: not-eligible (4 files > 2-file criterion)
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # R4 — every task; workers write DQ and STOP
---

# [role:impl-task] m2-rooms-a Task 0+1 — pre-flight probes + BridgeConfig extension + rusqlite bridge_room store

## 1. Role + dispatch line

```
[role:impl-task] m2-rooms-a task-0+1 preflight+config+store — see .claude/PRPs/briefs/m2-rooms-a-impl-1.md
```

## 2. Scope

**Task 0 (no commit):** run the 7 pre-flight probes from plan §13 Task 0. Report results inline. If any probe fails, write a `kind: "blocker"` DQ entry and STOP.

**Task 1 (one commit):** add `rusqlite` to the bridge crate; extend `BridgeConfig` with three new env-backed fields and repoint `brehon_read_url`; create the `bridge_room` rusqlite store module.

**Produce (Task 1 — exactly 4 files):**

```yaml
creates:
  - services/bridge/src/bridge_room.rs
modifies:
  - services/bridge/Cargo.toml          # add rusqlite = { version = "0.32", features = ["bundled"] }
  - services/bridge/src/config.rs       # +brehon_room_event_url, +bridge_callback_secret, +legal_contact_mxid; repoint brehon_read_url comment
  - services/bridge/src/main.rs         # add `mod bridge_room;`
requires: []
```

**Pre-Shape-G: write a `validate-pending-laptop` DQ entry with `commands: ["cd services/bridge && cargo check"]`, commit + push, then STOP. Do NOT run cargo on the daemon.**

**Do NOT:**
- Touch any file under `crates/`, `migrations/`, `docs/`, `services/bridge/tests/`.
- Add `services/bridge` to the workspace `[workspace.members]` array — it must stay in `exclude`.
- Add matrix-sdk, ruma, or any Matrix dep to `services/bridge/Cargo.toml`.
- Commit to `governance-v0` — commit to your task branch only.
- Run any cargo command.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### Task 0 probes — run these first, report results

```bash
# Probe 0 — branch
git branch --show-current
# EXPECT: phase-m2-rooms-a

# Probe 1 — bridge stays workspace-excluded
python -c "import tomllib,sys; d=tomllib.load(open('Cargo.toml','rb')); \
  ex=d['workspace'].get('exclude',[]); me=d['workspace'].get('members',[]); \
  print('EXCLUDE_OK' if any('services/bridge' in e for e in ex) and not any('services/bridge' in m for m in me) else 'EXCLUDE_FAIL')"
# EXPECT: EXCLUDE_OK

# Probe 2 — zero-Matrix-deps invariant
cargo tree --workspace > /tmp/probe2-tree.log 2>&1 && grep -cE 'matrix-sdk|ruma' /tmp/probe2-tree.log
# EXPECT: 0

# Probe 3 — m2-core-hook primitives present on base
grep -c 'pub const ENTRY_KIND_ROOM_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 10
grep -c 'fn append_room_event' crates/api/api/src/governance/governance_log.rs
# EXPECT: 1

# Probe 4 — no /governance/room-event route exists yet
grep -c 'room-event' crates/api/routes/src/lib.rs
# EXPECT: 0

# Probe 5 — bridge has NO rusqlite yet
grep -c 'rusqlite' services/bridge/Cargo.toml
# EXPECT: 0 (grep -c returns exit 1 on no match — that is expected, not a failure)

# Probe 6 — registry total const count
grep -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 65
```

If ANY probe produces an unexpected value, write a `kind: "blocker"` DQ entry explaining which probe failed and what you observed, commit + push, and STOP. Do not proceed to Task 1.

### MIRROR refs for Task 1 — read these EXACT files

**`services/bridge/src/config.rs`** (full file) — `BridgeConfig` struct + `from_env()` function. This is the single construction site for `BridgeConfig` (confirmed: no other `BridgeConfig { ... }` struct literal exists). Your three new fields follow the same pattern: add to the struct, read from environment in `from_env()`.

**`services/bridge/Cargo.toml`** (full file) — existing dependency format to mirror when adding `rusqlite`.

**`services/bridge/src/main.rs`** (full file) — existing `mod` declaration pattern to mirror when adding `mod bridge_room;`.

### Plan sections
`.claude/PRPs/plans/m2-rooms-a.plan.md` §"Task 1: BridgeConfig extension + rusqlite bridge_room store" — the full IMPLEMENT block.

### Lessons (mandatory — R8 key constraint)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ entry + push, then STOP; never run cargo on the daemon.

**R8 WARNING (binding):** `services/bridge/` uses a DIFFERENT toolchain from the Lemmy workspace. Do NOT apply Lemmy-workspace lessons (`feedback_lemmy_error_no_std_error.md`, Diesel lessons, `--features full`, `LemmyResult`, `LemmyError`) to `services/bridge/**`. The bridge uses `anyhow::Result`, axum, matrix-sdk — none of those are present in the binary crates.

## 4. Implementation

### 4.1 `services/bridge/Cargo.toml` — add rusqlite

Under `[dependencies]`, add:
```toml
rusqlite = { version = "0.32", features = ["bundled"] }
```
`bundled` avoids a system libsqlite dependency. Version pin is advisory — if `0.32` fails at the validate-pending-laptop step, the advisor will adjust.

### 4.2 `services/bridge/src/config.rs` — extend BridgeConfig

Add three fields to the `BridgeConfig` struct:
```rust
pub brehon_room_event_url: String,      // URL for POST /governance/room-event (binary callback)
pub bridge_callback_secret: String,     // Bearer secret for bridge<->binary auth (BRIDGE_CALLBACK_SECRET)
pub legal_contact_mxid: String,         // MXID for legal contact in emergency rooms
```

In `from_env()`, read them from environment:
```rust
brehon_room_event_url: std::env::var("BREHON_ROOM_EVENT_URL")
    .unwrap_or_else(|_| "http://localhost:8536/governance/room-event".to_string()),
bridge_callback_secret: std::env::var("BRIDGE_CALLBACK_SECRET")
    .expect("BRIDGE_CALLBACK_SECRET must be set"),
legal_contact_mxid: std::env::var("LEGAL_CONTACT_MXID")
    .unwrap_or_else(|_| "@legal:localhost".to_string()),
```

`brehon_read_url` keeps its existing field name but its `.env` value is what gets repointed at the new GET route (T4b adds the route; T1 only adds the config field — no code change to where `brehon_read_url` is used yet).

GOTCHA: `BridgeConfig` is constructed only in `from_env()`. No cross-crate callers. Adding fields requires updating only `from_env()`.

### 4.3 `services/bridge/src/bridge_room.rs` — NEW: rusqlite store

Create the file with:

```rust
use rusqlite::{Connection, Result, params};

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS bridge_room (
            case_id   INTEGER NOT NULL,
            room_type TEXT    NOT NULL,
            matrix_room_id TEXT,
            lifecycle_state TEXT,
            last_seen_governance_log_row_id INTEGER,
            reveal_state TEXT,
            PRIMARY KEY (case_id, room_type)
        );"
    )?;
    Ok(conn)
}

pub fn lookup(conn: &Connection, case_id: i64, room_type: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT matrix_room_id FROM bridge_room WHERE case_id = ?1 AND room_type = ?2"
    )?;
    let mut rows = stmt.query(params![case_id, room_type])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

pub fn upsert(
    conn: &Connection,
    case_id: i64,
    room_type: &str,
    matrix_room_id: &str,
    lifecycle_state: &str,
    reveal_state: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO bridge_room (case_id, room_type, matrix_room_id, lifecycle_state, reveal_state)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(case_id, room_type) DO UPDATE SET
           matrix_room_id = excluded.matrix_room_id,
           lifecycle_state = excluded.lifecycle_state,
           reveal_state = excluded.reveal_state",
        params![case_id, room_type, matrix_room_id, lifecycle_state, reveal_state],
    )?;
    Ok(())
}

pub fn set_watermark(conn: &Connection, case_id: i64, row_id: i64) -> Result<()> {
    conn.execute(
        "UPDATE bridge_room SET last_seen_governance_log_row_id = ?1 WHERE case_id = ?2",
        params![row_id, case_id],
    )?;
    Ok(())
}
```

GOTCHA (from plan §13 T1): rusqlite is synchronous. `bridge_room` writes are off the provisioning hot path, so direct calls are acceptable here. If a write ever lands on a hot path in a future task, wrap in `tokio::task::spawn_blocking`.

### 4.4 `services/bridge/src/main.rs` — add `mod bridge_room;`

Add `mod bridge_room;` after the existing `mod appservice;` declaration (or wherever the other `mod` declarations are — mirror the existing ordering).

## 5. Validate-pending-laptop entry (write-then-stop, R4)

After implementing Task 1, commit the 4 files on your task branch. Then:

1. Generate a new DQ id: `bash scripts/brehon/dq-v3-new-entry.sh`
2. Write the fragment to `/tmp/dq-t1-frag.json`:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "m2-rooms-a Task 1 validate-pending-laptop: cd services/bridge && cargo check",
  "options": ["pass", "fail"],
  "context": "Task 1 complete: rusqlite dep added to bridge/Cargo.toml, BridgeConfig extended with 3 fields, bridge_room.rs store created, mod bridge_room added to main.rs. Waiting for laptop cargo check.",
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null,
  "commands": ["cd services/bridge && cargo check"],
  "branch": "phase-m2-rooms-a",
  "phase_task": 1,
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```
3. Append: `bash scripts/brehon/dq-v3-append-fragment.sh /tmp/dq-t1-frag.json --pending`
4. `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised validate-pending-laptop for m2-rooms-a task-1" && git push origin <your-branch>`
5. **STOP. Do not run cargo. The laptop advisor runs the commands.**

## 6. Commit message (Task 1)

```
feat(bridge): add rusqlite bridge_room store + extend BridgeConfig (task 1)
```

## 7. DQ blocker protocol

If you hit an unexpected obstacle (probe failure, compilation assumption wrong, import not found), write a `kind: "blocker"` DQ entry via `bash scripts/brehon/dq-v3-append-fragment.sh <frag> --pending`, commit + push on your task branch, and stop. Do not guess.

## HANDOVER

```yaml
HANDOVER:
  task: m2-rooms-a-impl-1
  filesCreated: [services/bridge/src/bridge_room.rs]
  filesModified:
    - services/bridge/Cargo.toml
    - services/bridge/src/config.rs
    - services/bridge/src/main.rs
  keyDecisions:
    - "T0 probes passed (expected — gate 1 approved by user post-plan review)"
    - "rusqlite 0.32 bundled — avoids system libsqlite"
    - "BridgeConfig: +brehon_room_event_url, +bridge_callback_secret, +legal_contact_mxid"
    - "bridge_room table: (case_id, room_type) PK; open/lookup/upsert/set_watermark API"
    - "validate-pending-laptop DQ written; waiting for laptop cargo check"
  nextTask: "T2 impl-task (room_provisioner.rs + appservice.rs wiring) — requires T1 validate pass"
```
