---
role: impl-task
phase: m3-core-infra
task: 2
parallel: true
cohort: [1, 2]
base_branch: phase-m3-core-infra
created: 2026-06-18
---

# Impl-task brief — m3-core-infra Task 2 [P]: `bridge_room` RTC-state columns + idempotent ALTER guards

**Role:** `[role:impl-task]`
**Dispatch:** `[role:impl-task] m3-core-infra task2 bridge-room-rtc-cols — see .claude/PRPs/briefs/m3-core-infra-impl-2.md`
**Base branch:** `phase-m3-core-infra`
**Plan:** `.claude/PRPs/plans/m3-core-infra.plan.md` Task 2 + §10.5

---

## 1. Role + dispatch line

You are the **impl-task** subagent (Sonnet 4.6 — pattern-following from MIRROR refs). Execute plan Task 2, ONE commit, write the `validate-pending-laptop-linux` DQ, then **STOP**.

This is a `[P]` cohort member (cohort = Task 1 ∥ Task 2). Task 1 modifies `migrations/**` + `crates/api/api/src/governance/bridge_read.rs` only — **zero file overlap** with this task. Do not touch any `crates/` or `migrations/` file.

---

## 2. Scope

Extend the embedded SQLite schema in `services/bridge/src/bridge_room.rs::open()` with three nullable RTC-state columns + idempotent ALTER guards for DBs provisioned before M3.

**FILES (exhaustive — touch nothing else):**

```yaml
creates: []
modifies:
  - services/bridge/src/bridge_room.rs   # CREATE TABLE cols + ALTER guards + #[cfg(test)] test
```

**IMPLEMENT** per §10.5 (mirror `services/bridge/src/bridge_room.rs:3-17` verbatim shape):
1. Add three columns to the `CREATE TABLE IF NOT EXISTS bridge_room (...)` block, inside the existing column list (before `PRIMARY KEY`):
   ```
   chair_id        TEXT,   -- current chair pseudonym
   queue_state     TEXT,   -- FIFO raised-hand queue (JSON text)
   recording_config TEXT,  -- recording knobs (JSON text)
   ```
2. After the `execute_batch(...)`, add the idempotent ALTER loop (match-and-swallow SQLite's "duplicate column name", propagate any other error):
   ```rust
   for col in ["chair_id", "queue_state", "recording_config"] {
       let stmt = format!("ALTER TABLE bridge_room ADD COLUMN {col} TEXT");
       match conn.execute(&stmt, []) {
           Ok(_) => {}
           Err(rusqlite::Error::SqliteFailure(_, Some(msg)))
               if msg.contains("duplicate column name") => {}
           Err(e) => return Err(e),
       }
   }
   ```
3. Add a `#[cfg(test)]` test module (or extend the existing one) that:
   - opens a FRESH db → asserts the 3 columns exist via `PRAGMA table_info(bridge_room)`.
   - opens a db pre-seeded with the OLD 6-column shape → asserts `open()` adds the 3 columns AND a second `open()` call is idempotent (no error, no duplicate columns).

**Commit:** `feat(bridge): add bridge_room RTC-state columns + idempotent ALTER guards (task 2)`

**Do NOT:**
- Touch any file under `crates/` or `migrations/` (Task 1's lane).
- Blindly ignore ALL ALTER errors — match ONLY the duplicate-column-name case; propagate everything else.
- Run cargo on the daemon (see §4).

---

## 3. Required reading

1. `.claude/PRPs/plans/m3-core-infra.plan.md` — Task 2 + §10.5 (the MIRROR ref)
2. `services/bridge/src/bridge_room.rs:3-17` — the `open()` fn + CREATE TABLE block to mirror; note the existing 6 columns
3. `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — bridge compiles on Linux only (`cargo-linux.sh --manifest-path`); never `cd services/bridge && cargo`
4. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ, then STOP; do NOT run cargo on the daemon
5. `.claude/rules/decision-queue.md` "Mid-task visibility" — commit+push DQ on the worker branch

### 3a. Handover from prior cohort

(none — first cohort)

---

## 4. Constraints

- **`CREATE TABLE IF NOT EXISTS` does NOT alter an existing table** — the ALTER guards are load-bearing for any bridge DB created before M3. SQLite has no `ADD COLUMN IF NOT EXISTS`; match-and-swallow the duplicate-column error per column.
- **Bridge compiles on LINUX ONLY** (`ruma-common` E0119 on Windows). You do NOT compile it. Write a `validate-pending-laptop-linux` DQ entry with:
  ```
  commands: [
    "./scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml",
    "./scripts/brehon/cargo-linux.sh clippy --manifest-path services/bridge/Cargo.toml --no-deps -- -D warnings",
    "./scripts/brehon/cargo-linux.sh test --manifest-path services/bridge/Cargo.toml bridge_room"
  ]
  ```
  plus `branch: "<your worker branch>"`, `phase_task: 2`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id; append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Commit + push the DQ on your worker branch, then **STOP**. The laptop advisor runs the Docker Linux build + test and mutates the entry. Do NOT run cargo-linux.sh or any cargo yourself.
- **One commit** for the impl. The DQ commit is separate (`chore(decision-queue): impl raised validate-pending-linux — bridge-room-rtc-cols`).
- **DQ mid-task discipline:** if blocked, write a `kind: "blocker"` DQ (`from: "impl"`), commit + push on the worker branch, stop.
- **LESSON trailer:** end the impl commit body with `LESSON: <one-line>` if durable; else omit.

---

## 5. Success signals (advisor verifies)

- `bridge_room.rs` `CREATE TABLE` block has `chair_id` / `queue_state` / `recording_config` TEXT columns.
- The post-create ALTER loop matches ONLY `duplicate column name` and propagates other errors.
- A `#[cfg(test)]` test asserts column presence on fresh AND old-shape DBs + idempotency on second `open()`.
- `validate-pending-laptop-linux` DQ entry pushed on the worker branch (`phase_task: 2`).
- No cargo run on the daemon; no `crates/`/`migrations/` files touched.

---

## HANDOVER

```yaml
HANDOVER:
  task: m3-core-infra-task2
  filesCreated: []
  filesModified: [services/bridge/src/bridge_room.rs]
  keyDecisions:
    - "bridge_room gains chair_id/queue_state/recording_config (nullable TEXT)"
    - "idempotent ALTER loop swallows only SQLite duplicate-column-name; propagates other errors"
    - "test covers fresh-db + old-6-col-shape + second-open idempotency"
  notes: "<fill in: worker branch, validate-pending-laptop-linux DQ id>"
```
