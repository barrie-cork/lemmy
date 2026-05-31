# Brief: v1-quality-r3c-fix-impl-1

## 1. Role + dispatch line

`[role:impl-task] v1-quality-r3c-fix-impl-1 — fix T3 compile: move DISABLE_* tests to v1_rt_r3_fixtures + add missing imports — see .claude/PRPs/briefs/v1-quality-r3c-fix-impl-1.md`

## 2. Scope

**Root cause:** T3 added `test_brehon_disable_snapshot_job` + `test_brehon_disable_fed_replay_cleanup_job` into `v1_rt_r4_fixtures`, but both tests call `boot_context()` (defined in `v1_rt_r3_fixtures` only) and reference `reputation_snapshot::table`, `reputation_snapshot::run_snapshot_batch`, `federation_inbox_nonce::table`, `FederationInboxNonce`, `FederationInboxNonceInsertForm` — none of which are imported in r4.

**Fix:** 3 edits to `crates/server/tests/e2e.rs`:
1. Add missing `use` statements to `v1_rt_r3_fixtures`'s import block
2. Remove the two test fns from `v1_rt_r4_fixtures` (revert the T3 insertion there)
3. Insert the two test fns into `v1_rt_r3_fixtures` (before its closing `}`)

**Commit subject:** `fix(e2e): move DISABLE_* guard tests to v1_rt_r3_fixtures + add missing imports`

**Exactly one file to edit:** `crates/server/tests/e2e.rs`

**Do NOT:**
- Edit any other `crates/**` files
- Edit any `migrations/**`, `docs/**`, or `.claude/**` files
- Run `cargo test` — write the `validate-pending-laptop` DQ entry and stop

## 3. Required reading

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — LemmyResult<()> + ? pattern
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — verify each anchor count=1 BEFORE Edit
- `.claude/PRPs/plans/v1-quality-r3c.plan.md` — §8 test design context

### §3a. Pre-locate anchors (run BEFORE writing any Edit)

**Anchor 1** — import insertion in v1_rt_r3_fixtures (count=1 confirmed):
```
  use lemmy_db_schema::newtypes::ModerationCaseId;
  use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
  use reqwest_middleware::ClientBuilder;

  /// Seed one person/local_user pair. Mirror governance_fixtures::seed_user
```
Run: `grep -c 'use reqwest_middleware::ClientBuilder;' crates/server/tests/e2e.rs | head -1`
NOTE: this line is not unique alone — use the 5-line block above as the old_string.

**Anchor 2** — remove tests from v1_rt_r4_fixtures (count=1 confirmed):
```
sponsor_allowlist row must be absent after remove");
```
Run: `grep -c 'sponsor_allowlist row must be absent after remove' crates/server/tests/e2e.rs`
Expected: **1**

**Anchor 3** — insert tests into v1_rt_r3_fixtures (count=1 confirmed):
```
1 evidence_cited row for reporter with delta == DEFAULT_DELTAS_EVIDENCE_CITED
```
Run: `grep -c 'evidence_cited row for reporter with delta' crates/server/tests/e2e.rs`
Expected: **1**

## 4. Constraints

### Edit 1 — Add imports to v1_rt_r3_fixtures

**old_string:**
```rust
  use lemmy_db_schema::newtypes::ModerationCaseId;
  use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
  use reqwest_middleware::ClientBuilder;

  /// Seed one person/local_user pair. Mirror governance_fixtures::seed_user
```

**new_string:**
```rust
  use lemmy_db_schema::newtypes::ModerationCaseId;
  use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
  use reqwest_middleware::ClientBuilder;
  use lemmy_api::governance::reputation_snapshot::run_snapshot_batch;
  use lemmy_db_schema::source::governance::federation_inbox_nonce::{
    FederationInboxNonce, FederationInboxNonceInsertForm,
  };
  use lemmy_db_schema_file::schema::{federation_inbox_nonce, reputation_snapshot};

  /// Seed one person/local_user pair. Mirror governance_fixtures::seed_user
```

### Edit 2 — Remove tests from v1_rt_r4_fixtures

The entire two-test block (from `  #[tokio::test` after the sponsor_allowlist `Ok(())` through the final `}` before the module closing `}`) must be replaced with just the closing `}`.

**old_string:**
```rust
    assert!(!still_exists, "sponsor_allowlist row must be absent after remove");

    Ok(())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn test_brehon_disable_snapshot_job() -> LemmyResult<()> {
    let _guard = EnvVarGuard::set("BREHON_DISABLE_SNAPSHOT_JOB", "1");
    let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    let before: i64 = reputation_snapshot::table
      .count()
      .get_result(&mut async_conn)
      .await?;

    reputation_snapshot::run_snapshot_batch(&context).await?;

    let after: i64 = reputation_snapshot::table
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(before, after, "BREHON_DISABLE_SNAPSHOT_JOB guard must prevent snapshot writes");

    Ok(())
  }

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
}
```

**new_string:**
```rust
    assert!(!still_exists, "sponsor_allowlist row must be absent after remove");

    Ok(())
  }
}
```

### Edit 3 — Insert tests into v1_rt_r3_fixtures

**old_string:**
```rust
      assert_eq!(
      evidence_cited_rows, 1,
      "1 evidence_cited row for reporter with delta == DEFAULT_DELTAS_EVIDENCE_CITED"
    );
    Ok(())
  }
}

mod v1_rt_r4_fixtures {
```

**new_string:**
```rust
      assert_eq!(
      evidence_cited_rows, 1,
      "1 evidence_cited row for reporter with delta == DEFAULT_DELTAS_EVIDENCE_CITED"
    );
    Ok(())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn test_brehon_disable_snapshot_job() -> LemmyResult<()> {
    let _guard = EnvVarGuard::set("BREHON_DISABLE_SNAPSHOT_JOB", "1");
    let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

    let before: i64 = reputation_snapshot::table
      .count()
      .get_result(&mut async_conn)
      .await?;

    run_snapshot_batch(&context).await?;

    let after: i64 = reputation_snapshot::table
      .count()
      .get_result(&mut async_conn)
      .await?;
    assert_eq!(before, after, "BREHON_DISABLE_SNAPSHOT_JOB guard must prevent snapshot writes");

    Ok(())
  }

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
}

mod v1_rt_r4_fixtures {
```

### Import path verification

Before using any import path, verify with grep:
- `run_snapshot_batch`: `grep -rn "pub async fn run_snapshot_batch" crates/` → `crates/api/api/src/governance/reputation_snapshot.rs:489`
  - Import path: `lemmy_api::governance::reputation_snapshot::run_snapshot_batch`
- `FederationInboxNonce`, `FederationInboxNonceInsertForm`: `grep -rn "pub struct FederationInboxNonce\b" crates/` → `crates/db_schema/src/source/governance/federation_inbox_nonce.rs`
  - Import path: `lemmy_db_schema::source::governance::federation_inbox_nonce::{FederationInboxNonce, FederationInboxNonceInsertForm}`
- `federation_inbox_nonce` schema + `reputation_snapshot` schema: both in `lemmy_db_schema_file::schema`

**Note on `run_snapshot_batch` call site:** In Edit 3, the fn is called as `run_snapshot_batch(&context).await?` (imported directly), NOT as `reputation_snapshot::run_snapshot_batch(&context).await?` — `reputation_snapshot` as a schema module has no `run_snapshot_batch` function.

**After all 3 edits, validate:**
```bash
grep -c 'test_brehon_disable_snapshot_job' crates/server/tests/e2e.rs       # EXPECT: 1
grep -c 'test_brehon_disable_fed_replay_cleanup_job' crates/server/tests/e2e.rs  # EXPECT: 1
```

**After the edits:** write a `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-v1-quality-r3c",
  "phase_task": "3-fix"
}
```
Commit + push the DQ entry, then **stop**. Do NOT run `cargo-check.sh` yourself.

**Mandatory lessons fired:**
- `feedback_lemmy_error_no_std_error.md` — e2e.rs edit
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — e2e.rs edit (3 anchors, all count=1 pre-verified)

**Attribution:** commit as `solo-dev <112749825@umail.ucc.ie>`. Never write `answered_by: "advisor"`.

**Mid-task push:** after DQ write, commit + push immediately to `phase-v1-quality-r3c`.
