---
phase: m2-core-hook
role: impl-task
n: 5
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: governance-v0
task_number: 5
requires: [4]
minimax_trial: not-eligible (single-file, minimal logic — below qualifying threshold)
---

# [role:impl-task] m2-core-hook Task 5 — add append_room_event typed wrapper

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-5 add append_room_event wrapper — see .claude/PRPs/briefs/m2-core-hook-impl-5.md
```

## 2. Scope

Add `append_room_event` as a typed, kind-gated wrapper in `crates/api/api/src/governance/governance_log.rs`. **One file, modify-only.** **Pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself.**

**Produce (exactly 1 file modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/governance_log.rs
requires: [4]   # ENTRY_KIND_ROOM_* consts + shim re-export must exist (Task 4)
```

**Do NOT:**
- Modify `Cargo.toml` or any other file.
- Add a new `governance_log.rs` — only edit the existing one in `crates/api/api/src/governance/`.
- Call the `append` fn from `db_schema` directly (use the shim re-export already in this file).
- Commit to `governance-v0` — your worktree will branch from `governance-v0`; commit to your task branch only.
- Run any cargo command.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT files on your base branch (`governance-v0`)

**The file to edit:** `crates/api/api/src/governance/governance_log.rs`
- Read the full file. Note: it already `pub use`-re-exports `append` and all 10 `ENTRY_KIND_ROOM_*` consts from the `db_schema` shim. Your wrapper calls `append(pool, kind, value, actor_pseudonym)` via that re-export — one import path for all callers.
- The `append` fn signature (for reference; it is re-exported, not defined here):
  ```rust
  pub async fn append(
    pool: &mut DbPool<'_>,
    entry_kind: &str,
    payload: Value,
    actor_pseudonym: Option<String>,
  ) -> LemmyResult<GovernanceLog>
  ```
- The file is `#[cfg(feature = "full")]`-gated at module level or per-fn level — confirm and mirror the pattern for your new fn.

**The consts you need:** `crates/db_schema/src/source/governance/governance_log.rs` lines ~239–248 — the 10 `ENTRY_KIND_ROOM_*` consts. They are already re-exported by the shim in the file you're editing; no new import needed.

**Scrub-json behaviour:** `crates/db_schema/src/source/governance/governance_log.rs` — read the `scrub_json` call inside `append` (around line 282). The `matrix_room_id` field (`!room:server` form) may match the URL/mention regex and get redacted. To avoid this, document in a code comment that callers should pass `matrix_room_id` as an opaque non-`!`-prefixed token if redaction is unacceptable, OR strip the `!` prefix before storing. Do NOT change `scrub_json` itself.

**Pattern for kind-allowlist:** scan the file for any existing `const … &[&str]` or match-on-const pattern to mirror the style. If none exists, use:
```rust
const ROOM_KINDS: &[&str] = &[
  ENTRY_KIND_ROOM_CREATED,
  ENTRY_KIND_ROOM_ARCHIVED,
  // ... all 10
];
```

### Plan section
`.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Task 5: ADD the `append_room_event(...)` typed wrapper".

### Lessons (mandatory)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ entry + push, then STOP.

## 4. Implementation

### 4.1 What to add

Add two items in `crates/api/api/src/governance/governance_log.rs`, after the existing `pub use` block:

**A) `RoomEventPayload` struct:**

```rust
/// Payload for room-lifecycle governance log entries written by the M2 bridge.
///
/// All string fields pass through `scrub_json` inside `append` — store
/// `matrix_room_id` as an opaque non-URL token (strip the leading `!`) to
/// avoid the URL-regex redaction path.
#[derive(serde::Serialize)]
pub struct RoomEventPayload {
  pub case_id: i32,
  pub matrix_room_id: Option<String>,
  pub lifecycle_stage: String,
  pub member_count: Option<i32>,
}
```

**B) `append_room_event` fn:**

```rust
/// Typed, kind-gated wrapper around [`append`] for M2 room-lifecycle entries.
///
/// Returns `Err(LemmyErrorType::Unknown("not a room entry kind: …"))` if
/// `kind` is not one of the 10 `ENTRY_KIND_ROOM_*` consts — enforcing the
/// ADR-008 integrity gate so the bridge cannot write arbitrary entry kinds.
#[cfg(feature = "full")]
pub async fn append_room_event(
  pool: &mut DbPool<'_>,
  kind: &str,
  payload: RoomEventPayload,
  actor_pseudonym: Option<String>,
) -> LemmyResult<GovernanceLog> {
  const ROOM_KINDS: &[&str] = &[
    ENTRY_KIND_ROOM_CREATED,
    ENTRY_KIND_ROOM_ARCHIVED,
    ENTRY_KIND_ROOM_MEMBER_ADDED,
    ENTRY_KIND_ROOM_MEMBER_REMOVED,
    ENTRY_KIND_ROOM_RECORDING_UPLOADED,
    ENTRY_KIND_ROOM_TRANSCRIPT_READY,
    ENTRY_KIND_ROOM_IDENTITY_REVEALED,
    ENTRY_KIND_ROOM_DECISION_RELAYED,
    ENTRY_KIND_ROOM_BRIDGE_ERROR,
    ENTRY_KIND_ROOM_LIFECYCLE_EVENT,
  ];
  if !ROOM_KINDS.contains(&kind) {
    return Err(LemmyError::from(LemmyErrorType::Unknown(
      format!("not a room entry kind: {kind}"),
    )));
  }
  let value = serde_json::to_value(&payload)
    .map_err(|e| LemmyError::from(LemmyErrorType::Unknown(e.to_string())))?;
  append(pool, kind, value, actor_pseudonym).await
}
```

**Imports to add** (only if not already present in the file):
- `use lemmy_utils::error::{LemmyError, LemmyErrorType};` — check if these are already imported; add only what's missing.
- `use serde_json::Value;` — check if already imported.
- `use lemmy_db_schema::utils::DbPool;` — may already be re-exported via the existing shim block; confirm.
- `use lemmy_db_schema::source::governance::governance_log::GovernanceLog;` — already in the `pub use` block; no new import.

Read the existing imports at the top of the file and add only what is genuinely missing. Do NOT duplicate existing imports.

### 4.2 validate-pending-laptop DQ entry

After writing the file and committing, append a `validate-pending-laptop` DQ entry via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 5,
  "commands": [
    "./scripts/brehon/cargo-check.sh -p lemmy_api --features full"
  ],
  "question": "cargo check for append_room_event wrapper + RoomEventPayload struct in governance_log.rs — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 5 added append_room_event (kind-allowlist gate + serde_json::to_value + calls append re-export) and RoomEventPayload struct in crates/api/api/src/governance/governance_log.rs. Only crate affected: lemmy_api. Laptop validates.",
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

### 4.3 Commit + push discipline

- **Commit subject:** `feat(api): add append_room_event typed wrapper + RoomEventPayload (task 5)`
- Stage exactly:
  ```
  git add crates/api/api/src/governance/governance_log.rs \
          .claude/decision-queue.json
  git commit -m "feat(api): add append_room_event typed wrapper + RoomEventPayload (task 5)"
  git push origin <your worktree branch>
  ```
- Then **STOP**. Do not run cargo.

### 4.4 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null`.
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓

## HANDOVER

```yaml
HANDOVER:
  task: m2-core-hook-task-5
  filesCreated: []
  filesModified:
    - crates/api/api/src/governance/governance_log.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "RoomEventPayload: case_id/matrix_room_id/lifecycle_stage/member_count — no raw usernames/emails"
    - "ROOM_KINDS allowlist as const &[&str] inside fn body"
    - "Kind-gate error: LemmyErrorType::Unknown(format!(not a room entry kind: {kind}))"
    - "Calls append re-export (already in pub use block) — no db_schema direct import"
    - "matrix_room_id scrub-json note: strip ! prefix to avoid URL-regex redaction"
  notes: "Task 8 e2e tests assert append_room_event round-trips. Task 5 base is governance-v0 (not phase-m2-core-hook) — advisor will forward-merge gov-v0 into phase branch before Task 8."
```
