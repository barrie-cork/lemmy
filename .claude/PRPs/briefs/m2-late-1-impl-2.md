# Brief: m2-late-1 Task 2 — Diesel models + `SanctionEventId` newtype

## 1. Role + dispatch

`[role:impl-task] m2-late-1-task-2-diesel-models — see .claude/PRPs/briefs/m2-late-1-impl-2.md`

Pre-Shape-G plan. Workers write `validate-pending-laptop` DQ and STOP; do NOT run workspace cargo on the daemon.

## 2. Scope

**Produce:**

1. **Create** `crates/db_schema/src/source/governance/sanction_event.rs` — `SanctionEvent` model + `SanctionEventInsertForm`.
2. **Create** `crates/db_schema/src/source/governance/sanction_subscriber.rs` — `SanctionSubscriber` model + `SanctionSubscriberInsertForm`.
3. **Modify** `crates/db_schema/src/source/governance/mod.rs` — add `pub mod sanction_event;` and `pub mod sanction_subscriber;` (alphabetical order relative to siblings).
4. **Modify** `crates/db_schema/src/newtypes.rs` — add `SanctionEventId` newtype (mirrors existing newtypes; single file, NOT a `newtypes/` directory — R6).

Then write the `validate-pending-laptop` DQ entry and STOP. Do NOT run workspace cargo on the daemon.

**Explicit boundaries (do NOT touch):**
- Do NOT edit `crates/api/**` — that is T3/T4's domain.
- Do NOT edit `crates/db_schema_file/**` — T1 already completed this.
- Do NOT edit `crates/server/**` or `services/bridge/**`.
- Do NOT edit `crates/server/tests/e2e.rs` or any e2e file — isolated to T8.
- Do NOT run `cargo check --workspace` on the daemon (R7).

## 3. Required reading

**Before your first edit, Read these files:**

- `crates/db_schema/src/source/governance/sanction.rs:1-48` — the MIRROR; both models must replicate this exact derive stack and attribute layout.
- `crates/db_schema/src/newtypes.rs:1-30` — MIRROR for `SanctionEventId` newtype pattern (`DieselNewType`, `ts_rs::TS`, `fmt::Display`).
- `crates/db_schema/src/source/governance/mod.rs` — verify current mod list; add new mods in alphabetical position.
- `crates/db_schema_file/src/schema.rs` — confirm `sanction_event` and `sanction_subscriber` tables exist (T1 added them). If absent, STOP and raise `kind: "blocker"` DQ — T1 must have been incomplete.
- `.claude/PRPs/plans/m2-late.plan.md` §10.5 — Diesel models + insert forms detail.
- `feedback_validate_pending_laptop_write_then_stop.md` — workers write the DQ entry and STOP; no daemon cargo.
- `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — newtype in single file `newtypes.rs`, never a directory.

**Pre-Shape-G mandatory constraint (§2.4 of advisor-orchestrator.md):**
Your brief §4 Constraint: write the `validate-pending-laptop` DQ entry with
`commands: ["./scripts/brehon/cargo-check.sh --workspace --features full"]`,
commit + push, then **STOP**. Do NOT run `cargo-check.sh` yourself — validation is delegated to the laptop advisor.

## 3a. Handover from prior cohort

T1 completed on `phase-m2-late-1` at SHA `298f7ed77` + session retro SHA `8099523f7`. DQ `001f1c47c5dc-001` resolved `result: pass`.

```yaml
prior_cohort_tasks:
  - task: 1
    commit: 298f7ed77
    filesCreated:
      - migrations/2026-06-07-000000-0000_add_sanction_event/up.sql
      - migrations/2026-06-07-000000-0000_add_sanction_event/down.sql
    filesModified:
      - crates/db_schema_file/src/enums.rs
      - crates/db_schema_file/src/schema.rs
    keyDecisions:
      - schema.rs hand-extended (not regenerated via print-schema — fork convention)
      - Migration uses ROOT migrations/ (not crates/db_schema/migrations/)
      - sanction_kind enum + sanction_event + sanction_subscriber tables all exist
    notes: T1 validate-pending-laptop resolved pass. schema.rs has sanction_event and sanction_subscriber table blocks + sql_types::SanctionKind. T2 requires: read schema.rs to confirm table presence before authoring models.
```

## 4. Constraints

- **R6:** `SanctionEventId` newtype is in the **single file** `crates/db_schema/src/newtypes.rs`. Do NOT create a `newtypes/` directory.
- **R7:** Do NOT run `./scripts/brehon/cargo-check.sh` or any `cargo` command on the daemon. After committing, write `validate-pending-laptop` DQ entry then STOP immediately.
- **MIRROR discipline:** replicate `sanction.rs:1-48` derive stack verbatim — `#[skip_serializing_none]`, `#[derive(PartialEq, Eq, Serialize, Deserialize, Debug, Clone)]`, `#[cfg_attr(feature = "full", derive(Identifiable, Queryable, Selectable))]`, `#[cfg_attr(feature = "full", diesel(table_name = <table>))]`, `#[cfg_attr(feature = "full", diesel(check_for_backend(diesel::pg::Pg)))]`, `#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]`, `#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]`. Insert forms mirror `SanctionInsertForm` (no `Identifiable`; uses `Insertable`/`AsChangeset` under `full`).
- **`governance_log_entry_hash: String`** in `SanctionEventInsertForm` (hex-encoded; insert form only; the `SanctionEvent` model field is also `String`).
- **`[P]` with T3:** T2 and T3 are parallel. T2 owns `db_schema` models + mod.rs + newtypes.rs. T3 owns governance_log.rs consts + api shim + sanction_kind_map.rs. These are file-disjoint — do NOT touch T3's files.
- **DQ commit subject:** `chore(decision-queue): impl raised validate-pending-laptop for m2-late-1 task-2`.
- **Impl commit subject:** `feat(db_schema): add SanctionEvent + SanctionSubscriber models + SanctionEventId newtype (task 2)`.
- One commit per task (impl commit + DQ commit = 2 commits total then STOP).
- **Mid-task push:** after EACH commit (impl and DQ), `git push origin phase-m2-late-1` immediately so the laptop advisor can see the DQ entry.

**Mandatory lesson fired (file-class table match):**
- `feedback_validate_pending_laptop_write_then_stop.md` — any new file under `crates/db_schema/**` in a pre-Shape-G plan.
- `feedback_newtype_locations_lemmy_db_schema_vs_file.md` — `newtypes.rs` edit.
