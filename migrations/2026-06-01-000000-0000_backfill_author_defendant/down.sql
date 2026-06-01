-- Reverse of ADR-017 task 3 up.sql.
-- Guarded reverse-backfill: nulls target_person_id ONLY on post/comment cases
-- whose current value still equals the content author's creator_id (i.e. rows
-- this migration's up.sql could have set). Person-targeted cases, and any
-- post/comment case whose target_person_id was set to some other person by a
-- different code path, are left untouched.
-- ============================================================
-- ADR exception trail (protected governance table — moderation_case)
-- ============================================================
-- DATA-only reverse UPDATE. governance_log untouched (ADR-008); integer FK
-- only, no PII (ADR-015); reversibility per ADR-010 / ADR-017.
--
-- Orphan-safe: if the content was hard-deleted after up ran, the subquery
-- returns NULL, the equality is NULL (not true), and the row is left as-is.
-- ============================================================

UPDATE moderation_case mc
SET target_person_id = NULL
WHERE mc.target_person_id IS NOT NULL
  AND mc.target_type IN ('Post', 'Comment')
  AND (
    mc.target_person_id = (SELECT p.creator_id FROM post p    WHERE p.id = mc.target_post_id)
    OR mc.target_person_id = (SELECT c.creator_id FROM comment c WHERE c.id = mc.target_comment_id)
  );
