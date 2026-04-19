DROP INDEX IF EXISTS idx_moderation_case_rule_set_version;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS rule_set_version_id;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS applied_config_snapshot;
