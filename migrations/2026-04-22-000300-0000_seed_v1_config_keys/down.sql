-- Reverse of 2026-04-22-000300-0000_seed_v1_config_keys up.sql.
-- Deletes the 27 admin-dashboard-owned rows seeded by up.sql. Does NOT
-- drop the governance_config table (that belongs to the v0 Phase 5a
-- migration `2026-04-18-000000-0000_add_governance_config`).
--
-- Idempotent by construction: DELETE ... WHERE key IN (...) against an
-- empty table is a no-op; rerunning down after up+down leaves the table
-- unchanged.

DELETE FROM governance_config
WHERE scope = 'instance' AND key IN (
    'jury.severity_thresholds.minor',
    'jury.severity_thresholds.moderate',
    'jury.severity_thresholds.severe',
    'jury.diversity_constraints_enabled',
    'jury.appeal_panel_size_increase',
    'jury.deadline_window_hours',
    'decay.negative_half_life_days',
    'decay.endorsement_strength_half_life_days',
    'decay.jury_reliability_half_life_days',
    'onboarding.sponsor_min_endorsement_strength',
    'onboarding.sponsor_allowlist_table_name',
    'onboarding.provisional_membership_cooldown_days',
    'founder.founder_seal_visible_in_profile',
    'deltas.participation_weekly_active',
    'participation.dormancy_window_days',
    'deltas.participation_dormant',
    'participation.attestation_enabled',
    'federation.inbound_advisory_only',
    'federation.peer_attestation_ttl_days',
    'federation.signature_required',
    'federation.quarantine_recommendation_severity_floor',
    'federation.outbound_publish_enabled',
    'rule_set.auto_carry_in_flight_cases',
    'rule_set.text_max_bytes',
    'rule_set.version_propagation_delay_hours',
    'governance.dashboard.html_pages_enabled',
    'governance.dashboard.step_up_enforced'
);
