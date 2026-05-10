---
phase: v1-RT-r1
role: impl-task
task: 2
brief_n: 2
authored: 2026-05-10
parallel_cohort: A
---

# [role:impl-task] v1-RT-r1 task 2 sponsor_allowlist ALTER (DROP NOT NULL + add 2 cols) — see .claude/PRPs/briefs/rt-r1-impl-2.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 task 2 extend sponsor_allowlist for r4 admin readers`

## §2 Scope

CREATE migration directory `migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/` with `up.sql` + `down.sql` per plan §10.2 skeleton **verbatim**.

**EXTEND the existing v1-AD-a-shipped table** (per DQ #181 — do NOT create a fresh table). The table already exists at `migrations/2026-04-22-000100-0000_add_sponsor_allowlist/` with shape `(id, community_id NOT NULL, person_id, created_at)`.

Up.sql:
1. `ALTER TABLE sponsor_allowlist ALTER COLUMN community_id DROP NOT NULL` + `COMMENT ON COLUMN`
2. `ALTER TABLE sponsor_allowlist ADD COLUMN added_by_admin_id INTEGER NOT NULL REFERENCES person(id) DEFAULT 1` then immediately `ALTER COLUMN added_by_admin_id DROP DEFAULT` (sentinel-default + drop pattern; safe because v1-AD-a table is empty per the doc-comment) + `COMMENT ON COLUMN`
3. `ALTER TABLE sponsor_allowlist ADD COLUMN note TEXT` + `COMMENT ON COLUMN`

Down.sql reverses: DROP COLUMN note → DROP COLUMN added_by_admin_id → SET NOT NULL on community_id (with documented caveat that down.sql fails if RT-r4-shipped NULL community_id rows exist — DBA must DELETE them first).

**Authority trail comments at top of up.sql:** PRD section 5.4; plan §10.2 Task 2; DQ #181 (EXTEND, do NOT create).

**No diesel struct edits in this task** — Task 7 owns `crates/db_schema/src/source/governance/sponsor_allowlist.rs` extension.

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.2 (skeleton verbatim) + §13 Task 2 (FILES YAML, GOTCHAs, MIRROR)
- `.claude/decision-queue.json` resolved entry **DQ #181** — EXTEND existing table
- `migrations/2026-04-22-000100-0000_add_sponsor_allowlist/up.sql` — original v1-AD-a CREATE TABLE for context
- `migrations/2026-04-22-000200-0000_add_case_applied_config_snapshot/up.sql` (mirror primary — multi-step ALTER pattern)
- `.claude/lessons/feedback_lemmy_migration_runner.md`
- `.claude/lessons/feedback_postgres_jsonb_canonicalization.md` — cite-only
- `.claude/rules/decision-queue.md` — Recipe 1

## §3a Handover from prior cohort

Task 0 pre-flight passed (#195). Phase branch tip = `80e2e5d0c`. Probe 4 verified `community_id NOT NULL` baseline on `sponsor_allowlist`. Probe 10 confirmed `SponsorAllowlistId` newtype already present in `crates/db_schema/src/newtypes.rs:331`.

## §4 Constraints

- **Per DQ #181: EXTEND, do NOT create.** Junior MUST NOT add a `CREATE TABLE sponsor_allowlist` statement. Migration is ALTER-only.
- **Sentinel default `1` is `person.id = 1`.** Safe assumption ONLY because v1-AD-a doc-comment confirms the table is empty. Junior verifies via Probe 4 in plan §13 Task 0 (already passed).
- **DROP DEFAULT immediately after ADD COLUMN.** Prevents accidental defaulting of future inserts. r4 endpoints MUST set `added_by_admin_id` explicitly.
- **down.sql `SET NOT NULL` caveat documented in comment.** Will fail if NULL community_id rows exist; documented for DBA to handle.
- **No Rust edits.** Plan §13 Task 7 owns `sponsor_allowlist.rs` struct extension.
- **DQ mid-task push rule:** Shape G — 2 `kind: "validate-pending"` DQ entries on push (workspace + migration workflows).
- **Attribution:** `from: "impl"`. Never `from: "advisor"` or `from: "planner"`.

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — SQL migration only.

## §5 Concurrency note

Cohort A 4-way parallel. Disjoint migration directories. Independent of Tasks 1, 3, 4.

## §6 COMMIT MESSAGE

`feat(v1-RT-r1): extend sponsor_allowlist (DROP NOT NULL community_id, add added_by_admin_id + note) per DQ #181 (task 2)`

Add HANDOVER YAML trailer.
