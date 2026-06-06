# [role:impl-task] governance-v0 bug fix: txn/race fixes #185 #186 #187

## 1. Role + dispatch

`[role:impl-task]` Fix three high-severity governance race/transactionality bugs (GH #185, #186, #187) on `governance-v0`.

**Base branch:** `governance-v0`  
**Worker branch:** derived from base branch HEAD by the daemon.

---

## 2. Scope

### What to produce

Three targeted fixes — one per issue — committed as three separate commits on the worker branch. No new features, no refactors beyond the minimal guard/reorder.

### Fix A — Issue #185: nonce inside handler transaction (`inbox.rs`)

**File:** `crates/apub/activities/src/governance/inbox.rs`

**Problem:** In `wrap_governance_inbound`, the replay-nonce INSERT (L584–604) runs on `conn` before `inner(activity, context).await` (L607). The nonce commits independently of the handler's own `run_transaction`. If the handler tx rolls back, the nonce row persists but no advisory row is written. AP redelivery then hits the unique PK and is rejected as `FederationActivityReplayed` — the notice is permanently lost.

**Fix:** Make the handler responsible for inserting its own nonce inside its transaction, OR make `wrap_governance_inbound` pass the nonce gate result into `inner` so the handler can commit it atomically with the advisory write. The simplest safe approach: **move the nonce INSERT inside the `inner` call** by passing the `(peer_domain, activity_id)` pair into `inner` as extra args, or by restructuring `wrap_governance_inbound` to open a transaction that wraps both the nonce INSERT and the `inner()` call.

**Concrete implementation approach (preferred for minimal diff):** Change `wrap_governance_inbound` to perform the nonce INSERT inside a `conn.run_transaction` block that also calls `inner()`. The nonce check (uniqueness guard returning 409) still happens on the first try; on rollback from within `inner()`, the nonce rolls back too, allowing correct AP redelivery.

Example structure (pseudo):
```rust
// After gate 5, instead of: nonce INSERT then inner(activity, context).await
conn.run_transaction(async |conn_tx| {
    // nonce INSERT here (same unique-violation check → return FederationActivityReplayed)
    // then call inner, passing conn_tx somehow — OR restructure inner to accept conn
}).await?
```

If the `inner` closure signature cannot accept a `conn` argument without large refactor, use **Option B**: convert the handler to be idempotent — on the nonce-unique-violation path, instead of rejecting with 409, query whether the advisory row already exists and if yes return `Ok(())` (idempotent redelivery). This is the "make idempotent" branch from the issue fix options. Pick whichever is a smaller diff; document the choice in the commit body.

### Fix B — Issue #186: capture `old_status` from inside the transaction (`admin_assign_jury.rs`)

**File:** `crates/api/api/src/governance/admin_assign_jury.rs`

**Problem:** `old_status` is read outside `run_transaction` (L97–102). A concurrent status transition between the outside read and the committed txn means `governance_case_after_transition` (L114) fires with a stale `old_status`.

**Fix:** Return `old_status` from `process_assignment` alongside the existing `AdminAssignJuryResponse`. Change `process_assignment` to return `LemmyResult<(AdminAssignJuryResponse, CaseStatus)>` where the `CaseStatus` is `case.status` read inside the txn (step 1 already reads the case at L133). The outer handler captures both, drops the pre-txn `case_for_hook` read (no longer needed for `old_status`), and passes the txn-stable value into the hook.

Specifically:
- In `process_assignment`: at L140 (after `GovernanceCase::<PreJuryAssignable>::try_from(case)?`), capture `let old_status = case.status;` (the status before the type-state guard changes it). Return `Ok((response, old_status))` at the function end.
- In the outer handler: destructure `let (outcome, old_status) = conn.run_transaction(...).await?;`; remove the pre-txn `case_for_hook` read and `old_status` capture (L97–102); the `case_for_hook` is still needed for the hook call's `&case_for_hook` arg — either re-read it after the txn OR pass it out of the txn too (add it to the tuple). Simplest: include `case_for_hook` in the txn return tuple as well.

### Fix C — Issue #187: status guard on decline UPDATE (`decline_jury_assignment.rs`)

**File:** `crates/api/api/src/governance/decline_jury_assignment.rs`

**Problem:** The status flip at L101–107 filters only by `jury_assignment::id`. No predicate guards against the row having been concurrently flipped (to Declined or Accepted). Two concurrent declines can both proceed and each seat a replacement, overfilling the panel.

**Fix:** Mirror `accept_jury_assignment.rs:136–155` exactly:
1. Add `.filter(jury_assignment::status.eq(JuryAssignmentStatus::Selected).or(jury_assignment::status.eq(JuryAssignmentStatus::Accepted)))` to the UPDATE at L101.
2. Capture the return as `rows_affected` and guard: `if rows_affected == 0 { return Err(LemmyErrorType::NotFound.into()); }`.
3. Only proceed to replacement selection (step 6, L157+) when `rows_affected == 1`.

Note: step 1 (L79–90) already filters the SELECT by `Selected | Accepted`, so the UPDATE predicate merely closes the READ COMMITTED window between step 1 and step 3.

### What NOT to produce

- No changes to `crates/db_schema/src/source/governance/governance_log.rs` (no new entry kinds needed).
- No changes to migrations.
- No changes to e2e tests (these are handler-level fixes; e2e coverage of the race path requires concurrent requests which is not feasible in the existing harness).
- No changes to any file outside the three listed above.

---

## 3. Required reading

- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — transaction discipline for multi-write handlers.
- `.claude/lessons/feedback_governance_type_state_handlers.md` — `GovernanceCase<S>` type-state pattern; understand how `try_from` works before touching `process_assignment`.
- `crates/api/api/src/governance/accept_jury_assignment.rs` L128–155 — the exact guard pattern to mirror for Fix C.
- `crates/apub/activities/src/governance/inbox.rs` L493–608 — full `wrap_governance_inbound` body before touching it.
- `crates/api/api/src/governance/admin_assign_jury.rs` L86–119 — outer handler and L123–300 `process_assignment` body before touching.
- `crates/api/api/src/governance/decline_jury_assignment.rs` L75–197 — full `process_decline` body before touching.

---

## 4. Constraints

1. **Three separate commits** — one per fix, in order A, B, C. Commit subjects: `fix(apub): nonce INSERT inside handler tx — closes #185`, `fix(governance): capture old_status from inside txn — closes #186`, `fix(governance): status guard on decline UPDATE — closes #187`.
2. **DoD per commit:** after each commit run `cargo check --workspace --features full` (via the wrapper: `./scripts/brehon/cargo-check.sh --workspace --features full`). All three must exit 0.
3. **Write a `validate-pending-laptop` DQ entry** after the third commit with `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`, `branch: <worker branch name>`, `phase_task: "governance-fix-high-1"`. Commit + push the DQ entry, then **stop**. Do NOT run cargo-check yourself after writing the DQ — that is delegated to the laptop advisor.
4. **No `answered_by: "advisor"` in DQ entries** from this worker session.
5. **No changes to files outside the three listed** under §2.
6. Fix A may be a 2–5 line change or a modest restructure; if the minimal diff for Fix A exceeds 40 lines, stop and raise a `kind: "blocker"` DQ explaining the chosen approach and its line count before proceeding.
