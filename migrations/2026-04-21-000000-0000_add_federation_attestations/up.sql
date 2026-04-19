-- Phase 6 Task 70 — federation persistence tables
--
-- Two new tables per [04 §3 Migration 4]:
--   * federation_attestation     — outbound trust attestations we have signed
--                                   (TrustedReporter, JuryEligible, etc.).
--   * remote_sanction_notice     — advisory rows mirroring inbound
--                                   PublishSanctionNotice activities. Per
--                                   [99 ADR-006] these are NEVER auto-applied;
--                                   `local_case_id` stays NULL until an admin
--                                   opens a corresponding local case.
--
-- Enum names (verified against migration 2026-04-15-100000-0000_add_governance_enums):
--   * `attestation_type`  (NOT `attestation_type_enum` — plan §task 70 used
--      the wrong suffix; actual DB enum is bare `attestation_type`).
--   * `sanction_action`   (same naming convention; bare).
--   * `sanction_scope`    (same).
--
-- Deviations from [04 §3 RemoteSanctionNotice]:
--   * `received_at TIMESTAMPTZ NOT NULL DEFAULT NOW()` is added beyond what
--     the design doc lists, per decision-queue DQ-6.1 (advisor-answered
--     2026-04-19): admin-review queries ("notices received in last 24h")
--     need it; non-breaking extension to the design doc.
--
-- No partial-unique idempotency index on remote_sanction_notice — we rely on
-- Lemmy's existing ReceivedActivity dedup at the HTTP layer per DQ-6.3.

CREATE TABLE federation_attestation (
    id SERIAL PRIMARY KEY,
    actor_url TEXT NOT NULL,
    subject_url TEXT NOT NULL,
    attestation_type attestation_type NOT NULL,
    valid_until TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    signature TEXT NOT NULL
);

CREATE INDEX idx_fed_attestation_subject ON federation_attestation (subject_url);

CREATE INDEX idx_fed_attestation_actor ON federation_attestation (actor_url);

CREATE TABLE remote_sanction_notice (
    id SERIAL PRIMARY KEY,
    source_instance TEXT NOT NULL,
    target_url TEXT NOT NULL,
    action sanction_action NOT NULL,
    scope sanction_scope NOT NULL,
    summary TEXT NOT NULL,
    published_at TIMESTAMPTZ NOT NULL,
    signature TEXT NOT NULL,
    local_case_id INTEGER REFERENCES moderation_case (id) ON DELETE SET NULL,
    received_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_remote_sanction_notice_target ON remote_sanction_notice (target_url);

CREATE INDEX idx_remote_sanction_notice_source ON remote_sanction_notice (source_instance, received_at);
