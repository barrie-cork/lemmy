#!/usr/bin/env bash
# Phase 5c task 62 probe runner — exercises bucket_query.sql against a
# throwaway schema on the pg-schema-gen testcontainer (pgautoupgrade
# pg18-alpine, port 5433). Verifies CASE WHEN + GROUP BY + ORDER BY
# pattern is valid on pg18.
#
# Run from repo root. Idempotent — drops/recreates the probe table.

set -euo pipefail

PGHOST=127.0.0.1 PGPORT=5433 PGUSER=lemmy PGPASSWORD=password PGDATABASE=lemmy
export PGHOST PGPORT PGUSER PGPASSWORD PGDATABASE

# Use docker exec to run psql inside the container — avoids needing a
# local psql install.
docker exec -i pg-schema-gen psql -U lemmy -d lemmy <<'SQL'
  CREATE SCHEMA IF NOT EXISTS probe_62;
  DROP TABLE IF EXISTS probe_62.reputation_snapshot CASCADE;
  CREATE TABLE probe_62.reputation_snapshot (
    person_id int PRIMARY KEY,
    jury_reliability int NOT NULL
  );
  INSERT INTO probe_62.reputation_snapshot VALUES
    (1, 0),     -- bucket "0"
    (2, 15),    -- bucket "1-30"
    (3, 25),    -- bucket "1-30"
    (4, 50),    -- bucket "31-80"
    (5, 150),   -- bucket "81-200"
    (6, 500);   -- bucket "200+"
  SET search_path = probe_62;
SQL

# Now run the bucket query and assert 5 rows with expected counts.
ROWS=$(docker exec -i pg-schema-gen psql -U lemmy -d lemmy -t -A -F '|' -q <<'SQL'
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
    FROM probe_62.reputation_snapshot
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
SQL
)

echo "---result---"
echo "$ROWS"
echo "---"

# Expect exactly 5 rows, ordered, with counts 1/2/1/1/1 respectively.
EXPECTED=$(printf "0|1\n1-30|2\n31-80|1\n81-200|1\n200+|1")
if [ "$ROWS" = "$EXPECTED" ]; then
  echo "PROBE_62_OK: CASE WHEN bucketing works on pg18, shape/ordering correct"
  exit 0
else
  echo "PROBE_62_FAIL: expected vs actual mismatch"
  echo "expected:"
  echo "$EXPECTED"
  exit 1
fi
