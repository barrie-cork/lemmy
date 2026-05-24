> **[CLOSED — shipped 2026-05-24, advisory-lock phase]** This brief is from the
> completed `v1-federation-inbound-e` sub-phase (pg_advisory_xact_lock eviction fix).
> The active phase brief is `v1-federation-inbound-e-planning-1.md` (per-peer actor-map
> bound + SHA-256 nonce).

# Impl-task brief — v1-federation-inbound-e Task 1

**Role:** `[role:impl-task]`
**Phase:** `v1-federation-inbound-e`
**Task:** 1 of 4 (Task 1)
**Authored:** 2026-05-22
**Base branch:** `phase-v1-federation-inbound-e`

---

## 1. Role + dispatch line

```
[role:impl-task] v1-federation-inbound-e task 1 — see .claude/PRPs/briefs/v1-federation-inbound-e-impl-1.md
```

---

## 2. Scope

### 2.1 What to produce

One commit on `phase-v1-federation-inbound-e` that modifies exactly one file:

```
crates/apub/activities/src/governance/inbox.rs
```

### 2.2 What to implement

In `crates/apub/activities/src/governance/inbox.rs`:

1. **Add `acquire_evict_lock` helper** above the existing `CountRow` declaration (currently at line 634). Verbatim shape — §10.1 of the plan:

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

2. **Rename + simplify `evict_oldest_unreviewed_if_needed` → `evict_oldest_unreviewed_if_needed_in_tx`** at lines 646-703. Drop the inner `conn.run_transaction(...)` wrapper. Change the conn-arg type from `&mut DbConn<'_>` to `&mut AsyncPgConnection`. Verbatim shape — §10.2 of the plan.

3. **Refactor caller 1 — `receive_remote_sanction_notice` at lines 188-228.** Delete the pre-tx `evict_oldest_unreviewed_if_needed(...)` call at line 191. Modify the existing `run_transaction(|conn| async move { ... })` body to FIRST call `acquire_evict_lock` + `evict_oldest_unreviewed_if_needed_in_tx`, THEN the existing `insert_remote_sanction_notice` + `governance_log::append(ENTRY_KIND_FEDERATION_SANCTION_RECEIVED, ...)`. Verbatim shape — §10.3 post-fix block. The `if let Err(e) = &outcome { ... }` block at lines 211-226 stays UNCHANGED.

4. **Refactor caller 2 — `receive_remote_trust_attestation` at lines 300-339.** Same consolidated-tx shape. table name `"federation_attestation"`. INSERT-form `FederationAttestationInsertForm`. Receipt's ENTRY_KIND `ENTRY_KIND_FEDERATION_ATTESTATION_RECEIVED`. Pre-tx work at lines 244-303 unchanged.

5. **Refactor caller 3 — `receive_remote_moderation_label` at lines 750-799.** Same consolidated-tx shape. table name `"remote_moderation_label"`. INSERT-form `RemoteModerationLabelInsertForm`. Receipt's ENTRY_KIND `ENTRY_KIND_FEDERATION_LABEL_RECEIVED`. The `let trust = federation_inbox_check_peer_trust(&peer_domain, conn).await?;` + `let form = ...` stay outside the consolidated tx.

### 2.3 Scope boundary

**IN scope (exactly one file):**
- `crates/apub/activities/src/governance/inbox.rs`

**OUT of scope (catch-fire if touched):**
- `crates/apub/activities/src/governance/publish_trust_attestation.rs`
- `crates/api/api/src/governance/reputation_snapshot.rs`
- `crates/db_schema/` — no struct-field changes, no new ENTRY_KIND
- `migrations/**` — no migrations in this phase
- Any file not listed above
- `crates/server/tests/e2e.rs` — that is Task 2

### 2.4 Callsite enumeration (mandatory — `feedback_fix_impl_enumerate_all_callsites.md`)

Before making any edit, run:
```bash
rg "evict_oldest_unreviewed_if_needed" crates/
```

**EXPECT:** exactly 4 matches, all in `crates/apub/activities/src/governance/inbox.rs`:
- Line ~646: function definition
- Line ~191: caller 1 (`receive_remote_sanction_notice`)
- Line ~305: caller 2 (`receive_remote_trust_attestation`)
- Line ~761: caller 3 (`receive_remote_moderation_label`)

If any match is outside `inbox.rs`, file `kind: "blocker"` DQ and STOP.

---

## 3. Required reading

- `.claude/PRPs/plans/v1-federation-inbound-e.plan.md` — full plan. Read §4 (solution), §10.1, §10.2, §10.3 (verbatim code shapes), §11 (file list), §13 Task 1 (GOTCHA bullets), §15 (DoD).
- `.claude/lessons/feedback_pg_advisory_xact_lock_void_decode.md` — **MANDATORY file-class lesson**: `.execute()` NOT `.load()` for `pg_advisory_xact_lock` (void PG function).
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — **MANDATORY file-class lesson**: multi-write handler atomicity rationale.
- `.claude/lessons/feedback_fix_impl_pre_push_cargo_check.md` — **MANDATORY**: run `bash scripts/brehon/cargo-check.sh --workspace --features full` BEFORE push. Non-zero exit → patch in same commit OR file `kind: "blocker"` DQ.
- `.claude/lessons/feedback_fix_impl_enumerate_all_callsites.md` — enumerate ALL callsites before edit.
- `crates/api/api/src/governance/reputation_snapshot.rs:821-848` — MIRROR for `acquire_evict_lock` shape (existing `acquire_advisory_xact_lock`).
- `crates/apub/activities/src/governance/inbox.rs:634-703` — existing `CountRow` + `evict_oldest_unreviewed_if_needed` (the function being renamed).
- `crates/apub/activities/src/governance/inbox.rs:188-228` — caller 1.
- `crates/apub/activities/src/governance/inbox.rs:300-339` — caller 2.
- `crates/apub/activities/src/governance/inbox.rs:750-799` — caller 3.
- `crates/diesel_utils/src/connection.rs:60-73` — `DbConn::run_transaction` signature.

### 3a. Handover from prior cohort

_(none — first cohort)_

---

## 4. Constraints

- **Branch:** must be `phase-v1-federation-inbound-e`. Check with `git branch --show-current` before any edit.
- **One file only:** if `git diff --stat` shows more than one file, STOP and file `kind: "blocker"` DQ.
- **No new imports:** `AsyncPgConnection` already imported at line 55; `diesel::sql_types::Text` is already in scope via existing usages.
- **`.execute()` not `.load()` for `pg_advisory_xact_lock`:** void Postgres function. Using `.load()` will panic at runtime. Per `feedback_pg_advisory_xact_lock_void_decode.md`.
- **`pg_advisory_xact_lock` must be inside the tx:** calling it outside a transaction is undefined per Postgres docs. The helper is called as the FIRST statement inside `conn.run_transaction(...)`.
- **Callsite bound = 3:** `rg "evict_oldest_unreviewed_if_needed\(" crates/` must return exactly 3 matches after the rename (all three callers must be updated to `_in_tx` name).
- **Pre-push cargo-check mandatory:** run `bash scripts/brehon/cargo-check.sh --workspace --features full` before `git push`. Log to `.claude/PRPs/debug/v1-federation-inbound-e-task1-prepush-check.log`. Non-zero exit → fix in same commit OR file `kind: "blocker"` DQ. NEVER `#[allow]`-spam to bypass.
- **validate-pending-laptop DQ entry:** after successful push, write `kind: "validate-pending-laptop"` to `.claude/decision-queue.json` naming these commands verbatim:
  - `cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-federation-inbound-e-task1-check.log 2>&1"`
  - `cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-federation-inbound-e-task1-clippy.log 2>&1"`
- **DQ mid-task push:** commit + push the DQ entry immediately after writing. Use `from: "impl"`, `answered_by: null`.
- **Commit message:** `feat(fed-in-e): serialise per-peer storage-cap eviction with pg_advisory_xact_lock (task 1)`
- **Attribution:** `from: "impl"` on DQ entries. Never `answered_by: "advisor"`.
- **Conformance-audit Tier-1:** `inbox.rs` is Tier-1 scope. Do not add AP vocabulary divergent from `lemmy_apub_objects` shapes. The change is purely concurrency-correctness (no new AP types, no new fields on serialisable types).
- **Do not touch:** peer-trust check at the top of each receiver; decode_*_object helpers (lines 357-388 + 707-715); insert helpers (lines 393-415); `get_inbound_config_int` helper (lines 425-438); `wrap_governance_inbound` (lines 477+); per-actor in-memory bound at `publish_trust_attestation.rs:158-167`.
