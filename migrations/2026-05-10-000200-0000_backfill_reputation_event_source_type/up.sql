-- v1-RT-r1 task 3: backfill reputation_event.source_event_type via
-- reason ILIKE precedence chain per DQ #184.
-- ============================================================
-- ADR exception trail (protected governance tables — append-only rows)
-- ============================================================
-- This migration is a DATA BACKFILL only (UPDATE on reputation_event).
-- No DDL changes; no column additions; no index changes.
-- The source_event_type column was added with NOT NULL DEFAULT 'Endorsement'
-- in 2026-05-10-000000-0000 (Task 1). This migration refines existing rows
-- that v0 emitters populated before the column existed at deploy time.
--
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-reputation-tuning.prd.md sections 7 (Backfill row) + 5.3 (source enumeration)
--   - Plan: .claude/PRPs/plans/v1-reputation-tuning-r1.plan.md section 10.3, Task 3
--   - DQ #184 (advisor 2026-05-10): use source_case_id + reason ILIKE;
--     do NOT reference non-existent endorsement_id / jury_vote_id columns
-- ============================================================

-- Precedence chain (apply in priority order — first match wins):
--   1. reason ILIKE 'sponsor_liability%' -> SponsorLiability
--   2. reason ILIKE 'jury_reliability%' OR 'jury_vote%' OR 'jury_align%' -> JuryVote
--   3. reason ILIKE 'founder_seed%' -> FounderSeed
--   4. otherwise -> Endorsement (column DEFAULT covers existing rows; no-op UPDATE)
--   5. fallback -> ManualSeed (no-op at backfill time; future-stale rows only)
--
-- The 'WHERE source_event_type = 'Endorsement'' guard on each UPDATE makes
-- this migration idempotent across down/up cycles — re-running does not
-- clobber rows already refined by a prior run.
--
-- Smoke check at retro time:
--   SELECT source_event_type, COUNT(*) FROM reputation_event GROUP BY 1;
-- On a seeded DB: at least Endorsement non-zero; other variants non-zero
-- only if v0 emitted sponsor_liability / jury_reliability / founder_seed rows.
-- On a fresh-deploy DB with no rows: zero total — correct.

-- Step 1: phase-5b sponsor_liability emitter rows -> SponsorLiability
UPDATE reputation_event
SET source_event_type = 'SponsorLiability'
WHERE source_event_type = 'Endorsement'
  AND reason ILIKE 'sponsor_liability%';

-- Step 2: phase-5b jury_mechanics emitter rows -> JuryVote
-- Matches 'jury_reliability_' prefix (v0 juror-aligned delta) plus
-- 'jury_vote%' and 'jury_align%' which may appear in hand-seeded rows.
UPDATE reputation_event
SET source_event_type = 'JuryVote'
WHERE source_event_type = 'Endorsement'
  AND (reason ILIKE 'jury_reliability%'
       OR reason ILIKE 'jury_vote%'
       OR reason ILIKE 'jury_align%');

-- Step 3: founder seed rows -> FounderSeed
UPDATE reputation_event
SET source_event_type = 'FounderSeed'
WHERE source_event_type = 'Endorsement'
  AND reason ILIKE 'founder_seed%';

-- Steps 4 + 5: Endorsement (default, covered by column DEFAULT — no UPDATE
-- needed) and ManualSeed (no-op at backfill; no known v0 reason prefix maps
-- to ManualSeed; future r3 emitters will write this variant explicitly).
