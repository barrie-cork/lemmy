CREATE TABLE rule_set_version (
    id SERIAL PRIMARY KEY,
    community_id INTEGER NOT NULL REFERENCES community (id) ON DELETE CASCADE,
    version INTEGER NOT NULL,
    parent_id INTEGER REFERENCES rule_set_version (id),
    text_sha256 BYTEA NOT NULL,
    rule_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_by INTEGER REFERENCES person (id) ON DELETE RESTRICT,
    UNIQUE (community_id, version)
);

CREATE INDEX idx_rule_set_version_community_created
    ON rule_set_version (community_id, created_at DESC);
