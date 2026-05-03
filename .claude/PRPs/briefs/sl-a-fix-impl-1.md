---
role: impl-task
plan_task: 1-fix
phase: v1-SL-a
created: 2026-05-03
related_dq: [122]
preallocated_dq: [125, 126]
---

# Brief — v1-SL-a fix-impl-1 — Split combined migration into enum-add + non-enum to fix Postgres unsafe-use-of-new-enum-value

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a fix-impl-1 — see .claude/PRPs/briefs/sl-a-fix-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a narrow §G4-classifier-derived fix-impl-task: split Task 1's combined migration directory into TWO sequential migration directories so Postgres allows the new enum values to be used by the backfill `UPDATE`. Net file change: **delete 1 directory (2 files), create 2 directories (4 files), no source code touched.**

**Shape G:** do NOT run cargo, diesel, or `lemmy_diesel_utils` locally. After your commit, push to your worker branch and write **two** `kind: "validate-pending"` DQ entries (workspace-check + migration-check) using the **pre-allocated ids 125 and 126** (per the new cohort dispatch DQ-id pre-allocation pattern, DQ #124 forward guard).

## 2. Scope

**Produce** (one source-code commit + one DQ commit):

1. **Source-code commit:**
   - DELETE `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`
   - DELETE `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql`
     (after delete, the directory is empty and `git rm` removes it.)
   - CREATE `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql`
   - CREATE `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql`
   - CREATE `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/up.sql`
   - CREATE `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/down.sql`
2. **DQ commit:** two `kind: "validate-pending"` entries with **id=125 (workspace-check)** and **id=126 (migration-check)** referencing the new workflow run ids your push triggers.

**Do NOT**:
- Touch ANY `crates/**`, `tests/**`, `scripts/**`, `docs/**`, `.github/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, ANY other `migrations/*` directory, or PRD/plan files. The migration-directory tree is the only path you change.
- Run cargo, diesel, or `lemmy_diesel_utils` locally. Shape G runs both workflows on push.
- Bump `PHASE_1_MIGRATION_COUNT`. That constant is owned by Task 8 in plan §13. The split takes the SL-a addition from `+1` to `+2` migrations; Task 8 will see and reflect it. Touching it here causes silent LIFO slot reuse — see lesson `feedback_phase1_migration_count_lifo.md`.
- Modify the seed-key list, hours values, backfill SQL shape, or COMMENT bodies. Plan §10.1 + the existing Task 1 commit are the canonical source — copy the SQL verbatim into the second directory; just split out the `ALTER TYPE` lines into the first directory.
- Self-pick the directory names — use the EXACT names listed above. The `2026-05-03-000000-0000` prefix is intentionally re-used for the enum migration so the cohort A audit trail (commit `f47bd3...` etc) still points at a `2026-05-03-000000-0000_*` directory.
- Compute your own DQ ids. Use `id=125` for workspace-check and `id=126` for migration-check, in that order.

**Commit messages** (exactly):

- Source-code commit: `fix(v1-SL-a): split combined migration — enum-add directory + grace-window directory (task 1 fix; DQ #122)`
- DQ commit: `chore(decision-queue): impl raised DQ #125 + #126 — sl-a-fix-impl-1 validate-pending`

## 3. Required reading

Read in this order before editing:

1. **DQ #122 on `phase-v1-SL-a` tip** — `git show origin/phase-v1-SL-a:.claude/decision-queue.json` and locate `id: 122`. The `log_slice` contains the failing line:
   ```
   Error: Failed to run 2026-05-03-000000-0000_add_sponsor_liability_grace_window with:
     unsafe use of new value "SponsorLiabilityPending" of enum type case_status
   ```
   This is the **smoking gun** for the split: Postgres refuses to use a freshly-added enum value within the same migration that adds it.

2. **Existing combined migration on phase tip** — `git show origin/phase-v1-SL-a:migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql` (106 lines). This is the canonical source of the SQL you are splitting. Same for `down.sql`. The split must preserve every line of SQL, every COMMENT, every `-- comment block` (including the ADR-exception-trail header).

3. **Plan §10.1 (canonical SQL skeleton):** `.claude/PRPs/plans/v1-sponsor-liability-a.plan.md` lines 539–711. This is the original IMPLEMENT directive. The split deviates from §10.1's "combined" shape — the deviation reason MUST be in your commit body (see "Authority trail" in §4 below).

4. **PRD §8.5 — "Single migration file"** — `.claude/PRPs/prds/v1-sponsor-liability.prd.md` lines 466–477. The PRD also prescribed the combined shape. Your fix deviates from BOTH the plan and the PRD; the deviation is grounded in the Postgres "unsafe use of new enum value" rule, which neither document accounted for.

5. **Phase 5b Restoration migration as the precedent for split** — `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` (2 lines: `-- no-transaction` + 1 `ALTER TYPE ADD VALUE`). This proves the "enum-only `-- no-transaction` migration in its own directory" pattern is established in the repo. Your enum-only migration mirrors this directly.

6. **`.claude/lessons/feedback_lemmy_migration_runner.md`** — explains `-- no-transaction` mechanics. The split's enum-only migration KEEPS `-- no-transaction`; the second migration (columns + indexes + seeds + backfill) does NOT need it (no `ALTER TYPE`), and including it would be wrong.

7. **`.claude/lessons/feedback_phase1_migration_count_lifo.md`** — confirms why you do NOT touch `PHASE_1_MIGRATION_COUNT`.

8. **`.claude/decision-queue.json` on phase-v1-SL-a tip** — confirm `id=125` and `id=126` are not in use (they shouldn't be; max id on phase tip is 124). If you see them already taken, STOP and file a DQ entry — do NOT improvise.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1
    commit: <find via git log on phase-v1-SL-a>
    filesCreated:
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql
    filesModified: []
    keyDecisions:
      - shipped per plan §10.1 verbatim, including combined ALTER TYPE + UPDATE in same migration directory
    notes: |
      Postgres refused the migration on the GH-Actions runner with
      "unsafe use of new value 'SponsorLiabilityPending' of enum type
      case_status" (DQ #122 log_slice). Per Postgres docs and the
      Phase 5b Restoration variant precedent, ALTER TYPE ADD VALUE
      and any UPDATE using that value must live in separate
      migrations. This fix-impl splits the combined directory.
  - task: 2
    keyDecisions:
      - extended CaseStatus enum with 3 SponsorLiability* variants
    notes: |
      Workspace-check fails on non-exhaustive match in cohort B's
      target files (planner-intentional per plan §14 Story 1 + DQ #117).
      NOT relevant to this fix-impl.
  - task: 3
    keyDecisions:
      - added 2 columns to moderation_case schema.rs block
    notes: |
      Workspace-check fails on Diesel Queryable derive macro mismatch
      (planner-intentional, Task 4 fixes). NOT relevant to this fix-impl.
  - task: 6
    keyDecisions:
      - 13 new config consts + match arms + SEEDED_KEYS
    notes: |
      Workspace-check fails on pre-existing JM-e clippy::map_err_ignore
      carry-forward (planner-intentional, Task 5 / fix-in-pr fixes).
      NOT relevant to this fix-impl.
  - task: 7
    keyDecisions:
      - 5 new ENTRY_KIND consts + governance_log entries
    notes: |
      Workspace-check fails on E0277 + clippy::map_err_ignore
      (planner-intentional). NOT relevant to this fix-impl.
```

The cohort A handover is informational. Your fix is scoped to Task 1's migration only — no awareness of Tasks 2/3/6/7 changes is needed.

## 4. The fix (precise)

### Migration A: `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/`

**up.sql** (8 lines, mirrors `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` shape):

```sql
-- v1-SL-a task 1 (split half 1 of 2): add three SponsorLiability* variants
-- to case_status enum.
--
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-sponsor-liability.prd.md §8.5 (combined-
--     migration shape, superseded by this split).
--   - Plan: .claude/PRPs/plans/v1-sponsor-liability-a.plan.md §10.1
--     (combined skeleton, superseded by this split).
--   - Fix: .claude/PRPs/briefs/sl-a-fix-impl-1.md (DQ #122 — Postgres
--     refuses "unsafe use of new value" within the same migration).
--   - Mirror: migrations/2026-04-19-000000-0000_add_restoration_sanction_variant
--     (Phase 5b precedent for enum-only -- no-transaction migration).

-- no-transaction
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityPending';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityFired';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityEscaped';
```

**down.sql** (5 lines):

```sql
-- Reverse of v1-SL-a task 1 (split half 1 of 2).
-- Postgres does not support DROP VALUE without a full type rebuild;
-- the three new variants stay as orphan values in case_status (per
-- Phase 5b Restoration precedent + PRD §3.4 doc-comment).
-- This file intentionally has no DDL.
```

### Migration B: `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/`

**up.sql** = the existing combined `up.sql` MINUS the three `ALTER TYPE` lines and MINUS the `-- no-transaction` directive. Specifically:

1. Keep the entire ADR-exception-trail header comment block at the top — but UPDATE the wording slightly to reflect the split (see below).
2. REMOVE the `-- no-transaction` line and the parenthetical "Required for ALTER TYPE..." comment line beneath it.
3. REMOVE the three `ALTER TYPE case_status ADD VALUE IF NOT EXISTS '...' ;` lines.
4. KEEP everything else byte-identical: the two `ALTER TABLE moderation_case ADD COLUMN`, both `COMMENT ON COLUMN`, the two `CREATE INDEX`, both `COMMENT ON INDEX`, the entire `INSERT INTO governance_config ... ON CONFLICT DO NOTHING` (13 rows), and the `UPDATE moderation_case SET status = 'SponsorLiabilityPending'` backfill block with all three nested EXISTS guards.

**Header comment** for the new `up.sql` (replaces the "no-transaction" ADR header in the original):

```sql
-- v1-SL-a task 1 (split half 2 of 2): columns + indexes + seeds + backfill.
-- ============================================================
-- ADR exception trail (protected governance tables)
-- ============================================================
-- ADDITIVE only: ADD COLUMN, CREATE INDEX, INSERT ... ON CONFLICT DO
-- NOTHING, UPDATE ... (backfill only, bounded by `decided_at >
-- now() - INTERVAL '24 hours'` per ADR-010 won't-disadvantage). No
-- DROP, no ALTER on existing columns, no UPDATE on already-fired
-- cases. No -- no-transaction needed (no ALTER TYPE in this file).
--
-- Controlling ADR: ADR-010 (staged releases).
-- Authority trail:
--   - PRD: .claude/PRPs/prds/v1-sponsor-liability.prd.md §8.5
--     (combined shape, superseded by this split).
--   - Plan: .claude/PRPs/plans/v1-sponsor-liability-a.plan.md §10.1
--     (combined skeleton, superseded by this split).
--   - Fix: .claude/PRPs/briefs/sl-a-fix-impl-1.md (DQ #122 —
--     Postgres refuses unsafe use of new enum value within the
--     same migration that added it).
--   - DQ #115 (advisor 2026-05-03): grace_window_*_hours stored as
--     raw integer hours, NOT micros-scaled.
-- ============================================================

-- (No -- no-transaction; this migration uses normal Diesel
-- transaction wrapping. The case_status enum values referenced in
-- the backfill UPDATE were added by the prior migration
-- 2026-05-03-000000-0000_add_case_status_sponsor_liability_variants,
-- which committed before this migration begins.)

ALTER TABLE moderation_case ADD COLUMN grace_expires_at TIMESTAMPTZ;
COMMENT ON COLUMN moderation_case.grace_expires_at IS
    'Per OQ-025 v1 sponsor-liability grace window. Set when transitioning
     Decided -> SponsorLiabilityPending; locked thereafter. NULL for cases
     not in the grace lifecycle (NoAction outcomes, no-sponsor target,
     v0 backfill exclusions).';

[... rest of file from current up.sql verbatim, starting at the
ALTER TABLE moderation_case ADD COLUMN liability_escape_reason JSONB;
line and continuing through the closing `);` of the UPDATE block ...]
```

**down.sql** = the existing combined `down.sql` MINUS the orphan-enum-value doc-comment paragraph at the end (since enum-value cleanup is now Migration A's down.sql concern, even though both deliberately leave the enum values orphaned). Specifically:

1. Replace the opening header to point at the split.
2. KEEP the `UPDATE moderation_case SET status = 'Decided'` reverse-backfill (intact with the `grace_expires_at IS NOT NULL` guard).
3. KEEP the `DELETE FROM governance_config WHERE scope = 'instance' AND key IN (...)` block (all 13 keys).
4. KEEP `DROP INDEX IF EXISTS surety_sponsored_id_active;` and `DROP INDEX IF EXISTS moderation_case_grace_expires_idx;`
5. KEEP `ALTER TABLE moderation_case DROP COLUMN IF EXISTS liability_escape_reason;` and `ALTER TABLE moderation_case DROP COLUMN IF EXISTS grace_expires_at;`
6. REMOVE the trailing "Postgres enum values..." paragraph (it now lives in Migration A's down.sql).

**Header for the new down.sql:**

```sql
-- Reverse of v1-SL-a task 1 (split half 2 of 2).
-- Reverses the backfill UPDATE, deletes 13 config seeds, drops both
-- partial indexes, drops the two new columns. Enum-value cleanup is
-- deferred to migration A (2026-05-03-000000-0000_add_case_status_sponsor_liability_variants);
-- see that file's down.sql for the orphan-enum-value documentation.
```

## 4a. Constraints

### Branch + commit discipline
- You start on a Junior worktree branched off `phase-v1-SL-a` (current tip `416da8202` per `git log -1 --oneline phase-v1-SL-a`; advisor manual finalize-merge of DQ #118 mutation landed at this SHA).
- Confirm `git merge-base --is-ancestor phase-v1-SL-a HEAD` exits 0 before any write.
- Two commits in this order: (1) source-code (the migration split), (2) DQ entries.
- One source-code commit. The DELETE + 4 CREATE go in the same commit.

### File discipline
- Files you may DELETE:
  - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql`
  - `migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql`
- Files you may CREATE:
  - `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql`
  - `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql`
  - `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/up.sql`
  - `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/down.sql`
- **Files you may NOT touch:** `crates/**`, `tests/**`, `scripts/**`, `docs/**`, `.github/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, ANY OTHER `migrations/*` directory, the plan, the PRD, ADRs. If you find yourself wanting to edit any of these, STOP and file a DQ.

### SQL discipline
- **Verbatim copy** of the SQL bodies from the existing combined `up.sql` / `down.sql` into the new Migration B files (only with the ALTER TYPE lines + `-- no-transaction` removed and the headers replaced as specified above).
- **Hours values are RAW INTEGERS, not micros-scaled** (per DQ #115). The 13 `INSERT INTO governance_config` rows are byte-identical between old and new Migration B `up.sql`.
- **PRESERVE the three nested EXISTS guards in the backfill UPDATE.** Do not collapse into a JOIN.
- **PRESERVE both COMMENTs on `grace_expires_at` and `liability_escape_reason`.**
- **PRESERVE both partial-index COMMENTs (`moderation_case_grace_expires_idx` and `surety_sponsored_id_active`).**
- **PRESERVE the down.sql LIFO order in Migration B** (UPDATE reverse → DELETE seeds → DROP indexes → DROP COLUMN).

### DQ discipline (pre-allocated ids 125 + 126)

- **Use id=125** for the workspace-check entry. **Use id=126** for the migration-check entry. Do NOT compute next_id; the ids are reserved.
- **Two `kind: "validate-pending"` entries**, one per workflow run.
- After the source-code commit pushes, capture both workflow run ids:
  ```bash
  git push origin HEAD
  # Wait ~10 seconds for GH Actions to trigger, then:
  gh run list --repo barrie-cork/lemmy --branch <your-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId
  gh run list --repo barrie-cork/lemmy --branch <your-branch> --workflow cargo-validate-migration --limit 1 --json databaseId
  ```
- DQ entry shapes (insert into `pending` array):

  ```json
  {
    "id": 125,
    "from": "impl",
    "kind": "validate-pending",
    "timestamp": "<UTC ISO 8601>",
    "question": "workspace-check for v1-SL-a task 1 fix (migration split)",
    "options": ["pass", "fail"],
    "context": "cargo-validate-workspace.yml triggered by push to <branch>; expects clippy::map_err_ignore + non-exhaustive-match still failing (planner-intentional cohort A barrier per DQ #117 Option B); only the migration-check (DQ #126) is the real signal for this fix",
    "workflow_run_id": <workspace databaseId>,
    "branch": "<your-worker-branch>",
    "phase_task": "1-fix",
    "result": null,
    "log_slice": null,
    "failed_jobs": null,
    "answer": null,
    "answered_by": null,
    "resolved_at": null
  }
  ```

  ```json
  {
    "id": 126,
    "from": "impl",
    "kind": "validate-pending",
    "timestamp": "<UTC ISO 8601>",
    "question": "migration-check for v1-SL-a task 1 fix (migration split)",
    "options": ["pass", "fail"],
    "context": "cargo-validate-migration.yml triggered by push to <branch>; expects pass (split fixes Postgres unsafe-use-of-new-enum-value rejection from DQ #122)",
    "workflow_run_id": <migration databaseId>,
    "branch": "<your-worker-branch>",
    "phase_task": "1-fix",
    "result": null,
    "log_slice": null,
    "failed_jobs": null,
    "answer": null,
    "answered_by": null,
    "resolved_at": null
  }
  ```
- Commit + push the DQ entries to your worker branch immediately per `.claude/rules/decision-queue.md` "Mid-task visibility". Two commits acceptable: (1) source-code, (2) DQ entries.

### DQ attribution
- No `answered_by: "advisor"` or `"user"` from this subagent.
- Self-resolve only as `"impl-self-resolved"`.
- New DQ entries from this task use `from: "impl"`.
- Use the **pre-allocated ids 125 and 126**. Do NOT compute next_id; they are reserved for this fix-impl per the DQ #124 forward-guard pattern.

### Commit shape
- Source-code commit:
  - Subject: `fix(v1-SL-a): split combined migration — enum-add directory + grace-window directory (task 1 fix; DQ #122)` (verbatim).
  - Body MUST include the `HANDOVER:` YAML trailer and an Authority-Trail block:
    ```
    Authority trail:
    - DQ #122 (failing log_slice on phase-v1-SL-a tip): "unsafe use
      of new value 'SponsorLiabilityPending' of enum type case_status"
    - PRD §8.5 prescribed combined shape — superseded by this split.
    - Plan §10.1 prescribed combined skeleton — superseded by this split.
    - Mirror: 2026-04-19-000000-0000_add_restoration_sanction_variant
      (Phase 5b enum-only -- no-transaction precedent).
    - Postgres rule: ALTER TYPE ADD VALUE values cannot be referenced
      in the same migration session that added them.

    HANDOVER:
    filesCreated:
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/down.sql
    filesDeleted:
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/down.sql
    filesModified: []
    keyDecisions:
      - split combined SL-a migration into enum-add (2026-05-03-000000-0000)
        + non-enum (2026-05-03-000100-0000) per Phase 5b Restoration precedent
      - migration count delta: SL-a contributes +2 migrations (was +1);
        Task 8 will see this and reflect in PHASE_1_MIGRATION_COUNT
      - preserved all SQL bodies byte-identical except moving 3 ALTER TYPE
        lines from old up.sql into the new enum-add directory
    notes: |
      The PRD §8.5 + plan §10.1 both prescribed the combined shape. This
      fix deviates from both because Postgres rejects the combined
      migration with "unsafe use of new enum value" — neither
      authoritative source accounted for this rule. The deviation is
      grounded in the Phase 5b Restoration migration precedent (already
      in the repo, 2 lines, enum-only) and the Postgres documentation.
      Surface this as a planning-miss for the v1-SL-a retro.
    ```
- DQ commit (separate):
  - Subject: `chore(decision-queue): impl raised DQ #125 + #126 — sl-a-fix-impl-1 validate-pending`
  - Body: brief 2-line summary of the two entries and their workflow_run_ids.

## 5. Validation gate (Shape G)

**No local cargo, no local Diesel, no local migration apply.** Validation is:

1. Push source-code commit to `origin/<your-worker-branch>`.
2. Wait ~10s. Confirm BOTH `cargo-validate-workspace.yml` AND `cargo-validate-migration.yml` triggered (the push touches `migrations/**`, both fire).
3. Capture both workflow run ids.
4. Write the two `kind: "validate-pending"` DQ entries with **id=125 (workspace-check)** and **id=126 (migration-check)**.
5. Commit + push the DQ commit.
6. Exit and let the advisor's polling loop dispatch ci-watchers.

Expected workflow outcomes (advisor-side, post-ci-watcher):

- **DQ #125 (workspace-check):** likely `result: "fail"` because cohort A's workspace failures (clippy::map_err_ignore + non-exhaustive match) are still on the phase tip and Tasks 4+5 haven't shipped yet. This is **planner-intentional per DQ #117 Option B**; the fix-impl is not expected to green workspace-check, only migration-check. The advisor §G4 classifier will hold without auto-fix.
- **DQ #126 (migration-check):** **must pass.** This is the actual signal for whether the split worked. If it fails, the fix-impl approach is wrong and the advisor will catch-fire.

## 6. Expected output (return to advisor)

```
## fix-impl-1 complete — combined SL-a migration split into 2 directories

**Branch:** junior/role-impl-task-...
**Commits:**
  - <sha-a> fix(v1-SL-a): split combined migration ... (task 1 fix; DQ #122)
  - <sha-b> chore(decision-queue): impl raised DQ #125 + #126 — sl-a-fix-impl-1 validate-pending
**Files deleted (2):**
  - migrations/2026-05-03-000000-0000_add_sponsor_liability_grace_window/{up,down}.sql
**Files created (4):**
  - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/{up,down}.sql
  - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/{up,down}.sql
**SQL preservation check:**
  - 3 ALTER TYPE lines moved from old up.sql to new enum-add up.sql ✓
  - 13 INSERT INTO governance_config rows byte-identical ✓
  - 3 nested EXISTS guards in backfill UPDATE preserved ✓
  - both partial-index CREATEs + COMMENTs preserved ✓
  - down.sql LIFO order preserved ✓
**Workflow runs captured:**
  - workspace-check workflow_run_id: <int> → DQ #125
  - migration-check workflow_run_id: <int> → DQ #126
**Next:** advisor dispatches 2 ci-watchers (DQ #125 + #126); on #126 pass, cohort B can dispatch
```

If any pre-write check failed (DQ #122 not visible, the existing combined directory missing, ids 125/126 already in use), replace the above with a DQ catch-fire entry committed + pushed to your worker branch immediately.
