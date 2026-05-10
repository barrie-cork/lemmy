-- Reverse of v1-RT-r1 Task 2 up.sql.
ALTER TABLE sponsor_allowlist DROP COLUMN IF EXISTS note;
ALTER TABLE sponsor_allowlist DROP COLUMN IF EXISTS added_by_admin_id;
ALTER TABLE sponsor_allowlist ALTER COLUMN community_id SET NOT NULL;
-- NOTE: SET NOT NULL fails if any rows have community_id IS NULL.
-- r4-allowlist-strategy is the first writer of NULL rows; if r4 has
-- shipped + populated NULL rows before this down.sql is invoked, the
-- DBA must DELETE FROM sponsor_allowlist WHERE community_id IS NULL
-- first.
