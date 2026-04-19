-- Reverse Phase 6 Task 70 in dependency order.
-- The `attestation_type`, `sanction_action`, `sanction_scope` PG enums are
-- shared with other tables (sanction, etc.) — DO NOT drop them here.

DROP INDEX IF EXISTS idx_remote_sanction_notice_source;

DROP INDEX IF EXISTS idx_remote_sanction_notice_target;

DROP TABLE IF EXISTS remote_sanction_notice;

DROP INDEX IF EXISTS idx_fed_attestation_actor;

DROP INDEX IF EXISTS idx_fed_attestation_subject;

DROP TABLE IF EXISTS federation_attestation;
