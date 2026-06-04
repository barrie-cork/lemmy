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

**Why:** `CaseStatus` has **12** variants (confirmed 2026-06-04 at `crates/db_schema_file/src/enums.rs:393`;
the prior "13" count was stale — `CaseStatusTier` is a separate enum). Every new handler that
re-implements a runtime guard is a future diff site when a new variant lands.
The phantom wrapper centralises the check: add a variant → update one `TryFrom` match
per state that accepts it, not every handler that touches `case.status`.

**How to apply:**
1. New handler gates on exactly 1–2 variants → add a state marker to
   `crates/api/api/src/governance/state.rs` (the scaffold — NOT api_common; see Phase 7 recon
   for the placement rationale); call `GovernanceCase::try_from` immediately after the DB load.
2. Dual-role handlers (Original vs Appeal) → dispatch on `JuryAssignmentRole` FIRST, then
   `try_from` the appropriate state type per arm (pattern: `accept_jury_assignment.rs`).
3. **Site 6 pattern (success-not-error):** if the guard should return `Ok(...)` (not Err) on
   the rejected branch, use the `ActiveVoteResult` sentinel pattern:
   `GovernanceCase::<Active>::try_active_vote(case)` returns `AlreadyDecided` (map to Ok) or
   `Active(c)` (proceed). Do NOT use a plain `TryFrom` — that flips a 200 into a 404.
4. Return type stays `LemmyResult<()>`; keep `LemmyErrorType::NotFound` (NOT a new
   `InvalidCaseState` — would change HTTP status and leak internal state).
5. All 6 v1 `TODO(type-state)` sites were retrofitted in Phase 7 (2026-06-04, commits
   `ca1bfeab3` + `9d0048c16`). New handlers MUST use this pattern from day 1.

File-class trigger: any new file under `crates/api/api/src/governance/` that loads
a `ModerationCase` from DB and matches on `case.status` before proceeding.
