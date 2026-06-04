---
phase: m1-b
role: impl-task
n: 2
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-b
task_number: 2
---

# [role:impl-task] m1-b Task 2 — Diesel model + newtype + schema.rs

## 1. Role + dispatch line

```
[role:impl-task] m1-b task-2 governance_messaging_config model+newtype+schema — see .claude/PRPs/briefs/m1-b-impl-2.md
```

## 2. Scope

Add the Rust data-layer for the `governance_messaging_config` table created in Task 1: the Diesel model + InsertForm, the `MessagingConfigId` newtype, and the `schema.rs` table macro + joinable + allow-tables entry. **This is pre-Shape-G: write a `validate-pending-laptop` DQ entry, commit + push, then STOP. Do NOT run cargo yourself** (cargo runs on the laptop, never on the daemon).

**Produce (exactly 4 files — 1 create, 3 modify):**

```yaml
creates:
  - crates/db_schema/src/source/governance/governance_messaging_config.rs
modifies:
  - crates/db_schema/src/source/governance/mod.rs        # pub mod governance_messaging_config
  - crates/db_schema/src/newtypes.rs                     # MessagingConfigId(pub i32) adjacent to GovernanceConfigId (:299)
  - crates/db_schema_file/src/schema.rs                  # table macro + joinable + allow_tables_to_appear_in_same_query
requires:
  - task: 1   # SATISFIED: migration on phase-m1-b @ 7d51af10c; columns are the contract this model maps
```

**Do NOT:**
- Add a `value_float` column / field anywhere. The M1 migration is **int/bool/text ONLY** — see §4.0. The MIRROR sibling `GovernanceConfig` HAS a `value_float` column; **you must NOT copy that field**. This is the single most important divergence from the sibling.
- Derive `AsChangeset` on the InsertForm (the table is append-only by design — mirror the sibling's deliberate omission + its doc-comment rationale).
- Touch any file outside the 4 listed.
- Run `cargo check` / `cargo clippy` / any cargo or docker command. Validation is the laptop's job (R9 — write-then-stop).
- Commit to `governance-v0` or any branch other than your task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

### 3a. Handover from prior cohort (Task 1, #574)

```yaml
prior_cohort_tasks:
  - task: 1
    commit: 3773cf323
    finalize_merge: 7d51af10c        # daemon-merged into phase-m1-b
    validated: pass                  # migrate-roundtrip.sh exit 0 (advisor-laptop 2026-06-04)
    filesCreated:
      - migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql
      - migrations/2026-06-03-000000-0000_add_governance_messaging_config/down.sql
    keyDecisions:
      - "typed-column config table (int/bool/text); NO value_float; NO JSONB"
      - "UNIQUE INDEX (scope,key,valid_from) — append-history (GOTCHA-50a)"
      - "columns: id SERIAL, scope TEXT, key TEXT, value_type TEXT, value_int BIGINT, value_bool BOOLEAN, value_text TEXT, valid_from TIMESTAMPTZ, updated_by INTEGER REFERENCES person(id)"
    notes: "your model's column set + types MUST match these migration columns exactly — see §4.0"
```

### MIRROR refs (read and mirror shape exactly — but apply the §4.0 column divergence)
- `crates/db_schema/src/source/governance/governance_config.rs` (the full file — model struct + InsertForm + impls). **Mirror its derive stack, the `#[cfg_attr(feature = "full", ...)]` gating, the `#[skip_serializing_none]`, the append-only doc-comment on the InsertForm, and the `create` / `read_current` impl pattern. OMIT `value_float` everywhere.**
- `crates/db_schema/src/newtypes.rs:299` — `GovernanceConfigId(pub i32)`. Place `MessagingConfigId(pub i32)` adjacent (before the "Federation governance typed IDs (Phase 6)" section header) with the IDENTICAL derive block.
- `crates/db_schema_file/src/schema.rs` — `governance_config (id) { ... }` table macro (the sibling block); `joinable!(governance_config -> person (updated_by))` (~:1487); `governance_config,` in `allow_tables_to_appear_in_same_query!` (~:1601).

### Plan sections
- `.claude/PRPs/plans/m1.plan.md` §13 Task 2 (ACTION / IMPLEMENT file-by-file / MIRROR / GOTCHA / VALIDATE), §10.2 (model spec).

### Lessons (mandatory — fired by file-class table)
- `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — **fired by `newtypes.rs` edit.** `MessagingConfigId` lives in `crates/db_schema/src/newtypes.rs` (the `db_schema` crate), NOT in `db_schema_file`. The `schema.rs` table macro lives in `db_schema_file`. Two different crates — get the placement right.
- `.claude/lessons/feedback_features_full_workspace_only.md` — **fired by the `#[cfg(feature = "full")]` gate (GOTCHA).** The model must compile in BOTH feature modes; the Diesel derives (`Identifiable`/`Queryable`/`Selectable`/`Insertable`) + the `use ...schema::governance_messaging_config;` import are `#[cfg_attr(feature = "full", ...)]` / `#[cfg(feature = "full")]` gated exactly like the sibling. The laptop DoD runs `--features full --workspace` (never `-p <crate> --features full`).
- `.claude/lessons/feedback_plan_stub_uniformity_with_canonical_sibling.md` (consult) — the model mirrors a canonical sibling; match its return-type / error-handling / import shape, do not invent a new shape.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry + push, then stop.

> JSONB lesson does NOT apply (typed columns). InsertForm-default-propagation lesson (`feedback_insertform_default_propagation.md`) is informational only — this is a NEW InsertForm with no existing callers to break.

## 4. Constraints

### 4.0 COLUMN CONTRACT (the load-bearing divergence from the sibling)

The model maps the Task-1 migration table EXACTLY. The columns are (verified against `up.sql` on phase-m1-b):

| Column | SQL type | Rust model type |
|---|---|---|
| `id` | `SERIAL` (int4) | `MessagingConfigId` |
| `scope` | `TEXT NOT NULL` | `String` |
| `key` | `TEXT NOT NULL` | `String` |
| `value_type` | `TEXT NOT NULL` | `String` |
| `value_int` | `BIGINT` (nullable) | `Option<i64>` |
| `value_bool` | `BOOLEAN` (nullable) | `Option<bool>` |
| `value_text` | `TEXT` (nullable) | `Option<String>` |
| `valid_from` | `TIMESTAMPTZ NOT NULL` | `DateTime<Utc>` |
| `updated_by` | `INTEGER` (nullable, FK person) | `Option<PersonId>` |

**There is NO `value_float` column.** The sibling `GovernanceConfig` has `value_int`/`value_float`/`value_bool`/`value_text` — this table drops `value_float`. The migration's own up.sql comment states: *"No value_float column (M1 config keys are int/bool/text only…)"*. Mirror everything else about the sibling; remove the `value_float: Option<f64>` field from BOTH the model struct AND the InsertForm.

### 4.1 File 1 of 4 — `crates/db_schema/src/source/governance/governance_messaging_config.rs` (CREATE)

Mirror `governance_config.rs` structurally:
- Imports: `use crate::newtypes::MessagingConfigId;`, `use chrono::{DateTime, Utc};`, `use lemmy_db_schema_file::PersonId;`, `#[cfg(feature = "full")] use lemmy_db_schema_file::schema::governance_messaging_config;`, `use serde::{Deserialize, Serialize};`, `use serde_with::skip_serializing_none;`.
- `#[skip_serializing_none]` + the sibling's full derive stack (`PartialEq, Serialize, Deserialize, Debug, Clone` + `#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]` + `diesel(table_name = governance_messaging_config)` + `diesel(check_for_backend(diesel::pg::Pg))` + the `ts-rs` cfg_attrs).
- `pub struct GovernanceMessagingConfig { ... }` — the 9 fields from §4.0 (id, scope, key, value_type, value_int, value_bool, value_text, valid_from, updated_by). **No value_float.**
- `pub struct GovernanceMessagingConfigInsertForm { ... }` — `#[derive(Clone, Default)]` + `#[cfg_attr(feature = "full", derive(Insertable))]` + `diesel(table_name = governance_messaging_config)`. Fields: scope, key, value_type, value_int, value_bool, value_text, updated_by (no id, no valid_from — DB default; **no value_float**). NO `AsChangeset` (mirror sibling's append-only doc-comment).
- Impls per plan §10.2: a `create` (insert via the form) and a `read_current` that reads the `governance_messaging_config_current` view (mirror the sibling's `read_current` against `governance_config_current`). Match the sibling's conn-type + `LemmyResult<T>` error idiom exactly.

### 4.2 File 2 of 4 — `crates/db_schema/src/source/governance/mod.rs`
Add `pub mod governance_messaging_config;` (alphabetical placement among the existing `pub mod` lines).

### 4.3 File 3 of 4 — `crates/db_schema/src/newtypes.rs`
Add adjacent to `GovernanceConfigId` (~:299), BEFORE the `// === Federation governance typed IDs (Phase 6) ===` section header:
```rust
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
pub struct MessagingConfigId(pub i32);
```

### 4.4 File 4 of 4 — `crates/db_schema_file/src/schema.rs`
Three edits, mirroring `governance_config`:
- **Table macro** `governance_messaging_config (id) { ... }` adjacent to the `governance_config` macro. Columns + diesel types: `id -> Int4, scope -> Text, key -> Text, value_type -> Text, value_int -> Nullable<Int8>, value_bool -> Nullable<Bool>, value_text -> Nullable<Text>, valid_from -> Timestamptz, updated_by -> Nullable<Int4>`. **No `value_float -> Nullable<Float8>`.**
- **Joinable** near :1487: `diesel::joinable!(governance_messaging_config -> person (updated_by));`
- **allow_tables** near :1601: add `governance_messaging_config,` adjacent to `governance_config,` (keep alphabetical).

> Do NOT add a macro/joinable/allow-tables entry for the `governance_messaging_config_current` VIEW. Diesel's `table!` macro is for the base table; the `read_current` impl queries the view via the base table's macro + a `.filter`/raw approach mirroring the sibling's `read_current`. Follow the sibling exactly — if the sibling has a separate view mapping, mirror it; if it reads the view via `sql_query`/a `_current` table macro, mirror that.

### 4.5 validate-pending-laptop DQ entry
After writing the 4 files and committing, append a `validate-pending-laptop` DQ entry. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id (or `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 2,
  "commands": ["./scripts/brehon/cargo-check.sh --workspace --features full"],
  "question": "Workspace check for governance_messaging_config model+newtype+schema — laptop runs cargo.",
  "options": ["pass", "fail"],
  "context": "Task 2 added GovernanceMessagingConfig model + MessagingConfigId newtype + schema.rs table macro/joinable/allow_tables. Mirrors governance_config sibling minus value_float. cargo check --workspace --features full required (model must compile in full mode; the #[cfg(feature=full)] Diesel derives only activate under --features full). Laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

### 4.6 Commit + push discipline
- **Commit subject:** `feat(db_schema): add GovernanceMessagingConfig model + MessagingConfigId newtype + schema (task 2)`
- Sequence:
  ```
  git add crates/db_schema/ crates/db_schema_file/ .claude/decision-queue.json
  git commit -m "feat(db_schema): add GovernanceMessagingConfig model + MessagingConfigId newtype + schema (task 2)"
  git push origin <your worktree branch>
  ```
  (Or two commits — code first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m1-b task 2`.)
- End the commit body with a `LESSON:` trailer if you hit a footgun, and a `HANDOVER:` YAML trailer (see below).
- Then **STOP**. Do not run cargo.

### 4.7 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` and NEVER `approved_by` from this session.

### Mandatory lessons fired for this brief
- `newtypes.rs` edit → `feedback_newtype_locations_lemmy_db_schema_vs_file.md` ✓
- `#[cfg(feature = "full")]` gate → `feedback_features_full_workspace_only.md` ✓ (+ `feedback_features_full_p_crate_incompatible.md` — DoD uses `--workspace`, never `-p`)
- canonical-sibling mirror → `feedback_plan_stub_uniformity_with_canonical_sibling.md` ✓
- validate-pending-laptop → `feedback_validate_pending_laptop_write_then_stop.md` ✓

## HANDOVER (fill in your real values before final commit)

```yaml
HANDOVER:
  task: m1-b-task-2
  filesCreated:
    - crates/db_schema/src/source/governance/governance_messaging_config.rs
  filesModified:
    - crates/db_schema/src/source/governance/mod.rs
    - crates/db_schema/src/newtypes.rs
    - crates/db_schema_file/src/schema.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "GovernanceMessagingConfig model + InsertForm mirror governance_config MINUS value_float"
    - "MessagingConfigId(pub i32) in db_schema::newtypes adjacent to GovernanceConfigId"
    - "schema.rs: table macro + joinable(-> person updated_by) + allow_tables entry; no value_float column"
    - "no AsChangeset (append-only); read_current reads governance_messaging_config_current view"
  notes: "validation = cargo check --workspace --features full on laptop; worker wrote validate-pending-laptop DQ + stopped. Task 3 (DTOs) requires this model's MessagingConfigId + ConfigValueWithProvenance reuse."
```
