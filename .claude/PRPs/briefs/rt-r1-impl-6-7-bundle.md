---
phase: v1-RT-r1
role: impl-task
task: 6-7-bundle
brief_n: 6-7
authored: 2026-05-11
parallel_cohort: B-bundle (consolidates plan §13 Tasks 6+7)
---

# [role:impl-task] v1-RT-r1 tasks 6+7 BUNDLED — schema.rs + Diesel struct extensions — see .claude/PRPs/briefs/rt-r1-impl-6-7-bundle.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 tasks 6+7 bundled — schema.rs extensions + Diesel struct extensions in one commit`

## §2 Scope — BUNDLED (Tasks 6 + 7 in one Junior task, one commit)

**Why bundled (not separate):** Task 5 (`ReputationEventSourceType` enum, already shipped at commit `5142dba54` on phase-v1-RT-r1) FAILED its isolation `cargo-validate-workspace.yml` workflow (run 25668340599) because the enum references `crate::schema::sql_types::ReputationEventSourceType` — a path Task 6 creates. Same isolation-validation bug class as Cohort A Task 3 DQ #194. Plan §13's "Cohort B barrier. Sequential ordering required." marker was intended for commit-order serialization but doesn't address the cross-task validation dependency. Bundling Tasks 6+7 into one commit means the bundled push validates the COMPLETE `lemmy_db_schema_file` + `lemmy_db_schema` crate state (Task 5's enum + Task 6's schema.rs sql_types + Task 7's struct extensions all together) cleanly.

### §2.1 Sub-edit 1: `crates/db_schema_file/src/schema.rs` (plan §10.6)

THREE sub-edits in this file:

**(a) Add sql_types::ReputationEventSourceType after ReputationDimension (~line 118):**
```rust
  #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
  #[diesel(postgres_type(name = "reputation_event_source_type"))]
  pub struct ReputationEventSourceType;
```

**(b) Modify `reputation_event` table! block (line 1218-1229):**
- Add `use super::sql_types::ReputationEventSourceType;` at top of the `diesel::table!` block (alongside existing `use super::sql_types::ReputationDimension;`)
- Append two new fields at bottom of column list with `// v1-RT-r1 additions:` comment:
  - `dedupe_key -> Nullable<Text>,`
  - `source_event_type -> ReputationEventSourceType,`

**(c) Modify `sponsor_allowlist` table! block (line 1338-1343):**
- Change `community_id -> Int4,` to `community_id -> Nullable<Int4>,    // v1-RT-r1: was Int4 (NOT NULL); now nullable`
- Append two new fields with `// v1-RT-r1 additions:` comment:
  - `added_by_admin_id -> Int4,`
  - `note -> Nullable<Text>,`

### §2.2 Sub-edit 2: `crates/db_schema/src/source/governance/reputation_event.rs` (plan §10.7)

Per §10.7 verbatim:
- Modify the `use lemmy_db_schema_file::{...}` import to add `ReputationEventSourceType` alongside `ReputationDimension`
- Add 2 new fields to `ReputationEvent` struct: `dedupe_key: Option<String>` + `source_event_type: ReputationEventSourceType`
- Add 2 new fields to `ReputationEventInsertForm`: `dedupe_key: Option<String>` + `source_event_type: Option<ReputationEventSourceType>` (Option per `feedback_insertform_default_propagation.md` — NOT-NULL column DEFAULT covers callers that omit)

### §2.3 Sub-edit 3: `crates/db_schema/src/source/governance/sponsor_allowlist.rs` (plan §10.7)

Per §10.7 verbatim:
- Modify `SponsorAllowlist` struct: change `community_id: CommunityId` to `community_id: Option<CommunityId>`; add `added_by_admin_id: PersonId` + `note: Option<String>` fields
- Modify `SponsorAllowlistInsertForm`: change `community_id: CommunityId` to `community_id: Option<CommunityId>`; add `added_by_admin_id: PersonId` (REQUIRED, NOT Option) + `note: Option<String>` fields
- Pre-verify zero external callers of `SponsorAllowlistInsertForm`: `rg "SponsorAllowlistInsertForm" crates/` should return only references within `sponsor_allowlist.rs` itself. If any caller exists outside the file, file a DQ pending entry and STOP.

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.6 (Task 6 skeleton verbatim — 3 sub-edits in schema.rs)
- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.7 (Task 7 skeleton verbatim — 2 sub-edits in 2 files)
- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §13 Task 6 + Task 7 (FILES YAML, GOTCHAs, MIRROR)
- `crates/db_schema_file/src/schema.rs:118` (sql_types module — mirror for sub-edit (a))
- `crates/db_schema_file/src/schema.rs:1218-1229` (reputation_event table! — mirror for sub-edit (b))
- `crates/db_schema_file/src/schema.rs:1338-1343` (sponsor_allowlist table! — mirror for sub-edit (c))
- `crates/db_schema/src/source/governance/reputation_event.rs:1-42` (mirror for §2.2)
- `crates/db_schema/src/source/governance/sponsor_allowlist.rs:1-37` (mirror for §2.3)
- `.claude/lessons/feedback_insertform_default_propagation.md` — Option<> on insert form when column has DEFAULT
- `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — newtype crate placement convention (no new newtype here; SponsorAllowlistId + ReputationEventId already exist)
- `.claude/rules/decision-queue.md` — Recipe 1 (validate-pending DQ on push)

## §3a Handover from prior cohort

**Task 5 (#223) commit `5142dba54` ALREADY on phase-v1-RT-r1.** That commit added `ReputationEventSourceType` Rust enum to `crates/db_schema_file/src/enums.rs` — 9 variants matching Task 1's pg enum DDL. The enum's `ExistingTypePath = "crate::schema::sql_types::ReputationEventSourceType"` references a sql_types path that THIS bundle creates in §2.1(a).

Phase tip: `9f0e2c36f` (merge reconcile commit). Worker branch forks from there.

Cohort A complete (Tasks 1-4 + bundled re-validate). Cohort B-serial: Task 5 partially complete (commit landed but workflow failed in isolation). This bundle resolves the cross-task dependency.

## §4 Constraints

- **Bundled atomicity:** all 3 files modified in ONE commit. Junior MUST NOT split into multiple commits — the validation depends on all 3 files being present simultaneously.
- **`#[diesel(postgres_type(name = "reputation_event_source_type"))]`** uses the EXACT lowercase pg enum type name from Task 1's `CREATE TYPE` (`reputation_event_source_type`). DO NOT camelCase the `name` value.
- **Hand-edit `@generated` `schema.rs`** per governance convention (plan §10.6 GOTCHA). Document with inline `// v1-RT-r1 additions:` and `// v1-RT-r1: was Int4 (NOT NULL); now nullable` comments.
- **`joinable!` block at `schema.rs:1466-1482` is NOT edited.** Adding `joinable!(sponsor_allowlist -> person (added_by_admin_id))` would conflict with existing `sponsor_allowlist -> person (person_id)`.
- **`ReputationEventInsertForm.source_event_type: Option<>`** (NOT-NULL column DEFAULT 'Endorsement' covers callers that omit). Per `feedback_insertform_default_propagation.md`.
- **`ReputationEventInsertForm.dedupe_key: Option<String>`** (nullable column).
- **`SponsorAllowlistInsertForm.added_by_admin_id: PersonId`** REQUIRED (NOT Option). v1-AD-a-shipped table is empty; r4 endpoints set explicitly.
- **`AsChangeset` derive:** KEEP on `ReputationEventInsertForm`; DO NOT add to `SponsorAllowlistInsertForm`.
- **DQ mid-task push rule:** Shape G — 1 `kind: "validate-pending"` DQ entry on push for `cargo-validate-workspace.yml` workflow. NO migration workflow fires (no migrations/** path change).
- **Attribution:** `from: "impl"`, `answered_by: null`.

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Tasks 6 + 7 are non-test code (schema.rs + Diesel structs). No `LemmyResult` / `Box<dyn Error>` decision.

## §5 Concurrency note

Single Junior task (no parallel). After this bundle's workspace-check passes, the next stage is the parallel Cohort B-final: Tasks 8 + 9 + 10 (all `[P]` per plan §13). Tasks 8/9/10 do NOT have cross-task validation dependencies — Task 8 modifies config.rs, Task 9 modifies governance_log.rs (db_schema + api) + registry.md, Task 10 modifies e2e.rs. Each can isolate-validate independently.

**Forward-only lesson capture for retro Task 11:** plan §13 `[P]` and "Cohort B barrier. Sequential ordering required." markers conflate worktree-write disjointness with validation-time independence. Future plans should require an explicit `bundled: true` marker for tasks whose changes only collectively compile, OR a `validates_with: [N, M]` field indicating tasks whose validation must run jointly.

## §6 COMMIT MESSAGE

`feat(v1-RT-r1): bundled tasks 6+7 — schema.rs sql_types + table! extensions + Diesel struct extensions (resolves Task 5 isolation FAIL)`

Add HANDOVER YAML trailer with `filesCreated`, `filesModified`, `keyDecisions`, `notes` per template. `keyDecisions` MUST cite the Cohort A/B isolation-validation bug class as the rationale for bundling.
