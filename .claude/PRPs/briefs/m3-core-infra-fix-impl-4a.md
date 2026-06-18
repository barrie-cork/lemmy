# Brief: m3-core-infra fix-impl-4a

## §1 Role + dispatch line

`[role:impl-task] m3-core-infra-fix-impl-4a-e2e-crud-import — see .claude/PRPs/briefs/m3-core-infra-fix-impl-4a.md`

## §2 Scope

Fix E0432 + E0599 compile errors in `crates/server/tests/e2e/governance.rs` introduced
by Task 4. The worker's `m3_actor_pseudonym_endpoint_idempotent_opaque` test nested
`traits::Crud` inside a `lemmy_db_schema` use block — but `Crud` is not exported from
`lemmy_db_schema`; it lives in `lemmy_diesel_utils::traits::Crud`.

**Produces:** one corrected edit to `crates/server/tests/e2e/governance.rs` — move the
`Crud` import to a separate `use lemmy_diesel_utils::traits::Crud;` line.

**Do NOT touch:** any file outside `crates/server/tests/e2e/governance.rs`. Do NOT change
the test logic, only the import.

**Branch:** forks from `phase-m3-core-infra` (daemon-local tip after finalize-merge of
the Task 4 worker branch, or from the worker branch tip if finalize hasn't run yet).

## §3 Required reading

- `crates/server/tests/e2e/governance.rs:5198-5210` — sibling test that shows the correct
  `use lemmy_diesel_utils::traits::Crud;` standalone import (line 5202 is the canonical
  reference).
- `crates/server/tests/e2e/governance.rs:5398-5427` — the broken test `m3_actor_pseudonym_endpoint_idempotent_opaque`.
- `.claude/lessons/feedback_lemmy_error_no_std_error.md`
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate verbatim
  anchors before editing; uniqueness gate required.

## §4 Constraints

### The fix (exact)

In `m3_actor_pseudonym_endpoint_idempotent_opaque` (lines ~5398-5427), the use block
currently reads:

```rust
  use lemmy_db_schema::{
    source::{
      instance::Instance,
      person::{Person, PersonInsertForm},
    },
    traits::Crud,
  };
```

Change it to:

```rust
  use lemmy_db_schema::source::{
    instance::Instance,
    person::{Person, PersonInsertForm},
  };
  use lemmy_diesel_utils::traits::Crud;
```

This is the only edit. The test body (Person::create callsite, assertions) is correct
and must NOT be changed.

### Anchor uniqueness gate (mandatory pre-edit)

Run BOTH of these before editing:
1. `grep -c 'traits::Crud,' crates/server/tests/e2e/governance.rs` → must be `1`
   (the broken import is unique to this test)
2. `grep -c 'm3_actor_pseudonym_endpoint_idempotent_opaque' crates/server/tests/e2e/governance.rs` → must be `1`

If either returns > 1, STOP and raise a DQ blocker.

### Validate

Write a `validate-pending-laptop` DQ entry with:
```json
{
  "kind": "validate-pending-laptop",
  "commands": [
    "cmd //c \"scripts\\\\brehon\\\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/m3-core-infra-fix4a-check.log 2>&1\""
  ],
  "branch": "phase-m3-core-infra",
  "phase_task": "4a",
  "e2e_filter": "test(m3_actor_pseudonym)"
}
```

Commit + push, then **stop**. Do NOT run cargo yourself.
