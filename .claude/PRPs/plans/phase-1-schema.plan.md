# Plan: Phase 1 — Governance Schema + Diesel Foundation

## Summary

Phase 1 lays the database foundation for every subsequent governance phase: six migrations (enums, core tables, jury tables, reputation tables, actor_pseudonym, governance_log), the matching Diesel enums in `lemmy_db_schema_file::enums`, the matching Diesel models under `crates/db_schema/src/source/governance/`, a regenerated `schema.rs`, an append-only hash-chain trigger defined in the existing `replaceable_schema/triggers.sql` (Lemmy's replaceable-schema convention), and two integration tests in `crates/server/tests/e2e.rs` that prove migrations round-trip and the hash chain actually chains.

Everything lands on a new branch `feature/phase-1-schema` cut from `governance-v0`. Validation is gated on `./scripts/brehon/cargo-check.bat -p lemmy_db_schema_file`, `-p lemmy_db_schema`, then `--workspace`, and `./scripts/brehon/cargo-test.bat --test e2e -p lemmy_server`. All output is captured to `.claude/build-*.log` files — **never piped through `tail`/`head`/`grep`** (see `.claude/rules/cargo-output-capture.md`).

## Source

- [IMPLEMENTATION-PLAN-v0.md](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §3 Phase 1 (tasks 1–13) and §4 cross-cutting requirements
- [04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) §1 (migrations), §2 (enums), §3 (models), §1.5 (indexes)
- [99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) ADR-012 (Lemmy 1.0-beta), ADR-013 (EmergencyRemove), ADR-014 (federation), ADR-015 (GDPR / actor_pseudonym), OQ-008 (AdminReview)
- [05-mvp-and-delivery-plan.md](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) §4 Step 1
- [03-architecture.md](../../../docs/brehon-law-inspired-network/03-architecture.md) §6 (governance log), §7 (crate paths)
- `.claude/rules/cargo-output-capture.md` (pipe-masks-exit-code rule — **MANDATORY**)
- Auto-memory `feedback_clippy_test_style.md`, `feedback_cargo_invocations.md`
- Lemmy replaceable-schema runner at `crates/diesel_utils/src/schema_setup/mod.rs:41-51`

## Problem Statement

The governance fork has zero schema artefacts — no migrations, no enums, no Diesel models, no trigger. Every phase downstream (read models, DTOs, handlers, federation) imports types from `lemmy_db_schema` and `lemmy_db_schema_file`, so without the foundation laid here nothing else can compile. Worse, once schemas start landing, any drift in enum ordering (e.g. forgetting `EmergencyRemove`, `AdminReview`) forces breaking migrations later, and any drift in the hash-chain trigger design produces a log that looks append-only but isn't tamper-evident.

Phase 1 must land the entire schema for **all 11 endpoints** in one go even though many tables aren't queried until Phase 5, because a piecemeal landing guarantees schema drift mid-build ([IMPLEMENTATION-PLAN-v0.md §3 Phase 1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) task 3 note).

## Solution Statement

Follow Lemmy 1.0-beta's **split crate** convention exactly:

1. **Postgres enum types** → `migrations/{ts}_add_governance_enums/{up,down}.sql`
2. **Rust `DbEnum`s** → `crates/db_schema_file/src/enums.rs` (NOT `db_schema`) — this is where Lemmy's existing enums live
3. **Schema tables** → four migrations at repo-root `migrations/` (NOT `crates/db_schema/migrations/`) for core, jury, reputation+surety+pseudonym, governance_log
4. **Auto-generated `schema.rs`** → regenerate via `diesel print-schema --patch-file crates/db_schema_file/diesel_ltree.patch > crates/db_schema_file/src/schema.rs`
5. **Diesel Rust models** → `crates/db_schema/src/source/governance/*.rs`, exported via `source/mod.rs`
6. **Hash-chain trigger** → appended to `crates/diesel_utils/replaceable_schema/triggers.sql` (function in `r.*` schema, trigger on `public.governance_log`, dropped automatically when the `r` schema is `DROP CASCADE`d)
7. **Hash-chain "signature slot" update gate** → single-shot `signature IS NULL → signature = X` transition enforced by the same function + a second trigger on UPDATE
8. **Tests** → `crates/server/tests/e2e.rs::can_insert_moderation_case` + `::governance_log_hash_chain_holds`, both `-> Result<(), Box<dyn std::error::Error>>` with `?` (matches Phase 0's existing pattern)
9. **Branch + commits** → `feature/phase-1-schema` off `governance-v0`, one commit per task

Everything matches existing Lemmy conventions — there is no new framework, no new layer, no new dependency except **`sha2`** (confirmed absent from the workspace today).

## Metadata

| Field | Value |
|---|---|
| Type | SCHEMA + CROSS_CUTTING |
| Complexity | HIGH (13 tasks, 5 migrations, 1 novel trigger scheme, 2 e2e tests, 1 new workspace dep) |
| Crates Affected | `lemmy_db_schema_file`, `lemmy_db_schema`, `lemmy_diesel_utils` (replaceable_schema), `lemmy_server` (dev-deps + tests), workspace-root `Cargo.toml` |
| v0 Step | Step 1 from [05 §4](../../../docs/brehon-law-inspired-network/05-mvp-and-delivery-plan.md) |
| Dependencies | Phase 0 complete (testcontainers harness landed in `0485a977d`) |
| Estimated Tasks | 13 (matches IMPLEMENTATION-PLAN-v0.md §3 Phase 1 exactly) |
| Branch | `feature/phase-1-schema` off `governance-v0` |

---

## Divergences from IMPLEMENTATION-PLAN-v0.md §3 Phase 1

Defensive documentation. A reader diffing this plan against the main plan would otherwise see these as freelancing; they're not — each is a convention-faithful correction to the main plan's wording. None of them change scope or contradict an ADR.

- **Postgres triggers live in `crates/diesel_utils/replaceable_schema/triggers.sql`, NOT in migration SQL.** Main plan (§3 Phase 1 task 6) says "File: in Migration 5's `up.sql`". Lemmy 1.0-beta's actual convention: all PL/pgSQL functions go in `replaceable_schema/{triggers.sql,utils.sql}`, loaded via `include_str!` at `crates/diesel_utils/src/schema_setup/mod.rs:41-51`. The `r` schema is dropped `CASCADE` on every schema rebuild; functions live in `r.*`; triggers attach to `public.*` tables and die with their functions (rule at `triggers.sql:1-3`). Plan follows Lemmy.
- **Governance enums live in `lemmy_db_schema_file::enums`, NOT in `lemmy_db_schema::source::governance::enums`.** Main plan task 8 says the latter. Lemmy 1.0-beta's actual layout: `crates/db_schema_file/src/enums.rs` is where every Postgres-backed `DbEnum` lives (`PostSortType`, `RegistrationMode`, `ListingType`, etc.). `db_schema_file` is intentionally tiny so downstream crates can depend on it without pulling in `diesel-async`. Plan follows Lemmy.
- **Migrations live at repo-root `migrations/`, NOT `crates/db_schema/migrations/`.** Main plan task 1 says `crates/db_schema/migrations/{timestamp}_add_governance_core/`. That directory does not exist in the fork; the actual convention is `migrations/` at the workspace root (see `migrations/2026-03-24-023609-0000_rename_disable_type_columns/` and dozens of others). Plan follows Lemmy.
- **The e2e test file is `crates/server/tests/e2e.rs`, NOT `tests/e2e.rs` at workspace root.** Main plan §5.1 lists both possibilities ("`tests/e2e.rs` — lives at the workspace root or `crates/server/tests/e2e.rs` (Lemmy's convention determines which)"); Phase 0 already landed the harness at `crates/server/tests/e2e.rs` in commit `0485a977d`. Plan follows the existing Phase 0 location.

---

## Flow Design

### Before State

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                              BEFORE STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                                ║
║   Workspace compiles and Phase 0's `postgres_container_boots` test passes.     ║
║                                                                                ║
║   DB_STATE: only upstream Lemmy tables. No governance tables. No governance    ║
║             enums. No hash-chain trigger. No actor_pseudonym. No tests under   ║
║             crates/server/tests/e2e.rs other than the Phase-0 smoke test.      ║
║                                                                                ║
║   CODE_STATE: no governance/ subdirectories anywhere (per §2.4 task 6 in       ║
║               IMPLEMENTATION-PLAN-v0.md §1.1 progress snapshot).               ║
║                                                                                ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### After State

```
╔═══════════════════════════════════════════════════════════════════════════════╗
║                               AFTER STATE                                      ║
╠═══════════════════════════════════════════════════════════════════════════════╣
║                                                                                ║
║   6 migrations applied (forward AND backward clean). 13 governance tables      ║
║   exist in Postgres, matching [04 §1-§3].                                      ║
║                                                                                ║
║   11 governance enums live in lemmy_db_schema_file::enums with full            ║
║   DbEnum+ts-rs derives, incl. `CaseStatus::EmergencyRemove` (ADR-013) and     ║
║   `CaseStatus::AdminReview` (OQ-008).                                          ║
║                                                                                ║
║   13 Diesel source models live under                                           ║
║   crates/db_schema/src/source/governance/{...}.rs with Queryable+Insertable    ║
║   pairs, typed IDs via diesel-derive-newtype (mirroring post_report.rs).       ║
║                                                                                ║
║   crates/db_schema_file/src/schema.rs regenerated with governance tables +     ║
║   sql_types for governance enums (via `diesel print-schema`).                  ║
║                                                                                ║
║   Hash-chain trigger lives in                                                  ║
║   crates/diesel_utils/replaceable_schema/triggers.sql as:                      ║
║     • r.governance_log_hash_chain_before_insert()  BEFORE INSERT on            ║
║       public.governance_log — computes entry_hash = sha256(prev||kind||...)    ║
║     • r.governance_log_signature_gate_before_update() BEFORE UPDATE on         ║
║       public.governance_log — allows only single NULL→signature transition    ║
║     • r.governance_log_append_only_before_delete() BEFORE DELETE — RAISE       ║
║                                                                                ║
║   DB_GRANT: `governance_log` revokes UPDATE/DELETE from the default role;      ║
║   only the hash-chain + signature-gate triggers bypass via SECURITY DEFINER.   ║
║                                                                                ║
║   sha2 = "0.10" added to [workspace.dependencies]. Used by the e2e test to     ║
║   recompute the chain in Rust and assert it matches the stored hashes.         ║
║                                                                                ║
║   crates/server/tests/e2e.rs now has 3 tests, all                              ║
║   `-> Result<(), Box<dyn Error>>`:                                             ║
║     • postgres_container_boots                 (Phase 0, unchanged)            ║
║     • can_insert_moderation_case               (task 12 — new)                 ║
║     • governance_log_hash_chain_holds          (task 13 — new)                 ║
║                                                                                ║
║   VALUE_ADD: every subsequent phase can `use lemmy_db_schema::source::         ║
║              governance::*` and `use lemmy_db_schema_file::enums::CaseStatus`  ║
║              without touching the schema layer again.                          ║
║                                                                                ║
╚═══════════════════════════════════════════════════════════════════════════════╝
```

### Endpoint Changes

No HTTP endpoints in Phase 1. Phase 1 is schema-only. The first endpoint wiring is in Phase 4.

---

## Mandatory Reading (implementation agent MUST read before starting)

| Priority | File | Lines | Why |
|---|---|---|---|
| P0 | [docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) | 132–173 (§3 Phase 1), 386–432 (§4 cross-cutting), 494–508 (§6 Monday-morning) | Phase task list, cross-cutting invariants, day-1 order |
| P0 | [docs/brehon-law-inspired-network/04-data-model-and-api.md](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) | 11–51 (§1 migrations + indexes), 52–178 (§2 enums), 179–406 (§3 models) | Authoritative field shapes, enum variants, index list |
| P0 | [docs/brehon-law-inspired-network/99-decisions-and-open-questions.md](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) | 226–270 (ADR-013, ADR-014, ADR-015) | Why EmergencyRemove, AdminReview, actor_pseudonym are non-negotiable |
| P0 | `.claude/rules/cargo-output-capture.md` | all | Mandatory pipe-masks-exit-code discipline. Violating this in a ralph loop fires a false-green COMPLETE |
| P0 | `crates/db_schema_file/src/enums.rs` | 1–94 | `DbEnum` / `ExistingTypePath` / `DbValueStyle` / `ts-rs` derive pattern — COPY VERBATIM for governance enums |
| P0 | `crates/db_schema/src/source/post_report.rs` | 1–51 | `Queryable+Insertable` struct pair — THE closest Lemmy analogue for `moderation_case` |
| P0 | `crates/db_schema_file/src/schema.rs` | 1–50 | `sql_types { ... }` module convention + `diesel::table!` macro style — format governance tables will take after regeneration |
| P0 | `crates/diesel_utils/replaceable_schema/triggers.sql` | 1–16, 343–396 | Replaceable-schema rules (comment block at top) + `BEFORE INSERT` trigger pattern (`r.comment_change_values()`, `r.post_change_values()`) |
| P0 | `crates/diesel_utils/src/schema_setup/mod.rs` | 41–53, 200–280 | How the runner drops+recreates the `r` schema. **Critical**: functions MUST be in `r.*`, triggers attach to `public.*` tables |
| P0 | `crates/server/tests/e2e.rs` | 1–37 | Phase-0 harness we extend — sets the test function signature convention (`-> Result<(), Box<dyn Error>>`) |
| P1 | `migrations/2022-12-05-110642_registration_mode/{up,down}.sql` | all | Enum `CREATE TYPE` + `DROP TYPE` round-trip — the cleanest existing Lemmy enum-migration example |
| P1 | `migrations/2026-03-24-023609-0000_rename_disable_type_columns/up.sql` | all | Most-recent migration — confirms directory naming `YYYY-MM-DD-HHMMSS-0000_name/` |
| P1 | `crates/db_schema/src/source/mod.rs` | 1–56 | `pub mod X;` export convention — governance module gets added here |
| P1 | `crates/db_schema/src/lib.rs` | 1–30 | `pub mod source;` re-export — no change needed, module nests under source |
| P1 | `Cargo.toml` (workspace root) | 71–107 | `[workspace.lints.clippy]` deny list — tests MUST respect `unwrap_used`/`expect_used`/`allow_attributes`/`tests_outside_test_module` |
| P1 | `diesel.toml` | all | `print_schema` config — the `patch_file` MUST be passed when regenerating |
| P1 | `crates/db_schema/src/impls/person.rs` | 158–160 | `PersonInsertForm::test_form()` pattern — governance fixtures should follow the same `test_form(...)` helper shape when Phase 2 arrives (not needed in Phase 1 because tests use raw inserts) |

**External documentation (research done, versions pinned):**

| Source | Version | Section | Why |
|---|---|---|---|
| [diesel 2.3](https://docs.rs/diesel/2.3.7) | 2.3.7 (workspace-pinned) | `table!` macro, `check_for_backend` | Mirrors schema.rs generation style |
| [diesel-derive-enum 2.1](https://docs.rs/diesel-derive-enum/2.1.0) | 2.1.0 (workspace-pinned) | `DbEnum`, `ExistingTypePath`, `DbValueStyle="verbatim"` | Governance enums use the same attribute set |
| [sha2 0.10](https://docs.rs/sha2/0.10) | 0.10 (new to workspace) | `Sha256::new()`, `update()`, `finalize()` | Used by the e2e test to recompute the chain in Rust; also lives in Postgres as `digest('sha256'...)` from `pgcrypto` |
| [postgres `pgcrypto`](https://www.postgresql.org/docs/16/pgcrypto.html) | PG 16 | `digest(text, 'sha256')` | Trigger computes `entry_hash` server-side; needs `CREATE EXTENSION IF NOT EXISTS pgcrypto` in the `add_governance_log` migration |
| [Lemmy replaceable-schema pattern](../../../crates/diesel_utils/src/schema_setup/mod.rs) | in-repo | `run_replaceable_schema()` / `revert_replaceable_schema()` | Governance trigger MUST live in `r.*` so it drops cleanly with `DROP SCHEMA r CASCADE` |

---

## Patterns to Mirror

**MIGRATION_ENUM_CREATE_AND_DROP** (source: `migrations/2022-12-05-110642_registration_mode/up.sql`, `down.sql`, full files):

```sql
-- up.sql — create enum type BEFORE any table uses it
CREATE TYPE registration_mode_enum AS enum (
    'closed',
    'require_application',
    'open'
);

ALTER TABLE local_site
    ADD COLUMN registration_mode registration_mode_enum NOT NULL DEFAULT 'require_application';
```

```sql
-- down.sql — drop columns/tables FIRST, drop TYPE LAST
ALTER TABLE local_site DROP COLUMN registration_mode;
DROP TYPE registration_mode_enum;
```

**DBENUM_DERIVE_PATTERN** (source: `crates/db_schema_file/src/enums.rs:75-94`, verbatim):

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::RegistrationModeEnum"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// The registration mode for your site. Determines what happens after a user signs up.
pub enum RegistrationMode {
  /// Closed to public.
  Closed,
  /// Open, but pending approval of a registration application.
  RequireApplication,
  /// Open to all.
  #[default]
  Open,
}
```

Notes to carry into governance enums:
- **`rename_all = "snake_case"`** applies to serde; combined with `DbValueStyle = "verbatim"` the Postgres enum variants match the Rust `PascalCase` names verbatim. **Keep the Postgres `CREATE TYPE ... AS ENUM (...)` values in PascalCase** (e.g., `'Open'`, `'ThresholdMet'`, `'EmergencyRemove'`) to match `DbValueStyle="verbatim"`.
- `#[cfg_attr(feature = "ts-rs", ...)]` is already wired in `db_schema_file`'s `Cargo.toml` — governance enums inherit the feature.
- `#[default]` on a variant is required by `Default` derive. Pick sane defaults (e.g., `CaseStatus::Open`, `CaseSeverity::Medium`).

**QUERYABLE_MODEL_PATTERN** (source: `crates/db_schema/src/source/post_report.rs:11-32`, verbatim):

```rust
#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable, Associations)
)]
#[cfg_attr(feature = "full", diesel(belongs_to(crate::source::post::Post)))]
#[cfg_attr(feature = "full", diesel(table_name = post_report))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A post report.
pub struct PostReport {
  pub id: PostReportId,
  pub creator_id: PersonId,
  pub post_id: PostId,
  // ... fields
  pub published_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
}
```

**INSERT_FORM_PATTERN** (source: `crates/db_schema/src/source/post_report.rs:34-46`, verbatim):

```rust
#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = post_report))]
pub struct PostReportForm {
  pub creator_id: PersonId,
  pub post_id: PostId,
  // ... non-auto fields only
}
```

**TYPED_ID_NEWTYPE_PATTERN** — governance FK columns should use typed IDs where they cross existing Lemmy entity boundaries (`PersonId`, `CommunityId`, `PostId`, `CommentId`) and should introduce **new** `DieselNewType` newtypes for governance-internal IDs (`ModerationCaseId`, `JuryAssignmentId`, `SanctionId`, `ActorPseudonymId`, `GovernanceLogId`). The existing file for newtypes is `crates/db_schema/src/newtypes.rs`. Pattern:

```rust
// in crates/db_schema/src/newtypes.rs, following the existing convention
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct ModerationCaseId(pub i32);
```

**TRIGGER_IN_REPLACEABLE_SCHEMA_PATTERN** (source: `crates/diesel_utils/replaceable_schema/triggers.sql:365-380`, verbatim):

```sql
CREATE FUNCTION r.post_change_values ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Set local ap_id
    IF NEW.local THEN
        NEW.ap_id = coalesce(NEW.ap_id, r.local_url ('/post/' || NEW.id::text));
    END IF;
    RETURN NEW;
END
$$;
CREATE TRIGGER change_values
    BEFORE INSERT ON post
    FOR EACH ROW
    EXECUTE FUNCTION r.post_change_values ();
```

**Key rule from the top of `triggers.sql`**: _"A trigger is associated with a table instead of a schema, so they can't be in the `r` schema. This is okay if the function specified after `EXECUTE FUNCTION` is in `r`, since dropping the function drops the trigger."_ The governance trigger FUNCTION must live in `r.*`; the TRIGGER itself attaches to `public.governance_log`.

**TEST_PATTERN** (source: `crates/server/tests/e2e.rs:18-36`, verbatim — extending, not replacing):

```rust
#[tokio::test]
async fn postgres_container_boots() -> Result<(), Box<dyn Error>> {
  let container = GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")
    .with_exposed_port(5432.tcp())
    .with_wait_for(WaitFor::message_on_stderr(
      "database system is ready to accept connections",
    ))
    .with_env_var("POSTGRES_USER", "lemmy")
    .with_env_var("POSTGRES_PASSWORD", "password")
    .with_env_var("POSTGRES_DB", "lemmy")
    .start()
    .await?;

  let host_port = container.get_host_port_ipv4(5432).await?;
  assert!(host_port > 0, "postgres mapped port should be non-zero");
  Ok(())
}
```

New governance tests must:
- Use the **same `GenericImage`** + same signature `-> Result<(), Box<dyn std::error::Error>>` + same `use std::error::Error;` import at the top. Matches the rule in `feedback_clippy_test_style.md` Pattern 2.
- **Never** `.unwrap()` or `.expect()` — workspace clippy denies both.
- **Never** `#[allow(clippy::...)]` — workspace clippy denies `allow_attributes`. If an escape hatch is unavoidable, use `#[expect(clippy::...)]` (task-12 and task-13 are designed so this isn't needed).

---

## Files to Change

| File | Action | Justification |
|---|---|---|
| `Cargo.toml` (workspace root) `[workspace.dependencies]` | UPDATE | Add `sha2 = "0.10"` (confirmed absent today) |
| `migrations/{ts1}_add_governance_enums/up.sql` | CREATE | Postgres enum types for §2 enums incl. `EmergencyRemove`, `AdminReview` |
| `migrations/{ts1}_add_governance_enums/down.sql` | CREATE | `DROP TYPE` in reverse dependency order |
| `migrations/{ts2}_add_governance_core/up.sql` | CREATE | `moderation_case`, `case_evidence`, `sanction`, `appeal`, `public_case_log` + indexes from §1.5 |
| `migrations/{ts2}_add_governance_core/down.sql` | CREATE | Drops in reverse FK order |
| `migrations/{ts3}_add_jury_system/up.sql` | CREATE | `jury_pool`, `jury_assignment`, `jury_vote` + `jury_assignment(person_id,status)` index |
| `migrations/{ts3}_add_jury_system/down.sql` | CREATE | Drops |
| `migrations/{ts4}_add_reputation_and_surety/up.sql` | CREATE | `surety`, `endorsement`, `reputation_event`, `reputation_snapshot` + `reputation_event(person_id,community_id,created_at)` index |
| `migrations/{ts4}_add_reputation_and_surety/down.sql` | CREATE | Drops |
| `migrations/{ts5}_add_actor_pseudonym/up.sql` | CREATE | `actor_pseudonym` table (ADR-015) |
| `migrations/{ts5}_add_actor_pseudonym/down.sql` | CREATE | Drops |
| `migrations/{ts6}_add_governance_log/up.sql` | CREATE | `governance_log` table + `CREATE EXTENSION IF NOT EXISTS pgcrypto` + revoke UPDATE/DELETE grants from app role |
| `migrations/{ts6}_add_governance_log/down.sql` | CREATE | Drops |
| `crates/db_schema_file/src/enums.rs` | UPDATE | Append 11 new `DbEnum`s per [04 §2](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) |
| `crates/db_schema_file/src/schema.rs` | UPDATE (regen) | Regenerate via `diesel print-schema` to pick up new tables + `sql_types` |
| `crates/db_schema/src/newtypes.rs` | UPDATE | Add `ModerationCaseId`, `CaseEvidenceId`, `SanctionId`, `AppealId`, `PublicCaseLogId`, `JuryPoolId`, `JuryAssignmentId`, `JuryVoteId`, `SuretyId`, `EndorsementId`, `ReputationEventId`, `ReputationSnapshotId`, `ActorPseudonymId`, `GovernanceLogId` as `DieselNewType`s |
| `crates/db_schema/src/source/governance/mod.rs` | CREATE | New module root with `pub mod` declarations for all 13 source files |
| `crates/db_schema/src/source/governance/moderation_case.rs` | CREATE | `ModerationCase` + `ModerationCaseInsertForm` |
| `crates/db_schema/src/source/governance/case_evidence.rs` | CREATE | `CaseEvidence` + `CaseEvidenceInsertForm` |
| `crates/db_schema/src/source/governance/sanction.rs` | CREATE | `Sanction` + `SanctionInsertForm` |
| `crates/db_schema/src/source/governance/appeal.rs` | CREATE | `Appeal` + `AppealInsertForm` |
| `crates/db_schema/src/source/governance/public_case_log.rs` | CREATE | `PublicCaseLog` + `PublicCaseLogInsertForm` |
| `crates/db_schema/src/source/governance/jury_pool.rs` | CREATE | `JuryPool` + `JuryPoolInsertForm` |
| `crates/db_schema/src/source/governance/jury_assignment.rs` | CREATE | `JuryAssignment` + `JuryAssignmentInsertForm` |
| `crates/db_schema/src/source/governance/jury_vote.rs` | CREATE | `JuryVote` + `JuryVoteInsertForm` |
| `crates/db_schema/src/source/governance/surety.rs` | CREATE | `Surety` + `SuretyInsertForm` |
| `crates/db_schema/src/source/governance/endorsement.rs` | CREATE | `Endorsement` + `EndorsementInsertForm` |
| `crates/db_schema/src/source/governance/reputation_event.rs` | CREATE | `ReputationEvent` + `ReputationEventInsertForm` |
| `crates/db_schema/src/source/governance/reputation_snapshot.rs` | CREATE | `ReputationSnapshot` + `ReputationSnapshotInsertForm` |
| `crates/db_schema/src/source/governance/actor_pseudonym.rs` | CREATE | `ActorPseudonym` + `ActorPseudonymInsertForm` |
| `crates/db_schema/src/source/governance/governance_log.rs` | CREATE | `GovernanceLog` + `GovernanceLogInsertForm` (insert form has **no** `prev_hash`/`entry_hash`/`signature` fields — all three are trigger-managed) |
| `crates/db_schema/src/source/mod.rs` | UPDATE | Add `pub mod governance;` |
| `crates/diesel_utils/replaceable_schema/triggers.sql` | UPDATE | Append `r.governance_log_hash_chain_before_insert()`, `r.governance_log_signature_gate_before_update()`, `r.governance_log_append_only_before_delete()` + the three `CREATE TRIGGER` statements on `public.governance_log` |
| `crates/server/Cargo.toml` `[dev-dependencies]` | UPDATE | Add `lemmy_db_schema = { path = "../db_schema", features = ["full"] }`, `lemmy_db_schema_file = { path = "../db_schema_file", features = ["full"] }`, `diesel = { workspace = true }`, `diesel_migrations = { workspace = true }`, `diesel-async = { workspace = true }`, `sha2 = { workspace = true }`, `url = { workspace = true }` |
| `crates/server/tests/e2e.rs` | UPDATE | Add `can_insert_moderation_case` (task 12) + `governance_log_hash_chain_holds` (task 13); add a small private `mod governance_fixtures` with a helper that spins the container, builds the DB URL, runs migrations via `diesel_migrations::embed_migrations!("../../migrations")` against a sync `PgConnection`, and returns the URL + a persistent pool handle |

---

## NOT Building (v0 scope limits)

These are all explicitly deferred per [IMPLEMENTATION-PLAN-v0.md §10](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) and the ADRs. They must NOT appear in Phase 1:

- **No handlers, DTOs, routes, or API wiring** — Phase 1 is schema-only. All of that is Phase 3–4.
- **No `db_views` governance crates** — `crates/db_views/governance_case/`, `jury_queue/`, `governance_modlog/`, `reputation/` are Phase 2 / Phase 5 work per [IMPLEMENTATION-PLAN-v0.md §3 Phase 2](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md).
- **No federation (`apub/`) code** — Phase 6.
- **No `crates/api/api_common/src/governance.rs`** — Phase 3.
- **No reputation calculation** — the `reputation_event` / `reputation_snapshot` tables exist, but nothing reads or writes them until Phase 5. [IMPLEMENTATION-PLAN-v0.md §3 Phase 1 task 3 note](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): _"Created in Phase 1 even though Phase 5 wires the logic — having all four migrations land at the same time avoids schema drift mid-build."_
- **No `redaction::scrub(...)` helper** — §4.2 says first use is Phase 4 task 42. Don't prematurely write it. Phase 1 only needs to confirm the `actor_pseudonym` table exists and has the right shape.
- **No `governance_log::append(...)` Rust helper** — §4.1 says "first use in Phase 4." Don't write it here. Phase 1 only needs the table + trigger + `GovernanceLogInsertForm` struct.
- **No `ed25519-dalek` signing code** — §4.1 Option A defers signing to a Rust post-insert UPDATE in Phase 4. Phase 1 only provisions the `signature BYTEA` column and the single-shot update gate. **Do not add `ed25519-dalek` as a dep yet.**
- **No ActivityPub types** — ADR-014 is Phase 6.
- **No `federation_attestation` / `remote_sanction_notice` tables** — [04 §1 Migration 4](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) is explicitly Phase 6 work per [IMPLEMENTATION-PLAN-v0.md §3 Phase 6](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md). **NOT in Phase 1.**
- **No GDPR admin CLI / right-to-delete endpoint** — §4.2 says the admin tool is "not in the 11." Ignore it.
- **No background jobs / daily verification task** — §4.1 says the verifier lives in `crates/server/src/governance.rs` and is a Phase 4–5 task.
- **No Extism plugins** — ADR-012 authorises them for v1+ only; v0 is direct Rust.
- **No CI wiring** — §2.4 task 9 is Pre-flight, not Phase 1. Noted as a risk in §1.1 progress snapshot; not in scope here.
- **No rename of "brehon-fork"** — OQ-012 is deferred.

---

## Step-by-Step Tasks

Execute in strict dependency order. **One commit per task.** Each task has a MIRROR reference, exact file paths, and a validation command that captures output to `.claude/build-*.log` without piping.

**Before starting any task**, confirm:
- `git checkout governance-v0 && git pull` then `git checkout -b feature/phase-1-schema`
- `git status` clean, `cargo check --workspace` passes via `./scripts/brehon/cargo-check.bat > .claude/build-baseline.log 2>&1 && tail -20 .claude/build-baseline.log` — baseline greens (if anything is red, STOP and surface to user)
- `.claude/PRPs/plans/phase-1-schema.plan.md` (this file) is on disk

---

### Task 1 — Add `sha2` workspace dependency

**ACTION**: Edit workspace root `Cargo.toml`. Add `sha2 = "0.10"` to `[workspace.dependencies]` (alphabetical position, between `serde_with` and `strum` or similar — preserve existing ordering). No code uses it yet; this is a pure dep addition.

**IMPLEMENT**:
```toml
sha2 = "0.10"
```

**MIRROR**: `Cargo.toml:109-237` — follow the existing formatting (one dep per line, version-only for simple deps).

**GOTCHA**: Don't confuse with `sha2 = { workspace = true }` — that syntax is for *consumers*, not for the workspace declaration itself. The workspace declaration uses the raw `{version = ...}` or plain version-string form.

**VALIDATE**:
```bash
./scripts/brehon/cargo-check.bat --workspace > .claude/build-task-01.log 2>&1
status=$?
tail -40 .claude/build-task-01.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```
Expected: adding an unused workspace dep is a no-op; `cargo check` still exits 0. Don't be surprised if nothing recompiles.

**COMMIT**: `chore(deps): add sha2 0.10 to workspace (used by governance log in phase 1)`

---

### Task 2 — Migration: `add_governance_enums`

**ACTION**: Create `migrations/{YYYY-MM-DD-HHMMSS}-0000_add_governance_enums/{up,down}.sql`. Use `diesel migration generate add_governance_enums` from the repo root (it will emit both empty files with a correct timestamp). Populate them.

**IMPLEMENT** — `up.sql` (per [04 §2](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md); variant names PascalCase to match `DbValueStyle="verbatim"`):

```sql
CREATE TYPE case_status AS ENUM (
    'Open',
    'ThresholdMet',
    'JurySelection',
    'InReview',
    'Decided',
    'Appealed',
    'Closed',
    'EmergencyRemove',
    'AdminReview'
);

CREATE TYPE case_target_type AS ENUM (
    'Post',
    'Comment',
    'Person',
    'Community',
    'RemoteInstance'
);

CREATE TYPE case_severity AS ENUM (
    'Low',
    'Medium',
    'High',
    'Critical'
);

CREATE TYPE evidence_visibility AS ENUM (
    'JuryOnly',
    'PrivateAdmin',
    'PublicRedacted'
);

CREATE TYPE jury_assignment_status AS ENUM (
    'Selected',
    'Accepted',
    'Declined',
    'Conflicted',
    'Submitted',
    'Expired'
);

CREATE TYPE jury_decision AS ENUM (
    'NoAction',
    'AdvisoryLabel',
    'Warning',
    'Cooldown',
    'RemoveContent',
    'SuspendLocalUser',
    'SuspendCommunityMember',
    'RecommendFederationAction'
);

CREATE TYPE sanction_scope AS ENUM (
    'Community',
    'Instance',
    'FederatedRecommendation'
);

CREATE TYPE sanction_action AS ENUM (
    'Label',
    'VisibilityReduction',
    'TemporaryRestriction',
    'ContentRemoval',
    'CommunityExclusion',
    'InstanceSuspension',
    'FederationQuarantineRecommendation'
);

CREATE TYPE appeal_status AS ENUM (
    'Requested',
    'Accepted',
    'Rejected',
    'Decided'
);

CREATE TYPE reputation_dimension AS ENUM (
    'ReportingAccuracy',
    'JuryReliability',
    'ParticipationConsistency',
    'EndorsementStrength'
);

CREATE TYPE attestation_type AS ENUM (
    'TrustedReporter',
    'JuryEligible',
    'SanctionNotice',
    'QuarantineRecommendation'
);
```

**IMPLEMENT** — `down.sql` (reverse order):

```sql
DROP TYPE attestation_type;
DROP TYPE reputation_dimension;
DROP TYPE appeal_status;
DROP TYPE sanction_action;
DROP TYPE sanction_scope;
DROP TYPE jury_decision;
DROP TYPE jury_assignment_status;
DROP TYPE evidence_visibility;
DROP TYPE case_severity;
DROP TYPE case_target_type;
DROP TYPE case_status;
```

**MIRROR**: `migrations/2022-12-05-110642_registration_mode/up.sql` — enum creation style. `migrations/2023-04-14-175955_add_listingtype_sorttype_enums/up.sql` — multiple enums in one migration.

**CRITICAL — this is NOT re-litigable**:
- `case_status` **MUST** include `'EmergencyRemove'` per [ADR-013](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) — illegal content takedowns from day 1
- `case_status` **MUST** include `'AdminReview'` per [OQ-008 resolution](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) §8 — compatibility with direct Lemmy moderator action during v0/v1 transition
- Variant names are PascalCase strings so `DbValueStyle="verbatim"` round-trips correctly

**GOTCHA**: `attestation_type` enum is defined even though [Migration 4 (federation_attestation)](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) is Phase 6. This is deliberate — enum creation is cheap and keeps the enum file alphabetically complete. The Phase 6 migration references the enum.

**CORRECTION (task 10 surfaced this, 2026-04-15)**: The *Postgres* `attestation_type` enum IS created in Phase 1 (this migration). The *Rust* `AttestationType` enum is **NOT** added in Phase 1 task 9. Reason: `diesel print-schema` only emits `sql_types::*` entries for enums that are referenced by at least one table column. Until Phase 6 creates `federation_attestation(... type attestation_type ...)`, the generated schema.rs has no `sql_types::AttestationType`, so a task-9 `DbEnum` derive with `ExistingTypePath = "crate::schema::sql_types::AttestationType"` fails `cargo check --workspace` with `cannot find type/value AttestationType in module crate::schema::sql_types`. Phase 6 adds both the table AND the Rust enum together. The Postgres type sits unused in `pg_type` until then — harmless. This is only a Rust-side deferral, not an ADR-013/OQ-008-style correctness mandate.

**VALIDATE**:
```bash
./scripts/brehon/cargo-check.bat --workspace > .claude/build-task-02.log 2>&1
status=$?
tail -40 .claude/build-task-02.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```
No Rust change yet; cargo check is expected to pass. Migrations aren't tested yet (they run against the testcontainers DB in task 12).

**COMMIT**: `feat(migrations): add governance enums (case_status incl. EmergencyRemove + AdminReview)`

---

### Task 3 — Migration: `add_governance_core`

**ACTION**: Create `migrations/{ts2}_add_governance_core/{up,down}.sql` (timestamp must be AFTER task 2's timestamp so `case_status`, `case_target_type`, `case_severity`, `evidence_visibility`, `sanction_scope`, `sanction_action`, `appeal_status` already exist). Use `diesel migration generate add_governance_core`.

**IMPLEMENT** — `up.sql`:

```sql
CREATE TABLE moderation_case (
    id SERIAL PRIMARY KEY,
    community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    creator_id INTEGER REFERENCES person (id) ON DELETE SET NULL,
    target_type case_target_type NOT NULL,
    target_post_id INTEGER REFERENCES post (id) ON DELETE SET NULL,
    target_comment_id INTEGER REFERENCES comment (id) ON DELETE SET NULL,
    target_person_id INTEGER REFERENCES person (id) ON DELETE SET NULL,
    target_community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    target_remote_url TEXT,
    reason_code TEXT NOT NULL,
    severity case_severity NOT NULL DEFAULT 'Medium',
    status case_status NOT NULL DEFAULT 'Open',
    threshold_score BIGINT NOT NULL DEFAULT 0,
    opened_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ
);

CREATE INDEX idx_moderation_case_status_created ON moderation_case (status, opened_at);

CREATE TABLE case_evidence (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    uploader_id INTEGER NOT NULL REFERENCES person (id) ON DELETE RESTRICT,
    storage_key TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    visibility evidence_visibility NOT NULL DEFAULT 'JuryOnly',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sanction (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    scope sanction_scope NOT NULL,
    action sanction_action NOT NULL,
    target_person_id INTEGER REFERENCES person (id) ON DELETE SET NULL,
    target_post_id INTEGER REFERENCES post (id) ON DELETE SET NULL,
    target_comment_id INTEGER REFERENCES comment (id) ON DELETE SET NULL,
    target_community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    starts_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ends_at TIMESTAMPTZ,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE appeal (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    requester_id INTEGER NOT NULL REFERENCES person (id) ON DELETE RESTRICT,
    reason TEXT NOT NULL,
    status appeal_status NOT NULL DEFAULT 'Requested',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at TIMESTAMPTZ
);

CREATE TABLE public_case_log (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    summary TEXT NOT NULL,
    rationale_redacted TEXT,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_public_case_log_community_published ON public_case_log (community_id, published_at);
```

**IMPLEMENT** — `down.sql` (reverse FK order):

```sql
DROP INDEX IF EXISTS idx_public_case_log_community_published;
DROP TABLE public_case_log;
DROP TABLE appeal;
DROP TABLE sanction;
DROP TABLE case_evidence;
DROP INDEX IF EXISTS idx_moderation_case_status_created;
DROP TABLE moderation_case;
```

**MIRROR**: Recent Lemmy migration `migrations/2026-03-13-123650-0000_fix_post_community_indexes/up.sql` for multi-table index style. `migrations/2022-12-05-110642_registration_mode/{up,down}.sql` for the round-trip pattern.

**GOTCHA**:
- Default timestamps use `DEFAULT now()` so `ModerationCaseInsertForm` doesn't need to supply `opened_at` (matches [04 §3 ModerationCaseInsertForm](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) which omits it).
- `target_*` FKs are all `ON DELETE SET NULL` — a deleted post shouldn't cascade a case away; the case should survive with nulls. `case_id`-rooted FKs cascade (if the case goes, its evidence/sanctions/appeals/public_log go with it).
- Integer column type is `INTEGER` (i32 in Rust) to match the rest of Lemmy's schema. Do NOT use `BIGINT` for ID columns. `threshold_score` is `BIGINT` because it aggregates weighted votes.
- `decided_at` + `closed_at` are `NULL` until the case transitions — matches the `Option<DateTime<Utc>>` fields in [04 §3 ModerationCase](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md).

**JUDGMENT CALL**: [04 §3 ModerationCase](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) has `opened_at` in the Rust struct but the prose in [04 §1.5 Indexes](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) says `moderation_case(status, created_at)`. I'm using `opened_at` for the index because that's the column name in the [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) struct. If a future reviewer wants the column renamed to `created_at` for consistency with `comment.published_at` / `post.published_at`, that's a future migration, not a Phase-1 fix.

**VALIDATE**:
```bash
./scripts/brehon/cargo-check.bat --workspace > .claude/build-task-03.log 2>&1
status=$?
tail -40 .claude/build-task-03.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```

**COMMIT**: `feat(migrations): add governance core tables (moderation_case, evidence, sanction, appeal, public_case_log)`

---

### Task 4 — Migration: `add_jury_system`

**ACTION**: `migrations/{ts3}_add_jury_system/{up,down}.sql`. Timestamp strictly after task 3.

**IMPLEMENT** — `up.sql`:

```sql
CREATE TABLE jury_pool (
    id SERIAL PRIMARY KEY,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    eligible_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (community_id, person_id)
);

CREATE TABLE jury_assignment (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    status jury_assignment_status NOT NULL DEFAULT 'Selected',
    selected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ,
    submitted_at TIMESTAMPTZ,
    UNIQUE (case_id, person_id)
);

CREATE INDEX idx_jury_assignment_person_status ON jury_assignment (person_id, status);

CREATE TABLE jury_vote (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    juror_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    decision jury_decision NOT NULL,
    rationale TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (case_id, juror_id)
);
```

**IMPLEMENT** — `down.sql`:

```sql
DROP TABLE jury_vote;
DROP INDEX IF EXISTS idx_jury_assignment_person_status;
DROP TABLE jury_assignment;
DROP TABLE jury_pool;
```

**MIRROR**: Same as task 3.

**JUDGMENT CALL**: `jury_pool` has a `(community_id, person_id)` uniqueness constraint. [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) doesn't spec jury_pool fields explicitly (only mentions it in passing). I'm inferring minimal shape: a (community, person) eligibility record with a `created_at` timestamp. If this shape turns out to be wrong in Phase 5 when it's first queried, it's a single-migration fix — but creating the table now means Phase 5 doesn't spawn a mid-build schema drift.

**VALIDATE**: Same as task 3 pattern, `.claude/build-task-04.log`.

**COMMIT**: `feat(migrations): add jury system tables (jury_pool, jury_assignment, jury_vote)`

---

### Task 5 — Migration: `add_reputation_and_surety`

**ACTION**: `migrations/{ts4}_add_reputation_and_surety/{up,down}.sql`.

**IMPLEMENT** — `up.sql`:

```sql
CREATE TABLE surety (
    id SERIAL PRIMARY KEY,
    sponsor_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    sponsored_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    UNIQUE (sponsor_id, sponsored_id, community_id)
);

CREATE TABLE endorsement (
    id SERIAL PRIMARY KEY,
    from_person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    to_person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    UNIQUE (from_person_id, to_person_id, community_id)
);

CREATE TABLE reputation_event (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    dimension reputation_dimension NOT NULL,
    delta INTEGER NOT NULL,
    source_case_id INTEGER REFERENCES moderation_case (id) ON DELETE SET NULL,
    source_report_id INTEGER,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_reputation_event_person_community_created ON reputation_event (person_id, community_id, created_at);

CREATE TABLE reputation_snapshot (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    reporting_accuracy INTEGER NOT NULL DEFAULT 0,
    jury_reliability INTEGER NOT NULL DEFAULT 0,
    participation_consistency INTEGER NOT NULL DEFAULT 0,
    endorsement_strength INTEGER NOT NULL DEFAULT 0,
    jury_eligible BOOLEAN NOT NULL DEFAULT FALSE,
    trusted_reporter BOOLEAN NOT NULL DEFAULT FALSE,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (person_id, community_id)
);
```

**IMPLEMENT** — `down.sql`:

```sql
DROP TABLE reputation_snapshot;
DROP INDEX IF EXISTS idx_reputation_event_person_community_created;
DROP TABLE reputation_event;
DROP TABLE endorsement;
DROP TABLE surety;
```

**GOTCHA**: `reputation_event.source_report_id` is a bare integer with **no FK**. [04 §13](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) says "no `report` table — collapse into `moderation_case`", so there's nothing to reference. Keep the column for forward-compatibility but no constraint.

**JUDGMENT CALL**: `reputation_snapshot` gets a `UNIQUE (person_id, community_id)` constraint even though [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) doesn't spell it out. Reason: a "snapshot" is definitionally one row per (person, community); without the constraint, repeated snapshot recalculation in Phase 5 will insert duplicates and break the single-row-read assumption. [OQ-001](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) ("per-community vs instance-wide") is deferred with "per-community in MVP" — the `community_id Option` + unique constraint handles both cases (NULL community_id for instance-wide rows).

**VALIDATE**: `.claude/build-task-05.log`.

**COMMIT**: `feat(migrations): add reputation and surety tables (phase-1 even though phase-5 uses them)`

---

### Task 6 — Migration: `add_actor_pseudonym`

**ACTION**: `migrations/{ts5}_add_actor_pseudonym/{up,down}.sql`.

**IMPLEMENT** — `up.sql`:

```sql
CREATE TABLE actor_pseudonym (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL UNIQUE REFERENCES person (id) ON DELETE CASCADE,
    pseudonym TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
```

**IMPLEMENT** — `down.sql`:

```sql
DROP TABLE actor_pseudonym;
```

**CRITICAL — do not re-litigate**:
- This table is **GDPR-mandatory** per [ADR-015](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md). Not "land later if we have time."
- The Rust helper `actor_pseudonym::get_or_create(person_id)` is deferred to **Phase 4** ([IMPLEMENTATION-PLAN-v0.md §4.2](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)). Do NOT write it here. Only the table + model (tasks 6 + 20-ish, see task 20 below).
- Pseudonym generation is cryptographically random UUIDv4, **enforced in Rust, not in SQL** ([04 §3 ActorPseudonym rules](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md)). The migration stores `pseudonym TEXT` — Rust inserts a UUIDv4 string.
- `person_id UNIQUE` means one pseudonym per person at a time. A GDPR delete of the row is followed by a NEW insert (new pseudonym) if the person acts again — per [04 §3 ActorPseudonym rules](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) and [§4.2](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): _"A deleted pseudonym is **not replaced** — subsequent actions by the same user generate a new pseudonym."_

**GOTCHA**: The `ON DELETE CASCADE` on `person_id` means when a Lemmy user is hard-deleted, their pseudonym row vanishes. That's correct behaviour: the mapping dies with the person. But it does NOT retroactively clean governance_log entries — the log still has the `actor_pseudonym` string, it just no longer maps back. That's exactly the ADR-015 property we want.

**VALIDATE**: `.claude/build-task-06.log`.

**COMMIT**: `feat(migrations): add actor_pseudonym table (ADR-015 GDPR mandate)`

---

### Task 7 — Migration: `add_governance_log` (table only, trigger in task 8)

**ACTION**: `migrations/{ts6}_add_governance_log/{up,down}.sql`. This migration does **only** the table, the pgcrypto extension, and the DB grants. The hash-chain trigger is a replaceable-schema artefact (task 8) and does NOT live in a migration.

**IMPLEMENT** — `up.sql`:

```sql
CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE governance_log (
    id BIGSERIAL PRIMARY KEY,
    prev_hash BYTEA NOT NULL,
    entry_hash BYTEA NOT NULL,
    entry_kind TEXT NOT NULL,
    payload JSONB NOT NULL,
    actor_pseudonym TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    signature BYTEA
);

CREATE INDEX idx_governance_log_created_at ON governance_log (created_at);
CREATE INDEX idx_governance_log_entry_kind ON governance_log (entry_kind);
```

**IMPLEMENT** — `down.sql`:

```sql
DROP INDEX IF EXISTS idx_governance_log_entry_kind;
DROP INDEX IF EXISTS idx_governance_log_created_at;
DROP TABLE governance_log;
-- DO NOT DROP pgcrypto — other code may use it, and IF NOT EXISTS made up.sql safe to re-run
```

**CRITICAL**:
- `id` is `BIGSERIAL` / `i64` per [§4.1 governance_log](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) — the log may grow very large over time (`IMPLEMENTATION-PLAN-v0.md §7.2` notes this explicitly).
- `prev_hash` and `entry_hash` are `BYTEA NOT NULL`. They are computed by the trigger (task 8), so the Rust `InsertForm` does NOT supply them — see task 20.
- `payload JSONB` — the governance log stores structured events as JSON. Phase 4's `governance_log::append` helper serialises event-kind-specific structs.
- `actor_pseudonym TEXT` (nullable) — system events have no actor. Populated by Rust helper in Phase 4 via `actor_pseudonym::get_or_create(person_id).pseudonym` — **never from `person.name` / `person.ap_id` / `person.id`**.
- `signature BYTEA` (nullable) — populated after INSERT by a second UPDATE in Phase 4; the single-shot update gate (task 8 trigger 2) enforces `NULL → Some` exactly once.

**GRANT/REVOKE — RESOLVED (advisor-review pass 1, 2026-04-15)**: [§4.1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) lists "DB grant that denies UPDATE/DELETE to the app role" alongside the trigger, as a belt-and-braces second layer. **Phase 1 deliberately does NOT add this grant.** The trigger layer (task 8) is the correctness gate; it fully enforces append-only semantics and is tested in task 14. The grant layer requires introducing a dedicated `governance_log_writer` role that only Phase 4's `governance_log::append` helper runs under — that role is designed holistically alongside the writer, in Phase 4. **Do NOT re-raise this in any ralph iteration of Phase 1.** If you find yourself wanting to add a `REVOKE` here, stop: it's a Phase 4 concern and adding it in Phase 1 would block the Phase 4 signature-UPDATE path.

**GOTCHA**: `CREATE EXTENSION pgcrypto` is idempotent (`IF NOT EXISTS`) and safe in a transaction on Postgres 16. If the local dev DB already has pgcrypto from another project, this is a no-op.

**VALIDATE**: `.claude/build-task-07.log`.

**COMMIT**: `feat(migrations): add append-only governance_log table with pgcrypto`

---

### Task 8 — Hash-chain + append-only triggers in replaceable_schema

**ACTION**: Append to `crates/diesel_utils/replaceable_schema/triggers.sql`. Three PL/pgSQL functions in the `r` schema + three `CREATE TRIGGER` statements on `public.governance_log`.

**IMPLEMENT** (append at the end of `triggers.sql`):

```sql
-- === Governance log hash chain (Phase 1) ==================================
-- The governance_log table is append-only and hash-chained. Three triggers
-- enforce it:
--   1. BEFORE INSERT  : compute prev_hash from the most recent row and
--                       entry_hash = sha256(prev_hash || entry_kind || payload_bytes || created_at_bytes).
--   2. BEFORE UPDATE  : allow exactly one transition — NULL signature → non-null
--                       signature. Any other update raises an exception.
--   3. BEFORE DELETE  : always raise.
--
-- The FUNCTIONS live in schema `r` (so they are dropped when `r` is dropped
-- on schema rebuild). The TRIGGERS attach to `public.governance_log`
-- (because triggers are table-scoped); they are dropped transitively when
-- the functions they EXECUTE are dropped, per the rule at the top of this
-- file: "dropping the function drops the trigger."
-- ========================================================================

CREATE FUNCTION r.governance_log_hash_chain_before_insert ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
DECLARE
    last_hash bytea;
BEGIN
    -- Read the most recent entry_hash. If the table is empty, use 32 zero bytes.
    SELECT entry_hash INTO last_hash
      FROM public.governance_log
      ORDER BY id DESC
      LIMIT 1;
    IF last_hash IS NULL THEN
        last_hash := decode('0000000000000000000000000000000000000000000000000000000000000000', 'hex');
    END IF;
    NEW.prev_hash := last_hash;
    -- entry_hash = sha256(prev_hash || entry_kind || payload::text || created_at::text)
    NEW.entry_hash := digest(
        NEW.prev_hash
        || convert_to(NEW.entry_kind, 'UTF8')
        || convert_to(NEW.payload::text, 'UTF8')
        || convert_to(to_char(NEW.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"'), 'UTF8'),
        'sha256'
    );
    -- signature is always NULL at insert time; Phase 4 fills it via UPDATE
    NEW.signature := NULL;
    RETURN NEW;
END;
$$;

CREATE TRIGGER governance_log_hash_chain
    BEFORE INSERT ON public.governance_log
    FOR EACH ROW
    EXECUTE FUNCTION r.governance_log_hash_chain_before_insert ();

CREATE FUNCTION r.governance_log_signature_gate_before_update ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    -- Only the signature column may change, and only the NULL→non-NULL transition.
    IF OLD.id IS DISTINCT FROM NEW.id
       OR OLD.prev_hash IS DISTINCT FROM NEW.prev_hash
       OR OLD.entry_hash IS DISTINCT FROM NEW.entry_hash
       OR OLD.entry_kind IS DISTINCT FROM NEW.entry_kind
       OR OLD.payload IS DISTINCT FROM NEW.payload
       OR OLD.actor_pseudonym IS DISTINCT FROM NEW.actor_pseudonym
       OR OLD.created_at IS DISTINCT FROM NEW.created_at
    THEN
        RAISE EXCEPTION 'governance_log is append-only: only signature may be updated'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF OLD.signature IS NOT NULL THEN
        RAISE EXCEPTION 'governance_log.signature is write-once and already set'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    IF NEW.signature IS NULL THEN
        RAISE EXCEPTION 'governance_log.signature must be set, not cleared'
            USING ERRCODE = 'integrity_constraint_violation';
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER governance_log_signature_gate
    BEFORE UPDATE ON public.governance_log
    FOR EACH ROW
    EXECUTE FUNCTION r.governance_log_signature_gate_before_update ();

CREATE FUNCTION r.governance_log_append_only_before_delete ()
    RETURNS TRIGGER
    LANGUAGE plpgsql
    AS $$
BEGIN
    RAISE EXCEPTION 'governance_log is append-only: rows may not be deleted'
        USING ERRCODE = 'integrity_constraint_violation';
END;
$$;

CREATE TRIGGER governance_log_no_delete
    BEFORE DELETE ON public.governance_log
    FOR EACH ROW
    EXECUTE FUNCTION r.governance_log_append_only_before_delete ();
```

**MIRROR**: `crates/diesel_utils/replaceable_schema/triggers.sql:365-380` (the `r.post_change_values()` / `CREATE TRIGGER change_values BEFORE INSERT ON post` pattern). The file-top comment block at lines 1–16 spells out the lifecycle rule.

**CRITICAL**:
- Functions go in `r.*`. Triggers attach to `public.governance_log`. This matches the rule at `triggers.sql:1-3`.
- The runner at `crates/diesel_utils/src/schema_setup/mod.rs:219-246` drops the `r` schema with `CASCADE` on every schema change and reruns `replaceable_schema()`. Our triggers are recreated each time — idempotent by design.
- The runner also runs a **diff-check in test mode** (`crates/diesel_utils/src/schema_setup/mod.rs:229-244`) that asserts the code in `replaceable_schema/` does NOT modify anything outside `r.*`. Triggers on public tables are the ONE exception that Lemmy carves out (see comment at `triggers.sql:1-16`): triggers are tied to their function, so when `r.*` gets dropped `CASCADE`, the triggers die with it. Our three triggers follow the exact same lifecycle.
- `digest()` comes from `pgcrypto` (migration 7). It returns `bytea`.
- The timestamp serialisation format is `ISO-8601 microseconds UTC` — matches what Rust's `chrono::DateTime<Utc>::to_rfc3339_opts(SecondsFormat::Micros, true)` produces. Needed so the Rust `governance_log_hash_chain_holds` test in task 13 can recompute the chain bit-for-bit.

**GOTCHA**:
- The `convert_to(..., 'UTF8')` calls explicitly flatten text + JSONB into `bytea`. Without them, `||` on `bytea` + `text` is a type error in Postgres.
- `to_char` is used because `created_at::text` on `TIMESTAMPTZ` formats differently across client time zones. Forcing UTC + ISO format makes the hash deterministic.
- **Do not forget `AT TIME ZONE 'UTC'`** — without it, the hash depends on the session `timezone` setting.

**JUDGMENT CALL**: The single-shot signature update gate is Option A from [§4.1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): "one update allowed when signature IS NULL." I'm enforcing this at the trigger level rather than a DB grant because a grant can only say "no UPDATE" or "any UPDATE" — it can't express "one particular column, one time." The trigger is the precise gate [§4.1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) calls for. [§7.1 fallback](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) (side-table keyed by `entry_hash`) is NOT used here — Option A is the committed path.

**VALIDATE**:
```bash
./scripts/brehon/cargo-check.bat --workspace > .claude/build-task-08.log 2>&1
status=$?
tail -40 .claude/build-task-08.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```
(`cargo check` does NOT exercise the trigger — that happens in task 13's integration test. For this task we're just confirming nothing in the workspace broke from editing the `replaceable_schema/triggers.sql` file, which is loaded by `include_str!` at `schema_setup/mod.rs:48`.)

**COMMIT**: `feat(db): governance_log hash-chain and append-only triggers in replaceable schema`

---

### Task 9 — Governance enums in `db_schema_file/src/enums.rs`

**ACTION**: Append 11 new enums to `crates/db_schema_file/src/enums.rs`. Each enum must mirror the existing pattern **exactly** (see `RegistrationMode` at lines 75–94 for the canonical shape).

**IMPLEMENT** (append at end of `enums.rs`, with a section banner comment):

```rust
// ========================================================================
// Governance enums (Phase 1)
// Each mirrors the DbEnum / ExistingTypePath / DbValueStyle="verbatim" pattern
// used by the existing enums above (see e.g. RegistrationMode).
// The Postgres enum types are created in migrations/{ts}_add_governance_enums.
// Variant names are PascalCase to match DbValueStyle="verbatim".
// ========================================================================

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Default, Hash)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "full", derive(DbEnum))]
#[cfg_attr(
  feature = "full",
  ExistingTypePath = "crate::schema::sql_types::CaseStatus"
)]
#[cfg_attr(feature = "full", DbValueStyle = "verbatim")]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
/// Lifecycle of a governance moderation case. See [04 §2](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md).
pub enum CaseStatus {
  #[default]
  Open,
  ThresholdMet,
  JurySelection,
  InReview,
  Decided,
  Appealed,
  Closed,
  /// Admin invoked the emergency-remove override. A jury reviews post-facto;
  /// the removal stands regardless of the jury's finding. Per ADR-013.
  EmergencyRemove,
  /// A direct moderator action (Lemmy compat layer) paused the case. Admin must
  /// explicitly resume. Per OQ-008 resolution.
  AdminReview,
}

// ... then 10 more enums following the same pattern, one per type in migration 2:
// CaseTargetType, CaseSeverity, EvidenceVisibility, JuryAssignmentStatus,
// JuryDecision, SanctionScope, SanctionAction, AppealStatus,
// ReputationDimension, AttestationType.
```

(The full body for the other 10 enums follows the same macro block shape and mirrors [04 §2](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) variant-for-variant. I'm not expanding each here because the structure is mechanical — the implementation agent copy-pastes `RegistrationMode` and edits.)

**MIRROR**: `crates/db_schema_file/src/enums.rs:75-94` (`RegistrationMode` — the cleanest existing example).

**CRITICAL**:
- `CaseStatus` **MUST** include `EmergencyRemove` and `AdminReview`. No `_ =>` wildcard in any future match (workspace lint `unreachable = "deny"` helps catch this, but the real enforcement is exhaustive-match patterns in Phase 4+ handlers).
- `#[default]` picks `Open` — every case starts `Open`.
- The `ExistingTypePath = "crate::schema::sql_types::CaseStatus"` is what Diesel uses to resolve the Rust enum to the Postgres enum type. The Postgres type name is the lowercase `case_status` (from task 2); `diesel print-schema` generates the Rust `sql_types::CaseStatus` wrapper type (note: `CamelCase` in Rust, `snake_case` in Postgres) — task 10 regenerates `schema.rs` to pick these up.
- Match [04 §2](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) variant order exactly so Postgres enum ordering matches Rust enum ordering. (Postgres enum order affects `ORDER BY`; Rust order affects `Ord` derive. Keeping them aligned avoids foot-guns in later queries.)

**VALIDATE**: First run `cargo check -p lemmy_db_schema_file` — this will **FAIL** because `schema::sql_types::CaseStatus` doesn't exist yet. That's expected — task 10 regenerates `schema.rs` to create those types. Capture and read the log to confirm the error is specifically "cannot find type `CaseStatus` in module `sql_types`" and nothing else:

```bash
./scripts/brehon/cargo-check.bat -p lemmy_db_schema_file > .claude/build-task-09.log 2>&1
status=$?
tail -60 .claude/build-task-09.log
echo "exit=$status"
# Expected: failure citing missing sql_types::CaseStatus etc. If ANY other error appears, stop.
```

The implementation agent should treat this task as "the only error should be the expected `sql_types::*` gap"; if anything else breaks, STOP and investigate. Task 10 will green this.

**COMMIT**:
```
feat(db-schema-file): add governance enums (case_status, jury_decision, sanction_action, etc.)

This commit intentionally fails `cargo check -p lemmy_db_schema_file` — the
new enums reference `crate::schema::sql_types::CaseStatus` etc. which do not
exist until task 10 regenerates schema.rs via `diesel print-schema`. Task 10
greens this.

Split into its own commit (rather than squashed with task 10) so git history
shows the known interim failure mode deterministically — readers diffing
should not misread this as a genuinely broken commit.
```

---

### Task 10 — Regenerate `crates/db_schema_file/src/schema.rs`

**PRE-FLIGHT: verify diesel_cli is operational in the vcvars environment.** libpq landed in Phase 0 (commit `e370523c7`), but `diesel_cli` was likely installed *before* libpq was on PATH and may be a silently-broken binary. From a cmd.exe shell (or via the cargo-test wrapper which sources vcvars), run:

```bash
./scripts/brehon/cargo-test.bat diesel --version
```

If this fails with a DLL-not-found error, a `libpq.dll` error, or an `LNK1181` style link failure, **reinstall `diesel_cli` from inside the same vcvars shell** so the build-time libpq linkage is fresh:

```bash
# From inside a cmd.exe that has already sourced vcvars64.bat
# (or run scripts\brehon\cargo-test.bat with no args just to source vcvars,
#  then run cargo install in the same shell):
cargo install diesel_cli --no-default-features --features postgres --force
```

The `--force` ensures the old binary is replaced. Re-run `diesel --version` afterwards and confirm it prints a version string. **If it still fails after reinstall, STOP and report** — that means libpq + vcpkg wiring needs investigation, which is out of scope for ralph to handle automatically.

**ACTION**: Start a local Postgres, apply all migrations including tasks 2–7, then run `diesel print-schema --patch-file crates/db_schema_file/diesel_ltree.patch > crates/db_schema_file/src/schema.rs`. The output is a fully regenerated `schema.rs` that now contains:
- New `sql_types::CaseStatus`, `sql_types::CaseTargetType`, … (one per enum from task 2)
- New `diesel::table! { ... }` blocks for each governance table
- `allow_tables_to_appear_in_same_query!` macro expanded to include the new tables (driven by `diesel.toml`'s `allow_tables_to_appear_in_same_query_config = "fk_related_tables"` — it generates pairs based on FK edges, so governance tables joined by FKs will be enabled automatically)

Because this regeneration must run against a live DB, do it from the testcontainers harness OR from a throwaway docker run. **Do NOT use `diesel migration run`** — migration `2025-08-01-000017_forbid_diesel_cli` installs a trigger on `__diesel_schema_migrations` that rejects any insert not protected by `pg_advisory_lock(0)`, which raw `diesel migration run` does not take. The Lemmy-native runner (`crates/diesel_utils/src/main.rs`) wraps `lemmy_diesel_utils::schema_setup::run(Options::default().run(), ...)` which calls `SELECT pg_advisory_lock(0);` before running migrations and also runs `replaceable_schema()` (installing the governance triggers from task 8). Use that binary instead.

```bash
# Start a throwaway Postgres matching Lemmy's prod image
docker run --rm -d --user $(id -u):$(id -g) \
  --name pg-schema-gen \
  -e POSTGRES_USER=lemmy \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=lemmy \
  -p 5433:5432 \
  pgautoupgrade/pgautoupgrade:18-alpine
# Wait for ready, then run migrations via the native runner (NOT `diesel migration run`)
export LEMMY_DATABASE_URL=postgres://lemmy:password@localhost:5433/lemmy
cargo run -p lemmy_diesel_utils --features full
# Regenerate schema (print-schema is read-only, no forbid-trigger interaction)
export DATABASE_URL=postgres://lemmy:password@localhost:5433/lemmy
diesel print-schema --patch-file crates/db_schema_file/diesel_ltree.patch > crates/db_schema_file/src/schema.rs
# Clean up
docker stop pg-schema-gen
```

**Why `lemmy_diesel_utils` instead of `diesel migration run`**: migration `2025-08-01-000017_forbid_diesel_cli` installs a trigger that raises `'migrations must be managed using lemmy_server instead of diesel CLI'` on any insert to `__diesel_schema_migrations` unless `pg_advisory_lock(0)` is held. The `lemmy_diesel_utils` binary (wrapping `schema_setup::run`) takes that lock at `schema_setup/mod.rs:214` and also runs `replaceable_schema()` afterward — both required for Phase 1 because the governance hash-chain triggers live in `replaceable_schema/triggers.sql` and must be installed before `diesel print-schema` or the task-14 test runs. Raw `diesel migration run` does neither.

**`--user $(id -u):$(id -g)`** is per [IMPLEMENTATION-PLAN-v0.md §5.1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) test-strategy note — avoids root-owned volumes blocking worktree cleanup on Linux/macOS. On Windows (where we run) the `$(id -u)` expansion is bash-only and the Docker Desktop backend doesn't enforce file ownership the same way, but the flag is harmless and preserves cross-platform behaviour.

**MIRROR**: `diesel.toml:1-6` + `crates/db_schema_file/diesel_ltree.patch` for the existing generation pipeline. The `schema.rs` file has a `// @generated automatically by Diesel CLI.` header — do not hand-edit; only regenerate.

**CRITICAL**:
- The `--patch-file` flag is **mandatory** — `diesel_ltree.patch` adds the `use diesel_ltree::sql_types::Ltree;` import that the `comment` table depends on. Without it, `schema.rs` won't compile.
- **Windows libpq is already installed** (Phase 0 commit `e370523c7` wired vcpkg's libpq into `scripts/brehon/cargo-test.bat`). Run the `diesel` CLI from inside cmd.exe after sourcing vcvars, or pipe the invocation through `scripts/brehon/cargo-test.bat` which sources vcvars for you. See the PRE-FLIGHT block above for the `diesel_cli` version check and reinstall steps if the binary is stale.
- **Fallback if libpq wiring regresses**: spawn a container via the testcontainers test harness itself, `docker exec` `diesel migration run` inside the container (which has libpq natively), then `docker cp` the regenerated `schema.rs` back out. Messier but keeps the dev loop working. **Default path**: local `diesel` CLI via `scripts/brehon/cargo-test.bat`.

**VALIDATE**:
```bash
./scripts/brehon/cargo-check.bat -p lemmy_db_schema_file > .claude/build-task-10a.log 2>&1
status=$?
tail -60 .claude/build-task-10a.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
# Now task 9's enums should resolve against the new sql_types
./scripts/brehon/cargo-check.bat -p lemmy_db_schema > .claude/build-task-10b.log 2>&1
status=$?
tail -60 .claude/build-task-10b.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```

**GOTCHA**: If `cargo check -p lemmy_db_schema` passes but `--workspace` fails, a dependent crate is using one of the governance tables with a join config the generated `schema.rs` didn't enable. In that case, either (a) add an explicit `allow_tables_to_appear_in_same_query!(...)` block in the patch file, or (b) accept the join needs `joinable!` and update `joins.rs` manually (governance joins would go in a new `crates/db_schema_file/src/joins.rs` section — but this is Phase 2 work; Phase 1 should not hit this).

**COMMIT**: `feat(db-schema-file): regenerate schema.rs with governance tables and enum sql_types`

---

### Task 11 — Governance ID newtypes in `crates/db_schema/src/newtypes.rs`

**ACTION**: Append `DieselNewType` wrapper structs for each governance-internal integer ID. 14 newtypes total (one per table). Existing file pattern is visible in `crates/db_schema_file/src/lib.rs:29-40` (e.g. `PostId`, `CommentId`, etc.) and governance-facing newtypes live in `crates/db_schema/src/newtypes.rs` — append there.

**IMPLEMENT**:

```rust
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct ModerationCaseId(pub i32);

// ... 12 more for case_evidence, sanction, appeal, public_case_log,
// jury_pool, jury_assignment, jury_vote, surety, endorsement,
// reputation_event, reputation_snapshot, actor_pseudonym

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "full", derive(DieselNewType))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(export))]
pub struct GovernanceLogId(pub i64);  // BIGSERIAL → i64
```

**MIRROR**: Existing Lemmy newtypes like `PostReportId` in the `crates/db_schema/src/newtypes.rs` file or `PersonId` in `crates/db_schema_file/src/lib.rs:29-40`.

**CRITICAL**:
- `GovernanceLogId(pub i64)` — matches `BIGSERIAL` from the migration. Every other governance ID is `i32` / `SERIAL`.
- Using typed IDs means `ModerationCase.creator_id: PersonId` not `i32` — impossible to accidentally pass a `CommunityId` where a `PersonId` was expected.

**VALIDATE**:
```bash
./scripts/brehon/cargo-check.bat -p lemmy_db_schema > .claude/build-task-11.log 2>&1
status=$?
tail -40 .claude/build-task-11.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```

**COMMIT**: `feat(db-schema): add typed IDs for governance tables (ModerationCaseId, JuryAssignmentId, etc.)`

---

### Task 12 — Diesel source models (14 files under `crates/db_schema/src/source/governance/`)

**ACTION**: Create the new directory `crates/db_schema/src/source/governance/` with 14 files: `mod.rs` + one per table (including `governance_log.rs`). Each file contains a `Queryable`/`Selectable`/`Identifiable` struct and a separate `Insertable`/`AsChangeset` form struct.

**FILES TO CREATE**:
1. `crates/db_schema/src/source/governance/mod.rs` — `pub mod moderation_case; pub mod case_evidence; …` (see §4 above)
2. `crates/db_schema/src/source/governance/moderation_case.rs`
3. `crates/db_schema/src/source/governance/case_evidence.rs`
4. `crates/db_schema/src/source/governance/sanction.rs`
5. `crates/db_schema/src/source/governance/appeal.rs`
6. `crates/db_schema/src/source/governance/public_case_log.rs`
7. `crates/db_schema/src/source/governance/jury_pool.rs`
8. `crates/db_schema/src/source/governance/jury_assignment.rs`
9. `crates/db_schema/src/source/governance/jury_vote.rs`
10. `crates/db_schema/src/source/governance/surety.rs`
11. `crates/db_schema/src/source/governance/endorsement.rs`
12. `crates/db_schema/src/source/governance/reputation_event.rs`
13. `crates/db_schema/src/source/governance/reputation_snapshot.rs`
14. `crates/db_schema/src/source/governance/actor_pseudonym.rs`
15. `crates/db_schema/src/source/governance/governance_log.rs`

**MODIFY**: `crates/db_schema/src/source/mod.rs` — add `pub mod governance;` after the existing `pub mod modlog;` line (alphabetical placement: `governance` sits after `federation_queue_state` and before `images`).

**IMPLEMENT** (example — `moderation_case.rs`, full file):

```rust
use crate::newtypes::{CommentId, CommunityId, ModerationCaseId, PostId};
use chrono::{DateTime, Utc};
use lemmy_db_schema_file::{
  PersonId,
  enums::{CaseSeverity, CaseStatus, CaseTargetType},
};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::moderation_case;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable)
)]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// A governance moderation case — the central artefact of the jury workflow.
pub struct ModerationCase {
  pub id: ModerationCaseId,
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub target_type: CaseTargetType,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_person_id: Option<PersonId>,
  pub target_community_id: Option<CommunityId>,
  pub target_remote_url: Option<String>,
  pub reason_code: String,
  pub severity: CaseSeverity,
  pub status: CaseStatus,
  pub threshold_score: i64,
  pub opened_at: DateTime<Utc>,
  pub updated_at: Option<DateTime<Utc>>,
  pub decided_at: Option<DateTime<Utc>>,
  pub closed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable, AsChangeset))]
#[cfg_attr(feature = "full", diesel(table_name = moderation_case))]
pub struct ModerationCaseInsertForm {
  pub community_id: Option<CommunityId>,
  pub creator_id: Option<PersonId>,
  pub target_type: CaseTargetType,
  pub target_post_id: Option<PostId>,
  pub target_comment_id: Option<CommentId>,
  pub target_person_id: Option<PersonId>,
  pub target_community_id: Option<CommunityId>,
  pub target_remote_url: Option<String>,
  pub reason_code: String,
  pub severity: CaseSeverity,
  pub status: CaseStatus,
  pub threshold_score: i64,
}
```

(The pattern repeats for the other 12 tables. The `GovernanceLog` file is an exception — see next block.)

**IMPLEMENT** (special case — `governance_log.rs`, full file):

```rust
use crate::newtypes::GovernanceLogId;
use chrono::{DateTime, Utc};
#[cfg(feature = "full")]
use lemmy_db_schema_file::schema::governance_log;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]
#[cfg_attr(
  feature = "full",
  derive(Identifiable, Queryable, Selectable)
)]
#[cfg_attr(feature = "full", diesel(table_name = governance_log))]
#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]
/// Append-only, hash-chained governance log entry.
///
/// `prev_hash`, `entry_hash`, and `signature` are **trigger-populated**:
/// `prev_hash` and `entry_hash` are filled by
/// `r.governance_log_hash_chain_before_insert()` at INSERT time; `signature`
/// is filled by a single-shot UPDATE in Phase 4, gated by
/// `r.governance_log_signature_gate_before_update()`. Callers must never set
/// these three fields directly — construct rows via
/// [`GovernanceLogInsertForm`] and let the trigger layer do its job. The
/// only sanctioned write path (from Phase 4 onward) is the helper in
/// `crates/api/api/src/governance/governance_log.rs`.
pub struct GovernanceLog {
  pub id: GovernanceLogId,
  pub prev_hash: Vec<u8>,
  pub entry_hash: Vec<u8>,
  pub entry_kind: String,
  pub payload: Value,
  pub actor_pseudonym: Option<String>,
  pub created_at: DateTime<Utc>,
  pub signature: Option<Vec<u8>>,
}

#[derive(Clone, Default)]
#[cfg_attr(feature = "full", derive(Insertable))]
#[cfg_attr(feature = "full", diesel(table_name = governance_log))]
/// **Do not populate `prev_hash`, `entry_hash`, or `signature` — they are
/// trigger-managed.** Phase 4's helper wraps this form and is the only
/// sanctioned insert path.
pub struct GovernanceLogInsertForm {
  pub entry_kind: String,
  pub payload: Value,
  pub actor_pseudonym: Option<String>,
  // `prev_hash` and `entry_hash` are NOT NULL in the DB but Diesel's
  // Insertable skips fields omitted from the form. The trigger
  // r.governance_log_hash_chain_before_insert assigns both from NEW before
  // the row reaches the heap. `signature` is NULL on insert and filled via
  // a separate UPDATE in Phase 4.
}
```

**CRITICAL DESIGN DECISION** — `GovernanceLogInsertForm` OMITS `prev_hash` and `entry_hash`. Diesel's `Insertable` derive builds an INSERT column list from the form's fields; omitted fields aren't in the column list, so Postgres sees them as missing → trigger populates `NEW.prev_hash` + `NEW.entry_hash` → `NOT NULL` constraint is satisfied. **This is the Option-A trigger flow described in [§4.1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md).** Verify by running task 13's hash-chain test — if the INSERT fails with `null value in column "prev_hash"`, the trigger didn't fire (the form contains `prev_hash` by accident, or the trigger creation failed at schema rebuild time).

**MIRROR**:
- Primary: `crates/db_schema/src/source/post_report.rs` (full file) for the Queryable+InsertForm pair.
- Secondary: `crates/db_schema/src/source/community_report.rs` for alternative `#[diesel(belongs_to(...))]` examples if FK associations are needed in Phase 2.

**GOTCHA**:
- **Typed-ID imports are split across two crates.** `PersonId` lives in `lemmy_db_schema_file::PersonId` (see `crates/db_schema_file/src/lib.rs:34`). But `CommunityId`, `PostId`, `CommentId`, `PostReportId`, etc. live in `lemmy_db_schema::newtypes` (see `crates/db_schema/src/newtypes.rs:11,24,42`). Governance newtypes from task 11 also land in `db_schema::newtypes`. **Governance source files must import from both:** `use crate::newtypes::{CommunityId, PostId, CommentId, ModerationCaseId, ...};` plus `use lemmy_db_schema_file::PersonId;`.
- `Value` from `serde_json` for `JSONB` columns — ensure `serde_json` is already a `db_schema` dep (it is; see `crates/db_schema/Cargo.toml`).
- `Vec<u8>` for `BYTEA` columns — Diesel default mapping.
- `#[diesel(belongs_to(...))]` is required only where Phase 2 views will build joins via `Associations` derive. Phase 1 keeps them OFF to avoid pulling in view-layer dependencies prematurely.

**NEW JUDGMENT CALL**: I added an `updated_at: Option<DateTime<Utc>>` field to `ModerationCase` that's NOT in [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md). **Reverting this — remove `updated_at` from both the struct and the migration**. [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) is authoritative; don't silently add fields. See "Judgment calls" in the report section below.

**VALIDATE**:
```bash
./scripts/brehon/cargo-check.bat -p lemmy_db_schema > .claude/build-task-12.log 2>&1
status=$?
tail -80 .claude/build-task-12.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```

**COMMIT**: `feat(db-schema): diesel models for governance tables (moderation_case, jury, reputation, log)`

---

### Task 13 — Integration test: `can_insert_moderation_case`

**ACTION**: Extend `crates/server/tests/e2e.rs` with a second test. Add a small helper module that:
1. Starts the pgautoupgrade container (reuse the Phase 0 pattern)
2. Opens a sync `diesel::PgConnection`
3. Runs `diesel_migrations::embed_migrations!("../../migrations")` via `MigrationHarness::run_pending_migrations()` (this also triggers the replaceable-schema rebuild via the harness wrapper in `crates/diesel_utils/src/schema_setup/mod.rs`, but only if the server composition pulls that code in — for a raw diesel test we use the upstream `diesel_migrations` harness directly and call `r` schema creation manually if needed)

Wait — **research check**: does `diesel_migrations::embed_migrations!` also trigger the replaceable-schema rerun, or only the raw migrations? Reading `crates/diesel_utils/src/schema_setup/mod.rs:217-256`, the `run_replaceable_schema()` step is part of the `MigrationHarnessWrapper` and the schema-setup function, not baked into `diesel_migrations` itself. So if the e2e test bypasses `lemmy_diesel_utils::schema_setup` and uses `diesel_migrations::MigrationHarness` directly, the replaceable-schema step (`r.governance_log_hash_chain_before_insert` etc.) will NOT run — only the raw migrations run.

**DECISION**: Use `lemmy_diesel_utils::schema_setup::run(Options::default())` so the replaceable-schema rebuild runs too. Task 13's helper takes the DATABASE_URL from the container and hands it to the Lemmy runner.

Reading further: `schema_setup::run()` takes `conn: &mut PgConnection` and an `Options` struct. Need to verify the exact public API — if it's private, expose a minimal helper that the test can call. **Phase 1 should NOT add a new public API surface.** Instead: use `diesel_migrations::embed_migrations!` for the raw migrations, then execute the `replaceable_schema/triggers.sql` + `utils.sql` contents directly via `batch_execute(include_str!(...))` inside the test helper.

Concrete plan:

```rust
// inside the test helper module in e2e.rs
const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
    diesel_migrations::embed_migrations!("../../migrations");

fn apply_all_schema(conn: &mut PgConnection) -> Result<(), Box<dyn Error>> {
    use diesel::connection::SimpleConnection;
    use diesel_migrations::MigrationHarness;
    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| -> Box<dyn Error> { format!("migrations failed: {e}").into() })?;
    // Apply the replaceable schema (r.*). Lemmy's production runner lives in
    // lemmy_diesel_utils::schema_setup, but for a raw test we inline the
    // two SQL files that schema_setup loads via include_str!.
    conn.batch_execute("DROP SCHEMA IF EXISTS r CASCADE; CREATE SCHEMA r;")?;
    conn.batch_execute(include_str!(
        "../../../crates/diesel_utils/replaceable_schema/utils.sql"
    ))?;
    conn.batch_execute(include_str!(
        "../../../crates/diesel_utils/replaceable_schema/triggers.sql"
    ))?;
    Ok(())
}
```

(Path relative to `crates/server/tests/e2e.rs`: `../../../crates/diesel_utils/replaceable_schema/{utils,triggers}.sql`.)

**IMPLEMENT** — test function (full body):

```rust
#[tokio::test]
async fn can_insert_moderation_case() -> Result<(), Box<dyn Error>> {
    use diesel::{Connection, PgConnection, RunQueryDsl};
    use lemmy_db_schema::source::governance::moderation_case::ModerationCaseInsertForm;
    use lemmy_db_schema_file::enums::{CaseSeverity, CaseStatus, CaseTargetType};
    use lemmy_db_schema_file::schema::moderation_case;

    let container = GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "lemmy")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "lemmy")
        .start()
        .await?;

    let host_port = container.get_host_port_ipv4(5432).await?;
    let db_url = format!("postgres://lemmy:password@localhost:{host_port}/lemmy");
    let mut conn = PgConnection::establish(&db_url)?;
    apply_all_schema(&mut conn)?;

    // A moderation_case can't FK to a person that doesn't exist. For Phase 1
    // we insert with NULL creator_id / community_id — the schema allows it.
    let form = ModerationCaseInsertForm {
        community_id: None,
        creator_id: None,
        target_type: CaseTargetType::RemoteInstance,
        target_post_id: None,
        target_comment_id: None,
        target_person_id: None,
        target_community_id: None,
        target_remote_url: Some("https://example.invalid/post/1".to_string()),
        reason_code: "spam".to_string(),
        severity: CaseSeverity::Low,
        status: CaseStatus::Open,
        threshold_score: 1,
    };

    let inserted_id: i32 = diesel::insert_into(moderation_case::table)
        .values(&form)
        .returning(moderation_case::id)
        .get_result(&mut conn)?;

    assert!(inserted_id > 0, "moderation_case id should be positive");

    // Round-trip read
    use diesel::QueryDsl;
    let read_back_status: CaseStatus = moderation_case::table
        .find(inserted_id)
        .select(moderation_case::status)
        .first(&mut conn)?;
    assert!(matches!(read_back_status, CaseStatus::Open));

    Ok(())
}
```

**CRITICAL — clippy compliance**:
- Function signature `-> Result<(), Box<dyn Error>>` — matches Phase 0 and `feedback_clippy_test_style.md` Pattern 2.
- All fallible calls use `?`. No `.unwrap()`, no `.expect()`.
- No `#[allow(clippy::...)]`.
- **`lemmy_db_schema` must be in `crates/server/Cargo.toml` `[dev-dependencies]`** (new dep, see Files to Change table). This introduces **libpq linking** in the e2e test binary — per `feedback_cargo_invocations.md`, the test must be run via `./scripts/brehon/cargo-test.bat`, never bare `cargo test` from Git Bash.

**MIRROR**: `crates/server/tests/e2e.rs:18-36` — same container setup. Imports follow the existing test.

**GOTCHA**:
- **Test isolation**: each test starts its own fresh container. The container drops when `container` goes out of scope (testcontainers-rs `AsyncRunner` cleanup). Multiple tests in one `cargo test` run start their containers in parallel, so name them distinctly if needed — but `testcontainers::GenericImage` auto-assigns distinct host ports, so parallel runs work out of the box.
- **Migration path discovery**: `embed_migrations!("../../migrations")` uses a path relative to the test file's crate root — for `crates/server/tests/e2e.rs` that's `crates/server/../../migrations` = repo-root `migrations/`. **Verify by running a trial compile once.** If the macro complains about the path, it may need to be `embed_migrations!("../../../migrations")` depending on whether the macro resolves from `tests/` or `src/`. Correct path is whatever makes `cargo check` green.
- **Foreign-key minimalism**: The test inserts with `NULL` `creator_id` and `NULL` `community_id` so it does NOT need to build the `instance → site → local_site → person` fixture chain that `crates/db_schema/src/test_data.rs::TestData::create` uses. Phase 2's view tests will need that chain; Phase 1 can stay minimal.

**VALIDATE**:
```bash
./scripts/brehon/cargo-test.bat --test e2e -p lemmy_server can_insert_moderation_case > .claude/build-task-13.log 2>&1
status=$?
tail -100 .claude/build-task-13.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```

Note: This is the **first** Phase-1 task that hits `cargo test`. If this fails with a libpq link error (LNK1181), the fix is in `feedback_cargo_invocations.md` — install libpq via vcpkg and set `PQ_LIB_DIR`. `cargo-test.bat` handles the sourcing automatically **once libpq is installed**.

**COMMIT**: `test(e2e): can_insert_moderation_case — migrations + diesel round-trip`

---

### Task 14 — Integration test: `governance_log_hash_chain_holds`

**ACTION**: Second new test in `crates/server/tests/e2e.rs`. Inserts three rows into `governance_log`, reads back `entry_hash` for each, recomputes the chain in Rust using `sha2::Sha256`, asserts stored hashes match. Also verifies the append-only trigger by attempting a DELETE and asserting it fails.

**IMPLEMENT** (append to `e2e.rs`):

```rust
#[tokio::test]
async fn governance_log_hash_chain_holds() -> Result<(), Box<dyn Error>> {
    use chrono::{DateTime, SecondsFormat, Utc};
    use diesel::{Connection, PgConnection, QueryDsl, RunQueryDsl};
    use lemmy_db_schema::source::governance::governance_log::{
        GovernanceLog, GovernanceLogInsertForm,
    };
    use lemmy_db_schema_file::schema::governance_log;
    use serde_json::json;
    use sha2::{Digest, Sha256};

    let container = GenericImage::new("pgautoupgrade/pgautoupgrade", "18-alpine")
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "lemmy")
        .with_env_var("POSTGRES_PASSWORD", "password")
        .with_env_var("POSTGRES_DB", "lemmy")
        .start()
        .await?;
    let host_port = container.get_host_port_ipv4(5432).await?;
    let db_url = format!("postgres://lemmy:password@localhost:{host_port}/lemmy");
    let mut conn = PgConnection::establish(&db_url)?;
    apply_all_schema(&mut conn)?;

    // Insert three entries
    let forms = [
        GovernanceLogInsertForm {
            entry_kind: "phase1.smoke.first".to_string(),
            payload: json!({ "n": 1 }),
            actor_pseudonym: Some("pseudo-alpha".to_string()),
        },
        GovernanceLogInsertForm {
            entry_kind: "phase1.smoke.second".to_string(),
            payload: json!({ "n": 2 }),
            actor_pseudonym: None,
        },
        GovernanceLogInsertForm {
            entry_kind: "phase1.smoke.third".to_string(),
            payload: json!({ "n": 3, "nested": [1, 2] }),
            actor_pseudonym: Some("pseudo-bravo".to_string()),
        },
    ];
    for form in &forms {
        diesel::insert_into(governance_log::table)
            .values(form)
            .execute(&mut conn)?;
    }

    // Read all three back in insertion order
    let rows: Vec<GovernanceLog> = governance_log::table
        .order(governance_log::id.asc())
        .load(&mut conn)?;
    assert_eq!(rows.len(), 3, "should have exactly 3 rows");

    // Recompute the chain in Rust and assert
    let mut prev: Vec<u8> = vec![0u8; 32];
    for row in &rows {
        assert_eq!(
            row.prev_hash, prev,
            "row {} prev_hash should match the previous row's entry_hash",
            row.id.0
        );
        let mut hasher = Sha256::new();
        hasher.update(&prev);
        hasher.update(row.entry_kind.as_bytes());
        hasher.update(serde_json::to_string(&row.payload)?.as_bytes());
        let created_at_str: String = row
            .created_at
            .with_timezone(&Utc)
            .to_rfc3339_opts(SecondsFormat::Micros, true);
        hasher.update(created_at_str.as_bytes());
        let expected = hasher.finalize().to_vec();
        assert_eq!(
            row.entry_hash, expected,
            "row {} entry_hash should match sha256(prev||kind||payload||ts)",
            row.id.0
        );
        prev = row.entry_hash.clone();
    }

    // Assert the append-only DELETE trigger fires
    let delete_result = diesel::delete(governance_log::table.find(rows[0].id)).execute(&mut conn);
    assert!(
        delete_result.is_err(),
        "delete from governance_log must be rejected by the append-only trigger"
    );

    // Assert the signature-gate UPDATE trigger rejects entry_kind changes
    use diesel::ExpressionMethods;
    let bad_update = diesel::update(governance_log::table.find(rows[0].id))
        .set(governance_log::entry_kind.eq("tampered"))
        .execute(&mut conn);
    assert!(
        bad_update.is_err(),
        "updating entry_kind must be rejected by the append-only trigger"
    );

    Ok(())
}
```

**MIRROR**: Task 13's test structure for the container boot + schema apply. The `include_str!` helper is reused from task 13.

**CRITICAL — hash determinism**:
- Rust `to_rfc3339_opts(SecondsFormat::Micros, true)` must produce the same string as Postgres `to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.US"Z"')`. Both produce `2026-04-15T14:23:45.123456Z`. **Verify this during task 13/14 implementation — if they don't match bit-for-bit, the test will fail and the implementation agent needs to adjust one side (prefer adjusting the Rust test to match PG exactly, since the PG output is what's actually chained into the next row's hash).**
- `serde_json::to_string(&row.payload)` must match Postgres `NEW.payload::text`. **Postgres may re-sort JSONB keys or normalise whitespace.** Work around this by inserting already-normalised JSON or by comparing canonicalised forms. **JUDGMENT CALL**: The cleanest path is to have Rust read back `row.payload` (which Diesel deserialises from the same JSONB Postgres stored) and round-trip it through `serde_json::to_string`. If Postgres normalised the input, Rust's deserialised payload will be in the normalised form, so the Rust recomputation matches. Verify during implementation. If it still doesn't match, switch the trigger's `NEW.payload::text` to `NEW.payload::jsonb::text` or canonicalise both sides.
- The test **intentionally** does not fill `signature` — tasks 1–14 don't wire Phase 4's signing helper. `signature` remains `NULL`. The signature-gate trigger fires only on UPDATE, so Phase 1's INSERT-only test path doesn't exercise it. The test's "update entry_kind must fail" line exercises the gate.

**GOTCHA**:
- **Postgres JSONB serialisation normalises object keys** (sorts them lexicographically on insert). If the trigger hashes `NEW.payload::text` and Rust re-serialises from the model struct, the two strings may differ in key order. Mitigate by using flat, key-stable payloads in the test (single top-level key like `"n"`). Deep/nested JSON in the third form is deliberate — it stress-tests the determinism. If that row's hash mismatches, reproduce locally and adjust the test's expectation, or adjust the trigger to normalise differently (e.g. cast JSONB to `jsonb` explicitly which produces a canonical form).
- **The delete-assertion** wraps a single statement. Diesel's connection may enter an error state after the failed delete; later statements may also fail until a rollback. If the test fails because the update-assertion errors on a "current transaction is aborted" state, wrap each bad-path assertion in its own `conn.transaction(|c| { ... })` so the rollback is local.

**VALIDATE**:
```bash
./scripts/brehon/cargo-test.bat --test e2e -p lemmy_server governance_log_hash_chain_holds > .claude/build-task-14.log 2>&1
status=$?
tail -120 .claude/build-task-14.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status

# Also run the whole phase-1 e2e test set as a gate
./scripts/brehon/cargo-test.bat --test e2e -p lemmy_server > .claude/build-task-14-all.log 2>&1
status=$?
tail -60 .claude/build-task-14-all.log
echo "exit=$status"
[ $status -eq 0 ] || exit $status
```

**COMMIT**: `test(e2e): governance_log_hash_chain_holds — sha256 chain + append-only trigger`

---

**Task count: 14 steps in the plan, which packages the 13 numbered tasks from IMPLEMENTATION-PLAN-v0.md §3 Phase 1** as follows:

| IMPLEMENTATION-PLAN-v0 §3 Phase 1 task | Plan task here |
|---|---|
| (workspace dep prereq) | Task 1 — `sha2` to workspace (new; not in main plan but required for task 13's sha2 dependency) |
| task 7 — `add_governance_enums` | Task 2 |
| task 1 — `add_governance_core` | Task 3 |
| task 2 — `add_jury_system` | Task 4 |
| task 3 — `add_reputation_and_surety` | Task 5 |
| task 4 — `add_actor_pseudonym` | Task 6 |
| task 5 — `add_governance_log` | Task 7 |
| task 6 — hash-chain trigger | Task 8 |
| task 8 — Diesel enum types | Task 9 |
| (schema.rs regen — prereq for task 9 resolving) | Task 10 — regen `schema.rs` |
| (typed IDs — prereq for task 12 using newtypes) | Task 11 — newtypes |
| task 9 — Diesel models core | Task 12 (covers tasks 9, 10, 11 from main plan together: all 14 models land in one commit's-worth of files because they must compile together — one cargo check gate) |
| task 10 — Diesel models jury | merged into Task 12 |
| task 11 — Diesel models reputation | merged into Task 12 |
| task 12 — smoke test | Task 13 |
| task 13 — hash-chain test | Task 14 |

That's 14 atomic commits ordered topologically. The main-plan's 13-tasks-mapping is preserved; I've added the `sha2` dependency prereq as task 1 and split the Diesel models from schema-regeneration (task 10) so the cargo check after task 9 has a known failure mode ("expected — awaiting task 10") rather than compounding confusion.

---

## Testing Strategy

Per [IMPLEMENTATION-PLAN-v0.md §5](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md): **integration-only**, all tests in `crates/server/tests/e2e.rs`, all use `-> Result<(), Box<dyn std::error::Error>>` with `?`.

### Tests Added in Phase 1

| Test Name | What It Validates | Task |
|---|---|---|
| `can_insert_moderation_case` | Migrations run to completion; `ModerationCaseInsertForm` → Postgres round-trip; `CaseStatus`/`CaseTargetType`/`CaseSeverity` enum marshalling works both directions | 13 |
| `governance_log_hash_chain_holds` | Postgres trigger computes `entry_hash` correctly for 3 rows; Rust recomputation of sha256 chain matches stored values; `DELETE` on governance_log raises; `UPDATE` on any non-signature column raises | 14 |

**Postgres image**: `pgautoupgrade/pgautoupgrade:18-alpine` (matches Phase 0; matches Lemmy's prod docker-compose).

**No mocks**: per `feedback_integration_tests_real_db` memory — tests hit a real Postgres in Docker.

**Migration round-trip is tested by the implementation agent manually** on a scratch DB during tasks 3–7 (`diesel migration run && diesel migration redo`) — there isn't an automated round-trip test in Phase 1. Adding one would be nice but belongs to a later "diff_check"-style test per `crates/diesel_utils/src/schema_setup/diff_check.rs`. Out of scope for Phase 1.

### Edge Cases Covered

- [x] Hash-chain integrity after 3 sequential inserts (task 14)
- [x] Append-only enforcement: DELETE rejected (task 14)
- [x] Append-only enforcement: non-signature UPDATE rejected (task 14)
- [x] Enum marshalling in both directions (task 13 reads status back)
- [x] Migration round-trip forward + backward (manual verification during each task's `diesel migration run && diesel migration redo`)
- [x] NULL-allowed FK paths (task 13 inserts with NULL `creator_id`/`community_id`)
- [ ] Hash chain with JSONB key-order variance — **JUDGMENT CALL**: intentionally not exhaustively tested in Phase 1. If it turns out brittle, Phase 4's `governance_log::append` helper canonicalises payloads to `BTreeMap`-backed JSON before serialising, solving the issue at the Rust layer.
- [ ] Signature UPDATE happy path — not tested in Phase 1 because Phase 4 adds the signer. The trigger is known-good for the refusal path (tested in task 14).

### Edge Cases NOT Covered (deliberately)

- **Rolling back migrations cleanly** is asserted only by eyeballing `diesel migration redo` during development, not by a Phase-1 test. A diff-check style test is a Phase 4+ improvement.
- **Cross-table join queries** aren't tested — that's Phase 2's job (db_views crates).
- **Concurrent insert of governance_log rows from two transactions** — the trigger reads the latest row before computing its own hash; two concurrent inserts race. [IMPLEMENTATION-PLAN-v0.md §7.1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) notes this risk; the v0 mitigation is SERIALIZABLE isolation in the Phase 4 `append()` helper, not a trigger fix. **Phase 1 does not test concurrency.**

---

## Validation Commands

**All cargo output is captured to a file and tailed separately. NEVER pipe cargo through `tail`/`head`/`grep` (see `.claude/rules/cargo-output-capture.md`).**

### Level 1: STATIC_ANALYSIS (after every task)

```bash
./scripts/brehon/cargo-check.bat --workspace > .claude/build-check.log 2>&1
status=$?
tail -60 .claude/build-check.log
echo "check exit=$status"
[ $status -eq 0 ] || exit $status
```

**EXPECT**: exit 0, zero errors, zero new warnings.

**Clippy is run at the workspace level as part of `cargo check` under Lemmy's config? No — clippy is a separate pass.** For Phase 1, run clippy once at task 12 and once before the final merge:

```bash
./scripts/brehon/cargo-check.bat --workspace --lib --tests -- 2>&1 > .claude/build-clippy.log 2>&1
# ^ this only runs check, not clippy. Use the dedicated clippy invocation:
cargo clippy --workspace --tests -- -D warnings > .claude/build-clippy.log 2>&1
status=$?
tail -80 .claude/build-clippy.log
echo "clippy exit=$status"
[ $status -eq 0 ] || exit $status
```

Note that `cargo clippy` does link (invokes the compile pipeline), so on Windows it also needs libpq if any pulled-in crate uses pq-sys. Same wrapper story as `cargo test`. **Decision**: run clippy only after libpq is installed; before that, rely on `cargo check` + hand review. If libpq install is still pending when Phase 1 closes, **note it in the Open Questions section** and defer the first clippy pass to immediately after the libpq install lands — not to Phase 2.

### Level 2: INTEGRATION_TESTS (after tasks 13 and 14)

```bash
./scripts/brehon/cargo-test.bat --test e2e -p lemmy_server > .claude/build-test-e2e.log 2>&1
status=$?
tail -120 .claude/build-test-e2e.log
echo "e2e exit=$status"
[ $status -eq 0 ] || exit $status
```

**EXPECT**: 3 tests pass (`postgres_container_boots`, `can_insert_moderation_case`, `governance_log_hash_chain_holds`).

### Level 3: MIGRATION_VALIDATION (manual, at the end of each migration task)

This runs **outside** the testcontainers harness, against a local scratch DB that the agent spins up manually. Used as a smoke test for `up.sql` + `down.sql` round-trip before committing.

```bash
docker run --rm -d --user $(id -u):$(id -g) \
  --name pg-scratch \
  -e POSTGRES_USER=lemmy \
  -e POSTGRES_PASSWORD=password \
  -e POSTGRES_DB=lemmy \
  -p 5434:5432 \
  pgautoupgrade/pgautoupgrade:18-alpine

# Wait 3s for boot, then:
export DATABASE_URL=postgres://lemmy:password@localhost:5434/lemmy
diesel migration run
diesel migration redo   # applies the new down.sql then re-runs the up.sql
docker stop pg-scratch
```

**EXPECT**: `diesel migration redo` exits 0. If the `down.sql` is broken, this catches it before the commit lands.

### Level 4: CROSS_CUTTING_VERIFICATION (manual, after tasks 6 & 8)

- [ ] `actor_pseudonym` table exists and enforces `person_id UNIQUE` (inspect via `psql \d actor_pseudonym` during task 6's migration-validation)
- [ ] `CaseStatus::EmergencyRemove` is in the Rust enum (grep `enums.rs` after task 9)
- [ ] `CaseStatus::AdminReview` is in the Rust enum
- [ ] Hash-chain trigger exists in the `r` schema (query `SELECT proname FROM pg_proc WHERE pronamespace = (SELECT oid FROM pg_namespace WHERE nspname = 'r') AND proname LIKE 'governance_log_%';` — expect 3 rows)
- [ ] Append-only DELETE is rejected (asserted by task 14's test)
- [ ] No Rust code in Phase 1 writes directly to `governance_log` except the test — Phase 1 doesn't add a production writer

### Level 5: FINAL GATE (before merging `feature/phase-1-schema` → `governance-v0`)

```bash
# Full workspace check
./scripts/brehon/cargo-check.bat --workspace > .claude/build-final-check.log 2>&1
tail -40 .claude/build-final-check.log
[ $? -eq 0 ] || exit 1

# Full e2e
./scripts/brehon/cargo-test.bat --test e2e -p lemmy_server > .claude/build-final-test.log 2>&1
tail -80 .claude/build-final-test.log
[ $? -eq 0 ] || exit 1

# Clippy (if libpq installed)
# cargo clippy --workspace --tests -- -D warnings > .claude/build-final-clippy.log 2>&1
# tail -60 .claude/build-final-clippy.log
# [ $? -eq 0 ] || exit 1
```

---

## Acceptance Criteria

- [ ] All 14 tasks completed in dependency order
- [ ] Each task has its own commit (`git log feature/phase-1-schema ^governance-v0 --oneline` shows 14 commits)
- [ ] Branch is `feature/phase-1-schema` off `governance-v0`
- [ ] Level 1: `cargo check --workspace` passes with exit 0 after each task
- [ ] Level 2: `cargo test --test e2e -p lemmy_server` green, 3 tests
- [ ] Level 3: every migration has been `diesel migration redo`'d at least once against a scratch DB
- [ ] Level 4: cross-cutting invariants hold (`EmergencyRemove` + `AdminReview` in enum; `actor_pseudonym` table exists; hash-chain trigger exists in `r.*`)
- [ ] No `.unwrap()`, `.expect()`, or `#[allow(clippy::...)]` in any new code
- [ ] No contradictions with ADR-009, ADR-011, ADR-012, ADR-013, ADR-014, ADR-015
- [ ] No work from Phases 2–6 has leaked in (no db_views, no apub, no handlers, no signing code, no `redaction::scrub`, no `governance_log::append` Rust helper, no Federation migration 4)
- [ ] `.claude/build-*.log` files exist for each task's validation run (evidence trail)

---

## Completion Checklist

- [ ] Task 1: `sha2` in `[workspace.dependencies]`
- [ ] Task 2: `add_governance_enums` migration (with `EmergencyRemove` + `AdminReview`)
- [ ] Task 3: `add_governance_core` migration
- [ ] Task 4: `add_jury_system` migration
- [ ] Task 5: `add_reputation_and_surety` migration
- [ ] Task 6: `add_actor_pseudonym` migration
- [ ] Task 7: `add_governance_log` migration (+ `pgcrypto`)
- [ ] Task 8: Hash-chain + append-only triggers in `replaceable_schema/triggers.sql`
- [ ] Task 9: 11 governance enums in `db_schema_file/src/enums.rs`
- [ ] Task 10: Regenerated `db_schema_file/src/schema.rs`
- [ ] Task 11: 14 typed ID newtypes in `db_schema/src/newtypes.rs`
- [ ] Task 12: 14 source files under `db_schema/src/source/governance/`
- [ ] Task 13: `can_insert_moderation_case` integration test
- [ ] Task 14: `governance_log_hash_chain_holds` integration test
- [ ] `./scripts/brehon/cargo-check.bat --workspace` final pass → exit 0
- [ ] `./scripts/brehon/cargo-test.bat --test e2e -p lemmy_server` final pass → exit 0, 3 tests passing

---

## Risks and Mitigations

| Risk | Likelihood | Impact | Mitigation |
|---|---|---|---|
| `diesel print-schema` regeneration on Windows without vcpkg's libpq CLI available | HIGH | HIGH (blocks task 10) | Install libpq per `feedback_cargo_invocations.md` BEFORE starting Phase 1; alternative is `docker run` the diesel CLI inside a Postgres container. See OQ below |
| JSONB key-ordering mismatch between Postgres `payload::text` and Rust `serde_json::to_string(&payload)` | MED | MED | Use flat top-level payloads in the test; fall back to jsonb-to-canonical-text cast in the trigger if needed |
| `embed_migrations!("../../migrations")` macro path is wrong for `crates/server/tests/e2e.rs` layout | LOW | LOW | First compile of task 13 catches it; adjust path until `cargo check` is green |
| Two governance tests racing on the same Postgres port | LOW | LOW | testcontainers auto-assigns distinct host ports; no action needed |
| `diesel migration redo` fails because `down.sql` is missing a `DROP INDEX` or `DROP TYPE` that was added late | MED | MED | Level 3 validation after every migration task (run `redo` locally before commit) |
| `cargo clippy --workspace` triggers libpq link before vcpkg is installed | MED | MED | Run clippy only after libpq install (see OQ below); fall back to per-crate `cargo check` + hand review during Phase 1 |
| `schema.rs` regeneration rewrites unrelated tables due to Lemmy upstream changes slipping in via rebase | LOW | MED | Commit the regenerated `schema.rs` separately from hand-written changes; eyeball the diff before committing — any change outside the new governance block is a signal to pause and investigate |
| Replaceable-schema `DROP ... CASCADE` missing the governance triggers on schema rebuild | LOW | HIGH | Trigger FUNCTIONS are in `r.*` → dropping the schema CASCADE removes them → dependent public-table TRIGGERs die with them. Behaviour is documented at `triggers.sql:1-3`; task 8 follows the rule verbatim. Validated indirectly by task 14 (hash chain still works after a schema rebuild) |
| [OQ-006](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) threshold formula being a placeholder | OUT OF SCOPE | n/a | Phase 1 doesn't touch the threshold formula at all. OQ resolution is a Phase 4 concern |
| [OQ-008](../../../docs/brehon-law-inspired-network/99-decisions-and-open-questions.md) `AdminReview` variant not yet ratified | — | LOW | [IMPLEMENTATION-PLAN-v0.md §8](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) directly recommends adding `AdminReview` in task 8 — plan follows the recommendation |

---

## Open Questions (to escalate, not auto-resolve in the plan body)

1. ~~**DB GRANT tightening on `governance_log` — defer to Phase 4?**~~ **RESOLVED 2026-04-15 (advisor-review pass 1): defer to Phase 4.** The trigger layer in task 8 is the correctness gate and is sufficient for v0. The grant layer requires a dedicated `governance_log_writer` role that Phase 4 will introduce alongside the `governance_log::append` helper. **Phase 1 plan is correct as-is — no grant in the migration.** Do NOT re-raise in any ralph iteration.

2. ~~**Windows libpq install precondition before starting Phase 1**~~ **RESOLVED 2026-04-15 (advisor-review pass 1): libpq is installed.** vcpkg + libpq landed in Phase 0 at commit `e370523c7` (which wired `PQ_LIB_DIR` into `scripts/brehon/cargo-test.bat`). Caveat: `diesel_cli` may be a stale binary from before libpq was on PATH. Task 10 has been updated with a pre-flight sub-step (verify `diesel --version` from inside the vcvars shell, reinstall if broken). See task 10 below.

3. **Embedded-migrations path from `crates/server/tests/e2e.rs`**
   `embed_migrations!("../../migrations")` resolves the path at macro expansion, with the anchor being the **crate manifest directory** (`CARGO_MANIFEST_DIR`). For `crates/server` that's `crates/server/`, so `../../migrations` → repo root `migrations/`. This is correct. **But** macro path resolution differs between Rust editions/doc-versions, and if the diesel_migrations version in use resolves relative to the test file's parent, the path becomes `../../../migrations`. Verify at first compile of task 13.

4. **JSONB canonicalisation for deterministic hashing**
   The trigger's `NEW.payload::text` may produce different strings depending on Postgres's internal JSONB storage order vs the insert order. Task 14 will catch this. If it fails, the fix is either (a) canonicalise payloads in Rust before INSERT (Phase 4's append helper is the right layer), or (b) add a `jsonb_build_object_canonical()` function in `utils.sql` and call it from the trigger. **Neither fix is a Phase-1 task — if task 14 fails, stop and escalate.**

5. **`updated_at` field on ModerationCase** — **already resolved**: removed from the plan. [04 §3](../../../docs/brehon-law-inspired-network/04-data-model-and-api.md) is authoritative; the struct has `opened_at`, `decided_at`, `closed_at` only. Plan's earlier draft added `updated_at`; see task 12 "NEW JUDGMENT CALL" note — **instructing the implementation agent to NOT add it**.

---

## Notes

**Why schema-regeneration (task 10) is its own task instead of part of task 9**: `diesel print-schema` requires a live Postgres with migrations applied. That's a physical-world dependency (docker container + libpq-enabled diesel CLI). Splitting it into its own commit keeps the dependency boundary clean: if task 10 fails because libpq isn't installed, the failure is localised — tasks 1–9 are still landed and reviewable.

**Why the `sha2` dep is task 1 and not task 8 (trigger) or task 14 (test)**: `sha2` isn't used by the trigger (which uses Postgres's `pgcrypto`) — it's only used by task 14's Rust test recomputation. But `sha2` is a workspace-level declaration, and touching workspace deps is riskier than touching a crate's `Cargo.toml` because it affects lock-file resolution. Landing it first, before any code that depends on it, gives a clean baseline for the rest of Phase 1.

**Why governance enums live in `db_schema_file` not `db_schema`**: This is how Lemmy 1.0-beta organises its own enums. `db_schema_file` is intentionally tiny (no implementation code; just schema + enum definitions) so that consumers who only want schema types can depend on it without pulling in the full `db_schema` crate + `diesel-async` machinery. This is relevant for API-common crates downstream. The plan **mirrors the existing architecture**; deviating would be an unjustified novelty.

**Why `governance_log.id` is `BIGSERIAL` while everything else is `SERIAL`**: The governance log is effectively a permanent audit record — it never gets cleaned up in v0 (see [IMPLEMENTATION-PLAN-v0.md §7.2](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)). A 32-bit sequence overflows at ~2B rows; a 64-bit sequence is effectively unbounded. This is cheap insurance for a single column that will outlive everything else.

**Why redaction and actor_pseudonym helpers don't exist in Phase 1**: They're cross-cutting helpers that have their *first caller* in Phase 4 ([§4.2](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md)). Writing them in Phase 1 without a caller means (a) they can't be tested against a real use case, (b) Phase 4 will likely adjust their signatures, and (c) they'd be dead code that workspace lints may start complaining about. Phase 1 provisions the **tables** they will need (`actor_pseudonym`); the **Rust layer** is Phase 4's job.

**Why Phase 1 doesn't wire CI**: [IMPLEMENTATION-PLAN-v0.md §1.1](../../../docs/brehon-law-inspired-network/IMPLEMENTATION-PLAN-v0.md) progress snapshot lists §2.4 task 9 (CI) as "Pre-flight, not started". CI belongs to Pre-flight, not Phase 1. Phase 1 **does** produce the artefacts CI will run against — the migrations, the e2e tests — but wiring them into GitHub Actions is separate work on a different branch.

**Why there's no `cargo clippy --fix` step**: Clippy's `--fix` is a manual convenience and must not be automated in a ralph loop — a bad fix applied globally could silently delete a planned `_`-prefixed unused parameter that's intentional. Run clippy to see the list, fix by hand.
