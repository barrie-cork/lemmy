DROP INDEX IF EXISTS idx_governance_log_entry_kind;
DROP INDEX IF EXISTS idx_governance_log_created_at;
DROP TABLE governance_log;
-- DO NOT DROP pgcrypto — other code may use it, and IF NOT EXISTS made up.sql safe to re-run
