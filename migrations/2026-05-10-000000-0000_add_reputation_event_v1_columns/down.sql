-- Reverse of v1-RT-r1 Task 1 up.sql.
ALTER TABLE reputation_event DROP COLUMN IF EXISTS source_event_type;
DROP INDEX IF EXISTS reputation_event_dedupe_key_partial_idx;
ALTER TABLE reputation_event DROP COLUMN IF EXISTS dedupe_key;
DROP TYPE IF EXISTS reputation_event_source_type;
