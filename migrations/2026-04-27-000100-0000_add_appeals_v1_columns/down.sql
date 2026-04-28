-- Reverse of v1-JM-d Task 1 part B in LIFO order. Drop the new columns
-- on appeal + moderation_case first; the companion 2026-04-27-000000
-- down.sql then drops the appeal_requester_role enum type after this
-- migration releases the dependency.
--
-- ============================================================
-- ADR exception trail (protected governance tables: moderation_case, appeal)
-- ============================================================
-- .coderabbit.yaml migrations/** rule flags DROP COLUMN on protected
-- governance tables. This down.sql drops 3 appeal columns + 1
-- moderation_case column in LIFO order so the companion up.sql is
-- reversible per ADR-010's staged-release reversibility requirement.
--
-- No row data other than the columns themselves is touched. No
-- governance_log row is mutated (ADR-008 append-only preserved). No
-- actor_pseudonym row is mutated (ADR-015 pseudonymisation preserved).
--
-- IF EXISTS keeps each statement idempotent across partial up/down
-- cycles (per .claude/rules/cargo-output-capture.md companion: a
-- partial revert that already dropped one column should not poison the
-- rest of the down).
--
-- Authority trail:
--   - Plan:  .claude/PRPs/plans/v1-jury-mechanics-d.plan.md §10.1, §13 Task 1
--   - PRD:   .claude/PRPs/prds/v1-jury-mechanics.prd.md §6.1, §6.4, §6.6, §9.3
--   - ADR-010: reversibility requirement
--   - ADR-008 / ADR-015: respected (governance_log + actor_pseudonym untouched)
-- ============================================================

ALTER TABLE moderation_case DROP COLUMN IF EXISTS winning_decision;

ALTER TABLE appeal DROP COLUMN IF EXISTS threshold_count_snapshot;
ALTER TABLE appeal DROP COLUMN IF EXISTS panel_size_snapshot;
ALTER TABLE appeal DROP COLUMN IF EXISTS requester_role;
