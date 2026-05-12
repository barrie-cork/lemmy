---
phase: v1-RT-r1
role: impl-task
task: 1
brief_n: 1
authored: 2026-05-10
parallel_cohort: A
---

# [role:impl-task] v1-RT-r1 task 1 reputation_event v1 columns + source_type enum — see .claude/PRPs/briefs/rt-r1-impl-1.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 task 1 add reputation_event dedupe_key + source_event_type + enum`

## §2 Scope

CREATE migration directory `migrations/2026-05-10-000000-0000_add_reputation_event_v1_columns/` with `up.sql` + `down.sql` per plan §10.1 skeleton **verbatim**.

Up.sql adds (in order, single transaction — NO `-- no-transaction` directive):
1. `CREATE TYPE reputation_event_source_type AS ENUM (...9 PascalCase variants...)`
2. `ALTER TABLE reputation_event ADD COLUMN dedupe_key TEXT` + `COMMENT ON COLUMN`
3. `CREATE UNIQUE INDEX reputation_event_dedupe_key_partial_idx ON reputation_event (dedupe_key) WHERE dedupe_key IS NOT NULL` + `COMMENT ON INDEX`
4. `ALTER TABLE reputation_event ADD COLUMN source_event_type reputation_event_source_type NOT NULL DEFAULT 'Endorsement'` + `COMMENT ON COLUMN`

Down.sql reverses LIFO with `DROP COLUMN IF EXISTS` / `DROP INDEX IF EXISTS` / `DROP TYPE IF EXISTS`.

**Authority trail comments at top of up.sql:** PRD sections 5.3 + 7; plan §10.1 Task 1; DQ #182 (governance/ subdir paths).

**No diesel struct or schema.rs edits in this task** — those are Tasks 5/6/7. Only the SQL migration files.

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.1 (skeleton verbatim) + §13 Task 1 (FILES YAML, GOTCHAs, MIRROR)
- `migrations/2026-04-15-100000-0000_add_governance_enums/up.sql` (mirror primary — PostgreSQL ENUM CREATE TYPE pattern)
- `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql` (mirror secondary — multi-step ALTER TABLE pattern)
- `.claude/lessons/feedback_lemmy_migration_runner.md` — `cargo run -p lemmy_diesel_utils --features full -- migration run` for local apply; never raw `diesel migration run`
- `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — cite-only (RT-r1 ships zero JSONB)
- `.claude/rules/decision-queue.md` — Recipe 1 (validate-pending DQ) on push

## §3a Handover from prior cohort

Task 0 pre-flight harness audit completed clean (#195 done 2026-05-10T14:13Z). 9 probes PASS + 2 non-blocking WARN (Probe 1 expected pattern; Probe 8 yamllint missing — workflow flags manually verified). Zero DQ blockers.

Phase branch tip at `phase-v1-RT-r1` = `80e2e5d0c` (bm-cut runlog seed). No prior task commits to merge with.

## §4 Constraints

- **Migration directory timestamp:** `2026-05-10-000000-0000` is exact. Re-verify at task start that no migration with this timestamp exists; the most recent migration on `governance-v0` at plan-write time was `2026-05-03-000100-0000_add_sponsor_liability_grace_window` per plan §13 Task 1 GOTCHA.
- **PascalCase enum variants** match `DbValueStyle = "verbatim"` on Rust side (Task 5 will add the matching Rust enum).
- **NOT NULL DEFAULT 'Endorsement'** exploits Postgres 11+ `attmissingval` (O(1) backfill at schema-add time). Backfill UPDATE is Task 3, not this task.
- **No diesel/Rust edits.** Plan §13 Task 5 owns the Rust enum addition; Task 6 owns `schema.rs` regen; Task 7 owns the diesel struct extension.
- **DQ mid-task push rule:** Recipe 1 — Shape G — push branch + write 2 `kind: "validate-pending"` DQ entries (one for `cargo-validate-workspace.yml` workflow_run_id, one for `cargo-validate-migration.yml` workflow_run_id) per plan §13 Task 1 "Push and exit (Shape G)".
- **Attribution:** DQ entries use `from: "impl"`, `answered_by: null`. Never `from: "advisor"` or `from: "planner"`.
- **Branch:** must be on a worker branch off `phase-v1-RT-r1` (Junior worktree assigns automatically).

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 1 is SQL migration files only. No Rust authoring; no `LemmyResult` / `Box<dyn Error>` decision.

## §5 Concurrency note

Cohort A is 4-way parallel (Tasks 1, 2, 3, 4). Each task creates files in disjoint migration directories (timestamps `000000`, `000100`, `000200`, `000300`) — zero file overlap per plan §13 FILES YAML. Each task gets its own Junior worktree per `feedback_parallel_agents_one_worktree_per_agent.md`.

## §6 COMMIT MESSAGE

`feat(v1-RT-r1): add reputation_event v1 columns + reputation_event_source_type enum + partial unique index (task 1)`

Add HANDOVER YAML trailer with `filesCreated`, `filesModified`, `keyDecisions`, `notes` per template.
