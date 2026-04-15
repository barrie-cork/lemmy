CREATE TABLE surety (
    id SERIAL PRIMARY KEY,
    sponsor_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    sponsored_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    UNIQUE (sponsor_id, sponsored_id, community_id)
);

CREATE TABLE endorsement (
    id SERIAL PRIMARY KEY,
    from_person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    to_person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    UNIQUE (from_person_id, to_person_id, community_id)
);

CREATE TABLE reputation_event (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    dimension reputation_dimension NOT NULL,
    delta INTEGER NOT NULL,
    source_case_id INTEGER REFERENCES moderation_case (id) ON DELETE SET NULL,
    source_report_id INTEGER,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ
);

CREATE INDEX idx_reputation_event_person_community_created ON reputation_event (person_id, community_id, created_at);

CREATE TABLE reputation_snapshot (
    id SERIAL PRIMARY KEY,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    reporting_accuracy INTEGER NOT NULL DEFAULT 0,
    jury_reliability INTEGER NOT NULL DEFAULT 0,
    participation_consistency INTEGER NOT NULL DEFAULT 0,
    endorsement_strength INTEGER NOT NULL DEFAULT 0,
    jury_eligible BOOLEAN NOT NULL DEFAULT FALSE,
    trusted_reporter BOOLEAN NOT NULL DEFAULT FALSE,
    calculated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (person_id, community_id)
);
