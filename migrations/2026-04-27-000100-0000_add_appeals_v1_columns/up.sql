-- v1-JM-d Task 1 part B — ALTER appeal + moderation_case for v1 appeals.
-- ============================================================
-- ADR exception trail (protected governance tables: moderation_case, appeal)
-- ============================================================
-- Additive ALTER (ADD COLUMN only). ADR-008 / ADR-015 satisfied: no DROP,
-- no UPDATE-in-place; new columns carry typed enum / decision-enum (no
-- free-text PII surface).
--
-- Controlling ADR: ADR-010 (staged releases) authorises the schema
-- extension; reversibility via companion down.sql holds.
--
-- Authority trail:
--   - Plan:  .claude/PRPs/plans/v1-jury-mechanics-d.plan.md §10.1, §10.2, §10.3, §10.4, §13 Task 1
--   - PRD:   .claude/PRPs/prds/v1-jury-mechanics.prd.md §6.1, §6.4, §6.6, §9.3
--   - ADR-010: schema extension authority (reversibility via down.sql)
--   - ADR-008 / ADR-015: respected (no governance_log mutation, no free-text PII surface)
-- ============================================================

-- Appeal table — requester_role discriminator + appeal-panel snapshot
-- columns (PRD §6.1, §6.4, §6.6 — appeal-panel rows live in
-- jury_assignment with role=Appeal but their snapshot parameters land
-- here so moderation_case.panel_size_snapshot stays immutable per
-- ADR-010).
ALTER TABLE appeal ADD COLUMN requester_role appeal_requester_role
    NOT NULL DEFAULT 'Defendant';
ALTER TABLE appeal ADD COLUMN panel_size_snapshot INTEGER;
ALTER TABLE appeal ADD COLUMN threshold_count_snapshot INTEGER;

-- moderation_case — winning_decision recorded at submit_jury_vote
-- decision time so request_appeal's reporter-rights check (PRD §6.4)
-- doesn't re-tally jury_vote rows.
ALTER TABLE moderation_case ADD COLUMN winning_decision jury_decision;

-- No backfill: pre-JM-d Decided cases have no recorded winning_decision
-- and reporter-rights eligibility is naturally false (Some(_)
-- match-arm); existing pre-JM-d Appeal rows pre-date the bounded-window
-- regression and were filed by defendants -> DEFAULT 'Defendant' is
-- correct.
