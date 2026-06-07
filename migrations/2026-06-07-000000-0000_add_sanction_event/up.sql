CREATE TYPE sanction_kind AS ENUM ('prevent_post', 'mute_voice', 'hide_content', 'restrict_reach');

CREATE TABLE sanction_event (
  id SERIAL PRIMARY KEY,
  sanction_id INTEGER NOT NULL REFERENCES sanction(id) ON DELETE CASCADE,
  sanction_kind sanction_kind NOT NULL,
  subject_actor_pseudonym TEXT NOT NULL,          -- actor_pseudonym.pseudonym ONLY (ADR-015)
  effective_from TIMESTAMPTZ NOT NULL,
  effective_until TIMESTAMPTZ NULL,
  governance_log_entry_hash TEXT NOT NULL          -- hex-encoded governance_log.entry_hash
);

CREATE TABLE sanction_subscriber (
  id SERIAL PRIMARY KEY,
  callback_url TEXT NOT NULL UNIQUE,
  active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
