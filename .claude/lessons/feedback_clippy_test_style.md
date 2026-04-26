---
name: Lemmy workspace test-style constraints
description: How to write test functions in the brehon-fork / Lemmy workspace without tripping the strict clippy config
type: feedback
originSessionId: c40129b7-ba97-4f49-af10-23426290c800
---
The Lemmy workspace `[workspace.lints.clippy]` denies, among other things:

- `unwrap_used`
- `expect_used`
- `allow_attributes` — so `#[allow(clippy::unwrap_used)]` does NOT work as an escape hatch
- `tests_outside_test_module`
- `dbg_macro`, `panic`, `unreachable`, `indexing_slicing`, and many more

**Two legal escape hatches for tests:**

1. `async fn -> LemmyResult<()>` with `?` propagation. This is the idiomatic Lemmy pattern — used in `crates/db_views/post/src/test.rs` and throughout `crates/api/api_utils/src/utils.rs`. Requires depending on `lemmy_utils` (for `LemmyResult`).
2. `async fn -> Result<(), Box<dyn std::error::Error>>` with `?` propagation. Cleaner for tests that don't otherwise pull in lemmy_utils — used in Phase 0's `crates/server/tests/e2e.rs`. No extra crate dependencies.
3. `#[expect(clippy::unwrap_used, clippy::tests_outside_test_module)]` is allowed (because `expect` attributes are different from `allow` attributes under `allow_attributes = deny`). Used in `crates/utils/tests/test_errors_used.rs`. This is the escape hatch when neither Result approach fits.

**Never** write `.expect("...")` or `.unwrap()` in a fresh test body without one of the three patterns above — the workspace clippy will reject it at lint time, even in integration tests under `tests/*.rs`. Plan files written at a higher abstraction level may show `.expect(...)` examples; those are illustrative, not literal, and need to be rewritten to satisfy clippy.

**How to apply:** When writing a new integration test in this workspace, pick Pattern 2 (`Box<dyn Error>`) if the test has no lemmy-native crate deps, Pattern 1 (`LemmyResult`) if it does, or Pattern 3 (`#[expect(...)]`) only if neither is viable.
