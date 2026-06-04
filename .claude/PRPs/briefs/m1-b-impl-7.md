# m1-b Task 7 — e2e tests (clean-posture #3 + validator #5)

## 1. Role + dispatch line

`[role:impl-task]` m1-b Task 7 — add two e2e tests to `crates/server/tests/e2e.rs`: `messaging_enabled=false` clean-posture + identity-policy validator rejection. Single Sonnet arm (e2e is NOT MiniMax-eligible).

## 2. Scope

Add **exactly two** new `#[tokio::test(flavor = "multi_thread")]` async fns to `crates/server/tests/e2e.rs`, mirroring the existing `admin_set_config_*` block **Case A verbatim** (`async fn <name>() -> lemmy_utils::error::LemmyResult<()>`, bare `?`, the `admin_config_fixtures` seed helpers).

**Commit ONLY** `crates/server/tests/e2e.rs` + `.claude/decision-queue.json` (the validate-pending DQ you raise). Do NOT touch `crates/api/**`, `crates/db_schema/**`, `Cargo.toml`, or any other file.

**Boundaries:** do NOT author production code; do NOT add new seed/auth helpers (reuse `admin_config_fixtures::*`); do NOT run cargo yourself (laptop validates — see §4); do NOT edit any test other than the two you add.

### 2.1 The two tests

**Test 1 — `messaging_disabled_preserves_governance_posture`** (criterion #3, clean posture):
- With the default `messaging_enabled=false` (no config row written), exercise a representative governance flow + a PM send, and assert the governance e2e behaviour is unchanged (no bridge surface, no error from the bridge_notify no-op path). `bridge_notify::notify_if_enabled` returns `Ok(())` immediately when `messaging_enabled` is false/absent — the test confirms the PM path + a governance assertion both succeed with no messaging config present.
- Mirror the seed/bootstrap of `admin_set_config_happy_path` (`:6235`): `admin_config_fixtures::bootstrap()` → `bootstrap_instance` → `seed_user(..., is_admin: true)`. Then perform a PM send (use the existing PM-send path already exercised elsewhere in e2e.rs if one exists; otherwise assert that with no `governance_messaging_config` row, a `GovernanceMessagingConfig::read_current(pool, "instance", "messaging_enabled")` returns `None` → the clean-posture invariant). Keep it minimal and Case-A-shaped.

**Test 2 — `messaging_identity_policy_rejects_jury_override`** (criterion #5, validator rejection):
- Call the Task-5 handler **directly** (NOT an HTTP path — mirror the `admin_set_config_*` direct-handler idiom) with an identity-policy that the validator must reject:
  ```rust
  use lemmy_api::governance::messaging_config::admin_set_messaging_config;
  use lemmy_api_common::governance::AdminSetMessagingConfig;
  use actix_web::web::Json;
  // ... bootstrap + seed_user(is_admin: true) as in admin_set_config_happy_path ...
  let result = admin_set_messaging_config(
    Json(AdminSetMessagingConfig {
      scope: "jury".to_string(),
      key: "identity_policy".to_string(),
      value: serde_json::json!("real_name"),
    }),
    admin_view,            // the LocalUserView from seed_user (admin)
    context.clone(),
  ).await;
  assert!(result.is_err(), "identity_policy=real_name for jury scope must be rejected (ADR-015)");
  ```
- The validator (`messaging_config.rs:69`) rejects when `key == "identity_policy" && scope.starts_with("jury"|"appeal") && value != "pseudonymous"` with `LemmyErrorType::Unknown(...)`. The test only needs to assert `result.is_err()` (the validator returns the error before any DB write — no governance_log row to check, unlike the `non_admin` rejection test). If you want a stronger assertion, match the error string contains `"pseudonymous"` — but `is_err()` is the contract.
- Seed an **admin** user (so the rejection is from the validator, not the `is_admin` gate) — `seed_user(&context, instance.id, "admin_ip", true)`.

## 3. Required reading (read these FIRST, before any Edit)

**Mandatory file-class lessons** (e2e.rs edit, ≥2 edits, Case A):
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **Case A**: test fn + all helpers return `lemmy_utils::error::LemmyResult<()>`, bare `?`, NO `.map_err` bridges. The `admin_set_config_*` sibling is already Case A — mirror it verbatim.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` for any direct DB read.
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — **pre-locate every verbatim `old_string`/`new_string` anchor before editing**; e2e.rs is ~18.5k lines and anchor collisions hang the worker.

**Plan + sibling:**
- `.claude/PRPs/plans/m1.plan.md` §13 Task 7 (line 556) + R4 (line 122, e2e error-shape Case A).
- The sibling block to mirror: `crates/server/tests/e2e.rs:6235-6702` (`admin_set_config_*`). The Case A signature, `admin_config_fixtures::bootstrap/bootstrap_instance/seed_user` helpers, and the rejection-assert idiom (`:6504` `admin_set_config_non_admin_rejected_with_denial_log`) are all there.

## 4. Constraints

1. **Case A error-shape (verbatim).** Both new fns: `#[tokio::test(flavor = "multi_thread")]` (NOT bare `#[tokio::test]`) + `async fn <name>() -> lemmy_utils::error::LemmyResult<()>` + bare `?`. Mirror `admin_set_config_happy_path` exactly. Per `feedback_lemmy_error_no_std_error.md` Case A.

2. **Uniqueness gate (pre-dispatch, MANDATORY).** Before each Edit, confirm the `old_string` anchor is unique: `grep -c '<anchor>' crates/server/tests/e2e.rs` must return `1`. The verified-safe insertion anchor is the doc-comment line at 6705:
   ```
   /// Task 8 test 9: GET /admin/config without filters returns every
   ```
   (grep -c == 1 on phase-m1-b@a54dea3d7). **Insert both new tests immediately BEFORE that line** (after the closing `}` of `admin_set_config_community_scope_by_moderator` at line 6702). Use a 3-line `old_string` window ending at that doc-comment so the anchor is unique — do NOT anchor on a bare `}` or `Ok(())` (76 collisions). If your chosen anchor returns > 1, widen it until it's 1 before editing.

3. **Reuse helpers, invent nothing.** Use `admin_config_fixtures::bootstrap()` → `(ContainerAsync, Data<LemmyContext>, String /*db_url*/)`, `bootstrap_instance(&context)` → `Instance`, `seed_user(&context, instance.id, "<name>", is_admin)` → `(PersonId, LocalUserView)`. There is NO auth-cookie/login idiom — auth is the `LocalUserView` from `seed_user` (the `is_admin: bool` arg). Do not add new helpers.

4. **Handler call shape (Task 5).** `admin_set_messaging_config(Json(data): Json<AdminSetMessagingConfig>, local_user_view: LocalUserView, context: Data<LemmyContext>) -> LemmyResult<Json<AdminSetMessagingConfigResponse>>`. DTO: `AdminSetMessagingConfig { scope: String, key: String, value: serde_json::Value }`. Imports: `lemmy_api::governance::messaging_config::admin_set_messaging_config`, `lemmy_api_common::governance::AdminSetMessagingConfig`, `actix_web::web::Json`. Place per-fn `use` blocks inside each test fn body (the `admin_set_config_*` sibling does this — mirror it).

5. **validate-pending-laptop-e2e, write-then-stop.** After committing both tests, write a `kind: "validate-pending-laptop-e2e"` DQ entry with:
   ```
   commands: ["./scripts/brehon/cargo-test.sh --test e2e -p lemmy_server messaging_disabled_preserves_governance_posture messaging_identity_policy_rejects_jury_override"]
   ```
   (the two new fns by name; the laptop also runs the governance subset). Set `branch` = your worker branch, `phase_task: 7`, `e2e_filter: null`. Commit + push the DQ, then **STOP**. Do NOT run cargo yourself — validation is delegated to the laptop advisor. Per `feedback_validate_pending_laptop_write_then_stop.md`.

6. **DQ id via helper.** Generate the DQ id with `bash scripts/brehon/dq-v3-new-entry.sh` and append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`. Never hand-compute `max(all_ids)+1`.

7. **Commit message.** `feat(e2e): add messaging clean-posture + identity-policy rejection tests (task 7)`. End the commit body with a `LESSON:` trailer if you hit any non-obvious e2e edit footgun, OR a `kind: "log"` DQ entry — never both.
