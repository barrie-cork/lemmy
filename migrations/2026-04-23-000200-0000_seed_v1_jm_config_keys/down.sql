-- Reverse of 2026-04-23-000200-0000_seed_v1_jm_config_keys up.sql.
-- Deletes exactly the 27 v1-JM-a keys seeded by the companion up.sql.
-- The parity tests + config_parity_round_trip e2e walk ensures drift between
-- this list and up.sql's INSERT row list is caught at test time.
DELETE FROM governance_config
WHERE scope = 'instance' AND key IN (
    'jury.panel_size.regular.minor',
    'jury.panel_size.regular.moderate',
    'jury.panel_size.regular.severe',
    'jury.panel_size.founder.minor',
    'jury.panel_size.founder.moderate',
    'jury.panel_size.founder.severe',
    'jury.panel_size.probation.minor',
    'jury.panel_size.probation.moderate',
    'jury.panel_size.probation.severe',
    'jury.quorum_fraction.minor',
    'jury.quorum_fraction.moderate',
    'jury.quorum_fraction.severe',
    'jury.threshold_fraction.minor',
    'jury.threshold_fraction.moderate',
    'jury.threshold_fraction.severe',
    'jury.constraints.no_majority_from_same_sponsor_cluster',
    'jury.constraints.geographic_diversity_preferred',
    'jury.constraints.no_recent_juror_repeat',
    'jury.constraints.juror_cooldown_days',
    'jury.constraints.no_same_endorsement_chain',
    'jury.constraints.max_retries_before_relax',
    'jury.max_concurrent_assignments_per_juror_total',
    'appeal.panel_size_multiplier',
    'appeal.panel_size_floor_increment',
    'appeal.threshold_tier_bump',
    'appeal.window_days',
    'appeal.auto_select_on_appeal_acceptance'
);
