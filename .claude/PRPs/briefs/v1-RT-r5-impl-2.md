# Brief: v1-RT-r5 impl-task 2 — rollup DTOs in `api_common/governance.rs`

## 1. Role + dispatch line

`[role:impl-task]` v1-RT-r5 Task 2 — rollup DTOs in api_common/governance.rs

## 2. Scope

Add `AdminReputationRollup` (request DTO) + `AdminReputationRollupResponse` (response DTO)
and the `ReputationSnapshot` import to `crates/api/api_common/src/governance.rs`.
One file, one commit.

**Produce:**
- `use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot;`
  import (currently absent from this file)
- `AdminReputationRollup` request struct with derive stack:
  ```rust
  #[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct AdminReputationRollup { pub person_id: PersonId }
  ```
- `AdminReputationRollupResponse` response struct with derive stack:
  ```rust
  #[skip_serializing_none]
  #[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
  #[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
  #[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
  pub struct AdminReputationRollupResponse {
      pub rollup: Option<ReputationSnapshot>,
      pub contributing: Vec<ReputationSnapshot>,
  }
  ```
- One commit on the task worktree branch
- `validate-pending-laptop` DQ entry with
  `commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`

**Do NOT:**
- Touch any file outside `crates/api/api_common/src/governance.rs`
- Copy `Copy`/`Default`/`Hash` onto the response struct (`ReputationSnapshot` lacks them
  → compile error)
- Add new migrations, new entry-kind consts, new routes
- Commit to `governance-v0` or any branch other than the task worktree branch
- Write `approved_by: "advisor"` in any DQ entry

## 3. Required reading (in order)

Read all of these before the first Edit:

### Schema / type definitions
- `crates/db_schema/src/source/governance/reputation_snapshot.rs:1-47` — `ReputationSnapshot`
  derive stack: `Debug, Queryable, Identifiable, PartialEq, Eq, Serialize, Deserialize,
  Clone, ts_rs::TS` — notably **NO** `Copy`, `Default`, `Hash`. The response DTO contains
  this type so MUST drop those traits.

### MIRROR refs (read and mirror their shape exactly)
- `crates/api/api_common/src/governance.rs:369-380` — `AdminReputationStats` request struct
  (mirror the full derive stack for the request DTO — it has `Copy`/`Hash` because all
  its fields are `Copy`)
- `crates/api/api_common/src/governance.rs:423-430` — `AdminReputationStatsResponse`
  (mirror structure for the response DTO, BUT note difference: that response contains
  `Vec<GovConfig>` and `Vec<GovernanceCaseRow>` — both types have `Copy` + `Hash`. Our
  response contains `ReputationSnapshot` which does NOT → drop `Copy`/`Default`/`Hash`)
- `crates/api/api_common/src/governance.rs:30-40` — imports block at top of Group A;
  the new `ReputationSnapshot` import belongs near the `use lemmy_db_schema::...` cluster

### Lessons (mandatory)
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — no function authored in this
  task, but read for context on the types used downstream
- `.claude/lessons/feedback_features_full_workspace_only.md` — validation command must be
  `--workspace --features full` (not `-p lemmy_api_common --features full`)
- `.claude/lessons/feedback_features_full_p_crate_incompatible.md` — detail

## 4. Constraints

- **Request DTO derive set:** `Copy`/`Hash`/`Default` are all safe because `PersonId`
  is `Copy`. Keep full sibling stack from `AdminReputationStats :369`.

- **Response DTO derive set:** DROP `Copy`/`Default`/`Hash`. Keep `Debug, Serialize,
  Deserialize, Clone, PartialEq, Eq`. Add `#[skip_serializing_none]` (rollup is `Option`).
  Keep the two `cfg_attr` ts-rs lines.

- **Import placement:** place `use lemmy_db_schema::source::governance::reputation_snapshot::ReputationSnapshot;`
  near the top of the file in the `use lemmy_db_schema::...` cluster (around line 4–15).

- **Alphabetical placement of new structs:** place `AdminReputationRollup` and
  `AdminReputationRollupResponse` near `AdminReputationStats` / `AdminReputationStatsResponse`
  (both are in the admin-governance group, currently around line 369+). Alphabetically
  `AdminReputationRollup` sorts before `AdminReputationStats`.

- **DQ mid-task push:** after writing the `validate-pending-laptop` entry, immediately:
  `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised validate-pending-laptop DQ — v1-RT-r5 task 2" && git push origin <branch>`.

- **No `answered_by: "advisor"` from this session.** Use `answered_by: "impl-self-resolved"`
  for self-resolutions.

- **commit subject pattern:** `feat(api-common): add AdminReputationRollup DTOs (task 2)`

### Mandatory lessons fired for this brief
- `api_common/governance.rs` (struct with `#[cfg(feature = "ts-rs")]`):
  `feedback_features_full_workspace_only.md` ✓
- `api_common/governance.rs` (MIRROR ref `AdminReputationStats`):
  `feedback_lemmy_error_no_std_error.md` ✓ (contextual)
