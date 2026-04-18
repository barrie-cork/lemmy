-- Phase 5c task 62 probe: verify `CASE WHEN` bucketing shape works on
-- postgres 18 (the testcontainer image). This is the pattern
-- `admin_reputation_stats` uses for its 4 dimension histograms.
--
-- Expected shape: 5 rows, columns (bucket text, count bigint).
--
-- Run: psql -f scratch/phase-5c-probes/bucket_query.sql
--
-- Prereq: the target database has a `reputation_snapshot` table with
-- a `jury_reliability int` column. If tested against the e2e
-- testcontainer mid-test, insert a few sample rows first; if tested
-- against a fresh shell, the COUNT will be 0 in every bucket (that's
-- fine — shape is what we're verifying, not the data).

SELECT bucket, COUNT(*)::bigint AS count
FROM (
  SELECT
    CASE
      WHEN jury_reliability = 0 THEN '0'
      WHEN jury_reliability BETWEEN 1 AND 30 THEN '1-30'
      WHEN jury_reliability BETWEEN 31 AND 80 THEN '31-80'
      WHEN jury_reliability BETWEEN 81 AND 200 THEN '81-200'
      ELSE '200+'
    END AS bucket
  FROM reputation_snapshot
) s
GROUP BY bucket
ORDER BY
  CASE bucket
    WHEN '0' THEN 0
    WHEN '1-30' THEN 1
    WHEN '31-80' THEN 2
    WHEN '81-200' THEN 3
    ELSE 4
  END;
