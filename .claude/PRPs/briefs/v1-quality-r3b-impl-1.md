---
role: impl-task
task: 1
phase: v1-quality-r3b
plan: .claude/PRPs/plans/v1-quality-r3b.plan.md
created: 2026-05-31
authored_by: advisor (canonical brehon-fork / governance-v0 session, Mode B)
base_branch: phase-v1-quality-r3b
---

# impl-task brief — v1-quality-r3b Task 1

**Role:** `[role:impl-task]`  
**Task:** T1 — Capture DB URL at LemmyContext::create; switch handler; drop test band-aids  
**Phase:** v1-quality-r3b  
**Base branch:** `phase-v1-quality-r3b`  
**Commit subject:** `fix(governance): capture DB URL at LemmyContext::create (issue #167)`

---

## 1. Role + dispatch line

```
[role:impl-task] v1-quality-r3b T1 — LemmyContext DB URL capture + admin_audit_stream fix — see .claude/PRPs/briefs/v1-quality-r3b-impl-1.md
```

---

## 2. Scope

**Produce:** one commit on `phase-v1-quality-r3b` that atomically:

1. Adds `db_url: String` field to `LemmyContext` struct (after `rate_limit_cell`)
2. Captures `SETTINGS.get_database_url()` inside `create()` before the struct literal
3. Adds `pub fn database_url(&self) -> &str { &self.db_url }` accessor
4. Switches `admin_audit_stream.rs:125` from `context.settings().get_database_url()` to `context.database_url()`; drops the `&` at line 126
5. Removes the 3 `_g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", …)` band-aid lines from the 3 `admin_audit_stream_*` test bodies; renames `db_url` → `_db_url` at the 3 destructure lines

**Files to modify (exactly these 3, no others):**
- `crates/api/api_utils/src/context.rs`
- `crates/api/api/src/governance/admin_audit_stream.rs`
- `crates/server/tests/e2e.rs`

**Do NOT:**
- Add a new parameter to `LemmyContext::create()` — capture happens internally (Option A)
- Remove or change the fixture-internal guard at `e2e.rs:6128` (it makes capture-at-create see the testcontainer URL — it MUST stay)
- Touch any file outside the 3-file list
- Add a `SETTINGS` import — it is already imported at `context.rs:5-8`
- Create migrations, new endpoints, or new test fixtures

---

## 3. Required reading (load-bearing — read these first)

### 3a. MIRROR refs (read these before writing any code)

1. **`crates/api/api_utils/src/context.rs:12-56`** — read the full struct definition (`:12-21`), `create()` body (`:24-38`), and the `secret()` accessor (`:54-56`). The field, capture, and new accessor mirror this exact pattern.

2. **`crates/api/api/src/governance/admin_audit_stream.rs:120-132`** — read lines 120-132 to see the exact `get_database_url()` call at :125 and the `tokio_postgres::connect(&db_url, NoTls)` call at :126. The `&` at 126 is the GOTCHA — it must become `db_url` (no `&`) after the change.

3. **`crates/server/tests/e2e.rs:6107-6135`** — the `admin_config_fixtures::bootstrap()` fixture. The guard at :6128 MUST NOT be removed. Read this to understand the fix at a glance.

### 3b. Lessons (mandatory per file-class table)

4. **`.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md`** — verbatim `old_string`/`new_string` anchors are pre-located in §5 below; use them exactly. Do NOT attempt to locate anchors by Read/Grep at task time.

5. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — no new `Box<dyn Error>`; all new code uses `LemmyResult<()>` with `?`. This fix adds zero error-conversion code — the handler's existing error shape is unchanged.

6. **`.claude/lessons/feedback_rust_visibility_cross_crate.md`** — the `database_url()` accessor MUST be `pub` (called from `lemmy_api` which is a different crate from `lemmy_api_utils`).

---

## 4. Constraints

- **Option A (zero callsite changes):** `create()` reads `SETTINGS.get_database_url()` INTERNALLY. The 16 `LemmyContext::create(…)` call sites (context.rs:79, lib.rs:210, 14 in e2e.rs) need NO change — they pass no `db_url` argument. Only the `create()` body and struct literal change.
- **SETTINGS already imported** at `context.rs:5-8` — do NOT add an import.
- **`create()` is not async** — `get_database_url()` is infallible (`-> String`, no `?`). The body stays synchronous.
- **Accessor visibility:** `pub fn database_url(&self) -> &str` (NOT `pub(crate)`).
- **e2e fixture guard at :6128 is SACRED** — do NOT remove it.
- **DQ mid-task discipline:** if a blocker arises, write a `kind: "blocker"` DQ entry via `bash scripts/brehon/dq-v3-new-entry.sh`, commit + push, stop.
- **VALIDATE after implementing** per §6 before committing. Run cargo-check.bat first (fast), then cargo-clippy.bat, then confirm the grep gate, then write the commit. Gate-4 full e2e runs via the `validate-pending-laptop` handler AFTER commit — do NOT run it locally during task (it takes ~26 min).

---

## 5. Pre-located verbatim edit anchors

**CRITICAL:** Use these exact `old_string` / `new_string` pairs. Do NOT Read/Grep e2e.rs to find these anchors — the pre-location IS the brief.

### Edit A — context.rs: add `db_url` field to struct

**File:** `crates/api/api_utils/src/context.rs`

**old_string** (the closing of rate_limit_cell field):
```
  rate_limit_cell: RateLimit,
}
```
*(This is the final field + struct closing brace. If there are other fields after `rate_limit_cell`, read context.rs:12-21 first to confirm the exact closing — the MIRROR ref at §3a.1 is authoritative.)*

**new_string:**
```
  rate_limit_cell: RateLimit,
  /// Database URL captured at construction time from `SETTINGS.get_database_url()`.
  /// Read this (via `database_url()`) instead of re-reading `LEMMY_DATABASE_URL`
  /// from the live process environment — the env var may be unset by the time a
  /// handler runs (the test EnvVarGuard drops at fixture return). See issue #167.
  db_url: String,
}
```

### Edit B — context.rs: add capture in `create()` + `db_url` in struct literal

**File:** `crates/api/api_utils/src/context.rs`

Read lines `:24-38` (MIRROR ref §3a.1) to locate the `create()` body. Add `let db_url = SETTINGS.get_database_url();` before the `LemmyContext {` struct literal, and add `db_url,` to the struct literal after `rate_limit_cell`.

*(This edit's exact old_string depends on the current create() body — read :24-38 first, then apply the change. The struct literal must include `db_url` as a new field.)*

### Edit C — context.rs: add `database_url()` accessor

**File:** `crates/api/api_utils/src/context.rs`

Add after the existing `secret(&self)` accessor (MIRROR ref §3a.1 `:54-56`):

```rust
  pub fn database_url(&self) -> &str {
    &self.db_url
  }
```

### Edit D — admin_audit_stream.rs: switch to captured URL

**File:** `crates/api/api/src/governance/admin_audit_stream.rs`

**old_string** (read :120-132 first to confirm exact text, MIRROR §3a.2):
```
    let db_url = context.settings().get_database_url();
    let (pg_client, pg_conn) = match tokio_postgres::connect(&db_url, NoTls).await {
```

**new_string:**
```
    let db_url = context.database_url();
    let (pg_client, pg_conn) = match tokio_postgres::connect(db_url, NoTls).await {
```

**GOTCHA:** `database_url()` returns `&str`. The old `db_url` was `String` so `&db_url` coerced to `&str`. The new `db_url` is already `&str`, so `&db_url` would be `&&str` — which does NOT implement `TryInto<Config>` for `tokio_postgres::connect`. Drop the `&`. Forgetting this → `E0277`.

### Edit E — e2e.rs band-aid #1: `admin_audit_stream_forbidden_for_non_admin`

**File:** `crates/server/tests/e2e.rs`

**old_string** (3 lines — function signature makes it unique):
```
async fn admin_audit_stream_forbidden_for_non_admin() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

**new_string:**
```
async fn admin_audit_stream_forbidden_for_non_admin() -> lemmy_utils::error::LemmyResult<()> {
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;
  use lemmy_utils::error::LemmyErrorType;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
```

### Edit F — e2e.rs band-aid #2: `admin_audit_stream_enforces_per_admin_cap`

**File:** `crates/server/tests/e2e.rs`

**old_string:**
```
async fn admin_audit_stream_enforces_per_admin_cap() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::http::StatusCode;
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

**new_string:**
```
async fn admin_audit_stream_enforces_per_admin_cap() -> lemmy_utils::error::LemmyResult<()> {
  use actix_web::http::StatusCode;
  use lemmy_api::governance::admin_audit_stream::admin_audit_stream;

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
```

### Edit G — e2e.rs band-aid #3: `admin_audit_stream_emits_frame_on_config_change`

**File:** `crates/server/tests/e2e.rs`

**old_string** (use line from the imports block just before bootstrap — makes this site unique):
```
  use std::{future::poll_fn, pin::Pin, time::Duration as StdDuration};

  let (_container, context, db_url) = admin_config_fixtures::bootstrap().await?;
  let _g_db_url = EnvVarGuard::set("LEMMY_DATABASE_URL", &db_url);
```

**new_string:**
```
  use std::{future::poll_fn, pin::Pin, time::Duration as StdDuration};

  let (_container, context, _db_url) = admin_config_fixtures::bootstrap().await?;
```

---

## 6. Validation gate (run before committing)

```bash
# Check compiles — fast gate first
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-quality-r3b-task1-check.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0

# Lint — catch unused_variable (db_url→_db_url) and E0277 (&& str)
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-quality-r3b-task1-clippy.log 2>&1"
echo "exit: $?"
# EXPECT: exit 0

# Grep gate — get_database_url gone from handler
grep -c "get_database_url" crates/api/api/src/governance/admin_audit_stream.rs
# EXPECT: 0

# Verify EnvVarGuard count dropped from 17 to 14
grep -c 'EnvVarGuard::set("LEMMY_DATABASE_URL"' crates/server/tests/e2e.rs
# EXPECT: 14
```

Gate-4 full e2e (`cargo test --workspace --test e2e --features full`) runs via `validate-pending-laptop` AFTER the commit — do NOT run it locally during the task (takes ~26 min). The impl-task writes the `validate-pending-laptop` DQ entry after pushing the commit; the advisor laptop session runs the e2e.

---

## 7. Commit

**Subject:** `fix(governance): capture DB URL at LemmyContext::create (issue #167)`

**Body:** close Issue #167; removes the per-test `EnvVarGuard` band-aids introduced by v1-quality-r2 fix-impl-1; the fixture-internal guard at e2e.rs:6128 is preserved (it makes capture-at-create see the testcontainer URL).

After committing, write a `kind: "validate-pending-laptop"` DQ entry (use `bash scripts/brehon/dq-v3-new-entry.sh` for the id) with:
- `commands`: `["cmd //c \"scripts\\\\brehon\\\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-quality-r3b-task1-e2e.log 2>&1\""]`
- `branch`: `phase-v1-quality-r3b`
- `phase_task`: 1

Then commit the DQ entry and push.

---

## 8. Mandatory lesson injections fired (brief-author record)

| File pattern | Lesson | Fired because |
|---|---|---|
| `crates/server/tests/e2e.rs` (≥2 edits) | `feedback_fix_impl_pre_locate_e2e_anchors.md` | 6 edits in e2e.rs (3 delete-guard + 3 rename-binding) |
| `crates/server/tests/e2e.rs` | `feedback_lemmy_error_no_std_error.md` | Any edit to e2e.rs |
| `crates/api/api_utils/src/context.rs` (accessor) | `feedback_rust_visibility_cross_crate.md` | Cross-crate accessor (`pub` required) |
| Any new accessor returning `LemmyResult` | `feedback_lemmy_error_no_std_error.md` | Belt-and-suspenders; confirmed no new error shape |
