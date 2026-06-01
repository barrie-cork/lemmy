-- smoke-test-setup.sql
-- Idempotent bootstrap patches for single-user Brehon smoke testing.
-- Run once after a fresh stack spin-up, before Phase 3.
--
-- Usage:
--   docker exec docker-postgres-1 psql -U lemmy -d lemmy -f /dev/stdin < docker/scripts/smoke-test-setup.sql
-- or:
--   docker exec -i docker-postgres-1 psql -U lemmy -d lemmy < docker/scripts/smoke-test-setup.sql
--
-- Safe to re-run on an already-patched instance (all statements are idempotent).

-- 1. Admin account: enable jury eligibility
--    Fresh instances have accepted_application=false; jury selector excludes them.
UPDATE local_user
   SET accepted_application = true
 WHERE person_id = (SELECT id FROM person WHERE name = 'lemmy' LIMIT 1)
   AND accepted_application = false;

-- 2. Jury config: set panel_size to 1 across all severity tiers
--    Fresh single-user instance has only 1 eligible juror; default panel_size=5 blocks assign-jury.
UPDATE governance_config
   SET value_int = 1
 WHERE scope = 'instance'
   AND key LIKE 'jury.panel_size%'
   AND value_int > 1;

-- 3. Jury config: set quorum to 1 (must be <= panel_size)
UPDATE governance_config
   SET value_int = 1
 WHERE scope = 'instance'
   AND key = 'jury.quorum'
   AND value_int > 1;

-- 4. Sponsorship: remove account-age gate for endorsement testing
--    Default is 30 days; fresh accounts are 0 days old, causing not_found on endorsement.
UPDATE governance_config
   SET value_int = 0
 WHERE scope = 'instance'
   AND key = 'onboarding.sponsor_min_account_age_days'
   AND value_int > 0;

-- 5. Sponsorship: remove minimum endorsement-strength requirement
--    Default is 25; fresh instances have no endorsement history.
UPDATE governance_config
   SET value_int = 0
 WHERE scope = 'instance'
   AND key = 'onboarding.sponsor_min_endorsement_strength'
   AND value_int > 0;

-- Verify patches applied
SELECT 'local_user.accepted_application' AS patch,
       CASE WHEN accepted_application THEN 'OK' ELSE 'FAILED' END AS status
  FROM local_user
  JOIN person ON person.id = local_user.person_id
 WHERE person.name = 'lemmy'
UNION ALL
SELECT 'jury.quorum = 1',
       CASE WHEN value_int = 1 THEN 'OK' ELSE 'FAILED (value=' || value_int || ')' END
  FROM governance_config
 WHERE scope = 'instance' AND key = 'jury.quorum'
UNION ALL
SELECT 'onboarding.sponsor_min_account_age_days = 0',
       CASE WHEN value_int = 0 THEN 'OK' ELSE 'FAILED (value=' || value_int || ')' END
  FROM governance_config
 WHERE scope = 'instance' AND key = 'onboarding.sponsor_min_account_age_days'
UNION ALL
SELECT 'onboarding.sponsor_min_endorsement_strength = 0',
       CASE WHEN value_int = 0 THEN 'OK' ELSE 'FAILED (value=' || value_int || ')' END
  FROM governance_config
 WHERE scope = 'instance' AND key = 'onboarding.sponsor_min_endorsement_strength';
