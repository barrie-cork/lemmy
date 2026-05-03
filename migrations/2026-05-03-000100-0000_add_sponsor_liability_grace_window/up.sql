-- v1-SL-a task 1 (split half 2 of 2): columns + indexes + seeds + backfill.
-- ============================================================
-- ADR exception trail (protected governance tables)
-- ============================================================
-- ADDITIVE only: ADD COLUMN, CREATE INDEX, INSERT ... ON CONFLICT DO
-- NOTHING, UPDATE ... (backfill only, bounded by `decided_at >
-- now() - INTERVAL '24 hours'` per ADR-010 won't-disadvantage). No
-- DROP, no ALTER on existing columns, no UPDATE on already-fired
-- cases. No -- no-transaction needed (no ALTER TYPE in this file).
--
-- Controlling ADR: ADR-010 (staged releases).
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-sponsor-liability.prd.md §8.5
--     (combined shape, superseded by this split).
--   - Plan: .claude/PRPs/plans/v1-sponsor-liability-a.plan.md §10.1
--     (combined skeleton, superseded by this split).
--   - Fix: .claude/PRPs/briefs/sl-a-fix-impl-1.md (DQ #122 —
--     Postgres refuses unsafe use of new enum value within the
--     same migration that added it).
--   - DQ #115 (advisor 2026-05-03): grace_window_*_hours stored as
--     raw integer hours, NOT micros-scaled.
-- ============================================================

-- (No -- no-transaction; this migration uses normal Diesel
-- transaction wrapping. The case_status enum values referenced in
-- the backfill UPDATE were added by the prior migration
-- 2026-05-03-000000-0000_add_case_status_sponsor_liability_variants,
-- which committed before this migration begins.)

ALTER TABLE moderation_case ADD COLUMN grace_expires_at TIMESTAMPTZ;
COMMENT ON COLUMN moderation_case.grace_expires_at IS
    'Per OQ-025 v1 sponsor-liability grace window. Set when transitioning
     Decided -> SponsorLiabilityPending; locked thereafter. NULL for cases
     not in the grace lifecycle (NoAction outcomes, no-sponsor target,
     v0 backfill exclusions).';

ALTER TABLE moderation_case ADD COLUMN liability_escape_reason JSONB;
COMMENT ON COLUMN moderation_case.liability_escape_reason IS
    'Per OQ-025 + OQ-V1-SL-05: structured escape-reason payload.
     Schema (version: 1):
       {"version": 1,
        "reason": "sponsor_revoked"|"restoration_completed"|"admin_override",
        "actor_pseudonym": "<scrubbed via ADR-015>",
        "endorsement_id": <i32>|null,
        "restoration_id": <i32>|null}.
     NULL for non-escaped cases.';

CREATE INDEX moderation_case_grace_expires_idx
    ON moderation_case (grace_expires_at)
    WHERE status = 'SponsorLiabilityPending';
COMMENT ON INDEX moderation_case_grace_expires_idx IS
    'Per PRD §8.1: scheduler primary access path. Partial index keeps
     the index small (~handful of pending cases at any time) and bounds
     the SL-c grace-check batch query in O(rows-pending).';

CREATE INDEX surety_sponsored_id_active
    ON surety (sponsored_id, sponsor_id)
    WHERE revoked_at IS NULL;
COMMENT ON INDEX surety_sponsored_id_active IS
    'Per Issue #24 (CodeRabbit, PR #10). Speeds up
     jury_common::select_eligible_jurors EXISTS subquery and
     apply_sponsor_liability sponsor enumeration. Partial WHERE matches
     the existing "active sureties" filter pattern in both call sites.';

INSERT INTO governance_config (scope, key, value_type, value_int, value_float, value_bool, value_text) VALUES
    -- 6 grace-window hour keys (raw integer hours per DQ #115)
    ('instance', 'liability.grace_window_minor_hours',                   'int',   24,    NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_moderate_hours',                'int',   72,    NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_severe_hours',                  'int',   168,   NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_minimum_hours',                 'int',   1,     NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_maximum_hours',                 'int',   720,   NULL, NULL,  NULL),
    ('instance', 'liability.grace_window_alert_threshold_hours',         'int',   24,    NULL, NULL,  NULL),
    -- 2 restoration-escape keys (per PRD §7.3)
    ('instance', 'liability.restoration_escapes_liability',              'bool',  NULL,  NULL, true,  NULL),
    ('instance', 'liability.restoration_severity_reduction_steps',       'int',   0,     NULL, NULL,  NULL),
    -- 1 multi-sponsor escape rule (per PRD §13.1 OQ-V1-SL-01)
    ('instance', 'liability.multi_sponsor_escape_rule',                  'text',  NULL,  NULL, NULL,  'any_revocation'),
    -- 1 revocation rate-limit (per PRD §12.1)
    ('instance', 'liability.revoke_rate_limit_per_day',                  'int',   5,     NULL, NULL,  NULL),
    -- 3 grace-check scheduler keys (per PRD §6.4)
    ('instance', 'job.grace_check_interval_minutes',                     'int',   5,     NULL, NULL,  NULL),
    ('instance', 'job.grace_check_batch_size',                           'int',   100,   NULL, NULL,  NULL),
    ('instance', 'job.grace_check_staleness_alert_multiplier',           'float', NULL,  2.0,  NULL,  NULL)
ON CONFLICT (scope, key, valid_from) DO NOTHING;

UPDATE moderation_case
SET status = 'SponsorLiabilityPending',
    grace_expires_at = decided_at + INTERVAL '24 hours'
WHERE status = 'Decided'
  AND decided_at IS NOT NULL
  AND decided_at > now() - INTERVAL '24 hours'
  AND target_person_id IS NOT NULL
  AND id IN (
    SELECT mc.id
    FROM moderation_case mc
    WHERE EXISTS (
      SELECT 1 FROM surety s
      WHERE s.sponsored_id = mc.target_person_id
        AND s.revoked_at IS NULL
    )
    AND EXISTS (
      SELECT 1 FROM sanction sa
      WHERE sa.case_id = mc.id
    )
    AND NOT EXISTS (
      SELECT 1 FROM reputation_event re
      WHERE re.source_case_id = mc.id
        AND re.reason = 'sponsor_liability_applied'
    )
  );
