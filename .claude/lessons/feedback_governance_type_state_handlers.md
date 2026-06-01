---
name: feedback_governance_type_state_handlers
description: Wrap loaded ModerationCase in a phantom-typed struct when a new handler requires a specific CaseStatus to be valid; shifts precondition from distributed runtime match to a single compile-time boundary
type: feedback
originSessionId: 2026-05-31-advisor
---

When a new governance handler accepts only one or two `CaseStatus` variants, define a
phantom-typed wrapper so the precondition is in the function signature, not repeated
in another match block.

```rust
// Define once in crates/api_common/src/governance/state.rs
use std::marker::PhantomData;
use lemmy_db_schema::source::governance::moderation_case::ModerationCase;
use lemmy_utils::error::{LemmyError, LemmyErrorType, LemmyResult};
use lemmy_db_schema_file::enums::CaseStatus;

pub struct JurySelection;
pub struct InReview;
pub struct Appealed;
// … one zero-sized struct per CaseStatus variant you need

pub struct GovernanceCase<S> {
  pub inner: ModerationCase,
  _state: PhantomData<S>,
}

impl TryFrom<ModerationCase> for GovernanceCase<InReview> {
  type Error = LemmyError;
  fn try_from(c: ModerationCase) -> LemmyResult<Self> {
    match c.status {
      CaseStatus::InReview => Ok(Self { inner: c, _state: PhantomData }),
      _ => Err(LemmyErrorType::NotFound.into()),
    }
  }
}
// One impl per state marker; the match is exhaustive here, nowhere else.
```

Handler signature becomes:
```rust
async fn accept_vote(case: GovernanceCase<InReview>, …) -> LemmyResult<()>
```

**Why:** `CaseStatus` has 13 variants (v1 will add more). Every new handler that
re-implements a runtime guard is a future diff site when a new variant lands.
The phantom wrapper centralises the check: add a variant → update one `TryFrom` match
per state that accepts it, not every handler that touches `case.status`.

**How to apply:**
1. New handler gates on exactly 1–2 variants → define state marker(s) in
   `crates/api_common/src/governance/state.rs`; call `GovernanceCase::try_from`
   immediately after the DB load.
2. Dual-role handlers (Original vs Appeal) → dispatch on `JuryAssignmentRole` FIRST
   (existing pattern in `accept_jury_assignment.rs:110`), then `try_from` the
   appropriate state type for each arm.
3. Do NOT retrofit existing handlers in the same task — add
   `// TODO(type-state): …` at the guard site (see existing annotations) and reference
   this lesson. Retrofit is tracked separately.
4. Return type stays `LemmyResult<()>`; no new error crates needed. If
   `LemmyErrorType::InvalidCaseState` is missing, add it to the enum in
   `crates/utils/src/error.rs` before using it here.

File-class trigger: any new file under `crates/api/api/src/governance/` that loads
a `ModerationCase` from DB and matches on `case.status` before proceeding.
