# Plan: Phase 3 — API Common DTOs (Tasks 31–37)

## Table of Contents

| Section | Line |
|---|---|
| Summary | 14 |
| Source | 20 |
| Problem Statement | 29 |
| Solution Statement | 34 |
| Metadata | 47 |
| §1 Divergence Analysis | 59 |
| §1.1 Watch 1: DTO Location | 61 |
| §1.2 Watch 2: Newtype IDs vs raw i32 | 113 |
| §1.3 Watch 3: ts-rs Derive Pattern | 158 |
| §1.4 Watch 4: Response Wrappers | 197 |
| §2 DTO Struct Definitions (13 structs) | 231 |
| §2.1 Group A — Reports + Cases (4 structs) | 235 |
| §2.2 Group B — Jury (3 structs) | 312 |
| §2.3 Group C — Appeals (1 struct) | 360 |
| §2.4 Group D — Reputation / Trust (3 structs) | 388 |
| §2.5 Group E — Public Log (1 struct) | 438 |
| §2.6 Group F — Re-export module (1 struct: SuccessResponse) | 465 |
| §3 Patterns to Mirror | 476 |
| §4 Files to Change | 546 |
| §5 Task Definitions (7 tasks) | 580 |
| Task 31 — Scaffold governance.rs + wire in lib.rs | 584 |
| Task 32 — Reports + Cases DTOs | 623 |
| Task 33 — Jury DTOs | 668 |
| Task 34 — Appeals + Public Log DTOs | 706 |
| Task 35 — Reputation / Trust DTOs | 747 |
| Task 36 — Cargo.toml wiring (deps + ts-rs feature) | 788 |
| Task 37 — Compile + ts-rs export check | 836 |
| §6 NOT Building (v0 scope limits) | 876 |
| §7 Validation Commands | 892 |
| §8 Open Questions to Escalate | 930 |
| §9 Risks and Mitigations | 946 |
| §10 Acceptance Criteria | 972 |
| §11 Completion Checklist | 992 |

---

## Summary

Phase 3 defines the 13 request/response DTO structs from [04 §5] in
`crates/api/api_common/src/governance.rs` so Phase 4 handlers have
type-safe inputs/outputs. This is a pure-types phase: no queries, no
handlers, no migrations, no business logic. Seven tasks, one commit
per task, all landing on a feature branch cut from governance-v0 at
`ce6d1775a`.

## Source

- [IMPLEMENTATION-PLAN-v0.md](docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)
  §3 Phase 3 (tasks 31–37)
- [04-data-model-and-api.md](docs/brehon-law-inspired-network/04-data-model-and-api.md)
  §5 (API request/response types)
- [05-mvp-and-delivery-plan.md](docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md)
  §4 Step 3 (definition of done)
- Relevant ADRs from [99](docs/brehon-law-inspired-network/99-decisions-and-open-questions.md):
  ADR-013 (EmergencyRemove), ADR-015 (GDPR pseudonyms)

## Problem Statement

Phase 4 handlers need typed request/response structs to compile. Without
them the handler signatures cannot be written. [04 §5] specifies 13 DTO
structs across five domain groups (reports, jury, appeals, reputation,
public log). These must exist, derive the correct serde + ts-rs stack,
use governance newtypes (not raw i32), and be importable from
`lemmy_api_common::governance`.

## Solution Statement

Create `crates/api/api_common/src/governance.rs` as a **definitions
module** (not a re-export hub). This is a deliberate, documented
deviation from the upstream Lemmy convention where `api_common` modules
are pure re-export hubs fronting DTOs defined in `db_views/<domain>/
src/api.rs`. The rationale is in §1.1 below. The file defines all 13
structs using governance newtypes from `lemmy_db_schema::newtypes` and
governance enums from `lemmy_db_schema_file::enums`, following the same
derive stack observed in upstream DTOs (`crates/db_views/comment/
src/api.rs`, `crates/db_views/modlog/src/api.rs`).

## Metadata

| Field | Value |
|---|---|
| Type | DTO |
| Complexity | LOW |
| Crates Affected | `api_common` (primary), workspace `Cargo.toml` (feature wiring) |
| v0 Step | Step 3 from [05 §4] |
| Dependencies | Phase 1 (newtypes + enums in db_schema/db_schema_file) |
| Estimated Tasks | 7 |
| Feature Branch | `feature/phase-3-api-common-dtos` from `governance-v0` at `ce6d1775a` |
| Commit Convention | `feat(api_common): task N — <summary>` |

---

## §1 Divergence Analysis

### §1.1 Watch 1: DTO Location

**The upstream convention:**
Upstream Lemmy 1.0-beta defines DTOs in `crates/db_views/<domain>/
src/api.rs`. The corresponding `crates/api/api_common/src/<domain>.rs`
module is a pure re-export hub — it contains zero struct definitions,
only `pub use` statements. Verified against:

- `crates/api/api_common/src/comment.rs` (lines 1–28) — 100% re-exports
  from `lemmy_db_views_comment::api::*`
- `crates/api/api_common/src/modlog.rs` (lines 1–2) — 100% re-exports
  from `lemmy_db_views_modlog::api::*`
- `crates/api/api_common/src/report.rs` (lines 1–32) — 100% re-exports
  from `lemmy_db_views_report_combined::api::*`
- `crates/api/api_common/src/person.rs` (lines 1–32) — 100% re-exports
  from `lemmy_db_views_person::api::*`

**The design doc:**
[04 §5] says DTOs go in `crates/api/api_common/src/governance.rs`.

**Decision: Option B — define DTOs directly in
`crates/api/api_common/src/governance.rs`.**

Rationale:

1. **Governance is fork-only code.** There is zero upstream merge-conflict
   risk. The upstream convention exists to isolate domain-specific types
   in their own crate so that multiple view crates don't need to depend
   on each other. For governance, all three view crates
   (governance_case, jury_queue, governance_modlog) are fork-only and
   have no upstream equivalent.

2. **Several DTOs don't map to any single view crate.**
   `CreateEndorsement`, `RevokeEndorsement`, `GetMyReputation`, and
   `RequestAppeal` span multiple tables (endorsement + surety +
   reputation_snapshot) or will map to a Phase 5 view crate
   (`reputation`) that doesn't exist yet.

3. **One file is simpler.** Splitting 13 structs across three or four
   `api.rs` files in different crates creates busywork with no
   structural benefit for fork-only code. A single 200-line file is
   easier to review and modify.

4. **Phase 4 can move to re-export-hub style later** if the governance
   view crates grow their own `api.rs`. The api_common module would
   then switch from struct definitions to re-exports, matching upstream.
   This is a one-commit refactor when it becomes useful — not now.

**What this means for Cargo.toml:**
Because the DTOs are defined in api_common itself (not in the view
crates), api_common does NOT need new dependencies on the governance
view crates in Phase 3. It only needs `lemmy_db_schema` (for newtypes)
and `lemmy_db_schema_file` (for enums), both of which are already
dependencies. The governance view crate dependencies get added in
Phase 4 when handlers need to import view structs.

### §1.2 Watch 2: Newtype IDs vs raw i32

**The design doc:**
[04 §5] uses raw `pub case_id: i32`, `pub person_id: i32`, etc.

**On-disk reality:**
Phase 1 defined 14 governance newtypes in
`crates/db_schema/src/newtypes.rs:206–293`:
- `ModerationCaseId(pub i32)`
- `CaseEvidenceId(pub i32)`
- `SanctionId(pub i32)`
- `AppealId(pub i32)`
- `PublicCaseLogId(pub i32)`
- `JuryPoolId(pub i32)`
- `JuryAssignmentId(pub i32)`
- `JuryVoteId(pub i32)`
- `SuretyId(pub i32)`
- `EndorsementId(pub i32)`
- `ReputationEventId(pub i32)`
- `ReputationSnapshotId(pub i32)`
- `ActorPseudonymId(pub i32)`
- `GovernanceLogId(pub i64)` — BIGSERIAL, not i32

Upstream Lemmy newtypes (PersonId, CommunityId, PostId, CommentId) are
in `crates/db_schema/src/newtypes.rs:6–42` and
`crates/db_schema_file/src/lib.rs:29–34` (PersonId lives in
db_schema_file, not db_schema).

**Decision: Use newtypes everywhere.**

Mapping from [04 §5] raw types to Phase 1 newtypes:

| [04 §5] field | Corrected type |
|---|---|
| `case_id: i32` | `case_id: ModerationCaseId` |
| `community_id: Option<i32>` | `community_id: Option<CommunityId>` |
| `target_id: i32` | See §2.1 note — this field is ambiguous |
| `person_id: i32` | `person_id: PersonId` |
| `endorsement_id: i32` | `endorsement_id: EndorsementId` |

Import paths:
```rust
use lemmy_db_schema::newtypes::{
    ModerationCaseId, EndorsementId,
    // + CommunityId, PostId, CommentId are also in db_schema::newtypes
};
use lemmy_db_schema_file::PersonId; // PersonId is in db_schema_file, not db_schema
```

### §1.3 Watch 3: ts-rs Derive Pattern

**Verified upstream pattern** from `crates/db_views/comment/src/api.rs:12–19`
and `crates/db_views/modlog/src/api.rs:10–15`:

```rust
// Full derive stack for a DTO struct WITH Option fields:
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ExampleDto { ... }

// For a response wrapper (often no Option fields):
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct ExampleResponse { ... }
```

**Default derive risk:**
All governance enums from Phase 1 derive `Default` (verified in
`crates/db_schema_file/src/enums.rs:374–599`):
- `CaseStatus` defaults to `Open`
- `CaseTargetType` defaults to `Post`
- `CaseSeverity` defaults to `Medium`
- `JuryDecision` defaults to `NoAction`
- `SanctionScope` defaults to `Community`
- `SanctionAction` defaults to `Label`
- `AppealStatus` defaults to `Requested`
- `CaseStatus` defaults to `Open`

Since all referenced governance enums implement `Default`, the `Default`
derive is safe on all DTO structs that reference them.

**Which feature flag to use:**
api_common's `Cargo.toml` (line 23) uses `ts-rs` as the feature name,
NOT `full`. The `full` feature (line 22) is empty. DTOs use
`#[cfg_attr(feature = "ts-rs", ...)]`.

### §1.4 Watch 4: Response Wrappers

**Upstream convention verified:**
Upstream Lemmy DOES use response wrappers. Every POST handler returns a
`*Response` struct wrapping a view type, not the view directly:
- `CommentResponse { comment_view: CommentView }` — `crates/db_views/comment/src/api.rs:17`
- `PostResponse { post_view: PostView }` — `crates/db_views/post/src/api.rs:235`
- `CommentReportResponse { comment_report_view: CommentReportView }` — `crates/db_views/report_combined/src/api.rs:43`

Paginated endpoints return `PagedResponse<ViewType>` from
`lemmy_diesel_utils::pagination` — already re-exported by api_common
lib.rs line 25.

**Decision for governance DTOs:**

Per the advisor instruction: "Do NOT create response wrappers unless
the upstream convention specifically requires them. Phase 4's handlers
will return Phase 2 view structs directly."

However, the upstream convention DOES require wrappers. Resolution:

- **`CreateGovernanceReportResponse`**: Keep as specified in [04 §5]. It
  is NOT a view wrapper — it returns `case_id + threshold_met`, which is
  handler-computed, not a read model. This is structurally different from
  `CommentResponse` (which wraps a view). The response struct IS needed.

- **GET endpoint responses**: Phase 4 handlers returning single items
  will wrap the Phase 2 view structs in thin response types (e.g.,
  `GovernanceCaseResponse { case_view: GovernanceCaseDetailView }`).
  Those response wrappers will be defined in Phase 4 when the handlers
  are written, NOT in Phase 3. Phase 3 only defines the request DTOs +
  the one `CreateGovernanceReportResponse` from [04 §5].

- **Paginated GET endpoints**: Will use the existing
  `PagedResponse<GovernanceCaseSummaryView>` pattern. No custom list-
  response struct needed.

**Net: Phase 3 defines 13 structs. 12 are request DTOs. 1 is
`CreateGovernanceReportResponse`.** Response wrappers for GET endpoints
are deferred to Phase 4 task definitions.

---

## §2 DTO Struct Definitions (13 structs)

### §2.1 Group A — Reports + Cases (4 structs)

Per [04 §5] "Reports and cases", corrected with newtypes.

**Struct 1: `CreateGovernanceReport`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a governance report (may open or append to a case).
/// Per [04 §6.1] and [04 §13] — every report creates/appends to a
/// `moderation_case` directly. No separate `report` table in v0.
pub struct CreateGovernanceReport {
  pub community_id: Option<CommunityId>,
  pub target_type: CaseTargetType,
  /// The ID of the target entity. Interpretation depends on `target_type`:
  /// - Post → PostId
  /// - Comment → CommentId
  /// - Person → PersonId (as i32)
  /// - Community → CommunityId (as i32)
  ///
  /// Kept as raw i32 because a single field serves multiple newtype
  /// domains. The handler in Phase 4 will cast to the appropriate
  /// newtype based on `target_type`.
  pub target_id: i32,
  pub reason_code: String,
  pub description: Option<String>,
}
```

**Note on `target_id: i32`:** This is the ONE field that stays raw i32.
[04 §5] defines it as a polymorphic ID — its meaning varies by
`target_type`. Wrapping it in a single newtype would be misleading.
The Phase 4 handler will match on `target_type` and cast to PostId /
CommentId / PersonId / CommunityId as needed.

**Struct 2: `CreateGovernanceReportResponse`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from creating a governance report.
/// `case_id` is present if a case was opened or appended to.
/// `threshold_met` indicates whether the case crossed the weight
/// threshold for jury selection.
pub struct CreateGovernanceReportResponse {
  pub case_id: Option<ModerationCaseId>,
  pub threshold_met: bool,
}
```

**Struct 3: `GetGovernanceCase`**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Read a single governance case. Permission-aware: caller role
/// determines which fields are visible (public / juror / admin).
pub struct GetGovernanceCase {
  pub case_id: ModerationCaseId,
}
```

No `#[skip_serializing_none]` — no Option fields. Added `Copy` since
all fields are Copy.

**Struct 4: `ListGovernanceCases`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// List governance cases, filtered by community and/or status.
/// Phase 4 handler wraps `list_open_cases_for_community` from Phase 2.
pub struct ListGovernanceCases {
  pub community_id: Option<CommunityId>,
  pub status: Option<CaseStatus>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}
```

### §2.2 Group B — Jury (3 structs)

Per [04 §5] "Jury", corrected with newtypes.

**Struct 5: `SubmitJuryVote`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Submit a jury vote on a case. Caller must have an `Accepted`
/// jury assignment for this case.
pub struct SubmitJuryVote {
  pub case_id: ModerationCaseId,
  pub decision: JuryDecision,
  pub rationale: Option<String>,
}
```

`Default` included per upstream convention — every request DTO derives
Default uniformly (`CreateComment`, `CreateCommentLike`, etc.).
`ModerationCaseId(0)` + `JuryDecision::NoAction` is nonsensical but
harmless; the handler validates.

**Struct 6: `AcceptJuryAssignment`**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Accept a jury assignment. Caller must have a `Selected`
/// assignment for this case.
pub struct AcceptJuryAssignment {
  pub case_id: ModerationCaseId,
}
```

**Struct 7: `DeclineJuryAssignment`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Decline a jury assignment. Triggers replacement juror selection.
pub struct DeclineJuryAssignment {
  pub case_id: ModerationCaseId,
  pub reason: Option<String>,
}
```

### §2.3 Group C — Appeals (1 struct)

Per [04 §5] "Appeals", corrected with newtypes.

**Struct 8: `RequestAppeal`**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Request an appeal on a decided case. Caller must be the sanction
/// target. Appeal window must be open (`now() < case.closed_at`).
/// In v0, appeals sit in `Appealed` state until admin closes — the
/// full re-jury flow is v1 per [05 §3].
pub struct RequestAppeal {
  pub case_id: ModerationCaseId,
  pub reason: String,
}
```

No `Default` or `Copy` — `reason: String` prevents Copy, and a default
empty reason is not meaningful. No `#[skip_serializing_none]` — no
Option fields.

### §2.4 Group D — Reputation / Trust (3 structs)

Per [04 §5] "Reputation / trust", corrected with newtypes.

**Struct 9: `GetMyReputation`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Get the calling user's own reputation summary.
/// Scoped to a community if `community_id` is provided.
/// Per [04 §6.2] — wraps `read_reputation_summary` from Phase 5.
pub struct GetMyReputation {
  pub community_id: Option<CommunityId>,
}
```

`Copy` is valid: `Option<CommunityId>` is `Option<i32>` which is Copy.

**Struct 10: `CreateEndorsement`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create an endorsement of another user. Per [04 §6.1]:
/// validates `can_sponsor` capability, enforces max-5-active and
/// 48h cooldown per [01 §5.1].
pub struct CreateEndorsement {
  pub person_id: PersonId,
  pub community_id: Option<CommunityId>,
}
```

**Struct 11: `RevokeEndorsement`**

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Revoke an existing endorsement. Per [04 §6.1]: authorise
/// revocation (must be the endorser), soft-revoke via `revoked_at`.
/// Note: [05 §2] lists endorsement/revoke as sprint-2 (deferred
/// from the initial 11), but defining the DTO costs nothing.
pub struct RevokeEndorsement {
  pub endorsement_id: EndorsementId,
}
```

No `#[skip_serializing_none]` — no Option fields.

### §2.5 Group E — Public Log (1 struct)

Per [04 §5] "Public log", corrected with newtypes.

**Struct 12: `ListGovernanceModlog`**

```rust
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// List the public governance modlog. Public endpoint, no auth required.
/// Per [04 §6.2] — wraps `list_public_case_log` from Phase 2b.
/// Pagination via `page` + `limit`.
pub struct ListGovernanceModlog {
  pub community_id: Option<CommunityId>,
  pub page: Option<i64>,
  pub limit: Option<i64>,
}
```

`Copy` is valid: all fields are `Option<i32>` or `Option<i64>`.

### §2.6 Group F — Re-export of SuccessResponse

**Struct 13 is NOT a new struct.** Upstream provides
`lemmy_db_views_site::api::SuccessResponse` (re-exported via
`lemmy_api_common::SuccessResponse` at lib.rs:21). Several governance
POST endpoints that don't return domain data (AcceptJuryAssignment,
DeclineJuryAssignment) will return `SuccessResponse` in Phase 4.
No action needed in Phase 3 — the type already exists.

**Revised count: 12 new structs defined + 1 existing type reused = 13
total types referenced by Phase 4 handlers.**

---

## §3 Patterns to Mirror

**UPSTREAM_DTO_DERIVE_STACK (with Option fields):**
```rust
// SOURCE: crates/db_views/comment/src/api.rs:22-30
// COPY THIS PATTERN for DTOs with Option fields:
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Create a comment.
pub struct CreateComment {
  pub content: String,
  pub post_id: PostId,
  pub parent_id: Option<CommentId>,
  pub language_id: Option<LanguageId>,
}
```

**UPSTREAM_DTO_DERIVE_STACK (no Option fields, Copy-able):**
```rust
// SOURCE: crates/db_views/comment/src/api.rs:35-41
// COPY THIS PATTERN for simple DTOs:
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Like a comment.
pub struct CreateCommentLike {
  pub comment_id: CommentId,
  pub is_upvote: Option<bool>,
}
```

**UPSTREAM_RESPONSE_STRUCT:**
```rust
// SOURCE: crates/db_views/comment/src/api.rs:11-18
// COPY THIS PATTERN for response wrappers:
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A comment response.
pub struct CommentResponse {
  pub comment_view: CommentView,
}
```

**UPSTREAM_RE_EXPORT_MODULE:**
```rust
// SOURCE: crates/api/api_common/src/modlog.rs:1-2
// When we later migrate to re-export pattern:
pub use lemmy_db_schema::{newtypes::ModlogId, source::modlog::Modlog};
pub use lemmy_db_views_modlog::api::GetModlog;
```

**GOVERNANCE_NEWTYPE_IMPORT:**
```rust
// SOURCE: crates/db_schema/src/newtypes.rs:211-215
// Import pattern for governance IDs:
use lemmy_db_schema::newtypes::{ModerationCaseId, EndorsementId};
```

**GOVERNANCE_ENUM_IMPORT:**
```rust
// SOURCE: crates/db_schema_file/src/enums.rs:382-408
// Import pattern for governance enums:
use lemmy_db_schema_file::enums::{CaseStatus, CaseTargetType, JuryDecision};
```

**PERSON_ID_IMPORT:**
```rust
// SOURCE: crates/db_schema_file/src/lib.rs:29-34
// PersonId is in db_schema_file, NOT db_schema:
use lemmy_db_schema_file::PersonId;
```

---

## §4 Files to Change

| # | File | Action | Task | Justification |
|---|---|---|---|---|
| 1 | `crates/api/api_common/Cargo.toml` | UPDATE | 31 | Add `serde` + `serde_with` direct deps (first api_common module to define structs) |
| 2 | `crates/api/api_common/src/governance.rs` | CREATE | 31 | New module — all 12 DTO structs |
| 3 | `crates/api/api_common/src/lib.rs` | UPDATE | 31 | Add `pub mod governance;` |
| 4 | `crates/api/api_common/src/governance.rs` | UPDATE | 32 | Add Group A structs (4) |
| 5 | `crates/api/api_common/src/governance.rs` | UPDATE | 33 | Add Group B structs (3) |
| 6 | `crates/api/api_common/src/governance.rs` | UPDATE | 34 | Add Group C + E structs (2) |
| 7 | `crates/api/api_common/src/governance.rs` | UPDATE | 35 | Add Group D structs (3) |
| 8 | No file changes | VERIFY | 36 | Cargo.toml verification (no-op — deps added in task 31) |
| 9 | No file changes | VERIFY | 37 | Compile check + ts-rs feature check |

**Files NOT changed:**
- NO changes to Phase 1 or Phase 2 source files
- NO changes to `crates/db_schema/`
- NO changes to `crates/db_views/governance_case/`
- NO changes to `crates/db_views/jury_queue/`
- NO changes to `crates/db_views/governance_modlog/`
- NO migrations
- NO handler code
- NO route code

**Cargo.toml note:** Task 31 adds `serde` and `serde_with` as direct
dependencies — needed because governance.rs is the first api_common
module to define structs (every other module is a pure re-export hub).
`lemmy_db_schema` and `lemmy_db_schema_file` are already present. The
`ts-rs` feature already activates `lemmy_db_schema/ts-rs` and
`lemmy_db_schema_file/ts-rs`. The governance view crate deps
(`lemmy_db_views_governance_case`, etc.) are NOT needed yet — they'll
be added in Phase 4 when handlers import view structs.

---

## §5 Task Definitions (7 tasks)

### Task 31 — Scaffold governance.rs + wire in lib.rs

**ACTION:** Create the governance module file and wire it into the
api_common crate.

**IMPLEMENT:**

1. Add `serde` and `serde_with` as direct dependencies to
   `crates/api/api_common/Cargo.toml` under `[dependencies]`:
   ```toml
   serde.workspace = true
   serde_with.workspace = true
   ```
   Both are already declared in the workspace root `Cargo.toml`.
   Needed because governance.rs is the first api_common module to
   define structs directly (Option B) — every other module is a pure
   re-export hub that never imports serde. Rust 2021 edition does not
   allow importing transitive dependencies.

2. Create `crates/api/api_common/src/governance.rs` with the module
   header and imports:

```rust
use lemmy_db_schema::newtypes::{CommunityId, EndorsementId, ModerationCaseId};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseStatus, CaseTargetType, JuryDecision},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
```

3. Add `pub mod governance;` to `crates/api/api_common/src/lib.rs`
   after line 18 (after `pub mod tagline;`).

**MIRROR:** `crates/api/api_common/src/modlog.rs` for module wiring;
`crates/db_views/comment/src/api.rs` for import structure.

**GOTCHA:** The import set in the scaffold is a starting set — tasks
32–35 will add more imports as needed (e.g., `JuryDecision` for
task 33). Keep the import block sorted alphabetically by crate.

**VALIDATE:**
```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common"
```
Expected: exit 0 (empty module compiles).

**COMMIT:** `feat(api_common): task 31 — scaffold governance DTO module`

---

### Task 32 — Reports + Cases DTOs

**ACTION:** Add the 4 Group A structs to governance.rs.

**IMPLEMENT:** Add the 4 structs from §2.1 exactly as specified:
- `CreateGovernanceReport`
- `CreateGovernanceReportResponse`
- `GetGovernanceCase`
- `ListGovernanceCases`

**IMPORTS to add (if not already present):**
```rust
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId};
use lemmy_db_schema_file::enums::{CaseStatus, CaseTargetType};
```

**MIRROR:** `crates/db_views/comment/src/api.rs:22–30` (CreateComment
pattern) and `crates/db_views/modlog/src/api.rs:10–37` (GetModlog
pattern for list DTOs).

**GOTCHA:** `target_id: i32` stays raw — see §2.1 note on polymorphic
IDs. Do NOT wrap it in a newtype.

**GOTCHA:** `CreateGovernanceReportResponse.case_id` is
`Option<ModerationCaseId>`, not `ModerationCaseId` — the case may not
be created if the report appends to an existing case that hasn't
crossed the threshold.

**VALIDATE:**
```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common"
```
Expected: exit 0.

**COMMIT:** `feat(api_common): task 32 — reports + cases DTOs`

---

### Task 33 — Jury DTOs

**ACTION:** Add the 3 Group B structs to governance.rs.

**IMPLEMENT:** Add the 3 structs from §2.2:
- `SubmitJuryVote`
- `AcceptJuryAssignment`
- `DeclineJuryAssignment`

**IMPORTS to add:**
```rust
use lemmy_db_schema_file::enums::JuryDecision;
```

**MIRROR:** `crates/db_views/comment/src/api.rs:35–41`
(CreateCommentLike pattern for simple action DTOs).

**GOTCHA:** If `Default` is needed for `SubmitJuryVote` for
deserialization compatibility, add it. But verify the upstream pattern:
`CreateComment` derives Default (has `String::default()`),
`CreateCommentLike` derives Default (has `Option<bool>`). Since
`SubmitJuryVote` has `ModerationCaseId` (which derives Default as
`ModerationCaseId(0)`) and `JuryDecision` (which defaults to
`NoAction`), Default CAN be derived. Include it for consistency.

**VALIDATE:**
```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common"
```
Expected: exit 0.

**COMMIT:** `feat(api_common): task 33 — jury DTOs`

---

### Task 34 — Appeals + Public Log DTOs

**ACTION:** Add Group C (1 struct) and Group E (1 struct) to
governance.rs. Grouped because they're trivially small.

**IMPLEMENT:** Add 2 structs from §2.3 and §2.5:
- `RequestAppeal`
- `ListGovernanceModlog`

**IMPORTS to add:** None beyond what's already present (ModerationCaseId,
CommunityId already imported in task 31/32).

**MIRROR:** `crates/db_views/modlog/src/api.rs:10–37` (GetModlog for
pagination pattern).

**GOTCHA:** `RequestAppeal` does NOT derive `Default` or `Copy`
because `reason: String` prevents Copy and an empty default reason is
semantically wrong. But: if the upstream convention is to always derive
Default on request DTOs (which it is — `CreateComment` has
`String::default()` and still derives Default), then include Default
for consistency. **The implementer should follow upstream: include
Default.** An empty String is accepted by serde but the handler will
validate.

**VALIDATE:**
```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common"
```
Expected: exit 0.

**COMMIT:** `feat(api_common): task 34 — appeals + public log DTOs`

---

### Task 35 — Reputation / Trust DTOs

**ACTION:** Add Group D (3 structs) to governance.rs.

**IMPLEMENT:** Add 3 structs from §2.4:
- `GetMyReputation`
- `CreateEndorsement`
- `RevokeEndorsement`

**IMPORTS to add:**
```rust
use lemmy_db_schema::newtypes::EndorsementId;
use lemmy_db_schema_file::PersonId;
```

**MIRROR:** `crates/db_views/comment/src/api.rs:35–41`
(CreateCommentLike for simple action DTOs).

**GOTCHA:** `PersonId` is imported from `lemmy_db_schema_file`, NOT
from `lemmy_db_schema::newtypes`. Verified at
`crates/db_schema_file/src/lib.rs:29–34`. This is a Lemmy convention —
PersonId lives in the "schema file" crate because it's referenced
across the crate boundary between db_schema and db_schema_file.

**GOTCHA:** `CreateEndorsement` and `RevokeEndorsement` are in v0 scope
([05 §2] lists endorsement as endpoint 11). `RevokeEndorsement` is
sprint-2 per [05 §2] ("No endorsement revoke") but defining the DTO
now costs nothing and avoids a Phase 5 edit to governance.rs.

**VALIDATE:**
```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common"
```
Expected: exit 0.

**COMMIT:** `feat(api_common): task 35 — reputation + trust DTOs`

---

### Task 36 — Cargo.toml verification (no-op)

**ACTION:** Verify that `crates/api/api_common/Cargo.toml` has all
necessary dependencies and feature flags after tasks 31–35.

**IMPLEMENT:** This task is a verification checkpoint — no file changes
expected. Task 31 already added `serde` and `serde_with`. Confirm:

1. **Dependencies:** `lemmy_db_schema`, `lemmy_db_schema_file`, `serde`,
   `serde_with` all present in `[dependencies]`.
2. **ts-rs feature:** `lemmy_db_schema/ts-rs` and
   `lemmy_db_schema_file/ts-rs` present in `[features] ts-rs`.
3. **No governance view crate deps yet:** `lemmy_db_views_governance_case`,
   `lemmy_db_views_jury_queue`, `lemmy_db_views_governance_modlog` are
   NOT referenced — those are Phase 4 work.

**If all checks pass, skip the commit** — an empty commit is pointless.
Merge the verification into task 35's commit or proceed directly to
task 37.

**If a missing dep is discovered**, fix it here and commit as:
`fix(api_common): task 36 — add missing dep for governance DTOs`

**VALIDATE:**
```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common"
```
Expected: exit 0 (same as task 35).

---

### Task 37 — Compile + ts-rs export check

**ACTION:** Run all validation commands against the completed
governance.rs module. This is the phase-close verification.

**IMPLEMENT:**

1. Run `cargo check -p lemmy_api_common` — basic compile.
2. Run `cargo check -p lemmy_api_common --features ts-rs` — ts-rs
   derives compile under the feature flag.
3. Run `cargo clippy -p lemmy_api_common --no-deps -- -D warnings`
   via `cargo-clippy.bat` — no lint warnings introduced.
4. Run `cargo check --workspace` — workspace-wide check to confirm
   no breakage.

**VALIDATE (all four commands):**
```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common > .claude/build-task37a.log 2>&1"
echo "exit: $?"
tail -20 .claude/build-task37a.log

cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common --features ts-rs > .claude/build-task37b.log 2>&1"
echo "exit: $?"
tail -20 .claude/build-task37b.log

cmd //c "scripts\brehon\cargo-clippy.bat -p lemmy_api_common --no-deps -- -D warnings > .claude/build-task37c.log 2>&1"
echo "exit: $?"
tail -20 .claude/build-task37c.log

cmd //c "scripts\brehon\cargo-check.bat --workspace > .claude/build-task37d.log 2>&1"
echo "exit: $?"
tail -20 .claude/build-task37d.log
```

Expected: all exit 0.

**COMMIT:** `docs(report): task 37 — Phase 3 compile verification`

---

## §6 NOT Building (v0 scope limits)

- **No queries** — DTO structs only, no impl blocks
- **No handler code** — handlers are Phase 4
- **No migration code** — schema is Phase 1
- **No route code** — routes are Phase 4
- **No editing Phase 1 or Phase 2 source files** — DTOs only import
  from existing types
- **No fields beyond [04 §5]** — the 12 structs match the design doc
  exactly, with newtype corrections
- **No Queryable/Selectable derives** — DTOs are request/response
  types, not DB rows
- **No response wrappers for GET endpoints** — deferred to Phase 4
  when handlers are written
- **No `api.rs` files in governance view crates** — those would follow
  the upstream convention of view-crate-local DTOs, but we're using
  Option B (centralized in api_common) per §1.1
- **No governance view crate deps in api_common Cargo.toml** — deferred
  to Phase 4

---

## §7 Validation Commands

Use these exact commands. Do NOT substitute npm/pnpm/etc. This is Rust.

### Level 1: PER-CRATE CHECK (run after every task)

```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common"
```
**EXPECT:** exit 0, zero errors.

### Level 2: TS-RS FEATURE CHECK (run at task 37)

```
cmd //c "scripts\brehon\cargo-check.bat -p lemmy_api_common --features ts-rs"
```
**EXPECT:** exit 0. If the `ts-rs` feature doesn't exist on
api_common, this is a plan bug — the feature IS declared at
Cargo.toml:23.

### Level 3: CLIPPY (run at task 37)

```
cmd //c "scripts\brehon\cargo-clippy.bat -p lemmy_api_common --no-deps -- -D warnings"
```
**EXPECT:** exit 0, zero warnings.

### Level 4: WORKSPACE CHECK (run at task 37)

```
cmd //c "scripts\brehon\cargo-check.bat --workspace"
```
**EXPECT:** exit 0. Confirms governance DTOs don't break anything.

### Validation NOT needed in Phase 3:

- No `cargo test` — there are no tests for DTO struct definitions
- No `diesel migration` — no schema changes
- No cross-cutting verification — no writes to governance log
- No manual curl — no endpoints to call

---

## §8 Open Questions to Escalate

### Questions that surfaced during planning

**Q1: `target_id` polymorphic field.**
[04 §5] defines `CreateGovernanceReport.target_id: i32` as a
polymorphic ID whose meaning depends on `target_type`. This plan keeps
it as raw `i32` because no single newtype is correct. Phase 4's handler
must cast to the right newtype. **Is this acceptable, or should the DTO
use a tagged union?**

Advisor recommendation in the briefing was to use newtypes "where [04
§5] says `case_id: i32`, the plan must specify `case_id:
ModerationCaseId`" — but `target_id` is a special case where the field
is genuinely polymorphic. The plan chooses raw i32 with a doc comment.

**Q2: Default derive on SubmitJuryVote.**
The plan includes Default on `SubmitJuryVote` for upstream consistency
(`CreateComment` derives Default), but a vote with
`ModerationCaseId(0)` and `JuryDecision::NoAction` is meaningless.
Include Default or not?

**Neither Q1 nor Q2 blocks Phase 3 execution.** The implementer should
follow the plan as written and revisit in Phase 4 if issues arise.

---

## §9 Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| Import path wrong (newtype/enum not found) | LOW | LOW | Task 36 verifies all imports compile. Exploration confirmed exact paths. |
| Default derive fails on a governance enum | LOW | LOW | All governance enums confirmed to derive Default in Phase 1 (§1.3). |
| ts-rs feature breaks on governance enums | LOW | MED | ts-rs derives confirmed on all governance newtypes and enums in Phase 1. Task 37 validates. |
| Upstream rebase changes api_common lib.rs | LOW | LOW | governance.rs is fork-only — no merge conflict with upstream modules. The only upstream-touching line is `pub mod governance;` in lib.rs. |
| Clippy deny-warnings catches an upstream lint in api_common | MED | LOW | Use `--no-deps` flag to scope clippy to api_common only, not transitive deps. |
| Phase 4 needs response wrappers we didn't define | LOW | LOW | Phase 4 can add them — the wrapper pattern is trivial (2-field struct + derive stack). |

---

## §10 Acceptance Criteria

- [ ] All 12 governance DTO structs compile in `crates/api/api_common/src/governance.rs`
- [ ] `pub mod governance;` in lib.rs exports the module
- [ ] Every struct follows the upstream derive stack pattern (§3)
- [ ] Every ID field uses the Phase 1 newtype (except `target_id: i32`, documented in §2.1)
- [ ] Every enum field uses the Phase 1 enum from `lemmy_db_schema_file::enums`
- [ ] `PersonId` is imported from `lemmy_db_schema_file`, not `lemmy_db_schema`
- [ ] `#[skip_serializing_none]` is present on every struct with Option fields
- [ ] `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]` + `ts(optional_fields, export)` on every struct
- [ ] `cargo check -p lemmy_api_common` passes (exit 0)
- [ ] `cargo check -p lemmy_api_common --features ts-rs` passes (exit 0)
- [ ] `cargo check --workspace` passes (exit 0)
- [ ] No new cargo clippy warnings introduced
- [ ] No Phase 1 or Phase 2 source files were modified
- [ ] No fields beyond [04 §5] were added
- [ ] No Queryable/Selectable derives on any DTO struct

---

## §11 Completion Checklist

- [ ] Task 31 completed: governance.rs scaffold + lib.rs wiring
- [ ] Task 32 completed: 4 reports + cases DTOs
- [ ] Task 33 completed: 3 jury DTOs
- [ ] Task 34 completed: 2 appeals + public log DTOs
- [ ] Task 35 completed: 3 reputation + trust DTOs
- [ ] Task 36 completed: Cargo.toml verification
- [ ] Task 37 completed: full compile + ts-rs + clippy + workspace check
- [ ] All 7 commits landed on `feature/phase-3-api-common-dtos`
- [ ] All acceptance criteria (§10) met
- [ ] Plan file updated with completion notes (if any deviations)
