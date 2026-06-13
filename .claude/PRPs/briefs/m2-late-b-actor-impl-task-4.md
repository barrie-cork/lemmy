---
role: impl-task
task_number: 4
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_newtype_locations_lemmy_db_schema_vs_file.md  # newtype placement rule
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G
---

# impl-task brief — m2-late-b-actor Task 4: ADD `ActorAppLinkId` newtype

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 4 of 13 (`[P]` — cohort-1 alongside Task 2 and Task 3)
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13
**Depends on:** Task 1 (migration). Does NOT depend on Task 2 or Task 3.

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-4 newtype ActorAppLinkId — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-4.md
```

You are the **impl-task** subagent (Sonnet 4.6 — pattern-following from MIRROR refs). Execute exactly what is described here.

---

## 2. Scope

**Produce:**
- `ActorAppLinkId(pub i32)` newtype added to `crates/db_schema/src/newtypes.rs`
- `validate-pending-laptop` DQ entry (commit + push)

**Do NOT:**
- Edit `actor_app_link.rs` (Task 3)
- Edit `mod.rs` (Task 5)
- Touch any other file

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 4"
2. `crates/db_schema/src/newtypes.rs` lines 289-293 — **PRIMARY MIRROR**: `ActorPseudonymId` macro; copy verbatim and rename
3. `.claude/lessons/feedback_newtype_locations_lemmy_db_schema_vs_file.md` — **MANDATORY**: newtypes live in `crates/db_schema/src/newtypes.rs` — NOT `db_schema_file`
4. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY**

---

## 4. Constraints

### Implementation

Read `crates/db_schema/src/newtypes.rs` around lines 205-295 to find the governance newtype block. Locate `ActorPseudonymId` at ~line 289-293.

Add **immediately after the existing governance newtype block** (after `ActorPseudonymId`), in a comment-delimited subsection:

```rust
// --- M2 / ADR-016 B-actor ---
make_thin_alias!(ActorAppLinkId, i32);
```

The `make_thin_alias!` macro expands to the full derive stack:
`Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize` + `#[cfg_attr(feature = "full", derive(DieselNewType))]` + ts-rs cfg_attrs.

**GOTCHA — macro name:**
- Use `make_thin_alias!(ActorAppLinkId, i32)` — copy the EXACT macro call from the `ActorPseudonymId` line at ~289
- The macro handles `DieselNewType` gating automatically; do NOT add `#[cfg_attr(feature = "full", ...)]` manually

**GOTCHA — no Display impl:**
- Governance IDs omit `impl std::fmt::Display` — do NOT add one (matches `ActorPseudonymId` convention)

**GOTCHA — no pub use:**
- The newtype is exported via the normal `crate::newtypes::ActorAppLinkId` path; no extra `pub use` needed here

**GOTCHA — placement:**
- Must be in `crates/db_schema/src/newtypes.rs`, NOT in `crates/db_schema_file/` (the file-generated schema crate)
- Add AFTER the existing governance block, not in the middle of it

### validate-pending-laptop (pre-Shape-G, MANDATORY)

After committing:

1. Write a `kind: "validate-pending-laptop"` DQ entry:
   ```json
   {
     "commands": [
       "cargo check -p lemmy_db_schema"
     ],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 4
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` + `dq-v3-append-fragment.sh --pending`.

2. **Commit** + **push** + **STOP**.

---

## 5. Commit

```
feat(db_schema): add ActorAppLinkId newtype (B-actor ADR-016 task 4)
```

Stage only: `crates/db_schema/src/newtypes.rs`

Then DQ entry commit (separate).

---

## 6. DoD

- [ ] `ActorAppLinkId(pub i32)` (or `make_thin_alias!(ActorAppLinkId, i32)`) present in `newtypes.rs`
- [ ] Placed in governance block after `ActorPseudonymId`
- [ ] `DieselNewType` derive present (via macro or explicit cfg_attr)
- [ ] `validate-pending-laptop` DQ committed + pushed

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-4
  branch: phase-m2-late-b-actor
  filesModified:
    - crates/db_schema/src/newtypes.rs
    - .claude/decision-queue.json
  keyDecisions:
    - "ActorAppLinkId uses make_thin_alias! macro (mirrors ActorPseudonymId exactly)"
    - "Placed after governance newtype block in newtypes.rs"
  notes: "Task 4 of 13. Once this merges, Task 3's cargo check will compile ActorAppLink."
```
