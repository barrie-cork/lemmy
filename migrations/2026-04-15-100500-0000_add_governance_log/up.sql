CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE governance_log (
    id BIGSERIAL PRIMARY KEY,
    prev_hash BYTEA NOT NULL,
    entry_hash BYTEA NOT NULL,
    entry_kind TEXT NOT NULL,
    payload JSONB NOT NULL,
    actor_pseudonym TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    signature BYTEA
);

CREATE INDEX idx_governance_log_created_at ON governance_log (created_at);
CREATE INDEX idx_governance_log_entry_kind ON governance_log (entry_kind);
