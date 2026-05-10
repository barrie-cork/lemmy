-- Reverse of v1-RT-r1 Task 3 up.sql.
-- Resets backfilled source_event_type variants back to column DEFAULT
-- 'Endorsement'. This is safe to run before or after Task 1 down.sql
-- (which drops the column entirely); the column existence is a pre-
-- condition for this UPDATE, so Task 1 down.sql must be applied AFTER
-- this one in a full revert sequence.
--
-- Authority trail:
--   - Plan: .claude/PRPs/plans/v1-reputation-tuning-r1.plan.md section 10.3, Task 3
--   - DQ #184 (advisor 2026-05-10)
UPDATE reputation_event
SET source_event_type = 'Endorsement'
WHERE source_event_type IN ('SponsorLiability', 'JuryVote', 'FounderSeed');
