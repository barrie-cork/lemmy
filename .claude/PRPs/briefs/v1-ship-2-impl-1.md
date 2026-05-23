# Brief: v1-ship-2 impl Task 1 — scaffold mod v1_ship_2_fixtures + Test 1

## 1. Role + dispatch line

`[role:impl-task] v1-ship-2 Task 1 scaffold mod + request_appeal test — see .claude/PRPs/briefs/v1-ship-2-impl-1.md`

## 2. Scope

### 2.1 §G4 canonical recipe (Case A discipline — mandatory verbatim citation)

> | Failure signature | Auto-fix | Source lesson |
> | Any test fn in e2e.rs | All test fn signatures MUST be `async fn <name>() -> LemmyResult<()>`. All helper fn signatures MUST be `async fn <name>(...) -> LemmyResult<T>`. Bare `?` propagation throughout. NO `Box<dyn Error>`. NO `.map_err(|e| format!("{e}").into())` closures. | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.2 What to produce

Append a new `mod v1_ship_2_fixtures { ... }` module at the **end** of `crates/server/tests/e2e.rs` (after the closing `}` of `mod v1_federation_inbound_a_fixtures` at line 15449). The module contains:

1. The canonical-sibling-shape scaffolding (imports + `mint_jwt` helper) from plan §10.1 verbatim.
2. Test fn `request_appeal_happy_path_and_auth_failure` covering:
   - Happy path: `POST /api/v4/governance/appeal` returns 200 with `RequestAppealResponse` shape.
   - Failure mode: same POST without `Authorization` header returns 401.

**Commit subject:** `feat(e2e): v1-ship-2 Task 1 — scaffold mod + request_appeal test (task 1)`

### 2.3 IMPLEMENT steps

**File: `crates/server/tests/e2e.rs`**

**Edit discipline (MANDATORY per `feedback_junior_worker_e2e_edit_hang.md` spirit):**
- `old_string` targets the closing `}` of `mod v1_federation_inbound_a_fixtures` + the blank line following it (≤5 lines total). NEVER target the middle of an existing module.
- `new_string` re-emits those same lines, then appends the new `mod v1_ship_2_fixtures { ... }` block.

**Step A — Module scaffolding (verbatim from plan §10.1):**

```rust
mod v1_ship_2_fixtures {
  use super::*;
  use actix_web::{App, test, web::Data};
  use chrono::{Duration, Utc};
  use diesel::ExpressionMethods;
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api_common::governance::{
    CreateEndorsementResponse, GetMyReputationResponse, ListGovernanceModlog,
    RequestAppeal, RequestAppealResponse,
  };
  use lemmy_api_utils::{claims::Claims, context::LemmyContext};
  use lemmy_db_schema::{
    newtypes::LocalUserId,
    source::{
      instance::Instance,
      governance::{
        moderation_case::ModerationCaseInsertForm,
        public_case_log::PublicCaseLogInsertForm,
      },
    },
  };
  use lemmy_db_schema_file::{
    PersonId,
    enums::{CaseSeverity, CaseStatus, CaseTargetType, JuryDecision},
    schema::moderation_case,
  };
  use lemmy_db_views_governance_modlog::GovernanceModlogView;
  use lemmy_routes::middleware::session::SessionMiddleware;
  use lemmy_utils::{error::LemmyResult, rate_limit::RateLimit};

  async fn mint_jwt(ctx: &LemmyContext, local_user_id: LocalUserId) -> LemmyResult<String> {
    let req = test::TestRequest::default().to_http_request();
    let token = Claims::generate(local_user_id, None, req, ctx).await?;
    Ok(token.into_inner())
  }

  // (test fns appended here, one per task)
}
```

**Step B — Test fn `request_appeal_happy_path_and_auth_failure` inside the module:**

Signature: `async fn request_appeal_happy_path_and_auth_failure() -> LemmyResult<()>`
Attribute: `#[tokio::test(flavor = "multi_thread")]`

Steps (verbatim from plan §13 Task 1):

1. `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;`
2. `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
3. `let (target_pid, target_lu_view) = governance_fixtures::seed_user(&context, instance.id, "ship2_appeal_target", false).await?;`
4. Seed Decided case + UPDATE appeal_window_expires_at + panel_size_snapshot — copy plan §10.3 verbatim, substituting `target_pid` for `target_person_id`. Capture the inserted case_id via `.returning(moderation_case::id).get_result(...)` (do NOT hardcode `case_id:1`).
5. `let target_jwt = mint_jwt(&context, target_lu_view.local_user.id).await?;`
6. Build actix App per plan §10.2. Include the rate-limit bucket override block from `e2e.rs:4110-4122` (mirror that pattern exactly).
7. (happy path) POST `/api/v4/governance/appeal` with `Authorization: Bearer {target_jwt}` + `Content-Type: application/json` + body `{"case_id":<inserted_case_id>,"reason":"v1_ship_2 appeal probe"}`. Assert status == 200. `let body: RequestAppealResponse = test::read_body_json(resp).await;`. Assert `body.appeal_id.0 > 0` AND `body.case_id` equals the inserted ModerationCaseId.
8. (auth failure) POST same URI + body WITHOUT Authorization header. Assert status == 401.
9. `Ok(())`

### 2.4 Validate gate (Linux wrapper — EliteDesk runs Linux)

```bash
./scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-ship-2-task1-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task1-check.log
# EXPECT: exit 0

./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task1-clippy.log 2>&1
echo "exit: $?"
# EXPECT: exit 0

./scripts/brehon/cargo-test.sh --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-2-task1-test-no-run.log 2>&1
echo "exit: $?"
# EXPECT: exit 0

./scripts/brehon/cargo-test.sh --workspace --test e2e --features full request_appeal_happy_path_and_auth_failure > .claude/PRPs/debug/v1-ship-2-task1-e2e.log 2>&1
echo "exit: $?"
tail -40 .claude/PRPs/debug/v1-ship-2-task1-e2e.log
# EXPECT: exit 0; "1 passed; 0 failed" in tail
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
    "./scripts/brehon/cargo-test.sh --workspace --test e2e --features full request_appeal_happy_path_and_auth_failure"
  ],
  "branch": "phase-v1-ship-2",
  "phase_task": 1,
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```

### 2.5 Explicit boundaries

- **DO NOT** touch any existing test fn or module.
- **DO NOT** add new handler files, DTO files, or migrations.
- **DO NOT** insert in the middle of `mod v1_federation_inbound_a_fixtures`.
- **COMMIT ONLY** `crates/server/tests/e2e.rs`.
- **DO NOT** write `answered_by: "advisor"` in any DQ entry.

## 3. Required reading (read before first Edit)

**In order:**

1. `crates/server/tests/e2e.rs:15388-15449` — `mod v1_federation_inbound_a_fixtures` (canonical Case A sibling — full module body; read verbatim before writing a single line)
2. `crates/server/tests/e2e.rs:4131-4137` — actix App composition pattern (§10.2 mirror)
3. `crates/server/tests/e2e.rs:4110-4122` — rate-limit bucket override block
4. `crates/server/tests/e2e.rs:4261-4324` — Decided-case + UPDATE seeding pattern (§10.3 mirror; copy verbatim)
5. `crates/server/tests/e2e.rs:793-852` — `governance_fixtures::bootstrap` + `seed_user` signatures
6. `crates/api/api_common/src/governance.rs:230-305` — `RequestAppeal`, `RequestAppealResponse` DTOs (verify field names before writing assertion)
7. `crates/api/api_crud/src/governance/request_appeal.rs:49-198` — appeal handler body (understand the failure mode path)
8. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (mandatory per file-class table in advisor-orchestrator.md §2.4)
9. `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection::establish` + `DbPool::Conn` pattern
10. `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate
11. `.claude/PRPs/plans/v1-ship-2.plan.md` §10 (Patterns to mirror) — canonical plan-time MIRROR refs

## 4. Constraints

1. **Case A is non-negotiable.** All `async fn` in this module return `LemmyResult<()>` (test fns) or `LemmyResult<T>` (helpers). Zero `Box<dyn Error>`. Zero `.map_err` bridges. If a Diesel call returns a non-LemmyError, wrap with `.map_err(|e| LemmyErrorType::Unknown(e.to_string()).into())?`.
2. **Append-only Edit.** `old_string` is ≤5 lines targeting the closing `}` of `mod v1_federation_inbound_a_fixtures`. Never a large block.
3. **DQ mid-task push.** Any `kind: "blocker"` or `kind: "validate-pending-laptop"` entry must be committed + pushed immediately per `.claude/rules/decision-queue.md` "Mid-task visibility". Use `bash scripts/brehon/dq-v3-append-fragment.sh` to write the DQ entry.
4. **Linux invocation.** Use `./scripts/brehon/cargo-*.sh` (not `.bat`).
5. **Do not hardcode case_id.** Always capture the returned id from the INSERT via `.returning(moderation_case::id).get_result(&mut async_conn).await?`.
6. **One commit.** Commit subject: `feat(e2e): v1-ship-2 Task 1 — scaffold mod + request_appeal test (task 1)`. Include `LESSON:` trailer if any surprising finding encountered.
