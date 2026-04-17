-- Phase 5a task 51: person.membership_state deferred-enforcement column.
--
-- Per [99 OQ-016]: ship the column in v0 with DEFAULT 'member' so v1 can
-- flip `config.onboarding.enforce_membership_state = true` without a schema
-- migration on `person` (which is expensive at scale).
--
-- Postgres 11+ fast-path: ADD COLUMN NOT NULL DEFAULT <non-volatile> is
-- metadata-only via pg_attribute.attmissingval. `'member'::membership_state`
-- is a literal (non-volatile) — no table rewrite. v1-era large-instance
-- deployments should still verify with EXPLAIN before running in prod.

CREATE TYPE membership_state AS ENUM ('member', 'provisional', 'suspended');
ALTER TABLE person ADD COLUMN membership_state membership_state NOT NULL DEFAULT 'member';

COMMENT ON COLUMN person.membership_state IS
    'Deferred-enforcement per [99 OQ-016]. NOT READ by any v0 handler. v1 flips `onboarding.enforce_membership_state` to activate. Grep-guarded by scripts/brehon/lint-no-membership-read.sh.';
