---
phase: m2-core-hook
role: impl-task
n: 2
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-core-hook
task_number: 2
requires: 1
minimax_trial: eligible (Task 2 — control=Sonnet, trial=MiniMax-M2.7; both arms read THIS brief)
---

# [role:impl-task] m2-core-hook Task 2 — refactor bridge_notify.rs PM path onto tagged union

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-2 bridge_notify PM refactor onto union — see .claude/PRPs/briefs/m2-core-hook-impl-2.md
```

> **A/B note (does not change your work):** this task is dispatched as two parallel arms off the SAME base — a control arm (Sonnet) and a trial arm (MiniMax-M2.7). Each arm gets its own worktree branch and runs this brief independently. You do exactly what this brief says; do not reference or coordinate with the other arm.

## 2. Scope

Refactor the inline `Payload` struct in `notify_if_enabled` (in `crates/api/api_utils/src/bridge_notify.rs`) to use the new `BridgeNotifyPayload::PrivateMessage(PrivateMessagePayload)` tagged union from Task 1. **Two files, modify-only.**  **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself.**

**Produce (exactly 2 files modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api_utils/src/bridge_notify.rs     # replace inline Payload with BridgeNotifyPayload::PrivateMessage
  - crates/api/api_utils/Cargo.toml               # add lemmy_api_common = { workspace = true } to [dependencies]
requires: [1]   # BridgeNotifyPayload must exist in api_common before this task.
```

**Do NOT:**
- Add `governance_case_after_transition` (that is Task 3 — the new hook fn lives in a sibling task).
- Modify any file other than `bridge_notify.rs` and `api_utils/Cargo.toml`.
- Change the external observable behaviour of `notify_if_enabled` — the POST still fires to `BRIDGE_NOTIFY_URL` with the SAME JSON body (just now emitted by the tagged enum's `type_: "private_message"` variant).
- Add `serde_json` to `api_utils/Cargo.toml` — the typed `BridgeNotifyPayload` serialises via `reqwest`'s `.json()` which only needs `Serialize` (already present). No `serde_json::Value` construction needed.
- Run any cargo command. Validation is the laptop's job (write-then-stop).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT line ranges on your base branch

**The fn to modify:** `crates/api/api_utils/src/bridge_notify.rs:1-49` (full file; it is short ~49 lines) — the `notify_if_enabled` fn with the inline `#[derive(serde::Serialize)] struct Payload { private_message_id, creator_id, recipient_id }`. Your task is to remove that inline struct and instead build `BridgeNotifyPayload::PrivateMessage(PrivateMessagePayload { … })`.

**The DTO you import:** `crates/api/api_common/src/governance.rs` — grep for `PrivateMessagePayload` and `BridgeNotifyPayload` to find the exact lines Task 1 added (they are at the bottom of the file). Confirm the import path is `lemmy_api_common::governance::{BridgeNotifyPayload, PrivateMessagePayload}`.

**Workspace dep table:** `crates/api/api_utils/Cargo.toml` — the `[dependencies]` section. Confirm `lemmy_api_common` is ABSENT (you must add it). Check the workspace root `Cargo.toml` to confirm `lemmy_api_common` is already a workspace dep (so you just write `lemmy_api_common = { workspace = true }`).

**Workspace root dep entry:** `Cargo.toml` (root) — grep `lemmy_api_common` to confirm the workspace alias. Mirror the exact form used for `lemmy_db_schema = { workspace = true }` in `api_utils/Cargo.toml`.

### Plan sections
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Step-by-Step Tasks" Task 2 (ACTION / IMPLEMENT / MIRROR / GOTCHA / VALIDATE).
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Patterns to Mirror" → `FIRE_AND_FORGET_CONFIG_GATE`.

### Lessons (mandatory)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then stop. (MANDATORY — validate-pending-laptop.)

## 4. Constraints

### 4.0 What the refactor looks like (authoritative)

**BEFORE** (in `notify_if_enabled`, after the `if !enabled { return Ok(()); }` gate):

```rust
  #[derive(serde::Serialize)]
  struct Payload {
    private_message_id: i32,
    creator_id: i32,
    recipient_id: i32,
  }
  let payload = Payload {
    private_message_id: view.private_message.id.0,
    creator_id: view.creator.id.0,
    recipient_id: view.recipient.id.0,
  };
  if let Err(e) = context
    .client()
    .post(BRIDGE_NOTIFY_URL)
    .json(&payload)
    .send()
    .await
  {
    tracing::warn!("bridge notify failed (bridge may be down — non-fatal): {e}");
  }
```

**AFTER** (same location, inline struct replaced with the tagged enum):

```rust
  let payload = BridgeNotifyPayload::PrivateMessage(PrivateMessagePayload {
    private_message_id: view.private_message.id.0,
    creator_id: view.creator.id.0,
    recipient_id: view.recipient.id.0,
  });
  if let Err(e) = context
    .client()
    .post(BRIDGE_NOTIFY_URL)
    .json(&payload)
    .send()
    .await
  {
    tracing::warn!("bridge notify failed (bridge may be down — non-fatal): {e}");
  }
```

Add the import at the top of `bridge_notify.rs`:
```rust
use lemmy_api_common::governance::{BridgeNotifyPayload, PrivateMessagePayload};
```

The `#[derive(serde::Serialize)] struct Payload { … }` block is **deleted entirely** (replaced by the import + construction above).

### 4.1 Wire-format change (acceptable — document in commit body)

The JSON body changes from `{"private_message_id":…, "creator_id":…, "recipient_id":…}` to `{"type_":"private_message","private_message_id":…, "creator_id":…, "recipient_id":…}`. This is intentional — the M2 bridge (not yet built) expects the tagged union; M1 bridge (localhost:9009, never ran in prod) was a stub. Document in the commit body: `WIRE_FORMAT: PM payload now includes type_:"private_message" discriminator tag`.

### 4.2 Cargo.toml change

In `crates/api/api_utils/Cargo.toml`, under `[dependencies]`, add exactly:
```toml
lemmy_api_common = { workspace = true }
```
Insert it in the existing alphabetical-ish grouping near other `lemmy_*` deps. No `features` needed — the governance DTOs are unconditionally compiled (no `full` gate on the serde derives).

**Why `api_utils` can depend on `api_common`:** `api_common` does NOT depend on `api_utils` (confirmed from `crates/api/api_common/Cargo.toml`). No circular dep.

### 4.3 validate-pending-laptop DQ entry

After writing both files and committing, append a `validate-pending-laptop` DQ entry. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh` (or append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 2,
  "commands": [
    "./scripts/brehon/cargo-check.sh -p lemmy_api_utils --features full"
  ],
  "question": "Workspace check for bridge_notify.rs PM refactor onto tagged union — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 2 replaced inline Payload struct in notify_if_enabled with BridgeNotifyPayload::PrivateMessage from api_common. Added lemmy_api_common dep to api_utils/Cargo.toml. Wire format gains type_:private_message discriminator. cargo check -p lemmy_api_utils --features full. Laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

> Command matches the plan §Task 2 `VALIDATE` line. Scoped to `lemmy_api_utils` (direct change) — full workspace is covered by Task 3's check.

### 4.4 Commit + push discipline

- **Commit subject:** `feat(api_utils): refactor bridge_notify PM path onto tagged union (task 2)`
- Commit body should include the `WIRE_FORMAT:` line from §4.1.
- Sequence:
  ```
  git add crates/api/api_utils/src/bridge_notify.rs crates/api/api_utils/Cargo.toml .claude/decision-queue.json
  git commit -m "feat(api_utils): refactor bridge_notify PM path onto tagged union (task 2)"
  git push origin <your worktree branch>
  ```
  (Or split into two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m2-core-hook task 2`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo.

### 4.5 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓
- No e2e / migration / handler-2-write / cfg-full-gate file-class match.

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m2-core-hook-task-2
  filesCreated: []
  filesModified:
    - crates/api/api_utils/src/bridge_notify.rs
    - crates/api/api_utils/Cargo.toml
    - .claude/decision-queue.json
  keyDecisions:
    - "Replaced inline Payload struct in notify_if_enabled with BridgeNotifyPayload::PrivateMessage(PrivateMessagePayload{…})"
    - "Added lemmy_api_common = { workspace = true } to api_utils/Cargo.toml (no circular dep: api_common doesn't depend on api_utils)"
    - "Wire format change: JSON gains type_:private_message discriminator tag — intentional, documented in commit body"
    - "Behaviour of notify_if_enabled unchanged: same config gate, same POST URL, same error-swallow, same warn! log"
  notes: "Task 3 will ADD governance_case_after_transition as a new sibling fn in the same file. Task 2 only refactors the existing PM path."
```
