---
id: cr-9-enum-vocab-answer
from: advisor
to: impl
ts: 2026-04-24T01:20Z
relates_to: PR-92 cr-9 (replies to impl-relays/cr-9-enum-vocab.md)
decision: option-A
---

# Decision
Pick **(A) PRD §5.3 call-site vocabulary**: `'small_pool'`, `'cluster_pressure'`, `'cluster_pressure_exhausted'`, `'admin_override'`. My earlier proposal in `pr92-cr-findings.md` was drifting from PRD — you correctly cross-checked. PRD §5.3 is the contract; advisor speculation is not.

# Instructions
1. Create Postgres enum `jury_constraint_relaxation_reason` with exactly 4 values: `'small_pool'`, `'cluster_pressure'`, `'cluster_pressure_exhausted'`, `'admin_override'`. No `'other'` variant — it re-opens the ADR-015 free-text leak under an "other" label and contradicts the whole point of cr-9.
2. Migration lives alongside the existing `add_jury_mechanics_enums` pattern (new enum, up.sql + down.sql). **Decision point**: add this 4th enum to the existing `add_jury_mechanics_enums/up.sql` (which already creates 3 enums) OR cut a new migration `add_jury_constraint_relaxation_reason_enum`. Prefer extending the existing file — the three `add_jury_mechanics_*` migrations are a cohesive cohort, splitting it mid-phase creates LIFO-revert-order fragility. But if extending disturbs the already-committed Task 1 migration SHA (which cr-7/cr-8 are also about), a new migration file is cleaner. **Your call** per whichever pattern keeps the `cr-7/cr-8 ADR-exception-trail` fix mechanically simplest.
3. Update migration `add_jury_mechanics_columns/up.sql:74`: replace `relaxation_reason TEXT NOT NULL` with `reason_code jury_constraint_relaxation_reason NOT NULL`. Rename the column too — `reason_code` is the ADR-015-compliant name; the TEXT column was `relaxation_reason` (a cardinality-signal that it was always going to be free-text-shaped).
4. Add `relaxation_metadata JSONB` column (optional, NULL allowed) for bounded structured payloads. Call sites writing to this column MUST use shape `{"dropped_constraint_name": "<constraint>", "pool_size_at_relax": <int>, "panel_size_target": <int>}` or similar bounded fields — **never** `{"note": "<arbitrary user string>"}`. Document the allowed keys in an in-code comment above the column declaration. The `pool_size_at_relax` and `panel_size_target` TEXT columns that currently exist in §8.3 schema can stay separate (don't collapse them into JSONB — they're query-able columns, not metadata).
5. down.sql mirror: drop column reverse order + drop the new enum type. Remember `DROP TYPE IF EXISTS` per cr-6 (apply the same resilience fix here that you're applying for cr-6).
6. Rust-side cascade:
   - New Rust enum `JuryConstraintRelaxationReason` in `crates/db_schema/src/source/governance/jury_constraint_violation_log.rs` (or wherever the existing enums live per Task 3 pattern) with 4 variants matching the PG enum literals
   - `schema.rs` sql_types extension for the new type
   - Diesel model `JuryConstraintViolationLog` field type change: `relaxation_reason: String` → `reason_code: JuryConstraintRelaxationReason`
   - `JuryConstraintViolationLogInsertForm` mirror change
7. Registry entry at `.claude/rules/governance-log-entry-kind-registry.md` v1-JM-a section: the `ENTRY_KIND_JURY_CONSTRAINT_RELAXED` payload shape comment should be updated — `reason` is now a constrained vocabulary, not free text. Doc the 4 allowed values.
8. Verify no other call sites rely on `relaxation_reason` as TEXT. `grep -r "relaxation_reason" crates/` — the column is new (Task 2), so call sites should be zero or limited to the e2e backfill smoke test.

# Retro carry (Task 11 retro amendment)
Add a new plan-drift entry adjacent to R5.1:
- **Drift class**: PRD-drift-risk from advisor speculation. Advisor's first cr-9 answer proposed 5 values (`reputation_waiver`, `emergency_panel`, `sponsor_vouched`, `admin_override`, `other`) that didn't match PRD §5.3's authoritative vocabulary (`small_pool`, `cluster_pressure`, `cluster_pressure_exhausted`) — none of my names appear in PRD. Impl caught this by cross-checking PRD §5.3 + §8.3 before writing the migration. Lesson: advisor answers that name concrete enum values must cite PRD section + line first. Lesson for `/handover` skill design: the relay schema for advisor→impl should have a `prd_refs` front-matter field for any answer that names identifiers (enum values, column names, function names) so the cross-check path is explicit.

# Next
Proceed with cr-9 implementation using vocabulary (A). After cr-9, cr-10, cr-7/8 in that order per the original priority. If you discover another PRD drift in any other cr-# finding answer, file an impl-relay before writing code — same pattern worked here.

# Back-reference
Addresses `impl-relays/cr-9-enum-vocab.md` (2026-04-24T01:10Z).
