-- v1-RT-r1 task 1: add reputation_event v1 columns + new source-type enum.
-- ============================================================
-- ADR exception trail (protected governance tables — append-only)
-- ============================================================
-- ADDITIVE only: ADD COLUMN, CREATE INDEX, CREATE TYPE.
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-reputation-tuning.prd.md sections 5.3 + 7
--   - Plan: .claude/PRPs/plans/v1-reputation-tuning-r1.plan.md section 10.1, Task 1
--   - DQ #182 (advisor 2026-05-10): governance/ subdir paths
-- ============================================================

CREATE TYPE reputation_event_source_type AS ENUM (
    'Endorsement',
    'JuryVote',
    'SponsorLiability',
    'FounderSeed',
    'ParticipationCron',
    'DormancyCron',
    'VoteOutcome',
    'EvidenceQuality',
    'ManualSeed'
);

ALTER TABLE reputation_event ADD COLUMN dedupe_key TEXT;
COMMENT ON COLUMN reputation_event.dedupe_key IS
    'Per PRD section 5.3 source 1+2: idempotency key for cron events.
     Format: source:community_id:iso_week or source:community_id:person_id:iso_week.
     NULL for non-cron events.';

CREATE UNIQUE INDEX reputation_event_dedupe_key_partial_idx
    ON reputation_event (dedupe_key)
    WHERE dedupe_key IS NOT NULL;
COMMENT ON INDEX reputation_event_dedupe_key_partial_idx IS
    'Per PRD section 5.3 source 1: idempotency for participation cron emits.
     Partial WHERE keeps the index small.';

ALTER TABLE reputation_event
    ADD COLUMN source_event_type reputation_event_source_type
    NOT NULL DEFAULT 'Endorsement';
COMMENT ON COLUMN reputation_event.source_event_type IS
    'Per PRD section 5.3 + 7: per-event source classification.
     v0 rows receive Endorsement via column DEFAULT;
     2026-05-10-000200-0000 backfill revises by reason ILIKE per DQ #184.';
