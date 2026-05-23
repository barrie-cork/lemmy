# v1-federation-inbound-e — Task 2 impl-task brief

## 1. Role + dispatch line

```
[role:impl-task] v1-federation-inbound-e task 2 — see .claude/PRPs/briefs/v1-federation-inbound-e-impl-2.md
```

**Junior task description (exact string to pass to `mcp__junior-brehon__create_task`):**
```
[role:impl-task] v1-federation-inbound-e task 2 — see .claude/PRPs/briefs/v1-federation-inbound-e-impl-2.md
```

---

## 2. Scope

Append one new module (`mod v1_federation_inbound_e_fixtures`) to the **end** of:
```
crates/server/tests/e2e.rs
```

The module contains exactly one test: `storage_cap_holds_under_concurrent_receivers`.

### What to produce

A block of ~150 lines appended after the last line of `crates/server/tests/e2e.rs`:

```rust
mod v1_federation_inbound_e_fixtures {
    // imports (mirror v1_federation_inbound_b_fixtures §imports verbatim)
    // helper: build_sanction_notice_for_e(peer_domain, seq) — unique ID per seq
    // helper: bootstrap_with_peer_e(domain) — wraps bootstrap_with_peer(domain, Some(FederationPeerTrust::Allowlisted))
    // test: storage_cap_holds_under_concurrent_receivers
}
```

### Test specification

```rust
#[tokio::test(flavor = "multi_thread")]
async fn storage_cap_holds_under_concurrent_receivers() -> LemmyResult<()> {
    // Phase 1: bootstrap + seed 5 rows
    //   bootstrap_with_peer_e("concurrent.test") → (container, config, url, instance_id)
    //   set remote cap: RemotePeerConfig { max_unreviewed_sanction_notices: 5, .. }
    //   seed 5 rows via ActivityTrait::receive for seq 0..4
    //
    // Phase 2: 8 concurrent receive calls (all unique seq IDs)
    //   tokio::task::spawn × 8, seq 5..12
    //   each calls ActivityTrait::receive(activity_i, &data, &config, pool.clone())
    //   collect JoinHandle results, unwrap all
    //
    // Phase 3: assertions (both must pass)
    //   final_count = SELECT COUNT(*) FROM remote_sanction_notice WHERE peer_domain='concurrent.test'
    //   assert_eq!(final_count, 5);      // Race B gate: cap holds under contention
    //   drop_log_count = SELECT COUNT(*) FROM governance_modlog WHERE action='sanction_notice_cap_evicted'
    //     AND actor_peer_domain='concurrent.test'
    //   assert_eq!(drop_log_count, 8);   // Race A gate: every eviction logged
    Ok(())
}
```

### Boundaries (do NOT cross these)

- **Only file modified**: `crates/server/tests/e2e.rs` — append only, no edits to existing lines.
- **Do NOT import or call** `build_minimal_sanction_notice_activity` or `build_unique_sanction_notice_activity` from `v1_federation_inbound_b_fixtures` — those are scoped to that module. Define your own builder in this module.
- **Do NOT edit** `crates/apub/activities/src/governance/inbox.rs` (that is Task 1's file, already committed).
- **Do NOT create** new files. One append to one existing file.
- **Commit subject** (exact): `feat(fed-in-e): e2e test storage_cap_holds_under_concurrent_receivers (task 2)`

### DQ to raise after push

After committing and pushing to the worker branch, raise a `kind: "validate-pending-laptop"` DQ entry with:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "commands": [
    "cargo check --workspace --features full",
    "cargo clippy --workspace --features full --no-deps -- -D warnings",
    "cargo test --workspace --test e2e --features full --no-run",
    "cargo test --workspace --test e2e --features full storage_cap_holds_under_concurrent_receivers"
  ],
  "branch": "junior/<your-worker-branch-name>",
  "phase_task": 2
}
```

Commit + push the DQ entry to your worker branch immediately after raising it (mid-task push required per `.claude/rules/decision-queue.md` "Mid-task visibility").

---

## 3. Required reading (read ALL before writing any code)

### 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: 6961a915e
    filesModified: [crates/apub/activities/src/governance/inbox.rs]
    keyDecisions:
      - "acquire_evict_lock() helper acquires pg_advisory_xact_lock keyed on (peer_domain, 'remote_sanction_notice')"
      - "evict_oldest_unreviewed_if_needed renamed to evict_oldest_unreviewed_if_needed_in_tx — now called inside a run_transaction block"
      - "Three callers (receive_new, receive_cached, receive_nocap) each wrap acquire_evict_lock() + evict call + insert into a single run_transaction"
    notes: "async move closures move captured Strings; add a pre-clone (si_tx/pd_tx) before run_transaction when the original variable is needed in a post-await error handler. LESSON trailer on commit."
```

### 3b. Plan

Read `.claude/PRPs/plans/v1-federation-inbound-e.plan.md` — **§13 Task 2** in full.

Focus on:
- FILES YAML (creates/modifies/requires)
- The exact assertion values (`final_count == 5`, `drop_log_count == 8`)
- The DoD commands (§15 Task 2 DoD)

### 3c. MIRROR reference (canonical sibling — read before writing any code)

Read `crates/server/tests/e2e.rs` lines **15797–15950** (`mod v1_federation_inbound_b_fixtures`).

Mirror these elements verbatim:
- The `use` import block at the top of the module — copy identically
- The `bootstrap_with_peer` wrapper signature (it takes `domain: &str, trust: Option<FederationPeerTrust>`) — call it with `Some(FederationPeerTrust::Allowlisted)` for the e_fixtures wrapper
- The outer test function signature: `async fn <name>() -> LemmyResult<()>` — this is **Case A** (see §3d below)
- The `context.reset_request_count()` call pattern if used in Phase 1 setup

**CRITICAL DISCREPANCY vs plan**: the plan's pseudo-code for `bootstrap_with_peer` omits the `trust` parameter. The actual function signature in the codebase requires it. Use `Some(FederationPeerTrust::Allowlisted)`.

### 3d. Mandatory lessons (file-class injection — no judgment call, all apply)

**`feedback_lemmy_error_no_std_error.md`** — read in full.

This test returns `LemmyResult<()>`. That is **Case A**:
- Outer function: `async fn test() -> LemmyResult<()>`
- All helpers: return `LemmyResult<T>` consistently
- Use `?` operator directly — NO `.map_err(|e| format!("{e}").into())?` bridges
- Mirror the canonical sibling `v1_federation_inbound_b_fixtures` shape verbatim

Do NOT mix Case A outer with Case B helpers. If the sibling uses `LemmyResult<()>`, mirror exactly.

**`feedback_async_pool_test_pattern.md`** — read in full.

- `AsyncPgConnection::establish(&db_url)` pattern for pool in e2e tests
- `DbPool::Conn` for the connection type
- `lemmy_server::db_pool` for the pool accessor used in existing e2e tests

**`feedback_clippy_test_style.md`** — read in full.

- No `unwrap()` or `expect()` in test bodies — use `?` instead
- No `#[allow(unused)]` or similar suppression attributes without a LESSON: DQ entry
- The test must compile cleanly with `cargo clippy --workspace --features full --no-deps -- -D warnings`

### 3e. Additional context

**`feedback_pg_advisory_xact_lock_void_decode.md`** — skim.
Context only: Task 1 used `pg_advisory_xact_lock`; your test exercises its effect. You do not call this function directly, but understanding why the lock exists (TOCTOU serialization) helps you write the assertions correctly.

**`.claude/rules/decision-queue.md`** §"Mid-task visibility" + §"validate-pending-laptop" shape.
Required: you must commit + push the DQ entry to your worker branch immediately after raising it. The advisor reads the phase branch; entries trapped in a local worktree are invisible.

---

## 4. Constraints (hard — no deviation without `kind: "blocker"` DQ)

### 4a. Pre-push cargo-check (mandatory gate)

Before pushing your worker branch, run:
```
bash scripts/brehon/cargo-check.sh --workspace --features full
```

Non-zero exit → patch in-scope issues in the same commit; file `kind: "blocker"` DQ if out-of-scope. NEVER use `#[allow(...)]` to bypass. Do not push until this exits 0.

### 4b. Test shape — `tokio::test(flavor = "multi_thread")`

The concurrency test requires multi-thread runtime. Do NOT use `#[tokio::test]` without `flavor = "multi_thread"`. Single-thread executor serialises tasks and the race regression cannot be observed.

### 4c. Unique activity IDs for concurrent calls

Each of the 8 concurrent `receive()` calls must use a **distinct** activity URL/ID. If two calls share the same activity ID, the Lemmy deduplication layer will reject the duplicate. Use a sequence counter (0-indexed within the concurrent batch, starting at 5 since seeding used 0-4) to generate unique IDs. Pattern from sibling:

```rust
fn build_sanction_notice_for_e(peer_domain: &str, seq: u32) -> PublishSanctionNotice {
    // unique activity_id: format!("https://{peer_domain}/sanction/{seq}")
    // mirror build_unique_sanction_notice_activity from b_fixtures
}
```

### 4d. Append-only — do NOT edit existing lines

The sibling `mod v1_federation_inbound_b_fixtures` ends at some line before EOF. Append the new module AFTER it. Do NOT reorder, reformat, or touch any existing line. A stale cached build may break if existing line numbers shift.

### 4e. DQ raise + commit + push sequence

1. Implement the test, commit with `feat(fed-in-e): e2e test storage_cap_holds_under_concurrent_receivers (task 2)`
2. Run `bash scripts/brehon/cargo-check.sh --workspace --features full` — must exit 0
3. Push the impl commit to your worker branch
4. Raise the `validate-pending-laptop` DQ entry (shape in §2 above)
5. Commit + push the DQ entry immediately: `chore(decision-queue): impl raised validate-pending-laptop DQ <id> (task 2)`

Do NOT reverse steps 4 and 5. The entry must be pushed before the task exits.

### 4f. LESSON trailer (required)

If you discover a non-obvious constraint during implementation, append a `LESSON:` line to the end of your impl commit message body:

```
LESSON: <one-line durable observation for the advisor to harvest at retro>
```

This is a contract, not optional — if the implementation surface was smooth, write `LESSON: (none — clean implementation)`.

### 4g. File-ownership boundary

- WRITE: `crates/server/tests/e2e.rs` (append only), `.claude/decision-queue.json` (DQ raise)
- READ: `crates/apub/activities/src/governance/inbox.rs`, `.claude/PRPs/plans/v1-federation-inbound-e.plan.md`, sibling in `e2e.rs`
- NEVER WRITE: `.claude/PRPs/plans/**`, `.claude/rules/**`, `.claude/lessons/**`, `crates/apub/**` (impl-task owns only files in its own brief's FILES YAML)
