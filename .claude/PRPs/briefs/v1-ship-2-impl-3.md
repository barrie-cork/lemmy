# Brief: v1-ship-2 impl Task 3 — Test 3 get_my_reputation_happy_path_and_no_auth

## 1. Role + dispatch line

`[role:impl-task] v1-ship-2 Task 3 reputation test — see .claude/PRPs/briefs/v1-ship-2-impl-3.md`

## 2. Scope

### 2.1 §G4 canonical recipe (Case A discipline — mandatory verbatim citation)

> | Failure signature | Auto-fix | Source lesson |
> | Any test fn in e2e.rs | All test fn signatures MUST be `async fn <name>() -> LemmyResult<()>`. All helper fn signatures MUST be `async fn <name>(...) -> LemmyResult<T>`. Bare `?` propagation throughout. NO `Box<dyn Error>`. NO `.map_err(|e| format!("{e}").into())` closures. | `feedback_lemmy_error_no_std_error.md` Case A |

### 2.2 What to produce

Append a new test fn `get_my_reputation_happy_path_and_no_auth` **inside** `mod v1_ship_2_fixtures` in `crates/server/tests/e2e.rs`. Tasks 1 and 2 already landed the module scaffold and the first two test fns. This task only appends one test fn.

**Edit discipline (MANDATORY per `feedback_junior_worker_e2e_edit_hang.md`):**
- `old_string` targets ≤5 lines: the closing `}` of Task 2's test fn + blank line + module's closing `}`. NEVER target the middle of any existing test fn.
- `new_string` re-emits Task 2's test fn closing `}` + blank line + new test fn body + blank line + module's closing `}`.

**Commit subject:** `feat(e2e): v1-ship-2 Task 3 — reputation test (task 3)`

### 2.3 IMPLEMENT steps

**File: `crates/server/tests/e2e.rs`**

**Step A — Read before writing (MANDATORY):**

Before any Edit, scan the end of `mod v1_ship_2_fixtures` (last ~30 lines) to identify the exact `old_string` target (Task 2's test fn closing `}` + blank line + module `}`). Note that Task 2's worker named its test fn `list_governance_modlog_returns_seeded_entry` — read the actual current content to find the right closing lines.

**Step B — Test fn `get_my_reputation_happy_path_and_no_auth` inside the module:**

Signature: `async fn get_my_reputation_happy_path_and_no_auth() -> LemmyResult<()>`
Attribute: `#[tokio::test(flavor = "multi_thread")]`

Steps (verbatim from plan §13 Task 3):

1. `let (_container, context, _db_url) = governance_fixtures::bootstrap().await?;`
2. `let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;`
3. `let (_user_pid, user_lu_view) = governance_fixtures::seed_user(&context, instance.id, "ship2_rep_user", false).await?;`
4. `let user_jwt = mint_jwt(&context, user_lu_view.local_user.id).await?;`
5. Build actix App per §10.2 (mirror `e2e.rs:4131-4137`). Include rate-limit bucket override block from `e2e.rs:4110-4122`.
6. (happy path, authed): GET `/api/v4/governance/reputation/me` with `Authorization: Bearer {user_jwt}`. Assert status == 200. `let body: GetMyReputationResponse = test::read_body_json(resp).await;`. Assert `body.view.active_sanctions == 0` (fresh user has no active sanctions — mirrors the existing sweep test). Assert the body's `view` field deserialises (Serde will fail-loud on schema drift).
7. (no-auth failure): GET same URI WITHOUT the Authorization header. Assert status == 401. (LocalUserView extractor rejects missing JWT; failure is 401 not 403.)
8. `Ok(())`

**GOTCHA:** `load_or_compute_snapshot` on first call — the handler calls this which writes a `reputation_snapshot` row if none exists. The test does NOT need to seed any reputation events; the compute path writes a default snapshot. `active_sanctions == 0` is robust because `count_active_sanctions` returns 0 when no `sanction` rows exist.

### 2.4 Validate gate (Linux wrapper — EliteDesk runs Linux)

```bash
./scripts/brehon/cargo-check.sh --workspace --features full > .claude/PRPs/debug/v1-ship-2-task3-check.log 2>&1
echo "exit: $?"
tail -20 .claude/PRPs/debug/v1-ship-2-task3-check.log
# EXPECT: exit 0

./scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-ship-2-task3-clippy.log 2>&1
echo "exit: $?"
# EXPECT: exit 0

./scripts/brehon/cargo-test.sh --workspace --features full --test e2e --no-run > .claude/PRPs/debug/v1-ship-2-task3-test-no-run.log 2>&1
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
  "phase_task": 3,
  "result": null,
  "log_slice": null,
  "failed_commands": null
}
```

### 2.5 Explicit boundaries

- **DO NOT** touch Tasks 1 or 2's test fns or any existing test fn or module.
- **DO NOT** add new handler files, DTO files, or migrations.
- **DO NOT** insert in the middle of `mod v1_ship_2_fixtures`.
- **COMMIT ONLY** `crates/server/tests/e2e.rs`.
- **DO NOT** write `answered_by: "advisor"` in any DQ entry.

## 3. Required reading (read before first Edit)

**In order:**

1. `crates/server/tests/e2e.rs` — scan end of `mod v1_ship_2_fixtures` (last ~30 lines) to identify exact `old_string` target (Task 2's test fn closing `}` + module `}`)
2. `crates/api/api/src/governance/get_my_reputation.rs:1-50` — reputation/me handler (confirms `LocalUserView` required extractor → 401 on missing JWT)
3. `crates/api/api_common/src/governance.rs:230-305` — `GetMyReputationResponse` DTO (verify `view` field + `active_sanctions` field name)
4. `crates/server/tests/e2e.rs:4326-4337` — existing GET /reputation/me probe shape (§10.2 mirror)
5. `crates/server/tests/e2e.rs:4131-4137` — actix App composition pattern
6. `crates/server/tests/e2e.rs:4110-4122` — rate-limit bucket override block
7. `crates/server/tests/e2e.rs:793-852` — `governance_fixtures::bootstrap` + `seed_user` signatures
8. `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A discipline (mandatory per file-class table)
9. `.claude/lessons/feedback_async_pool_test_pattern.md` — pool pattern (for reference)
10. `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — append-only Edit discipline (≤5-line `old_string`)
11. `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` — sibling-mirror gate

## 4. Constraints

1. **Case A is non-negotiable.** All `async fn` in this module return `LemmyResult<()>` (test fns) or `LemmyResult<T>` (helpers). Zero `Box<dyn Error>`. Zero `.map_err` bridges.
2. **Append-only Edit.** `old_string` is ≤5 lines targeting Task 2's test fn closing `}` + blank line + module's closing `}`. Never target the middle of an existing test fn.
3. **DQ mid-task push.** Any `kind: "blocker"` or `kind: "validate-pending-laptop"` entry must be committed + pushed immediately. Use `bash scripts/brehon/dq-v3-append-fragment.sh`.
4. **Linux invocation.** Use `./scripts/brehon/cargo-*.sh` (not `.bat`).
5. **401 not 403.** The no-auth failure mode is 401 (JWT missing → extractor rejects). Do not assert 403.
6. **No reputation seeding needed.** The handler's `load_or_compute_snapshot` writes a default on first call. Assert `active_sanctions == 0` directly.
7. **One commit.** Commit subject: `feat(e2e): v1-ship-2 Task 3 — reputation test (task 3)`. Include `LESSON:` trailer if any surprising finding encountered.
