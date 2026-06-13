---
role: impl-task
task_number: 2
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_diesel_print_schema_auto_applies_patch.md  # schema.rs regen via diesel print-schema + patch_file
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G validate-pending-laptop constraint
---

# impl-task brief — m2-late-b-actor Task 2: REGEN `schema.rs` via diesel print-schema

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 2 of 13 (`[P]` — first in cohort-1 along with Tasks 3+4, but Tasks 3+4 can run in parallel with this one since they are pure file-creates with no schema dependency until cargo check)
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Task 1 (migration must have merged + `validate-pending-laptop` must have passed)

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-2 schema-regen actor-app-link — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-2.md
```

You are the **impl-task** subagent (Sonnet 4.6 — pattern-following from MIRROR refs). Execute exactly what is described here. No impl beyond what is listed.

---

## 2. Scope

**Produce:**
- Updated `crates/db_schema_file/src/schema.rs` with the new `actor_app_link` table! block, `joinable!` entry, and `allow_tables_to_appear_in_same_query!` entry
- `validate-pending-laptop` DQ entry (commit + push on the phase branch)

**Do NOT:**
- Create any Rust model files (Task 3)
- Add newtypes (Task 4)
- Run `cargo check` yourself — delegated to laptop advisor
- Edit any other file

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 2" — canonical task description and GOTCHAs
2. `.claude/lessons/feedback_diesel_print_schema_auto_applies_patch.md` — **MANDATORY**: `diesel print-schema` auto-applies `patch_file` from `diesel.toml`; warns on stale hunks; do NOT manually edit schema.rs before running print-schema
3. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY** (pre-Shape-G): write `validate-pending-laptop` DQ then STOP
4. `crates/db_schema_file/src/schema.rs` lines 157-164 — **MIRROR**: existing `actor_pseudonym` `table!` block shape
5. `crates/db_schema_file/src/schema.rs` around line 1503 — **MIRROR**: the `joinable!(actor_pseudonym -> person (person_id))` line; new `joinable!(actor_app_link -> actor_pseudonym (brehon_actor_id))` goes alphabetically near it
6. `crates/db_schema_file/src/schema.rs` around line 1629 — **MIRROR**: `allow_tables_to_appear_in_same_query!` block; `actor_app_link,` goes alphabetically near top (before `actor_pseudonym`)
7. `.claude/rules/decision-queue.md` — DQ write discipline

---

## 4. Constraints

### Implementation

**Step 1: Run the migration (if not already applied)**

The Task 1 migration must already be applied to the test DB. Confirm by checking if `actor_app_link` exists:

```bash
# On the daemon: check if migration already ran
diesel migration list 2>&1 | grep actor_app_link
```

If not applied, run:
```bash
cargo run -p lemmy_diesel_utils --features full
```

(Per `feedback_lemmy_migration_runner.md`: no sub-command args; this runs all pending migrations.)

**Step 2: Run `diesel print-schema`**

```bash
diesel print-schema > crates/db_schema_file/src/schema.rs
```

This regenerates the full schema. The `diesel.toml` `patch_file` applies automatically.

**Step 3: Verify the generated output**

After regen, confirm:
- `actor_app_link` `table!` block exists with:
  - `id -> Int4`
  - `brehon_actor_id -> Int4`
  - `app_id -> Text`
  - `app_local_id -> Text`
  - `created_at -> Timestamptz`
  - `revoked_at -> Nullable<Timestamptz>`
- `joinable!(actor_app_link -> actor_pseudonym (brehon_actor_id));` present (alphabetically near `actor_pseudonym -> person`)
- `actor_app_link,` in `allow_tables_to_appear_in_same_query!` (alphabetically near top of the list)

**GOTCHA — schema.rs location:**
- File is at `crates/db_schema_file/src/schema.rs` — NOT `crates/db_schema/src/schema.rs`
- It is **generated** — always run `diesel print-schema` to regenerate, never hand-edit the `table!` block

**GOTCHA — patch_file:**
- `diesel.toml` contains a `patch_file` that auto-applies ltree or other patches
- Per `feedback_diesel_print_schema_auto_applies_patch.md`: the patch applies automatically during regen; if you see a "warn: hunk #2 is stale" message, that is expected and not a blocker

**GOTCHA — joinable! placement:**
- New entry: `joinable!(actor_app_link -> actor_pseudonym (brehon_actor_id));`
- Alphabetically `actor_app_link` sorts BEFORE `actor_pseudonym` — place it before the `actor_pseudonym -> person` joinable line

**GOTCHA — allow_tables order:**
- `actor_app_link,` sorts BEFORE `actor_pseudonym,` alphabetically — it goes near the top of the list

### validate-pending-laptop (pre-Shape-G, MANDATORY)

After committing the schema.rs change:

1. Write a `kind: "validate-pending-laptop"` DQ entry with:
   ```json
   {
     "commands": [
       "cargo check -p lemmy_db_schema_file"
     ],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 2
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id.
   Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append it.

2. **Commit** the DQ entry: `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised validate-pending-laptop task-2 schema-regen"`

3. **Push** to origin: `git push origin phase-m2-late-b-actor`

4. **STOP. Do NOT run `cargo check` yourself.**

### DQ blocker discipline

If `diesel print-schema` fails (DB connection error, missing env), write a `kind: "blocker"` DQ entry, commit + push, then STOP.

---

## 5. Commit

Single commit:
```
feat(db_schema_file): regen schema.rs — add actor_app_link table (task 2)
```

Stage only: `crates/db_schema_file/src/schema.rs`

Then the DQ entry commit (separate).

---

## 6. DoD

- [ ] `actor_app_link` `table!` block present in `crates/db_schema_file/src/schema.rs`
- [ ] `revoked_at -> Nullable<Timestamptz>` field in the block
- [ ] `joinable!(actor_app_link -> actor_pseudonym (brehon_actor_id));` present + alphabetically ordered
- [ ] `actor_app_link,` in `allow_tables_to_appear_in_same_query!` + alphabetically ordered
- [ ] `validate-pending-laptop` DQ entry committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-2
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/db_schema_file/src/schema.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "schema.rs at crates/db_schema_file/src/schema.rs (generated, not hand-edited)"
    - "diesel print-schema auto-applies patch_file from diesel.toml"
    - "joinable! + allow_tables entries added alphabetically"
    - "validate-pending-laptop: commands=[cargo check -p lemmy_db_schema_file]"
  notes: "Task 2 of 13. Tasks 3+4 (model + newtype) can begin after this; all three feed into Task 5."
```
