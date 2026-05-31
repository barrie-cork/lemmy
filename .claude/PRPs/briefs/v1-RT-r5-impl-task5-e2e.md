# Brief: v1-RT-r5 Task 5 — e2e coverage for reputation rollup

## 1. Role + dispatch line

`[role:impl-task] v1-RT-r5 task-5 e2e — see .claude/PRPs/briefs/v1-RT-r5-impl-task5-e2e.md`

## 2. Scope

Add one new module `mod v1_rt_r5_fixtures` at the **end** of
`crates/server/tests/e2e.rs`, after the closing `}` of `mod v1_rt_r4_fixtures`
(line 18423 at brief-authoring time — verify the actual line with
`grep -n 'mod v1_rt_r4_fixtures' crates/server/tests/e2e.rs` before editing).

Add exactly **4 `#[tokio::test(flavor = "multi_thread")]` tests** inside that module:

| Test function name | Story |
|---|---|
| `rollup_cron_materialises_instance_wide_mean` | Story 1 |
| `rollup_excludes_banned_communities` | Story 2 |
| `admin_reputation_rollup_endpoint` | Story 3 |
| `rollup_emits_governance_log_entry` | Story 4 |

**Do NOT touch any other file.** All runtime is in Tasks 1–4.

**Commit once** after all 4 tests pass the `--no-run` compile gate: single
commit with subject `feat(e2e): Task 5 — reputation rollup e2e tests (v1-RT-r5)`.

**Write `validate-pending-laptop-e2e`** DQ entry with
`commands: ["./scripts/brehon/cargo-test.sh --workspace --features full --test e2e"]`
plus a `--no-run` compile gate first; commit + push; then **stop**.
Do NOT run cargo-test yourself — validation is delegated to the laptop advisor.

## 3. Required reading

**Mandatory file-class lessons** (all apply):

- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — Case A (uniform
  `LemmyResult<()>` throughout, bare `?`). The sibling `v1_rt_r3_fixtures`
  uses Case A; mirror it verbatim.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — `AsyncPgConnection`
  + `DbPool::Conn` pattern for direct DB queries in e2e.
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate
  every `old_string` anchor before editing; confirm `grep -c '<anchor>'
  crates/server/tests/e2e.rs` == 1 each.
- `.claude/lessons/feedback_features_full_workspace_only.md` — always
  `--workspace --features full`.
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — never
  `-p lemmy_server --features full`.
- `.claude/lessons/feedback_clippy_test_style.md` — `LemmyResult<()>`,
  no `unwrap`/`expect`.

**Canonical mirror fixture** (read before writing a single line of test):

- `crates/server/tests/e2e.rs:17150-17229` — `mod v1_rt_r3_fixtures` header
  (imports, module doc, helper shapes). This is the CANONICAL sibling.
- `crates/server/tests/e2e.rs:17592-17646` — `participation_activity_cron_emits_plus_one_per_active_user`
  (mirror for `boot_context`, `EnvVarGuard::set`, `seed_named_community`,
  cron-call pattern, governance_log count assertion, ISO-week xfail guard).

**Implementation files produced by Tasks 1–4** (read to understand call
signatures — do NOT modify):

- `crates/api/api/src/governance/reputation_snapshot.rs` lines ~136–165
  (`RollupBatchOutcome`) and ~564+ (`compute_rollup_snapshot`,
  `run_rollup_batch`, `load_rollup_candidates`).
- `crates/api/api/src/governance/admin_reputation_rollup.rs` — full file
  (handler signature; call directly, not via HTTP).
- `crates/api/api_common/src/governance.rs` — `AdminReputationRollup` +
  `AdminReputationRollupResponse` structs.
- `crates/db_schema/src/source/governance/reputation_snapshot.rs` —
  `ReputationSnapshotInsertForm` (all fields; seed per-community rows via
  `diesel::insert_into(reputation_snapshot::table).values(&ReputationSnapshotInsertForm{...}).execute(&mut conn)`).
- `crates/db_schema/src/source/governance/sanction.rs` — `SanctionInsertForm`
  (seed a community-scoped active sanction for Story 2 banned exclusion;
  requires a `ModerationCaseId` — seed a bare `moderation_case` row first).

## 4. Implementation

### 4.0 Pre-locate anchors

Before opening any Edit tool call, run:
```bash
grep -c '^}$' crates/server/tests/e2e.rs
```
The insertion point is a single `}` on its own line at the end of the file
(the `mod v1_rt_r4_fixtures` closing brace). The anchor for the Edit must
be **the final closing brace + EOF** — use the last N lines of the file as
the `old_string` to ensure uniqueness. Example:
```bash
tail -5 crates/server/tests/e2e.rs
```
Confirm the chosen `old_string` appears exactly once with `grep -c`.

### 4.1 Module header

```rust
mod v1_rt_r5_fixtures {
  //! v1-RT-r5 instance-wide reputation rollup cron + admin endpoint e2e.
  //!
  //! 4 stories / 4 tests per plan §16a:
  //!   Story 1 — weekly cron materialises integer-mean rollup row (1 test)
  //!   Story 2 — banned communities excluded from rollup (1 test)
  //!   Story 3 — admin endpoint returns rollup + contributing; rejects non-admin (1 test)
  //!   Story 4 — ROLLUP_RECOMPUTED governance-log entry emitted (1 test)
  //!
  //! Case A error shape: outer `LemmyResult<()>` + helpers `LemmyResult<T>`.
  //! Mirror: v1_rt_r3_fixtures (e2e.rs:17150) for boot + helper shape.

  use super::*;
  use super::EnvVarGuard;
  use actix_web::web::{Data, Json, Query};
  use chrono::Utc;
  use diesel::{ExpressionMethods, OptionalExtension, QueryDsl};
  use diesel_async::{AsyncConnection, AsyncPgConnection, RunQueryDsl};
  use lemmy_api::governance::{
    admin_reputation_rollup::admin_reputation_rollup,
    reputation_snapshot::run_rollup_batch,
  };
  use lemmy_api_common::governance::{AdminReputationRollup, AdminReputationRollupResponse};
  use lemmy_api_utils::context::LemmyContext;
  use lemmy_db_schema::source::{
    governance::{
      moderation_case::ModerationCaseInsertForm,
      reputation_snapshot::ReputationSnapshotInsertForm,
      sanction::SanctionInsertForm,
    },
    instance::Instance,
    local_user::{LocalUser, LocalUserInsertForm},
    person::{Person, PersonInsertForm},
  };
  use lemmy_db_schema_file::{
    InstanceId, PersonId,
    enums::{SanctionAction, SanctionScope},
    schema::{governance_log, moderation_case, reputation_snapshot as rs_table, sanction},
  };
  use lemmy_db_views_local_user::LocalUserView;
  use lemmy_diesel_utils::traits::Crud;
  use lemmy_utils::error::LemmyResult;
```

Adjust imports as needed based on what the compiler requires — add anything
missing, remove unused imports. Follow the `use diesel_async::RunQueryDsl`
pattern from the sibling for `.execute()` and `.get_result()`.

### 4.2 `boot_context` reuse

Reuse `v1_rt_r3_fixtures::boot_context` via `super::v1_rt_r3_fixtures` is
NOT available (submodules are siblings, not hierarchical). Instead, define a
thin local wrapper that calls the `v1_rt_r3_fixtures` local `boot_context`
function — but that function is private to its module. The right pattern is
to use `governance_fixtures::bootstrap()` (the top-level helper used by RT-r4
fixtures) which returns `(container, context, db_url)`, OR replicate the
`boot_context` pattern inline using `governance_fixtures::start_postgres()` +
`governance_fixtures::apply_all_schema()` per the RT-r3 fixture body at lines
17437-17500.

**Recommended:** use `governance_fixtures::bootstrap()` as RT-r4 does (3-tuple
return, already returns a seeded DB). If `bootstrap()` doesn't initialise
default governance config knobs needed for capability thresholds, fall back to
the full `boot_context` replication. Read both RT-r3 and RT-r4 boot helpers
before deciding.

**Instance name discipline:** every test MUST seed its own `Instance` with a
unique hostname (e.g., `v1_rt_r5_s1.example.com`, `v1_rt_r5_s2.example.com`,
etc.). Do NOT use `test.invalid` shared between tests — that collides.

### 4.3 Story 1: `rollup_cron_materialises_instance_wide_mean`

Seed 2 communities. Seed per-community `reputation_snapshot` rows for one
person:
- Community A: `reporting_accuracy=10, jury_reliability=20, participation_consistency=30, endorsement_strength=40`
- Community B: `reporting_accuracy=30, jury_reliability=40, participation_consistency=50, endorsement_strength=60`

Call `run_rollup_batch(&context).await?`. Assert the `community_id IS NULL`
row exists with integer means:
- `reporting_accuracy = (10+30)/2 = 20`
- `jury_reliability = (20+40)/2 = 30`
- `participation_consistency = (30+50)/2 = 40`
- `endorsement_strength = (40+60)/2 = 50`

The integer truncation here divides evenly; this avoids any ambiguity about
floor vs round.

Seed per-community rows via `diesel::insert_into(rs_table::table).values(&ReputationSnapshotInsertForm{...}).execute(&mut async_conn).await?`.

Assert the rollup row via a Diesel query:
```rust
let rollup: Option<reputation_snapshot::ReputationSnapshot_type_here> = rs_table::table
  .filter(rs_table::person_id.eq(person_id))
  .filter(rs_table::community_id.is_null())
  .first(...)
  .await
  .optional()?;
assert!(rollup.is_some(), "rollup row must exist after cron");
let r = rollup.unwrap();
assert_eq!(r.reporting_accuracy, 20, "mean of [10,30]");
// ... etc.
```

Use the actual Diesel type — `ReputationSnapshot` from
`lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot`.

### 4.4 Story 2: `rollup_excludes_banned_communities`

Seed 2 communities. Seed per-community snapshots for one person:
- Community A: `reporting_accuracy=10, jury_reliability=10, participation_consistency=10, endorsement_strength=10`
- Community B: `reporting_accuracy=50, jury_reliability=50, participation_consistency=50, endorsement_strength=50`

Seed an active community-scoped `sanction` against the person for Community B
(making them "banned" from it). Requires inserting a `moderation_case` row
first to satisfy the FK:
```rust
// Insert a bare case to satisfy sanction FK.
let case: moderation_case::SomeType = diesel::insert_into(moderation_case::table)
  .values(&ModerationCaseInsertForm { ... })
  .get_result(&mut async_conn).await?;

diesel::insert_into(sanction::table)
  .values(&SanctionInsertForm {
    case_id: case.id,
    scope: SanctionScope::Community,
    action: SanctionAction::Suspend,
    target_person_id: Some(person_id),
    target_community_id: Some(community_b.id),
    active: Some(true),
    ..Default::default()
  })
  .execute(&mut async_conn).await?;
```

Call `run_rollup_batch(&context).await?`. Assert:
- The `community_id IS NULL` row's dimensions equal Community A's values
  (denominator=1, mean of [10] = 10).
- `outcome.rows_written == 1`.

Check `ModerationCaseInsertForm`'s required fields from the source struct
before writing the seed — read the `ModerationCaseInsertForm` definition to
see which fields have defaults and which don't.

### 4.5 Story 3: `admin_reputation_rollup_endpoint`

After calling `run_rollup_batch` (which seeds the rollup row), call the
handler directly (not via HTTP, same pattern as RT-r4's
`admin_sponsor_allowlist::add`):

```rust
let resp = admin_reputation_rollup(
  Query(AdminReputationRollup { person_id }),
  context.clone(),
  admin_view,
)
.await?
.into_inner();
assert!(resp.rollup.is_some(), "admin sees rollup row");
assert!(!resp.contributing.is_empty(), "contributing non-empty");
```

Also test the non-admin rejection: call with a non-admin `LocalUserView` and
assert `result.is_err()`.

### 4.6 Story 4: `rollup_emits_governance_log_entry`

After calling `run_rollup_batch(&context).await?`, query:
```rust
let count: i64 = governance_log::table
  .filter(governance_log::entry_kind.eq("rollup_recomputed"))
  .count()
  .get_result(&mut async_conn).await?;
assert_eq!(count, 1, "exactly one rollup_recomputed governance log entry");
```

Optionally verify the payload fields (`person_id`, `contributing_community_count`,
`rollup_dimensions`, `recomputed_at`) by selecting `governance_log::payload`
and parsing the JSON — but a simple count assertion is sufficient for the
story gate.

### 4.7 Validate-pending-laptop-e2e DQ entry

After committing the tests, write a DQ entry:

```json
{
  "kind": "validate-pending-laptop-e2e",
  "from": "impl",
  "phase_task": "5",
  "branch": "phase-v1-RT-r5",
  "commands": [
    "./scripts/brehon/cargo-test.sh --no-run -p lemmy_server --test e2e",
    "./scripts/brehon/cargo-test.sh --workspace --features full --test e2e"
  ]
}
```

Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`. Use
`bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to
append safely. Commit + push.

## 5. Constraints

- **Case A error shape everywhere:** `LemmyResult<()>` + bare `?`, no
  `Box<dyn Error>`.
- **Do NOT run `cargo-test.sh` yourself** — write the DQ entry, commit, push, STOP.
- **No edits outside `crates/server/tests/e2e.rs`** — all impl is in Tasks 1–4.
- **Unique instance hostnames** per test to avoid cross-test DB collision.
- **EnvVarGuard::set("BREHON_DISABLE_ROLLUP_JOB", "1")** in Story 1 (prevent
  the actual cron from firing concurrently during the test run; you control
  timing by calling `run_rollup_batch` directly).
- **Integer-division means in assertions must match the helper exactly.** All
  test seeds must use values that divide evenly OR explicitly account for
  truncation. Recommended: use values divisible by the denominator to avoid
  ambiguity in the brief.
- **Mandatory lessons fired:**
  - `feedback_lemmy_error_no_std_error.md` — file-class match: e2e.rs edit
  - `feedback_async_pool_test_pattern.md` — file-class match: e2e.rs edit
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` — file-class match: ≥2 e2e edits
  - `feedback_features_full_workspace_only.md` — DQ validation commands
  - `feedback_features_full_p_crate_incompatible.md` — DQ validation commands
  - `feedback_clippy_test_style.md` — LemmyResult<()> test style
