-- v1-JM-d Task 1 part A — new Postgres enum for appeal requester-role
-- discriminator. Mirrors the case_status_tier / severity_tier /
-- jury_assignment_role pattern from JM-a 2026-04-23-000000.
--
-- ============================================================
-- ADR exception trail (new Postgres type — not a protected-table ALTER)
-- ============================================================
-- `CREATE TYPE` is not flagged by the .coderabbit.yaml migrations/**
-- protected-table rule (which targets ALTER/DROP/TRUNCATE on existing
-- governance tables). This is a new enum type only; no existing
-- governance-table row is touched, no governance_log entry is mutated,
-- no actor_pseudonym row is rebound. ADR-008 and ADR-015 are preserved
-- by construction (nothing destructive happens).
--
-- Controlling ADR: **ADR-010 (staged releases)** authorises the schema
-- extension; reversibility via companion down.sql holds.
--
-- Authority trail:
--   - Plan:  .claude/PRPs/plans/v1-jury-mechanics-d.plan.md §10.1, §10.3, §13 Task 1
--   - PRD:   .claude/PRPs/prds/v1-jury-mechanics.prd.md §6.4, §9.3
--   - ADR-010: schema extension authority (reversibility via down.sql)
--   - ADR-008 / ADR-015: respected (no governance_log mutation, no PII surface)
-- ============================================================
--
-- Verbatim PascalCase variants per the DbEnum convention used by
-- `severity_tier`, `case_status_tier`, `jury_assignment_role`.
CREATE TYPE appeal_requester_role AS ENUM (
    'Defendant',
    'OriginalReporter'
);
