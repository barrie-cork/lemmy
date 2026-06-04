---
phase: m1-b
role: impl-task
n: 1
authored: 2026-06-04
authored_by: advisor (canonical brehon-fork / governance-v0 session)
base_branch: phase-m1-b
task_number: 1
---

# [role:impl-task] m1-b Task 1 — `governance_messaging_config` migration

## 1. Role + dispatch line

```
[role:impl-task] m1-b task-1 governance_messaging_config migration — see .claude/PRPs/briefs/m1-b-impl-1.md
```

## 2. Scope

Create the up/down SQL for the typed-column messaging-config table. This is a SQL-only task — no Rust. **Write the migration, write a `validate-pending-laptop` DQ entry, commit + push, then STOP.** Do NOT run any cargo/docker validation yourself (M1 is pre-Shape-G; cargo + Docker run on the laptop, never on the daemon).

**Produce (exactly two files):**

```yaml
creates:
  - migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql
  - migrations/2026-06-03-000000-0000_add_governance_messaging_config/down.sql
modifies: []
```

- **`up.sql`** — VERBATIM from plan §10.1 (reproduced in §4 below). The table + typed CHECK + UNIQUE INDEX `(scope, key, valid_from)` + INDEX `(scope, key)` + `_current` DISTINCT-ON view + seed rows (`messaging_enabled=false`, `identity_policy='pseudonymous'`) with **literal `valid_from`** + `ON CONFLICT DO NOTHING`.
- **`down.sql`** — `DROP VIEW governance_messaging_config_current; DROP TABLE governance_messaging_config;` (view first, then table).
- One commit on the task worktree branch.
- A `validate-pending-laptop` DQ entry (shape in §4).

**Do NOT:**
- Touch any file outside the new migration directory.
- Use `(scope, key)` as the UNIQUE INDEX target — it MUST be `(scope, key, valid_from)` (append-history; GOTCHA-50a).
- Use `now()` in the seed rows' `valid_from` — use a **literal timestamp** so re-runs are idempotent under `ON CONFLICT`.
- Add a `value_float` column (M1 is int/bool/text only).
- Run `diesel migration run` (forbid-trigger — see lesson) OR `migrate-roundtrip.sh` OR any `cargo` / `docker` command. Validation is the laptop's job.
- Commit to `governance-v0` or any branch other than the task worktree branch.
- Write `approved_by: "advisor"` in any DQ entry.

## 3. Required reading (in order)

Read all of these before writing the SQL:

### MIRROR ref (read and mirror its shape exactly)
- `migrations/2026-04-18-000000-0000_add_governance_config/up.sql` — the canonical typed-column config table. The new table is the SAME shape minus `value_float` (and minus its float CHECK arm), with the table/index/view names swapped to `governance_messaging_config*` and different seed rows. Mirror the comment style, the `valid_from` index rationale comment, and the `_current` view structure.

### Plan sections
- `.claude/PRPs/plans/m1.plan.md` §10.1 (the verbatim up.sql — reproduced in §4), §13 Task 1 (ACTION/GOTCHA/VALIDATE)

### Lessons (mandatory — fired by file-class table for `migrations/**`)
- `.claude/lessons/feedback_lemmy_migration_runner.md` — **why the worker must NOT run the migration**: raw `diesel migration run/redo/revert` is blocked by the upstream `forbid_diesel_cli` trigger; only `cargo run -p lemmy_diesel_utils --features full` (which `migrate-roundtrip.sh` wraps) works, and that runs on the laptop. Read so you understand why validation is delegated.
- `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — write the DQ entry and push, then stop; do NOT run validation yourself.

> JSONB lesson (`feedback_postgres_jsonb_canonicalization.md`) does NOT apply — this table uses typed columns (int/bool/text), no JSONB.

## 4. Constraints + verbatim SQL

### 4.1 `up.sql` — VERBATIM (plan §10.1)

```sql
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
CREATE UNIQUE INDEX governance_messaging_config_scope_key_valid_from_idx
    ON governance_messaging_config (scope, key, valid_from);   -- append-history; NOT (scope,key) — GOTCHA-50a
CREATE INDEX governance_messaging_config_scope_key_idx
    ON governance_messaging_config (scope, key);
CREATE VIEW governance_messaging_config_current AS
SELECT DISTINCT ON (scope, key)
    id, scope, key, value_type, value_int, value_bool, value_text, valid_from, updated_by
FROM governance_messaging_config
ORDER BY scope, key, valid_from DESC;
```

**Seed rows** (append after the view; use a **literal** `valid_from`, e.g. `'2026-06-03 00:00:00+00'`, and `ON CONFLICT DO NOTHING` against the `(scope, key, valid_from)` unique index):
- `messaging_enabled` → `value_type='bool'`, `value_bool=false`, `scope='instance'`
- `identity_policy` → `value_type='text'`, `value_text='pseudonymous'`, `scope='instance'`

Mirror the seed-INSERT idiom from the MIRROR `governance_config/up.sql` (it seeds 34 rows with `ON CONFLICT DO NOTHING` — copy the column-list + VALUES + `ON CONFLICT` shape, adjusting columns to this table).

### 4.2 `down.sql`

```sql
DROP VIEW governance_messaging_config_current;
DROP TABLE governance_messaging_config;
```

### 4.3 validate-pending-laptop DQ entry

After writing both SQL files and committing them, append a `validate-pending-laptop` DQ entry to `.claude/decision-queue.json`. Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id (or `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending`). Required fields:

```json
{
  "from": "impl",
  "kind": "validate-pending-laptop",
  "branch": "<your worktree branch>",
  "phase_task": 1,
  "commands": ["./scripts/brehon/migrate-roundtrip.sh"],
  "question": "Migration round-trip for governance_messaging_config — laptop runs (Docker + cargo).",
  "options": ["pass", "fail"],
  "context": "Task 1 created migrations/2026-06-03-000000-0000_add_governance_messaging_config/{up,down}.sql. Round-trip via migrate-roundtrip.sh (auto-detects new migration vs governance-v0; spins ephemeral pgautoupgrade:18 containers; applies via cargo run -p lemmy_diesel_utils --features full). Docker + cargo required — laptop only.",
  "result": null,
  "log_slice": null,
  "failed_commands": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

> `migrate-roundtrip.sh` takes **no positional arg** — it auto-detects the new migration via `git diff --diff-filter=A origin/governance-v0...HEAD`. Do not pass the migration id.

### 4.4 Commit + push discipline

- **Commit subject:** `feat(migration): add governance_messaging_config typed-column table (task 1)`
- After committing the two SQL files AND writing the DQ entry:
  ```
  git add migrations/ .claude/decision-queue.json
  git commit -m "feat(migration): add governance_messaging_config typed-column table (task 1)"
  git push origin <your worktree branch>
  ```
  (Or two commits — migration first, then `chore(decision-queue): impl raised validate-pending-laptop DQ — m1-b task 1` — either is fine; both files must be pushed.)
- Then **STOP**. Do not run validation.

### 4.5 Attribution
- `from: "impl"` on the DQ entry; `answered_by: null` (the laptop advisor mutates it).
- NEVER `answered_by: "advisor"` from this session.

### Mandatory lessons fired for this brief
- `migrations/**` (new migration): `feedback_lemmy_migration_runner.md` ✓ (why no daemon-side migration run)
- `validate-pending-laptop` constraint: `feedback_validate_pending_laptop_write_then_stop.md` ✓
- JSONB (`feedback_postgres_jsonb_canonicalization.md`): NOT fired — typed columns, no JSONB.

## HANDOVER

```yaml
HANDOVER:
  task: m1-b-task-1
  filesCreated:
    - migrations/2026-06-03-000000-0000_add_governance_messaging_config/up.sql
    - migrations/2026-06-03-000000-0000_add_governance_messaging_config/down.sql
  filesModified: [.claude/decision-queue.json]
  keyDecisions:
    - "typed-column config table (int/bool/text); NO value_float; NO JSONB"
    - "UNIQUE INDEX (scope,key,valid_from) — append-history (GOTCHA-50a)"
    - "seed messaging_enabled=false + identity_policy='pseudonymous' with literal valid_from + ON CONFLICT DO NOTHING"
  notes: "validation = migrate-roundtrip.sh on laptop (Docker+cargo); worker wrote validate-pending-laptop DQ + stopped"
```
