# Brief: v1-RT-r5 fix-impl-1 — CR findings (PR #171)

## 1. Role + dispatch line

`[role:impl-task] v1-RT-r5 fix-impl-1 CR fixes — see .claude/PRPs/briefs/v1-RT-r5-fix-impl-1.md`

## 2. Scope

Fix 4 CodeRabbit findings from PR #171 in exactly 2 files:

- `crates/api/api/src/governance/reputation_snapshot.rs`
- `crates/api/api/src/governance/admin_reputation_rollup.rs`

**Do NOT touch any other file.** No e2e.rs edits. No migrations. No new files.

After fixing, write a `validate-pending-laptop` DQ entry with
`commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`,
commit + push, then **stop**. Do NOT run cargo yourself.

## 3. Required reading

**Mandatory lessons (all apply):**

- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — `conn.run_transaction(async |conn| { ... })` pattern for 2+ DB writes
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A error shape throughout
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection` usage
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect`

**MIRROR refs (read before editing):**

- `crates/api/api/src/governance/submit_jury_vote.rs:144` — canonical `conn.run_transaction(async |conn| { ... })` pattern in this codebase
- `crates/api/api/src/governance/reputation_snapshot.rs:564-740` — `compute_rollup_snapshot` function (the target of fixes 1–3)
- `crates/api/api/src/governance/admin_reputation_rollup.rs:38-55` — `contributing` query (target of fix 4)

**Pre-locate anchors** (run before any Edit):
```bash
grep -c "return Ok(None)" crates/api/api/src/governance/reputation_snapshot.rs
grep -c "chunk_size_usize" crates/api/api/src/governance/reputation_snapshot.rs
grep -c "load::<ReputationSnapshot>" crates/api/api/src/governance/admin_reputation_rollup.rs
```
Confirm each returns `1` before using as an Edit anchor.

## 4. Implementation

### Fix 1 — Transaction wrap (CRITICAL: blocks merge)

In `reputation_snapshot.rs`, function `compute_rollup_snapshot` (line ~564):

The section from comment `// 8. Load existing rollup row` through the final
`Ok(Some(new_snapshot))` (lines ~678–740) does multiple DB writes (upsert + 
governance_log::append calls) WITHOUT a transaction. Wrap steps 8–11 in a
`conn.run_transaction` block:

```rust
// Pattern from submit_jury_vote.rs:144
conn.run_transaction(async |conn| {
  // 8. Load existing rollup row for capability-flip detection.
  let old_snapshot: Option<ReputationSnapshot> = reputation_snapshot::table
    .filter(reputation_snapshot::person_id.eq(person_id))
    .filter(reputation_snapshot::community_id.is_null())
    .first::<ReputationSnapshot>(conn)
    .await
    .optional()?;

  // 9. Upsert rollup row (community_id = None, shared write path :770).
  let new_snapshot = upsert_snapshot(conn, &form, now).await?;

  // 10. Emit ROLLUP_RECOMPUTED.
  governance_log::append(...).await?;

  // 11. Emit CAPABILITY_CHANGED per flip.
  for change in &changes {
    governance_log::append(...).await?;
  }

  Ok(Some(new_snapshot))
})
.await
```

The `form` and `changes` variables must be computed BEFORE the transaction block
(they don't require DB access — they're computed from `contributing` which is
loaded in steps 1–7 above). Move `detect_capability_changes` inside the
transaction (it needs `old_snapshot` which is now inside the tx). The
`governance_log::append` calls take `&mut (&mut *conn).into()` — inside the
transaction closure `conn` is the transaction connection, so the pattern stays
the same. Read `submit_jury_vote.rs:137-160` for the exact closure shape
(`.boxed()` may be needed for the async closure).

### Fix 2 — Delete stale rollup on denominator == 0

In `reputation_snapshot.rs`, in the `denominator == 0` branch (line ~607):

Before returning `Ok(None)`, delete any existing instance-wide rollup row:

```rust
if denominator == 0 {
  // All communities banned; delete stale rollup row if it exists.
  diesel::delete(
    reputation_snapshot::table
      .filter(reputation_snapshot::person_id.eq(person_id))
      .filter(reputation_snapshot::community_id.is_null()),
  )
  .execute(conn)
  .await?;
  return Ok(None);
}
```

Import `diesel::delete` — check if already imported via `use diesel::prelude::*`
or similar at the top of the file.

### Fix 3 — Guard chunks(0) panic

In `reputation_snapshot.rs`, in `run_rollup_batch` (line ~755):

After `chunk_size_usize` is computed, add a zero-guard before the loop:

```rust
let chunk_size_usize = usize::try_from(chunk_size).map_err(|_e| {
  LemmyErrorType::Unknown(format!(
    "job.snapshot_batch_chunk_size out of range for usize: {chunk_size}"
  ))
})?;

// Guard: chunks(0) panics. Treat 0 as 1 to prevent panic on misconfiguration.
let chunk_size_usize = if chunk_size_usize == 0 { 1 } else { chunk_size_usize };
```

This is a one-line re-binding of the same variable — simple shadow.

### Fix 4 — ORDER BY on contributing query

In `admin_reputation_rollup.rs` (line ~43):

Add `.order(reputation_snapshot::community_id.asc())` before `.load`:

```rust
let contributing: Vec<ReputationSnapshot> = reputation_snapshot::table
  .filter(reputation_snapshot::person_id.eq(person_id))
  .filter(reputation_snapshot::community_id.is_not_null())
  .order(reputation_snapshot::community_id.asc())
  .load::<ReputationSnapshot>(conn)
  .await?;
```

### 4.7 validate-pending-laptop DQ entry

After committing all fixes with subject
`fix(reputation): CR fixes — tx wrap, stale delete, chunks guard, ORDER BY (v1-RT-r5)`,
write DQ entry:

```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "phase_task": "fix-impl-1",
  "branch": "phase-v1-RT-r5",
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"]
}
```

Generate id via `bash scripts/brehon/dq-v3-new-entry.sh`. Append via
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`.
Commit + push. **Stop immediately — do NOT run cargo.**

## 5. Constraints

- **Case A error shape** everywhere: `LemmyResult<()>` + bare `?`
- **No cargo on daemon** — write DQ + stop only
- **Only 2 files**: `reputation_snapshot.rs` and `admin_reputation_rollup.rs`
- **Single commit** covering all 4 fixes
- **Mandatory lessons fired:** `feedback_multi_write_handlers_need_transactions.md`,
  `feedback_lemmy_error_no_std_error.md`, `feedback_fix_impl_pre_locate_e2e_anchors.md`
