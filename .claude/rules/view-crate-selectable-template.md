---
paths:
  - "crates/db_views/**/*.rs"
---

# View crate Selectable template

When creating a new `crates/db_views/*` crate in the Brehon fork, follow
this rule before writing any view struct that derives
`#[cfg_attr(feature = "full", derive(Queryable, Selectable))]`.

This rule loads at session start (along with the rest of `.claude/rules/`). Every ralph loop that touches a new `db_views` crate reads it.

## The rule

For **every field** in a new view struct, classify it as one of three
kinds:

**(a) Embed from a source struct.** The field's value comes directly
from a row of a source table (e.g. `moderation_case`, `jury_assignment`)
that Diesel already knows how to `Queryable`. These can coexist with
`Selectable` via `#[diesel(embed)]` on the field.

**(b) Explicit `select_expression`.** The field's value comes from a SQL
expression that Diesel can type — a `sql::<BigInt>(...)` correlated
subquery, a `.nullable()` cast, a function call. These can coexist with
`Selectable` via `#[diesel(select_expression = ..., select_expression_type = ...)]`
on the field.

**(c) Bare scalar.** The field is a plain type (`i64`, `Option<DateTime<Utc>>`,
`i32`, etc.) with no source-table origin and no `select_expression`
annotation. Usually because the value is a stub (hardcoded constant,
placeholder for future schema), a post-query derivation, or a map-step
output.

**If ANY field in the struct is kind (c), the struct CANNOT derive
`Selectable`.** This is not a workaround or a preference — it's a
Diesel macro-expansion hard limit. The derive macro tries to generate a
`SELECT` expression covering every field, and a bare scalar with no
source-column and no explicit expression gives it nothing to emit.

## The fallback shape

When a struct contains kind (c) fields, use this pattern:

1. **Drop `Selectable` from the derive.** Keep `Queryable` and any
   serialization/tagging derives.
2. **Define a private tuple-row type** inside the query function (not
   at module scope — workspace lints deny `items_after_statements` when
   an inner type alias follows statements, and `type_complexity` when
   the tuple has >3 components). Use a module-scope `type` alias if the
   tuple is reused across queries in the same file.
3. **Load the tuple via `.select((col1, col2, ...))` + `.load::<TupleType>(conn)`**.
4. **Map the tuple to the view struct** in a separate step, filling the
   bare-scalar fields with their stub/constant/derivation values.

Example (from Phase 2a `GovernanceCaseDetailRow`):

```rust
// Module scope — type alias for the tuple load
type CaseDetailTuple = (
    ModerationCase,             // kind (a) — embeds
    Vec<Evidence>,              // kind (a) — embeds
    // ... other embeds
);

pub async fn read_case_detail(
    conn: &mut AsyncPgConnection,
    case_id: ModerationCaseId,
) -> Result<GovernanceCaseDetailRow, Error> {
    let row: CaseDetailTuple = moderation_case::table
        .filter(moderation_case::id.eq(case_id))
        .select((
            ModerationCase::as_select(),
            // ... other selects
        ))
        .first(conn)
        .await?;

    Ok(GovernanceCaseDetailRow {
        case: row.0,
        evidence: row.1,
        // Kind (c) fields get their stubs here:
        reporter_count: 0_i64,       // stub per plan §2 drift-1
        jury_needed: 5_i32,          // constant per [05 §3]
        // ... etc
    })
}
```

## Diagnostic signature

If you see this error:

```
error[E0433]: failed to resolve: unresolved import `crate::schema::governance_case_detail_rows`
 --> crates/db_views/governance_case/src/lib.rs:NN:N
```

(or any variant where the error references a table-name derived from the
struct name with `_rows` or `_views` appended), that's Diesel's derive
macro trying to find a schema module named after the struct because it
couldn't derive `Selectable` from the field shapes. The fix is to drop
`Selectable` from the derive, not to create the non-existent schema
module.

## Phase 2a existence proofs

Three Phase 2a view structs demonstrate the trichotomy:

- **`GovernanceCaseSummaryView`** — contains bare-scalar `reporter_count:
  i64 = 0_i64` (drift stub). Cannot derive `Selectable`. Uses
  tuple-load + HashMap pattern for task 16.
- **`GovernanceCaseDetailRow`** — contains bare-scalar `reporter_count`
  and `jury_needed: i32 = 5` fields. Cannot derive `Selectable`. Uses
  tuple-load + map pattern for task 17.
- **`JuryQueueView`** — contains bare-scalar `deadline_at:
  Option<DateTime<Utc>> = None` (drift stub). Cannot derive
  `Selectable`. Uses tuple-load + map pattern for tasks 22–24.

If Phase 2b (or any later phase) hits this class of error when creating
`governance_modlog`, `reputation`, or any other view crate, the fix
path is already known: drop `Selectable`, use tuple-load + map.

## When to skip

Never skip. If you're tempted to keep `Selectable` "just this once" and
fight the compiler with `#[diesel(...)]` attribute combinations, you're
about to waste 3+ iterations and then fall back to the tuple-load shape
anyway. Start with the fallback on any struct that has kind (c) fields.

## References

- Phase 2a task 15 (`GovernanceCaseDetailRow`) — the existence proof
- Phase 2a retrospective §3 (plan §6 Selectable template was
  incompatible with bare-scalar fields)
- Plan `§12 R2` fallback strategy, phase-2a plan
