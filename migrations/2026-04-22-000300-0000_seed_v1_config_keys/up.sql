-- v1-AD-a task 7: seed 27 admin-dashboard-owned governance_config rows.
--
-- Authoritative scope (per plan §4.1, advisor edit #1): admin-dashboard-v1
-- PRD §5.2 enumeration minus 10 sponsor-liability-v1 rows minus
-- rule_set.active_version_id (absence-of-row IS the "no active version"
-- signal per plan §4.1 / task 6 GOTCHA). Count reconciles to
-- EXPECTED_SEED_COUNT_V1_AD = 27 in
-- crates/api/api/src/governance/config.rs.
--
-- rule_set.active_version_id is DELIBERATELY OMITTED — no const, no seed,
-- no CONFIG_KEY_METADATA entry, no const_default_int arm. v1-AD-c adds
-- all three when implementing rule-set creation.
--
-- Byte-for-byte the same key/value/type tuple set as the v1-AD-a additions
-- block in SEEDED_KEYS_WITH_CONSTS (config.rs lines 611-679). The parity
-- test `every_seeded_key_has_const_fallback` + the e2e
-- `config_parity_round_trip` walk this list and fail closed on drift.
--
-- Idempotency: every row pins valid_from to the STABLE LITERAL
-- '2026-04-22T00:03:00Z' so reruns hit the same row under the unique index
-- on (scope, key, valid_from) and ON CONFLICT DO NOTHING is a true no-op.
-- Without the literal, valid_from defaults to now() and each rerun inserts
-- a duplicate active row (same class as JM-a cr-10, fixed here retroactively
-- per audit §3.D.6). Pre-existing admin edits at valid_from = now() survive
-- reapply because the unique index treats them as distinct rows.

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text, valid_from) VALUES
    ('instance', 'jury.severity_thresholds.minor',                     'text', NULL,  NULL, NULL,  'majority',  '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'jury.severity_thresholds.moderate',                  'text', NULL,  NULL, NULL,  '60%',       '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'jury.severity_thresholds.severe',                    'text', NULL,  NULL, NULL,  '75%',       '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'jury.diversity_constraints_enabled',                 'bool', NULL,  NULL, true,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'jury.appeal_panel_size_increase',                    'int',  2,     NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'jury.deadline_window_hours',                         'int',  72,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'decay.negative_half_life_days',                      'int',  180,   NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'decay.endorsement_strength_half_life_days',          'int',  90,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'decay.jury_reliability_half_life_days',              'int',  90,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'onboarding.sponsor_min_endorsement_strength',        'int',  25,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'onboarding.sponsor_allowlist_table_name',            'text', NULL,  NULL, NULL,  'sponsor_allowlist', '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'onboarding.provisional_membership_cooldown_days',    'int',  14,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'founder.founder_seal_visible_in_profile',            'bool', NULL,  NULL, true,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'deltas.participation_weekly_active',                 'int',  1,     NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'participation.dormancy_window_days',                 'int',  30,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'deltas.participation_dormant',                       'int',  -2,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'participation.attestation_enabled',                  'bool', NULL,  NULL, false, NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'federation.inbound_advisory_only',                   'bool', NULL,  NULL, true,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'federation.peer_attestation_ttl_days',               'int',  30,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'federation.signature_required',                      'bool', NULL,  NULL, true,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'federation.quarantine_recommendation_severity_floor','text', NULL,  NULL, NULL,  'moderate',  '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'federation.outbound_publish_enabled',                'bool', NULL,  NULL, true,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    -- rule_set.active_version_id deliberately NOT seeded — absence-of-row
    -- IS the "no active version" signal. See plan §4.1 + task 6 GOTCHA.
    ('instance', 'rule_set.auto_carry_in_flight_cases',                'bool', NULL,  NULL, true,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'rule_set.text_max_bytes',                            'int',  65536, NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'rule_set.version_propagation_delay_hours',           'int',  24,    NULL, NULL,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'governance.dashboard.html_pages_enabled',            'bool', NULL,  NULL, true,  NULL,        '2026-04-22T00:03:00Z'::timestamptz),
    ('instance', 'governance.dashboard.step_up_enforced',              'bool', NULL,  NULL, false, NULL,        '2026-04-22T00:03:00Z'::timestamptz)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
