CREATE TABLE jury_pool (
    id SERIAL PRIMARY KEY,
    community_id INTEGER REFERENCES community (id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    eligible_from TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (community_id, person_id)
);

CREATE TABLE jury_assignment (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    person_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    status jury_assignment_status NOT NULL DEFAULT 'Selected',
    selected_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    responded_at TIMESTAMPTZ,
    submitted_at TIMESTAMPTZ,
    UNIQUE (case_id, person_id)
);

CREATE INDEX idx_jury_assignment_person_status ON jury_assignment (person_id, status);

CREATE TABLE jury_vote (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    juror_id INTEGER NOT NULL REFERENCES person (id) ON DELETE CASCADE,
    decision jury_decision NOT NULL,
    rationale TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (case_id, juror_id)
);
