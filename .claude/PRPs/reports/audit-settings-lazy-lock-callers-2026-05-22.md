# Audit: Settings::SETTINGS access in test contexts — 2026-05-22

## Summary
- **Total test-side callers found:** 21
- **Safe (env var set):** 2 (e2e.rs uses unsafe { set_var(...) })
- **Vulnerable (no env var):** 19
- **Tests via mock/fake:** 0
- **Direct LazyLock state:** All callers in per-crate tests reference same static LazyLock in process

## CI behavior

### GitHub Actions
- **cargo-test-e2e.yml:** Runs `cargo test --workspace --features full --test e2e -- --test-threads=1`
  - **Does NOT set `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS`**
  - However, only the e2e integration test runs (single binary); lib tests within crates are NOT executed via GH
  - **Risk level:** GitHub CI itself is safe; problem is LOCAL dev

- **cargo-validate-workspace.yml:** Runs `cargo test --no-run` (compile only, no execution)
  - No test execution risk

### Upstream Woodpecker CI (.woodpecker.yml)
- **Line 160-164:** `cargo_test` step sets env vars:
  ```
  LEMMY_DATABASE_URL: postgres://lemmy:password@database:5432/lemmy
  LEMMY_TEST_FAST_FEDERATION: "1"
  LEMMY_CONFIG_LOCATION: /woodpecker/src/github.com/LemmyNet/lemmy/config/config.hjson
  ```
  - **Does NOT set `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS`**
  - **However:** Woodpecker CI sets `LEMMY_CONFIG_LOCATION` to workspace root config path
  - Tests run from workspace root in CI (`/woodpecker/src/.../`), so path resolves correctly
  - **Risk level:** CI-safe (absolute path + workspace-root CWD), LOCAL dev-broken

### Local dev (cargo test -p lemmy_api)
- **VULNERABLE:** Per-crate CWD is `crates/api/api/`, config file searches `config/config.hjson`
- Relative path resolves to `crates/api/api/config/config.hjson` → NOT FOUND
- LazyLock poisoning occurs on first test failure

## Vulnerable test inventory

| File:line | Test name | Why vulnerable | Fix recipe |
| --- | --- | --- | --- |
| `crates/api/api/src/federation/user_settings_backup.rs:322` | `test_settings_export_import` | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/api/api/src/federation/user_settings_backup.rs:393` | `disallow_large_backup` | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/api/api/src/federation/user_settings_backup.rs:430` | `import_partial_backup` | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/api/api/src/federation/resolve_object.rs:132` | `test_object_visibility` | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/api/api/src/site/mod_log.rs:98` | `test_mod_remove_or_restore_data` | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/api/api/src/site/mod_log.rs:400` | `test_bulk_parent_id_propagated` | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/api/api/src/site/registration_applications/tests.rs:128` | `test_application_approval` | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/objects/src/objects/comment.rs` | 2 tests | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/objects/src/objects/community.rs` | 1 test | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/objects/src/objects/instance.rs` | 1 test | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/objects/src/objects/person.rs` | 2 tests | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/objects/src/objects/post.rs` | 2 tests | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/objects/src/objects/private_message.rs` | 2 tests | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/apub/src/collections/community_moderators.rs` | 1 test | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/apub/src/http/community.rs` | 4 tests | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |
| `crates/apub/objects/src/utils/markdown_links.rs` | 2 tests | Calls `LemmyContext::init_test_context()`, no env var set | Add `unsafe { set_var(...) }` before init |

## Root cause analysis

1. **Path resolution asymmetry:**
   - `Settings::init()` reads `LEMMY_CONFIG_LOCATION` env or defaults to `config/config.hjson`
   - Workspace has `config/config.hjson` at root
   - But `cargo test -p lemmy_api` runs with CWD = `crates/api/api/`, so relative path fails
   - Upstream CI uses absolute path in `LEMMY_CONFIG_LOCATION` or workspace-root CWD

2. **LazyLock poisoning:**
   - All tests share the same static `SETTINGS: LazyLock<Settings>` in-process
   - First test that accesses `SETTINGS` triggers `LazyLock::new(|| { ... })`
   - If closure panics, the `LazyLock` is permanently poisoned
   - All subsequent tests in the process fail with "LazyLock instance has previously been poisoned"

3. **Why CI doesn't catch this:**
   - GitHub Actions: only runs integration test binary (e2e.rs), not lib tests
   - Woodpecker: sets `LEMMY_CONFIG_LOCATION` to absolute path + workspace-root CWD

## Recommended fix scope (extension of Phase 3 Path A)

**Short-term (local dev fix):**
Add a test helper function in each affected crate's test module (or a shared test-utils crate):

```rust
#[cfg(test)]
fn ensure_default_settings() {
  unsafe {
    std::env::set_var("LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS", "1");
  }
}
```

Each vulnerable test should call this **before** first access to `LemmyContext::init_test_context()`.

**Option A (per-test module):**
- Add helper in each `#[cfg(test)] mod tests` block
- Call in each test that uses `init_test_context()`

**Option B (global):**
- Add to a shared test-utils crate
- Create a test harness that wraps all lib tests to call it once per binary

**Long-term fix:**
- Modify `LemmyContext::init_test_context()` itself to call the helper automatically
- This is the most robust approach (no test-writer burden)

## Recommended DQ entries

1. **Move `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` setting to `.github/workflows/cargo-test-e2e.yml` AND `.woodpecker.yml`**
   - Prevents CI from accidentally running lib tests with config-file mode
   - Even if local CWD issue is fixed, CI-wide protection is hygiene best-practice

2. **Add a `#[ctor]` or global test fixture to set `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS=1` for all lib tests**
   - Use the `ctor` crate or Rust's native `#[ctor]` macro (if available in test target)
   - Centralized, one-time cost per test binary; no per-test boilerplate

3. **Document: "When adding a test that calls `LemmyContext::init_test_context()`, ensure `LEMMY_INITIALIZE_WITH_DEFAULT_SETTINGS` is set before any such call"**
   - Add to CONTRIBUTING.md or test-writing ADR

4. **Audit other LazyLock statics for similar issues**
   - Found one in `crates/apub/objects/src/utils/functions.rs:CACHE`
   - Review all `LazyLock` / `OnceLock` usages for test-context poisoning risk
