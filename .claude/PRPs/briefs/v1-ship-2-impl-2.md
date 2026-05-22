# Brief: v1-ship-2 impl Task 2 — Test 2 modlog_happy_path_and_unauthenticated_access

## 1. Role + dispatch line

`[role:impl-task] v1-ship-2 Task 2 modlog test — see .claude/PRPs/briefs/v1-ship-2-impl-2.md`

## 2. Scope

### 2.1 §G4 canonical recipe (Case A discipline — mandatory verbatim citation)

> | Failure signature | Auto-fix | Source lesson |
> | Any test fn in e2e.rs | All test fn signatures MUST be `async fn <name>() -> LemmyResult<()>`. All helper fn signatures MUST be `async fn <name>(...) -> LemmyResult<T>`. Bare `?` propagation throughout. NO `Box<dyn Error>`. NO `.map_err(|e| format!("{e}").into())` closures. | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.2 What to produce

Append a new test fn `modlog_happy_path_and_unauthenticated_access` **inside** `mod v1_ship_2_fixtures` in `crates/server/tests/e2e.rs`. The module and its scaffold (imports + `mint_jwt` helper) were landed by Task 1. This task only appends a test fn.

**Edit discipline (MANDATORY per `feedback_junior_worker_e2e_edit_hang.md`):**
- `old_string` targets ≤5 lines: the closing `}` of Task 1's test fn + blank line + module's closing `}`. NEVER target the middle of any existing test fn.
- `new_string` re-emits Task 1's test fn closing `}` + blank line + new test fn body + blank line + module's closing `}`.

**Commit subject:** `feat(e2e): v1-ship-2 Task 2 — modlog test (task 2)`

### 2.3 IMPLEMENT steps

**File: `crates/server/tests/e2e.rs`**

**Step A — Read before writing (MANDATORY):**

Before any Edit, read in this order:
1. `crates/server/tests/e2e.rs` — scan the end of `mod v1_ship_2_fixtures` (the last ~20 lines of the module) to identify the exact `old_string` target (Task 1's test fn closing `}` + blank line + module `}`).
2. `crates/db_schema/src/source/governance/public_case_log.rs` — read the `PublicCaseLogInsertForm` struct fields verbatim. Use the actual field names in your INSERT, not the plan's placeholder names.
3. `crates/db_views/governance_modlog/src/lib.rs` — read the `GovernanceModlogView` struct. Identify the pseudonym field name (may differ from `actor_pseudonym`) before writing the ADR-015 assertion.

**Step B — Test fn `modlog_happy_path_and_unauthenticated_access` inside the module:**

Signature: `async fn modlog_happy_path_and_unauthenticated_access() -> LemmyResult<()>`
Attribute: `#[tokio::test(flavor = "multi_thread")]`

Steps (verbatim from plan §13 Task 2):

1. `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;`
2. `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
3. `let (target_pid, _target_lu_view) = governance_fixtures::seed_user(&context, instance.id, "ship2_modlog_target", false).await?;`
4. Seed a Decided moderation_case: use the same pattern as Task 1 (INSERT via `ModerationCaseInsertForm` + UPDATE `appeal_window_expires_at` + `panel_size_snapshot`). Capture the inserted `case_id` via `.returning(moderation_case::id).get_result(...)`. Do NOT hardcode `case_id: 1`.
5. Insert a `PublicCaseLog` row: use `PublicCaseLogInsertForm` with `case_id = <seeded_case_id>` and a literal pseudonym string (e.g. `"test_pseudonym_xyz123"`). Read the actual InsertForm field names from `crates/db_schema/src/source/governance/public_case_log.rs` before writing. Use `diesel::insert_into(public_case_log::table).values(&pcl_form).execute(&mut async_conn).await?;`.
6. Build actix App per §10.2 pattern (mirror `e2e.rs:4131-4137`). Include rate-limit bucket override block from `e2e.rs:4110-4122`.
7. (happy path, unauthenticated): `GET /api/v4/governance/modlog` with NO Authorization header. Assert status == 200. `let body: Vec<GovernanceModlogView> = test::read_body_json(resp).await;`. Assert `body.len() >= 1`. Assert the first entry's pseudonym field equals `"test_pseudonym_xyz123"` — using the actual field name from `GovernanceModlogView` at step-A read time. If the body contains a top-level `person_id` field, file a `kind: "blocker"` DQ and STOP (ADR-015 violation — do not patch around it).
8. (second arm, authenticated succeeds): seed a second user `"ship2_modlog_reader"`, mint a JWT via `mint_jwt(&context, reader_lu_view.local_user.id).await?`. GET same URI with `Authorization: Bearer {jwt}`. Assert status == 200 AND `body.len() >= 1`.
9. `Ok(())`

**Schema import needed:** add `use lemmy_db_schema::source::governance::public_case_log::PublicCaseLogInsertForm;` inside the test fn OR at the top of the module (check if Task 1 already imports it — if not, add at module level or use the full path inside the fn).

**`public_case_log` schema import:** also add `use lemmy_db_schema_file::schema::public_case_log;` if not already in the module.

### 2.4 Validate gate (Linux wrapper — EliteDesk runs Linux)

```bash
./scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-ship-2-task2-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task2-check.log
# EXPECT: exit 0

./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task2-clippy.log 2>&1
echo "exit: $?"
# EXPECT: exit 0

./scripts/brehon/cargo-test.sh --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-2-task2-test-no-run.log 2>&1
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
  "phase_task": 2,
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```

### 2.5 Explicit boundaries

- **DO NOT** touch Task 1's test fn or any existing test fn or module.
- **DO NOT** add new handler files, DTO files, or migrations.
- **DO NOT** insert in the middle of `mod v1_ship_2_fixtures`.
- **COMMIT ONLY** `crates/server/tests/e2e.rs`.
- **DO NOT** write `answered_by: "advisor"` in any DQ entry.
- **If `GovernanceModlogView` leaks `person_id` in the response body:** file a `kind: "blocker"` DQ citing ADR-015, STOP. Do NOT patch the modlog handler inline.

## 3. Required reading (read before first Edit)

**In order:**

1. `crates/server/tests/e2e.rs` — scan end of `mod v1_ship_2_fixtures` to identify exact `old_string` target (last ~20 lines of the module, including Task 1's test fn closing `}` + module `}`)
2. `crates/db_schema/src/source/governance/public_case_log.rs` — `PublicCaseLogInsertForm` struct (actual field names; use verbatim)
3. `crates/db_views/governance_modlog/src/lib.rs` — `GovernanceModlogView` struct (pseudonym field name for ADR-015 assertion)
4. `crates/api/api/src/governance/list_modlog.rs:1-49` — modlog handler (confirms `Option<LocalUserView>` = public endpoint)
5. `crates/server/tests/e2e.rs:4261-4324` — Decided-case + PublicCaseLog seeding pattern (§10.4 mirror; copy verbatim)
6. `crates/server/tests/e2e.rs:4131-4137` — actix App composition pattern (§10.2 mirror)
7. `crates/server/tests/e2e.rs:4110-4122` — rate-limit bucket override block
8. `crates/server/tests/e2e.rs:793-852` — `governance_fixtures::bootstrap` + `seed_user` signatures
9. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (mandatory per file-class table)
10. `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` pattern
11. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — append-only Edit discipline (≤5-line `old_string`)
12. `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate

## 4. Constraints

1. **Case A is non-negotiable.** All `async fn` in this module return `LemmyResult<()>` (test fns) or `LemmyResult<T>` (helpers). Zero `Box<dyn Error>`. Zero `.map_err` bridges. Wrap Diesel non-LemmyError calls with `.map_err(|e| LemmyErrorType::Unknown(e.to_string()).into())?` only when strictly necessary.
2. **Append-only Edit.** `old_string` is ≤5 lines targeting Task 1's test fn closing `}` + blank line + module's closing `}`. Never target the middle of an existing test fn.
3. **DQ mid-task push.** Any `kind: "blocker"` or `kind: "validate-pending-laptop"` entry must be committed + pushed immediately per `.claude/rules/decision-queue.md` "Mid-task visibility". Use `bash scripts/brehon/dq-v3-append-fragment.sh` to write the DQ entry.
4. **Linux invocation.** Use `./scripts/brehon/cargo-*.sh` (not `.bat`).
5. **Do not hardcode case_id.** Capture the returned id via `.returning(moderation_case::id).get_result(&mut async_conn).await?`.
6. **ADR-015 assertion.** Assert the pseudonym field matches the literal string seeded. If body leaks `person_id`, file a blocker DQ — do NOT patch the handler.
7. **One commit.** Commit subject: `feat(e2e): v1-ship-2 Task 2 — modlog test (task 2)`. Include `LESSON:` trailer if any surprising finding encountered.
