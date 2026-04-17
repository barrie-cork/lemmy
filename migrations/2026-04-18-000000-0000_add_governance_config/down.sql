-- Phase 5a task 50 rollback. Reverse of up.sql in strict mirror order.

-- 1. Drop partial unique index and the can_sponsor column.
DROP INDEX IF EXISTS reputation_snapshot_person_null_community;
ALTER TABLE reputation_snapshot DROP COLUMN IF EXISTS can_sponsor;

-- 2. Drop the view, then the table (cascading indexes + CHECK drop).
--    DROP TABLE cascades its own indexes, so explicit DROP INDEX is unnecessary.
DROP VIEW IF EXISTS governance_config_current;
DROP TABLE IF EXISTS governance_config;
