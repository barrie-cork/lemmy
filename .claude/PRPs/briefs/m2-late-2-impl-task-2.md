---
phase: m2-late-2
role: impl-task
n: 2
authored: 2026-06-12
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-late-2
task_number: 2
parallel_group: cohort-2
---

# [role:impl-task] m2-late-2 Task 2 — CR-A atomicity fix

## 1. Role + dispatch line

```
[role:impl-task] m2-late-2 task-2 CR-A atomicity — see .claude/PRPs/briefs/m2-late-2-impl-task-2.md
```

## 2. Scope

Wrap the `sanction_event` INSERT + `governance_log::append` in one fresh `run_transaction`
inside `enqueue_sanction_event`, closing the ADR-008 atomicity gap (CR-A).

### IMPLEMENT

| File | Change |
|---|---|
| `crates/api/api/src/governance/sanction_publisher.rs` | Replace lines 178-207 (separate INSERT block + `append(&mut ctx.pool(), …)` call) with the single-`run_transaction` shape per Pattern 10.1 below. |

### Explicit out-of-scope

- Do NOT change `governance_log::append`'s public signature (D3: the reborrow shape is the
  fix; widening `append` is a scope escalation requiring a DQ).
- Do NOT move the subscriber POST loop inside the transaction — it must stay BEFORE the
  transaction (delivery latency must be outside the tx).
- Do NOT touch the vote transaction connection (`conn` from `process_vote`) — this fix opens
  a FRESH `get_conn` (R8 critical).
- Do NOT edit `m2_late.rs` or any other file beyond `sanction_publisher.rs`.
- Do NOT touch `services/bridge/`.

## 3. Implementation pattern (verbatim from plan §10.1)

**Mirror:** `crates/api/api/src/governance/admin_assign_jury.rs:218-231` (call shape)
**and** `crates/db_schema/src/source/governance/governance_log.rs:308-335` (inner SAVEPOINT).

Read BOTH mirror files before writing any code — the reborrow shape must be byte-aligned with
the proven exemplar.

Replace `sanction_publisher.rs:178-207` with:

```rust
let mut pool = ctx.pool();
let conn = &mut get_conn(&mut pool).await?;          // FRESH conn — NOT the vote tx (R8)
conn
  .run_transaction(async |conn| {
    let event_form = SanctionEventInsertForm {
      sanction_id: sanction.id,
      sanction_kind: payload.sanction_kind.clone(),
      subject_actor_pseudonym: subject.clone(),
      effective_from: sanction.starts_at,
      effective_until: sanction.ends_at,
      governance_log_entry_hash: payload.governance_log_entry_hash.clone(),
    };
    insert_into(sanction_event_dsl::table)
      .values(&event_form)
      .execute(conn)
      .await?;

    // Reborrow: `append`'s own run_transaction becomes a SAVEPOINT of THIS tx.
    governance_log::append(
      &mut (&mut *conn).into(),
      kind,
      serde_json::json!({
        "sanction_id": sanction.id.0,
        "sanction_kind": &payload.sanction_kind,
        "subscriber_count": subscribers.len(),
        "subject_actor_pseudonym": &subject,
      }),
      Some(subject.clone()),
    )
    .await?;

    Ok(())
  })
  .await?;
```

**Key details:**
- `kind` is the `ENTRY_KIND_*` constant already resolved before this block (the current code
  already selects the right kind; keep that logic).
- `subscribers` is the list from the read-subscribers step before line 178 (already in scope).
- `subject` is the `actor_pseudonym.pseudonym` already resolved before line 178.
- `payload` fields are already built by Task 1 (the `SanctionEventPayload` with `case_id`).
- `sanction_event_dsl` is already imported in the current file.
- The subscriber POST loop at current lines 150-175 (POSTs to each subscriber) stays BEFORE
  this block — do not move it.

**GOTCHA:** `append` returns `LemmyResult<GovernanceLog>`; inside the closure use `.await?;`
and discard the return value; the closure must return `LemmyResult<()>` (the `Ok(())` on the
last line). The reborrow `(&mut *conn)` is mandatory — a bare `conn.into()` would MOVE the
closure-local `conn`, making the `execute(conn)` above invalid. **If the reborrow does not
type-check, STOP and file a DQ (D3) — do NOT widen `governance_log::append`'s signature.**

## 4. Validation gate (validate-pending-laptop DQ — write then STOP)

After committing the change, write a `kind: "validate-pending-laptop"` DQ entry with:

```json
{
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings\"",
    "cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full sanction_event_delivered_to_subscriber\""
  ],
  "branch": "<current-worktree-branch>",
  "phase_task": 2,
  "e2e_filter": null
}
```

**NOTE:** e2e command is `--workspace --test e2e --features full` (NOT `-p lemmy_server
--features full` — that form is invalid; `lemmy_server` has no `full` feature). This is the
correct form per `feedback_features_full_p_crate_incompatible.md`.

Commit the DQ entry + push the branch, then **STOP**. Do NOT run cargo yourself.

Generate the DQ id via `bash scripts/brehon/dq-v3-new-entry.sh`.
Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`.

## 5. Required reading

- `.claude/PRPs/plans/m2-late-2.plan.md` — §13 Task 2 spec, Pattern 10.1 (exact target shape),
  §7 R8 (CRITICAL: fresh conn only), §7 R-multi-write, §9 mandatory reading list
- `crates/api/api/src/governance/admin_assign_jury.rs:218-231` — **canonical exemplar** of
  `governance_log::append` called inside a caller's `run_transaction`; mirror the call shape
  EXACTLY
- `crates/db_schema/src/source/governance/governance_log.rs:277-336` — `append` signature +
  SAVEPOINT doc-comment (lines 294-302: reborrow promotes inner `run_transaction` to SAVEPOINT)
- `crates/api/api/src/governance/sanction_publisher.rs:1-233` — full file: current non-atomic
  writes at lines 178-207; payload struct at 36-46; build site at 130-136; subscriber POST
  loop at ~150-175 (stays BEFORE the new tx)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — the CR-A fix IS this
  pattern; the lesson explains WHY `run_transaction` + reborrow is the right shape
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — any new `?` in the closure uses
  Case A (`.map_err(LemmyError::from)`) if the error type isn't already `LemmyError`

## 6. Constraints

- **R8 (CRITICAL):** `enqueue_sanction_event` runs OUTSIDE the vote transaction. The fresh
  `get_conn(&mut pool)` opened inside this fix MUST NOT be the `conn` passed in from
  `process_vote` or any outer transaction. Catch-fire if the fix reuses an outer conn.
- **NO cargo on EliteDesk (NO-CARGO-ON-ELITEDESK)** — write the validate-pending-laptop DQ
  and STOP. Do NOT invoke any cargo/bat wrapper.
- **`governance_log::append` signature UNCHANGED** — the fix uses the `&mut (&mut *conn).into()`
  reborrow, not a new overload. If the reborrow doesn't type-check: DQ, do not widen.
- **Subscriber POST loop stays BEFORE the transaction** — delivery latency is outside the tx.
- **Only `sanction_publisher.rs` is touched** — no other file changes.
- **DQ v3 id** — generate via `bash scripts/brehon/dq-v3-new-entry.sh`.
- **Mid-task push discipline** — commit DQ entry + push immediately after writing it.
- **Commit prefix** — `feat(governance): ` subject, with `LESSON:` trailer if any
  non-obvious finding arises.

### Mandatory lesson fires (file-class table check)

| File class | Match? | Lesson injected |
|---|---|---|
| Any handler doing 2+ DB writes | ✅ (the fix merges 2 writes into one tx) | `feedback_multi_write_handlers_need_transactions.md` (§5 above) |
| `crates/api/api/src/governance/**` ADR gate | ✅ (ADR-008 pin: atomic INSERT + audit-log) | see §6 R8 constraint + §5 lesson above |
| `crates/server/tests/e2e.rs` edits | ❌ no e2e edits in T2 | — |
| Any `#[cfg(feature = "full")]` gate | ❌ no new feature gates | — |
