-- Reverse of v1-JM-a cr-9-fix enum migration. Drop the type if it
-- exists — resilient to a partial forward-apply + revert cycle where
-- the type might already have been dropped by a prior revert.
DROP TYPE IF EXISTS jury_constraint_relaxation_reason;
