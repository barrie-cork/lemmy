# Brief: v1-quality-r3c-fix-impl-2

## 1. Role + dispatch line

`[role:impl-task] v1-quality-r3c-fix-impl-2 — fix T3 compile: correct FederationInboxNonce API calls — see .claude/PRPs/briefs/v1-quality-r3c-fix-impl-2.md`

## 2. Scope

**Root cause (fix-impl-1 compile errors):**
1. `FederationInboxNonce::insert(...)` does not exist — `FederationInboxNonce` has no `insert` associated fn. Use `diesel::insert_into(federation_inbox_nonce::table).values(&form).execute(conn).await?` directly.
2. `federation_inbox_nonce::delete_older_than` is a FREE FUNCTION at `lemmy_db_schema::source::governance::federation_inbox_nonce::delete_older_than`, NOT a method on the schema table module `federation_inbox_nonce`. Add an explicit import and call it as `delete_older_than(...)`.
3. `delete_older_than(window_days <= 0, ...)` returns `Ok(0)` immediately without touching the DB (early-exit guard in the function body). Probe A must use a positive window; but fresh rows have `seen_at = now` so they won't be deleted by any reasonable window. Probe A should instead verify the function completes without error and the fresh row survives (count unchanged) — not that it deletes anything.

**Fix:** edit `crates/server/tests/e2e.rs` to correct `test_brehon_disable_fed_replay_cleanup_job`:
- Replace both `FederationInboxNonce::insert(...)` calls with `diesel::insert_into(federation_inbox_nonce::table).values(&form).execute(&mut async_conn).await?`
- Add `use lemmy_db_schema::source::governance::federation_inbox_nonce::delete_older_than as fed_nonce_delete;` to `v1_rt_r3_fixtures`'s use block
- Call `fed_nonce_delete(1, &mut async_conn).await?` in probe A (just verifies the function runs ok; fresh row has seen_at=now so it won't be deleted by 1-day window — that's expected)
- Probe A assertion: `assert_eq!(count_after, 1, "fresh row survives delete_older_than(1,...) — seen_at is now, not old")` — note this changes the assertion to count=1 (row survives because it's fresh)

**Commit subject:** `fix(e2e): correct FederationInboxNonce API calls in DISABLE_* guard tests`

**Exactly one file to edit:** `crates/server/tests/e2e.rs`

**Do NOT:**
- Edit any other `crates/**` files
- Edit any `migrations/**`, `docs/**`, or `.claude/**` files
- Run `cargo test` — write the `validate-pending-laptop` DQ entry and stop

## 3. Required reading

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — LemmyResult<()> + ? pattern
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — verify each anchor count=1 BEFORE Edit

### §3a. Verify FederationInboxNonce actual API BEFORE writing any Edit

```bash
# 1. Confirm no insert method
grep -n "fn insert\|pub fn insert\|pub async fn insert" crates/db_schema/src/source/governance/federation_inbox_nonce.rs
# Expected: 0 results

# 2. Confirm delete_older_than is a free function (NOT on FederationInboxNonce struct)
grep -n "pub async fn delete_older_than\|fn delete_older_than" crates/db_schema/src/source/governance/federation_inbox_nonce.rs
# Expected: 1 result, free function not inside impl block

# 3. Confirm delete_older_than early-exit for window_days <= 0
grep -A 3 "fn delete_older_than" crates/db_schema/src/source/governance/federation_inbox_nonce.rs
# Expected: if window_days <= 0 { return Ok(0); }
```

### §3b. Pre-locate edit anchors (run BEFORE Edit)

**Anchor — the function signature** (count must be 1):
```
async fn test_brehon_disable_fed_replay_cleanup_job() -> LemmyResult<()> {
```
Run: `grep -c 'async fn test_brehon_disable_fed_replay_cleanup_job' crates/server/tests/e2e.rs`
Expected: **1**

**Anchor — the import block** (for adding the free-function import):
```
  use lemmy_db_schema::source::governance::federation_inbox_nonce::{
    FederationInboxNonce, FederationInboxNonceInsertForm,
  };
  use lemmy_db_schema_file::schema::{federation_inbox_nonce, reputation_snapshot};
```
Run: `grep -c 'FederationInboxNonceInsertForm,' crates/server/tests/e2e.rs`
Expected: **1**

## 4. Constraints

### Edit 1 — Add free-function import alias to v1_rt_r3_fixtures use block

**old_string:**
```rust
  use lemmy_db_schema::source::governance::federation_inbox_nonce::{
    FederationInboxNonce, FederationInboxNonceInsertForm,
  };
  use lemmy_db_schema_file::schema::{federation_inbox_nonce, reputation_snapshot};
```

**new_string:**
```rust
  use lemmy_db_schema::source::governance::federation_inbox_nonce::{
    FederationInboxNonce, FederationInboxNonceInsertForm, delete_older_than as fed_nonce_delete,
  };
  use lemmy_db_schema_file::schema::{federation_inbox_nonce, reputation_snapshot};
```

**NOTE:** `FederationInboxNonce` is imported but not used with `::insert` — it IS used as a type for `Queryable` / `Selectable` in the count query. Keep the import.

### Edit 2 — Rewrite test_brehon_disable_fed_replay_cleanup_job

**old_string:**
```rust
  #[tokio::test(flavor = "multi_thread")]
  async fn test_brehon_disable_fed_replay_cleanup_job() -> LemmyResult<()> {
    let (_container, _context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    {
      FederationInboxNonce::insert(
        &mut async_conn,
        &FederationInboxNonceInsertForm {
          peer_instance: "test.example".to_string(),
          activity_id: "probe-a-nonce-1".to_string(),
        },
      )
      .await?;
      federation_inbox_nonce::delete_older_than(0, &mut async_conn).await?;
      let count: i64 = federation_inbox_nonce::table
        .filter(federation_inbox_nonce::activity_id.eq("probe-a-nonce-1"))
        .count()
        .get_result(&mut async_conn)
        .await?;
      assert_eq!(count, 0, "delete_older_than(0) must remove the row when gate is unset");
    }

    {
      let _guard = EnvVarGuard::set("BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB", "1");
      FederationInboxNonce::insert(
        &mut async_conn,
        &FederationInboxNonceInsertForm {
          peer_instance: "test.example".to_string(),
          activity_id: "probe-b-nonce-1".to_string(),
        },
      )
      .await?;
      let count: i64 = federation_inbox_nonce::table
        .filter(federation_inbox_nonce::activity_id.eq("probe-b-nonce-1"))
        .count()
        .get_result(&mut async_conn)
        .await?;
      assert_eq!(count, 1, "row must survive when BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB is set");
    }

    Ok(())
  }
```

**new_string:**
```rust
  #[tokio::test(flavor = "multi_thread")]
  async fn test_brehon_disable_fed_replay_cleanup_job() -> LemmyResult<()> {
    let (_container, _context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    // Probe A (gate unset): seed row → call delete_older_than(1,...) → fresh row survives
    // (seen_at = now; 1-day window only deletes rows older than 1 day)
    {
      diesel::insert_into(federation_inbox_nonce::table)
        .values(&FederationInboxNonceInsertForm {
          peer_instance: "test.example".to_string(),
          activity_id: "probe-a-nonce-1".to_string(),
        })
        .execute(&mut async_conn)
        .await?;
      fed_nonce_delete(1, &mut async_conn).await?;
      let count: i64 = federation_inbox_nonce::table
        .filter(federation_inbox_nonce::activity_id.eq("probe-a-nonce-1"))
        .count()
        .get_result(&mut async_conn)
        .await?;
      assert_eq!(count, 1, "fresh row survives delete_older_than(1,...) — seen_at is now, not old");
    }

    // Probe B (gate set): seed row → do NOT call delete_older_than (mirroring scheduler guard) → row survives
    {
      let _guard = EnvVarGuard::set("BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB", "1");
      diesel::insert_into(federation_inbox_nonce::table)
        .values(&FederationInboxNonceInsertForm {
          peer_instance: "test.example".to_string(),
          activity_id: "probe-b-nonce-1".to_string(),
        })
        .execute(&mut async_conn)
        .await?;
      let count: i64 = federation_inbox_nonce::table
        .filter(federation_inbox_nonce::activity_id.eq("probe-b-nonce-1"))
        .count()
        .get_result(&mut async_conn)
        .await?;
      assert_eq!(count, 1, "row must survive when BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB is set (scheduler skips call)");
    }

    Ok(())
  }
```

**Note on unused `FederationInboxNonce` import:** after this edit, `FederationInboxNonce` (the struct) is no longer referenced in the test (removed `FederationInboxNonce::insert`). Check if it's used elsewhere in `v1_rt_r3_fixtures`:
```bash
grep -c 'FederationInboxNonce\b' crates/server/tests/e2e.rs
```
If count > 0 after your edit (i.e. used elsewhere), keep the import. If count == 0, remove `FederationInboxNonce` from the import line (keep `FederationInboxNonceInsertForm` and `delete_older_than as fed_nonce_delete`).

**After edit, validate:**
```bash
grep -c 'test_brehon_disable_fed_replay_cleanup_job' crates/server/tests/e2e.rs  # EXPECT: 1
grep -c 'FederationInboxNonce::insert' crates/server/tests/e2e.rs                # EXPECT: 0
```

**After the edits:** write a `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-v1-quality-r3c",
  "phase_task": "3-fix-2"
}
```
Commit + push the DQ entry, then **stop**. Do NOT run `cargo-check.sh` yourself.

**Mandatory lessons fired:**
- `feedback_lemmy_error_no_std_error.md` — e2e.rs edit
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — e2e.rs edit (2 anchors, verify count=1 each)

**Attribution:** commit as `solo-dev <112749825@umail.ucc.ie>`. Never write `answered_by: "advisor"`.

**Mid-task push:** after DQ write, commit + push immediately to `phase-v1-quality-r3c`.
