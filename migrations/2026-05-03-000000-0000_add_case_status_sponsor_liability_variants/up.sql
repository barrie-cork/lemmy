-- v1-SL-a task 1 (split half 1 of 2): add three SponsorLiability* variants
-- to case_status enum.
--
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-sponsor-liability.prd.md §8.5 (combined-
--     migration shape, superseded by this split).
--   - Plan: .claude/PRPs/plans/v1-sponsor-liability-a.plan.md §10.1
--     (combined skeleton, superseded by this split).
--   - Fix: .claude/PRPs/briefs/sl-a-fix-impl-1.md (DQ #122 — Postgres
--     refuses "unsafe use of new value" within the same migration).
--   - Mirror: migrations/2026-04-19-000000-0000_add_restoration_sanction_variant
--     (Phase 5b precedent for enum-only -- no-transaction migration).

-- no-transaction
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityPending';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityFired';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityEscaped';
