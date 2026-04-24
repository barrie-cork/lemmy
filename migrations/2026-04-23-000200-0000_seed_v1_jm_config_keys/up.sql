-- v1-JM-a task 7: seed 27 jury-mechanics-owned governance_config rows.
--
-- Authoritative scope (per PRD §10 defaults matrix): 9 panel_size cells
-- (status × severity) + 3 quorum_fraction + 3 threshold_fraction + 5
-- jury.constraints.* (4 bool + 1 int cooldown) + 2 jury.* (max_retries_before_relax,
-- max_concurrent_assignments_per_juror_total) + 5 appeal.* = 27 keys total.
-- Count reconciles to EXPECTED_SEED_COUNT_V1_JM = 27 in
-- crates/api/api/src/governance/config.rs.
--
-- Distinct from v1-AD-a's `jury.severity_thresholds.*` text display strings
-- (which are human labels like "60%"/"75%") and from v1-AD-a's coarse
-- `jury.diversity_constraints_enabled` toggle. Both coexist: the AD-a
-- coarse toggle acts as a global kill-switch; these granular JM-a keys
-- let communities tune individual constraints (no_majority_from_sponsor,
-- geographic_diversity, no_recent_juror_repeat, cooldown, endorsement_chain).
--
-- Distinct from v1-AD-a's `jury.appeal_panel_size_increase` (v0-era simple
-- int increment) — JM-a ships the multiplier + floor_increment formula
-- pair. The AD-a key remains seeded but is not read by v1-JM-c/d code.
--
-- Byte-for-byte the same key/value/type tuple set as the v1-JM-a additions
-- block in SEEDED_KEYS_WITH_CONSTS. The parity tests
-- `every_seeded_key_has_const_fallback` + `every_seeded_key_has_metadata`
-- + the e2e `config_parity_round_trip` walk this list and fail closed on
-- drift.
--
-- Idempotency: every row pins `valid_from` to a STABLE LITERAL — the
-- seed-migration timestamp `2026-04-23T00:02:00Z` — so that reruns target
-- the same row under the governance_config unique index on
-- (scope, key, valid_from) and `ON CONFLICT DO NOTHING` is a true no-op.
-- Without the literal, `valid_from` defaults to `now()` and each rerun
-- inserts a duplicate active row (cr-10 of PR #92). Pre-existing AD-a +
-- Phase 5a seeds have the same bug; retrofit tracked separately.
INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text, valid_from) VALUES
    ('instance', 'jury.panel_size.regular.minor',                          'int',   5,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.regular.moderate',                       'int',   5,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.regular.severe',                         'int',   7,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.founder.minor',                          'int',   5,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.founder.moderate',                       'int',   7,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.founder.severe',                         'int',   9,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.probation.minor',                        'int',   3,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.probation.moderate',                     'int',   5,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.panel_size.probation.severe',                       'int',   5,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.quorum_fraction.minor',                             'float', NULL,  0.6,    NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.quorum_fraction.moderate',                          'float', NULL,  0.6,    NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.quorum_fraction.severe',                            'float', NULL,  0.71,   NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.threshold_fraction.minor',                          'float', NULL,  0.5001, NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.threshold_fraction.moderate',                       'float', NULL,  0.6,    NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.threshold_fraction.severe',                         'float', NULL,  0.75,   NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.constraints.no_majority_from_same_sponsor_cluster', 'bool',  NULL,  NULL, true,  NULL,   '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.constraints.geographic_diversity_preferred',        'bool',  NULL,  NULL, true,  NULL,   '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.constraints.no_recent_juror_repeat',                'bool',  NULL,  NULL, true,  NULL,   '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.constraints.juror_cooldown_days',                   'int',   7,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.constraints.no_same_endorsement_chain',             'bool',  NULL,  NULL, false, NULL,   '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.constraints.max_retries_before_relax',              'int',   5,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'jury.max_concurrent_assignments_per_juror_total',        'int',   2,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'appeal.panel_size_multiplier',                           'float', NULL,  1.5,    NULL, NULL, '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'appeal.panel_size_floor_increment',                      'int',   2,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'appeal.threshold_tier_bump',                             'int',   1,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'appeal.window_days',                                     'int',   7,     NULL, NULL, NULL,    '2026-04-23T00:02:00Z'::timestamptz),
    ('instance', 'appeal.auto_select_on_appeal_acceptance',                'bool',  NULL,  NULL, true,  NULL,   '2026-04-23T00:02:00Z'::timestamptz)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
