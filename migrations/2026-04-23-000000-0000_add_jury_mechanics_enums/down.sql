-- PR #92 cr-6: `IF EXISTS` makes the revert resilient to re-runs and partial
-- down-migrations (matches the pattern in cr-9's add_jury_constraint_relaxation_reason_enum/down.sql).
DROP TYPE IF EXISTS jury_assignment_role;
DROP TYPE IF EXISTS case_status_tier;
DROP TYPE IF EXISTS severity_tier;
