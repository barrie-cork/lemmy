-- v1-JM-a cr-9 fix: bounded-vocabulary enum for jury_constraint_violation_log.reason_code.
--
-- PR #92 CodeRabbit cr-9 flagged the originally-proposed
-- `relaxation_reason TEXT NOT NULL` column on jury_constraint_violation_log
-- (at migration 2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql:74)
-- as an ADR-015 pseudonymisation violation: a free-text governance-audit
-- column can leak usernames, emails, or other PII supplied by call-site
-- code. Replacement: bounded enum `jury_constraint_relaxation_reason` +
-- `relaxation_metadata JSONB` (next migration).
--
-- Vocabulary matches PRD §5.3 R1/R2/R3 cascade verbatim (plus
-- `AdminOverride` from PRD §8.3's "etc." example). No `Other` variant —
-- that would re-open the ADR-015 free-text leak under an "other" label.
-- Call sites that need a new code must add a Postgres enum variant via
-- a new migration + matching Rust enum variant.
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
-- Controlling ADR: **ADR-015 (GDPR pseudonymisation)** — this enum
-- EXISTS specifically to eliminate a cr-9-flagged ADR-015 violation.
-- Authority trail:
--   - PR #92 CodeRabbit review, finding cr-9
--   - Advisor answer at `.claude/runlog/advisor-relays/cr-9-enum-vocab-answer.md`
--     (option (A) 4 values, no 'other', 2026-04-24)
--   - PRD §5.3 cascade table (small_pool / cluster_pressure / cluster_pressure_exhausted)
--   - PRD §8.3 example (admin_override)
-- ============================================================
--
-- Verbatim PascalCase variants per the DbEnum convention used by
-- `severity_tier`, `case_status_tier`, `jury_assignment_role` in the
-- companion migration at 2026-04-23-000000. PRD §5.3's snake_case
-- narrative (`small_pool`) corresponds 1:1 to the PascalCase on-disk
-- form (`SmallPool`) — same PRD-narrative-vs-on-disk relationship as
-- the other three JM-a enums (`minor` → `Minor`, etc.).
CREATE TYPE jury_constraint_relaxation_reason AS ENUM (
    'SmallPool',                -- PRD §5.3 R1: Phase 1 pool post-cooldown too small
    'ClusterPressure',          -- PRD §5.3 R2: Phase 2 sample violates sponsor-cluster constraint after N retries
    'ClusterPressureExhausted', -- PRD §5.3 R3: Phase 2 sample still violates after R2; drop constraint entirely
    'AdminOverride'             -- PRD §8.3: admin explicitly bypasses constraint cascade
);
