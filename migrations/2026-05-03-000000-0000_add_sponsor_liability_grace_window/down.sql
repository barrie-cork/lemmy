-- Reverse of v1-SL-a Task 1 up.sql.
--
-- Postgres enum-value drop is unsupported without full type rebuild —
-- the three new variants stay as orphan values in case_status (per
-- Phase 5b Restoration precedent + PRD §3.4 doc-comment). down.sql
-- documents this and recovers everything else.

-- Reverse the backfill: any case still in SponsorLiabilityPending
-- that was set by Task 1's UPDATE goes back to Decided. The
-- `grace_expires_at IS NOT NULL` guard distinguishes backfilled rows
-- from forward-progress writes (none yet at SL-a time, but
-- defence-in-depth).
UPDATE moderation_case
SET status = 'Decided',
    grace_expires_at = NULL
WHERE status = 'SponsorLiabilityPending'
  AND grace_expires_at IS NOT NULL;

DELETE FROM governance_config
WHERE scope = 'instance' AND key IN (
    'liability.grace_window_minor_hours',
    'liability.grace_window_moderate_hours',
    'liability.grace_window_severe_hours',
    'liability.grace_window_minimum_hours',
    'liability.grace_window_maximum_hours',
    'liability.grace_window_alert_threshold_hours',
    'liability.restoration_escapes_liability',
    'liability.restoration_severity_reduction_steps',
    'liability.multi_sponsor_escape_rule',
    'liability.revoke_rate_limit_per_day',
    'job.grace_check_interval_minutes',
    'job.grace_check_batch_size',
    'job.grace_check_staleness_alert_multiplier'
);

DROP INDEX IF EXISTS surety_sponsored_id_active;
DROP INDEX IF EXISTS moderation_case_grace_expires_idx;

ALTER TABLE moderation_case DROP COLUMN IF EXISTS liability_escape_reason;
ALTER TABLE moderation_case DROP COLUMN IF EXISTS grace_expires_at;

-- Postgres enum values (SponsorLiabilityPending/Fired/Escaped) remain
-- as orphan variants in the case_status type — Postgres does not
-- support DROP VALUE without a full type rebuild. This matches the
-- Phase 5b Restoration variant down.sql doc-comment precedent.
