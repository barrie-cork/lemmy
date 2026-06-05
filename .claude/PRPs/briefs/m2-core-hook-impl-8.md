---
phase: m2-core-hook
role: impl-task
n: 8
authored: 2026-06-05
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m2-core-hook
task_number: 8
requires: [4, 5, 6, 7]
minimax_trial: not-eligible (e2e test file)
---

# [role:impl-task] m2-core-hook Task 8 — add e2e tests (wrapper + suppression)

## 1. Role + dispatch line

```
[role:impl-task] m2-core-hook task-8 add e2e tests for append_room_event and hook suppression — see .claude/PRPs/briefs/m2-core-hook-impl-8.md
```

## 2. Scope

Add 3 deterministic e2e tests + 1 `kind: log` DQ entry (proposing a test seam) to `crates/server/tests/e2e/governance.rs`. **One file modified.** **Pre-Shape-G: write a `validate-pending-laptop-e2e` DQ entry, commit + push, then STOP.**

**Tests to add (append after the final `}` of `admin_emergency_remove_post_sets_author_defendant`):**

| Test fn | What it asserts |
|---|---|
| `m2_append_room_event_writes_chain_entry` | `append_room_event(pool, ENTRY_KIND_ROOM_CREATED, …)` inserts a `governance_log` row with `entry_kind = 'room_created'` and a non-empty `prev_hash` |
| `m2_append_room_event_rejects_non_room_kind` | `append_room_event(pool, "report_created", …)` returns `Err` containing "not a room entry kind" |
| `m2_hook_suppressed_when_messaging_disabled` | Default seed (`messaging_enabled` absent or false), drive `admin_assign_jury`, assert NO HTTP POST was attempted — verified by driving the handler and confirming it returns `Ok` (transport errors would panic; mock server not needed for suppression) |

**Do NOT implement the hook-fires assertion** (`m2_transition_hook_fires_when_messaging_enabled`). Instead raise a `kind: log` DQ proposing a `BRIDGE_NOTIFY_URL` env-var override test seam.

**Do NOT:**
- Modify any file other than `crates/server/tests/e2e/governance.rs` and `.claude/decision-queue.json`.
- Add a new `tests/*.rs` file — all tests go inside `governance.rs` via the existing `include!` wiring.
- Create any new module or `mod` declaration.
- Run any cargo command.
- Commit to `governance-v0` — your worktree branches from `phase-m2-core-hook`.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### Mandatory lessons (e2e file class)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — error-shape rules; use `LemmyResult<()>`; `.map_err` with annotated closure for non-LemmyError sources.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn`; use the test-pool pattern.
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim `old_string` anchors; confirm unique count with `grep -c '<anchor>' crates/server/tests/e2e/governance.rs` before editing.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write DQ entry + push, then STOP.

### MIRROR refs — read these EXACT files on your base branch (`phase-m2-core-hook`)

**Bootstrap pattern:** `governance_events_notify_fires` at line ~3840 of `crates/server/tests/e2e/governance.rs` — read lines 3840–3975. This is the canonical M2 test shape: `start_postgres`, `apply_all_schema`, `build_db_pool_for_tests`, `LemmyContext::create`. Mirror this exactly.

**`append_room_event` fn (Task 5 output):** `crates/api/api/src/governance/governance_log.rs` — read the full file. Your tests call `append_room_event` from here.

**ENTRY_KIND_ROOM_CREATED const:** also in `crates/api/api/src/governance/governance_log.rs` (re-exported from db_schema via the shim).

**Suppression test seam:** `crates/server/tests/e2e/admin_config.rs` lines ~614–640 — `governance_messaging_config` read in tests; shows how `messaging_enabled` is absent by default (migration seeds `messaging_enabled=false` row). Your suppression test just calls the handler without seeding `messaging_enabled=true` and asserts `Ok`.

**`admin_assign_jury` handler for suppression test:** `crates/api/api/src/governance/admin_assign_jury.rs` — read lines 1–80 to understand the handler signature + what case state is needed. You need a seeded case in `ThresholdMet` status to drive the jury assignment.

**Fixture helpers:** at the top of `crates/server/tests/e2e/governance.rs` — look for `governance_fixtures::bootstrap`, `governance_fixtures::start_postgres`, `governance_fixtures::db_url`, `EnvVarGuard::set`. Use these in your tests.

**Jury bootstrap:** `crates/server/tests/e2e/jury_mechanics.rs` lines ~955–1100 — shows how to seed a case at `ThresholdMet` and drive `admin_assign_jury`. Mirror the seeding approach.

**Diesel select from governance_log:** look for existing `governance_log::table` select calls in `governance.rs` — use the same import pattern.

### Plan section
`.claude/PRPs/plans/m2-core-transition-hook.plan.md` §"Task 8: ADD e2e tests".

## 4. Implementation

### 4.0 Anchor verification (MANDATORY before editing)

Run these checks and confirm each returns `1`:
```
grep -c "emergency_remove opens case with status = EmergencyRemove" crates/server/tests/e2e/governance.rs
```
If the count is not `1`, STOP and raise a `kind: blocker` DQ — do not attempt the edit.

### 4.1 Test: `m2_append_room_event_writes_chain_entry`

Append after the final `}` of `admin_emergency_remove_post_sets_author_defendant` (the last function in governance.rs, identified by its closing `Ok(())` + `}` following the `"emergency_remove opens case with status = EmergencyRemove (ADR-013)"` assert).

```rust
// ============================================================================
// M2-core-hook — Task 8: governance_case_after_transition hook e2e tests
// ============================================================================

#[tokio::test(flavor = "multi_thread")]
async fn m2_append_room_event_writes_chain_entry() -> lemmy_utils::error::LemmyResult<()> {
  use diesel_async::AsyncPgConnection;
  use lemmy_api::governance::governance_log::{ENTRY_KIND_ROOM_CREATED, append_room_event};
  use lemmy_api::governance::governance_log::RoomEventPayload;  // adjust path if needed
  use lemmy_db_schema::schema::governance_log;
  use diesel::prelude::*;
  use diesel_async::RunQueryDsl;

  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set(
    "GOVERNANCE_LOG_SIGNING_KEY",
    "0000000000000000000000000000000000000000000000000000000000000001",
  );

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

  {
    use diesel::Connection as _;
    use diesel::pg::PgConnection;
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let pool = lemmy_diesel_utils::connection::build_db_pool_for_tests();
  let mut pool_ref = pool;

  let payload = RoomEventPayload {
    case_id: 1,
    matrix_room_id: Some("local-room-abc123".to_string()),
    lifecycle_stage: "created".to_string(),
    member_count: Some(5),
  };

  let entry = append_room_event(&mut pool_ref, ENTRY_KIND_ROOM_CREATED, payload, None).await?;

  assert_eq!(
    entry.entry_kind, "room_created",
    "m2: append_room_event wrote correct entry_kind"
  );
  assert!(
    !entry.prev_hash.is_empty(),
    "m2: hash-chain prev_hash is populated"
  );

  // Verify row is in the DB
  let mut conn = AsyncPgConnection::establish(&db_url).await?;
  let count: i64 = governance_log::table
    .filter(governance_log::entry_kind.eq("room_created"))
    .count()
    .get_result(&mut conn)
    .await?;
  assert_eq!(count, 1, "m2: exactly one room_created governance_log row");

  Ok(())
}
```

> **Import note:** `append_room_event` and `RoomEventPayload` are in `crates/api/api/src/governance/governance_log.rs`. In the test context (included into `lemmy_server` tests via `include!`), the import path is `lemmy_api::governance::governance_log::append_room_event` (or similar — confirm from existing `lemmy_api::governance::` imports in governance.rs). Adapt the import to match what's already in use at the top of governance.rs.

### 4.2 Test: `m2_append_room_event_rejects_non_room_kind`

```rust
#[tokio::test(flavor = "multi_thread")]
async fn m2_append_room_event_rejects_non_room_kind() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::governance_log::{append_room_event, RoomEventPayload};

  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set(
    "GOVERNANCE_LOG_SIGNING_KEY",
    "0000000000000000000000000000000000000000000000000000000000000001",
  );

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

  {
    use diesel::Connection as _;
    use diesel::pg::PgConnection;
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let pool = lemmy_diesel_utils::connection::build_db_pool_for_tests();
  let mut pool_ref = pool;

  let payload = RoomEventPayload {
    case_id: 1,
    matrix_room_id: None,
    lifecycle_stage: "test".to_string(),
    member_count: None,
  };

  let result = append_room_event(&mut pool_ref, "report_created", payload, None).await;
  assert!(result.is_err(), "m2: non-room kind must be rejected");
  let err_str = format!("{:?}", result.unwrap_err());
  assert!(
    err_str.contains("not a room entry kind"),
    "m2: error message contains expected text, got: {err_str}"
  );

  Ok(())
}
```

### 4.3 Test: `m2_hook_suppressed_when_messaging_disabled`

This test seeds a case and calls `governance_case_after_transition` with `messaging_enabled` absent (default false). Since the function POSTs to `http://localhost:9009` which is not listening in tests, if messaging were enabled the `.send()` would fail — but it's fire-and-forget (`.ok()` or `if let Err`) so it wouldn't panic. The real assertion is: the fn returns `Ok(())` without error (transport swallowed) AND does NOT return an error from the DB reads.

```rust
#[tokio::test(flavor = "multi_thread")]
async fn m2_hook_suppressed_when_messaging_disabled() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::web::Data;
  use lemmy_api_utils::{context::LemmyContext, request::client_builder};
  use lemmy_db_schema::source::{
    instance::Instance,
    moderation_case::{ModerationCase, ModerationCaseInsertForm},
  };
  use lemmy_db_schema_file::enums::{CaseStatus, CaseTargetType};
  use lemmy_diesel_utils::{connection::build_db_pool_for_tests, traits::Crud};
  use lemmy_utils::{rate_limit::RateLimit, settings::SETTINGS};
  use reqwest_middleware::ClientBuilder;
  use lemmy_api_utils::bridge_notify::governance_case_after_transition;

  let _g_init = EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  let _g_gov = EnvVarGuard::set(
    "GOVERNANCE_LOG_SIGNING_KEY",
    "0000000000000000000000000000000000000000000000000000000000000001",
  );

  let (_container, host_port) = governance_fixtures::start_postgres().await?;
  let db_url = governance_fixtures::db_url(host_port);
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);

  {
    use diesel::Connection as _;
    use diesel::pg::PgConnection;
    let mut sync_conn = PgConnection::establish(&db_url)?;
    governance_fixtures::apply_all_schema(&mut sync_conn)?;
  }

  let pool = build_db_pool_for_tests();
  let client = client_builder(&SETTINGS).build()?;
  let middleware_client = ClientBuilder::new(client).build();
  let secret = lemmy_db_schema::source::secret::Secret {
    id: 0,
    jwt_secret: String::new().into(),
  };
  let rate_limit = RateLimit::with_debug_config();
  let context = Data::new(LemmyContext::create(
    pool,
    middleware_client.clone(),
    middleware_client,
    secret,
    rate_limit,
  ));

  // Seed minimal case (messaging_enabled is absent = false by default)
  let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
  let case = ModerationCase::create(
    &mut context.pool(),
    &ModerationCaseInsertForm {
      instance_id: instance.id,
      status: CaseStatus::Active,
      target_type: CaseTargetType::Person,
      target_id: 1,
      target_person_id: None,
      community_id: None,
      opened_by_person_id: None,
      summary: None,
      report_count: Some(1),
    },
  )
  .await?;

  // Should return Ok — messaging_enabled is false, no-op path
  governance_case_after_transition(
    &context,
    &case,
    Some(CaseStatus::Active),
    CaseStatus::ThresholdMet,
  )
  .await?;

  Ok(())
}
```

> **Adapt imports:** read the existing imports in `governance.rs` and `jury_mechanics.rs` to confirm the correct paths for `ModerationCaseInsertForm`, `CaseStatus`, etc. Use what's already imported where possible.

### 4.4 `kind: log` DQ for BRIDGE_NOTIFY_URL seam

After writing the tests but BEFORE the validate-pending DQ entry, raise this DQ entry (goes directly to `resolved[]` — it's a log, not a blocker):

```json
{
  "from": "impl",
  "kind": "log",
  "question": "Test seam needed: BRIDGE_NOTIFY_URL is hardcoded in bridge_notify.rs — no env-var override for tests.",
  "options": ["add BRIDGE_NOTIFY_URL env-var override", "add a test-only mock server fixture"],
  "context": "governance_case_after_transition POSTs to http://localhost:9009/brehon/notify (hardcoded const). The suppression test (messaging_enabled=false) is deterministic. The fires test (messaging_enabled=true) requires either: (a) env-var override so tests can point to a mock server, or (b) a wiremock/httpmock fixture. Recommending option (a): read BRIDGE_NOTIFY_URL from env with fallback to the hardcoded const, then point tests at a one-shot mock server (like governance_events_notify_fires uses PG NOTIFY). This is a Task 9 scope item.",
  "answer": "file as advisor lesson; propose BRIDGE_NOTIFY_URL env-var override as a follow-on Task 9",
  "answered_by": "impl-self-resolved"
}
```

Append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json>` (without `--pending` — it goes to resolved directly).

### 4.5 validate-pending-laptop-e2e DQ entry

After the log DQ and the commit, append this entry `--pending`:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop-e2e",
  "branch": "<your worktree branch>",
  "phase_task": 8,
  "commands": [
    "./scripts/brehon/cargo-test.bat --workspace --test e2e m2_ 2>&1"
  ],
  "e2e_filter": "test(m2_)",
  "question": "e2e tests for m2_append_room_event_writes_chain_entry, m2_append_room_event_rejects_non_room_kind, m2_hook_suppressed_when_messaging_disabled — laptop runs cargo test.",
  "options": ["pass", "fail"],
  "context": "Task 8 added 3 deterministic e2e tests (2 append_room_event wrapper tests + 1 suppression test). Hook-fires test deferred to Task 9 (needs BRIDGE_NOTIFY_URL seam — log DQ raised). Tests append to governance.rs after admin_emergency_remove_post_sets_author_defendant.",
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

### 4.6 Commit + push discipline

- **Commit subject:** `feat(e2e): add m2 append_room_event + suppression e2e tests (task 8)`
- Stage exactly:
  ```
  git add crates/server/tests/e2e/governance.rs \
          .claude/decision-queue.json
  git commit -m "feat(e2e): add m2 append_room_event + suppression e2e tests (task 8)"
  git push origin <your worktree branch>
  ```
- Then **STOP**. Do not run cargo.

### 4.7 Attribution
- Log DQ: `answered_by: "impl-self-resolved"`.
- Validate-pending DQ: `answered_by: null`.
- NEVER `answered_by: "advisor"` and NEVER `approved_by`.

### Mandatory lessons fired for this brief
- e2e file class → `feedback_lemmy_error_no_std_error.md` ✓
- e2e file class → `feedback_async_pool_test_pattern.md` ✓
- e2e file (≥2 edits) → `feedback_fix_impl_pre_locate_e2e_anchors.md` ✓
- validate-pending → `feedback_validate_pending_laptop_write_then_stop.md` ✓

## HANDOVER

```yaml
HANDOVER:
  task: m2-core-hook-task-8
  filesCreated: []
  filesModified:
    - crates/server/tests/e2e/governance.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "3 tests: m2_append_room_event_writes_chain_entry, m2_append_room_event_rejects_non_room_kind, m2_hook_suppressed_when_messaging_disabled"
    - "hook-fires test deferred to Task 9 (BRIDGE_NOTIFY_URL seam needed — log DQ raised)"
    - "suppression test: no mock server needed (messaging_enabled=false = no-op = Ok())"
    - "append_room_event tests: require Docker (testcontainers) — laptop runs e2e suite"
  notes: "After Task 8 merges, all phase-m2-core-hook work is done — bm-pr can proceed."
```
