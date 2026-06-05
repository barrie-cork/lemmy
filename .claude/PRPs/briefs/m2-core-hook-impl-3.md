---
phase: m2-core-hook
role: impl-task
n: 3
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-core-hook
task_number: 3
requires: 2
minimax_trial: eligible (Task 3 — control=Sonnet, trial=MiniMax-M2.7; both arms read THIS brief)
---

# [role:impl-task] m2-core-hook Task 3 — add governance_case_after_transition to bridge_notify.rs

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-3 governance_case_after_transition fn — see .claude/PRPs/briefs/m2-core-hook-impl-3.md
```

> **A/B note (does not change your work):** this task is dispatched as two parallel arms off the SAME base — a control arm (Sonnet) and a trial arm (MiniMax-M2.7). Each arm gets its own worktree branch and runs this brief independently. You do exactly what this brief says; do not reference or coordinate with the other arm.

## 2. Scope

Add `pub async fn governance_case_after_transition(...)` as a new sibling function in `crates/api/api_utils/src/bridge_notify.rs`. **One file, modify-only.** **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself.**

**Produce (exactly 1 file modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api_utils/src/bridge_notify.rs     # add governance_case_after_transition fn + imports
requires: [2]   # Task 2 refactored notify_if_enabled; this task adds the sibling fn in the same file.
```

**Do NOT:**
- Modify `Cargo.toml` — `lemmy_api_common` was already added in Task 2; `lemmy_db_schema` is already present.
- Wire the hook at any call site (Tasks 6 and 7 do that).
- Rename or change `notify_if_enabled` in any way.
- Create any new file.
- Run any cargo command.
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT locations on your base branch

**The sibling fn to mirror:** `crates/api/api_utils/src/bridge_notify.rs:1-49` (the full file as it exists after Task 2 — `notify_if_enabled` with the `BridgeNotifyPayload::PrivateMessage` construction). Your new fn mirrors its EXACT structure: same config gate, same fire-and-forget `.ok()` POST, same `tracing::warn!`, same return type.

**The imports at the top of bridge_notify.rs:** note which types are already imported. You will add `ModerationCase` and `CaseStatus` to the existing use lines (or add new use lines) — see §4.1.

**The CaseTransitionEvent type:** `crates/api/api_common/src/governance.rs` — grep for `CaseTransitionEvent` to confirm its fields: `case_id: i32`, `old_status: Option<CaseStatus>`, `new_status: CaseStatus`, `community_id: Option<i32>`, `target_type: String`. Your fn builds this struct.

**The ModerationCase model:** `crates/db_schema/src/source/governance/moderation_case.rs` — confirm the field names: `id` (ModerationCaseId, use `.0` for i32), `community_id` (Option<CommunityId>, use `.map(|c| c.0)`), `target_type` (has a `to_string()` or `as_str()` — check).

### Plan sections
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Step-by-Step Tasks" Task 3 (ACTION / IMPLEMENT / MIRROR / GOTCHA / VALIDATE).

### Lessons (mandatory)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then STOP. (MANDATORY.)

## 4. Constraints

### 4.0 The new function (authoritative shape)

Add this function **after** the closing `}` of `notify_if_enabled`:

```rust
/// Fire-and-forget notify to the Matrix bridge when a moderation case transitions status.
/// Reads `messaging_enabled`; false → no-op. True → POST CaseTransition event to bridge.
/// NEVER fails the governance path — transport errors are swallowed (plan §10.5, ADR-012).
pub async fn governance_case_after_transition(
  context: &LemmyContext,
  case: &ModerationCase,
  old_status: Option<CaseStatus>,
  new_status: CaseStatus,
) -> LemmyResult<()> {
  let pool = &mut context.pool();
  let enabled = match GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled")
    .await?
  {
    Some(row) => row.value_bool.unwrap_or(false),
    None => false,
  };
  if !enabled {
    return Ok(());
  }
  let payload = BridgeNotifyPayload::CaseTransition(CaseTransitionEvent {
    case_id: case.id.0,
    old_status,
    new_status,
    community_id: case.community_id.map(|c| c.0),
    target_type: case.target_type.to_string(),
  });
  if let Err(e) = context
    .client()
    .post(BRIDGE_NOTIFY_URL)
    .json(&payload)
    .send()
    .await
  {
    tracing::warn!("bridge notify (case transition) failed (bridge may be down — non-fatal): {e}");
  }
  Ok(())
}
```

**Verify before committing:**
- `case.target_type.to_string()` compiles — check whether `target_type` is a `String` (then `.clone()` or just move) or an enum/custom type (then `.to_string()` via `Display`). Read `ModerationCase` at the MIRROR ref above before writing. If it is already `String`, use `.clone()` (the `case` borrow is `&`).
- `case.community_id.map(|c| c.0)` — `CommunityId` is a newtype wrapping `i32`, so `.0` gives the inner value.
- `case.id.0` — `ModerationCaseId` is a newtype wrapping `i32`.

### 4.1 Imports to add

The existing file already imports `BridgeNotifyPayload` and `PrivateMessagePayload` from `lemmy_api_common::governance`. Add `CaseTransitionEvent` to that same use line:

```rust
use lemmy_api_common::governance::{BridgeNotifyPayload, CaseTransitionEvent, PrivateMessagePayload};
```

Add new use lines for the db_schema types (api_utils already depends on lemmy_db_schema — no Cargo.toml change needed):

```rust
use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
use lemmy_db_schema_file::enums::CaseStatus;
```

**Check the existing import structure first** (read the file on your base branch) — `lemmy_db_schema_file` may already be imported via another use line; if `CaseStatus` is not yet imported, add it. If `lemmy_db_schema` is already used for other imports, mirror the existing import path pattern.

### 4.2 validate-pending-laptop DQ entry

After writing the file and committing, append a `validate-pending-laptop` DQ entry. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh` (or use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 3,
  "commands": [
    "./scripts/brehon/cargo-check.sh -p lemmy_api_utils --features full"
  ],
  "question": "Workspace check for governance_case_after_transition fn in bridge_notify.rs — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 3 added governance_case_after_transition to bridge_notify.rs: same config gate as notify_if_enabled, builds BridgeNotifyPayload::CaseTransition(CaseTransitionEvent{...}) from ModerationCase fields. Added CaseTransitionEvent to existing api_common import; added ModerationCase + CaseStatus imports from lemmy_db_schema/lemmy_db_schema_file. No Cargo.toml change. cargo check -p lemmy_api_utils --features full. Laptop only.",
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

- **Commit subject:** `feat(api_utils): add governance_case_after_transition hook fn (task 3)`
- Sequence:
  ```
  git add crates/api/api_utils/src/bridge_notify.rs .claude/decision-queue.json
  git commit -m "feat(api_utils): add governance_case_after_transition hook fn (task 3)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then the `chore(decision-queue):` commit.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and the `HANDOVER:` YAML trailer below.
- Then **STOP**. Do not run cargo.

### 4.4 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null`.
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓
- No e2e / migration / handler-2-write / cfg-full-gate file-class match.

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m2-core-hook-task-3
  filesCreated: []
  filesModified:
    - crates/api/api_utils/src/bridge_notify.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "Added governance_case_after_transition: same config gate + fire-and-forget POST as notify_if_enabled"
    - "Builds BridgeNotifyPayload::CaseTransition(CaseTransitionEvent{case_id, old_status, new_status, community_id, target_type})"
    - "Imports added: CaseTransitionEvent from api_common; ModerationCase + CaseStatus from db_schema"
    - "No Cargo.toml change needed — api_utils already depends on lemmy_api_common (Task 2) + lemmy_db_schema"
  notes: "Tasks 6 + 7 wire this fn at 12 call sites across handlers + cron. Task 3 only defines the fn."
```
