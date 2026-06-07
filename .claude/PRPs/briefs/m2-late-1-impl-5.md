# Brief: m2-late-1 Task 5 — Spawn site (`submit_jury_vote.rs`)

## 1. Role + dispatch

`[role:impl-task] m2-late-1-task-5-spawn-site — see .claude/PRPs/briefs/m2-late-1-impl-5.md`

Pre-Shape-G plan. Worker writes `validate-pending-laptop` DQ and STOP; does NOT run workspace cargo on the daemon.

## 2. Scope

**Produce:**

1. **Modify** `crates/api/api/src/governance/submit_jury_vote.rs` — add the post-transaction re-query + `tokio::spawn(enqueue_sanction_event(...))` call after `conn.run_transaction(...)` returns `Ok` in `process_vote`. ONE spawn site only.

Then write the `validate-pending-laptop` DQ entry and STOP. Do NOT run workspace cargo on the daemon.

**Explicit boundaries (do NOT touch):**
- Do NOT edit `crates/server/**` — T6's domain.
- Do NOT edit `services/bridge/**` — T7's domain.
- Do NOT edit `crates/server/tests/e2e.rs` or `crates/server/tests/e2e/**` — isolated to T8.
- Do NOT run `cargo check --workspace` on the daemon.

## 3. Required reading

**Before your first edit, Read these files:**

- `crates/api/api/src/governance/submit_jury_vote.rs:163-178` — `governance_case_after_transition` post-tx re-query (MIRROR for the new re-query shape).
- `crates/api/api/src/governance/submit_jury_vote.rs:452-480` — sanction emission inside the tx (shows Sanction struct fields; provides context for where process_vote's tx block ends).
- `crates/api/api/src/governance/submit_jury_vote.rs:1059-1061` — "No sanction row, no federation outbound" in `process_appeal_vote` — confirm there is NO spawn site there (STOP tripwire).
- `crates/api/api/src/governance/sanction_publisher.rs` — the T4 output; understand `enqueue_sanction_event` signature: `(sanction: Sanction, ctx: SanctionContext) -> LemmyResult<()>`.
- `.claude/PRPs/plans/m2-late.plan.md §10.3` — verbatim spawn-site body spec.
- `feedback_validate_pending_laptop_write_then_stop.md` — STOP after DQ entry.
- `feedback_multi_write_handlers_need_transactions.md` — NOTE: the spawn runs OUTSIDE the transaction; do NOT wrap in a transaction.

## 3a. Handover from prior cohort

T4 completed on laptop worktree at `0a9d01430`. `sanction_publisher.rs` + `pub mod sanction_publisher` are live. DQ `282a5cbdfaa8-001` resolved pass.

## 4. Implementation spec (verbatim from plan §10.3)

Insert the following block in `process_vote`, AFTER `conn.run_transaction(...) -> Ok` returns, BEFORE the function's final return:

```rust
// Post-tx re-query: find the active sanction written in this transaction.
// Re-query is used instead of threading the sanction through process_vote's
// 5 return sites — mirrors governance_case_after_transition pattern (:163-178).
let published: Option<Sanction> = sanction::table
  .filter(sanction::case_id.eq(data.case_id))
  .filter(sanction::active.eq(true))
  .first::<Sanction>(&mut context.pool().get().await?)
  .await
  .optional()?;

if let Some(sanction) = published {
  if sanction.target_person_id.is_some() {
    let ctx = context.clone();
    tokio::spawn(async move {
      if let Err(e) = enqueue_sanction_event(sanction, ctx).await {
        tracing::warn!("sanction publish failed: {e}");
      }
    });
  }
}
```

**Imports to add** (if not already present in `submit_jury_vote.rs`):
- `use crate::governance::sanction_publisher::enqueue_sanction_event;`
- `diesel::OptionalExtension` — check if already imported; add only if missing.
- `sanction::table` / `sanction::case_id` / `sanction::active` — likely already imported for the existing sanction emission at :452-480; verify before adding.

**Exact insertion point:** find the line in `process_vote` where `conn.run_transaction(...)` completes and returns `Ok(output)` (or similar). The re-query + spawn block goes AFTER that, BEFORE `return Ok(output)` or before the final expression that yields the function result.

**STOP tripwire:** if you see a `tokio::spawn` call already present in `submit_jury_vote.rs`, STOP and raise a `kind: "blocker"` DQ — the spawn site may already exist. If absent, proceed.

## 5. Constraints

- **R7:** Do NOT run `./scripts/brehon/cargo-check.sh` or any `cargo` command on the daemon. After committing, write `validate-pending-laptop` DQ entry then STOP immediately.
- **ONE spawn site only:** `process_vote` only. Not `process_appeal_vote` (:1059-1061 writes no sanction row).
- **Outside the tx:** the `published` re-query and `tokio::spawn` are OUTSIDE `conn.run_transaction(...)`. Do NOT move them inside.
- **DQ commit subject:** `chore(decision-queue): impl raised validate-pending-laptop for m2-late-1 task-5`.
- **Impl commit subject:** `feat(api): wire enqueue_sanction_event spawn site in submit_jury_vote.rs (task 5)`.
- One impl commit + DQ commit then STOP. Push both commits to `origin phase-m2-late-1` immediately after each. Note: `git push` via HTTPS may hang silently on the daemon — do NOT retry push more than once.

**Mandatory lessons fired:**
- `feedback_validate_pending_laptop_write_then_stop.md` — new file under `crates/api/api/src/governance/**`.
- `feedback_multi_write_handlers_need_transactions.md` — NOTE: intentionally NOT wrapped in a transaction (runs outside the vote tx).
