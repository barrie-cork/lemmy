---
name: Planner enumerates struct callsites at plan time
description: When a plan task adds a field to a public struct, plan §11 must list ALL `rg`-discoverable callsites in `modifies:` arrays — not just the home file. Otherwise compile errors surface only at fix-impl time.
type: feedback
---

# Plan §11 must enumerate all struct callsites when adding a field

When a planning task adds a field to a **public** struct (or any struct
exported beyond a single file), the plan author MUST run
`rg "<StructName>" crates/ tests/` at plan-authoring time and list
every caller file in plan §11 "Files to change" — not just the home
file where the struct is defined.

**Why:** Per `.claude/PRPs/reports/v1-RT-r1-retro.md` §2 Planning miss
+ §3 Lesson 1. RT-r1 Task 7 added two new fields (`dedupe_key`,
`source_event_type`) to `ReputationEventInsertForm`. Plan §11 listed
the home file `crates/db_schema/src/source/governance/reputation_event.rs`
under Task 7 but did NOT list the 10 caller sites:

- `crates/api/api/src/governance/sponsor_liability.rs:296`
- `crates/api/api/src/governance/submit_jury_vote.rs:935`
- `crates/api/api_crud/src/governance/create_endorsement.rs:278+291`
- `crates/tools/seed_founders/src/main.rs:173`
- `crates/server/tests/e2e.rs:2902+2925+3935+5345+7986`

Result: fix-impl-1 (2 callsites in `crates/api/api/src/governance/`)
patched the 2 sites that the E0063 compile error pointed to. The
remaining 8 sites were silenced by the cascade-stop and surfaced only
at the SECOND workspace check, requiring fix-impl-2 (8 more sites in 3
additional files). The whole 2-cycle dance was preventable: a
plan-time `rg` would have captured the upper bound.

**Companion lesson:** `feedback_fix_impl_enumerate_all_callsites.md`
codified the discipline at fix-impl-brief authoring time (§G4
classifier). This lesson moves the enumeration **upstream** to plan
authoring, eliminating the need for advisor-side fix-impl recovery in
the common case.

**How to apply:** When authoring a plan task that modifies a public
struct's field list:

```bash
# 1. From plan-authoring shell (advisor's CWD):
rg "<StructName>" crates/ tests/
# Capture ALL file:line hits — include test files; some callers may be
# in tests/, not just crates/api/.

# 2. Filter to constructor sites (the `<Type> { ... }` shape that needs
# field padding) vs reference sites (use statements, type annotations,
# Default::default() callers) that compile cleanly regardless.

# 3. List each constructor-site file in plan §11 under the task that
# adds the field. Don't bundle this with the struct-defining file —
# call out each caller crate explicitly so §13 FILES YAML modifies:
# arrays carry the right set.
```

**Plan §11 shape (after this lesson lands):** for any task with
ACTION `extend struct <Type>` or `add field to <Type>`:

```markdown
## 11. Files to change

### `<home crate>`

- `crates/<home>/src/.../<file>.rs` — `<Type>` definition extended with
  new field(s). **Task N**.

### Caller crates (compiles-only-after-Task-N)

- `crates/api/api/src/governance/<callerA>.rs` — pad `<Type> { ... }`
  literal with new field default. **Task M (or fix-impl-1 if cohort
  isolation is desired)**.
- `crates/api/api_crud/src/governance/<callerB>.rs` — same pad.
- `crates/server/tests/e2e.rs` — pad N test-fixture sites (line refs
  from plan-time `rg`).

### Total caller count

`rg "<Type>" crates/ tests/` at plan-time returned <N> constructor
sites in <M> files. Plan author commits this enumeration to plan body
so deviation between plan-time and impl-time is auditable.
```

The cohort decision (single task covers home + callers vs separate
tasks vs separate fix-impl) is a judgment call per plan §5 complexity
score — but the enumeration is **mandatory** regardless of which
cohort shape is chosen.

**Edge cases:**

- **Public struct exported via `pub use`:** treat `pub use` shims (e.g.
  `crates/api/api/src/governance/governance_log.rs`) as caller sites for
  this rule's purposes — the shim doesn't add field semantics but
  re-export ordering must still be audited.
- **Generated code (`schema.rs`, `*.pb.rs`):** may reference the type
  but rebuild on every cargo run; safe to ignore in §11 enumeration.
- **`Default::default()` users:** `<Type> { ..Default::default() }`
  compiles cleanly because the new field gets the type's `Default`
  value. If the new field's type has a `Default` impl, these sites do
  NOT need padding — verify before listing.
- **Macro-expanded constructors:** plain `rg "<Type>"` may miss macros
  like `make_form!(<Type>, ...)`. Grep for the macro name too if the
  codebase uses one.
- **Test fixtures with mock constructors:** some test files build the
  struct via helpers (`fn mk_event() -> <Type> { ... }`). The helper
  IS the constructor site — pad there once instead of N test sites.

**Detection (retro):** a `feat(<phase>): extend <Type>` commit followed
within 1-2 commits by a `fix(<phase>): pad <N> <Type> literals` commit
is evidence the plan under-enumerated. Retro flags it. The cost is
1-N wasted fix-impl cycles; the fix is plan-authoring discipline.

**Companion lessons:**

- `feedback_fix_impl_enumerate_all_callsites.md` — the advisor-side
  fallback when this plan-side rule was missed.
- `feedback_explicit_file_arrays_on_tasks.md` — §13 FILES YAML
  `modifies:` arrays must match §11 enumerations exactly.
- `feedback_insertform_default_propagation.md` — when callers may pass
  `None` and rely on the NOT-NULL DB DEFAULT (most common pad pattern).

**Where codified:** `.claude/PRPs/templates/plan.template.md` §11
instruction text + this lesson file.
