-- Phase 5a task 50 rollback. Reverse of up.sql in strict mirror order.

-- 1. Rescale threshold_score from micros back to integer units
--    (idempotent round-trip under `diesel migration redo` — GOTCHA-50b).
UPDATE moderation_case SET threshold_score = threshold_score / 1000000;

-- 2. Drop partial unique index and the can_sponsor column.
DROP INDEX IF EXISTS reputation_snapshot_person_null_community;
ALTER TABLE reputation_snapshot DROP COLUMN IF EXISTS can_sponsor;

-- 3. Drop the view, then the table (cascading indexes + CHECK drop).
DROP VIEW IF EXISTS governance_config_current;
DROP INDEX IF EXISTS governance_config_scope_key_idx;
DROP INDEX IF EXISTS governance_config_scope_key_valid_from_idx;
DROP TABLE IF EXISTS governance_config;
