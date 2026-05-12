-- v1-RT-r1 task 2: extend sponsor_allowlist (shipped pre-v1-AD-a per
-- DQ #181) for r4-allowlist-strategy admin readers.
-- ============================================================
-- Authority trail:
--   - PRD: section 5.4 third sponsor-gate strategy
--   - Plan: section 10.2, Task 2
--   - DQ #181 (advisor 2026-05-10): EXTEND, do NOT create
-- ============================================================

ALTER TABLE sponsor_allowlist
    ALTER COLUMN community_id DROP NOT NULL;
COMMENT ON COLUMN sponsor_allowlist.community_id IS
    'Per PRD section 5.4: NULL means instance-wide allowlist;
     NON-NULL scopes to one community.';

ALTER TABLE sponsor_allowlist
    ADD COLUMN added_by_admin_id INTEGER NOT NULL
        REFERENCES person(id);
COMMENT ON COLUMN sponsor_allowlist.added_by_admin_id IS
    'Per PRD section 5.4: admin who added the row (audit trail).
     Required non-null. r4 endpoints set from caller person_id.';

ALTER TABLE sponsor_allowlist ADD COLUMN note TEXT;
COMMENT ON COLUMN sponsor_allowlist.note IS
    'Per PRD section 5.4: admin-supplied free-text rationale. NULL allowed.';
