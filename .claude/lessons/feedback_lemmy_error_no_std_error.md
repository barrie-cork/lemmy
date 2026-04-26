---
name: LemmyError doesn't implement std::error::Error
description: LemmyResult<T> wraps LemmyError which can't be ?-converted into Box<dyn Error>. Tests calling Lemmy-native functions need .map_err(|e| format!("{e}").into()) bridge. Surfaced at Phase 2b task 30 smoke tests.
type: feedback
originSessionId: e31990ad-143c-4e12-9bfb-8b135b83d7e6
---
`LemmyError` does not implement `std::error::Error`, so `LemmyResult<T>` cannot use `?` in test functions that return `Result<(), Box<dyn Error>>`.

**Why:** Phase 2b task 30 smoke tests call view-crate query functions returning `LemmyResult<Vec<View>>`. The test function signature is `async fn foo() -> Result<(), Box<dyn Error>>`. The `?` operator on `LemmyResult` fails because `LemmyError` doesn't satisfy the `Into<Box<dyn Error>>` bound. Mechanical fix: `.map_err(|e| format!("{e}").into())` on every Lemmy-native call.

**How to apply:** Any test (Phase 3 DTO tests, Phase 4 handler tests, Phase 5 reputation tests) that calls a function returning `LemmyResult` will hit this. Add the `.map_err(...)` bridge at the call site. Don't attempt to patch `LemmyError` upstream — that's an upstream decision. The bridge is 20 characters per call and compiles cleanly.
