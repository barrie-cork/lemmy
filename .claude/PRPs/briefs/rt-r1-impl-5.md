---
phase: v1-RT-r1
role: impl-task
task: 5
brief_n: 5
authored: 2026-05-11
parallel_cohort: B-serial (no [P])
---

# [role:impl-task] v1-RT-r1 task 5 add ReputationEventSourceType Rust enum — see .claude/PRPs/briefs/rt-r1-impl-5.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 task 5 add ReputationEventSourceType enum (9 variants)`

## §2 Scope

UPDATE `crates/db_schema_file/src/enums.rs` — append new `ReputationEventSourceType` enum AFTER the existing `ReputationDimension` declaration (line 615). Plan §10.5 skeleton **verbatim**:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::ReputationEventSourceType"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Per-event source classification for `reputation_event` rows. Per
/// PRD section 5.3 + 7. v0 rows receive Endorsement via column DEFAULT;
/// 2026-05-10-000200-0000 backfill revises by reason ILIKE per DQ #184.
/// r3 emitters write the matching variant explicitly going forward.
pub enum ReputationEventSourceType {
  #[default]
  Endorsement,
  JuryVote,
  SponsorLiability,
  FounderSeed,
  ParticipationCron,
  DormancyCron,
  VoteOutcome,
  EvidenceQuality,
  ManualSeed,
}
```

**No other file edits.** `schema.rs` extension is Task 6; Diesel struct extension is Task 7.

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.5 (skeleton verbatim) + §13 Task 5 (FILES YAML, GOTCHAs, MIRROR)
- `crates/db_schema_file/src/enums.rs:598-615` (mirror primary — `ReputationDimension` enum)
- `crates/db_schema_file/src/enums.rs:327-335` (mirror reference — `NotificationType` enum for variant-ordering convention)
- **Do NOT mirror `MembershipState`** (per plan §13 Task 5 explicit guidance)
- `.claude/lessons/feedback_clippy_test_style.md` — `LemmyResult<()>` not required for non-test code; verify
- `.claude/rules/decision-queue.md` — Recipe 1 (validate-pending DQ on push)

## §3a Handover from prior cohort (Cohort A)

Cohort A (Tasks 1-4) complete on `phase-v1-RT-r1` at tip `58e887d64`. All 4 SQL migrations shipped:
- `2026-05-10-000000-0000_add_reputation_event_v1_columns/` — Task 1 (creates `reputation_event_source_type` pg enum type with 9 variants matching this Rust enum's 9 variants)
- `2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/` — Task 2
- `2026-05-10-000200-0000_backfill_reputation_event_source_type/` — Task 3
- `2026-05-10-000300-0000_seed_v1_rt_config_keys/` — Task 4 (26 net-new keys)

Bundled migration validation: `gh run view 25640324089: success` on `junior/revalidate-rt-r1-bundle`.

**Task 5 reads Task 1's enum DDL** — the Rust enum's 9 variants must match the pg enum's 9 variants in NAME + ORDER. Postgres `CREATE TYPE` (Task 1's `up.sql`) is the source of truth:
```sql
CREATE TYPE reputation_event_source_type AS ENUM (
    'Endorsement', 'JuryVote', 'SponsorLiability', 'FounderSeed',
    'ParticipationCron', 'DormancyCron', 'VoteOutcome', 'EvidenceQuality', 'ManualSeed'
);
```
Rust enum mirrors PascalCase verbatim per `DbValueStyle = "verbatim"`.

## §4 Constraints

- **Cohort B barrier: Sequential ordering required.** Task 5 must complete (workspace-check pass) BEFORE Task 6 dispatches (Task 6 references `crate::schema::sql_types::ReputationEventSourceType` which Task 6's schema.rs edit creates, but Task 6 ALSO references this Rust enum via the Diesel `DbEnum` derive — circular-but-resolvable at the schema.rs-additions-merged stage).
- **#[default] MUST be Endorsement.** Must match Postgres column DEFAULT `'Endorsement'` (Task 1). Drift breaks runtime parsing.
- **PascalCase variants 1:1 with Task 1 pg enum.** `DbValueStyle = "verbatim"` enforces this; mismatched names would error at compile via `DbEnum` macro.
- **Existing imports cover all derives.** Do NOT add new `use` statements (per §10.5 GOTCHA).
- **No new newtype.** Task 5 is enum-only; no struct.
- **DQ mid-task push rule:** Shape G — 1 `kind: "validate-pending"` DQ entry on push (only `cargo-validate-workspace.yml` fires; `cargo-validate-migration.yml` does NOT fire for crates/** path changes). Per plan §13 Task 5 "Push and exit (Shape G)".
- **Attribution:** `from: "impl"`, `answered_by: null`.

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 5 is Rust enum addition only. No test code; no `LemmyResult` / `Box<dyn Error>` decision.

## §5 Concurrency note

Cohort B-serial (no `[P]` marker). Task 5 dispatched alone. After Task 5 done + workspace-check pass, advisor authors + dispatches Task 6 alone. Then Task 7 alone. Then parallel cohort Tasks 8+9+10.

**Forward-only lesson from Cohort A:** parallel ci-watcher dispatch off same phase tip caused merge-time resurrection of resolved DQ entries. Serial dispatch + serial ci-watcher avoids that race. The Cohort A SQL-migration `[P]` markers were correct at the worktree-write-disjointness level but the implicit data-dependency caused Task 3's per-task workflow to fail. Plan §13 explicitly marks Tasks 5/6/7 as "Cohort B barrier. Sequential ordering required." — those markers are load-bearing; Junior must respect.

## §6 COMMIT MESSAGE

`feat(v1-RT-r1): add ReputationEventSourceType Rust enum — 9 variants matching reputation_event_source_type pg_type (task 5)`

Add HANDOVER YAML trailer with `filesCreated`, `filesModified`, `keyDecisions`, `notes` per template.
