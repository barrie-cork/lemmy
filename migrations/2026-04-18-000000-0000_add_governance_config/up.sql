-- Phase 5a task 50: governance_config table + typed-value CHECK + _current view +
-- 34 seed rows, plus reputation_snapshot.can_sponsor column, the
-- reputation_snapshot partial-unique-index for community_id IS NULL, and the
-- threshold_score micros rescale.
--
-- Design intent:
--   - governance_config rows are typed (int|float|bool|text) via a CHECK
--     discriminator. Each row has a valid_from timestamp; admin edits insert
--     new rows and the governance_config_current view reads the latest.
--   - v0 uses edit-via-psql for admin config writes. The wrapper script
--     scripts/brehon/admin-config-write.sh (deferred to 5c sibling docs per
--     decision-queue #13) INSERTs both the config row AND a governance_log
--     entry in one tx so admin cascades (raising thresholds.jury_reliability
--     from 50 to 80 etc.) have attribution at action time (Watch 11).
--   - To evolve the CHECK in v1 (e.g. adding 'json' value_type), add a new
--     value_type + a new column + a new CHECK in a fresh migration. Do NOT
--     modify this CHECK in-place; Postgres CHECK evolution is non-trivial.

CREATE TABLE governance_config (
    id          SERIAL PRIMARY KEY,
    scope       TEXT NOT NULL,
    key         TEXT NOT NULL,
    value_type  TEXT NOT NULL,
    value_int   BIGINT,
    value_float DOUBLE PRECISION,
    value_bool  BOOLEAN,
    value_text  TEXT,
    valid_from  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by  INTEGER REFERENCES person (id) ON DELETE RESTRICT,
    CONSTRAINT governance_config_typed CHECK (
        (value_type = 'int'   AND value_int   IS NOT NULL AND value_float IS NULL     AND value_bool IS NULL     AND value_text IS NULL) OR
        (value_type = 'float' AND value_float IS NOT NULL AND value_int   IS NULL     AND value_bool IS NULL     AND value_text IS NULL) OR
        (value_type = 'bool'  AND value_bool  IS NOT NULL AND value_int   IS NULL     AND value_float IS NULL    AND value_text IS NULL) OR
        (value_type = 'text'  AND value_text  IS NOT NULL AND value_int   IS NULL     AND value_float IS NULL    AND value_bool IS NULL)
    )
);

-- Unique on (scope, key, valid_from) supports append-history semantics: an
-- admin edit inserts a new row and keeps the old row for audit reconstruction.
-- Do NOT use (scope, key) as the conflict target — GOTCHA-50a.
CREATE UNIQUE INDEX governance_config_scope_key_valid_from_idx
    ON governance_config (scope, key, valid_from);

CREATE INDEX governance_config_scope_key_idx
    ON governance_config (scope, key);

-- governance_config_current — most-recent row per (scope, key).
CREATE VIEW governance_config_current AS
SELECT DISTINCT ON (scope, key)
    id, scope, key, value_type, value_int, value_float, value_bool, value_text, valid_from, updated_by
FROM governance_config
ORDER BY scope, key, valid_from DESC;

-- reputation_snapshot.can_sponsor — populated by Phase 5a task 53; NOT read
-- by any v0 handler (enforced by scripts/brehon/lint-no-can-sponsor-read.sh).
-- [99 OQ-014]: v0 uses age-only sponsor gate; v1 flips config to activate
-- the reputation-gate read path without a schema migration.
ALTER TABLE reputation_snapshot ADD COLUMN can_sponsor BOOLEAN NOT NULL DEFAULT false;
COMMENT ON COLUMN reputation_snapshot.can_sponsor IS
    'Computed by Phase 5a task 53 but NOT read by any v0 handler. See [99 OQ-014]; v1 flips config.sponsorship.require_reputation_gate = true to activate enforcement.';

-- Postgres treats NULL as distinct in unique constraints, so instance-scoped
-- snapshots (community_id IS NULL) would not dedupe without this partial index.
-- Task 53's recompute_snapshot upsert relies on this for the NULL-community path.
CREATE UNIQUE INDEX reputation_snapshot_person_null_community
    ON reputation_snapshot (person_id) WHERE community_id IS NULL;

-- Note: the micros rescale (integer-unit → micros) was moved to Phase 5b task 58,
-- which owns the config-driven threshold formula. Phase 5a leaves the handler
-- (`create_report.rs` with `V0_THRESHOLD=3`, `V0_REPORTER_WEIGHT=1`) in integer
-- units so `report_to_modlog_golden_path` semantics are preserved.

-- Seed 34 instance-scoped config rows. ON CONFLICT DO NOTHING on
-- (scope, key, valid_from) makes this idempotent — reruns after manual
-- admin edits preserve the admin edits (the unique index on
-- (scope, key, valid_from) distinguishes valid_from=now() from the seed's
-- valid_from=seed-time).
INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    ('instance', 'thresholds.jury_reliability',           'int',  50,         NULL,    NULL, NULL),
    ('instance', 'thresholds.reporting_accuracy',         'int',  50,         NULL,    NULL, NULL),
    ('instance', 'thresholds.endorsement_strength',       'int',  25,         NULL,    NULL, NULL),
    ('instance', 'jury.panel_size',                       'int',  5,          NULL,    NULL, NULL),
    ('instance', 'jury.quorum',                           'int',  3,          NULL,    NULL, NULL),
    ('instance', 'jury.age_requirement_days',             'int',  60,         NULL,    NULL, NULL),
    ('instance', 'jury.max_concurrent_assignments',       'int',  3,          NULL,    NULL, NULL),
    ('instance', 'jury.fallback_on_small_pool',           'bool', NULL,       NULL,    true, NULL),
    ('instance', 'deltas.juror_aligned',                  'int',  10,         NULL,    NULL, NULL),
    ('instance', 'deltas.juror_outlier',                  'int',  -5,         NULL,    NULL, NULL),
    ('instance', 'deltas.reporter_upheld',                'int',  10,         NULL,    NULL, NULL),
    ('instance', 'deltas.reporter_dismissed',             'int',  -5,         NULL,    NULL, NULL),
    ('instance', 'deltas.endorsement_created_sponsor',    'int',  5,          NULL,    NULL, NULL),
    ('instance', 'deltas.endorsement_created_sponsee',    'int',  5,          NULL,    NULL, NULL),
    ('instance', 'deltas.sponsor_liability_minor',        'int',  -10,        NULL,    NULL, NULL),
    ('instance', 'deltas.sponsor_liability_moderate',     'int',  -50,        NULL,    NULL, NULL),
    ('instance', 'deltas.sponsor_liability_severe',       'int',  -200,       NULL,    NULL, NULL),
    ('instance', 'liability.founder_multiplier',          'float', NULL,      2.0,     NULL, NULL),
    ('instance', 'liability.regular_multiplier',          'float', NULL,      1.0,     NULL, NULL),
    ('instance', 'liability.sponsor_liability_floor',     'int',  0,          NULL,    NULL, NULL),
    ('instance', 'report.base_weight',                    'float', NULL,      1.0,     NULL, NULL),
    ('instance', 'report.clamp_min',                      'float', NULL,      0.1,     NULL, NULL),
    ('instance', 'report.clamp_max',                      'float', NULL,      2.0,     NULL, NULL),
    ('instance', 'report.recency_half_life_hours',        'float', NULL,      168.0,   NULL, NULL),
    ('instance', 'report.case_threshold_micros',          'int',  3000000,    NULL,    NULL, NULL),
    ('instance', 'decay.positive_half_life_days',         'int',  90,         NULL,    NULL, NULL),
    ('instance', 'onboarding.default_membership_state',   'text', NULL,       NULL,    NULL, 'member'),
    ('instance', 'onboarding.sponsor_gate_strategy',      'text', NULL,       NULL,    NULL, 'age'),
    ('instance', 'onboarding.sponsor_min_account_age_days','int', 30,         NULL,    NULL, NULL),
    ('instance', 'founder.max_founders_active',           'int',  20,         NULL,    NULL, NULL),
    ('instance', 'founder.max_expires_days',              'int',  365,        NULL,    NULL, NULL),
    ('instance', 'founder.max_seed_delta',                'int',  200,        NULL,    NULL, NULL),
    ('instance', 'job.snapshot_interval_seconds',         'int',  900,        NULL,    NULL, NULL),
    ('instance', 'job.snapshot_batch_chunk_size',         'int',  500,        NULL,    NULL, NULL)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
