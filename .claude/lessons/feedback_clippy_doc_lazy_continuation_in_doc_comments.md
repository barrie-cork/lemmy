---
name: clippy::doc_lazy_continuation fires on `+` and indented continuations in /// comments
description: Workspace clippy with -D warnings flags `+` mid-line and similar tokens in doc comments as malformed list continuations. Reword to plain prose; do NOT add #[allow]. Two follow-up commits in one day; pattern recurs.
type: feedback
originSessionId: f3663548-c6e4-49d0-84b9-1077376a717b
---
Workspace clippy with `-D warnings` denies `clippy::doc_lazy_continuation` (implied by `-D clippy::style`). It fires when a `///` (or `//!`) doc comment has a continuation line that *looks* like a markdown list-item bullet but isn't indented under one.

**The trigger:** any line in a doc comment that starts with `+`, `-`, `*`, `1.`, or similar at column 0 (after `///`). Clippy reads it as a stray list bullet, not as the rest of the previous prose paragraph.

**Two recurring shapes:**

1. **`+` mid-paragraph for "and" or "plus":**
   ```rust
   /// Reject remote actor `+` enforce federated scope before
   /// the activity is constructed.
   ```
   Clippy: `error: doc list item without indentation` at the `+` line.

2. **Continuation lines under a paragraph that look like a list because of leading punctuation:**
   ```rust
   /// Step 1: validate scope.
   /// - sanction-notice from this instance an invariant violation, not a
   /// recoverable error. Returns `LemmyErrorType::Unknown` (no dedicated
   ```
   Clippy treats the `-` line as a list opener and the next two unindented lines as broken continuations.

**Fix shape (always):** reword to plain prose. Replace `+` with "and" or "plus"; spell out parentheticals; if you genuinely want a list, indent the continuation lines by 2 spaces.

```rust
/// Reject remote actor and enforce federated scope before
/// the activity is constructed.
```

**What NOT to do:**

- ❌ `#[allow(clippy::doc_lazy_continuation)]` — workspace forbids `clippy::allow_attributes` (per `feedback_clippy_test_style`). Adding the allow opens a new fight.
- ❌ Add a blank `///` line as the help text suggests — fixes the lint but reads worse than rewording.
- ❌ Move the doc comment to `#[doc = "..."]` — works but uglier and lint may follow.

**Recurrence count (2026-04-19):** at least 2× in a single day —
- polish-2 (PR #67): commit `17cbaa2d6` "fix(apub): reword assert_actor_is_local doc to satisfy doc_lazy_continuation lint" — needed after first clippy run failed.
- polish-3 PR #65 prep: same lint hit on `publish_sanction_notice.rs:405-408` ("+ at col 5" pattern) earlier the same day.

**Why it slips through pre-validation:**

`cargo check --workspace` does NOT run clippy. `cargo check --workspace --features full` does NOT run clippy. The lint only fires when `cargo clippy ... -- -D warnings` runs. Per-task validation that stops at `check` will green-light a doc-comment that clippy will reject in the next pass.

**Detection pattern for plan DoDs:**

Whenever a plan DoD includes `cargo check`, also include `cargo clippy ... -- -D warnings` for the same scope. This is independent of whether code paths are touched — doc-comment reformatting can fire the lint even on a "no semantic change" commit.

**Anti-pattern to avoid:** writing terse multi-line doc-comments under `///` with mid-paragraph special tokens. Default to plain prose continuations or properly-indented lists.
