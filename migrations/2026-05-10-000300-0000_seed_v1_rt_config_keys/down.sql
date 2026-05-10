-- Reverse of v1-RT-r1 Task 4 up.sql.
-- Deletes the 26 reputation-tuning-owned rows seeded by up.sql. Does NOT
-- drop the governance_config table (that belongs to the v0 Phase 5a
-- migration `2026-04-18-000000-0000_add_governance_config`).
--
-- The 3 v1-AD-a-shipped duplicates (deltas.participation_weekly_active,
-- participation.dormancy_window_days, deltas.participation_dormant) are
-- NOT listed here — they are owned by v1-AD-a's down.sql per DQ #187.
--
-- Idempotent by construction: DELETE ... WHERE key IN (...) against an
-- empty table is a no-op; rerunning down after up+down leaves the table
-- unchanged.
DELETE FROM governance_config
WHERE scope = 'instance' AND key IN (
    'decay.reporting_accuracy.positive_half_life_days',
    'decay.reporting_accuracy.negative_half_life_days',
    'decay.jury_reliability.positive_half_life_days',
    'decay.jury_reliability.negative_half_life_days',
    'decay.participation_consistency.positive_half_life_days',
    'decay.participation_consistency.negative_half_life_days',
    'decay.endorsement_strength.positive_half_life_days',
    'decay.endorsement_strength.negative_half_life_days',
    'bounds.reporting_accuracy.floor',
    'bounds.reporting_accuracy.ceiling',
    'bounds.jury_reliability.floor',
    'bounds.jury_reliability.ceiling',
    'bounds.participation_consistency.floor',
    'bounds.participation_consistency.ceiling',
    'bounds.endorsement_strength.floor',
    'bounds.endorsement_strength.ceiling',
    'deltas.participation_juror_aligned',
    'participation.activity_threshold_comments',
    'participation.lookback_days',
    'deltas.evidence_cited',
    'deltas.evidence_bad_faith',
    'participation.evidence_cited_rationale_threshold_chars',
    'job.participation_interval_days',
    'job.rollup_interval_days',
    'job.rollup_equal_weights',
    'feature.reputation_v1_decay_enabled'
);
