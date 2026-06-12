---
phase: m2-late-2
role: impl-task
n: 3
authored: 2026-06-12
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-late-2
task_number: 3
parallel_group: cohort-1
---

# [role:impl-task] m2-late-2 Task 3 — bridge case_id mirror + lookup_by_case

## 1. Role + dispatch line

```
[role:impl-task] m2-late-2 task-3 bridge case_id mirror — see .claude/PRPs/briefs/m2-late-2-impl-task-3.md
```

## 2. Scope

Add `case_id: i64` to the bridge-local `SanctionEventPayload` mirror so it deserialises
correctly from the workspace payload, and add `lookup_by_case` to `bridge_room.rs` so Task 4
can look up rooms by case.

### IMPLEMENT

| File | Change |
|---|---|
| `services/bridge/src/sanction_handler.rs` | Add `pub case_id: i64` field to the local `SanctionEventPayload` struct (the `#[derive(Deserialize)]` mirror). |
| `services/bridge/src/bridge_room.rs` | Add `lookup_by_case` function per Pattern 10.2. |

### Explicit out-of-scope

- Do NOT touch `crates/` — workspace changes are Task 1.
- Do NOT write the power-level enforcement logic — that is Task 4.
- Do NOT add `services/bridge` to `workspace.members` or use `--workspace` for bridge cargo
  checks — R9 hard rule.
- Do NOT call `cargo check` or any cargo command yourself — write the validate-pending-laptop
  DQ and STOP (NO-CARGO-ON-ELITEDESK).

## 3. Implementation pattern (verbatim from plan §10)

### Pattern — bridge SanctionEventPayload mirror field

In `services/bridge/src/sanction_handler.rs`, locate the local `SanctionEventPayload` struct
(it has `#[derive(Deserialize)]` and a `sanction_kind: String` field, currently around
line 59). Add immediately after `sanction_kind: String`:

```rust
pub case_id: i64,
```

This is a Serde `Deserialize`-only mirror — no `Serialize` needed. The field name must
match the workspace payload field exactly (`case_id`) for JSON deserialisation to work.

### Pattern 10.2 — lookup_by_case

In `services/bridge/src/bridge_room.rs`, add after the existing `lookup` function
(currently lines 19–29):

```rust
pub fn lookup_by_case(conn: &Connection, case_id: i64) -> Result<Vec<(String, String)>> {
    let mut stmt = conn.prepare(
        "SELECT room_type, matrix_room_id FROM bridge_room WHERE case_id = ?1 AND matrix_room_id IS NOT NULL"
    )?;
    let rows = stmt.query_map(params![case_id], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;
    rows.collect()
}
```

`Connection` is the rusqlite connection type already imported in this file. `Result` is
`rusqlite::Result`. `params!` is `rusqlite::params!`. Mirror the same import style as the
existing `lookup` function immediately above.

**GOTCHA:** `case_id` column in `bridge_room` is `INTEGER` (i64 in Rust). The `?1` bind
parameter in SQLite accepts `i64` natively via `rusqlite::ToSql`.

## 4. Validation gate (validate-pending-laptop DQ — write then STOP)

After committing the change, write a `kind: "validate-pending-laptop"` DQ entry with:

```json
{
  "commands": [
    "bash -c 'cd services/bridge && cargo check'",
    "bash -c 'cd services/bridge && cargo clippy --no-deps -- -D warnings'"
  ],
  "branch": "<current-worktree-branch>",
  "phase_task": 3,
  "e2e_filter": null
}
```

**IMPORTANT — advisor will run these as Linux docker commands:** The laptop advisor will
convert these to `scripts/brehon/cargo-linux.sh check --manifest-path services/bridge/Cargo.toml`
(via Docker `rust:1.95`) because bridge crates do NOT compile on Windows (ruma-common E0119).
Write the commands exactly as above — the advisor handles the platform translation.

Commit the DQ entry + push the branch, then **STOP**.

Generate the DQ id via `bash scripts/brehon/dq-v3-new-entry.sh`.
Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`.

## 5. Required reading

- `.claude/PRPs/plans/m2-late-2.plan.md` — §13 Task 3 spec, Pattern 10.2 (`lookup_by_case`),
  §15.4 LINUX-BRIDGE RIDER (bridge cargo is Docker/Linux-only — do not attempt cargo locally)
- `project_laptop_canonical_cargo_runner.md` (PMD) — NO-CARGO-ON-ELITEDESK; the worker runs
  no cargo; write DQ and stop
- `.claude/lessons/feedback_bridge_validates_on_linux_not_windows.md` — bridge is Linux-only;
  cargo cannot run on Windows for bridge crates; this applies to the advisor's pre-launch
  sanity too (Docker), not the worker

## 6. Constraints

- **R9 (CRITICAL):** `services/bridge` stays in root `Cargo.toml` `exclude` array. NEVER use
  `--workspace` or `--manifest-path` that includes bridge in a workspace cargo command. Always
  use `cargo check --manifest-path services/bridge/Cargo.toml` (or `cd services/bridge &&
  cargo check`) scoped to bridge only.
- **NO cargo on EliteDesk (NO-CARGO-ON-ELITEDESK)** — write the validate-pending-laptop DQ
  and STOP. The daemon has no cargo on PATH, no `cmd.exe`, and no vcpkg. Any cargo invocation
  will fail with `command not found` or a missing libpq error.
- **Serde mirror only** — `SanctionEventPayload` in `sanction_handler.rs` is
  `#[derive(Deserialize)]` only, no `Serialize`. Don't add Serialize.
- **No new DB tables or migrations** — `bridge_room` table already has `case_id` column from
  m2-late-1. `lookup_by_case` is a pure read.
- **DQ v3 id** — generate via `bash scripts/brehon/dq-v3-new-entry.sh`.
- **Mid-task push discipline** — commit DQ entry + push immediately after writing it before
  stopping.
- **Commit prefix** — `feat(bridge): ` subject, with `LESSON:` trailer if any non-obvious
  finding arises.

### Mandatory lesson fires (file-class table check)

| File class | Match? | Lesson injected |
|---|---|---|
| `services/bridge/` edits | ✅ | R9 constraint (§6 above); `feedback_bridge_validates_on_linux_not_windows.md` (§5) |
| `crates/server/tests/e2e.rs` edits | ❌ no e2e edits | — |
| Any new test returning `Result<(), Box<dyn Error>>` | ❌ no new tests | — |
