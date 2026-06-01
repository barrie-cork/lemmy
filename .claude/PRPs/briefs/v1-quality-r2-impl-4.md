# Brief: v1-quality-r2 impl-task 4 — EnvVarGuard hoist + boot_context thread-guards refactor

## 1. Role + dispatch line

`[role:impl-task]` v1-quality-r2 task 4 — EnvVarGuard hoist + boot_context refactor (closes #159) — see `.claude/PRPs/briefs/v1-quality-r2-impl-4.md`

## 2. Scope

**Goal:** implement Task 4 (C4-A) from `.claude/PRPs/plans/v1-quality-r2.plan.md`.

- **Hoist** `struct EnvVarGuard` + `impl EnvVarGuard` + `impl Drop for EnvVarGuard` (~32 lines) from inside `mod v1_rt_r3_fixtures` to the test-crate root (between `}` at line 109 and `// ===` comment at line 111, i.e. before `mod governance_fixtures {`).
- **boot_context refactor:** change return type to add `Vec<EnvVarGuard>` as fifth tuple element; rewrite 3 `unsafe { std::env::set_var(...) }` blocks as `guards.push(EnvVarGuard::set(...))`.
- **10 callsite updates:** `let (_container, context, _federation_context, db_url) = boot_context().await?;` → `let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;` (and `federation_config` variants — see §2.2).
- **`use super::EnvVarGuard;` add** inside `mod v1_rt_r3_fixtures` (after `use super::*;`) so existing `EnvVarGuard::set(...)` usages in that module still compile after the struct is hoisted out.
- **Delete** the EnvVarGuard struct block from inside `mod v1_rt_r3_fixtures` (the hoist-delete Edit).

**Out of scope:**
- Do NOT touch T5's 13 `LEMMY_DATABASE_URL` setter sites.
- Do NOT edit any module other than the hoist/delete cluster in `mod v1_rt_r3_fixtures` and the callsite update lines enumerated in §2.2.
- Do NOT change test logic, assertions, or any other code.
- Do NOT add `pub` keyword to `EnvVarGuard` (not required for test-crate-root visibility within the same file).

**Commit subject:** `refactor(e2e): hoist EnvVarGuard + thread guards through boot_context (closes #159, task 4)`

**Single file:** `crates/server/tests/e2e.rs` only.

## 2.1 Pre-locate anchor check (R11)

Per `feedback_fix_impl_pre_locate_e2e_anchors.md`: before any Edit, locate each `old_string` verbatim in the current file. If ANY anchor does not match the live file byte-for-byte, STOP — raise `kind: "blocker"` DQ. Do NOT fuzzy-match.

All anchors below were pre-located against `origin/phase-v1-quality-r2` tip (commit `993f2ee8f`). T3 doc-comment edits shifted line numbers from the plan's original references.

## 2.2 Verbatim edit pairs (apply in this order)

### Edit A — Hoist INSERT (add EnvVarGuard at test-crate root)

Insert the struct block between the `}` closing the last top-level test fn and the `// ===` section comment that precedes `mod governance_fixtures`.

**old_string** (the blank line + comment block that precedes `mod governance_fixtures {`):
```
// ============================================================================
// Phase 1 — governance schema + hash-chain trigger smoke tests
// ============================================================================

mod governance_fixtures {
```

**new_string** (insert EnvVarGuard BEFORE the section comment):
```
struct EnvVarGuard {
  key: &'static str,
  prev: Option<String>,
}

impl EnvVarGuard {
  fn set(key: &'static str, value: &str) -> Self {
    let prev = std::env::var(key).ok();
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var(key, value);
    }
    Self { key, prev }
  }
}

impl Drop for EnvVarGuard {
  fn drop(&mut self) {
    // SAFETY: same justification — single-threaded test runner.
    unsafe {
      match &self.prev {
        Some(prev) => std::env::set_var(self.key, prev),
        None => std::env::remove_var(self.key),
      }
    }
  }
}

// ============================================================================
// Phase 1 — governance schema + hash-chain trigger smoke tests
// ============================================================================

mod governance_fixtures {
```

---

### Edit B — Hoist DELETE (remove EnvVarGuard from inside mod v1_rt_r3_fixtures)

**old_string** (the struct + two impls inside the module):
```
  struct EnvVarGuard {
    key: &'static str,
    prev: Option<String>,
  }

  impl EnvVarGuard {
    fn set(key: &'static str, value: &str) -> Self {
      let prev = std::env::var(key).ok();
      // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
      unsafe {
        std::env::set_var(key, value);
      }
      Self { key, prev }
    }
  }

  impl Drop for EnvVarGuard {
    fn drop(&mut self) {
      // SAFETY: same justification — single-threaded test runner.
      unsafe {
        match &self.prev {
          Some(prev) => std::env::set_var(self.key, prev),
          None => std::env::remove_var(self.key),
        }
      }
    }
  }

  /// Seed one person/local_user pair. Mirror governance_fixtures::seed_user
```

**new_string** (struct removed; seed_person doc-comment is the new first content after the use block):
```
  /// Seed one person/local_user pair. Mirror governance_fixtures::seed_user
```

---

### Edit C — Add `use super::EnvVarGuard;` inside mod v1_rt_r3_fixtures

After hoist, `EnvVarGuard` lives at test-crate root. The module must import it explicitly.

**old_string**:
```
  use super::*;
  use actix_web::web::{Data, Json};
```

**new_string**:
```
  use super::*;
  use super::EnvVarGuard;
  use actix_web::web::{Data, Json};
```

---

### Edit D — boot_context return type signature

**old_string**:
```
  async fn boot_context() -> LemmyResult<(
    testcontainers::ContainerAsync<testcontainers::GenericImage>,
    Data<LemmyContext>,
    activitypub_federation::config::FederationConfig<LemmyContext>,
    String,
  )> {
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    // SAFETY: tests run with --test-threads=1; no concurrent env mutation.
    unsafe {
      std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
      std::env::set_var("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX);
    }
    let (container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    // SAFETY: tests run with --test-threads=1.
    unsafe {
      std::env::set_var("LEMMY_DATABASE_URL", &db_url);
    }
```

**new_string**:
```
  async fn boot_context() -> LemmyResult<(
    testcontainers::ContainerAsync<testcontainers::GenericImage>,
    Data<LemmyContext>,
    activitypub_federation::config::FederationConfig<LemmyContext>,
    String,
    Vec<EnvVarGuard>,
  )> {
    const SIGNING_SEED_HEX: &str =
      "0000000000000000000000000000000000000000000000000000000000000001";
    let mut guards: Vec<EnvVarGuard> = Vec::with_capacity(3);
    guards.push(EnvVarGuard::set("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1"));
    guards.push(EnvVarGuard::set("GOVERNANCE_LOG_SIGNING_KEY", SIGNING_SEED_HEX));
    let (container, host_port) = governance_fixtures::start_postgres().await?;
    let db_url = governance_fixtures::db_url(host_port);
    guards.push(EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url));
```

---

### Edit E — boot_context return value (add guards to Ok tuple)

**old_string**:
```
    Ok((container, context, federation_config, db_url))
  }
```

**new_string**:
```
    Ok((container, context, federation_config, db_url, guards))
  }
```

---

### Edits F1–F10 — 10 callsite updates

Each destructure gains `_env_guards` as the fifth element. Apply in file order.

**F1** (line ~17654 post-T3 shift):

old_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

**F2** (line ~17709, second occurrence of same pattern):

old_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

> **NOTE for F1/F2/F3/F4:** these four callsites share the identical surrounding 3-line pattern. Use surrounding function-name anchors to uniquely locate each one. The functions are (in file order):
> - F1: `async fn activity_cron_emits_participation_consistency_for_active_member`
> - F2: `async fn activity_cron_idempotent_within_same_window`
> - F3: `async fn dormancy_cron_emits_minus_one_for_dormant_member`
> - F4: `async fn dormancy_cron_idempotent_within_same_window`

**F3** (line ~17749):

old_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

**F4** (line ~17793):

old_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
    let _guard = EnvVarGuard::set("BREHON_DISABLE_PARTICIPATION_JOB", "1");
    let (_container, context, _federation_context, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

> **ANCHOR STRATEGY for F1-F4:** the 3-line pattern repeats 4×. Use a wider anchor that includes the preceding `#[tokio::test]` and function name line for each. If the Edit tool sees a non-unique `old_string`, expand to include the test function name 2-3 lines above as additional context.

**F5** (line ~17834 — `federation_config` variant):

old_string:
```
    let (_container, context, federation_config, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
    let (_container, context, federation_config, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

> **NOTE for F5/F6/F7/F8/F9/F10:** these use `federation_config` (not `_federation_context`). Same uniqueness issue — expand anchor with function name if needed.
> - F5: `async fn vote_outcome_emits_plus_one_for_majority_aligned_jurors`
> - F6: same pattern, next test (minority jurors)
> - F7: `async fn flag_bad_faith_returns_403_for_non_admin`
> - F8: `async fn flag_bad_faith_returns_400_for_non_emergency_remove_status`
> - F9: `async fn flag_bad_faith_admin_on_emergency_remove_case_emits_minus_one`
> - F10: the evidence-cited heuristic test (last in module)

**F6** (line ~17898):

old_string:
```
    // Distinct test focus: minority jurors get zero rows under VoteOutcome.
    let (_container, context, federation_config, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
    // Distinct test focus: minority jurors get zero rows under VoteOutcome.
    let (_container, context, federation_config, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

**F7** (line ~17942):

old_string:
```
  async fn flag_bad_faith_returns_403_for_non_admin() -> LemmyResult<()> {
    let (_container, context, federation_config, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
  async fn flag_bad_faith_returns_403_for_non_admin() -> LemmyResult<()> {
    let (_container, context, federation_config, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

**F8** (line ~17968):

old_string:
```
  async fn flag_bad_faith_returns_400_for_non_emergency_remove_status() -> LemmyResult<()> {
    let (_container, context, federation_config, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
  async fn flag_bad_faith_returns_400_for_non_emergency_remove_status() -> LemmyResult<()> {
    let (_container, context, federation_config, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

**F9** (line ~18001):

old_string:
```
  async fn flag_bad_faith_admin_on_emergency_remove_case_emits_minus_one() -> LemmyResult<()> {
    let (_container, context, federation_config, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
  async fn flag_bad_faith_admin_on_emergency_remove_case_emits_minus_one() -> LemmyResult<()> {
    let (_container, context, federation_config, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

**F10** (line ~18059):

old_string:
```
  ) -> LemmyResult<()> {
    let (_container, context, federation_config, db_url) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```
new_string:
```
  ) -> LemmyResult<()> {
    let (_container, context, federation_config, db_url, _env_guards) = boot_context().await?;
    let mut async_conn = AsyncPgConnection::establish(&db_url).await?;
```

> **NOTE for F10:** the `old_string` starts with `) -> LemmyResult<()> {` which may not be unique. Expand to include the preceding function signature line if needed. The function is the last test in `mod v1_rt_r3_fixtures` — check that the anchor is unique before applying.

## 3. Required reading

- `.claude/PRPs/plans/v1-quality-r2.plan.md` §10.1 (EnvVarGuard struct verbatim), §10.3 (boot_context return-type extension target shape)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — e2e.rs error shape (LemmyResult<()> throughout)
- `.claude/lessons/feedback_async_pool_test_pattern.md` — async pool pattern in e2e.rs
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate anchors BEFORE any Edit; stop on mismatch

**Mandatory MIRROR refs** (read before editing):
- `crates/server/tests/e2e.rs:17172-17196` (current `mod v1_rt_r3_fixtures {` opening with `use super::*;`) — verify anchor bytes match §2.2 Edit C before applying.
- `crates/server/tests/e2e.rs:17491-17538` (current `boot_context()` fn) — verify anchor bytes match §2.2 Edits D+E before applying.

## 4. Constraints

1. **Pre-push cargo check:** run `bash scripts/brehon/cargo-check.sh --workspace --features full` before pushing. Non-zero exit → fix in same commit (if in-scope) or raise `kind: "blocker"` DQ (if out-of-scope). Never `#[allow]`-spam.
2. **Single file:** only `crates/server/tests/e2e.rs` is in scope. Any other file → stop and raise `kind: "blocker"` DQ.
3. **`_env_guards` not `_`:** the binding name `_env_guards` (underscore-prefixed) silences the unused-binding warning but does NOT cause early drop. Using bare `_` would drop guards immediately (load-bearing difference).
4. **R11 anchor discipline:** if ANY `old_string` fails to match verbatim, STOP — do not fuzzy-match — raise `kind: "blocker"` DQ.
5. **DQ mid-task push:** if you raise a `kind: "blocker"` DQ entry, commit + push `decision-queue.json` immediately on the worker branch so the advisor can see it.
6. **validate-pending-laptop DQ:** after successful push, write a `kind: "validate-pending-laptop"` DQ entry with commands:
   - `bash scripts/brehon/cargo-check.sh --workspace --features full`
   - `bash scripts/brehon/cargo-clippy.sh --workspace --features full --no-deps -- -D warnings`
   - `bash scripts/brehon/cargo-test.sh --test e2e --no-run --workspace --features full`
   - Python audit: verify EnvVarGuard is at crate root + boot_context returns 5-tuple + `use super::EnvVarGuard;` in mod v1_rt_r3_fixtures.

**Mandatory file-class lessons fired:** `feedback_lemmy_error_no_std_error.md` (e2e.rs edit), `feedback_async_pool_test_pattern.md` (e2e.rs edit), `feedback_fix_impl_pre_locate_e2e_anchors.md` (≥2 edits to e2e.rs).
