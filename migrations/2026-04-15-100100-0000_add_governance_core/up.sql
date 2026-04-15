CREATE TABLE moderation_case (
    id SERIAL PRIMARY KEY,
    community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    creator_id INTEGER REFERENCES person (id) ON DELETE SET NULL,
    target_type case_target_type NOT NULL,
    target_post_id INTEGER REFERENCES post (id) ON DELETE SET NULL,
    target_comment_id INTEGER REFERENCES comment (id) ON DELETE SET NULL,
    target_person_id INTEGER REFERENCES person (id) ON DELETE SET NULL,
    target_community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    target_remote_url TEXT,
    reason_code TEXT NOT NULL,
    severity case_severity NOT NULL DEFAULT 'Medium',
    status case_status NOT NULL DEFAULT 'Open',
    threshold_score BIGINT NOT NULL DEFAULT 0,
    opened_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at TIMESTAMPTZ,
    closed_at TIMESTAMPTZ
);

CREATE INDEX idx_moderation_case_status_created ON moderation_case (status, opened_at);

CREATE TABLE case_evidence (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    uploader_id INTEGER NOT NULL REFERENCES person (id) ON DELETE RESTRICT,
    storage_key TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    mime_type TEXT NOT NULL,
    visibility evidence_visibility NOT NULL DEFAULT 'JuryOnly',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE sanction (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    scope sanction_scope NOT NULL,
    action sanction_action NOT NULL,
    target_person_id INTEGER REFERENCES person (id) ON DELETE SET NULL,
    target_post_id INTEGER REFERENCES post (id) ON DELETE SET NULL,
    target_comment_id INTEGER REFERENCES comment (id) ON DELETE SET NULL,
    target_community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    starts_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ends_at TIMESTAMPTZ,
    active BOOLEAN NOT NULL DEFAULT TRUE
);

CREATE TABLE appeal (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    requester_id INTEGER NOT NULL REFERENCES person (id) ON DELETE RESTRICT,
    reason TEXT NOT NULL,
    status appeal_status NOT NULL DEFAULT 'Requested',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    decided_at TIMESTAMPTZ
);

CREATE TABLE public_case_log (
    id SERIAL PRIMARY KEY,
    case_id INTEGER NOT NULL REFERENCES moderation_case (id) ON DELETE CASCADE,
    community_id INTEGER REFERENCES community (id) ON DELETE SET NULL,
    summary TEXT NOT NULL,
    rationale_redacted TEXT,
    published_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_public_case_log_community_published ON public_case_log (community_id, published_at);
