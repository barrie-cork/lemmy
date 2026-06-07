# Brief: m2-late-1 fix-impl-1 — CR fix-in-PR corrections (6 findings)

## 1. Role + dispatch line

`[role:impl-task]` m2-late-1 fix-impl-1 — CR fix-in-pr corrections (6 items from PR #192)

Base branch: `phase-m2-late-1`

## 2. Scope

Apply all 6 fix-in-pr CodeRabbit findings from PR #192 review. All are narrow,
targeted changes. Do NOT touch any file not listed below. Do NOT run cargo
yourself — write the `validate-pending-laptop` DQ entry and stop.

Files to edit (one commit per logical group is acceptable, or one combined commit):

1. `crates/api/api/src/governance/submit_jury_vote.rs`
2. `crates/api/api/src/governance/sanction_publisher.rs`
3. `crates/server/src/lib.rs`
4. `crates/server/tests/e2e/m2_late.rs`
5. `migrations/2026-06-07-000000-0000_add_sanction_event/up.sql`
6. `services/bridge/src/sanction_handler.rs`

Do NOT touch:
- Any other migration files
- `governance.rs` or `m2_late.rs` beyond the one change specified below
- Any file under `.claude/`, `docs/`, `scripts/`

## 3. Required reading

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error shape discipline
- `.claude/lessons/feedback_multi_write_handlers_need_transactions.md` — transaction discipline
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — DQ write-and-stop

## 4. Changes (apply each exactly as described)

### Fix 1 — `submit_jury_vote.rs`: sanction ID equality guard before enqueue

**File:** `crates/api/api/src/governance/submit_jury_vote.rs`

**Problem (CR-4):** The post-transaction re-query finds ANY active sanction for the
case. Votes after quorum (votes 4 and 5 in a 5-juror panel) each find the active
sanction written at vote 3 and spawn another `enqueue_sanction_event`. The test
fixes this with a `break` at quorum, but the API itself must guard: only enqueue
when `outcome.case_decided` is true (i.e. this vote was the quorum-crossing vote).

**Change:** Wrap the `if let Some(sanction) = published` block with an outer guard
on `outcome.case_decided`. The outcome is already in scope as the variable `outcome`
returned from `run_transaction`. Change:

```rust
  if let Some(sanction) = published {
    if sanction.target_person_id.is_some() {
```

to:

```rust
  if outcome.case_decided {
    if let Some(sanction) = published {
      if sanction.target_person_id.is_some() {
```

And close the extra `if outcome.case_decided` block (add one extra `}` at the end
of that block, before `Ok(Json(outcome))`).

Exact anchor to find (unique in file):
```
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

Replace with:
```rust
  if outcome.case_decided {
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
  }
```

### Fix 2 — `sanction_publisher.rs`: return Err on missing governance_log row

**File:** `crates/api/api/src/governance/sanction_publisher.rs`

**Problem (CR-3):** `row.map(|r| hex::encode(&r.entry_hash)).unwrap_or_default()`
silently produces an empty string when no `governance_log` row exists for this
`sanction_created` event. An empty `governance_log_entry_hash` in the payload is
an invalid B-publish event.

**Change:** Replace `.unwrap_or_default()` with an explicit error. Find (unique):

```rust
    row.map(|r| hex::encode(&r.entry_hash)).unwrap_or_default()
```

Replace with:

```rust
    match row {
      Some(r) => hex::encode(&r.entry_hash),
      None => {
        return Err(LemmyError::from(anyhow::anyhow!(
          "sanction-created governance_log entry not found for case_id {}",
          sanction.case_id.0
        )))
      }
    }
```

The `LemmyError::from` + `anyhow::anyhow!` pattern is the project standard
(see `feedback_lemmy_error_no_std_error.md`). `anyhow` is already in scope in
this crate (check `Cargo.toml` for `anyhow`; if missing, use `LemmyError::from_message`
or the equivalent available in `lemmy_utils::LemmyError`). Look at sibling
functions in the file to find the exact error-construction idiom used there —
use the same pattern verbatim.

### Fix 3 — `sanction_publisher.rs`: reactivate deactivated subscribers on seed

**File:** `crates/api/api/src/governance/sanction_publisher.rs`

**Problem (CR-2 — medium, but also a correctness issue):** `do_nothing()` on
conflict means a previously deactivated subscriber (`active = false`) at the same
URL will not be re-enabled. Startup seeding should reactivate it.

**Change:** In `seed_sanction_subscriber`, replace the `do_nothing()` conflict
handler. Find (unique):

```rust
  insert_into(sanction_subscriber_dsl::table)
    .values(&form)
    .on_conflict(sanction_subscriber_dsl::callback_url)
    .do_nothing()
    .execute(conn)
    .await?;
```

Replace with:

```rust
  insert_into(sanction_subscriber_dsl::table)
    .values(&form)
    .on_conflict(sanction_subscriber_dsl::callback_url)
    .do_update()
    .set(sanction_subscriber_dsl::active.eq(true))
    .execute(conn)
    .await?;
```

### Fix 4 — `lib.rs`: trim and empty-check env var before seeding

**File:** `crates/server/src/lib.rs`

**Problem (CR-5):** `std::env::var("BRIDGE_SANCTION_CALLBACK_URL")` succeeds on
whitespace-only values, passing a blank URL to `seed_sanction_subscriber`. The DB
`UNIQUE` constraint will accept it, causing a permanently-unusable subscriber row.

**Change:** Find (unique):

```rust
  if let Ok(url) = std::env::var("BRIDGE_SANCTION_CALLBACK_URL") {
    lemmy_api::governance::sanction_publisher::seed_sanction_subscriber(
      &url,
      &mut (&pool).into(),
    )
    .await?;
  }
```

Replace with:

```rust
  if let Ok(url) = std::env::var("BRIDGE_SANCTION_CALLBACK_URL") {
    let url = url.trim().to_string();
    if !url.is_empty() {
      lemmy_api::governance::sanction_publisher::seed_sanction_subscriber(
        &url,
        &mut (&pool).into(),
      )
      .await?;
    }
  }
```

### Fix 5 — `m2_late.rs`: change test flavor to `current_thread`

**File:** `crates/server/tests/e2e/m2_late.rs`

**Problem (CR-7):** `#[tokio::test(flavor = "multi_thread")]` combined with
`EnvVarGuard::set` (which calls `std::env::set_var`) is technically UB in Rust
1.77+ — `set_var` is not safe to call when other threads exist.

**Change:** Find (unique):

```rust
  #[tokio::test(flavor = "multi_thread")]
  pub async fn sanction_event_delivered_to_subscriber() -> LemmyResult<()> {
```

Replace with:

```rust
  #[tokio::test(flavor = "current_thread")]
  pub async fn sanction_event_delivered_to_subscriber() -> LemmyResult<()> {
```

### Fix 6 — `up.sql`: add CHECK constraint on effective window

**File:** `migrations/2026-06-07-000000-0000_add_sanction_event/up.sql`

**Problem (CR-8):** The `sanction_event` table has no constraint preventing
`effective_until <= effective_from`. Add a CHECK constraint.

**Change:** Find (unique, within the CREATE TABLE sanction_event block):

```sql
  effective_until TIMESTAMPTZ NULL,
```

Replace with:

```sql
  effective_until TIMESTAMPTZ NULL,
  CONSTRAINT sanction_event_valid_window_chk CHECK (effective_until IS NULL OR effective_until > effective_from),
```

### Fix 7 — `sanction_handler.rs`: fix `applied` field + extend kind mappings

**File:** `services/bridge/src/sanction_handler.rs`

**Problem (CR-9, two parts):**
(a) `applied: true` is semantically wrong — enforcement is deferred to m2-late-2.
    The response should signal `applied: false` (received but not enforced).
(b) `sanction_kind_to_power_level` only handles `"ban"` and `"mute"` but the wire
    format now includes `"prevent_post"`, `"mute_voice"`, `"hide_content"`,
    `"restrict_reach"` (JSON snake_case from `#[serde(rename_all = "snake_case")]`).

**Change (a):** Find (unique):

```rust
        Json(SanctionEventResponse {
            applied: true,
            reason: format!(
```

Replace with:

```rust
        Json(SanctionEventResponse {
            applied: false,
            reason: format!(
```

**Change (b):** Find (unique):

```rust
fn sanction_kind_to_power_level(sanction_kind: &str) -> i32 {
    match sanction_kind {
        "ban" | "mute" => 0,
        _ => 50,
    }
}
```

Replace with:

```rust
fn sanction_kind_to_power_level(sanction_kind: &str) -> i32 {
    match sanction_kind {
        "ban" | "mute" | "prevent_post" | "mute_voice" => 0,
        "hide_content" | "restrict_reach" => 25,
        _ => 50,
    }
}
```

## 5. Validation gate (write DQ and stop — do NOT run cargo)

After all edits are committed and pushed, write a `validate-pending-laptop` DQ
entry to `.claude/decision-queue.json` using
`bash scripts/brehon/dq-v3-append-fragment.sh` with:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "timestamp": "<ISO8601 now>",
  "question": "cargo check + clippy pass after fix-impl-1 CR fixes?",
  "options": ["pass", "fail"],
  "context": "fix-impl-1: 6 CR fix-in-pr items across 5 files. No migration schema changes (up.sql CHECK constraint addition is additive).",
  "commands": [
    "./scripts/brehon/cargo-check.sh --workspace --features full",
    "./scripts/brehon/cargo-check.sh -p lemmy_server --features full 2>&1 | grep -c 'error' | grep -q '^0$'"
  ],
  "branch": "phase-m2-late-1",
  "phase_task": "fix-impl-1",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null,
  "approved_by": null,
  "approved_at": null
}
```

Commit the DQ write:
```
chore(decision-queue): impl raised validate-pending-laptop for fix-impl-1 CR fixes
```

Then **STOP**. Do not run cargo. The laptop advisor runs the validation.

## 6. Constraints

- **Do NOT run cargo check, clippy, or any cargo command** on the daemon. Write
  the validate-pending-laptop DQ and stop. This is a hard rule (NO CARGO ON ELITEDESK).
- **One worktree, one branch**: all commits go to `phase-m2-late-1`.
- **Commit granularity**: one commit for all 7 code changes + one commit for the DQ
  write. Or split by file if cleaner. Do not squash the DQ commit with the code changes.
- **Attribution**: `answered_by: null` in the DQ — only the laptop advisor sets
  `answered_by: "advisor-laptop"`.
- **Migration note**: the CHECK constraint in `up.sql` is being added to an already-
  written migration. This migration has been applied in the e2e test environment
  but NOT in production (not yet merged). Adding the constraint to the migration
  file is correct — it will be applied atomically when the migration runs in prod.
