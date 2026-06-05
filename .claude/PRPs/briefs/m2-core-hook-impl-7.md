---
phase: m2-core-hook
role: impl-task
n: 7
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-core-hook
task_number: 7
requires: [3]
minimax_trial: eligible (Task 7 — control=Sonnet, trial=MiniMax-M2.7; both arms read THIS brief)
---

# [role:impl-task] m2-core-hook Task 7 — wire governance_case_after_transition at cron sites + site 12

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-7 wire hook at cron sites 9-12 — see .claude/PRPs/briefs/m2-core-hook-impl-7.md
```

> **A/B note (does not change your work):** this task is dispatched as two parallel arms off the SAME base — a control arm (Sonnet) and a trial arm (MiniMax-M2.7). Each arm gets its own worktree branch and runs this brief independently. You do exactly what this brief says; do not reference or coordinate with the other arm.

## 2. Scope

Wire `governance_case_after_transition` at cron sites 9, 10, 11 and the api_crud loop site 12. **Two files, modify-only.** **Pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself.**

**Produce (exactly 2 files modified, 0 created):**

```yaml
creates: []
modifies:
  - crates/api/api/src/governance/appeal_window_expiry.rs     # site 9: accumulate + fire after loop
  - crates/api/api/src/governance/sponsor_liability_grace.rs  # sites 10,11: fire after each per-case txn
requires: [3]   # governance_case_after_transition fn must exist (Task 3)
```

> Note: site 12 (`revoke_endorsement.rs`) is also part of this wiring but that file is in `api_crud`. If the brief lists 2 files and the plan requires 3, read the plan §Task 7 notes again. Per the plan, site 12's accumulate-then-fire may require also modifying `crates/api/api_crud/src/governance/revoke_endorsement.rs`. If so, add it to your modifies list — accuracy beats the 2-file brief count. Do NOT wire it from inside the `run_transaction` closure.

**Do NOT:**
- Wire handler sites 1–8 (that is Task 6).
- Modify `bridge_notify.rs`, `Cargo.toml`, or governance_log files.
- Fire the hook INSIDE any `run_transaction(...)` closure or while holding the DB connection.
- Create any new file.
- Run any cargo command.
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### MIRROR refs — read these EXACT files on your base branch (`phase-m2-core-hook`)

**The fn to call:** `crates/api/api_utils/src/bridge_notify.rs` — confirm `governance_case_after_transition` signature: `(context: &LemmyContext, case: &ModerationCase, old_status: Option<CaseStatus>, new_status: CaseStatus) -> LemmyResult<()>`.

**Site 9 — appeal_window_expiry.rs:** read lines ~38–75. This is a CRON BATCH — a `for` loop processes multiple cases. The loop holds the DB conn (via `run_transaction` or bare execute). The hook MUST NOT fire inside the loop while the conn is held. Instead: collect `(case: ModerationCase, new_status: CaseStatus)` tuples into a `Vec` inside the loop, then iterate and fire after the loop completes (conn returned). Pattern:

```rust
let mut transitions: Vec<(ModerationCase, CaseStatus)> = Vec::new();
for case in &batch {
  // ... existing transition logic (run_transaction or execute) ...
  transitions.push((case.clone(), CaseStatus::Closed));  // adjust status
}
// After loop — conn free
for (case, new_status) in transitions {
  governance_case_after_transition(&context, &case, Some(CaseStatus::Active), new_status)
    .await.ok();
}
```

Derive `old_status` from the pre-transition case status (read from the case object, or the known pre-state for appeal-window expiry cases).

**Sites 10+11 — sponsor_liability_grace.rs:** read lines ~160–169 and ~405–516. The fn `fire_or_escape_case_inner` runs a `run_transaction` for each case. After each per-case `run_transaction(...).await` resolves, fire the hook once, deriving new_status from the outcome (`Fired → SponsorLiabilityFired`, `Escaped → SponsorLiabilityEscaped`). The `case`/`re_loaded` snapshot before the txn captures `old_status = SponsorLiabilityPending`. Pattern:

```rust
let result = pool.run_transaction(|conn| { ... }).await?;
// After per-case txn — conn released
let new_status = match result {
  PerCaseOutcome::Fired => CaseStatus::SponsorLiabilityFired,
  PerCaseOutcome::Escaped => CaseStatus::SponsorLiabilityEscaped,
};
governance_case_after_transition(&context, &case, Some(CaseStatus::SponsorLiabilityPending), new_status)
  .await.ok();
```

Adapt to the real enum names on your branch.

**Site 12 — revoke_endorsement.rs:** read lines ~141–149 and ~247–312. `process_revocation` loops over `pending_cases` and runs a `run_transaction` at ~:142 for each. Inside the closure, escapes happen for each case that transitions `SponsorLiabilityPending → SponsorLiabilityEscaped`. Pattern: accumulate escaped-case tuples inside the closure's return (thread them out via `outcome`), then fire hooks after `run_transaction(...).await?` at the handler level — NOT inside the closure. Check what `outcome` already contains: if it already tracks escaped cases, extend it; if not, return the Vec from the closure.

### Plan sections
- `.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Task 7: WIRE the hook at the 3 cron sites + the api_crud loop site (site 12)".
- Transition-site reference table (sites 9–12).

### Lessons (mandatory)
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ entry + push, then STOP.
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — hook is OUTSIDE the txn boundary.

## 4. Constraints

### 4.0 The accumulate-then-fire pattern for batch/loop sites (sites 9, 12)

NEVER call `governance_case_after_transition` while holding a DB connection inside a loop. The fn does HTTP I/O — it cannot overlap with a DB transaction or connection hold.

For batch sites (9, 12):
```rust
// Phase 1: collect transitions (inside loop, using DB)
let mut pending_hooks: Vec<(ModerationCase, Option<CaseStatus>, CaseStatus)> = Vec::new();
for case in &batch {
  // ... existing DB work ...
  pending_hooks.push((case.clone(), Some(old_status), new_status));
}
// Phase 2: fire hooks (after loop, DB conn returned)
for (case, old_status, new_status) in pending_hooks {
  governance_case_after_transition(&context, &case, old_status, new_status).await.ok();
}
```

### 4.1 Imports to add

In each file, add:

```rust
use lemmy_api_utils::bridge_notify::governance_case_after_transition;
```

`lemmy_api` already depends on `lemmy_api_utils`. `lemmy_api_crud` also already depends on `lemmy_api_utils`. No Cargo.toml change needed.

`ModerationCase` and `CaseStatus` should already be imported in these files (they perform transitions). Confirm on your branch.

### 4.2 validate-pending-laptop DQ entry

After writing all files and committing, append a `validate-pending-laptop` DQ entry via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 7,
  "commands": [
    "./scripts/brehon/cargo-check.sh -p lemmy_api --features full",
    "./scripts/brehon/cargo-check.sh -p lemmy_api_crud --features full",
    "./scripts/brehon/cargo-clippy.sh -p lemmy_api -p lemmy_api_crud --features full --no-deps -- -D warnings"
  ],
  "question": "Workspace check for governance_case_after_transition wired at cron sites 9-11 + site 12 — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 7 wired governance_case_after_transition at: site9=appeal_window_expiry (accumulate-then-fire after loop), sites10+11=sponsor_liability_grace fire_or_escape_case_inner (after each per-case run_transaction, old=SponsorLiabilityPending), site12=revoke_endorsement process_revocation (accumulate escaped cases inside closure, fire after run_transaction returns at handler level). All outside txn boundaries. Import: lemmy_api_utils::bridge_notify::governance_case_after_transition. Laptop only.",
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

- **Commit subject:** `feat(api): wire governance_case_after_transition at cron sites 9-11 + site 12 (task 7)`
- Sequence:
  ```
  git add crates/api/api/src/governance/appeal_window_expiry.rs \
          crates/api/api/src/governance/sponsor_liability_grace.rs \
          .claude/decision-queue.json
  ```
  If site 12 required modifying `revoke_endorsement.rs`, also stage:
  ```
  git add crates/api/api_crud/src/governance/revoke_endorsement.rs
  ```
  Then:
  ```
  git commit -m "feat(api): wire governance_case_after_transition at cron sites 9-11 + site 12 (task 7)"
  git push origin <your worktree branch>
  ```
- Then **STOP**. Do not run cargo.

### 4.4 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null`.
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓
- Multi-write handlers → `feedback_multi_write_handlers_need_transactions.md` ✓ (hook is OUTSIDE txn)

## HANDOVER

```yaml
HANDOVER:
  task: m2-core-hook-task-7
  filesCreated: []
  filesModified:
    - crates/api/api/src/governance/appeal_window_expiry.rs
    - crates/api/api/src/governance/sponsor_liability_grace.rs
    - crates/api/api_crud/src/governance/revoke_endorsement.rs  # if site 12 required it
    - .claude/decision-queue.json
  keyDecisions:
    - "Site 9: accumulate-then-fire after batch loop (never inside loop with conn held)"
    - "Sites 10+11: fire after each per-case run_transaction in fire_or_escape_case_inner"
    - "Site 12: accumulate escaped cases inside txn closure, fire after commit at handler level"
    - "Import: lemmy_api_utils::bridge_notify::governance_case_after_transition; no Cargo.toml change"
  notes: "Task 8 e2e tests require Tasks 5+6+7 all merged. Task 6 covers handler sites 1-8."
```
