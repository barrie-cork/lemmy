DELETE FROM governance_config WHERE scope = 'instance' AND valid_from = '2026-05-17T00:00:00Z'::timestamptz AND key IN (
  'federation.inbound.default_trust_for_new_peers',
  'federation.inbound.per_peer_rate_per_hour',
  'federation.inbound.per_actor_attestation_rate_per_hour',
  'federation.inbound.per_peer_storage_cap',
  'federation.inbound.max_payload_bytes_sanction_notice',
  'federation.inbound.max_payload_bytes_trust_attestation',
  'federation.inbound.max_payload_bytes_moderation_label',
  'federation.inbound.replay_window_days',
  'federation.inbound.replay_cleanup_cron_interval_minutes',
  'federation.inbound.summary_max_chars',
  'federation.inbound.admin_review_default_filter_days'
);

DROP INDEX IF EXISTS idx_fa_source;
DROP INDEX IF EXISTS idx_fa_admin_action;
ALTER TABLE federation_attestation
  DROP COLUMN IF EXISTS dismissal_rationale,
  DROP COLUMN IF EXISTS admin_action,
  DROP COLUMN IF EXISTS admin_reviewed_at,
  DROP COLUMN IF EXISTS peer_trust_level_at_receipt,
  DROP COLUMN IF EXISTS received_at,
  DROP COLUMN IF EXISTS source_instance;

DROP INDEX IF EXISTS idx_rsn_admin_action;
ALTER TABLE remote_sanction_notice
  DROP COLUMN IF EXISTS dismissal_rationale,
  DROP COLUMN IF EXISTS admin_action,
  DROP COLUMN IF EXISTS admin_reviewed_at,
  DROP COLUMN IF EXISTS peer_trust_level_at_receipt;

DROP TABLE IF EXISTS remote_moderation_label;
DROP TABLE IF EXISTS federation_inbox_nonce;
DROP TABLE IF EXISTS federation_inbox_dropped_log;
DROP TABLE IF EXISTS federation_peer;

DROP TYPE IF EXISTS federation_inbox_admin_action_enum;
DROP TYPE IF EXISTS federation_peer_trust_enum;
