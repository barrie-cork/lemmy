# Plan: Phase 2a — `governance_case` + `jury_queue` read-model crates

## Summary

Phase 2a ships two of the three Phase 2 view crates — `crates/db_views/governance_case` and `crates/db_views/jury_queue` — with the structs from [04 §4.1] / [04 §4.2] and the queries named in [IMPLEMENTATION-PLAN-v0.md §Phase 2] tasks 14–24. No migrations, no new Diesel source types, no API layer, no smoke tests (those belong to Phase 2b task 30). Every file mirrors the existing `crates/db_views/modlog` + `crates/db_views/report_combined` conventions verified against HEAD `c51147754`. The phase lands on a throwaway branch `feature/phase-2a-governance-case-jury-queue` cut from `governance-v0`, with one commit per task (11 task commits).

## Source

- [IMPLEMENTATION-PLAN-v0.md §Phase 2, tasks 14–24](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) — authoritative task bodies
- [04 §4.1 and §4.2](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) — canonical view struct shapes
- [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) — source-table Diesel model listings (ground-truth cross-check)
- [05 §3 + §4 Step 2](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) — v0 simplifications (5-juror panel, quorum 3)
- ADRs: ADR-007 (hardcoded capability checks), ADR-010 (solo-dev stack), ADR-013 (report→case collapse + EmergencyRemove), ADR-015 (GDPR pseudonyms)

## Problem Statement

Phase 4's governance handlers need read models that can answer six concrete questions against the Phase 1 tables: (1) what cases are open in this community, (2) what is the full hydrated detail of case X, (3) what cases is person P a target of, (4) what cases are awaiting jury selection, (5) what jury assignments does person P currently hold, (6) how many un-submitted jury assignments exist for the timeout job. Without these queries the Phase 4 handlers cannot be written, and without the crates themselves those queries have nowhere to live.

## Solution Statement

Create two workspace members under `crates/db_views/` that follow the `modlog` + `report_combined` convention verbatim: `Cargo.toml` with workspace inheritance + `full`/`ts-rs` feature gates, `src/lib.rs` with the view structs under `#[cfg_attr(feature = "full", derive(Queryable, Selectable))]`, and `src/impls.rs` (gated `#[cfg(feature = "full")]`) with one free function per query. Queries return plain `Vec<View>` (no pagination wrapper — Phase 4 handlers add that layer) and take `pool: &mut DbPool<'_>`. No `api.rs` submodule is needed in Phase 2a because DTOs land in Phase 3's `api_common`.

## Metadata

| Field | Value |
|---|---|
| Type | READ_MODEL |
| Complexity | MEDIUM (no migrations, but join queries + Phase 1 drift require care) |
| Crates Affected | `db_views/governance_case` (new), `db_views/jury_queue` (new), workspace `Cargo.toml` |
| v0 Step | Step 2 of [05 §4](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) |
| Dependencies | Phase 1 (ships Diesel source types, enums, newtypes, schema bindings) |
| Tasks | 11 (task numbers 14–24 from the main plan) |
| Branch | `feature/phase-2a-governance-case-jury-queue` (cut from `governance-v0`) |
| Iteration Budget | `--max-iterations 25` for `/prp-ralph` |
| Soft cap per task | 5 iterations — stop-and-diagnose if exceeded |
| Confidence | **8/10** — see §Confidence rationale below |

---

## 1. Divergences from main plan ([IMPLEMENTATION-PLAN-v0.md §Phase 2])

Three structural corrections surfaced from actually reading the workspace. None change scope.

1. **`responded_at`, not `accepted_at`, on `jury_assignment`.** The main plan body mentions "`selected_at` / `accepted_at`" informally. The Phase 1 schema binding at `crates/db_schema_file/src/schema.rs:460-468` and the source struct at `crates/db_schema/src/source/governance/jury_assignment.rs:17-25` both expose `responded_at: Option<DateTime<Utc>>` — there is no `accepted_at`. `list_jury_assignments_for_person` must sort by `selected_at` (per task 22's "Sorted by `selected_at`" note), not invent an `accepted_at`.
2. **Query return shape — `Vec<View>`, not `PagedResponse<View>`.** The main plan is silent on pagination and the existing `modlog::impls` uses `PagedResponse` because its caller is a route handler. In Phase 2a the queries are internal helpers consumed by Phase 4 handlers which themselves will wrap in pagination. Mirroring Lemmy convention, we return `LemmyResult<Vec<T>>` from the view crates and let Phase 4 decide pagination shape. This also avoids coupling Phase 2a to `PaginationCursor` ergonomics that `modlog` inherited from the `combined` infrastructure — governance has no combined table, so the cursor would be dead weight.
3. **`Selectable` via `#[diesel(embed)]` for embedded source rows.** `GovernanceCaseDetailView { case_row: ModerationCase, sanctions: Vec<Sanction>, ... }` mixes an embedded Selectable row (`ModerationCase`) with a count (`evidence_count: i64`) and a collection (`sanctions: Vec<Sanction>`). Diesel Queryable cannot load a `Vec<Sanction>` in a single select, so `read_case_detail` is a **multi-query function**: one select for the case + counts using `embed`, plus a second query for `Sanction::belonging_to(&case_row)`. This is an implementation detail — not a divergence from [04] — but it changes task 17's shape enough to call out here.

## 2. Divergences from [04] — prose-vs-code drift to log

Three fields in [04 §4.1] / [04 §4.2] reference source columns that **do not exist on the Phase 1 schema or on the [04 §3] source-table listings themselves**. [04] is internally inconsistent: §3 defines `ModerationCase` with 16 columns (verified both in the canonical doc at line 183-204 and in the Phase 1 implementation); §4 then references `reporter_count`, `jury_needed`, `jury_submitted`, and `deadline_at` which cannot be sourced from `ModerationCase` or `JuryAssignment` directly.

**These are NOT Phase 1 bugs — they are prose-vs-code drift inside [04].** Code in Phase 1 matches [04 §3] exactly. Fixing them by adding schema columns would contradict [04 §3]. The correct move per ADR discipline is: compute each field at query time from Phase 1 state, and log the drift for homeserver-side correction of [04 §4]. Code wins.

### Drift 1 — `GovernanceCaseSummaryView.reporter_count: i64`

- **Spec location:** [04 §4.1](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) line 421
- **Problem:** There is no `case_report` table, no `reporter_count` column on `moderation_case`, and no `reporters` JSONB. ADR-013 collapses reports into `moderation_case` itself ([04 §13 shortcut]), which means "reporter count" has no natural source other than counting distinct reporter identities — but no such identity trail is stored.
- **Phase 2a resolution:** **Return `0_i64`** as a stub constant in the Selectable expression. The field satisfies the [04] contract at the type level; Phase 4 handlers and the Phase 2b golden-path test will surface the need if any consumer relies on a non-zero value, at which point Phase 3+ can introduce a proper `case_report` table in its own ADR. Document the stub with a `TODO(brehon-fork): [04 §4.1] drift — reporter_count has no source in Phase 1 schema` comment at the field initialization site.
- **Why stub and not a new table in Phase 2a:** Adding schema is a migration, which the briefing explicitly forbids for Phase 2a ("No migrations, no new Diesel models, no new enums").
- **Homeserver-side action (out of this repo):** [04 §4.1] should either (a) mark these fields as derived from a new reports table to be added in Phase 1, or (b) remove them from the view spec. Advisor to decide which when this plan is reviewed.

### Drift 2 — `GovernanceCaseSummaryView.jury_needed: i32` and `jury_submitted: i32`

- **Spec location:** [04 §4.1](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) lines 422-423
- **Problem:** `ModerationCase` has no `jury_size_target` column. Per [05 §3] the v0 simplification hardcodes jury size to 5, so `jury_needed` is a constant. `jury_submitted` is a count of `jury_assignment` rows with `status = Submitted` for the case — which IS derivable, but requires a subquery or a separate aggregation round-trip.
- **Phase 2a resolution:** `jury_needed = 5` as a hardcoded constant (cite [05 §3]). `jury_submitted` is computed via a correlated `COUNT(*) FILTER (WHERE status = 'submitted')` subquery on `jury_assignment` joined by `case_id` — expressed as a `diesel::dsl::sql::<BigInt>()` expression embedded in the select list. This keeps `list_open_cases_for_community` a single round-trip. If the `sql::<BigInt>` macro is fragile in the current Diesel version, fall back to two round-trips (main query + a grouped `.load::<(i32, i64)>` for the counts) — this fallback is logged as a fallback risk below.
- **Homeserver-side action:** [04 §4.1] should either document `jury_needed` as "constant per [05 §3]" or add a `jury_size_target` column to `ModerationCase` in [04 §3]. Same for `jury_submitted` — either document as "derived from jury_assignment" or provide a denormalized counter column.

### Drift 3 — `JuryQueueView.deadline_at: Option<DateTime<Utc>>`

- **Spec location:** [04 §4.2](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) line 449
- **Problem:** `jury_assignment` has `selected_at`, `responded_at`, `submitted_at` — no `deadline_at`. `moderation_case` has `decided_at` and `closed_at` — neither of which is "jury deadline." The timeout-job semantics for task 24 are "how many jury assignments are unsubmitted past their deadline," but the deadline itself isn't stored.
- **Phase 2a resolution:** **Return `None`** for `deadline_at` in the JuryQueueView Selectable expression. Document with a `TODO(brehon-fork): [04 §4.2] drift — deadline_at has no source in Phase 1 schema; Phase 4 jury-timeout job will need either a deadline column or a computed offset from selected_at` comment. Task 24's `count_unsubmitted_jury_assignments` can still be implemented — it just returns the count where `status = 'selected'` or `status = 'accepted'` with no time filter for Phase 2a. The actual timeout filter is a Phase 4 concern and depends on the homeserver-side decision about whether to add a column or compute `selected_at + 72h`.
- **Homeserver-side action:** [04 §4.2] should clarify whether `deadline_at` is (a) stored on `jury_assignment` (column addition, new migration), (b) computed as `selected_at + config_duration` (hardcoded constant), or (c) deferred out of v0 entirely.

## 3. Phase 1 follow-ups discovered

**Zero.** Every struct, enum, typed ID, table binding, and derive macro the Phase 2a queries touch is already in place and correct:

- `ModerationCase`, `JuryAssignment`, `Sanction`, `Appeal`, `CaseEvidence` all have `#[derive(Queryable, Selectable, Identifiable)]` behind `feature = "full"` (verified at `crates/db_schema/src/source/governance/moderation_case.rs:14`, `jury_assignment.rs:14`, `sanction.rs:14`, `appeal.rs:14`, `case_evidence.rs:14`).
- All typed IDs exist in `crates/db_schema/src/newtypes.rs`: `ModerationCaseId` (line 215), `CaseEvidenceId` (221), `SanctionId` (227), `AppealId` (233), `JuryAssignmentId` (251), and the non-governance IDs used by joins (`CommunityId` line 42, `PostId` line 11, `CommentId` line 24, `PersonId` from `lemmy_db_schema_file`).
- `CaseStatus::EmergencyRemove` exists (`crates/db_schema_file/src/enums.rs:404`) per ADR-013. All other enums in [04 §4.1] / [04 §4.2] scope exist with all variants.
- `crates/db_schema_file/src/schema.rs` has table bindings for `moderation_case` (687-704), `jury_assignment` (460-468), `sanction` (1120-1132), `appeal` (126-134), `case_evidence` (141-150), plus the joinable! declarations (1211-1306) that Diesel needs for join queries.
- `crates/db_schema/src/utils.rs` / `lemmy_diesel_utils::connection` provides `DbPool`, `get_conn`, and the pool-wiring that tests use.

The three drift items in §2 are spec-level inconsistencies in [04 §4], not Phase 1 implementation gaps.

---

## 4. Flow Design

Phase 2a is backend-only. The "flow" is the dependency chain from Phase 4 handlers (future) down into Phase 2a queries down into Phase 1 source tables.

### Before state

```
╔════════════════════════════════════════════════════════════════════════════╗
║                       PHASE 1 HEAD (c51147754)                              ║
╠════════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║   lemmy_db_schema::source::governance::{moderation_case, jury_assignment,  ║
║     sanction, appeal, case_evidence, ...}   — 14 source structs            ║
║                                                                            ║
║   lemmy_db_schema_file::schema::{moderation_case, jury_assignment, ...}    ║
║     — table bindings + joinable! declarations                              ║
║                                                                            ║
║   CONSUMER: none yet — no read models, no API, no handlers                 ║
║                                                                            ║
╚════════════════════════════════════════════════════════════════════════════╝
```

### After state

```
╔════════════════════════════════════════════════════════════════════════════╗
║                       PHASE 2A HEAD (11 commits ahead)                      ║
╠════════════════════════════════════════════════════════════════════════════╣
║                                                                            ║
║   crates/db_views/governance_case:                                         ║
║     pub struct GovernanceCaseSummaryView {...}                             ║
║     pub struct GovernanceCaseDetailView {...}                              ║
║     #[cfg(feature="full")] mod impls:                                      ║
║       fn list_open_cases_for_community(pool, CommunityId) -> Vec<Summary>  ║
║       fn read_case_detail(pool, ModerationCaseId)       -> DetailView      ║
║       fn list_cases_for_person(pool, PersonId)          -> Vec<Summary>    ║
║       fn list_cases_needing_jury_selection(pool)        -> Vec<Summary>    ║
║                                                                            ║
║   crates/db_views/jury_queue:                                              ║
║     pub struct JuryQueueView {...}                                         ║
║     #[cfg(feature="full")] mod impls:                                      ║
║       fn list_jury_assignments_for_person(pool, PersonId)     -> Vec<J.Q>  ║
║       fn list_available_jury_cases_for_person(pool, PersonId) -> Vec<J.Q>  ║
║       fn count_unsubmitted_jury_assignments(pool)             -> i64       ║
║                                                                            ║
║   CONSUMER: Phase 3 (api_common DTOs) and Phase 4 (handlers) can call in   ║
║                                                                            ║
╚════════════════════════════════════════════════════════════════════════════╝
```

### Endpoint changes

None. Phase 2a adds no HTTP routes. Phase 4 wires the endpoints that consume these queries.

---

## 5. Mandatory Reading (implementation agent MUST read before task 1)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | `docs/brehon-law-inspired-network/04-data-model-and-api.md` | 407-458 | Canonical `GovernanceCaseSummaryView` / `DetailView` / `JuryQueueView` shapes + query lists |
| P0 | `crates/db_views/modlog/Cargo.toml` | 1-42 | Exact Cargo.toml template for a db_views crate — workspace inheritance, feature gates |
| P0 | `crates/db_views/modlog/src/lib.rs` | 1-49 | Exact lib.rs shape — `#[cfg(feature = "full")] pub mod impls`, `Queryable`+`Selectable` derives gated by feature, `#[diesel(embed)]` on source-row fields |
| P0 | `crates/db_views/modlog/src/impls.rs` | 1-177 | Canonical join-query pattern: `joins()` fn with `auto_type`, `.inner_join()` / `.left_join()`, `.select(X::as_select())`, `get_conn(pool).await?`, `.load::<X>(conn).await?` |
| P0 | `crates/db_views/report_combined/Cargo.toml` | 1-45 | Second Cargo.toml template (shows `chrono` + `ts-rs` gating when the crate uses timestamps) |
| P0 | `crates/db_schema/src/source/governance/moderation_case.rs` | 1-56 | `ModerationCase` struct (all 16 fields, typed IDs, `Selectable` derive) |
| P0 | `crates/db_schema/src/source/governance/jury_assignment.rs` | 1-34 | `JuryAssignment` struct (7 fields, `responded_at` NOT `accepted_at`) |
| P0 | `crates/db_schema/src/source/governance/sanction.rs` | 1-47 | `Sanction` struct — needed for `read_case_detail`'s `Vec<Sanction>` embed |
| P0 | `crates/db_schema/src/source/governance/appeal.rs` | 1-35 | `Appeal` struct for `appeal_status` sourcing |
| P0 | `crates/db_schema_file/src/enums.rs` | 382-568 | All governance enums — every match on `CaseStatus` must cover `EmergencyRemove` + `AdminReview` |
| P0 | `crates/db_schema_file/src/schema.rs` | 460-468, 687-704, 1034-1132, 1211-1306 | Table bindings + `joinable!` for `moderation_case`, `jury_assignment`, `sanction`, `appeal`, `case_evidence`, `public_case_log`, `reputation_snapshot` |
| P0 | `crates/db_schema/src/newtypes.rs` | 11, 24, 42, 215-251 | Typed ID definitions for Phase 2a query signatures |
| P0 | `Cargo.toml` (workspace root) | 1-9, 27-68, 109-147 | `[workspace.package]` inheritance fields + `[workspace] members` format + `[workspace.dependencies]` path-dep format |
| P0 | `.claude/rules/cargo-output-capture.md` | all 69 | Capture every cargo invocation to a log file; pipes mask exit codes |
| P0 | `.claude/rules/no-cargo-output-paste.md` | all 70 | Never dump full cargo logs into conversation; `tail -20` from the log file |
| P1 | `crates/db_views/report_combined/src/lib.rs` | 1-90 | Second lib.rs template for the more complex embed case (if `governance_case` detail view needs the report_combined style) |
| P1 | `crates/db_views/modlog/src/impls.rs` | 204-350 | Test pattern — `#[cfg(test)]`, `init_data`/`cleanup`, `build_db_pool_for_tests`, `#[serial]`. Phase 2a adds NO tests, but the pattern is required for any future Phase 2b task-30 additions |
| P1 | `crates/server/tests/e2e.rs` | all | Verify Phase 1's `phase1_migrations_round_trip` is intact — the Phase 2a final gate re-runs this test and expects `4 passed, 0 failed` |
| P1 | `docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md` | 176-220 | Phase 2 task bodies (tasks 14–30 — Phase 2a implements 14–24 only, know the Phase 2b boundary) |

**External documentation:** none new. Phase 2a introduces no new crate dependencies beyond what `crates/db_views/modlog` already pulls in transitively.

---

## 6. Patterns to Mirror

Every snippet below is copied verbatim from the workspace at HEAD `c51147754`. **Do not invent alternatives.**

### Cargo.toml skeleton

```toml
# SOURCE: crates/db_views/modlog/Cargo.toml:1-42
# NEW FILE: crates/db_views/governance_case/Cargo.toml
#           crates/db_views/jury_queue/Cargo.toml

[package]
name = "lemmy_db_views_governance_case"   # or lemmy_db_views_jury_queue
version.workspace = true
edition.workspace = true
description.workspace = true
license.workspace = true
homepage.workspace = true
documentation.workspace = true
repository.workspace = true
rust-version.workspace = true

[lib]
doctest = false

[lints]
workspace = true

[features]
full = [
  "lemmy_utils",
  "diesel",
  "diesel-async",
  "i-love-jesus",
  "lemmy_db_schema/full",
  "lemmy_db_schema_file/full",
  "lemmy_diesel_utils/full",
]
ts-rs = ["dep:ts-rs", "lemmy_db_schema/ts-rs", "lemmy_db_schema_file/ts-rs"]

[dependencies]
lemmy_db_schema = { workspace = true }
lemmy_utils = { workspace = true, optional = true }
lemmy_db_schema_file = { workspace = true }
lemmy_diesel_utils = { workspace = true }
diesel = { workspace = true, optional = true }
diesel-async = { workspace = true, optional = true }
serde = { workspace = true }
serde_with = { workspace = true }
chrono = { workspace = true }
ts-rs = { workspace = true, optional = true }
i-love-jesus = { workspace = true, optional = true }
```

Phase 2a needs `chrono` explicitly because the view structs carry `DateTime<Utc>` fields directly (unlike `modlog` which inherits chrono transitively — the `report_combined` crate explicitly lists it, which is the pattern to mirror).

### lib.rs skeleton — view struct with embedded source rows

```rust
// SOURCE PATTERN: crates/db_views/modlog/src/lib.rs:1-49
// NEW FILE: crates/db_views/governance_case/src/lib.rs

use chrono::{DateTime, Utc};
use lemmy_db_schema::source::governance::{
  appeal::Appeal,
  moderation_case::ModerationCase,
  sanction::Sanction,
};
use lemmy_db_schema_file::enums::{
  AppealStatus,
  CaseSeverity,
  CaseStatus,
  CaseTargetType,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
#[cfg(feature = "full")]
use diesel::{Queryable, Selectable};

#[cfg(feature = "full")]
pub mod impls;

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
pub struct GovernanceCaseSummaryView {
  pub case_id: i32,
  pub status: CaseStatus,
  pub severity: CaseSeverity,
  pub reason_code: String,
  pub opened_at: DateTime<Utc>,
  pub community_id: Option<i32>,
  pub community_name: Option<String>,
  pub target_type: CaseTargetType,
  pub reporter_count: i64,
  pub jury_needed: i32,
  pub jury_submitted: i32,
}

#[skip_serializing_none]
#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "full", derive(Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export, optional_fields))]
pub struct GovernanceCaseDetailView {
  #[cfg_attr(feature = "full", diesel(embed))]
  pub case_row: ModerationCase,
  #[cfg_attr(feature = "full", diesel(select_expression = diesel::dsl::sql::<diesel::sql_types::BigInt>("0")))]
  pub evidence_count: i64,
  pub appeal_status: Option<AppealStatus>,
  pub target_creator_id: Option<i32>,
  // sanctions is populated by a second query in read_case_detail — not part of the
  // single-row Selectable shape. Field stays on the struct for API contract fidelity.
  #[serde(default)]
  #[cfg_attr(feature = "full", diesel(skip_insertion))]
  pub sanctions: Vec<Sanction>,
}
```

**Exact shape TBD by the implementer** — the plan commits to the field list from [04 §4.1]. The `#[diesel(skip_insertion)]` / non-Queryable treatment of `sanctions` is necessary because `Vec<Sanction>` is a fan-out and Diesel can't produce it in a single row load. The implementer's task 17 is the right place to finalize whether `sanctions` stays on the struct (populated by a second query in `read_case_detail`) or lives on a wrapper returned by `read_case_detail` — both are acceptable, the first matches [04] literally, the second is cleaner Rust. **Default decision: keep the field on the struct, populate via a second query in the impl, so [04]'s contract is preserved verbatim.** If during task 15 implementation this turns out to collide with Queryable derivation, split into two structs and record the deviation in this plan's Divergences §1.

### Query pattern — list with filters + join

```rust
// SOURCE PATTERN: crates/db_views/modlog/src/impls.rs:43-67 (joins fn) + 109-177 (list fn)
// NEW FILE: crates/db_views/governance_case/src/impls.rs

use crate::GovernanceCaseSummaryView;
use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, dsl::sql};
use diesel::sql_types::BigInt;
use diesel_async::RunQueryDsl;
use lemmy_db_schema::newtypes::{CommunityId, ModerationCaseId, PersonId};
use lemmy_db_schema_file::enums::CaseStatus;
use lemmy_db_schema_file::schema::{community, jury_assignment, moderation_case};
use lemmy_diesel_utils::connection::{DbPool, get_conn};
use lemmy_utils::error::LemmyResult;

pub async fn list_open_cases_for_community(
  pool: &mut DbPool<'_>,
  community_id: CommunityId,
) -> LemmyResult<Vec<GovernanceCaseSummaryView>> {
  let conn = &mut get_conn(pool).await?;

  let open_statuses = [
    CaseStatus::Open,
    CaseStatus::ThresholdMet,
    CaseStatus::JurySelection,
    CaseStatus::InReview,
  ];

  moderation_case::table
    .left_join(community::table.on(community::id.nullable().eq(moderation_case::community_id)))
    .filter(moderation_case::community_id.eq(community_id))
    .filter(moderation_case::status.eq_any(open_statuses))
    .select((
      moderation_case::id,
      moderation_case::status,
      moderation_case::severity,
      moderation_case::reason_code,
      moderation_case::opened_at,
      moderation_case::community_id,
      community::name.nullable(),
      moderation_case::target_type,
      sql::<BigInt>("0"),                // reporter_count — [04 §4.1] drift stub
      sql::<diesel::sql_types::Integer>("5"),  // jury_needed — [05 §3] constant
      sql::<diesel::sql_types::Integer>(
        "(SELECT COUNT(*)::int4 FROM jury_assignment ja \
          WHERE ja.case_id = moderation_case.id AND ja.status = 'submitted')"
      ),
    ))
    .load::<GovernanceCaseSummaryView>(conn)
    .await
    .map_err(Into::into)
}
```

The `sql::<BigInt>` and correlated-subquery approach is the least-ceremony way to handle the §2 drift. If the implementer discovers the subquery plus tuple select pattern is too fragile (for example, `sql::<Integer>("5")` cast complaints), the fallback is: do a plain select of raw columns, load into a tuple, then build `GovernanceCaseSummaryView` in a `.map(...)` post-processing step with the constants inlined in Rust. This is slower but more resilient. Task 16 picks which path; record the decision in the task commit body.

### Query pattern — detail view with two queries

```rust
// SOURCE PATTERN: same impls.rs join style, but with a belonging_to second pass
pub async fn read_case_detail(
  pool: &mut DbPool<'_>,
  case_id: ModerationCaseId,
) -> LemmyResult<GovernanceCaseDetailView> {
  let conn = &mut get_conn(pool).await?;

  // First query: case + evidence count + appeal status + target_creator_id
  // (target_creator_id joined from post.creator_id / comment.creator_id / target_person_id)
  let (case_row, evidence_count, appeal_status, target_creator_id) =
    moderation_case::table
      .filter(moderation_case::id.eq(case_id))
      .left_join(appeal::table.on(appeal::case_id.eq(moderation_case::id)))
      .left_join(post::table.on(post::id.nullable().eq(moderation_case::target_post_id)))
      .left_join(comment::table.on(comment::id.nullable().eq(moderation_case::target_comment_id)))
      .select((
        ModerationCase::as_select(),
        sql::<BigInt>(
          "(SELECT COUNT(*) FROM case_evidence ce WHERE ce.case_id = moderation_case.id)"
        ),
        appeal::status.nullable(),
        // target_creator_id COALESCE: post.creator_id, comment.creator_id, target_person_id
        sql::<diesel::sql_types::Nullable<diesel::sql_types::Int4>>(
          "COALESCE(post.creator_id, comment.creator_id, moderation_case.target_person_id)"
        ),
      ))
      .first::<(ModerationCase, i64, Option<AppealStatus>, Option<i32>)>(conn)
      .await?;

  // Second query: sanctions belonging to this case
  let sanctions = sanction::table
    .filter(sanction::case_id.eq(case_id))
    .select(Sanction::as_select())
    .load::<Sanction>(conn)
    .await?;

  Ok(GovernanceCaseDetailView {
    case_row,
    evidence_count,
    appeal_status,
    target_creator_id,
    sanctions,
  })
}
```

### Workspace root Cargo.toml additions

```toml
# SOURCE: Cargo.toml:27-68  (members list, keep alphabetical grouping under crates/db_views/)
# EDIT 1: add two new members
[workspace]
members = [
  # ... existing entries ...
  "crates/db_views/governance_case",  # NEW — Phase 2a task 14
  "crates/db_views/jury_queue",       # NEW — Phase 2a task 20
  # ... rest ...
]

# SOURCE: Cargo.toml:109-147  (workspace dependencies, add two path deps)
# EDIT 2: add two path-dep entries (version pinned to workspace package version)
[workspace.dependencies]
# ... existing entries ...
lemmy_db_views_governance_case = { version = "=1.0.0-test-arm-qemu.0", path = "./crates/db_views/governance_case" }
lemmy_db_views_jury_queue = { version = "=1.0.0-test-arm-qemu.0", path = "./crates/db_views/jury_queue" }
```

### Commit message format

```
feat(db_views): task 14 — create lemmy_db_views_governance_case crate

Validation: scripts/brehon/cargo-check.bat -p lemmy_db_views_governance_case
captured to .claude/build-task-14.log; exit=0.
```

- Subject: `<type>(<scope>): task N — <summary>` (max 72 chars).
- Allowed types: `feat`, `test`, `refactor`, `chore`, `docs`.
- Non-task commits (plan tweaks, lockfile chores) use the same types **without** the `task N —` prefix.
- Body: one line naming the validation command and its captured log path + exit code.
- No `Co-Authored-By` footer — this is a Phase 2a convention decision because Phase 1 commits lacked consistent authorship and the retro didn't mandate any particular form. The `/prp-ralph` loop signs commits directly via git config.

---

## 7. Files to Change

| # | File | Action | Justification |
|---|---|---|---|
| 1 | `Cargo.toml` (workspace root) | UPDATE — add 2 members + 2 workspace deps | Required for new crates to be visible to `cargo check --workspace` |
| 2 | `Cargo.lock` | UPDATE (auto via `cargo check`) | Regenerated on first workspace build after new crate creation; stage alongside Cargo.toml edit per Phase 1 feedback memory |
| 3 | `crates/db_views/governance_case/Cargo.toml` | CREATE (task 14) | New crate manifest — see §6 template |
| 4 | `crates/db_views/governance_case/src/lib.rs` | CREATE (task 14) | Crate root — empty `pub mod impls;` scaffold in task 14, structs land in task 15 |
| 5 | `crates/db_views/governance_case/src/lib.rs` | UPDATE (task 15) | Add `GovernanceCaseSummaryView` + `GovernanceCaseDetailView` structs |
| 6 | `crates/db_views/governance_case/src/impls.rs` | CREATE (task 16) | First query function + `use` imports |
| 7 | `crates/db_views/governance_case/src/impls.rs` | UPDATE (tasks 17, 18, 19) | Additional query functions, each its own commit |
| 8 | `crates/db_views/jury_queue/Cargo.toml` | CREATE (task 20) | New crate manifest — same template |
| 9 | `crates/db_views/jury_queue/src/lib.rs` | CREATE (task 20) | Crate root — empty scaffold |
| 10 | `crates/db_views/jury_queue/src/lib.rs` | UPDATE (task 21) | Add `JuryQueueView` struct |
| 11 | `crates/db_views/jury_queue/src/impls.rs` | CREATE (task 22) | First jury-queue query function |
| 12 | `crates/db_views/jury_queue/src/impls.rs` | UPDATE (tasks 23, 24) | Additional query functions |
| 13 | `.claude/build-task-14.log` … `.claude/build-task-24.log` | CREATE (each task) | Captured cargo output per `cargo-output-capture.md` |
| 14 | `.claude/build-phase2a-final-check.log` | CREATE (final gate) | `cargo check --workspace` capture |
| 15 | `.claude/build-phase2a-final-test.log` | CREATE (final gate) | `cargo test --test e2e -p lemmy_server` capture |

**Files NOT touched (scope guardrails):**
- `crates/db_views/governance_modlog/**` — Phase 2b
- `crates/db_views/reputation/**` — Phase 5
- `crates/api/**` — Phase 3/4
- `crates/server/tests/e2e.rs` — no new tests in Phase 2a; the existing Phase 1 tests must still pass
- `migrations/**` — no migrations in Phase 2a
- `crates/db_schema/src/source/**` — Phase 1 types are frozen for Phase 2a
- `crates/db_schema_file/src/schema.rs` — no new table bindings

---

## 8. NOT Building (v0 scope limits restated)

- **No `crates/db_views/governance_modlog` crate** — Phase 2b (tasks 25–29)
- **No task 30 smoke tests in `tests/e2e.rs`** — Phase 2b
- **No `crates/db_views/reputation` crate** — deferred to Phase 5 per [IMPLEMENTATION-PLAN-v0.md §Phase 2] closing note
- **No API layer** (`api_common`, `api`, `api_crud`, `api/routes`) — Phase 3 and Phase 4
- **No federation code** — Phase 6
- **No endpoint registration** — Phase 4
- **No golden-path test (`report_to_modlog_golden_path`)** — Phase 4 task 48
- **No pagination wrapper (`PagedResponse<View>`)** — Phase 4 handlers add it; Phase 2a returns plain `Vec<T>`
- **No new Diesel migrations, models, or enums** — explicit briefing constraint
- **No Phase 1 schema fixes** — the two drift items in §2 are [04] prose corrections, not schema changes; if the advisor decides otherwise, that's a separate Phase 1 follow-up branch, not absorbed into Phase 2a
- **No touching `phase1_migrations_round_trip`** — if a Phase 2a change wants to modify the e2e test, something is wrong

---

## 9. Step-by-Step Tasks

**Task numbering matches [IMPLEMENTATION-PLAN-v0.md §Phase 2] tasks 14–24.** One commit per task. Each task has exactly one validation command that writes to `.claude/build-task-NN.log`.

### Task 0 (branch setup — not a task commit)

- **ACTION:** `git switch governance-v0 && git switch -c feature/phase-2a-governance-case-jury-queue`
- **Verify baseline:** local `governance-v0` matches `c51147754` exactly (Phase 1 HEAD + `no-cargo-output-paste` rule commit). `origin/governance-v0` should match `c51147754` or be an ancestor of it. Ancestry check: `git merge-base --is-ancestor 94eba51a0 c51147754` exits 0 (proves the Phase 1 merge HEAD is still in the chain).
- **If local `governance-v0` is past `c51147754`,** something has advanced the baseline since the plan was written. Surface to advisor before cutting the feature branch — the advisor needs to know whether the new commits belong in Phase 2a's history or should be left behind.
- **If `origin/governance-v0` has advanced past `c51147754`,** that's fine — someone or some later session pushed more. Fast-forward local to match (`git pull --ff-only`) so the feature branch is cut from the freshest baseline, then re-verify ancestry from `94eba51a0`.
- **Verification steps (run all three; all must pass before task 14 begins):**
    1. `git rev-parse governance-v0` → must equal `c51147754fd9eedc8f646099ec14c362b4a3d9c6` (or any later commit that descends from it — after applying the fast-forward rule above).
    2. `git merge-base --is-ancestor 94eba51a0 c51147754` → must exit 0, confirming the Phase 1 merge HEAD is still an ancestor.
    3. `git rev-parse feature/phase-2a-governance-case-jury-queue` → must equal local `governance-v0` HEAD immediately after the branch cut (proving the cut happened at the expected commit).
- **No commit created for task 0** — branch creation is a git operation, not a source change.
- **Expected iterations:** 1 (one git operation + three verification commands, no validation beyond `git status` clean).

### Task 14 — create `crates/db_views/governance_case` crate scaffold

- **Files created:** `crates/db_views/governance_case/Cargo.toml`, `crates/db_views/governance_case/src/lib.rs`
- **Files updated:** `Cargo.toml` (workspace root — add member + workspace dep), `Cargo.lock` (auto)
- **Implement:** Cargo.toml from §6 template verbatim (name = `lemmy_db_views_governance_case`). Empty `src/lib.rs` with just a module-level doc comment and nothing else — **no structs, no impls** (those land in task 15 and 16 respectively).
- **Rationale:** Tasks 14 and 20 per the briefing must be "new crate creation with no struct definitions" — crate creation is its own iteration boundary.
- **DoD:** `scripts/brehon/cargo-check.bat -p lemmy_db_views_governance_case` exits 0 against an empty lib.
- **Validation:** `scripts/brehon/cargo-check.bat -p lemmy_db_views_governance_case > .claude/build-task-14.log 2>&1` — read via `tail -20 .claude/build-task-14.log`
- **Commit staging:** `git diff --stat HEAD` MUST show exactly `Cargo.toml`, `Cargo.lock`, `crates/db_views/governance_case/Cargo.toml`, `crates/db_views/governance_case/src/lib.rs`. If `Cargo.lock` is missing, halt and investigate before committing (Phase 1 task-13 footgun).
- **Commit subject:** `feat(db_views): task 14 — create lemmy_db_views_governance_case crate`
- **Expected iterations:** 2 (one for crate creation, one for lockfile staging fix if the first attempt missed it)

### Task 15 — `GovernanceCaseSummaryView` + `GovernanceCaseDetailView` structs

- **Files updated:** `crates/db_views/governance_case/src/lib.rs`
- **Implement:** two struct definitions per §6 template, with all fields from [04 §4.1], derive macros per §6 template (`#[cfg_attr(feature = "full", derive(Queryable, Selectable))]`, etc.), imports at top from `lemmy_db_schema::source::governance::*` and `lemmy_db_schema_file::enums::*`. The `sanctions: Vec<Sanction>` field on `GovernanceCaseDetailView` is flagged with `#[diesel(skip_insertion)]` or similar — the implementer picks the exact attribute when Diesel complains, guided by the §6 fallback note.
- **DoD:** `scripts/brehon/cargo-check.bat -p lemmy_db_views_governance_case` exits 0; `cargo clippy -p lemmy_db_views_governance_case -- -D warnings` exits 0.
- **Validation:** same as task 14, log `.claude/build-task-15.log`
- **Commit subject:** `feat(db_views): task 15 — GovernanceCaseSummaryView + GovernanceCaseDetailView structs`
- **Expected iterations:** 2 (struct definition + fixup for Diesel Queryable field-order or embed-attribute warnings)

### Task 16 — query `list_open_cases_for_community`

- **Files created:** `crates/db_views/governance_case/src/impls.rs`
- **Files updated:** `crates/db_views/governance_case/src/lib.rs` (add `#[cfg(feature = "full")] pub mod impls;`)
- **Implement:** `pub async fn list_open_cases_for_community(pool: &mut DbPool<'_>, community_id: CommunityId) -> LemmyResult<Vec<GovernanceCaseSummaryView>>` per §6 query pattern. Filters by `community_id` equals + `status IN (Open, ThresholdMet, JurySelection, InReview)`. Computes `reporter_count = 0` (drift stub), `jury_needed = 5` (constant per [05 §3]), `jury_submitted` via correlated subquery on `jury_assignment` WHERE `case_id = moderation_case.id AND status = 'submitted'`.
- **Gotcha — `EmergencyRemove` and `AdminReview` excluded:** the filter `status IN (Open, ThresholdMet, JurySelection, InReview)` intentionally excludes both `EmergencyRemove` and `AdminReview`. This matches the task body "cases still needing action" semantics. **Do NOT add them** — they belong to Phase 4 admin views.
- **DoD:** `scripts/brehon/cargo-check.bat -p lemmy_db_views_governance_case` + clippy both clean.
- **Validation:** log `.claude/build-task-16.log`
- **Commit subject:** `feat(db_views): task 16 — list_open_cases_for_community query`
- **Expected iterations:** 2 (first pass + a probable fixup for the correlated-subquery `sql::<Integer>` typing)

### Task 17 — query `read_case_detail`

- **Files updated:** `crates/db_views/governance_case/src/impls.rs`
- **Implement:** `pub async fn read_case_detail(pool: &mut DbPool<'_>, case_id: ModerationCaseId) -> LemmyResult<GovernanceCaseDetailView>` per §6 two-query pattern. First query hydrates case row + evidence count + appeal status + target_creator_id (COALESCE over post.creator_id / comment.creator_id / target_person_id). Second query loads `Vec<Sanction>` via `sanction::table.filter(sanction::case_id.eq(case_id))`. **Redaction and permission checks are the Phase 4 handler's job, not this function's.** The view returns the fully-hydrated shape; the handler decides what to strip.
- **Gotcha — `EmergencyRemove` exhaustive match:** `read_case_detail` returns a `case_row: ModerationCase` which carries `CaseStatus`. This function itself doesn't match on status, but any caller that does must handle `EmergencyRemove` and `AdminReview` exhaustively per ADR-013. The doc comment on the function should remind the Phase 4 caller of this invariant.
- **DoD:** crate + clippy both clean.
- **Validation:** log `.claude/build-task-17.log`
- **Commit subject:** `feat(db_views): task 17 — read_case_detail query`
- **Expected iterations:** 2 (likely 1 if task 16 already ironed out `sql::` typing pain)

### Task 18 — query `list_cases_for_person`

- **Files updated:** `crates/db_views/governance_case/src/impls.rs`
- **Implement:** `pub async fn list_cases_for_person(pool: &mut DbPool<'_>, target_person_id: PersonId) -> LemmyResult<Vec<GovernanceCaseSummaryView>>`. Reuses the summary-view join shape from task 16; filter changes to `moderation_case::target_person_id.eq(target_person_id)` (no community filter, no status filter — a target sees all cases they're named in, active or closed).
- **DoD:** check + clippy clean.
- **Validation:** log `.claude/build-task-18.log`
- **Commit subject:** `feat(db_views): task 18 — list_cases_for_person query`
- **Expected iterations:** 1 (mechanical variation of task 16)

### Task 19 — query `list_cases_needing_jury_selection`

- **Files updated:** `crates/db_views/governance_case/src/impls.rs`
- **Implement:** `pub async fn list_cases_needing_jury_selection(pool: &mut DbPool<'_>) -> LemmyResult<Vec<GovernanceCaseSummaryView>>`. Filter: `moderation_case::status.eq(CaseStatus::ThresholdMet)`. No community scope (this feeds the Phase 4 background job which scans instance-wide).
- **DoD:** check + clippy clean.
- **Validation:** log `.claude/build-task-19.log`
- **Commit subject:** `feat(db_views): task 19 — list_cases_needing_jury_selection query`
- **Expected iterations:** 1

### Task 20 — create `crates/db_views/jury_queue` crate scaffold

- **Files created:** `crates/db_views/jury_queue/Cargo.toml`, `crates/db_views/jury_queue/src/lib.rs`
- **Files updated:** `Cargo.toml` (workspace root — add member + workspace dep), `Cargo.lock` (auto)
- **Implement:** Cargo.toml from §6 template (name = `lemmy_db_views_jury_queue`). Empty `src/lib.rs`. Same iteration-boundary rule as task 14.
- **DoD:** `scripts/brehon/cargo-check.bat -p lemmy_db_views_jury_queue` exits 0.
- **Validation:** log `.claude/build-task-20.log`
- **Commit staging:** same Cargo.lock check as task 14 — halt if missing.
- **Commit subject:** `feat(db_views): task 20 — create lemmy_db_views_jury_queue crate`
- **Expected iterations:** 2

### Task 21 — `JuryQueueView` struct

- **Files updated:** `crates/db_views/jury_queue/src/lib.rs`
- **Implement:** one struct per [04 §4.2]. All seven fields: `case_id: i32`, `severity: CaseSeverity`, `reason_code: String`, `opened_at: DateTime<Utc>`, `deadline_at: Option<DateTime<Utc>>`, `community_id: Option<i32>`, `community_name: Option<String>`. `deadline_at` is always `None` for Phase 2a (drift stub per §2). Derive macros follow §6 template.
- **DoD:** check + clippy clean.
- **Validation:** log `.claude/build-task-21.log`
- **Commit subject:** `feat(db_views): task 21 — JuryQueueView struct`
- **Expected iterations:** 1

### Task 22 — query `list_jury_assignments_for_person`

- **Files created:** `crates/db_views/jury_queue/src/impls.rs`
- **Files updated:** `crates/db_views/jury_queue/src/lib.rs` (add `#[cfg(feature = "full")] pub mod impls;`)
- **Implement:** `pub async fn list_jury_assignments_for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<Vec<JuryQueueView>>`. Joins `jury_assignment.table.inner_join(moderation_case::table.on(...)).left_join(community::table.on(...))`. Filter: `jury_assignment::person_id.eq(person_id)`. **Sort by `jury_assignment::selected_at.desc()`** (not `accepted_at` — divergence §1 item 1). Select tuple: `(case_id, severity, reason_code, opened_at, None::<DateTime<Utc>>, community_id, community.name.nullable())`. The `None` for `deadline_at` is expressed as `sql::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>>("NULL")` or — simpler — select the other six columns and build the view via `.map(...)` post-load.
- **DoD:** check + clippy clean.
- **Validation:** log `.claude/build-task-22.log`
- **Commit subject:** `feat(db_views): task 22 — list_jury_assignments_for_person query`
- **Expected iterations:** 2 (join ergonomics + the `None` selection pattern)

### Task 23 — query `list_available_jury_cases_for_person`

- **Files updated:** `crates/db_views/jury_queue/src/impls.rs`
- **Implement:** `pub async fn list_available_jury_cases_for_person(pool: &mut DbPool<'_>, person_id: PersonId) -> LemmyResult<Vec<JuryQueueView>>`. Per [IMPLEMENTATION-PLAN-v0.md §Phase 2] task 23: "v0 stays simple; Phase 5 adds reputation gating." Filter: `moderation_case::status.eq(CaseStatus::ThresholdMet)` AND `moderation_case::target_person_id.ne(Some(person_id))` AND `moderation_case::creator_id.ne(Some(person_id))` AND NOT EXISTS any `jury_assignment` row already linking this person to the case. No reputation check (reputation is a Phase 5 concern). Sort by `moderation_case::opened_at.asc()`.
- **Gotcha — `NOT EXISTS` subquery:** Diesel supports this via `diesel::dsl::not(diesel::dsl::exists(jury_assignment::table.filter(...)))` — verify against modlog idioms if fragile.
- **DoD:** check + clippy clean.
- **Validation:** log `.claude/build-task-23.log`
- **Commit subject:** `feat(db_views): task 23 — list_available_jury_cases_for_person query`
- **Expected iterations:** 2

### Task 24 — query `count_unsubmitted_jury_assignments`

- **Files updated:** `crates/db_views/jury_queue/src/impls.rs`
- **Implement:** `pub async fn count_unsubmitted_jury_assignments(pool: &mut DbPool<'_>) -> LemmyResult<i64>`. Single-table query on `jury_assignment`: `.filter(jury_assignment::status.eq_any([JuryAssignmentStatus::Selected, JuryAssignmentStatus::Accepted]))` then `.count().get_result::<i64>(conn).await?`. **No deadline filter** — per drift §2 item 3, Phase 2a ships the unfiltered count, and Phase 4's background job will add the `selected_at + offset` filter when the homeserver-side decision on deadline source is made.
- **DoD:** check + clippy clean.
- **Validation:** log `.claude/build-task-24.log`
- **Commit subject:** `feat(db_views): task 24 — count_unsubmitted_jury_assignments query`
- **Expected iterations:** 1

---

## 10. Final validation gate (after all 11 task commits land)

Execute on the `feature/phase-2a-governance-case-jury-queue` branch. Not a commit — artifacts go to `.claude/build-phase2a-final-*.log`. All five checks must pass before the advisor handoff.

- [ ] **Gate 1 — clean tree:** `git status` is clean modulo the untracked research PDF (already in `.git/info/exclude` per Phase 1). Any other untracked file halts the gate.
- [ ] **Gate 2 — workspace check:** `scripts/brehon/cargo-check.bat --workspace > .claude/build-phase2a-final-check.log 2>&1 ; echo "exit: $?"` — exit 0. Read the log via `tail -30 .claude/build-phase2a-final-check.log`. **Do NOT dump the full log** per `no-cargo-output-paste.md`.
- [ ] **Gate 3 — e2e round-trip:** `scripts/brehon/cargo-test.bat --test e2e -p lemmy_server > .claude/build-phase2a-final-test.log 2>&1 ; echo "exit: $?"` — exit 0. Expected output: **4 passed, 0 failed** (identical to Phase 1 final count — Phase 2a adds zero e2e tests). If the count is not 4, something is wrong: either a Phase 1 test regressed, or Phase 2a accidentally added a test, or the crate-gating broke one of the existing tests at `crates/server/tests/e2e.rs`.
- [ ] **Gate 4 — per-crate checks:**
    - `cargo check -p lemmy_db_views_governance_case` → exit 0
    - `cargo check -p lemmy_db_views_jury_queue` → exit 0
    (Run via the `cargo-check.bat` wrapper; both pipe to their own log files under `.claude/build-phase2a-gate4-*.log`.)
- [ ] **Gate 5 — commit count + prefix check:** `git log --oneline governance-v0..feature/phase-2a-governance-case-jury-queue | wc -l` → **exactly 11**. Spot-check with `git log --oneline governance-v0..HEAD` that every subject carries `task 14 —` through `task 24 —` in that order, with no gaps and no duplicates.

**If any gate fails, do NOT self-merge. Stop the /prp-ralph loop and hand back to the advisor.**

---

## 11. Acceptance criteria (12-item format mirroring Phase 1)

1. Two new crates exist at `crates/db_views/governance_case/` and `crates/db_views/jury_queue/`, each with `Cargo.toml` + `src/lib.rs` + `src/impls.rs`.
2. `GovernanceCaseSummaryView`, `GovernanceCaseDetailView`, and `JuryQueueView` structs match [04 §4.1] and [04 §4.2] field-for-field (with the three drift items resolved per §2).
3. All seven queries named in [IMPLEMENTATION-PLAN-v0.md §Phase 2 tasks 16–19, 22–24] are implemented as `pub async fn` with the signatures declared in §9.
4. `scripts/brehon/cargo-check.bat --workspace` exits 0 with zero warnings at HEAD of `feature/phase-2a-governance-case-jury-queue`.
5. `scripts/brehon/cargo-test.bat --test e2e -p lemmy_server` exits 0 with 4 passed, 0 failed (unchanged from Phase 1 final count).
6. `cargo clippy --workspace -- -D warnings` exits 0 — no new lints in either Phase 2a crate, no `#[allow]` attributes added.
7. No `.unwrap()`, no `.expect()`, no `#[allow(clippy::...)]` anywhere in the new code (workspace lints deny these).
8. Every task commit carries a subject line of the form `<type>(<scope>): task N — <summary>` where N ∈ {14..24}, in order.
9. Every task commit body names its validation command and its `.claude/build-task-NN.log` path with exit code.
10. Every task commit that touched `Cargo.toml` also staged `Cargo.lock` in the same commit (Phase 1 task-13 footgun averted).
11. `phase1_migrations_round_trip` test still passes — Phase 2a made no changes to `crates/server/tests/e2e.rs`.
12. No contradictions with the 15 ADRs in [99](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). The three drift items in §2 are [04] prose-level, not ADR-level.

---

## 12. Risks and mitigations

| # | Risk | Severity | Likelihood | Mitigation |
|---|---|---|---|---|
| R1 | Diesel `sql::<BigInt>(...)` correlated subquery fails to compile or errors at runtime with type mismatch | MED | MED | Fallback path: load raw columns into a tuple, build view via `.map(...)` in Rust with constants inlined. Decision recorded in task 16 commit body. |
| R2 | `#[diesel(embed)]` on `GovernanceCaseDetailView.case_row: ModerationCase` collides with the non-Queryable `sanctions: Vec<Sanction>` field | MED | MED | Fallback: split `GovernanceCaseDetailView` into `GovernanceCaseDetailRow` (Queryable, all scalar fields) and `GovernanceCaseDetailView` (wrapper with `sanctions`). Record as a post-facto divergence in §1. |
| R3 | Typed ID vs raw `i32` in view struct fields — [04 §4.1] uses `i32` literally, but Phase 1 source structs use `ModerationCaseId(i32)` | LOW | HIGH | Per §6 template, view structs use raw `i32` for id-like fields to match [04 §4.1] literally. The impl functions take typed IDs in their signatures (`CommunityId`, `PersonId`) and convert at the boundary via `.0` unwrapping where needed. Diesel's `DieselNewType` handles the bridging. |
| R4 | `CaseStatus` exhaustive match complaint from clippy if code touches it — Phase 2a doesn't match on `CaseStatus`, but if the compiler widens the Queryable derivation unexpectedly it might | LOW | LOW | Task 15 test-compiles the struct definitions in isolation. If any match appears, cover all variants (`Open`, `ThresholdMet`, `JurySelection`, `InReview`, `Decided`, `Appealed`, `Closed`, `EmergencyRemove`, `AdminReview`). |
| R5 | `jury_assignment` join ergonomics — `joinable!(jury_assignment -> moderation_case)` exists at schema.rs:1236 but the compile-time join shape needs testing | LOW | MED | Task 22 is the first jury-queue query and dedicates 2 iterations to working out the join pattern. If it takes 5+, soft cap kicks in and the advisor is notified. |
| R6 | Cargo.lock staging footgun from Phase 1 task 13 recurring | LOW | MED | Both crate-creation tasks (14, 20) include an explicit `git diff --stat HEAD` check in their DoD that must show `Cargo.lock` present. Halt-and-diagnose if missing. |
| R7 | `/prp-ralph` burning iterations past the 5-per-task soft cap because a later task's design was dependent on an earlier task's undiscovered fragility | LOW | LOW | §9 sequences the tasks so that the two risky ones (16 and 22) come first in their respective crates — if they're broken, the break surfaces before the mechanical variants (18, 19, 23, 24). |
| R8 | Advisor disagrees with §2 drift-stub strategy and wants schema follow-ups instead | LOW | MED | The advisor review gate between plan and `/prp-ralph` handoff is the moment to surface this. §2 explicitly calls out "advisor to decide" on each drift item. |
| R9 | Context budget blowout past ~200k despite the Phase 2a split | LOW | LOW | Phase 2a has 11 tasks, no migrations, no cross-cutting writes, no new tests, and reuses a crate pattern known from Phase 1's reading work. The `no-cargo-output-paste.md` rule loads automatically. Both factors together should keep each task <15k tokens. |
| R10 | `governance-v0` advances beyond `c51147754` between the plan landing and `/prp-ralph` running, putting the branch cut point ahead of the pre-flight check | LOW | LOW | Task 0 verifies the cut-point commit; if it has moved, the ralph loop halts and asks the advisor before proceeding. |

---

## 13. Out of scope (explicit — restated from briefing)

- **No `crates/db_views/governance_modlog` crate** (Phase 2b)
- **No task 30 smoke tests** (Phase 2b, the gate for all three crates together)
- **No `crates/db_views/reputation` crate** (Phase 5)
- **No API layer** — `crates/api/api_common/`, `crates/api/api/`, `crates/api/api_crud/`, `crates/api/routes/` (Phase 3 and Phase 4)
- **No federation code** (Phase 6)
- **No endpoint registration** (Phase 4)
- **No `report_to_modlog_golden_path` test** (Phase 4 task 48)
- **Tasks 14–24 only.** If any task body references a task number outside {14..24}, the plan has misread [IMPLEMENTATION-PLAN-v0.md §Phase 2]. Halt and re-read.
- **No merge into `governance-v0`.** Merge is a separate advisor handoff after the final gate report.

---

## 14. Confidence rationale

**8/10.**

- **+4** — the crate pattern is known verbatim from `modlog` + `report_combined` (both read end-to-end in Phase 2's pre-flight), the Phase 1 schema is intact and correct, all typed IDs and enums exist, and there are no new migrations.
- **+3** — the task decomposition is granular enough that each task compiles in isolation; the risky queries (16, 17, 22, 23) each have a documented fallback path.
- **+1** — Phase 1 shipped cleanly via the same `/prp-ralph` + `cargo-output-capture.md` discipline Phase 2a inherits, and the retro-driven context-budget rules (`no-cargo-output-paste.md`, 11-task split) target the exact failure mode that would otherwise put this over 8.
- **−2** — the two non-trivial unknowns are (a) whether `sql::<BigInt>(...)` correlated subqueries work cleanly in this Diesel version against the `lemmy_db_schema_file` generated schema and (b) whether the `GovernanceCaseDetailView` `#[diesel(embed)] case_row` pattern co-exists with the non-Queryable `sanctions: Vec<Sanction>` field. Both have fallback paths documented in §6 and §12, so the phase still ships if either unknown resolves badly — but either might cost an extra iteration on its task and push a task-17 commit past the 2-iteration estimate.

Not 9 because there are two medium-severity unknowns; not 7 because both are bounded to one task each with concrete fallbacks.

---

## 15. Plan post-write checklist

- [x] §1 Divergences from main plan — 3 items (responded_at vs accepted_at, Vec<View> vs PagedResponse, detail view two-query pattern)
- [x] §2 Divergences from [04] — 3 drift items (reporter_count, jury_needed/submitted, deadline_at)
- [x] §3 Phase 1 follow-ups — 0 items (explicitly stated with Phase 1 state verification evidence)
- [x] §9 Task list — exactly 11 tasks (14–24), one commit per task, no cross-task bundling
- [x] §10 Final validation gate — 5-check format mirroring Phase 1
- [x] §11 Acceptance criteria — 12 items mirroring Phase 1
- [x] §12 Risks — 10 items with severity × likelihood × mitigation
- [x] §13 Out of scope — explicit scope guardrails restated from briefing
- [x] §14 Confidence — 8/10 with detailed justification
