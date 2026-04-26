---
name: Plan VALIDATE "expect 0" wrong for interim-failure tasks that split enum + sql_types
description: When a task adds a Rust enum with ExistingTypePath referring to a not-yet-created sql_types struct, say "expect compile error" or combine tasks. "Expect 0" is misleading.
type: feedback
originSessionId: 211a8c3b-c503-4ae0-afdb-bd47dba4fdc6
---
**Rule:** When a plan task adds a Rust enum that uses `#[ExistingTypePath = "crate::schema::sql_types::X"]` and the referenced `sql_types::X` struct lands in a *later* task, the first task cannot possibly pass `cargo check`. The plan's §13 task-block VALIDATE section should say:
- `expect compile error: "unresolved import crate::schema::sql_types::X" — Task N+1's sql_types addition greens this`, OR
- combine the enum-addition task and the sql_types-addition task into one commit.

Do **not** write `expect 0` on the first task — that misleads impl into re-verifying a failure that was anticipated.

**Why:** v1-JM-a R3.2: plan §13 Task 3 said "expect 0" for `cargo-check --workspace --features full`, but Task 3 added 3 Rust enums referencing `sql_types` structs that didn't land until Task 4. Impl committed as intentional-interim-failure per Phase 1 `083a9f3f9` precedent. Retro-acceptable, but the plan wording should have made the interim-failure explicit rather than relying on precedent lore.

**How to apply:**
- **In plan drafting for Diesel enum additions:**
  - If combining: one task commits enum + sql_types + schema.rs sql_type entry together. Simpler. Use when the three files are small and changes are <100 lines combined.
  - If splitting: task N commits the enum with `expect compile error: "unresolved import ..."`; task N+1 commits sql_types + schema.rs and greens.
- **In plan review:** flag any `cargo check` VALIDATE that says "expect 0" on a task adding `ExistingTypePath` references. Verify the sql_types struct is in the same task's diff; if not, fail the review or convert to "expect compile error" wording.
- **In impl receiving a plan with this pattern:** if task N's diff references `sql_types::X` that isn't in the same diff, look ahead to task N+1 to confirm the green path. If confirmed, commit task N as interim-failure with a 2-line body note citing the Phase 1 precedent.

**Retire when:** plan template includes a pre-plan-review checkbox "every interim-failure task's VALIDATE block uses 'expect compile error' not 'expect 0'". Source: v1-JM-a retro §2.1 R3.2 + plan-amendment row 1 (§3.2).
