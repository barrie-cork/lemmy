-- v1-RT-r1 task 4: seed 26 reputation-tuning-owned governance_config rows.
-- ============================================================
-- Authoritative scope: PRD section 8 (29 rows) MINUS 3 v1-AD-a-shipped
-- per planner DQ #187:
--   - deltas.participation_weekly_active
--   - participation.dormancy_window_days
--   - deltas.participation_dormant
-- 26 rows below = 28 PRD knobs + 1 feature flag - 3 dupes.
--
-- Cumulative invariant: SEEDED_KEYS_WITH_CONSTS.len() must equal
-- 34 + 27 + 27 + 13 + 26 = 127. Parity test at config.rs:2692-2707
-- updated in Task 8.
--
-- Authority trail:
--   - PRD: section 8 Defaults Matrix
--   - Plan: section 10.4, Task 4
--   - DQ #185 (advisor): conceptual count is 29
--   - DQ #187 (planner): net-new is 26
-- ============================================================

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    -- 8 decay.<dimension>.<direction>_half_life_days (int)
    ('instance', 'decay.reporting_accuracy.positive_half_life_days',          'int',  90,    NULL, NULL,  NULL),
    ('instance', 'decay.reporting_accuracy.negative_half_life_days',          'int',  180,   NULL, NULL,  NULL),
    ('instance', 'decay.jury_reliability.positive_half_life_days',            'int',  90,    NULL, NULL,  NULL),
    ('instance', 'decay.jury_reliability.negative_half_life_days',            'int',  180,   NULL, NULL,  NULL),
    ('instance', 'decay.participation_consistency.positive_half_life_days',   'int',  60,    NULL, NULL,  NULL),
    ('instance', 'decay.participation_consistency.negative_half_life_days',   'int',  60,    NULL, NULL,  NULL),
    ('instance', 'decay.endorsement_strength.positive_half_life_days',        'int',  90,    NULL, NULL,  NULL),
    ('instance', 'decay.endorsement_strength.negative_half_life_days',        'int',  180,   NULL, NULL,  NULL),
    -- 8 bounds.<dimension>.<floor|ceiling> (int)
    ('instance', 'bounds.reporting_accuracy.floor',                           'int',  -100,  NULL, NULL,  NULL),
    ('instance', 'bounds.reporting_accuracy.ceiling',                         'int',  100,   NULL, NULL,  NULL),
    ('instance', 'bounds.jury_reliability.floor',                             'int',  -100,  NULL, NULL,  NULL),
    ('instance', 'bounds.jury_reliability.ceiling',                           'int',  100,   NULL, NULL,  NULL),
    ('instance', 'bounds.participation_consistency.floor',                    'int',  -100,  NULL, NULL,  NULL),
    ('instance', 'bounds.participation_consistency.ceiling',                  'int',  100,   NULL, NULL,  NULL),
    ('instance', 'bounds.endorsement_strength.floor',                         'int',  0,     NULL, NULL,  NULL),
    ('instance', 'bounds.endorsement_strength.ceiling',                       'int',  200,   NULL, NULL,  NULL),
    -- 1 deltas.participation_juror_aligned (PRD section 5.3 source 3)
    ('instance', 'deltas.participation_juror_aligned',                        'int',  1,     NULL, NULL,  NULL),
    -- 2 participation.* context knobs (activity_threshold + lookback)
    ('instance', 'participation.activity_threshold_comments',                 'int',  1,     NULL, NULL,  NULL),
    ('instance', 'participation.lookback_days',                               'int',  7,     NULL, NULL,  NULL),
    -- 2 deltas.evidence_* (PRD section 5.3 source 4)
    ('instance', 'deltas.evidence_cited',                                     'int',  1,     NULL, NULL,  NULL),
    ('instance', 'deltas.evidence_bad_faith',                                 'int',  -1,    NULL, NULL,  NULL),
    -- 1 participation.evidence_cited_rationale_threshold_chars
    ('instance', 'participation.evidence_cited_rationale_threshold_chars',    'int',  256,   NULL, NULL,  NULL),
    -- 2 job.* cadence knobs (instance-only)
    ('instance', 'job.participation_interval_days',                           'int',  7,     NULL, NULL,  NULL),
    ('instance', 'job.rollup_interval_days',                                  'int',  7,     NULL, NULL,  NULL),
    -- 1 job.rollup_equal_weights (instance-only)
    ('instance', 'job.rollup_equal_weights',                                  'bool', NULL,  NULL, true,  NULL),
    -- 1 feature flag (instance-only)
    ('instance', 'feature.reputation_v1_decay_enabled',                       'bool', NULL,  NULL, false, NULL)
ON CONFLICT (scope, key, valid_from) DO NOTHING;

-- Total: 8 + 8 + 1 + 2 + 2 + 1 + 2 + 1 + 1 = 26 net-new rows.
