---
role: impl-task
task_number: 5
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G
---

# impl-task brief — m2-late-b-actor Task 5: EXPORT `pub mod actor_app_link;` in `mod.rs`

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 5 of 13 (`[P]` — cohort-2 alongside Task 6)
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Tasks 3+4 merged (model + newtype must exist)

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-5 mod-export actor-app-link — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-5.md
```

---

## 2. Scope

**Produce:**
- One-line addition to `crates/db_schema/src/source/governance/mod.rs`: `pub mod actor_app_link;`
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Touch any other file
- Run cargo yourself

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 5"
2. `crates/db_schema/src/source/governance/mod.rs` — read the full file to find the alphabetical `pub mod` list
3. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Implementation

Read `crates/db_schema/src/source/governance/mod.rs`. The file has a flat alphabetical list of `pub mod` declarations. Find the line for `pub mod actor_pseudonym;` (line ~3) and `pub mod appeal;` (line ~4).

Insert **between** them (alphabetically `actor_app_link` < `actor_pseudonym`):

```rust
pub mod actor_app_link;
```

**GOTCHA — NO feature gate:**
- This is plain `pub mod actor_app_link;` — NOT wrapped in `#[cfg(feature = "full")]`
- Only `pub mod redaction;` is feature-gated in this file
- Feature gating happens per-derive *inside* `actor_app_link.rs`, not here

**GOTCHA — alphabetical order:**
- `actor_app_link` sorts BEFORE `actor_pseudonym` (app < pse)
- Final order in file: `..., pub mod actor_app_link;, pub mod actor_pseudonym;, pub mod appeal;, ...`

### validate-pending-laptop (MANDATORY)

After committing:

1. Write DQ entry:
   ```json
   {
     "commands": ["cargo check -p lemmy_db_schema"],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 5
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. Commit + push + **STOP**.

---

## 5. Commit

```
feat(db_schema): export pub mod actor_app_link in governance mod.rs (task 5)
```

Stage only: `crates/db_schema/src/source/governance/mod.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `pub mod actor_app_link;` present in `mod.rs` alphabetically before `actor_pseudonym`
- [ ] No `#[cfg(feature = "full")]` gate around it
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-5
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/db_schema/src/source/governance/mod.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "Plain pub mod (not feature-gated); alphabetically before actor_pseudonym"
  notes: "Task 5 of 13. After this + task-6, advisor handles task-8 (registry doc) directly."
```
