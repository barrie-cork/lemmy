---
phase: v1-RT-r1
role: impl-task
task: fix-impl-6
brief_n: 6
authored: 2026-05-12
parent_cr: cr-5
---

# [role:impl-task] RT-r1 fix-impl-6 — remove DEFAULT 1 from sponsor_allowlist migration — see .claude/PRPs/briefs/rt-r1-fix-impl-6.md

## §1 Role + dispatch

`[role:impl-task] RT-r1 fix-impl-6 — remove DEFAULT 1 from sponsor_allowlist migration`

## §2 Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md` §G4 classifier table)

This is a CR fix-in-pr, not an allowlist §G4 entry.

**CR finding cr-5:**
> In `migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/up.sql` around line 17-20: The migration currently sets `DEFAULT 1` for `sponsor_allowlist.added_by_admin_id` which hard-codes `person(id=1)`; instead, add the column without a default and allowing NULL (`ALTER TABLE ... ADD COLUMN added_by_admin_id INTEGER REFERENCES person(id)`), perform an explicit backfill step that sets `added_by_admin_id` to a legitimate existing admin id (e.g., via an UPDATE that selects a specific system/admin user by a stable attribute) or leave rows NULL if provenance is unknown, then `ALTER TABLE sponsor_allowlist ALTER COLUMN added_by_admin_id SET NOT NULL` (or keep nullable if NULLs are allowed) and avoid any DEFAULT 1; reference the column name `added_by_admin_id` and table `sponsor_allowlist` when implementing these steps.

### 2.2 Context

The current migration at `migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/up.sql` lines 17-20:
```sql
ALTER TABLE sponsor_allowlist
    ADD COLUMN added_by_admin_id INTEGER NOT NULL
        REFERENCES person(id) DEFAULT 1;
ALTER TABLE sponsor_allowlist
    ALTER COLUMN added_by_admin_id DROP DEFAULT;
```

The table is new in a prior migration (v0 bootstrap) and at v0 ship there are NO real rows (the seed_founders tool doesn't populate sponsor_allowlist). The backfill case is thus vacuous — no existing rows need a value. The cleanest fix consistent with CR's guidance:

```sql
ALTER TABLE sponsor_allowlist
    ADD COLUMN added_by_admin_id INTEGER REFERENCES person(id);
-- No existing rows → no backfill needed (table was empty at this migration point)
COMMENT ON COLUMN sponsor_allowlist.added_by_admin_id IS
    'Per PRD section 5.4: admin who added the row (audit trail).
     Required non-null for rows written by r4 endpoints. Legacy rows (none at v0) may be NULL.
     r4 endpoints set from caller person_id.';
```

Leave `added_by_admin_id` nullable (remove the `NOT NULL` + `DEFAULT 1` + `DROP DEFAULT` pattern). Future r4 write endpoints enforce non-null at the application layer (caller's person_id is always set). This avoids the `DEFAULT 1` hardcoding entirely without requiring a backfill query.

### 2.3 File edits

**File:** `migrations/2026-05-10-000100-0000_extend_sponsor_allowlist_for_r1/up.sql`

Replace the two ADD COLUMN + DROP DEFAULT statements for `added_by_admin_id` with a single ADD COLUMN (nullable, no DEFAULT):

```sql
ALTER TABLE sponsor_allowlist
    ADD COLUMN added_by_admin_id INTEGER REFERENCES person(id);
```

Also update the COMMENT to reflect nullable (remove "Required non-null. r4 endpoints set from caller person_id." or revise to note nullable at migration time).

**File:** `crates/db_schema/src/source/governance/sponsor_allowlist.rs`

Check if `SponsorAllowlistInsertForm` has `added_by_admin_id: i32` (NOT NULL). If so, change to `added_by_admin_id: Option<i32>` to match the now-nullable column. Also update `SponsorAllowlist` struct if it has `added_by_admin_id: i32`.

**File:** `crates/db_schema_file/src/schema.rs`

Check if `sponsor_allowlist` table definition has `added_by_admin_id -> Integer` (not nullable) — if so, change to `added_by_admin_id -> Nullable<Integer>` to match the migrated column.

Touch only these 3 files + `.claude/decision-queue.json`.

## §3 Required reading

1. `.claude/rules/decision-queue.md` — Recipe 1 if blocker found
2. `.claude/lessons/feedback_lemmy_migration_runner.md` — migration discipline

## §4 Constraints

- **Touch only:** the 3 files above + `.claude/decision-queue.json` (if blocker raised)
- **≤3 file edits** — migration SQL + Rust struct + schema.rs
- **Do NOT add a backfill UPDATE** — there are no existing rows at this migration point; the table was created in an earlier migration with zero rows written before this one
- **Keep nullable** — `added_by_admin_id INTEGER REFERENCES person(id)` without NOT NULL; application layer enforces non-null for new writes
- **Rust Diesel type:** `Nullable<Integer>` in schema.rs → `Option<i32>` in Rust structs
- **Callsite check:** grep `added_by_admin_id` across `crates/` to find any callsite setting `added_by_admin_id: <i32>` that needs wrapping in `Some(...)`. Most likely callers: `create_endorsement.rs`, `sponsor_liability.rs`, `seed_founders/src/main.rs`.
- **Commit message:** `fix(v1-RT-r1): remove DEFAULT 1 from sponsor_allowlist migration — nullable added_by_admin_id (cr-5)`
- **Shape G:** after committing, push worker branch; write `kind: "validate-pending"` DQ entry with `workflow_run_id` from `gh run list --repo barrie-cork/lemmy --branch <branch> --limit 1 --json databaseId`. Commit + push DQ entry.
- **DQ atomic raise:** `git add .claude/decision-queue.json && git commit && git push origin <branch>` THEN end task.
- **Encoding:** Python `json.dump(..., ensure_ascii=False, indent=2)` for DQ writes.
- **next_id:** compute across `.claude/decision-queue.json` AND `.claude/decision-queue-archive-*.json`. Current max is 211 (fix-impl-4+5 will have claimed 210+211) — next id is 212.
- **Attribution:** `from: "impl"`, never `from: "advisor"`.
- **Base branch:** `phase-v1-RT-r1`
