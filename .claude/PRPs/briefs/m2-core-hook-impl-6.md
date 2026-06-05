---
phase: m2-core-hook
role: impl-task
n: 6
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-core-hook
task_number: 6
requires: [3]
minimax_trial: not-eligible (5 files — over MiniMax 2-file limit)
---

# [role:impl-task] m2-core-hook Task 6 — wire governance_case_after_transition at 8 handler sites

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-6 wire hook at 8 handler sites — see .claude/PRPs/briefs/m2-core-hook-impl-6.md
```

## 2. Scope

Wire `governance_case_after_transition` at the 8 handler-side CaseStatus transition commit points (sites 1–8). **Five files, modify-only.** **Pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself.**

**Produce (exactly 5 files modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/admin_assign_jury.rs    # site 1
  - crates/api/api/src/governance/submit_jury_vote.rs     # sites 2,3,4,5,6
  - crates/api/api/src/governance/admin_close_case.rs     # site 7
  - crates/api/api_crud/src/governance/request_appeal.rs  # site 8
requires: [3]   # governance_case_after_transition fn must exist (Task 3)
```

**Do NOT:**
- Wire cron sites 9/10/11/12 (that is Task 7).
- Modify `bridge_notify.rs`, `Cargo.toml`, or governance_log files.
- Fire the hook INSIDE any `run_transaction(...)` closure (holds DB conn + risks txn rollback of an HTTP call).
- Create any new file.
- Run any cargo command.
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT files on your base branch (`phase-m2-core-hook`)

**The fn to call:** `crates/api/api_utils/src/bridge_notify.rs` — confirm `governance_case_after_transition` is `pub` and its signature: `(context: &LemmyContext, case: &ModerationCase, old_status: Option<CaseStatus>, new_status: CaseStatus) -> LemmyResult<()>`.

**Site 1 — admin_assign_jury.rs:** read lines ~95–201. The `run_transaction` boundary is at ~:198. Fire the hook after the `run_transaction(...).await?` returns, before `Ok(Json(...))`. The case in-scope post-txn is the updated case row.

**Sites 2–6 — submit_jury_vote.rs:** read lines ~145–151, 201, 215, 379–382, 501–518, 974–977, 1020–1026. There is ONE `run_transaction` at line ~149. All 5 sites (deadlock, SL-pending, decided, appeal-deadlock, appeal-verdict) are mutually-exclusive code paths inside the txn. Fire ONCE after the txn returns, deriving `new_status` from the `outcome`/`path_kind` that resolved. Do NOT fire per-path inside the txn.

**Site 7 — admin_close_case.rs:** read lines ~59–75. CRITICAL GOTCHA: at line ~:67, `try_from` MOVES the `case` struct. Add `let old_status = case.status;` (CaseStatus is Copy) BEFORE line 67. Fire after the `run_transaction` returns.

**Site 8 — request_appeal.rs:** read lines ~66–72, 85–158. The txn boundary is at ~:70. `lemmy_api_crud` already depends on `lemmy_api_utils` — the import resolves. Fire after the txn at ~:156.

**The mirror pattern:** `crates/api/api/src/governance/notify.rs:285-306` — the off-txn `.ok()` invocation style.

### Plan sections
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Task 6: WIRE the hook at the 8 handler sites".
- Transition-site reference table in the plan (sites 1–8).

### Lessons (mandatory)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ entry + push, then STOP.
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — the hook is OUTSIDE the txn; do not move it inside.

## 4. Constraints

### 4.0 The call pattern (authoritative shape)

Every site follows this shape:

```rust
// AFTER run_transaction(...).await? — txn committed, conn free
governance_case_after_transition(&context, &case, Some(old_status), new_status)
  .await
  .ok();  // fire-and-forget; bridge being down must NOT fail the handler
```

For `submit_jury_vote` (sites 2–6): the handler has a single `run_transaction` at ~:149. After it returns, the `outcome` / `path_kind` encodes which branch fired. Fire the hook ONCE with the resolved `new_status`. Example skeleton:

```rust
// after run_transaction returns
let (old_status, new_status) = match &outcome {
  VoteOutcome::Deadlock => (pre_txn_status, CaseStatus::AdminReview),
  VoteOutcome::SponsorLiabilityPending => (pre_txn_status, CaseStatus::SponsorLiabilityPending),
  VoteOutcome::Decided => (pre_txn_status, CaseStatus::Decided),
  // appeal paths
  VoteOutcome::AppealDeadlock => (pre_txn_status, CaseStatus::AdminReview),
  VoteOutcome::AppealVerdict => (pre_txn_status, CaseStatus::Closed),
  VoteOutcome::NoDecision => { /* no transition, skip */ return Ok(Json(response)); }
};
governance_case_after_transition(&context, &case, Some(old_status), new_status).await.ok();
```

Adapt exactly to match the real enum variant names and the response shape on your branch — read submit_jury_vote.rs before writing.

### 4.1 Imports to add

In each handler file, add:

```rust
use lemmy_api_utils::bridge_notify::governance_case_after_transition;
```

`lemmy_api` already depends on `lemmy_api_utils` (pre-existing dep). `lemmy_api_crud` also already depends on `lemmy_api_utils`. No Cargo.toml change.

For `ModerationCase` and `CaseStatus` — these are almost certainly already imported in each handler (they run transitions). Confirm the existing import paths; do not re-import.

### 4.2 admin_close_case.rs gotcha — capture old_status before the try_from move

```rust
let old_status = case.status;  // Add this line BEFORE the try_from that moves case
// ... existing try_from ...
// ... run_transaction ...
// after txn:
governance_case_after_transition(&context, &case, Some(old_status), CaseStatus::Closed).await.ok();
```

`CaseStatus` is `Copy` so `let old_status = case.status;` does not move `case`.

### 4.3 validate-pending-laptop DQ entry

After writing all five files and committing, append a `validate-pending-laptop` DQ entry via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 6,
  "commands": [
    "./scripts/brehon/cargo-check.sh -p lemmy_api --features full",
    "./scripts/brehon/cargo-check.sh -p lemmy_api_crud --features full",
    "./scripts/brehon/cargo-clippy.sh -p lemmy_api -p lemmy_api_crud --features full --no-deps -- -D warnings"
  ],
  "question": "Workspace check for governance_case_after_transition wired at sites 1-8 — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 6 added governance_case_after_transition hook call after run_transaction at 8 handler sites: site1=admin_assign_jury, sites2-6=submit_jury_vote (once post-txn, new_status from outcome), site7=admin_close_case (captured old_status before try_from move), site8=request_appeal (api_crud). All calls are outside run_transaction, fire-and-forget .ok(). Import: lemmy_api_utils::bridge_notify::governance_case_after_transition. No Cargo.toml change. Laptop only.",
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

- **Commit subject:** `feat(api): wire governance_case_after_transition at 8 handler sites (task 6)`
- Sequence:
  ```
  git add crates/api/api/src/governance/admin_assign_jury.rs \
          crates/api/api/src/governance/submit_jury_vote.rs \
          crates/api/api/src/governance/admin_close_case.rs \
          crates/api/api_crud/src/governance/request_appeal.rs \
          .claude/decision-queue.json
  git commit -m "feat(api): wire governance_case_after_transition at 8 handler sites (task 6)"
  git push origin <your worktree branch>
  ```
- Then **STOP**. Do not run cargo.

### 4.5 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null`.
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓
- Multi-write handlers → `feedback_multi_write_handlers_need_transactions.md` ✓ (hook is OUTSIDE txn)

## HANDOVER

```yaml
HANDOVER:
  task: m2-core-hook-task-6
  filesCreated: []
  filesModified:
    - crates/api/api/src/governance/admin_assign_jury.rs
    - crates/api/api/src/governance/submit_jury_vote.rs
    - crates/api/api/src/governance/admin_close_case.rs
    - crates/api/api_crud/src/governance/request_appeal.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "Hook called once post-run_transaction at all 8 sites; never inside transaction"
    - "submit_jury_vote: single hook call post-txn, new_status from outcome enum"
    - "admin_close_case: added let old_status = case.status before try_from move at :67"
    - "Import: lemmy_api_utils::bridge_notify::governance_case_after_transition; no Cargo.toml change"
  notes: "Task 7 covers cron sites 9,10,11,12 (different files). Task 8 e2e tests require Tasks 5+6+7 all merged."
```
