---
phase: v1-RT-r1
role: impl-task
task: 3
brief_n: 3
authored: 2026-05-10
parallel_cohort: A
---

# [role:impl-task] v1-RT-r1 task 3 backfill source_event_type via reason ILIKE — see .claude/PRPs/briefs/rt-r1-impl-3.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 task 3 backfill reputation_event.source_event_type per DQ #184`

## §2 Scope

CREATE migration directory `migrations/2026-05-10-000200-0000_backfill_reputation_event_source_type/` with `up.sql` + `down.sql` per plan §10.3 skeleton **verbatim**.

Up.sql performs 3 idempotent UPDATEs in priority order (per DQ #184 — first match wins):
1. `UPDATE reputation_event SET source_event_type = 'SponsorLiability' WHERE source_event_type = 'Endorsement' AND reason ILIKE 'sponsor_liability%'`
2. `UPDATE reputation_event SET source_event_type = 'JuryVote' WHERE source_event_type = 'Endorsement' AND (reason ILIKE 'jury_reliability%' OR reason ILIKE 'jury_vote%' OR reason ILIKE 'jury_align%')`
3. `UPDATE reputation_event SET source_event_type = 'FounderSeed' WHERE source_event_type = 'Endorsement' AND reason ILIKE 'founder_seed%'`

Each UPDATE's `WHERE source_event_type = 'Endorsement'` ensures idempotency — re-runs are no-ops because rows already updated to non-Endorsement values fall outside the WHERE clause.

Down.sql reverses: `UPDATE reputation_event SET source_event_type = 'Endorsement' WHERE source_event_type IN ('SponsorLiability', 'JuryVote', 'FounderSeed')`.

**Authority trail comments at top of up.sql:** PRD section 7 Backfill row + section 5.3 source enumeration; plan §10.3 Task 3; DQ #184 (use source_case_id + reason ILIKE; columns endorsement_id/jury_vote_id do NOT exist).

**Note:** Default `'Endorsement'` from Task 1's column DEFAULT covers v0 rows that don't match any of the 3 UPDATEs (per the 5-step DQ #184 precedence: step 4 = Endorsement-by-default, step 5 = ManualSeed fallback unused at backfill — applies to future stale rows).

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §10.3 (skeleton verbatim) + §13 Task 3 (FILES YAML, GOTCHAs, MIRROR)
- `.claude/decision-queue.json` resolved entry **DQ #184** — backfill heuristic precedence chain (5 steps, source_case_id + reason ILIKE)
- `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql` JM-a Task 2 backfill block (mirror primary)
- `crates/db_schema/src/source/governance/reputation_event.rs:17-28` — verifies columns are `source_case_id` + `source_report_id` (NOT `endorsement_id` / `jury_vote_id`)
- `.claude/lessons/feedback_lemmy_migration_runner.md`
- `.claude/rules/decision-queue.md` — Recipe 1

## §3a Handover from prior cohort

Task 0 pre-flight passed (#195). Phase branch tip = `80e2e5d0c`. Tasks 1, 2, 4 may be running in parallel (Cohort A); each in its own worktree. Task 3 timestamp `2026-05-10-000200-0000` sorts after Task 1's `2026-05-10-000000-0000` — ordering correct.

## §4 Constraints

- **Per DQ #184: use `source_case_id` + `reason ILIKE` only.** Junior MUST NOT reference `endorsement_id` or `jury_vote_id` — these columns do not exist on `reputation_event` (verified at brief-write via Read of source struct).
- **Idempotency invariant:** every UPDATE includes `WHERE source_event_type = 'Endorsement'` so re-runs are no-ops.
- **Migration timestamp ordering:** `2026-05-10-000200-0000` MUST sort lexicographically AFTER Task 1's `2026-05-10-000000-0000`. Task 3 depends conceptually on Task 1's column existing, but execution order is enforced by lexicographic timestamp at deploy time. Re-verify at task start.
- **No Rust edits.** SQL migration files only.
- **DQ mid-task push rule:** Shape G — 2 `kind: "validate-pending"` DQ entries on push.
- **Attribution:** `from: "impl"`.

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — SQL migration only.

## §5 Concurrency note

Cohort A 4-way parallel. Disjoint migration directories. Each task in its own worktree.

## §6 COMMIT MESSAGE

`feat(v1-RT-r1): backfill reputation_event.source_event_type via reason ILIKE precedence per DQ #184 (task 3)`

Add HANDOVER YAML trailer.
