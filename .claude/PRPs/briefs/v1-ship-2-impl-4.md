# Brief: v1-ship-2 impl Task 4 — Test 4 create_endorsement_happy_path_and_self_endorse_rejects

## 1. Role + dispatch line

`[role:impl-task] v1-ship-2 Task 4 endorsement test — see .claude/PRPs/briefs/v1-ship-2-impl-4.md`

## 2. Scope

### 2.1 §G4 canonical recipe (Case A discipline — mandatory verbatim citation)

> | Failure signature | Auto-fix | Source lesson |
> | Any test fn in e2e.rs | All test fn signatures MUST be `async fn <name>() -> LemmyResult<()>`. All helper fn signatures MUST be `async fn <name>(...) -> LemmyResult<T>`. Bare `?` propagation throughout. NO `Box<dyn Error>`. NO `.map_err(|e| format!("{e}").into())` closures. | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.2 What to produce

Append a new test fn `create_endorsement_happy_path_and_self_endorse_rejects` **inside** `mod v1_ship_2_fixtures` in `crates/server/tests/e2e.rs`. Tasks 1, 2, and 3 already landed the module scaffold and the first three test fns. This task only appends one test fn.

**Edit discipline (MANDATORY per `feedback_junior_worker_e2e_edit_hang.md`):**
- `old_string` targets ≤5 lines: the closing `}` of Task 3's test fn + blank line + module's closing `}`. NEVER target the middle of any existing test fn.
- `new_string` re-emits Task 3's test fn closing `}` + blank line + new test fn body + blank line + module's closing `}`.

**Step A — Read before writing (MANDATORY):**

Before any Edit, scan the end of `mod v1_ship_2_fixtures` (last ~30 lines) to identify the exact `old_string` target. Task 3's worker named its test fn `get_my_reputation_happy_path_and_no_auth` — verify by reading the actual current content. The `old_string` is Task 3's fn closing `}` + blank line + module's `}`.

**Commit subject:** `feat(e2e): v1-ship-2 Task 4 — endorsement test (task 4)`

### 2.3 IMPLEMENT steps

**File: `crates/server/tests/e2e.rs`**

**Test fn `create_endorsement_happy_path_and_self_endorse_rejects` inside the module:**

Signature: `async fn create_endorsement_happy_path_and_self_endorse_rejects() -> LemmyResult<()>`
Attribute: `#[tokio::test(flavor = "multi_thread")]`

Steps (verbatim from plan §13 Task 4):

1. `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;`
2. `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
3. Seed two users:
   - `let (sponsor_pid, sponsor_lu_view) = governance_fixtures::seed_user(&context, instance.id, "ship2_end_sponsor", false).await?;`
   - `let (sponsee_pid, _sponsee_lu_view) = governance_fixtures::seed_user(&context, instance.id, "ship2_end_sponsee", false).await?;`
4. Bypass the age gate by setting `onboarding.sponsor_gate_strategy = "open"` in `governance_config`. Use `AsyncPgConnection::establish(&db_url).await?` + Diesel insert (read `GovernanceConfigInsertForm` field names verbatim from `crates/db_schema/src/source/governance/governance_config.rs`):

   ```rust
   let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
   diesel::insert_into(governance_config::table)
     .values(&GovernanceConfigInsertForm {
       scope: "instance".to_string(),
       key: "onboarding.sponsor_gate_strategy".to_string(),
       value_type: "text".to_string(),
       value_text: Some("open".to_string()),
       ..Default::default()
     })
     .execute(&mut async_conn)
     .await?;
   ```

5. `let sponsor_jwt = mint_jwt(&context, sponsor_lu_view.local_user.id).await?;`
6. Build actix App per §10.2 (mirror `e2e.rs:4131-4137`). Include rate-limit bucket override block from `e2e.rs:4110-4122`.
7. (happy path): POST `/api/v4/governance/endorsement` with `Authorization: Bearer {sponsor_jwt}`, `Content-Type: application/json`, body `{"person_id": <sponsee_pid.0>}`. Assert status == 200. `let body: CreateEndorsementResponse = test::read_body_json(resp).await;`. Assert `body.endorsement_id.0 > 0`. Assert `body.surety_created == true` (fresh sponsee, no prior sureties).
8. (self-endorse failure): POST same URI with `Authorization: Bearer {sponsor_jwt}`, body `{"person_id": <sponsor_pid.0>}` (self-target). Assert status == 404. (Handler returns `LemmyErrorType::NotFound` at create_endorsement.rs:178-180.)
9. `Ok(())`

**GOTCHA — age gate fires BEFORE self-endorse check.** `enforce_age_gate` at create_endorsement.rs:169-176 fires before the self-endorse check at line 178. Step 4's INSERT into `governance_config` is mandatory — without it both the happy path AND the self-endorse arm will return non-200 from the age gate. Verify create_endorsement.rs:160-210 line ordering at task-time; if a refactor reordered the steps, file a `kind: "blocker"` DQ.

**GOTCHA — `Default::default()` on GovernanceConfigInsertForm.** The struct derives `Clone, Default`. Set `scope`, `key`, `value_type`, `value_text: Some("open".to_string())` and use `..Default::default()` for the remaining null fields (`value_int: None`, `value_float: None`, `value_bool: None`, `updated_by: None`).

**Schema imports needed** (add inside test fn or at module level if not already present):
- `use lemmy_db_schema::source::governance::governance_config::GovernanceConfigInsertForm;`
- `use lemmy_db_schema_file::schema::governance_config;`
- `use lemmy_api_common::governance::CreateEndorsementResponse;` (already in module import block from Task 1 scaffold)

### 2.4 Validate gate (Linux wrapper — EliteDesk runs Linux)

```bash
./scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-ship-2-task4-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task4-check.log
# EXPECT: exit 0

./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task4-clippy.log 2>&1
echo "exit: $?"
# EXPECT: exit 0

./scripts/brehon/cargo-test.sh --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-2-task4-test-no-run.log 2>&1
echo "exit: $?"
# EXPECT: exit 0
```

If validate fails: raise `kind: "blocker"` DQ with log slice + commit + push immediately.

After validate passes: raise `kind: "validate-pending-laptop"` DQ entry with:
```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "commands": [
    "./scripts/brehon/cargo-check.sh --workspace --features full",
    "./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings",
    "./scripts/brehon/cargo-test.sh --workspace --test e2e --features full --no-run"
  ],
  "branch": "phase-v1-ship-2",
  "phase_task": 4,
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```

### 2.5 Explicit boundaries

- **DO NOT** touch Tasks 1, 2, or 3's test fns or any existing test fn or module.
- **DO NOT** add new handler files, DTO files, or migrations.
- **DO NOT** insert in the middle of `mod v1_ship_2_fixtures`.
- **COMMIT ONLY** `crates/server/tests/e2e.rs`.
- **DO NOT** write `answered_by: "advisor"` in any DQ entry.

## 3. Required reading (read before first Edit)

**In order:**

1. `crates/server/tests/e2e.rs` — scan end of `mod v1_ship_2_fixtures` (last ~30 lines) to identify exact `old_string` target (Task 3's test fn closing `}` + blank line + module `}`)
2. `crates/api/api/src/governance/create_endorsement.rs:160-210` — verify step ordering: age gate (169-176) fires BEFORE self-endorse check (178-180); if reordered, file blocker DQ
3. `crates/api/api_common/src/governance.rs:302-310` — `CreateEndorsementResponse` DTO (verify `endorsement_id` + `surety_created` field names)
4. `crates/db_schema/src/source/governance/governance_config.rs:44-53` — `GovernanceConfigInsertForm` struct (actual field names verbatim)
5. `crates/server/tests/e2e.rs:4326-4337` — existing endorsement probe shape (§10.2 mirror)
6. `crates/server/tests/e2e.rs:4131-4137` — actix App composition pattern
7. `crates/server/tests/e2e.rs:4110-4122` — rate-limit bucket override block
8. `crates/server/tests/e2e.rs:793-852` — `governance_fixtures::bootstrap` + `seed_user` signatures
9. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (mandatory per file-class table)
10. `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` pattern
11. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — append-only Edit discipline (≤5-line `old_string`)
12. `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate

## 4. Constraints

1. **Case A is non-negotiable.** All `async fn` in this module return `LemmyResult<()>` (test fns) or `LemmyResult<T>` (helpers). Zero `Box<dyn Error>`. Zero `.map_err` bridges.
2. **Append-only Edit.** `old_string` is ≤5 lines targeting Task 3's test fn closing `}` + blank line + module's closing `}`. Never target the middle of an existing test fn.
3. **DQ mid-task push.** Any `kind: "blocker"` or `kind: "validate-pending-laptop"` entry must be committed + pushed immediately. Use `bash scripts/brehon/dq-v3-append-fragment.sh`.
4. **Linux invocation.** Use `./scripts/brehon/cargo-*.sh` (not `.bat`).
5. **404 not 409.** The self-endorse failure mode is 404 (LemmyErrorType::NotFound). Do not assert 403 or 409.
6. **Age gate bypass is mandatory.** Step 4's governance_config INSERT must run before Steps 7-8. Without it, the age gate rejects the sponsor and both arms fail.
7. **One commit.** Commit subject: `feat(e2e): v1-ship-2 Task 4 — endorsement test (task 4)`. Include `LESSON:` trailer if any surprising finding encountered.
