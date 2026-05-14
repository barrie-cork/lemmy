-- Reverse of 2026-04-22-000300-0000_seed_v1_config_keys up.sql.
-- Deletes exactly the 27 v1-AD-a seed rows identified by the stable
-- valid_from = '2026-04-22T00:03:00Z' literal the up.sql pins (per
-- audit §3.D.6 retrofit). Targeting the seed literal preserves any admin
-- edits at valid_from = now() that landed since seed-time — those are
-- community-authored config history and must survive revert+reapply.
-- Does NOT drop the governance_config table (that belongs to the v0
-- Phase 5a migration `2026-04-18-000000-0000_add_governance_config`).

DELETE FROM governance_config
WHERE scope = 'instance'
  AND valid_from = '2026-04-22T00:03:00Z'::timestamptz
  AND key IN (
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
