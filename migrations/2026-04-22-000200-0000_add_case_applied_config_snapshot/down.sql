-- Inverse of up.sql (v1-AD-a). Drops the two columns added by up.sql
-- plus the partial index. Both columns are v1-AD-a-exclusive additions
-- — no pre-existing data depends on them, so the DROP is a clean
-- rollback, not a destructive edit against the append-only audit
-- invariant. See up.sql header for the full authority trail.
DROP INDEX IF EXISTS idx_moderation_case_rule_set_version;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS rule_set_version_id;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS applied_config_snapshot;
