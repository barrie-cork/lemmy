-- v1-AD-a ADDITIVE extension to moderation_case.
--
-- Adds two nullable columns (no data loss, no NOT NULL, no backfill):
--   1. applied_config_snapshot JSONB — pins in-flight juries to the
--      panel-size / quorum / threshold config they were seated under.
--      Implements ADR-010's "no retroactive invalidation of in-flight
--      juries" invariant. See v1-admin-dashboard PRD §3.4.
--   2. rule_set_version_id INTEGER → rule_set_version(id) — pins a
--      case to the rule-set version active at decision time. Resolves
--      OQ-002. See v1-admin-dashboard PRD §3.6.
--
-- This migration is ADDITIVE (ADD COLUMN, nullable). The "protected
-- governance table ALTER" policy in .coderabbit.yaml:123-130 targets
-- destructive changes (DROP / TRUNCATE / UPDATE-in-place of audit
-- records) that contradict ADR-008 / ADR-015. This migration does
-- neither. Authority trail:
--   - Plan: .claude/PRPs/plans/completed/v1-admin-dashboard-a.plan.md §§ 1, 3
--   - PRD:  .claude/PRPs/prds/v1-admin-dashboard.prd.md §§ 3.4, 3.6
--   - Resolutions B1-B5 locked 2026-04-19 (plan §11).
ALTER TABLE moderation_case ADD COLUMN applied_config_snapshot JSONB;
ALTER TABLE moderation_case ADD COLUMN rule_set_version_id INTEGER REFERENCES rule_set_version (id);

CREATE INDEX idx_moderation_case_rule_set_version
    ON moderation_case (rule_set_version_id)
    WHERE rule_set_version_id IS NOT NULL;
