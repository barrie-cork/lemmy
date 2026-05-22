# Plan: v1-federation-inbound-e — TOCTOU fix in `evict_oldest_unreviewed_if_needed` (advisory-lock + same-tx consolidation)

## 1. Summary

This sub-phase closes the Time-of-Check-Time-of-Use race in `evict_oldest_unreviewed_if_needed` at `crates/apub/activities/src/governance/inbox.rs:646-703`. Today the function runs `SELECT COUNT(*)` outside any transaction, then optionally fires a separate transaction that DELETEs the oldest unreviewed row + INSERTs a drop-log + APPENDs a `governance_log` entry. Each caller (`receive_remote_sanction_notice`, `receive_remote_trust_attestation`, `receive_remote_moderation_label`) then runs ITS OWN second transaction to INSERT the new inbound row. Under concurrent inbound from the same `peer_domain` the read-then-write sequence races on TWO axes: (a) two concurrent evictors both observe `count == cap` and both delete a row (over-eviction); (b) caller B reads `count < cap` between caller A's evict-commit and A's insert-commit, so B skips eviction, then both insert, leaving `count == cap + 1` (over-cap). Acceptance: after Tasks 1+2 land, the COUNT-DELETE-INSERT-LOG sequence is serialised per `(peer_domain, table_name)` via `pg_advisory_xact_lock` acquired at the top of a single consolidated transaction, and a multi-thread e2e probe asserts that 8 concurrent inbound activities against a peer pinned at `cap = 5` leave the table at exactly `count == cap` (no over-cap, no over-eviction visible through the drop-log row count). Zero migrations, one helper-fn addition, three call-site refactors, one new e2e fixtures module.

## 2. Source

- `.claude/PRPs/handovers/v1-federation-inbound-e-bootstrap.md` @ `01ddb785a` (advisor handoff; primary scope source — the per-task-419 brief at `.claude/PRPs/briefs/v1-federation-inbound-e-planning-1.md` was NOT authored before this planning Junior dispatched; the bootstrap document carries the brief-equivalent §0/§4 watchpoints + scope walls. Filed as `kind: "log"` DQ at plan commit time per `feedback_junior_pmd_write_convention.md`.)
- `.claude/PRPs/plans/v1-federation-inbound-d.plan.md` §12 item #3 + §13 Task 3 Mandatory carry-forward — TOCTOU fix explicitly deferred from fed-in-d with the gate-1 question naming `atomic-SQL-with-RETURNING vs SELECT FOR UPDATE SKIP LOCKED`; this plan adds a third option (`pg_advisory_xact_lock` over a consolidated tx) and recommends it.
- `crates/apub/activities/src/governance/inbox.rs:188-228` (Caller 1 — `receive_remote_sanction_notice`)
- `crates/apub/activities/src/governance/inbox.rs:300-339` (Caller 2 — `receive_remote_trust_attestation`)
- `crates/apub/activities/src/governance/inbox.rs:750-799` (Caller 3 — `receive_remote_moderation_label`)
- `crates/apub/activities/src/governance/inbox.rs:634-703` (`CountRow` struct + `evict_oldest_unreviewed_if_needed` function — Task 1 fix site)
- `crates/api/api/src/governance/reputation_snapshot.rs:821-848` (`acquire_advisory_xact_lock` — MIRROR ref for Task 1's lock helper)
- `crates/api/api/src/governance/appeal_window_expiry.rs:1-87` (FOR UPDATE SKIP LOCKED canonical pattern — read so the planner can document why SKIP LOCKED is NOT the chosen path here)
- `crates/diesel_utils/src/connection.rs:60-73` (`DbConn::run_transaction` signature — needed to refactor evict into the caller's existing `run_transaction` block)
- `crates/server/tests/e2e.rs:15797-15891` (`v1_federation_inbound_b_fixtures::bootstrap_with_peer` + `blocklisted_peer_returns_403` — MIRROR ref for Task 2's fixtures-module shape, LemmyResult signature, and testcontainer pattern)
- DQ `<this-plan>` (planner-recommended gate-1 question on concurrency-model choice — filed at plan-commit as `from: "planner"`, `answered_by: "planner"`, pending advisor validation)

Lessons that bind decisions (cited where they fire):

- `.claude/lessons/feedback_pg_advisory_xact_lock_void_decode.md` — `.execute()` not `.load()` for `pg_advisory_xact_lock` (Task 1)
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — multi-write handler atomicity rationale
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — Task 2 e2e.rs edit budget (≤200 lines added)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A canonical: mirror `v1_federation_inbound_b_fixtures` sibling verbatim
- `.claude/lessons/feedback_async_pool_test_pattern.md` — testcontainer + `AsyncPgConnection::establish(&db_url)` pattern for Task 2
- `.claude/lessons/feedback_features_full_workspace_only.md` + `feedback_features_full_p_crate_incompatible.md`
- `.claude/lessons/feedback_clippy_test_style.md` — no `unwrap`/`expect` in test bodies
- `.claude/lessons/feedback_complexity_score_pre_split.md` — §5.1 score
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — §4 watchpoints name specific file/line
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML per task
- `.claude/lessons/feedback_cohort_validation_dependency_check.md` — `requires:` discipline
- `.claude/lessons/feedback_phase_lane_worktree_bootstrap_checklist.md`
- `.claude/lessons/feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`
- `.claude/lessons/feedback_pre_phase_dod_smoke_test.md` + `feedback_plan_dod_dry_run_at_write.md`
- `.claude/lessons/feedback_wrapper_script_flag_silence.md` — Task 0 Probes 1-4 rationale
- `.claude/lessons/feedback_brehon_verify_pre_merge.md` — §16a stories drive `/brehon-verify`
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md` — Phase-2 e2e invocation discipline
- `.claude/lessons/feedback_read_canonical_before_writing_spec.md`
- `.claude/lessons/feedback_principles_not_rules.md`
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md`
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md`

ADRs: none modified. The fix is a pure concurrency-correctness change. ADR-006 (advisory-only inbound persistence) is preserved by widening the atomic boundary, not breached — the consolidated transaction emits drop-log + drop's governance_log entry + receipt's INSERT + receipt's governance_log entry. The drop-log + the receipt INSERT have always been intended to land together; this plan makes that intent explicit. ADR-013, ADR-014, ADR-015 are untouched.

## 3. Problem statement

`evict_oldest_unreviewed_if_needed` (at `crates/apub/activities/src/governance/inbox.rs:646-703`) is called from three inbound governance receivers — `receive_remote_sanction_notice` (line 191), `receive_remote_trust_attestation` (line 305), and `receive_remote_moderation_label` (line 761) — each on the path that persists an advisory row from a federated peer. The function runs `SELECT COUNT(*)` outside any transaction, then conditionally fires a separate transaction that DELETEs the oldest row + INSERTs a drop-log + APPENDs a `governance_log` entry. The caller then runs ITS OWN second transaction to INSERT the new inbound row. So a single inbound activity touches TWO transactions:

1. **TX1 (inside `evict_oldest_unreviewed_if_needed`)** — DELETE oldest + INSERT drop-log + APPEND `federation_inbound_dropped_storage_cap_evicted`.
2. **TX2 (inside the receiver)** — INSERT new inbound row + APPEND `federation_*_received`.

Between TX1's commit and TX2's commit, the `(peer_domain, table_name)` set's row count is `cap - 1` (if eviction fired) or `cap` (if not). The COUNT check at TX1's start has no lock — Postgres' READ COMMITTED isolation re-evaluates each statement against the latest committed state, so two concurrent callers' COUNT reads race against each other AND against TX2's INSERT commit.

### Race A — over-eviction (two callers both delete)

Two callers A and B arrive concurrently. Both see `count == cap` (neither's TX1 has committed yet). Both enter TX1: A deletes one oldest row + writes drop-log; B deletes the NOW-oldest row + writes drop-log. A's TX2 inserts new row; B's TX2 inserts new row. Net count is `cap`, but TWO drop-log rows landed when only ONE should have. Admin reviewing the drop-log sees a spurious eviction event; an admin-recoverable row that was within the cap is gone.

### Race B — over-cap (B reads stale and skips eviction)

Caller A enters TX1, commits (count drops `cap → cap - 1`). Caller B's COUNT reads `count == cap - 1`. B skips eviction. A's TX2 inserts (count `cap - 1 → cap`). B's TX2 inserts (count `cap → cap + 1`). Net count is `cap + 1` — the table's per-peer cap is breached by 1. Under sustained inbound burst, the breach scales linearly with concurrency.

### Why the DELETE-subquery doesn't help

The DELETE itself uses `DELETE FROM {table_name} WHERE id = (SELECT id ... ORDER BY received_at ASC LIMIT 1)`. Under READ COMMITTED, the subquery is evaluated once at statement-start. Without a `FOR UPDATE` clause on the inner SELECT, two concurrent DELETEs can pick the same oldest row → only one succeeds (the other's WHERE-clause matches zero rows). Adding `FOR UPDATE` to the subquery without addressing the OUTER COUNT race only narrows Race A — Race B still applies because B's COUNT can see post-A-DELETE state.

### Concurrency surface in production

Inbound federation activities from one peer arrive via HTTP POST to `/inbox`. Lemmy's actix-web inbox handler spawns each activity on its own task; the per-peer rate-limit Gate 4 admits multiple concurrent receivers from a single allowlisted peer up to `federation.inbound.per_peer_rate_limit` per hour (e.g. 100/hour). 2-10 concurrent receivers from the same `peer_domain` is realistic — exactly the regime where Races A and B fire.

The fix consolidates TX1 + TX2 into a single transaction and acquires `pg_advisory_xact_lock(key(peer_domain, table_name))` at the top of that transaction. The lock is held until the consolidated transaction commits or rolls back; concurrent callers for the SAME `(peer_domain, table_name)` pair wait, then re-execute the COUNT against post-commit state. Per the v0 simplification list in CLAUDE.md, this stays "solo-dev stack" — `pg_advisory_xact_lock` is already wired at `reputation_snapshot.rs:843`.

## 4. Solution statement

Three changes land in one file (Task 1) + one new test module (Task 2).

### 4.1 New helper `acquire_evict_lock` (in `inbox.rs`)

```rust
async fn acquire_evict_lock(
  conn: &mut AsyncPgConnection,
  peer_domain: &str,
  table_name: &str,
) -> LemmyResult<()> {
  let key_input = format!("{peer_domain}\x00{table_name}");
  diesel::sql_query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
    .bind::<diesel::sql_types::Text, _>(key_input)
    .execute(conn)
    .await?;
  Ok(())
}
```

Lock key derivation uses Postgres' `hashtextextended(text, bigint) → bigint` for stability across process restarts and across the three callers. The `\x00` separator between `peer_domain` and `table_name` is unambiguous.

### 4.2 Rename `evict_oldest_unreviewed_if_needed` → `evict_oldest_unreviewed_if_needed_in_tx`

Takes `&mut AsyncPgConnection` (the live transaction's connection handle), assumes it is being called inside a transaction the caller already owns, and runs the COUNT + DELETE + drop-log INSERT + drop-log governance_log::append sequentially against the passed conn. No `run_transaction` inside — caller wraps everything.

### 4.3 Refactor the 3 call sites to consolidate evict + insert into ONE transaction with the lock

All three receivers collapse their two-transaction shape into one. The pre-tx work (decode, peer_domain extraction, evict_cap read) stays as-is. The inner `conn.run_transaction(|conn| async move { ... })` block now runs in this order:

```rust
let outcome = conn
  .run_transaction(|conn| {
    async move {
      acquire_evict_lock(conn, &source_instance, "remote_sanction_notice").await?;
      evict_oldest_unreviewed_if_needed_in_tx(
        &source_instance,
        "remote_sanction_notice",
        evict_cap,
        conn,
      )
      .await?;
      insert_remote_sanction_notice(&form, conn).await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
        payload,
        None,
      )
      .await?;
      Ok(())
    }
    .scope_boxed()
  })
  .await;
```

The two trust-attestation + moderation-label receivers receive the same shape with their respective `table_name` + INSERT-form + ENTRY_KIND constants.

### 4.4 Properties of this shape

1. **Single atomic boundary.** Eviction's COUNT, eviction's DELETE+log, and the new-row's INSERT+log all share one transaction.
2. **Lock granularity.** `(peer_domain, table_name)` — different peer_domains do not contend. Different table_names for the same peer_domain do not contend.
3. **Lock release.** `pg_advisory_xact_lock` releases automatically on commit OR rollback. No `pg_advisory_unlock` needed.
4. **No protocol surface change.** Function name + visibility change inside the same crate. No public API moves. No struct field added. No ADR change.
5. **ADR-006 invariant preserved by widening the atomic boundary.** The drop-log pair + the receipt pair both fire in one tx on the eviction path; only the receipt pair fires on the non-eviction path.

### 4.5 Why the bootstrap's two alternatives were not chosen

- **Atomic-SQL-with-RETURNING (multi-CTE)** — under READ COMMITTED the inline COUNT subquery is still racey across concurrent statements; SQL complexity grows. Not chosen.
- **`SELECT FOR UPDATE SKIP LOCKED`** — locks the row being deleted but does NOT serialise the COUNT-vs-INSERT-new race. Two concurrent callers each lock a different oldest row (the first locks its row + SKIPS to next-oldest for the second) → BOTH evict → over-eviction. Not chosen.
- **`pg_advisory_xact_lock` keyed on `(peer_domain, table_name)`** — chosen. Serialises the entire critical section. Uses an already-wired Brehon pattern (`reputation_snapshot::acquire_advisory_xact_lock`).

The planner-recommended choice is enumerated as a `kind: "blocker"` DQ (`from: "planner"`, `answered_by: "planner"` with rationale).

## 5. Metadata

- **Phase:** `v1-federation-inbound-e`
- **Branch:** `phase-v1-federation-inbound-e`
- **Target impl-task model:** `sonnet-4-6`
- **Estimated tasks:** 4 (Task 0 pre-flight + Task 1 fix + Task 2 e2e + Task 3 retro)
- **Estimated cargo budget:** N/A (validate-pending-laptop — Shape G SUSPENDED per DQ #229 until 2026-06-01)
- **Forbidden-window applicability:** standard
- **Complexity score:** **5/10** — below sonnet threshold of `> 8`; no split-DQ.

### 5.1 Complexity factor breakdown

| Factor | Weight | This plan | Notes |
|---|---|---|---|
| §13 impl tasks above 5 | +1 each | 0 | 2 impl tasks (Task 1 fix + Task 2 e2e) |
| Migrations touched | +2 each | 0 | Zero migrations (PRECON-4) |
| Crates touched | +1 each | 2 | `crates/apub/activities/` (Task 1) + `crates/server/` (Task 2) |
| `crates/server/tests/e2e.rs` edits | +3 each | 1 | Task 2 adds one new fixtures module |
| New ADR-affecting decisions | +2 each | 0 | None |
| Cargo budget peak above 6 GB | +1 per GB | 0 | Laptop runs |
| **Total** | — | **5/10** | Threshold `> 8` not crossed |

### 5.2 Per-task complexity ceiling

Not applicable — target is `sonnet-4-6`. Tasks 1+2 each modify one file in one crate; `[P]` cohort-compatible by file-set BUT Task 2 has `requires: [1]`. Serial in practice.

## 6. Relationship to other v1-federation-inbound-* sub-phases

- **Depends on:** v1-federation-inbound-a (`ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED` registered) — provides drop-log infra.
- **Depends on:** v1-federation-inbound-b (PR #139) — provides the storage-cap mechanism whose TOCTOU this plan closes; also provides the canonical `mod v1_federation_inbound_b_fixtures` shape.
- **Depends on:** v1-federation-inbound-c — reader-side `.order_by(valid_from.desc())`; unchanged here.
- **Depends on:** v1-federation-inbound-d (PR #146, `897d3f72d`) — per-actor in-memory rate-map bound at `publish_trust_attestation.rs:158-167`. Task 1 here does NOT touch that surface.
- **Carries forward to:** post-pilot follow-ups (per-peer in-memory bound, SHA-256 nonce, governance_config knob, Postgres-backed counters). None in scope.

## 7. Preflight guardrails inherited from prior phases

- **R1:** every `i32 ↔ i64` comparison uses `i64::from(...)`. The new code has no new int comparisons.
- **R5:** Task 0 enumerates ALL probes explicitly (Probes 0-9).
- **R6:** all clippy invocations use `--workspace --features full --no-deps -- -D warnings`.
- **R7:** Task 1 changes a private function's signature; Task 2 adds a new test module. R7 `cargo test --no-run` covers both.
- **R-laptop-cargo:** validate-pending-laptop DoD; cargo never runs on EliteDesk daemon.
- **R-windows-e2e:** bat wrapper invocation per `feedback_windows_e2e_requires_bat_wrapper.md`.
- **R-no-cohort:** Task 2 has `requires: [1]`; serial dispatch.
- **R-daemon-stale-ref:** PRE-PUSH MANDATE on every BM-verb brief.

## 8. Flow design

**Before Task 1 (the defect):** caller P1 invokes `receive_remote_sanction_notice` for an inbound from peer `attacker.test`. Concurrent caller P2 invokes the same function for a second inbound ~milliseconds later. Both observe `count == cap` → both enter TX1 → both delete a row → both insert a new row in their own TX2. Net: count stays at cap but drop-log has TWO rows for the same logical eviction. In the Race-B variant, P2 reads count BETWEEN P1's TX1 commit and TX2 commit → P2 skips evict → final count == cap + 1.

**After Task 1 (the fix):** P1's transaction acquires the advisory lock keyed on `("attacker.test", "remote_sanction_notice")`. P2 blocks at lock acquisition. P1's COUNT == cap → DELETE oldest → INSERT drop-log → APPEND drop-event → INSERT new sanction row → APPEND received-event → COMMIT (lock released). P2 acquires lock → COUNT == cap → DELETE oldest → INSERT drop-log → APPEND drop-event → INSERT new sanction row → APPEND received-event → COMMIT. Both completed; count == cap (correct); drop-log has 2 entries for 2 legitimate evictions.

The lock granularity matters: P3 inbound for `friendly.test` (different peer) acquires its own lock instantly. P4 inbound for `attacker.test` in `remote_moderation_label` (different table) acquires its own lock instantly. Only the SAME `(peer, table)` queue serialises.

## 9. Mandatory reading

### Schema / type definitions

- **`crates/apub/activities/src/governance/inbox.rs:634-640`** — `CountRow` struct definition.
- **`crates/apub/activities/src/governance/inbox.rs:646-703`** — current `evict_oldest_unreviewed_if_needed` body (Task 1 fix site).
- **`crates/apub/activities/src/governance/inbox.rs:128-228`** — `receive_remote_sanction_notice` (Caller 1; refactor lines 188-228).
- **`crates/apub/activities/src/governance/inbox.rs:244-339`** — `receive_remote_trust_attestation` (Caller 2; refactor lines 300-339).
- **`crates/apub/activities/src/governance/inbox.rs:722-?`** — `receive_remote_moderation_label` (Caller 3; refactor the bottom half).

### Existing patterns (MIRROR refs)

- **`crates/api/api/src/governance/reputation_snapshot.rs:821-848`** — `acquire_advisory_xact_lock`. MIRROR ref for Task 1's `acquire_evict_lock`.
- **`crates/api/api/src/governance/appeal_window_expiry.rs:1-87`** — FOR UPDATE SKIP LOCKED canonical pattern. READ-ONLY (explains why NOT chosen).
- **`crates/diesel_utils/src/connection.rs:60-73`** — `DbConn::run_transaction` signature.
- **`crates/server/tests/e2e.rs:15797-15891`** — `mod v1_federation_inbound_b_fixtures`. MIRROR ref for Task 2.

### Lessons + rules

- `.claude/rules/governance-log-entry-kind-registry.md` — confirms `_DROPPED_STORAGE_CAP_EVICTED` is shipped; Task 1 does NOT add a new entry kind. `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` MUST equal `55`.
- `.claude/rules/cargo-output-capture.md` + `no-cargo-output-paste.md`.
- `.claude/rules/advisor-orchestrator.md` §3.1.1 — conformance-audit prevention checkpoint for Task 1 brief.

## 10. Patterns to mirror

### 10.1 `acquire_evict_lock` — MIRROR `reputation_snapshot::acquire_advisory_xact_lock`

```rust
async fn acquire_evict_lock(
  conn: &mut AsyncPgConnection,
  peer_domain: &str,
  table_name: &str,
) -> LemmyResult<()> {
  let key_input = format!("{peer_domain}\x00{table_name}");
  diesel::sql_query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
    .bind::<diesel::sql_types::Text, _>(key_input)
    .execute(conn)
    .await?;
  Ok(())
}
```

**GOTCHA:**

- `.execute()` NOT `.load()` per `feedback_pg_advisory_xact_lock_void_decode.md`: `pg_advisory_xact_lock` returns `void`; `.load` would panic.
- `hashtextextended(text, bigint)` requires a `BigInt` second argument; hard-code `0`.
- Takes `&mut AsyncPgConnection` (not `&mut DbConn<'_>`).
- `pg_advisory_xact_lock` is blocking at the connection level; wait times bounded by Gate-4's per-peer rate cap upstream.

### 10.2 `evict_oldest_unreviewed_if_needed_in_tx` — MIRROR existing function body

```rust
async fn evict_oldest_unreviewed_if_needed_in_tx(
  peer_domain: &str,
  table_name: &str,
  cap: i64,
  conn: &mut AsyncPgConnection,
) -> LemmyResult<()> {
  let count_sql = format!(
    "SELECT COUNT(*)::bigint AS count FROM {table_name} \
     WHERE source_instance = $1 AND admin_reviewed_at IS NULL"
  );
  let count_row = diesel::sql_query(count_sql)
    .bind::<diesel::sql_types::Text, _>(peer_domain)
    .get_result::<CountRow>(conn)
    .await?;
  if count_row.count < cap {
    return Ok(());
  }
  let delete_sql = format!(
    "DELETE FROM {table_name} WHERE id = \
     (SELECT id FROM {table_name} WHERE source_instance = $1 \
      AND admin_reviewed_at IS NULL ORDER BY received_at ASC LIMIT 1)"
  );
  let form = FederationInboxDroppedLogInsertForm {
    source_instance: peer_domain.to_string(),
    activity_id: None,
    drop_reason: "storage_cap_evicted".to_string(),
    payload_excerpt: None,
  };
  let payload = json!({
    "peer_domain": peer_domain,
    "table": table_name,
    "reason": "storage_cap_evicted",
  });
  diesel::sql_query(delete_sql)
    .bind::<diesel::sql_types::Text, _>(peer_domain)
    .execute(conn)
    .await?;
  diesel::insert_into(federation_inbox_dropped_log::table)
    .values(&form)
    .execute(conn)
    .await?;
  governance_log::append(
    &mut (&mut *conn).into(),
    ENTRY_KIND_FEDERATION_INBOUND_DROPPED_STORAGE_CAP_EVICTED,
    payload,
    None,
  )
  .await?;
  Ok(())
}
```

**GOTCHA:**

- Conn type changed from `&mut DbConn<'_>` to `&mut AsyncPgConnection` (caller owns the tx now).
- `governance_log::append` takes `&mut DbPool<'_>`; the conversion `&mut (&mut *conn).into()` works via existing `From<&mut AsyncPgConnection> for DbPool<'_>` impl.
- No best-effort `persist_failed` log inside this function — caller owns that.

### 10.3 Caller refactor — collapse two tx into one

Pre-fix (showing `receive_remote_sanction_notice`):

```rust
let pool = &mut context.pool();
let conn = &mut get_conn(pool).await?;
evict_oldest_unreviewed_if_needed(&source_instance, "remote_sanction_notice", evict_cap, conn).await?;
let outcome = conn
  .run_transaction(|conn| {
    async move {
      insert_remote_sanction_notice(&form, conn).await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
        payload,
        None,
      )
      .await?;
      Ok(())
    }
    .scope_boxed()
  })
  .await;
```

Post-fix:

```rust
let pool = &mut context.pool();
let conn = &mut get_conn(pool).await?;
let outcome = conn
  .run_transaction(|conn| {
    async move {
      acquire_evict_lock(conn, &source_instance, "remote_sanction_notice").await?;
      evict_oldest_unreviewed_if_needed_in_tx(
        &source_instance,
        "remote_sanction_notice",
        evict_cap,
        conn,
      )
      .await?;
      insert_remote_sanction_notice(&form, conn).await?;
      governance_log::append(
        &mut (&mut *conn).into(),
        ENTRY_KIND_FEDERATION_SANCTION_RECEIVED,
        payload,
        None,
      )
      .await?;
      Ok(())
    }
    .scope_boxed()
  })
  .await;
```

**GOTCHA:**

- `&source_instance` (the binding from the fn body) borrows for the duration of the closure's awaits. The `async move` closure moves owned values (`form`, `payload`); references stay borrowed from outside.
- `evict_cap: i64` is Copy; moved trivially.
- `table_name` is a `&'static str` literal — inlined.
- The best-effort `persist_failed` emit BELOW the outcome match stays UNCHANGED.

### 10.4 Test fixtures module — MIRROR `v1_federation_inbound_b_fixtures`

`mod v1_federation_inbound_e_fixtures` matches the existing v1_federation_inbound_b_fixtures shape verbatim: `use super::*;` + activitypub_federation imports + `bootstrap_with_peer` async helper + `#[tokio::test(flavor = "multi_thread")]` + `LemmyResult<()>` outer.

**GOTCHA:**

- `tokio::test(flavor = "multi_thread")` is mandatory — default single-threaded runtime masks the race.
- Mandatory file-class lesson injection: `feedback_lemmy_error_no_std_error.md` (Case A) + `feedback_async_pool_test_pattern.md`.
- No `.unwrap()` / `.expect()` shotguns per `feedback_clippy_test_style.md`.
- Edit budget: ~150 lines added.
- `build_minimal_sanction_notice_activity` helper symbol verified at canonical-schema-first read.
- `context.reset_request_count()` or `Data::clone()` equivalent — verified at canonical-schema-first read.

## 11. Files to change

**`crates/apub/activities/`**:

- `crates/apub/activities/src/governance/inbox.rs` — (Task 1) add `acquire_evict_lock` helper; rename `evict_oldest_unreviewed_if_needed` to `evict_oldest_unreviewed_if_needed_in_tx` and change its conn type from `&mut DbConn<'_>` to `&mut AsyncPgConnection`; remove its inner `run_transaction` wrapper; refactor the 3 callers to consolidate evict + insert into a single `run_transaction` block with `acquire_evict_lock` as the first statement. Net diff: ~120 lines added + ~10 lines removed. No new imports.

**`crates/server/tests/`**:

- `crates/server/tests/e2e.rs` — (Task 2) append new `mod v1_federation_inbound_e_fixtures { ... }` after the existing `mod v1_federation_inbound_b_fixtures` block. One new `#[tokio::test(flavor = "multi_thread")]` (`storage_cap_holds_under_concurrent_receivers`) + the module's `bootstrap_with_peer` helper. Net diff: ~150 lines added.

**No other crate edits.** No edit to `crates/api/api/src/governance/`, `crates/db_schema/`, `crates/apub/activities/src/governance/publish_*.rs`, migrations, `Cargo.toml`, or `rust-toolchain.toml`.

**No struct-field add.** **`feedback_fix_impl_enumerate_all_callsites.md` DOES fire** for the function-rename: impl agent runs `rg "evict_oldest_unreviewed_if_needed" crates/` and enumerates the 3 call sites (all in `inbox.rs`).

**No migrations** (PRECON-4). **No new ENTRY_KIND.**

## 12. NOT building in v1-federation-inbound-e

1. **Per-peer in-memory rate-map bound** — fed-in-d user clarify B1 stands. Trigger: post-pilot DoS metrics.
2. **SHA-256 key-hash for per-actor map** — fed-in-d user clarify B3 stands.
3. **`governance_config`-backed `MAX_PER_ACTOR_RATE_ENTRIES`** — option-a deferral.
4. **Postgres-backed rate counters** — option-a deferral.
5. **`[P]` cohort dispatch** — Task 2 has `requires: [1]`; cohort logic falls back to serial.
6. **Helper extraction across `inbox.rs` + `publish_*.rs`** — fed-in-c PRECON-3 binds (circular-dep).
7. **New migration under `migrations/**`** — scope violation per PRECON-4. Catch-fire if proposed.
8. **Shape G workflow change** — SUSPENDED per DQ #229. Trigger: 2026-06-01 re-enable check.
9. **New ENTRY_KIND** — reuse `_DROPPED_STORAGE_CAP_EVICTED`. Catch-fire if new kind proposed.
10. **Alternative concurrency model** — see §4.5. Planner-recommended choice filed as gate-1 DQ.
11. **TX-level isolation upgrade (SERIALIZABLE / REPEATABLE READ)** — too broad.
12. **Timeout on `pg_advisory_xact_lock`** — Gate-4 bounds queue depth upstream.
13. **`#[serial_test]` dev-dep for Task 2** — unique `peer_domain` namespace prevents cross-test interference.
14. **Telemetry / span instrumentation for lock acquisition** — observability work.

---

## 13. Step-by-step tasks

Execute in dependency order. One commit per task. No `[P]` markers (Task 2 has `requires: [1]`; serial dispatch).

> **No cohort dispatch:** advisor dispatches Task 1, waits for finalize-merge, then Task 2, then Task 3.
>
> **Validate-pending-laptop DoD** (Shape-G suspended per DQ #229): each impl-task pushes its worker branch and writes `kind: "validate-pending-laptop"` to `.claude/decision-queue.json` naming §15 commands verbatim. Advisor reads on next poll, runs commands locally, mutates the entry. Cargo never runs on the EliteDesk daemon.

### Task 0: Pre-flight harness audit + branch verification

**Goal:** verify environment is ready for `v1-federation-inbound-e`; confirm branch is `phase-v1-federation-inbound-e`; confirm fed-in-d deliverables are intact; confirm pre-existing clippy baseline is clean.

**FILES:**

```yaml
creates: []
modifies: []
requires: []
```

**Probes (per `.claude/rules/pre-phase-harness-audit.md` — R5).** Run on the canonical laptop checkout (`C:/Users/barri/Developer/brehon-fork-fed-in-e` worktree, post-bm-cut):

```bash
# Probe 0 - Docker daemon
docker ps > /dev/null 2>&1 && echo "DOCKER OK" || { echo "DOCKER NOT RUNNING"; exit 1; }

# Probe 1 - per-crate check honors -p
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_utils > .claude/PRPs/debug/v1-federation-inbound-e-task0-probe1.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task0-probe1.log

# Probe 2 - feature flag activation
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_db_schema --features full > .claude/PRPs/debug/v1-federation-inbound-e-task0-probe2.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task0-probe2.log

# Probe 3 - cargo-test wrapper honors target selection
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server > .claude/PRPs/debug/v1-federation-inbound-e-task0-probe3.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task0-probe3.log

# Probe 4 - exit-code propagation on bogus feature
cmd //c "scripts\\brehon\\cargo-test.bat --test e2e --no-run -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-federation-inbound-e-task0-probe4a.log 2>&1"
echo "cargo-test.bat exit on bogus feature: $?"
cmd //c "scripts\\brehon\\cargo-check.bat -p lemmy_server --features nonexistent_xyz > .claude/PRPs/debug/v1-federation-inbound-e-task0-probe4b.log 2>&1"
echo "cargo-check.bat exit on bogus feature: $?"

# Probe 5 - branch verification
git branch --show-current
# EXPECT: phase-v1-federation-inbound-e
git log governance-v0..HEAD --oneline | wc -l
# EXPECT: 0 (phase branch just cut)

# Probe 6 - fed-in-d deliverable intact
grep -n "const MAX_PER_ACTOR_RATE_ENTRIES" crates/apub/activities/src/governance/publish_trust_attestation.rs
# EXPECT: 1 match (the per-actor in-memory bound shipped by fed-in-d)
grep -n "fn evict_oldest_unreviewed_if_needed" crates/apub/activities/src/governance/inbox.rs
# EXPECT: 1 match at ~line 646 (pre-fix name)
grep -c "evict_oldest_unreviewed_if_needed(" crates/apub/activities/src/governance/inbox.rs
# EXPECT: 3+ matches (3 call sites + definition without ())

# Probe 7 - MIRROR refs present
grep -n "fn acquire_advisory_xact_lock" crates/api/api/src/governance/reputation_snapshot.rs
# EXPECT: 1 match at line ~824
grep -n "FOR UPDATE SKIP LOCKED" crates/api/api/src/governance/appeal_window_expiry.rs
# EXPECT: appears in comment + impl (READ-ONLY)
grep -n "mod v1_federation_inbound_b_fixtures" crates/server/tests/e2e.rs
# EXPECT: 1 match at line ~15797

# Probe 8 - ENTRY_KIND parity baseline 55
rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs
# EXPECT: 55
rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l
# EXPECT: 55

# Probe 9 - workspace clippy baseline
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-e-task0-clippy-baseline.log 2>&1"
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-federation-inbound-e-task0-clippy-baseline.log
```

**EXPECT block:**

- Probes 0..3, 5..9 exit 0.
- Probe 4 (both lines) exits NON-ZERO (negative test).
- Probe 6 outputs the per-actor bound const + the 3 evict call sites.
- Probe 7 outputs all three MIRROR-ref matches.
- Probe 8 outputs `55` for both lines.

**No commit at Task 0.** If any probe fails, file a `kind: "blocker"` DQ (`from: "advisor"`) and STOP.

### Task 1: TOCTOU fix — `acquire_evict_lock` + `evict_oldest_unreviewed_if_needed_in_tx` + 3-caller refactor

**ACTION:** add `acquire_evict_lock` helper at module scope of `inbox.rs`; rename `evict_oldest_unreviewed_if_needed` to `evict_oldest_unreviewed_if_needed_in_tx` and change its conn-arg type from `&mut DbConn<'_>` to `&mut AsyncPgConnection`, dropping the inner `conn.run_transaction(...)` wrapper; refactor the three callers (lines 188-228, 300-339, 750-799) to consolidate evict + insert into a single `conn.run_transaction(...)` block beginning with `acquire_evict_lock(conn, ...)`.

**FILES:**

```yaml
creates: []
modifies:
  - crates/apub/activities/src/governance/inbox.rs
requires: []
```

**IMPLEMENT (file 1 of 1):** in `crates/apub/activities/src/governance/inbox.rs`:

1. **Add `acquire_evict_lock` helper** above the existing `CountRow` declaration (currently at line 634). Verbatim shape from §10.1.

2. **Rename + simplify `evict_oldest_unreviewed_if_needed` → `evict_oldest_unreviewed_if_needed_in_tx`** at lines 646-703. Verbatim shape from §10.2.

3. **Refactor caller 1 — `receive_remote_sanction_notice` at lines 188-228.** Delete the pre-tx `evict_oldest_unreviewed_if_needed(...)` call at line 191. Modify the existing `run_transaction(|conn| async move { ... })` body so it FIRST calls `acquire_evict_lock` + `evict_oldest_unreviewed_if_needed_in_tx`, THEN the existing `insert_remote_sanction_notice` + `governance_log::append(ENTRY_KIND_FEDERATION_SANCTION_RECEIVED, ...)`. Use the verbatim shape from §10.3. The `if let Err(e) = &outcome { ... }` block at lines 211-226 stays UNCHANGED.

4. **Refactor caller 2 — `receive_remote_trust_attestation` at lines 300-339.** Same shape; table name `"federation_attestation"`; INSERT-form `FederationAttestationInsertForm`; receipt's ENTRY_KIND `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED`. Pre-tx work at lines 244-303 unchanged.

5. **Refactor caller 3 — `receive_remote_moderation_label` at lines 750-799.** Same shape; table name `"remote_moderation_label"`; INSERT-form `RemoteModerationLabelInsertForm`; receipt's ENTRY_KIND `ENTRY_KIND_FEDERATION_LABEL_RECEIVED`. The `let trust = federation_inbox_check_peer_trust(&peer_domain, conn).await?;` + `let form = ...` stay outside the consolidated tx.

6. **Do not touch:** peer-trust check at the top of each receiver; the decode_*_object helpers (lines 357-388 + 707-715); the insert helpers (lines 393-415); the `get_inbound_config_int` helper (lines 425-438); `wrap_governance_inbound` (lines 477+); per-actor in-memory bound at `publish_trust_attestation.rs:158-167`.

**MIRROR:** `crates/api/api/src/governance/reputation_snapshot.rs:821-848` for `acquire_evict_lock`; existing `evict_oldest_unreviewed_if_needed` body for `_in_tx`'s body minus the `run_transaction` wrapper.

**GOTCHA:**

- `.execute(conn).await?` NOT `.load::<...>(conn)` for `pg_advisory_xact_lock` per `feedback_pg_advisory_xact_lock_void_decode.md`.
- **Callsite enumeration bound to 3 sites:** `rg "evict_oldest_unreviewed_if_needed\(" crates/` must return exactly 3 hits, all inside `crates/apub/activities/src/governance/inbox.rs`. Brief lists all 3 explicitly.
- **Pre-push `cargo-check.sh` mandatory** per `feedback_fix_impl_pre_push_cargo_check.md`. The rename + conn-arg type change surfaces borrow / move errors at compile time.
- `&source_instance` (or `&peer_domain` for moderation_label) borrows for the closure's awaits. The `async move` closure already moves owned values (`form`, `payload`); references stay borrowed.
- `table_name` is a `&'static str` literal — inlined.
- No new imports. `AsyncPgConnection` at line 55; `diesel::sql_types::Text` implicit via existing usages.
- `pg_advisory_xact_lock` REQUIRES being inside a tx; calling outside is undefined per Postgres docs.
- `hashtextextended($1, 0)` returns bigint — implicit cast to `pg_advisory_xact_lock(bigint)`.
- **Conformance-audit Tier-1 file:** `inbox.rs` is Tier-1 scope. Before authoring the Task-1 impl-task brief, advisor invokes the skill with `target_scope = file crates/apub/activities/src/governance/inbox.rs` per `.claude/rules/advisor-orchestrator.md` §3.1.1.

**VALIDATE (story-checkpoint feeds §16a Story 1):**

Worker pre-push:

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-e-task1-prepush-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task1-prepush-check.log
# EXPECT: exit 0

git diff --stat
# EXPECT: 1 file changed, ~120 insertions(+), ~10 deletions(-)
git add crates/apub/activities/src/governance/inbox.rs
git commit -m "feat(fed-in-e): serialise per-peer storage-cap eviction with pg_advisory_xact_lock (task 1)"
git push origin <worker-branch>
# Worker then writes kind: "validate-pending-laptop" DQ entry naming §15 commands verbatim.
```

Post-push (advisor-laptop):

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-e-task1-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task1-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-e-task1-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task1-clippy.log
# EXPECT: exit 0
```

Advisor mutates the `validate-pending-laptop` DQ entry on success.

### Task 2: e2e regression test — `storage_cap_holds_under_concurrent_receivers`

**ACTION:** append `mod v1_federation_inbound_e_fixtures { ... }` to `crates/server/tests/e2e.rs` after the existing `mod v1_federation_inbound_b_fixtures` block. The module's single new test seeds 5 rows in `remote_sanction_notice` for `concurrent.test`, sets the storage cap to 5, then fans out 8 concurrent `Activity::receive(PublishSanctionNotice, ...)` invocations and asserts the table count holds at 5 (no over-cap) and the drop-log records exactly 8 evictions (no over-eviction).

**FILES:**

```yaml
creates: []
modifies:
  - crates/server/tests/e2e.rs
requires:
  - task: 1
    reason: "Task 2 exercises Task 1's advisory-lock serialisation. The test would either no-op (lock absent → bypass) or fail the count assertions (lock absent → race fires) against pre-Task-1 HEAD."
```

**IMPLEMENT (file 1 of 1):**

At the bottom of `crates/server/tests/e2e.rs` (after `mod v1_federation_inbound_b_fixtures { ... }`), append:

```rust
mod v1_federation_inbound_e_fixtures {
  use super::*;
  use activitypub_federation::config::FederationConfig;
  use activitypub_federation::traits::Activity as ActivityTrait;
  use diesel::{ExpressionMethods, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_apub_activities::protocol::governance::publish_sanction_notice::PublishSanctionNotice;
  use lemmy_db_schema::source::governance::federation_peer::FederationPeerInsertForm;
  use lemmy_db_schema_file::InstanceId;
  use lemmy_db_schema_file::enums::FederationPeerTrust;
  use lemmy_db_schema_file::schema::{
    federation_inbox_dropped_log, federation_peer, governance_config, instance,
    remote_sanction_notice,
  };
  use lemmy_utils::error::LemmyResult;
  use testcontainers::{ContainerAsync, GenericImage};

  async fn bootstrap_with_peer(
    domain: &str,
  ) -> LemmyResult<(
    ContainerAsync<GenericImage>,
    FederationConfig<LemmyContext>,
    String,
    InstanceId,
  )> {
    let (container, actix_context, db_url) = governance_fixtures::bootstrap().await?;
    let federation_config = FederationConfig::builder()
      .domain((**actix_context).settings().hostname.clone())
      .app_data((**actix_context).clone())
      .debug(true)
      .http_fetch_limit(0)
      .build()
      .await?;
    let mut conn = AsyncPgConnection::establish(&db_url).await?;
    let peer_instance_id: i32 = diesel::insert_into(instance::table)
      .values((
        instance::domain.eq(domain),
        instance::published_at.eq(diesel::dsl::now),
      ))
      .returning(instance::id)
      .get_result(&mut conn)
      .await?;
    let form = FederationPeerInsertForm {
      instance_id: InstanceId(peer_instance_id),
      trust_level: Some(FederationPeerTrust::Allowlisted),
      added_by_actor: None,
      notes: None,
    };
    diesel::insert_into(federation_peer::table)
      .values(&form)
      .execute(&mut conn)
      .await?;
    Ok((container, federation_config, db_url, InstanceId(peer_instance_id)))
  }

  async fn seed_storage_cap(conn: &mut AsyncPgConnection, cap: i64) -> LemmyResult<()> {
    diesel::insert_into(governance_config::table)
      .values((
        governance_config::scope.eq("instance"),
        governance_config::key.eq("federation.inbound.per_peer_storage_cap"),
        governance_config::value_int.eq(Some(cap)),
        governance_config::valid_from.eq(diesel::dsl::now),
      ))
      .execute(conn)
      .await?;
    Ok(())
  }

  async fn preseed_sanction_notices(
    conn: &mut AsyncPgConnection,
    source_instance: &str,
    n: usize,
  ) -> LemmyResult<()> {
    for i in 0..n {
      diesel::sql_query(
        "INSERT INTO remote_sanction_notice (source_instance, target_url, action, scope, \
         signature, published_at, received_at) \
         VALUES ($1, $2, 'remove'::sanction_action_type, 'instance'::sanction_scope_type, \
         $3, NOW(), NOW() - ($4 || ' seconds')::interval)",
      )
      .bind::<diesel::sql_types::Text, _>(source_instance)
      .bind::<diesel::sql_types::Text, _>(format!("https://{source_instance}/target/{i}"))
      .bind::<diesel::sql_types::Text, _>(format!("https://{source_instance}/activity/preseed/{i}"))
      .bind::<diesel::sql_types::Text, _>(format!("{}", 100 - i))
      .execute(conn)
      .await?;
    }
    Ok(())
  }

  #[tokio::test(flavor = "multi_thread")]
  async fn storage_cap_holds_under_concurrent_receivers() -> LemmyResult<()> {
    let (_container, fed_cfg, db_url, _peer_id) =
      bootstrap_with_peer("concurrent.test").await?;

    let mut setup_conn = AsyncPgConnection::establish(&db_url).await?;
    seed_storage_cap(&mut setup_conn, 5).await?;
    preseed_sanction_notices(&mut setup_conn, "concurrent.test", 5).await?;

    let context = fed_cfg.to_request_data();

    let mut handles = Vec::with_capacity(8);
    for i in 0..8u32 {
      let context_i = context.reset_request_count();
      let handle = tokio::spawn(async move {
        let activity = build_minimal_sanction_notice_activity(
          &format!("https://concurrent.test/activity/concurrent/{i}"),
          "concurrent.test",
        )?;
        ActivityTrait::receive(activity, &context_i).await
      });
      handles.push(handle);
    }
    for handle in handles {
      handle.await.map_err(|e| {
        lemmy_utils::error::LemmyErrorType::Unknown(format!("join: {e}"))
      })??;
    }

    let mut verify_conn = AsyncPgConnection::establish(&db_url).await?;
    let final_count: i64 = remote_sanction_notice::table
      .filter(remote_sanction_notice::source_instance.eq("concurrent.test"))
      .filter(remote_sanction_notice::admin_reviewed_at.is_null())
      .count()
      .get_result(&mut verify_conn)
      .await?;
    assert_eq!(final_count, 5, "per-peer storage cap must hold under concurrent receivers (Race B regression)");

    let drop_log_count: i64 = federation_inbox_dropped_log::table
      .filter(federation_inbox_dropped_log::source_instance.eq("concurrent.test"))
      .filter(federation_inbox_dropped_log::drop_reason.eq("storage_cap_evicted"))
      .count()
      .get_result(&mut verify_conn)
      .await?;
    assert_eq!(drop_log_count, 8, "each concurrent receiver must fire exactly one eviction event (Race A regression)");

    Ok(())
  }
}
```

The impl agent at canonical-schema-first read time MAY rename helpers to match sibling-convention. The two assertions are the load-bearing claims — those are the regression gate for Race A + Race B.

**MIRROR:** `crates/server/tests/e2e.rs:15797-15891` (`mod v1_federation_inbound_b_fixtures`).

**GOTCHA:**

- `tokio::test(flavor = "multi_thread")` is mandatory — default runtime serialises tasks.
- Mandatory file-class lesson injection: `feedback_lemmy_error_no_std_error.md` (Case A) + `feedback_async_pool_test_pattern.md`.
- No `.unwrap()` / `.expect()` shotguns per `feedback_clippy_test_style.md`.
- Edit budget: ~150 lines added; below the ≤200-line discipline.
- `build_minimal_sanction_notice_activity` helper — impl agent locates the actual symbol at canonical-schema-first read.
- `context.reset_request_count()` — verify at canonical-schema-first read; if API differs, use `Data::clone()` or equivalent.
- Concurrent test isolation: `peer_domain = "concurrent.test"` is unique to this test; testcontainer-per-test pattern prevents cross-test state.
- Conformance-audit does NOT fire for Task 2 (e2e.rs is not Tier-1 scope).
- Pre-push `cargo-check.sh` still runs (any e2e.rs edit can surface unused-import warnings).

**VALIDATE (story-checkpoint feeds §16a Story 2):**

Worker pre-push:

```bash
bash scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-e-task2-prepush-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task2-prepush-check.log
# EXPECT: exit 0

git diff --stat
# EXPECT: 1 file changed, ~150 insertions(+), 0 deletions(-)
git add crates/server/tests/e2e.rs
git commit -m "test(fed-in-e): concurrent receivers respect storage cap (task 2)"
git push origin <worker-branch>
```

Post-push (advisor-laptop):

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-e-task2-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task2-check.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-e-task2-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task2-clippy.log
# EXPECT: exit 0

cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-e-task2-e2e-norun.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task2-e2e-norun.log
# EXPECT: exit 0

# Targeted e2e (~30-60s with testcontainer cold start):
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- v1_federation_inbound_e_fixtures::storage_cap_holds_under_concurrent_receivers > .claude/runlog/e2e-v1-federation-inbound-e-task2-targeted-<sha>.log 2>&1 && echo TARGETED_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-e-task2-targeted-<sha>.log || echo TARGETED_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-e-task2-targeted-<sha>.log"
# run_in_background: true
# EXPECT: TARGETED_EXIT_0
```

Phase-2 e2e full regression check (user gate 4):

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-e-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-e-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-e-<sha>.log"
# run_in_background: true - ~26 min
# EXPECT: E2E_EXIT_0
```

### Task 3: Retro

**ACTION:** author `.claude/PRPs/reports/v1-federation-inbound-e-retro.md` per `feedback_retro_not_report.md` + `feedback_four_role_retro_signals.md` + `feedback_retro_task_complexity_score.md`.

**FILES:**

```yaml
creates:
  - .claude/PRPs/reports/v1-federation-inbound-e-retro.md
modifies: []
requires:
  - task: 2
    reason: "Retro signals require Task 2's targeted e2e + the Phase-2 full e2e + any CR-fix-in-PR cycles having landed before authorship."
```

**IMPLEMENT:** four H2 sections (Advisor / Planning / Impl / BM) with signals + per-task complexity scores + lessons promoted this phase + carry-forward items. Mandatory sub-sections:

- **Advisor signal:** did the conformance-audit prevention checkpoint at Task 1 brief authoring catch any Tier-1 findings on `inbox.rs`? Did the cycle-count meta-rule fire? Did the missing-brief workflow gap re-surface (planner self-resolved `kind: "log"` DQ filed at plan commit)? Did the daemon-stale-ref BM-verb pattern recur (3rd occurrence threshold for `feedback_daemon_stale_bm_verb_brief_miss.md`)?
- **Planning signal:** was Pattern §10.1 (`acquire_evict_lock` mirroring `reputation_snapshot`) cited correctly by Task 1's impl-task brief? Was Pattern §10.4 (e2e fixtures module mirroring `v1_federation_inbound_b_fixtures`) cited correctly? Did the gate-1 concurrency-model DQ resolve cleanly? Did the canonical-schema-first read at Task 2 brief time identify the actual `build_minimal_sanction_notice_activity` helper symbol?
- **Impl signal:** per-task complexity expected `1/1/<short>/<short>` for Task 1 (one file, one commit, ~10-15 min runtime, ~2 min max log silence) and `1/1/<medium>/<short>` for Task 2 (one file, one commit, ~30-45 min runtime including testcontainer cold-start). Surface any divergence.
- **BM signal:** did `bm-cut` + `bm-pr` + `bm-merge` flow cleanly? Any CR triage cycles (CodeRabbit may flag the new `acquire_evict_lock` for lock-acquisition discipline)? Did `gh pr merge` exit clean? Phase branch deleted from origin per `bm-merge.md` L16?
- **Mandatory carry-forward (from bootstrap):** governance-v0 divergence check before gate 5; PRE-PUSH MANDATE on BM-verb briefs; daemon pre-dispatch ref check via ssh.
- **Mandatory carry-forward (this phase's contribution):** brief-skip incident — the planning Junior was queued without the brief file being committed. Retro proposes candidate lesson `feedback_advisor_must_commit_brief_before_planning_dispatch.md`.
- **Optional carry-forward:** if Phase-2 e2e regression uncovered new flake under tokio multi-thread mode, evaluate `serial_test` dev-dep.
- **Optional carry-forward:** post-pilot follow-ups (per-peer in-memory bound, SHA-256 key-hash, governance_config knob, Postgres-backed counters) still deferred.

**VALIDATE:** retro committed. Subject: `docs(retro): v1-federation-inbound-e — TOCTOU fix in evict_oldest_unreviewed_if_needed shipped`.

---

## 14. Testing strategy

Layer-by-layer:

- **Unit (compile-time):** `cargo check --workspace --features full` after Tasks 1+2 (§15.1).
- **Lint:** `cargo clippy --workspace --features full --no-deps -- -D warnings` after Tasks 1+2 (§15.2).
- **Test target compile:** `cargo test --workspace --features full --test e2e --no-run` after Tasks 1+2 (§15.3).
- **Targeted e2e:** `cargo test --workspace --test e2e --features full -- v1_federation_inbound_e_fixtures::storage_cap_holds_under_concurrent_receivers` after Task 2 (§15.4) — deterministic regression gate for Race A + Race B.
- **Phase-2 e2e full regression:** `cargo test --workspace --test e2e --features full` after Task 2 finalize-merge (§15.5).
- **Migration round-trip:** N/A (zero migrations per PRECON-4).

**Behavioural assertion (post-fix):** Task 2's targeted e2e, when run against the pre-Task-1 codebase, would FAIL both assertions: `final_count` would equal `cap + 1` or more (Race B), and `drop_log_count` would deviate from 8 (Race A). Post-fix, both assertions are deterministic — advisory lock serialises COUNT-DELETE-INSERT-LOG per `(peer_domain, table_name)`.

## 15. Validation commands (DoD)

> Every command MUST be dry-run by the advisor against current HEAD before plan approval (User Gate 1). Unexecutable commands are advisor-side rejection grounds.
>
> **DoD shape:** `validate-pending-laptop` (Shape G SUSPENDED until 2026-06-01, DQ #229 pending re-enable).

### 15.1 Per-task workspace check (Tasks 1, 2)

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-e-task<N>-check.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task<N>-check.log
```

**EXPECT:** exit 0.

### 15.2 Per-task clippy (Tasks 1, 2 — uniform R6)

```bash
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-e-task<N>-clippy.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task<N>-clippy.log
```

**EXPECT:** exit 0.

### 15.3 Test target compile (R7 — Tasks 1, 2)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-federation-inbound-e-task<N>-e2e-norun.log 2>&1"
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-federation-inbound-e-task<N>-e2e-norun.log
```

**EXPECT:** exit 0.

### 15.4 Targeted e2e — concurrent-receivers regression (Task 2 only)

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- v1_federation_inbound_e_fixtures::storage_cap_holds_under_concurrent_receivers > .claude/runlog/e2e-v1-federation-inbound-e-task2-targeted-<sha>.log 2>&1 && echo TARGETED_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-e-task2-targeted-<sha>.log || echo TARGETED_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-e-task2-targeted-<sha>.log"
# run_in_background: true
```

**EXPECT:** tail of the log contains `test v1_federation_inbound_e_fixtures::storage_cap_holds_under_concurrent_receivers ... ok` AND `TARGETED_EXIT_0`.

### 15.5 Phase-2 e2e full regression (post-finalize-merge — user-gate-4)

**(a) Local laptop bg** (default-recommended; ~26 min, zero billed):

```bash
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/runlog/e2e-v1-federation-inbound-e-<sha>.log 2>&1 && echo E2E_EXIT_0 >> .claude/runlog/e2e-v1-federation-inbound-e-<sha>.log || echo E2E_EXIT_NONZERO >> .claude/runlog/e2e-v1-federation-inbound-e-<sha>.log"
# run_in_background: true
```

**(b) GH dispatch** (escape hatch; ~26 min billed):

```bash
gh workflow run cargo-test-e2e.yml --repo barrie-cork/lemmy --ref phase-v1-federation-inbound-e
```

**EXPECT:** tail of log contains `E2E_EXIT_0`. All pre-existing fed-in-a/b/c/d tests pass; new fed-in-e test passes; no regression.

### 15.6 Shape G section — DORMANT until 2026-06-01

**NOT applicable.** Per PRECON-5 + DQ #229.

### 15.7 Cross-cutting verification

- [ ] `grep -n "fn acquire_evict_lock" crates/apub/activities/src/governance/inbox.rs` returns exactly 1 match.
- [ ] `grep -n "fn evict_oldest_unreviewed_if_needed_in_tx" crates/apub/activities/src/governance/inbox.rs` returns exactly 1 match.
- [ ] `grep -nE "fn evict_oldest_unreviewed_if_needed\b" crates/apub/activities/src/governance/inbox.rs` returns ZERO matches (old name gone).
- [ ] `grep -c "evict_oldest_unreviewed_if_needed_in_tx(" crates/apub/activities/src/governance/inbox.rs` returns at least 4 (3 calls + 1 definition).
- [ ] `grep -c "acquire_evict_lock(" crates/apub/activities/src/governance/inbox.rs` returns at least 4 (3 calls + 1 definition).
- [ ] `grep -c "pg_advisory_xact_lock" crates/apub/activities/src/governance/inbox.rs` returns exactly 1 match.
- [ ] `grep -cE "conn\.run_transaction" crates/apub/activities/src/governance/inbox.rs` returns exactly 3 (one per caller).
- [ ] `grep -c "mod v1_federation_inbound_e_fixtures" crates/server/tests/e2e.rs` returns exactly 1.
- [ ] `grep -c "storage_cap_holds_under_concurrent_receivers" crates/server/tests/e2e.rs` returns exactly 1.
- [ ] `rg -c '^pub const ENTRY_KIND_' crates/db_schema/src/source/governance/governance_log.rs` returns **55**.
- [ ] `rg '^\s+ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs | wc -l` returns **55**.
- [ ] No edit to `crates/db_schema/`, `migrations/**`, `crates/api/api/src/governance/**`, `crates/apub/activities/src/governance/publish_*.rs`, or `Cargo.toml`.
- [ ] No new file under `crates/apub/activities/src/governance/` or `crates/server/tests/`.
- [ ] R1: no `i32 as i64` casts in new code.
- [ ] R6: every §15.2 invocation uses `--no-deps -- -D warnings`.
- [ ] R7: Task 1 + Task 2 both ran `cargo test --no-run` (§15.3).
- [ ] PRECON-1 (scope: TOCTOU fix on `evict_oldest_unreviewed_if_needed` only) honoured.
- [ ] PRECON-2 (concurrency model: `pg_advisory_xact_lock` keyed on `(peer_domain, table_name)`) honoured.
- [ ] PRECON-3 (crate location) honoured.
- [ ] PRECON-4 (zero migrations) honoured.
- [ ] PRECON-5 (validate-pending-laptop DoD) honoured.
- [ ] PRECON-6 (no `[P]` cohort; serial dispatch) honoured.
- [ ] PRECON-7 (review-point sequencing) honoured.

### 15.8 ADR / OQ compliance

- [ ] **ADR-006:** preserved. The drop-log row + drop-event governance_log entry have always been written together; this plan widens the atomic boundary to include the receipt's INSERT + log. Eviction path emits 4 rows in one tx (drop INSERT + drop log + receipt INSERT + receipt log); non-eviction path emits 2 rows (receipt only). All four kinds are pre-existing.
- [ ] **ADR-013:** not in code path; unchanged.
- [ ] **ADR-014:** unchanged.
- [ ] **ADR-015:** unchanged. `peer_domain` stays plaintext (public AP routing data).
- [ ] **Append-only contract** (`governance_config`): preserved.

---

## 16. Acceptance criteria

- [ ] All 4 tasks (Task 0 + 3 work tasks) completed in dependency order.
- [ ] §15.1 (cargo check) exit 0 after Tasks 1, 2.
- [ ] §15.2 (cargo clippy `--no-deps -- -D warnings`) exit 0 after Tasks 1, 2.
- [ ] §15.3 (cargo test --no-run --test e2e) exit 0 after Tasks 1, 2 (R7).
- [ ] §15.4 (targeted e2e) — `storage_cap_holds_under_concurrent_receivers` passes; tail shows `TARGETED_EXIT_0`.
- [ ] §15.5 (Phase-2 e2e full regression) — all pre-existing tests pass; new test passes; tail shows `E2E_EXIT_0`.
- [ ] §15.6 N/A (Shape G dormant).
- [ ] §15.7 cross-cutting verification — all boxes ticked.
- [ ] §15.8 ADR/OQ compliance — all boxes ticked.
- [ ] §16a stories — all stories `[done]`.
- [ ] No edits to files outside §11 list.
- [ ] Retro committed per §13 Task 3.
- [ ] PR opens against `governance-v0` with `--repo barrie-cork/lemmy`.

---

## 16a. Stories (independently-testable behaviour units)

### Story 1: Per-peer storage cap holds under concurrent receivers (Race A + Race B closed)

- **User-facing behaviour:** when 8 concurrent inbound `PublishSanctionNotice` activities arrive for the same allowlisted peer that has already filled its per-peer storage cap, the receiver path serialises the eviction-and-insert sequence per `(peer_domain, table_name)`. Post-burst, table count holds at exactly `cap`, and `federation_inbox_dropped_log` records exactly one eviction per inbound.
- **Composing tasks:** Task 1 (fix) + Task 2 (e2e regression). Sequential (Task 2 `requires: [1]`).
- **Checkpoint command (laptop):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- v1_federation_inbound_e_fixtures::storage_cap_holds_under_concurrent_receivers"`. EXPECT exit 0; tail shows `test v1_federation_inbound_e_fixtures::storage_cap_holds_under_concurrent_receivers ... ok`.
- **Brief-Scope outputs to verify:**
  - `crates/apub/activities/src/governance/inbox.rs` contains `async fn acquire_evict_lock(` exactly once.
  - `inbox.rs` contains `async fn evict_oldest_unreviewed_if_needed_in_tx(` exactly once.
  - `inbox.rs` does NOT contain `async fn evict_oldest_unreviewed_if_needed(` (without `_in_tx`).
  - `inbox.rs` contains `pg_advisory_xact_lock(hashtextextended(` exactly once.
  - `inbox.rs` contains `acquire_evict_lock(conn,` at 3 call sites.
  - `inbox.rs` contains `evict_oldest_unreviewed_if_needed_in_tx(` at 3 call sites + 1 definition.
  - `crates/server/tests/e2e.rs` contains `mod v1_federation_inbound_e_fixtures {` exactly once.
  - `e2e.rs` contains `async fn storage_cap_holds_under_concurrent_receivers` exactly once.
  - `e2e.rs` contains `assert_eq!(final_count, 5,` (or equivalent multi-line form) — Race-B assertion.
  - `e2e.rs` contains `assert_eq!(drop_log_count, 8,` — Race-A assertion.

### Story 2: Phase-2 e2e regression gate is green

- **User-facing behaviour:** consolidated-tx refactor in Task 1 does NOT alter observable rate-limit semantics; all pre-existing fed-in-a/b/c/d tests still pass.
- **Composing tasks:** Task 1 + Task 2.
- **Checkpoint command (laptop, Phase-2 e2e):** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1"` with `run_in_background: true`. EXPECT `E2E_EXIT_0`.
- **Brief-Scope outputs to verify:**
  - `.claude/runlog/e2e-v1-federation-inbound-e-<sha>.log` exists and tail contains `E2E_EXIT_0`.
  - No new test failure (count of "FAILED" = 0).
  - Pre-existing `mod v1_federation_inbound_a_fixtures` + `mod v1_federation_inbound_b_fixtures` test counts match baseline.

### Story 3: Retro captures four-role signals + complexity scores + carry-forward

- **User-facing behaviour:** `bm-merge` runs only after retro authorship.
- **Composing tasks:** Task 3 (`requires: [2]`).
- **Checkpoint command:** `test -f .claude/PRPs/reports/v1-federation-inbound-e-retro.md && grep -c '^## ' .claude/PRPs/reports/v1-federation-inbound-e-retro.md` returns ≥ 4.
- **Brief-Scope outputs to verify:**
  - Retro file exists at canonical path.
  - Four H2 sections present: Advisor / Planning / Impl / BM.
  - Per-task complexity score block present.
  - Carry-forward block names: (a) brief-skip workflow gap + candidate lesson `feedback_advisor_must_commit_brief_before_planning_dispatch.md`; (b) confirmation of gate-1 concurrency-model DQ resolution; (c) status of post-pilot deferrals from §12.
  - Lessons promoted this phase (if any) land in the same retro commit body.

> **Verification mapping:** `/brehon-verify` iterates this section, runs each Story's Checkpoint, confirms each Brief-Scope output. Phantoms trigger catch-fire.

---

## 17. Completion checklist

- [ ] Task 0 audit complete (all 10 probe families).
- [ ] Tasks 1, 2 committed in dependency order (serial).
- [ ] Task 3 retro committed.
- [ ] §15 validation green at every gate.
- [ ] §16a stories all `[done]`.
- [ ] PR opened by BM session against `governance-v0` with `--repo barrie-cork/lemmy`.
- [ ] CodeRabbit review complete with findings triaged.
- [ ] `/brehon-verify` report at `.claude/PRPs/reports/v1-federation-inbound-e-verify.md` shows all stories ✓.
- [ ] PRECON-1 / PRECON-2 / PRECON-3 / PRECON-4 / PRECON-5 / PRECON-6 / PRECON-7 honoured.
- [ ] Post-merge phase branch retained for retro reads.

---

## 18. Risks and mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Impl agent picks a different concurrency model (atomic-SQL-with-RETURNING / SKIP LOCKED) at canonical-schema-first read time | LOW | HIGH | §4 watchpoint names `pg_advisory_xact_lock` as chosen; §10.1 shows verbatim shape; planner-recommended gate-1 DQ forces explicit advisor validation. §G4 allowlist does NOT cover concurrency-model changes; catch-fire if impl deviates. |
| Impl agent forgets to acquire the lock at the top of one of the three caller tx blocks | MEDIUM | HIGH | §G4 callsite enumeration via `rg "acquire_evict_lock(" crates/apub/activities/src/governance/inbox.rs` must return exactly 4 matches. §15.7 asserts this. Pre-push `cargo-check.sh` does NOT catch forgotten lock acquisition (code compiles); targeted e2e in §15.4 catches it. |
| `pg_advisory_xact_lock` causes connection-pool starvation under sustained concurrent inbound from one peer | LOW | MEDIUM | Gate-4 per-peer rate cap bounds queue depth. Lock release on tx commit/rollback ensures no leaks. If observed at pilot, retro proposes timeout-via-`pg_try_advisory_xact_lock`. |
| `hashtextextended` not available in Lemmy's targeted Postgres version | LOW | HIGH | `hashtextextended` is core Postgres since 11.0; Lemmy 1.0-beta targets Postgres ≥15. Verified at plan-time via absence of contradictory evidence. |
| Diesel async borrow checker rejects the consolidated tx (`source_instance` borrow into the closure conflicts) | LOW | MEDIUM | `async move` closures move owned values + borrow references from surrounding fn scope. The 3 caller refactors mirror an established pattern. Pre-push `cargo-check.sh` catches any borrow error. |
| Cycle-count meta-rule (≥3 fails on same `(error_class, file_basename)`) fires during Task 1 dispatch | LOW | HIGH | §4 sets honest expectation: function-rename cascade + borrow-checker errors anticipated; pre-push `cargo-check.sh` + brief's callsite-enumeration keep cycle count to 1. Meta-rule is hard refusal regardless. |
| Task 2's targeted e2e is flaky due to testcontainer startup variance | LOW | MEDIUM | Testcontainer-per-test isolation is established pattern. Unique `peer_domain` namespace prevents cross-test interference. |
| Phase-2 full e2e regression introduces a tokio-multi-thread spawn-ordering flake in an unrelated test | LOW | LOW | New test uses `flavor = "multi_thread"` only in its own `#[tokio::test]`; other tests keep their flavor. If flake emerges, retro evaluates `serial_test` dev-dep. |
| Advisor-side §3.4 DoD smoke fails at plan-approval time | LOW | HIGH | Task 0 probes capture baseline; if Probe 9 fails, advisor files DQ pending and requests planner narrowing OR a pre-phase `chore(lint)` commit. |
| Shape G re-enables mid-sub-phase (2026-06-01 boundary crossed) | LOW | LOW | PRECON-5 carries forward reminder; mechanical kind switch. |
| Junior worker forks from stale base and misses fed-in-d's per-actor bound | LOW | HIGH | bm-cut creates `phase-v1-federation-inbound-e` off post-fed-in-d-merge `governance-v0`; Probe 6 at Task 0 verifies `MAX_PER_ACTOR_RATE_ENTRIES` const exists. PRE-PUSH MANDATE on every brief. |
| Task 2's brief authored before Task 1 finalize-merges, missing Task 1's actual chosen lock-key derivation | LOW | LOW | Task 2's `requires: [1]` gates dispatch; advisor authors Task 2's brief AFTER Task 1 finalize-merge. |
| Missing `.claude/PRPs/briefs/v1-federation-inbound-e-planning-1.md` (advisor queued planning Junior without first authoring the brief) | OCCURRED | MEDIUM | Filed as `kind: "log"` planner DQ at this plan's commit. Bootstrap doc served as de-facto brief. Retro proposes candidate lesson `feedback_advisor_must_commit_brief_before_planning_dispatch.md`. |

---

## 19. Notes

- **Scope discipline** — excludes the per-peer in-memory bound, SHA-256 key-hash, governance_config knob, Postgres backing, helper extraction, cohort dispatch, Shape G switch, new ENTRY_KIND, isolation upgrade, lock timeout, telemetry, and dev-dep additions per §12 (14 items). The gate-1 concurrency-model DQ is the only open question.

- **No `[P]` cohort, no DQ pre-reservation** — Task 2 has `requires: [1]`; validation-dependency forces serial.

- **PRECON enumeration (7 items)** — recorded in §15.7 cross-cutting verification.

- **Planner-recommended gate-1 DQ filed at plan commit** — `from: "planner"`, `answered_by: "planner"`, `question: "concurrency model: pg_advisory_xact_lock per (peer_domain, table_name) vs SELECT FOR UPDATE SKIP LOCKED vs atomic-SQL-with-RETURNING — proceed with planner-recommended pg_advisory_xact_lock?"`. Advisor at User Gate 1 either resolves with `answered_by: "advisor"` (citation) or user-relays.

- **Brief-skip self-resolved `kind: "log"` DQ filed at plan commit** — recording the process gap (advisor queued planning Junior without committing the brief; bootstrap served as de-facto brief). Retro evaluates for lesson promotion.

- **Lessons promoted this phase (anticipated)** — candidate `feedback_advisor_must_commit_brief_before_planning_dispatch.md` may promote at retro time if user judges evidence sufficient.

- **Alternative approaches considered (rejected at plan-author time)** —
  1. Atomic-SQL-with-RETURNING — COUNT subquery race under READ COMMITTED; SQL complexity.
  2. `SELECT FOR UPDATE SKIP LOCKED` — doesn't serialise COUNT-vs-INSERT-new race.
  3. `SERIALIZABLE` isolation workspace-wide — too broad.
  4. `pg_try_advisory_xact_lock` + retry — complicates failure semantics.
  5. Move eviction inside `wrap_governance_inbound` — wrapper doesn't know table_name until handler delegate runs.

---

## 20. Confidence score

- **Plan correctness:** 9/10 — defect verified at plan-author time (`inbox.rs:646-703` + 3 call sites + `reputation_snapshot.rs:824` MIRROR + `appeal_window_expiry.rs:42` MIRROR); fix well-trodden in Brehon codebase; gate-1 DQ documents the trade-off. Minor uncertainty in (a) `hashtextextended` Postgres-version support (assumed ≥11; Lemmy targets ≥15) and (b) `context.reset_request_count()` exact API name in `activitypub_federation::config::Data`.
- **Cargo budget:** 10/10 — N/A (validate-pending-laptop runs on laptop).
- **Test coverage:** 8/10 — Task 2's targeted e2e directly exercises Race A AND Race B under deterministic multi-thread tokio fan-out; §15.5 full regression confirms Task 1 doesn't break pre-existing tests. Test exercises ONE receiver (`receive_remote_sanction_notice`); the other two inherit the same refactor pattern but are not directly exercised — relies on inductive argument that the 3 callers' refactors are mechanically identical. If pilot evidence shows otherwise, follow-up sub-phase adds dedicated probes.
