-- Reverse of v1-JM-a 2026-04-23-000100 in LIFO order. Drop the new
-- table + index first (FK to moderation_case), then drop the
-- jury_assignment additions, then the moderation_case additions.
--
-- ============================================================
-- ADR exception trail (protected governance tables: moderation_case, jury_assignment)
-- ============================================================
-- .coderabbit.yaml migrations/** rule (lines 123-130) flags DROP COLUMN
-- on protected governance tables. This down.sql drops 6 moderation_case
-- columns + 2 jury_assignment columns + 1 new table in LIFO order so
-- the companion up.sql is reversible per ADR-010's staged-release
-- reversibility requirement.
--
-- Reversibility authority: ADR-010 pairs every additive migration with
-- a matching down.sql for the solo-dev staged-release cadence. The
-- destructive column drops here are the MIRROR of the up.sql's additive
-- ALTERs — no row data other than the columns themselves is touched
-- (the 6 newly-added moderation_case columns hold pre-v1 snapshot data
-- that is re-derivable from up.sql's backfill UPDATE on the next
-- forward-apply). No data-preservation is required because:
--   - severity_tier / status_tier default to 'Minor' / 'Regular' per
--     §8.4 and are re-populated by attmissingval on re-apply.
--   - panel_size / quorum / threshold snapshots are constants (5/3/3)
--     for pre-v1 cases; v1 cases re-populate from the cascade config
--     the next time admin_assign_jury runs (v1-JM-b).
--   - appeal_window_expires_at is deterministic from decided_at +
--     closed_at per the CASE expression in up.sql.
--
-- No governance_log row is mutated by this revert (ADR-008 append-only
-- is preserved by not touching governance_log at all). No actor_pseudonym
-- row is mutated (ADR-015 pseudonymisation integrity preserved).
--
-- Controlling ADR for this change: **ADR-010** (same as up.sql —
-- reversibility requirement). ADR-008 / ADR-015 are respected by
-- leaving governance_log and actor_pseudonym untouched.
--
-- Authority trail:
--   - Plan:  .claude/PRPs/plans/phase-v1-JM-a.plan.md §10.4, §10.5, Task 2
--   - PRD:   .claude/PRPs/prds/v1-jury-mechanics.prd.md §8.1, §8.2, §8.3, §8.4
--   - ADR-010: reversibility + no retroactive invalidation
--   - ADR-008 / ADR-015: respected (governance_log + actor_pseudonym untouched)
-- ============================================================
DROP INDEX IF EXISTS idx_jcvl_case_id;
DROP TABLE IF EXISTS jury_constraint_violation_log;

ALTER TABLE jury_assignment DROP COLUMN IF EXISTS role;
ALTER TABLE jury_assignment DROP COLUMN IF EXISTS selected_under_constraints;

ALTER TABLE moderation_case DROP COLUMN IF EXISTS appeal_window_expires_at;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS threshold_count_snapshot;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS quorum_snapshot;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS panel_size_snapshot;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS status_tier;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS severity_tier;
