-- no-transaction
-- Postgres does not support dropping an enum value without rebuilding the entire type.
-- Down-path is intentionally a no-op; rolling back past this migration requires a full
-- type rebuild (DROP TYPE + CREATE TYPE + update every column that uses it).
-- See [99 OQ-003] for the reasoning behind making the variant reservation irreversible
-- at the v0 migration level.
SELECT 1;
