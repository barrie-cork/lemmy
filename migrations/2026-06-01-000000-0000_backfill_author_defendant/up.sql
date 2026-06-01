-- ADR-017 task 3: backfill moderation_case.target_person_id for the
-- historical post/comment-targeted tail, making the content author a
-- first-class governance defendant (appeal/sanction/reputation/liability).
-- ============================================================
-- ADR exception trail (protected governance table — moderation_case)
-- ============================================================
-- This migration is a DATA BACKFILL only (UPDATE on moderation_case).
-- No DDL changes; no column additions; no index changes; no constraint
-- changes. moderation_case has no CHECK constraint, and co-populating
-- target_post_id/target_comment_id alongside target_person_id is valid
-- (the dual-population shape already exists for the modlog table).
--
-- Authority trail:
--   - ADR-017 (99-decisions-and-open-questions.md): post/comment authors are
--     first-class defendants; target_person_id = content author's creator_id.
--   - ADR-008: governance_log is NOT touched (no audit row rewritten/inserted).
--   - ADR-015: target_person_id is an integer FK to person — NOT a direct
--     identifier; the actor_pseudonym mapping is untouched, no PII implication.
--   - ADR-010 (reversibility): the status filter below is LOAD-BEARING.
--     Jury panel size + juror exclusion read target_person_id ONLY at
--     jury-assignment time, which fires exclusively from the three statuses
--     Open / ThresholdMet / EmergencyRemove and atomically freezes
--     panel_size_snapshot. Excluding exactly those three statuses guarantees
--     this backfill cannot retroactively resize an in-flight or already-seated
--     jury. Cases past jury assignment have a frozen panel and are safe.
--
-- Enum literals are PascalCase ('Post','Comment', status literals) per the
-- DbValueStyle = verbatim Postgres enum definitions in
-- 2026-04-15-100000-0000_add_governance_enums/up.sql. Lowercase would match
-- nothing and silently invert the scope filter — do NOT lowercase.
--
-- Hard-deleted content nulls its FK (target_post_id/target_comment_id are
-- ON DELETE SET NULL), so orphaned cases fall out of the WHERE and stay NULL.
--
-- Idempotent: the `target_person_id IS NULL` guard makes re-running across
-- down/up cycles a no-op on rows already backfilled.
--
-- Smoke check at retro time:
--   SELECT status, count(*) FROM moderation_case
--   WHERE target_type IN ('Post','Comment') AND target_person_id IS NULL
--   GROUP BY status;
-- After up: Decided/Appealed/Closed/etc. post/comment rows are populated;
-- Open/ThresholdMet/EmergencyRemove rows are intentionally left NULL.
-- ============================================================

UPDATE moderation_case mc
SET target_person_id = CASE mc.target_type
    WHEN 'Post'    THEN (SELECT p.creator_id FROM post p    WHERE p.id = mc.target_post_id)
    WHEN 'Comment' THEN (SELECT c.creator_id FROM comment c WHERE c.id = mc.target_comment_id)
    ELSE mc.target_person_id
  END
WHERE mc.target_person_id IS NULL
  AND mc.target_type IN ('Post', 'Comment')
  AND (mc.target_post_id IS NOT NULL OR mc.target_comment_id IS NOT NULL)
  AND mc.status NOT IN ('Open', 'ThresholdMet', 'EmergencyRemove');
