ALTER TABLE moderation_case ADD COLUMN applied_config_snapshot JSONB;
ALTER TABLE moderation_case ADD COLUMN rule_set_version_id INTEGER REFERENCES rule_set_version (id);

CREATE INDEX idx_moderation_case_rule_set_version
    ON moderation_case (rule_set_version_id)
    WHERE rule_set_version_id IS NOT NULL;
