CREATE TABLE sponsor_allowlist (
    id SERIAL PRIMARY KEY,
    community_id INTEGER NOT NULL REFERENCES community (id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (community_id, person_id)
);
