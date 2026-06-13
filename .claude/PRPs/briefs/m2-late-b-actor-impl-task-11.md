---
role: impl-task
task_number: 11
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_bridge_validates_on_linux_not_windows.md
  - feedback_validate_pending_laptop_write_then_stop.md  # validate via cargo-linux.sh
---

# impl-task brief — m2-late-b-actor Task 11: CREATE `services/bridge/src/app_actor_link.rs`

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 11 of 13
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Task 10 merged (route wiring must exist; this task is bridge-only and independent of route compilation).

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-11 bridge-sqlite-cache app-actor-link — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-11.md
```

---

## 2. Scope

**Produce:**
- New file `services/bridge/src/app_actor_link.rs` — SQLite cache for `app_local_id ↔ brehon_actor_id` mapping
- `mod app_actor_link;` declaration in `services/bridge/src/main.rs`
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Touch `crates/**` (Tasks 1-10)
- Touch `services/bridge/src/link_handler.rs` (Task 12)
- Touch `services/bridge/src/appservice.rs`, `config.rs` (Task 12)
- Run cargo yourself — this is bridge code; it validates on Linux via cargo-linux.sh, NOT Windows-local cargo (hard rule)

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 11"
2. `services/bridge/src/bridge_room.rs` — **MIRROR**: verbatim `open/upsert/lookup` shape; copy the rusqlite patterns exactly
3. `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — **MANDATORY**: `services/bridge` does NOT compile on Windows; the validate-pending-laptop entry must use `scripts/brehon/cargo-linux.sh`, not Windows-local cargo
4. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Implementation

**New file: `services/bridge/src/app_actor_link.rs`**

Mirror `bridge_room.rs` exactly. The table has one TEXT PK and TEXT NOT NULL brehon_actor_id, plus two INTEGER timestamp columns:

```rust
use rusqlite::{Connection, Result, params};

pub fn open(path: &str) -> Result<Connection> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS app_actor_link (
            app_local_id   TEXT    PRIMARY KEY,
            brehon_actor_id TEXT   NOT NULL,
            linked_at      INTEGER,
            revoked_at     INTEGER
        );"
    )?;
    Ok(conn)
}

pub fn upsert(conn: &Connection, app_local_id: &str, brehon_actor_id: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO app_actor_link (app_local_id, brehon_actor_id, linked_at)
         VALUES (?1, ?2, strftime('%s', 'now'))
         ON CONFLICT(app_local_id) DO UPDATE SET
           brehon_actor_id = excluded.brehon_actor_id,
           linked_at       = excluded.linked_at,
           revoked_at      = NULL",
        params![app_local_id, brehon_actor_id],
    )?;
    Ok(())
}

pub fn lookup(conn: &Connection, app_local_id: &str) -> Result<Option<String>> {
    let mut stmt = conn.prepare(
        "SELECT brehon_actor_id FROM app_actor_link WHERE app_local_id = ?1 AND revoked_at IS NULL"
    )?;
    let mut rows = stmt.query(params![app_local_id])?;
    if let Some(row) = rows.next()? {
        Ok(row.get(0)?)
    } else {
        Ok(None)
    }
}

pub fn revoke(conn: &Connection, app_local_id: &str) -> Result<()> {
    conn.execute(
        "UPDATE app_actor_link SET revoked_at = strftime('%s', 'now') WHERE app_local_id = ?1",
        params![app_local_id],
    )?;
    Ok(())
}
```

**ADR-015 (load-bearing):** `brehon_actor_id` stored here is the **pseudonym UUID string** (`actor_pseudonym.pseudonym`), NEVER a Lemmy `person_id`, username, or email address. The incoming value comes from `LinkConfirmRequest.brehon_actor_id` (which Brehon echoes from the claim — already pseudonymised at source).

**Wire module in `services/bridge/src/main.rs`:**

Add `mod app_actor_link;` to the module list. Read the current `main.rs` and insert it alphabetically (after `appservice`, before `bridge_room`):

```rust
mod app_actor_link;
mod appservice;
mod bridge_room;
```

**GOTCHA — rusqlite is already present:** Do NOT add it to `Cargo.toml`. The crate already uses it (see `bridge_room.rs`).

**GOTCHA — Linux-only:** `services/bridge` is workspace-excluded and does NOT compile on Windows due to `ruma-common` E0119 vs `time`. You MUST NOT run `cargo` locally. Write the file, commit, push, write the DQ entry, and stop.

### validate-pending-laptop (MANDATORY — Linux variant)

The validate command for bridge code uses `cargo-linux.sh`, not Windows-local cargo.

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": ["scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml"],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 11
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**.

---

## 5. Commit

```
feat(bridge): SQLite cache app_actor_link — open/upsert/lookup/revoke (task 11)
```

Stage only: `services/bridge/src/app_actor_link.rs`, `services/bridge/src/main.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `services/bridge/src/app_actor_link.rs` created with `open`, `upsert`, `lookup`, `revoke` functions
- [ ] `CREATE TABLE IF NOT EXISTS app_actor_link` with `app_local_id TEXT PRIMARY KEY`, `brehon_actor_id TEXT NOT NULL`, `linked_at INTEGER`, `revoked_at INTEGER`
- [ ] `upsert` uses `ON CONFLICT(app_local_id) DO UPDATE SET ... = excluded.*` and resets `revoked_at = NULL`
- [ ] `lookup` filters `revoked_at IS NULL` (live links only)
- [ ] `mod app_actor_link;` present in `main.rs`
- [ ] ADR-015: `brehon_actor_id` parameter is the pseudonym UUID, not a person_id (code comment asserts this)
- [ ] `validate-pending-laptop` DQ committed + pushed (commands: `cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`)

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-11
  branch: phase-m2-late-b-actor
  filesModified:
    - services/bridge/src/app_actor_link.rs
    - services/bridge/src/main.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "revoke() sets revoked_at rather than DELETE (prospective unlink — chain not rewritten)"
    - "lookup() filters revoked_at IS NULL — only live links returned"
    - "validate via cargo-linux.sh (Docker rust:1.95) NOT Windows-local cargo"
    - "brehon_actor_id stored is pseudonym UUID string (ADR-015)"
  notes: "Task 11 of 13. Task 12 (link_handler.rs + wiring) follows."
```
