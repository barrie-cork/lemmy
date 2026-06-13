---
role: impl-task
task_number: 7
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G
---

# impl-task brief — m2-late-b-actor Task 7: SHIM re-export 2 new ENTRY_KIND consts

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 7 of 13
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Tasks 5+6 merged (consts must exist in db_schema before the shim re-exports them).

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-7 shim-reexport entry-kind-consts — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-7.md
```

---

## 2. Scope

**Produce:**
- 2-identifier addition to `crates/api/api/src/governance/governance_log.rs` shim `pub use` block
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Edit `crates/db_schema/src/source/governance/governance_log.rs` (Task 6 — already done)
- Edit `crates/api/api/src/governance/actor_app_link.rs` (Task 9)
- Edit any other file

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 7"
2. `crates/api/api/src/governance/governance_log.rs` lines 39-71 — **MIRROR**: the alphabetical `pub use` block; these 2 new identifiers sort FIRST (`ACTOR_` < `ADMIN_`)
3. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Implementation

Read `crates/api/api/src/governance/governance_log.rs`. Find the `pub use lemmy_db_schema::source::governance::governance_log::{` block starting at line 39.

The current first entry is `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED`. The two new consts sort BEFORE it alphabetically (`ACTOR_` < `ADMIN_`).

Add the two new identifiers as the FIRST entries in the `pub use` block:

```rust
pub use lemmy_db_schema::source::governance::governance_log::{
  ENTRY_KIND_ACTOR_APP_LINK_CREATED, ENTRY_KIND_ACTOR_APP_LINK_REVOKED,
  ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED, ENTRY_KIND_ADMIN_CONFIG_CHANGED,
  // ... rest unchanged
```

**GOTCHA — sort order:**
- `ACTOR_` < `ADMIN_` — the two new entries come FIRST in the `pub use` list
- Do NOT insert them elsewhere in the list

**GOTCHA — parity invariant:**
- The shim identifier count must equal the `db_schema` define count
- After this edit: `rg -c 'ENTRY_KIND_' crates/api/api/src/governance/governance_log.rs` (the `pub use` line occurrences) must be 2 more than before
- The total define count in `governance_log.rs` (db_schema side) is 69; the shim re-exports all of them — both sides must have the same 69 `ENTRY_KIND_*` identifiers after this edit

**GOTCHA — do NOT add `sign_link_claim` to the shim:**
- `sign_link_claim` is in `db_schema` but Task 9's handler imports it directly from `lemmy_db_schema::source::governance::governance_log::sign_link_claim` — it is NOT re-exported through this shim
- Only add the 2 `ENTRY_KIND_*` identifiers

### validate-pending-laptop (MANDATORY)

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": ["cargo check -p lemmy_api"],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 7
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**.

---

## 5. Commit

```
feat(api): shim re-export ENTRY_KIND_ACTOR_APP_LINK_{CREATED,REVOKED} (task 7)
```

Stage only: `crates/api/api/src/governance/governance_log.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `ENTRY_KIND_ACTOR_APP_LINK_CREATED,` and `ENTRY_KIND_ACTOR_APP_LINK_REVOKED,` present as the first two entries in the `pub use` block
- [ ] They sort before `ENTRY_KIND_ADMIN_CONFIG_CHANGE_DENIED` (alphabetical)
- [ ] `sign_link_claim` is NOT in the shim's `pub use`
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-7
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/api/api/src/governance/governance_log.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "2 new ENTRY_KIND consts re-exported in shim; sort first (ACTOR_ < ADMIN_)"
    - "sign_link_claim stays in db_schema only (not re-exported through shim)"
  notes: "Task 7 of 13. Task 9 (DTOs + handlers) follows."
```
