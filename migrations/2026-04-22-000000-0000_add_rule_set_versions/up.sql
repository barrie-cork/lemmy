-- v1-AD-a rule_set_version — append-only versioned rule-set table.
--
-- `created_by` uses ON DELETE RESTRICT by deliberate design, copying the
-- v0 precedent at `2026-04-18-000000-0000_add_governance_config/up.sql:29`
-- (`governance_config.updated_by`). Both columns record the admin who
-- seeded an append-only governance artifact (config row / rule-set
-- version) and both are audit-immutable in the same sense. ADR-015's
-- pseudonymisation obligation targets `governance_log` (via
-- `actor_pseudonym TEXT`), not governance-config / rule-set authoring
-- metadata. GDPR erasure for `created_by` is handled by admin-level
-- NULL overwrite (column is already nullable), not by cascading
-- DELETE FROM person.
--
-- See also: two existing RESTRICT precedents on person FKs in
-- `2026-04-15-100100-0000_add_governance_core/up.sql` —
-- `case_evidence.uploader_id` (line 25) and `appeal.requester_id`
-- (line 50) — both audit-immutable by design.
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
