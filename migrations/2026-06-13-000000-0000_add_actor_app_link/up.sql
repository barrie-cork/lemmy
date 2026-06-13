CREATE TABLE actor_app_link (
    id SERIAL PRIMARY KEY,
    brehon_actor_id INTEGER NOT NULL REFERENCES actor_pseudonym (id) ON DELETE CASCADE,
    app_id TEXT NOT NULL,
    app_local_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    UNIQUE (brehon_actor_id, app_id, app_local_id)
);
