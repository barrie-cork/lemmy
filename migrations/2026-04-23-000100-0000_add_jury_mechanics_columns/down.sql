-- Reverse of v1-JM-a 2026-04-23-000100 in LIFO order. Drop the new
-- table + index first (FK to moderation_case), then drop the
-- jury_assignment additions, then the moderation_case additions.
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
