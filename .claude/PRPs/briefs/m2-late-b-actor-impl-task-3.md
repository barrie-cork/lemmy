---
role: impl-task
task_number: 3
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G validate-pending-laptop constraint
  - feedback_newtype_locations_lemmy_db_schema_vs_file.md  # newtype placement
---

# impl-task brief — m2-late-b-actor Task 3: CREATE `actor_app_link.rs` model

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 3 of 13 (`[P]` — cohort-1 alongside Task 2 and Task 4)
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Task 1 (migration). Does NOT depend on Task 2's schema.rs commit (cargo check will resolve the table! from schema.rs when both are present — but if schema.rs hasn't been regenerated yet, use `--features full` which skips the table check until link time).

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-3 model actor-app-link — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-3.md
```

You are the **impl-task** subagent (Sonnet 4.6 — pattern-following from MIRROR refs). Execute exactly what is described here.

---

## 2. Scope

**Produce:**
- `crates/db_schema/src/source/governance/actor_app_link.rs` — `ActorAppLink` struct + `ActorAppLinkInsertForm`
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Edit `mod.rs` (Task 5)
- Add newtypes to `newtypes.rs` (Task 4 — but this file IMPORTS `ActorAppLinkId` which Task 4 will create; use it as if it exists)
- Run cargo yourself

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 3" + §"Patterns to Mirror / DIESEL_MODEL"
2. `crates/db_schema/src/source/governance/endorsement.rs` — **PRIMARY MIRROR**: verbatim copy-and-adapt; `revoked_at: Option<DateTime<Utc>>` as last field; `#[skip_serializing_none]`; full derive stack
3. `crates/db_schema/src/source/governance/actor_pseudonym.rs` — FK target; `ActorPseudonymId` newtype; `pseudonym TEXT` convention
4. `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — **MANDATORY**: newtypes live in `db_schema::newtypes`, not `db_schema_file`
5. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**: pre-Shape-G stop after DQ write

---

## 4. Constraints

### Implementation — copy `endorsement.rs` verbatim, swap field names

**File:** `crates/db_schema/src/source/governance/actor_app_link.rs`

```rust
use crate::newtypes::ActorAppLinkId;
use crate::newtypes::ActorPseudonymId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::actor_app_link;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]
#[cfg_attr(feature = "full", diesel(table_name = actor_app_link))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A portable Brehon actor ↔ external-app-user mapping (B-actor, ADR-016).
/// `brehon_actor_id` references actor_pseudonym(id); never stores raw person_id (ADR-015).
pub struct ActorAppLink {
  pub id: ActorAppLinkId,
  pub brehon_actor_id: ActorPseudonymId,
  pub app_id: String,
  pub app_local_id: String,
  pub created_at: DateTime<Utc>,
  pub revoked_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = actor_app_link))]
pub struct ActorAppLinkInsertForm {
  pub brehon_actor_id: ActorPseudonymId,
  pub app_id: String,
  pub app_local_id: String,
}
```

**GOTCHA — derive order:**
- Match `endorsement.rs` derive order exactly: `PartialEq, Eq, Serialize, Deserialize, Debug, Clone`
- `#[skip_serializing_none]` MUST be present (because `revoked_at` is `Option`)
- `Identifiable, Queryable, Selectable` under `#[cfg(feature = "full")]`

**GOTCHA — table import:**
- `use lemmy_db_schema_file::schema::actor_app_link;` gated under `#[cfg(feature = "full")]`
- This is `lemmy_db_schema_file` (the generated schema crate), NOT `lemmy_db_schema`

**GOTCHA — field order:**
- `revoked_at: Option<DateTime<Utc>>` MUST be the last field (matches DB column order; Diesel Queryable is order-sensitive)

**GOTCHA — InsertForm:**
- Omit `id`, `created_at`, `revoked_at` — these have DB defaults or are set via UPDATE
- `app_id: String` and `app_local_id: String` are owned strings (no lifetime)

**GOTCHA — ADR-015:**
- `brehon_actor_id: ActorPseudonymId` is a FK integer to `actor_pseudonym(id)` — this is the pseudonym table reference, NOT a raw `person_id`. The pseudonym UUID string is stored in the `actor_pseudonym.pseudonym` column; this integer FK is the internal join key. NEVER add a `person_id` field here.

### validate-pending-laptop (pre-Shape-G, MANDATORY)

After committing the model file:

1. Write a `kind: "validate-pending-laptop"` DQ entry:
   ```json
   {
     "commands": [
       "cargo check -p lemmy_db_schema"
     ],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 3
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id.
   Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append.

2. **Commit** DQ: `git commit -m "chore(decision-queue): impl raised validate-pending-laptop task-3 model"`

3. **Push**: `git push origin phase-m2-late-b-actor`

4. **STOP.**

---

## 5. Commit

```
feat(db_schema): add ActorAppLink model + InsertForm (B-actor ADR-016 task 3)
```

Stage only: `crates/db_schema/src/source/governance/actor_app_link.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `crates/db_schema/src/source/governance/actor_app_link.rs` created with exact struct shown
- [ ] `#[skip_serializing_none]` present
- [ ] `revoked_at: Option<DateTime<Utc>>` is last field
- [ ] `ActorAppLinkInsertForm` omits `id`/`created_at`/`revoked_at`
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-3
  branch: phase-m2-late-b-actor
  filesCreated:
    - crates/db_schema/src/source/governance/actor_app_link.rs
  filesModified:
    - .claude/decision-queue.json
  keyDecisions:
    - "Model mirrors endorsement.rs verbatim — revoked_at as last Option field"
    - "brehon_actor_id: ActorPseudonymId (FK to actor_pseudonym, not raw person_id)"
    - "ActorAppLinkId imported from crate::newtypes (added by Task 4)"
  notes: "Task 3 of 13. cargo check will resolve ActorAppLinkId once Task 4 merges."
```
