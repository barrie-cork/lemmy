# Brief: v1-quality-r3c-impl-3

## 1. Role + dispatch line

`[role:impl-task] v1-quality-r3c-impl-3 — BREHON_DISABLE_* guard tests — see .claude/PRPs/briefs/v1-quality-r3c-impl-3.md`

## 2. Scope

**Produce:** two new test functions in `crates/server/tests/e2e.rs` covering the two uncovered `BREHON_DISABLE_*` job guards (Issue addressed by Task 3 in plan §8).

**Exactly one file to edit:** `crates/server/tests/e2e.rs`

**Commit subject:** `test(e2e): add BREHON_DISABLE_SNAPSHOT_JOB + FED_REPLAY_CLEANUP_JOB guard tests`

**Do NOT:**
- Edit `.coderabbit.yaml` (T1 already done), or the Phase A endpoints array (T2 already done)
- Edit any `crates/**` files other than `crates/server/tests/e2e.rs`
- Edit any `migrations/**`, `docs/**`, or `.claude/**` files
- Run `cargo test` with live Docker — write the `validate-pending-laptop` DQ entry and stop
- Add a second `rate_limit.set_config` call (one exists at e2e.rs:4271-4282)

## 3. Required reading

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — new test fns return `LemmyResult<()>` with `?`, not `Result<(), Box<dyn Error>>`; mirror Case A pattern
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish(&db_url)` for direct DB probes
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — uniqueness gate: run `grep -c '<anchor>' crates/server/tests/e2e.rs` = 1 before every Edit
- `.claude/lessons/feedback_rate_limit_debug_config_post_bucket.md` — set_config already present; do NOT add second one
- `.claude/lessons/feedback_envvarguard_fixture_lifetime_footgun.md` — RAII guard must be held in the test body (`let _guard = EnvVarGuard::set(...)`), NOT in a bootstrap helper that returns without the guard
- `.claude/PRPs/plans/v1-quality-r3c.plan.md` — §8 flow design + §10.1 PARTICIPATION_JOB mirror + §10.4 FederationInboxNonce shape + §19 probe B rationale

### §3a. Pre-locate anchors (run BEFORE writing any Edit)

**Insertion anchor** (unique — confirmed count=1 at line 18428 on phase tip):
```
sponsor_allowlist row must be absent after remove");

    Ok(())
  }
}
```

Run: `grep -c 'sponsor_allowlist row must be absent after remove' crates/server/tests/e2e.rs`
Expected: **1**

**Mirror block start** (PARTICIPATION_JOB disable pattern — read lines 17600–17750):
```
BREHON_DISABLE_PARTICIPATION_JOB
```
Run: `grep -n 'BREHON_DISABLE_PARTICIPATION_JOB' crates/server/tests/e2e.rs | head -3`

**Verify guards not yet covered:**
```bash
grep -c 'BREHON_DISABLE_SNAPSHOT_JOB' crates/server/tests/e2e.rs       # EXPECT: 0
grep -c 'BREHON_DISABLE_FED_REPLAY_CLEANUP_JOB' crates/server/tests/e2e.rs  # EXPECT: 0
```

## 4. Constraints

**Test shapes to implement** (mirror `crates/server/tests/e2e.rs:17600-17750` PARTICIPATION_JOB pattern):

### Test 1: `test_brehon_disable_snapshot_job`

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_brehon_disable_snapshot_job() -> LemmyResult<()> {
  let _guard = EnvVarGuard::set("BREHON_DISABLE_SNAPSHOT_JOB", "1");
  let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

  // Count reputation_snapshot rows before
  let before: i64 = reputation_snapshot::table
    .count()
    .get_result(&mut async_conn)
    .await?;

  // Call job function — SNAPSHOT_JOB guard should cause early exit
  reputation_snapshot::run_snapshot_batch(&context).await?;

  // Assert no new rows written
  let after: i64 = reputation_snapshot::table
    .count()
    .get_result(&mut async_conn)
    .await?;
  assert_eq!(before, after, "BREHON_DISABLE_SNAPSHOT_JOB guard must prevent snapshot writes");

  Ok(())
}
```

### Test 2: `test_brehon_disable_fed_replay_cleanup_job`

```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_brehon_disable_fed_replay_cleanup_job() -> LemmyResult<()> {
  let (_container, _context, _federation_context, db_url, _env_guards) = boot_context().await?;
  let mut async_conn = AsyncPgConnection::establish(&db_url).await?;

  // Probe A (gate unset): seed row → delete_older_than removes it
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

  // Probe B (gate set): seed row → do NOT call delete_older_than (mirroring scheduler guard) → row survives
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
    // Do NOT call delete_older_than here — the scheduler guard prevents this call
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

**CRITICAL — FED_REPLAY_CLEANUP probe B rationale:** The guard lives in the scheduler CLOSURE, not inside `delete_older_than` itself. Probe B tests that the scheduler's intent is correct (skipping the call preserves the row) — it does NOT call `delete_older_than` under the guard, because in production the scheduler wouldn't call it either. Per DQ `a3d0e9941441-043` (resolved). Do NOT attempt to test the scheduler closure directly from e2e context.

**CRITICAL — EnvVarGuard lifetime:** Hold `_guard` in the test body with `let _guard = ...`. Do NOT acquire the guard inside a helper function that returns without it — it will drop immediately. Per `feedback_envvarguard_fixture_lifetime_footgun.md`.

**Edit target:** insert the two new test functions BEFORE the final `}` of the file. The full `old_string`:

```rust
    assert!(!still_exists, "sponsor_allowlist row must be absent after remove");

    Ok(())
  }
}
```

The `new_string` adds the two test functions after `Ok(())\n  }` and before the closing `}`:

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

**Validate after edit:**
```bash
grep -c 'test_brehon_disable_snapshot_job' crates/server/tests/e2e.rs       # EXPECT: 1
grep -c 'test_brehon_disable_fed_replay_cleanup_job' crates/server/tests/e2e.rs  # EXPECT: 1
./scripts/brehon/cargo-check.sh --workspace --features full   # DO NOT RUN — write DQ instead
```

**After the edit:** write a `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "from": "impl",
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "branch": "phase-v1-quality-r3c",
  "phase_task": 3
}
```
Commit + push the DQ entry, then **stop**. Do NOT run `cargo-check.sh` yourself.

**Mandatory lessons fired:**
- `feedback_lemmy_error_no_std_error.md` — e2e.rs new test fn
- `feedback_async_pool_test_pattern.md` — e2e.rs new test fn
- `feedback_fix_impl_pre_locate_e2e_anchors.md` — e2e.rs edit
- `feedback_envvarguard_fixture_lifetime_footgun.md` — EnvVarGuard in new test

**Attribution:** commit as `solo-dev <112749825@umail.ucc.ie>`. Never write `answered_by: "advisor"`.

**Mid-task push:** after every DQ write, commit + push immediately to `phase-v1-quality-r3c`.
