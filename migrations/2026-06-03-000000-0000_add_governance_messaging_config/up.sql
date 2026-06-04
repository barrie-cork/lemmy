-- M1 Task 1: governance_messaging_config table + typed-value CHECK + _current view +
-- 2 seed rows (messaging_enabled=false, identity_policy='pseudonymous').
--
-- Design intent:
--   - governance_messaging_config rows are typed (int|bool|text) via a CHECK
--     discriminator. Each row has a valid_from timestamp; admin edits insert
--     new rows and the governance_messaging_config_current view reads the latest.
--   - No value_float column (M1 config keys are int/bool/text only; if a float
--     key ever appears, add the column + CHECK arm in a fresh migration — never
--     mutate this CHECK in-place, per governance_config precedent).
--   - Seed rows pin valid_from to a stable literal so reruns are idempotent
--     under ON CONFLICT (scope, key, valid_from) DO NOTHING.

CREATE TABLE governance_messaging_config (
    id          SERIAL PRIMARY KEY,
    scope       TEXT NOT NULL,         -- 'instance' | 'community:<id>' | room-type
    key         TEXT NOT NULL,         -- 'messaging_enabled' | 'identity_policy' | 'hard_delete_after_days'
    value_type  TEXT NOT NULL,         -- 'int' | 'bool' | 'text'
    value_int   BIGINT,
    value_bool  BOOLEAN,
    value_text  TEXT,
    valid_from  TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_by  INTEGER REFERENCES person (id) ON DELETE RESTRICT,
    CONSTRAINT governance_messaging_config_typed CHECK (
        (value_type = 'int'  AND value_int  IS NOT NULL AND value_bool IS NULL AND value_text IS NULL) OR
        (value_type = 'bool' AND value_bool IS NOT NULL AND value_int  IS NULL AND value_text IS NULL) OR
        (value_type = 'text' AND value_text IS NOT NULL AND value_int  IS NULL AND value_bool IS NULL)
    )
);

-- Unique on (scope, key, valid_from) supports append-history semantics: an
-- admin edit inserts a new row and keeps the old row for audit reconstruction.
-- Do NOT use (scope, key) as the conflict target — GOTCHA-50a.
CREATE UNIQUE INDEX governance_messaging_config_scope_key_valid_from_idx
    ON governance_messaging_config (scope, key, valid_from);   -- append-history; NOT (scope,key) — GOTCHA-50a

CREATE INDEX governance_messaging_config_scope_key_idx
    ON governance_messaging_config (scope, key);

-- governance_messaging_config_current — most-recent row per (scope, key).
CREATE VIEW governance_messaging_config_current AS
SELECT DISTINCT ON (scope, key)
    id, scope, key, value_type, value_int, value_bool, value_text, valid_from, updated_by
FROM governance_messaging_config
ORDER BY scope, key, valid_from DESC;

-- Seed 2 instance-scoped config rows. ON CONFLICT DO NOTHING on
-- (scope, key, valid_from) makes this idempotent — reruns after manual
-- admin edits preserve the admin edits (the unique index on
-- (scope, key, valid_from) distinguishes the seed's pinned literal from
-- admin edits at valid_from = now()). Every row pins valid_from to the
-- stable literal '2026-06-03T00:00:00Z' so reruns hit the same row under
-- the unique index and ON CONFLICT DO NOTHING is a true no-op.
INSERT INTO governance_messaging_config (scope, key, value_type, value_int, value_bool, value_text, valid_from) VALUES
    ('instance', 'messaging_enabled', 'bool', NULL, false,         NULL,           '2026-06-03T00:00:00Z'::timestamptz),
    ('instance', 'identity_policy',   'text', NULL, NULL,          'pseudonymous', '2026-06-03T00:00:00Z'::timestamptz)
ON CONFLICT (scope, key, valid_from) DO NOTHING;
