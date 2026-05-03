-- Reverse of v1-SL-a task 1 (split half 1 of 2).
-- Postgres does not support DROP VALUE without a full type rebuild;
-- the three new variants stay as orphan values in case_status (per
-- Phase 5b Restoration precedent + PRD §3.4 doc-comment).
-- This file intentionally has no DDL.
SELECT 1;
