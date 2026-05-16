-- v1-federation-inbound-a task 1: combined schema + enum + tables + ALTERs + seed migration.
-- ============================================================
-- ADR exception trail (federation governance tables)
-- ============================================================
-- ADDITIVE only: CREATE TYPE, CREATE TABLE, CREATE INDEX, ALTER TABLE
-- ADD COLUMN, INSERT … ON CONFLICT DO NOTHING.
-- NO DROP, NO ALTER on existing columns, NO UPDATE on Phase-6 rows
-- (PRD §8.3: Phase 6 tables empty pre-v1).
--
-- Controlling ADRs: ADR-006 (advisory-only — local_case_id SET-NULL);
-- ADR-015 (pseudonymisation — actor_url/target_url/subject_url TEXT).
--
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-federation-inbound.prd.md §8.2
--   - Plan: .claude/PRPs/plans/v1-federation-inbound-a.plan.md §10.1, Task 1
--   - DQ #230 (advisor 2026-05-16): trust-state helpers in db_schema.
--   - DQ #232 (advisor 2026-05-16): shared-file edits append-only.
--
-- DEVIATION from plan §10.1 pre-condition checks (self-resolved DQ #240):
--   - Phase 6 enum names are bare (no _enum suffix):
--     attestation_type (not attestation_type_enum),
--     sanction_action (not sanction_action_enum),
--     sanction_scope (not sanction_scope_enum).
--   - The Phase 6 migration 2026-04-15-100000-0000_add_governance_enums/up.sql
--     + migration 2026-04-21-000000-0000_add_federation_attestations/up.sql
--     explicitly documents this (comment: "NOT attestation_type_enum").
--   - Pre-condition checks corrected to bare names to avoid RAISE EXCEPTION.
-- ============================================================

DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'federation_attestation') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 federation_attestation table (not present)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'remote_sanction_notice') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 remote_sanction_notice table (not present)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'remote_sanction_notice' AND column_name = 'received_at') THEN
    RAISE EXCEPTION 'federation-inbound-v1 assumes Phase 6 DQ-6.1 resolved with remote_sanction_notice.received_at column (not present)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'attestation_type') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 attestation_type (not registered)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'sanction_action') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 sanction_action (not registered)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'sanction_scope') THEN
    RAISE EXCEPTION 'federation-inbound-v1 requires Phase 6 sanction_scope (not registered)';
  END IF;
  IF NOT EXISTS (SELECT 1 FROM information_schema.columns WHERE table_name = 'remote_sanction_notice' AND column_name = 'source_instance' AND data_type = 'text') THEN
    RAISE EXCEPTION 'federation-inbound-v1 assumes remote_sanction_notice.source_instance is TEXT (peer domain).';
  END IF;
END
$$;

CREATE TYPE federation_peer_trust_enum AS ENUM ('unknown', 'allowlisted', 'untrusted_receive', 'blocklisted');
CREATE TYPE federation_inbox_admin_action_enum AS ENUM ('unreviewed', 'cross_linked', 'dismissed');

CREATE TABLE federation_peer (
  instance_id     INT PRIMARY KEY REFERENCES instance(id) ON DELETE CASCADE,
  trust_level     federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  added_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  added_by_actor  TEXT,
  notes           JSONB NOT NULL DEFAULT '{}'::JSONB,
  updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_federation_peer_trust ON federation_peer (trust_level);

CREATE TABLE federation_inbox_dropped_log (
  id              BIGSERIAL PRIMARY KEY,
  source_instance TEXT NOT NULL,
  activity_id     TEXT,
  drop_reason     TEXT NOT NULL,
  payload_excerpt TEXT,
  dropped_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX idx_fil_drop_source ON federation_inbox_dropped_log (source_instance, dropped_at DESC);
CREATE INDEX idx_fil_drop_reason ON federation_inbox_dropped_log (drop_reason, dropped_at DESC);

CREATE TABLE federation_inbox_nonce (
  peer_instance   TEXT NOT NULL,
  activity_id     TEXT NOT NULL,
  seen_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  PRIMARY KEY (peer_instance, activity_id)
);
CREATE INDEX idx_fin_nonce_seen_at ON federation_inbox_nonce (seen_at);

CREATE TABLE remote_moderation_label (
  id              SERIAL PRIMARY KEY,
  source_instance TEXT NOT NULL,
  actor_url       TEXT NOT NULL,
  target_url      TEXT NOT NULL,
  label           TEXT NOT NULL,
  summary         TEXT,
  published_at    TIMESTAMPTZ NOT NULL,
  signature       TEXT NOT NULL,
  local_case_id   INT REFERENCES moderation_case(id) ON DELETE SET NULL,
  received_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
  peer_trust_level_at_receipt federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  admin_reviewed_at TIMESTAMPTZ,
  admin_action    federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  dismissal_rationale TEXT
);
CREATE INDEX idx_rml_target ON remote_moderation_label (target_url);
CREATE INDEX idx_rml_source ON remote_moderation_label (source_instance, received_at DESC);
CREATE INDEX idx_rml_admin_action ON remote_moderation_label (admin_action) WHERE admin_action = 'unreviewed';

ALTER TABLE remote_sanction_notice
  ADD COLUMN peer_trust_level_at_receipt federation_peer_trust_enum NOT NULL DEFAULT 'unknown',
  ADD COLUMN admin_reviewed_at TIMESTAMPTZ,
  ADD COLUMN admin_action federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  ADD COLUMN dismissal_rationale TEXT;
CREATE INDEX idx_rsn_admin_action ON remote_sanction_notice (admin_action) WHERE admin_action = 'unreviewed';

ALTER TABLE federation_attestation
  ADD COLUMN source_instance TEXT,
  ADD COLUMN received_at TIMESTAMPTZ,
  ADD COLUMN peer_trust_level_at_receipt federation_peer_trust_enum,
  ADD COLUMN admin_reviewed_at TIMESTAMPTZ,
  ADD COLUMN admin_action federation_inbox_admin_action_enum NOT NULL DEFAULT 'unreviewed',
  ADD COLUMN dismissal_rationale TEXT;
CREATE INDEX idx_fa_admin_action ON federation_attestation (admin_action) WHERE admin_action = 'unreviewed';
CREATE INDEX idx_fa_source ON federation_attestation (source_instance, received_at DESC) WHERE source_instance IS NOT NULL;

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
  ('instance', 'federation.inbound.default_trust_for_new_peers',       'text',  NULL,   NULL,  NULL,  'unknown'),
  ('instance', 'federation.inbound.per_peer_rate_per_hour',             'int',   100,    NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.per_actor_attestation_rate_per_hour','int',   10,     NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.per_peer_storage_cap',               'int',   10000,  NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.max_payload_bytes_sanction_notice',  'int',   65536,  NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.max_payload_bytes_trust_attestation','int',   8192,   NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.max_payload_bytes_moderation_label', 'int',   8192,   NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.replay_window_days',                 'int',   7,      NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.replay_cleanup_cron_interval_minutes','int',  60,     NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.summary_max_chars',                  'int',   8000,   NULL,  NULL,  NULL),
  ('instance', 'federation.inbound.admin_review_default_filter_days',   'int',   7,      NULL,  NULL,  NULL)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
