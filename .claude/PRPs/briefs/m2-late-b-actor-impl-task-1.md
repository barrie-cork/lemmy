---
role: impl-task
task_number: 1
phase: m2-late-b-actor
base_branch: phase-m2-late-b-actor
created: 2026-06-13
mandatory_lessons_fired:
  - feedback_lemmy_migration_runner.md  # migrations/** pattern match
  - feedback_validate_pending_laptop_write_then_stop.md  # pre-Shape-G validate-pending-laptop constraint
---

# impl-task brief — m2-late-b-actor Task 1: CREATE migration `add_actor_app_link`

**Role:** `[role:impl-task]`
**Phase:** `m2-late-b-actor`
**Task number:** 1 of 13 (non-`[P]` — must complete before Task 2 cohort)
**Base branch:** `phase-m2-late-b-actor`
**Authored:** 2026-06-13

---

## 1. Role + dispatch line

```
[role:impl-task] m2-late-b-actor task-1 migration add_actor_app_link — see .claude/PRPs/briefs/m2-late-b-actor-impl-task-1.md
```

You are the **impl-task** subagent (Sonnet 4.6 — pattern-following from MIRROR refs). Execute exactly what is described here. No impl beyond what is listed.

---

## 2. Scope

**Produce:**
- `migrations/2026-06-13-000000-0000_add_actor_app_link/up.sql` — creates `actor_app_link` table
- `migrations/2026-06-13-000000-0000_add_actor_app_link/down.sql` — drops `actor_app_link` table
- `validate-pending-laptop` DQ entry (commit + push on the phase branch)

**Do NOT:**
- Touch `schema.rs` (that is Task 2)
- Write any Rust model files (Tasks 3–9)
- Write any bridge files (Tasks 11–12)
- Run `cargo check` yourself — that is delegated to the laptop advisor via `validate-pending-laptop`

---

## 3. Required reading

1. `.claude/PRPs/plans/m2-late-b-actor.plan.md` §"Task 1" — the canonical task description and GOTCHAs
2. `.claude/lessons/feedback_lemmy_migration_runner.md` — **MANDATORY** (migrations/** pattern match): how to run migrations via `cargo run -p lemmy_diesel_utils --features full` (no sub-command args); do NOT attempt to run this yourself
3. `.claude/lessons/feedback_validate_pending_laptop_write_then_stop.md` — **MANDATORY** (pre-Shape-G): write `validate-pending-laptop` DQ then STOP
4. `migrations/2026-04-15-100300-0000_add_reputation_and_surety/up.sql` lines 1-19 — **MIRROR**: composite `UNIQUE` + `revoked_at TIMESTAMPTZ` shape; copy this convention
5. `migrations/2026-06-07-000000-0000_add_sanction_event/down.sql` — **MIRROR**: most-recent governance migration; naming convention + reverse-order drop
6. `.claude/rules/decision-queue.md` — DQ write discipline (mid-task commit + push required)

---

## 4. Constraints

### Implementation — IMPLEMENT exactly this, no more

**`migrations/2026-06-13-000000-0000_add_actor_app_link/up.sql`:**
```sql
CREATE TABLE actor_app_link (
    id SERIAL PRIMARY KEY,
    brehon_actor_id INTEGER NOT NULL REFERENCES actor_pseudonym (id) ON DELETE CASCADE,
    app_id TEXT NOT NULL,
    app_local_id TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    UNIQUE (brehon_actor_id, app_id, app_local_id)
);
```

**`migrations/2026-06-13-000000-0000_add_actor_app_link/down.sql`:**
```sql
DROP TABLE actor_app_link;
```

**GOTCHA — dir name format:**
- Exact directory: `migrations/2026-06-13-000000-0000_add_actor_app_link/`
- Format: `YYYY-MM-DD-HHMMSS-0000_snake_case` — the `0000` suffix is **literal**, always 4 zeros
- No `metadata.toml`, no `README` — just `up.sql` and `down.sql`
- Must sort **AFTER** `2026-06-07-000000-0000_add_sanction_event/` — verify with `ls migrations/ | tail -5`

**GOTCHA — no standalone CREATE INDEX:**
- The `UNIQUE (brehon_actor_id, app_id, app_local_id)` constraint creates an implicit unique index
- The FK on `brehon_actor_id` and PK on `id` create their own indexes
- Do NOT add any `CREATE INDEX` statements — governance migration convention

**GOTCHA — FK target:**
- References `actor_pseudonym (id)` with `ON DELETE CASCADE` — exact spelling, no deviation

### validate-pending-laptop (pre-Shape-G, MANDATORY)

After committing the migration files:

1. Write a `kind: "validate-pending-laptop"` DQ entry with:
   ```json
   {
     "commands": [
       "cargo check --workspace --features full"
     ],
     "branch": "phase-m2-late-b-actor",
     "phase_task": 1
   }
   ```
   Use `bash scripts/brehon/dq-v3-new-entry.sh` for the id.
   Use `bash scripts/brehon/dq-v3-append-fragment.sh <fragment.json> --pending` to append it.

2. **Commit** the DQ entry: `git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised validate-pending-laptop task-1 migration"`

3. **Push** to origin: `git push origin phase-m2-late-b-actor`

4. **STOP. Do NOT run `cargo check` yourself.** The laptop advisor runs validation locally. Workers running cargo on the daemon cause file-lock contention (NO-CARGO-ON-ELITEDESK rule).

### ADR constraints

- **ADR-015** (pseudonymity): this migration does not touch `actor_pseudonym` content — `brehon_actor_id` is an FK integer reference, not a raw person identity. No constraint violation.
- **ADR-008** (append-only log): migration itself does not write to governance_log — that is Task 6/Task 9's responsibility.
- **ADR-004** (plane separation): this table lives in the Brehon-side Postgres DB — correct. No bridge dependency.

### DQ blocker discipline

If you hit a genuine blocker (FK target doesn't exist, migration dir naming ambiguity), write a `kind: "blocker"` DQ entry via `bash scripts/brehon/dq-v3-new-entry.sh`, commit + push, then STOP.

---

## 5. Commit

Single commit:
```
feat(db_schema): add actor_app_link migration (B-actor ADR-016 task 1)
```

Stage only:
- `migrations/2026-06-13-000000-0000_add_actor_app_link/up.sql`
- `migrations/2026-06-13-000000-0000_add_actor_app_link/down.sql`

Then the DQ entry commit (separate, as specified above).

---

## 6. DoD

- [ ] `migrations/2026-06-13-000000-0000_add_actor_app_link/up.sql` exists with the exact `actor_app_link` table DDL
- [ ] `migrations/2026-06-13-000000-0000_add_actor_app_link/down.sql` exists with `DROP TABLE actor_app_link;`
- [ ] Dir sorts after `2026-06-07-000000-0000_add_sanction_event/`
- [ ] `validate-pending-laptop` DQ entry committed + pushed (laptop advisor runs `cargo check --workspace --features full`)
- [ ] No cargo run on the daemon worker

---

## HANDOVER

```yaml
HANDOVER:
  task: m2-late-b-actor-impl-task-1
  branch: phase-m2-late-b-actor
  filesCreated:
    - migrations/2026-06-13-000000-0000_add_actor_app_link/up.sql
    - migrations/2026-06-13-000000-0000_add_actor_app_link/down.sql
  filesModified:
    - .claude/decision-queue.json
  keyDecisions:
    - "actor_app_link table: SERIAL PK, FK brehon_actor_id→actor_pseudonym(id) ON DELETE CASCADE, composite UNIQUE (brehon_actor_id, app_id, app_local_id), revoked_at TIMESTAMPTZ nullable"
    - "validate-pending-laptop DQ: commands=[cargo check --workspace --features full]"
    - "pre-Shape-G: DO NOT run cargo on daemon worker"
  notes: "Task 1 of 13. Task 2 (schema regen) is blocked on this validate-pending-laptop passing."
```
