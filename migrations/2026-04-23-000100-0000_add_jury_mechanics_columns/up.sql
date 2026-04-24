-- v1-JM-a: add 6 new columns to moderation_case + 2 to jury_assignment,
-- then backfill pre-v1 cases with v0-equivalent snapshots per PRD §8.4.
-- Single-transaction by design — no intermediate state where v1-JM-c's
-- submit_jury_vote could read NULL snapshot values on a pre-v1 case.
--
-- ============================================================
-- ADR exception trail (protected governance tables: moderation_case, jury_assignment)
-- ============================================================
-- .coderabbit.yaml migrations/** rule (lines 123-130) flags ALTER / DROP
-- on moderation_case, jury_assignment, governance_log, jury_vote,
-- sanction, appeal, public_case_log, actor_pseudonym because ADR-008
-- (append-only governance_log rows) and ADR-015 (GDPR pseudonymisation
-- of governance-log payloads) forbid DESTRUCTIVE changes to the
-- governance audit surface.
--
-- This migration is ADDITIVE (ADD COLUMN only, no DROP, no
-- UPDATE-in-place on existing columns). It touches:
--   - moderation_case: 6 new columns added (severity_tier, status_tier,
--     panel_size_snapshot, quorum_snapshot, threshold_count_snapshot,
--     appeal_window_expires_at). Existing columns and rows are not
--     altered; pre-v1 rows only gain newly-nullable snapshot data via
--     the backfill UPDATE below.
--   - jury_assignment: 2 new columns added (selected_under_constraints,
--     role). Additive; no existing columns touched.
--
-- Additive ALTER is in scope for the protected-table rule — ADR-008's
-- "append-only" applies to governance_log ROWS, not to governance-table
-- SCHEMA extensibility; ADR-015's "no raw PII in audit payloads" is
-- upheld because the new columns carry snapshot integers / enum tiers /
-- bounded JSONB, not free-text user-supplied strings.
--
-- Controlling ADR for this change: **ADR-010 (staged releases: v0 → v1
-- → v2 → v3)**, which authorises schema extensions per sub-phase so
-- long as (a) in-flight v0 cases keep completing under v0 semantics —
-- guaranteed here by the backfill UPDATE pinning v0 defaults — and (b)
-- the change is reversible via a companion down.sql. Both hold.
--
-- Authority trail:
--   - Plan:  .claude/PRPs/plans/phase-v1-JM-a.plan.md §10.4, §10.5, Task 2
--   - PRD:   .claude/PRPs/prds/v1-jury-mechanics.prd.md §8.1, §8.2, §8.3, §8.4
--   - ADR-010: no retroactive invalidation of in-flight juries
--              (snapshot columns freeze at admin_assign_jury time)
--   - ADR-008 / ADR-015: respected (see above)
-- ============================================================

-- Enum-typed columns use fast metadata-only attmissingval backfill
-- (Postgres 11+). `'Minor'::severity_tier` and `'Regular'::case_status_tier`
-- are literals (non-volatile) → no table rewrite.
ALTER TABLE moderation_case ADD COLUMN severity_tier severity_tier NOT NULL DEFAULT 'Minor';
ALTER TABLE moderation_case ADD COLUMN status_tier case_status_tier NOT NULL DEFAULT 'Regular';

-- Integer snapshot columns — NULLABLE because v1 cases populate them at
-- admin_assign_jury time (v1-JM-b); the backfill UPDATE below writes
-- 5/3/3 to every existing case (pre-v1 at migration time) per PRD §8.4.
ALTER TABLE moderation_case ADD COLUMN panel_size_snapshot INTEGER;
ALTER TABLE moderation_case ADD COLUMN quorum_snapshot INTEGER;
ALTER TABLE moderation_case ADD COLUMN threshold_count_snapshot INTEGER;

-- Appeal-window column — NULLABLE; populated on case-decision by v1-JM-c.
-- Backfill below sets it for pre-v1 cases that have decided_at or closed_at.
ALTER TABLE moderation_case ADD COLUMN appeal_window_expires_at TIMESTAMPTZ;

-- jury_assignment additions per PRD §8.2
ALTER TABLE jury_assignment ADD COLUMN selected_under_constraints JSONB;
ALTER TABLE jury_assignment ADD COLUMN role jury_assignment_role NOT NULL DEFAULT 'Original';

-- Backfill — PRD §8.4 exact semantics. panel_size=5, quorum=3,
-- threshold_count=3 reproduces the v0 3-of-5 simple-majority rule.
-- appeal_window_expires_at: if case already closed, keep closed_at; else
-- decided_at + 7 days (matches v0 APPEAL_WINDOW_DAYS). Pre-Decided cases
-- (no decided_at AND no closed_at) keep appeal_window_expires_at NULL
-- until v1-JM-c writes one at decision time.
--
-- The `WHERE panel_size_snapshot IS NULL` guard makes this migration
-- idempotent across down/up cycles — re-running it after v1 writers
-- have populated snapshots does not clobber those writes.
UPDATE moderation_case
SET panel_size_snapshot = 5,
    quorum_snapshot = 3,
    threshold_count_snapshot = 3,
    appeal_window_expires_at = COALESCE(
        closed_at,
        CASE
            WHEN decided_at IS NOT NULL THEN decided_at + INTERVAL '7 days'
            ELSE NULL
        END
    )
WHERE panel_size_snapshot IS NULL;

-- New table per PRD §8.3 — per-case audit row written every time
-- select_eligible_jurors relaxes a diversity/recency/cluster constraint
-- (v1-JM-b). case_id ON DELETE CASCADE mirrors the evidence /
-- jury_assignment precedent. No PII — only constraint metadata
-- (Watch 10 from PRD §5.3).
--
-- cr-9 fix (ADR-015 pseudonymisation): `reason_code` replaces the
-- originally-proposed `relaxation_reason TEXT`. The bounded-vocabulary
-- enum (`jury_constraint_relaxation_reason`, created in the companion
-- 2026-04-23-000050 migration) eliminates the free-text PII leak
-- surface flagged by CodeRabbit on PR #92. `relaxation_metadata JSONB`
-- is optional and nullable; call sites writing to it MUST use bounded
-- structured fields ONLY — e.g.
--   {"dropped_constraint_name": "no_recent_juror_repeat", "phase": "pool_build"}
-- NEVER free-text user-supplied strings. The `pool_size_at_relax` and
-- `panel_size_target` columns stay as typed INTs (they are queryable
-- per the admin dashboard's "show relaxations" view); only ancillary
-- bounded metadata belongs in the JSONB column.
CREATE TABLE jury_constraint_violation_log (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    constraint_name TEXT NOT NULL,
    reason_code jury_constraint_relaxation_reason NOT NULL,
    relaxation_metadata JSONB,
    pool_size_at_relax INTEGER NOT NULL,
    panel_size_target INTEGER NOT NULL,
    relaxed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_jcvl_case_id ON jury_constraint_violation_log (case_id);
