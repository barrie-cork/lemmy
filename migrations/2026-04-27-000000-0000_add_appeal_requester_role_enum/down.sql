-- Reverse of v1-JM-d Task 1 part A enum migration. Drop the type if it
-- exists — resilient to a partial forward-apply + revert cycle where
-- the type might already have been dropped by a prior revert. The
-- companion 2026-04-27-000100 down.sql drops the dependent column on
-- `appeal` first; this migration runs after that, so the type has no
-- remaining column dependency at drop time.
DROP TYPE IF EXISTS appeal_requester_role;
