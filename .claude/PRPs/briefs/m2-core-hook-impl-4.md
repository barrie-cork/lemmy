---
phase: m2-core-hook
role: impl-task
n: 4
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: governance-v0
task_number: 4
requires: []
minimax_trial: not-eligible (3 files — over MiniMax 2-file limit)
---

# [role:impl-task] m2-core-hook Task 4 — ENTRY_KIND_ROOM_* consts + shim re-export + registry doc

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-4 ENTRY_KIND_ROOM consts + shim + registry — see .claude/PRPs/briefs/m2-core-hook-impl-4.md
```

## 2. Scope

Add 10 `ENTRY_KIND_ROOM_*` constants to `governance_log.rs`, re-export them in the api shim, and update the registry doc. **Three files, modify-only.** No Cargo.toml change. No migration. **Pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself.**

**Produce (exactly 3 files modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/db_schema/src/source/governance/governance_log.rs   # add 10 ENTRY_KIND_ROOM_* consts
  - crates/api/api/src/governance/governance_log.rs             # add 10 to alphabetical pub use list
  - .claude/rules/governance-log-entry-kind-registry.md         # add M2 room kinds section, bump total 55→65
requires: []   # No dependency — pure const addition, no runtime dep on earlier tasks.
```

**Do NOT:**
- Add any migration — `entry_kind` is TEXT; the consts are compile-time Rust strings.
- Modify `bridge_notify.rs`, `Cargo.toml`, or any handler file.
- Wire `append_room_event` (that is Task 5).
- Run any cargo command.
- Commit to any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT locations on your base branch

**Existing consts (pattern to mirror):** `crates/db_schema/src/source/governance/governance_log.rs` — grep for `ENTRY_KIND_` to find the existing const block. Your 10 consts go AFTER the last existing const in a `// M2 room kinds (10)` block.

**Existing shim:** `crates/api/api/src/governance/governance_log.rs` — read the full file. It contains a `pub use lemmy_db_schema::source::governance::governance_log::{ ... }` list in alphabetical order. Add all 10 `ENTRY_KIND_ROOM_*` names to this list.

**Registry doc:** `.claude/rules/governance-log-entry-kind-registry.md` — read it fully. Find the most recent section, mirror its section heading format, and add a new `## M2 room kinds (10, m2-core-hook)` section. Update the `Acceptance-invariants` total line from 55 → 65.

### Plan sections
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Task 4: ADD the 10 ENTRY_KIND_ROOM_* consts" (ACTION / IMPLEMENT / VALIDATE).

### Lessons (mandatory)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ entry + push, then STOP.

## 4. Constraints

### 4.0 The 10 constants (authoritative)

Add this block to `governance_log.rs` after the last existing `ENTRY_KIND_*` const:

```rust
// M2 room kinds (10) — zero-migration; entry_kind is TEXT
pub const ENTRY_KIND_ROOM_CREATED: &str = "room_created";
pub const ENTRY_KIND_ROOM_ARCHIVED: &str = "room_archived";
pub const ENTRY_KIND_ROOM_MEMBER_ADDED: &str = "room_member_added";
pub const ENTRY_KIND_ROOM_MEMBER_REMOVED: &str = "room_member_removed";
pub const ENTRY_KIND_ROOM_RECORDING_UPLOADED: &str = "room_recording_uploaded";
pub const ENTRY_KIND_ROOM_TRANSCRIPT_READY: &str = "room_transcript_ready";
pub const ENTRY_KIND_ROOM_IDENTITY_REVEALED: &str = "room_identity_revealed";
pub const ENTRY_KIND_ROOM_DECISION_RELAYED: &str = "room_decision_relayed";
pub const ENTRY_KIND_ROOM_BRIDGE_ERROR: &str = "room_bridge_error";
pub const ENTRY_KIND_ROOM_LIFECYCLE_EVENT: &str = "room_lifecycle_event";
```

### 4.1 Shim re-export (alphabetical)

In `crates/api/api/src/governance/governance_log.rs`, add all 10 names to the existing `pub use` block. The block is alphabetical — insert them in the correct alphabetical position relative to existing names. Example: `ENTRY_KIND_REPORT_CREATED` < `ENTRY_KIND_ROOM_*` < `ENTRY_KIND_SANCTION_*` (if any).

### 4.2 Registry doc update

In `.claude/rules/governance-log-entry-kind-registry.md`:
1. Add a new section after the last existing entry-kind section:
   ```markdown
   ## M2 room kinds (10, m2-core-hook)

   | Const | Value | Emitter | Emitted when |
   |---|---|---|---|
   | `ENTRY_KIND_ROOM_CREATED` | `"room_created"` | `append_room_event` (pending bridge-side) | Bridge provisions a new Matrix room for a case |
   | `ENTRY_KIND_ROOM_ARCHIVED` | `"room_archived"` | `append_room_event` (pending bridge-side) | Bridge archives the room after case close |
   | `ENTRY_KIND_ROOM_MEMBER_ADDED` | `"room_member_added"` | `append_room_event` (pending bridge-side) | Juror accepted + added to room |
   | `ENTRY_KIND_ROOM_MEMBER_REMOVED` | `"room_member_removed"` | `append_room_event` (pending bridge-side) | Juror removed on recusal/timeout |
   | `ENTRY_KIND_ROOM_RECORDING_UPLOADED` | `"room_recording_uploaded"` | `append_room_event` (pending bridge-side) | Deliberation recording stored |
   | `ENTRY_KIND_ROOM_TRANSCRIPT_READY` | `"room_transcript_ready"` | `append_room_event` (pending bridge-side) | Transcript attached |
   | `ENTRY_KIND_ROOM_IDENTITY_REVEALED` | `"room_identity_revealed"` | `append_room_event` (pending bridge-side) | OQ-009 graduated-reveal threshold met (M3) |
   | `ENTRY_KIND_ROOM_DECISION_RELAYED` | `"room_decision_relayed"` | `append_room_event` (pending bridge-side) | Verdict relayed back to bridge |
   | `ENTRY_KIND_ROOM_BRIDGE_ERROR` | `"room_bridge_error"` | `append_room_event` (pending bridge-side) | Transient bridge error logged on-chain |
   | `ENTRY_KIND_ROOM_LIFECYCLE_EVENT` | `"room_lifecycle_event"` | `append_room_event` (pending bridge-side) | Generic room lifecycle event |
   ```
2. Find the Acceptance-invariants line that reads `total = 55` (or similar) and update it to `total = 65`.

### 4.3 validate-pending-laptop DQ entry

After writing the three files and committing, append a `validate-pending-laptop` DQ entry via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 4,
  "commands": [
    "./scripts/brehon/cargo-check.sh -p lemmy_db_schema --features full",
    "./scripts/brehon/cargo-check.sh -p lemmy_api --features full"
  ],
  "question": "Workspace check for 10 ENTRY_KIND_ROOM_* consts in governance_log.rs + shim re-export — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 4 added 10 ENTRY_KIND_ROOM_* string consts to crates/db_schema/.../governance_log.rs (M2 room kinds block) and re-exported all 10 from crates/api/api/.../governance_log.rs shim. Also updated .claude/rules/governance-log-entry-kind-registry.md with M2 section + bumped total 55→65. No migration. No Cargo.toml change. Laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null
}
```

### 4.4 Commit + push discipline

- **Commit subject:** `feat(db_schema): add 10 ENTRY_KIND_ROOM_* consts + shim re-export + registry (task 4)`
- Sequence:
  ```
  git add crates/db_schema/src/source/governance/governance_log.rs \
          crates/api/api/src/governance/governance_log.rs \
          .claude/rules/governance-log-entry-kind-registry.md \
          .claude/decision-queue.json
  git commit -m "feat(db_schema): add 10 ENTRY_KIND_ROOM_* consts + shim re-export + registry (task 4)"
  git push origin <your worktree branch>
  ```
- Then **STOP**. Do not run cargo.

### 4.5 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null`.
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓

## HANDOVER

```yaml
HANDOVER:
  task: m2-core-hook-task-4
  filesCreated: []
  filesModified:
    - crates/db_schema/src/source/governance/governance_log.rs
    - crates/api/api/src/governance/governance_log.rs
    - .claude/rules/governance-log-entry-kind-registry.md
    - .claude/decision-queue.json
  keyDecisions:
    - "Added 10 ENTRY_KIND_ROOM_* consts in M2 block after existing ENTRY_KIND_* consts"
    - "All 10 added to alphabetical pub use list in api shim"
    - "Registry doc: new M2 room kinds section + total bumped 55→65"
    - "No migration needed — entry_kind is TEXT"
  notes: "Task 5 uses these consts in append_room_event's allowlist. Task 4 is independent of Tasks 3/6/7 — it can run in parallel."
```
