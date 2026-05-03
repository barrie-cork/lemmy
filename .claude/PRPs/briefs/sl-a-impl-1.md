---
role: impl-task
plan_task: 1
phase: v1-SL-a
cohort: A
created: 2026-05-03
related_dq: []
---

# Brief — v1-SL-a Task 1 [P] — CREATE migration `2026-05-03-000000-0000_add_sponsor_liability_grace_window/{up,down}.sql`

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a task 1 — see .claude/PRPs/briefs/sl-a-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6) executing plan Task 1
of cohort A on `phase-v1-SL-a`. Cohort A = Tasks 1+2+3+6+7 (5-way
parallel per plan §13 line 1240); you own only
`migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/`.

## 2. Scope

**Produce:**

1. ONE commit creating two new files:
   - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`
   - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql`
2. **Two** `kind: "validate-pending"` DQ entries on your worker
   branch — one per workflow run id (workspace-check + migration-check).
3. Push your worker branch. Junior daemon finalize-merges into
   `phase-v1-SL-a`.

**Do NOT** in this task:

- Touch any other file (no `crates/**`, no `enums.rs`, no `schema.rs`,
  no `tests/**`, no scripts, no docs). The migration directory is the
  only path you create.
- Run cargo, diesel, or `lemmy_diesel_utils` locally. Shape G — both
  workflows fire on your push to `junior/<task-slug>`.
- Bump `PHASE_1_MIGRATION_COUNT` (that's Task 8; touching it here
  causes silent LIFO slot reuse — see lesson §3 Required reading 4).
- Apply the migration to a local Postgres. Validation is workflow-
  side; do not improvise.
- Modify the seed-key list, hours values, or backfill SQL shape.
  Plan §10.1 is the canonical source — copy verbatim.

## 3. Required reading

Read in this order before writing any SQL:

1. **Plan Task 1 body:** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md`
   lines 1415–1503 — task header, FILES YAML, IMPLEMENT directives,
   GOTCHAs (5 of them, all load-bearing).
2. **Plan §10.1 (canonical SQL skeleton):**
   `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` lines 539–711.
   This is the verbatim source for both files.
3. **Plan §8 (flow design)** for ordering rationale + PRD §8.5 cite:
   lines 367–433.
4. **`.claude/lessons/feedback_lemmy_migration_runner.md`** — explains
   the `-- no-transaction` directive (mandatory because `ALTER TYPE
   case_status ADD VALUE` cannot run inside a Postgres transaction)
   and the `pg_advisory_lock(0)` mechanism upstream relies on. **Do
   NOT remove or reorder the `-- no-transaction` directive on line 1
   after the header comment.**
5. **`.claude/lessons/feedback_phase1_migration_count_lifo.md`** —
   explains why this task does NOT touch
   `PHASE_1_MIGRATION_COUNT` (that's a separate Task 8 concern).
   The constant is LIFO-positional, not a semantic set.
6. **MIRROR refs cited by plan Task 1:**
   - `migrations/2026-04-22-000300-0000_seed_v1_config_keys/up.sql`
     — idempotent `ON CONFLICT` seed pattern (read up.sql in full).
   - `migrations/2026-04-23-000100-0000_add_jury_mechanics_columns/up.sql`
     — ADR-exception-trail header comment + ADD COLUMN + backfill
     UPDATE pattern.
7. **`.claude/decision-queue.json`** — confirm `pending == 0` at
   task start. SL-a planner DQ #116 is resolved.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 0
    commit: 1f6131bf823f006bd2ace06b2931e2c1b56cd8ad
    filesCreated: []
    filesModified:
      - scripts/brehon/migrate-roundtrip.sh
    keyDecisions:
      - replaced stub per DQ #114 (Option A: Task 0 of SL-a)
      - "lemmy_diesel_utils binary surface — no CLI sub-commands; uses
        LEMMY_DATABASE_URL env var only (binary at
        crates/diesel_utils/src/main.rs reads from env, takes no positional
        args). Adapted plan §13 GOTCHA cargo invocations accordingly."
    notes: |
      Probe 2 CaseStatus script count=14 (script precision issue: sed range
      includes CaseStatusTier); actual CaseStatus has 9 variants confirmed
      by file read. Probe 9 yamllint exit=127 (not installed); YAML
      validated via python yaml — all 3 files OK; flags --features full,
      --no-deps, -D warnings confirmed present.
```

The Task 0 keyDecisions are advisory for cohort A: the
`LEMMY_DATABASE_URL`/no-sub-command finding is **not** load-bearing
for Task 1 (no local cargo here), but it confirms the migration-
runner mechanism if you need to mentally validate the workflow that
will exercise this migration after push.

## 4. Constraints

### Branch + environment

- You start on a Junior worktree branch derived from `phase-v1-SL-a`
  (tip `1b47a3ff4` per `git log -1 --oneline phase-v1-SL-a`).
- `git branch --show-current` must return your `junior/<task-slug>`
  branch. Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD`
  exits 0 before any write.
- ONE source-code commit. Daemon finalize absorbs into `phase-v1-SL-a`.

### File discipline

- Files you may CREATE:
  - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`
  - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql`
- **Files you may NOT touch:** `crates/**`, `tests/**`, `scripts/**`,
  `docs/**`, `.github/**`, `Cargo.toml`, `Cargo.lock`,
  `rust-toolchain.toml`, ANY other `migrations/*` directory, any
  other `migrations/2026-05-03-*` directory variant. If you find
  yourself wanting to edit any of these, STOP and file a DQ.

### SQL discipline

- **Verbatim copy from plan §10.1.** Do not "tidy", reformat
  whitespace, alphabetise INSERTs differently, or substitute SQL
  syntax. Plan §10.1 already alphabetises seed rows and uses the
  documented hours values.
- **Preserve `-- no-transaction` directive** on line 1 after the
  header comment in `up.sql`. It is consumed by Diesel's runner and
  removing it breaks `ALTER TYPE case_status ADD VALUE` (Postgres
  refuses inside a txn).
- **Hours values are RAW INTEGERS, not micros-scaled** (per DQ #115
  resolved by advisor): the 6 `liability.grace_window_*_hours` keys
  carry `24/72/168/1/720/24` as integer hours. Bool keys
  (`liability.restoration_escapes_liability`) and enum keys
  (`liability.multi_sponsor_escape_rule`) are non-micros. The 6
  hour values feed the backfill UPDATE as `INTERVAL '<N> hours'`.
- **PRESERVE the three nested EXISTS guards in the backfill UPDATE**
  per plan Task 1 GOTCHA "PRD §8.4 backfill performance". Do not
  collapse into a JOIN at SL-a time. SL-c will profile + optimise
  later if needed.
- **`liability_escape_reason` JSONB column COMMENT** documents
  `{"version": 1, ...}` from day one (per plan §10.1 + Task 1
  GOTCHA OQ-V1-SL-05). SL-a does not write to this column; the
  COMMENT is the forward-compat schema reference.
- **`surety_sponsored_id_active` index inclusion** is intentional
  (Issue #24 fold-in per plan Task 1 GOTCHA). Index target:
  `(sponsored_id, sponsor_id) WHERE revoked_at IS NULL`. Do NOT
  drop or change the index expression.
- **down.sql is LIFO** per plan §10.1 down.sql skeleton. Order:
  reverse the backfill UPDATE → DELETE the 13 seeds → DROP partial
  indexes → DROP COLUMN → orphan-enum-value doc-comment (Postgres
  cannot DROP enum VALUEs added by a prior up.sql).

### Migration directory naming (re-verify at task start)

The plan expects `2026-05-03-000000-0000_add_sponsor_liability_grace_window`.
At task start, run `ls migrations/ | tail -3` and confirm no migration
has landed *between* brief-write and impl that would tie-break the
sort. If the most-recent migration date-prefix ≥ `2026-05-03-000000`,
pick `2026-05-03-000100-0000_add_sponsor_liability_grace_window`
instead and document in commit body. **Do not silently rename the
directory** without a commit-body note.

### Shape G discipline (no local cargo)

- Push your worker branch to `junior/<task-slug>`. Both
  `cargo-validate-workspace.yml` and `cargo-validate-migration.yml`
  fire because the push touches `migrations/**`.
- Capture **both** workflow run ids:
  ```bash
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  gh run list --repo barrie-cork/lemmy --branch <branch> --workflow cargo-validate-migration --limit 1 --json databaseId
  ```
- Write **two** `kind: "validate-pending"` DQ entries (one per
  workflow run id) per plan Task 1 "Push and exit" + `.claude/rules/
  decision-queue.md` Recipe 1. Each entry: distinct `id`,
  `from: "impl"`, `kind: "validate-pending"`, `workflow_run_id: <id>`,
  `branch: "<your branch>"`, `phase_task: 1`, `result: null`,
  `log_slice: null`, `failed_jobs: null`, `answer: null`,
  `answered_by: null`, `resolved_at: null`. Distinguish in
  `context` which workflow ("workspace-check" vs "migration-check").
- Commit + push the DQ entries to your worker branch immediately
  per `.claude/rules/decision-queue.md` "Mid-task visibility" — DO
  NOT batch with the migration commit. Two commits acceptable here:
  (1) `feat(v1-SL-a): combined migration ...`, (2)
  `chore(decision-queue): impl raised DQ #N + #N+1 — sl-a-task-1
  validate-pending`.

### DQ attribution

- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- Any new DQ entry from this task uses `from: "impl"`.
- Compute next id by scanning **both** the live file's `pending` +
  `resolved` arrays AND any `decision-queue-archive-*.json`
  siblings, then take `max + 1` (per `.claude/rules/decision-
  queue.md` "Next-id calculation"). DQ #50 collision was the lesson.

### Commit shape

- ONE source-code commit:
  - Subject: `feat(v1-SL-a): combined migration — case_status enum + grace columns + indexes + 13 config seeds + v0 mid-flight backfill (task 1)` (verbatim per plan §13 line 1503).
  - Body: include the `HANDOVER:` YAML trailer per
    `.claude/agents/impl-task.md` "Per-task commit shape":
    ```yaml
    HANDOVER:
    filesCreated:
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql
    filesModified: []
    keyDecisions:
      - <e.g. "preserved -- no-transaction directive per upstream forbid-trigger lesson">
      - <e.g. "verified directory sort: 2026-05-03-000000 is post-2026-04-27 (latest on phase tip)">
    notes: <any OBSERVED deviation from plan §10.1 verbatim, with reason>
    ```
- THEN the DQ-entries commit (separate, per "Shape G" above).
- Files in source-code commit: ONLY the two migration files. Files
  in DQ commit: ONLY `.claude/decision-queue.json`.

## 5. Validation gate

The plan §15 (DoD) names these commands. **You do not run them
locally.** They run in GH Actions on your push:

- `cargo-validate-workspace.yml` → `cargo check --workspace --features
  full` + `cargo clippy --workspace --features full --no-deps -- -D
  warnings`.
- `cargo-validate-migration.yml` → `lemmy_diesel_utils run` →
  `revert` → `run` round-trip on a fresh Postgres container.

If either fails, the ci-watcher mutates your `validate-pending`
entries with `result: "fail"` + `log_slice`. Advisor §G4 classifier
takes over. **Do not attempt local fix-up unless explicitly asked
in a follow-up task brief.**

## 6. Expected output (return to advisor)

```
## Task 1 complete — v1-SL-a combined migration shipped

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> feat(v1-SL-a): combined migration ... (task 1)
  - <sha-b> chore(decision-queue): impl raised DQ #<N> + #<N+1> — sl-a-task-1 validate-pending
**Files:** migrations/2026-05-03-<X>-000000-0000_add_sponsor_liability_grace_window/{up,down}.sql
**Migration directory chosen:** 2026-05-03-000000-0000 (or 2026-05-03-000100-0000 if drift detected — explain)
**`-- no-transaction` directive:** preserved on line 1 of up.sql
**13 seed rows:** alphabetised within INSERT, raw-hours per DQ #115
**Workflow runs captured:**
  - workspace-check workflow_run_id: <int> → DQ #<N>
  - migration-check workflow_run_id: <int> → DQ #<N+1>
**Next:** advisor dispatches 2 ci-watchers; cohort A (Tasks 1+2+3+6+7) advances per cohort-barrier semantics (plan §14 Story 1 = workspace-check success on Task 5's push, post cohort B)
```

If any pre-write check failed (DQ #116 unresolved, MIRROR file
missing, plan §10.1 unreachable), replace the above with a DQ
catch-fire entry committed + pushed to your worker branch
immediately.
