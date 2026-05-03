---
role: impl-task
plan_task: 1-fix-3
phase: v1-SL-a
created: 2026-05-03
related_dq: [134]
preallocated_dq: [135, 136]
---

# Brief — v1-SL-a fix-impl-3 — Add `SELECT 1;` to comments-only down.sql for split-enum-add migration

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a fix-impl-3 — see .claude/PRPs/briefs/sl-a-fix-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a narrow §G4-classifier-derived fix-impl-task: append `SELECT 1;` to `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql`. Net file change: **1 file modified, 1 line appended, 0 new files, no source code touched, no logic change.**

**Shape G:** do NOT run cargo, diesel, or `lemmy_diesel_utils` locally. After your commit, push to your worker branch and write **two** `kind: "validate-pending"` DQ entries (workspace-check + e2e on phase-tip after finalize-merge) using the **pre-allocated ids 135 and 136**.

## 2. Scope

**Produce** (one source-code commit + one DQ commit):

1. **Source-code commit:**
   - MODIFY `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql` only.
   - Append `SELECT 1;` as the new last line, after the existing comment block.
   - All existing comment lines remain unchanged.
2. **DQ commit:** two `kind: "validate-pending"` entries with **id=135 (workspace-check)** and **id=136 (e2e on phase-tip)**, written sequentially after the workflow runs your push triggers.

**Do NOT**:
- Touch ANY other file. Not `up.sql` (already correct after fix-impl-2 — `-- no-transaction` on line 1 + ALTER TYPE statements at bottom). Not other migrations. Not `crates/**`, `tests/**`, `scripts/**`, `docs/**`, `.github/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `.claude/lessons/**`. The single down.sql is the only path you change.
- Run cargo, diesel, or `lemmy_diesel_utils` locally. Shape G runs both workflows on push.
- Add or remove any comment lines. Pure 1-line append.
- Edit the comment text content. Pure trailing addition of `SELECT 1;`.
- Bump `PHASE_1_MIGRATION_COUNT`. That constant is owned by Task 8 (already shipped). The migration count is unchanged.
- Touch `migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/down.sql` — that one HAS DDL (the actual reverse of the columns added in its up.sql), so it is unaffected by this class of bug.
- Compute your own DQ ids. Use `id=135` for workspace-check and `id=136` for e2e on phase-tip.

**Commit messages** (exactly):

- Source-code commit: `fix(v1-SL-a): add SELECT 1; to comments-only down.sql for split-enum-add migration (task 1 fix-3; DQ #134)`
- DQ commit: `chore(decision-queue): impl raised DQ #135 + #136 — sl-a-fix-impl-3 validate-pending`

## 3. Required reading

Read in this order before editing:

1. **DQ #134 on `phase-v1-SL-a` tip** — `git show origin/phase-v1-SL-a:.claude/decision-queue.json` and locate `id: 134`. The `log_slice` contains the failing line:

   ```
   PT0.0203267S revert 2026-05-03-000100-0000_add_sponsor_liability_grace_window
   PT0.00287S revert 2026-05-03-000000-0000_add_case_status_sponsor_liability_variants
   Error: Failed to run 2026-05-03-000000-0000_add_case_status_sponsor_liability_variants
          with: Received an empty query
   ```

   The diagnostic ROOT CAUSE: diesel's `revert_migration` calls `batch_execute` on the down.sql file content. When the file is comments-only (no DDL), Postgres returns "Received an empty query". The mirror precedent has `SELECT 1;` as a no-op DDL to satisfy this check.

2. **Existing failing down.sql on phase tip** — `git show origin/phase-v1-SL-a:migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql` (5 lines). This is the SQL you are appending to. Current shape (5 lines, all comments, no DDL):

   ```sql
   -- Reverse of v1-SL-a task 1 (split half 1 of 2).
   -- Postgres does not support DROP VALUE without a full type rebuild;
   -- the three new variants stay as orphan values in case_status (per
   -- Phase 5b Restoration precedent + PRD §3.4 doc-comment).
   -- This file intentionally has no DDL.
   ```

3. **Mirror precedent — `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/down.sql`** — 7 lines:

   ```sql
   -- no-transaction
   -- Postgres does not support dropping an enum value without rebuilding the entire type.
   -- Down-path is intentionally a no-op; rolling back past this migration requires a full
   -- type rebuild (DROP TYPE + CREATE TYPE + update every column that uses it).
   -- See [99 OQ-003] for the reasoning behind making the variant reservation irreversible
   -- at the v0 migration level.
   SELECT 1;
   ```

   This is the **canonical pattern**. The trailing `SELECT 1;` is the no-op DDL that satisfies Postgres' empty-query check while keeping the migration semantically a no-op. The SL-a down.sql is missing this line.

4. **`.claude/lessons/feedback_lemmy_migration_runner.md`** — explains diesel migration runner mechanics. The runner does not skip empty SQL files; it sends them to Postgres which rejects with "Received an empty query".

5. **`.claude/decision-queue.json` on phase-v1-SL-a tip** — confirm `id=135` and `id=136` are not in use. Max id on phase tip is 134 (DQ #134 itself). If you see them already taken, STOP and file a DQ entry — do NOT improvise.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1-fix-2
    commit: 38a00305e
    filesCreated: []
    filesModified:
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql
    keyDecisions:
      - moved -- no-transaction directive from line 14 to line 1 of up.sql per mirror precedent
      - 13 header-comment lines preserved (shifted to lines 2-15)
      - 3 ALTER TYPE statements at the bottom unchanged
    notes: |
      fix-impl-2 corrected the directive-position bug surfaced by DQ #132.
      Workspace-check passed (DQ #133 resolved at run 25287071958).
      E2E on phase tip 41f84d937 then revealed a SECOND bug: down.sql for
      this same migration directory is comments-only, no DDL. Diesel's
      revert_migration → batch_execute returns "Received an empty query"
      from Postgres. This fix-impl-3 corrects the down.sql by appending
      `SELECT 1;` per the mirror precedent shape.

      Both fix-impl-1 and fix-impl-2 missed this invariant. The mirror
      precedent at 2026-04-19-000000-0000_add_restoration_sanction_variant/
      down.sql ends with `SELECT 1;` (line 7) — neither prior brief
      cited the trailing-DDL invariant alongside the directive-position
      one. Carry-forward to SL-a Task 9 retro: planner-side gap +
      §G4 allowlist extension candidate.
```

## 4. The fix (precise)

Single-file edit. Append one line (`SELECT 1;`) to the end of down.sql. No other changes.

**New canonical down.sql** (matches mirror pattern at `2026-04-19-000000-0000_add_restoration_sanction_variant/down.sql` for trailing no-op DDL):

```sql
-- Reverse of v1-SL-a task 1 (split half 1 of 2).
-- Postgres does not support DROP VALUE without a full type rebuild;
-- the three new variants stay as orphan values in case_status (per
-- Phase 5b Restoration precedent + PRD §3.4 doc-comment).
-- This file intentionally has no DDL beyond a no-op SELECT 1; to
-- satisfy Postgres' empty-query check during diesel revert.
SELECT 1;
```

Key structural points:
- **Lines 1-5**: existing comment block (last line edited slightly to mention the no-op SELECT — see below)
- **Line 6**: NEW comment line about the SELECT (added so future readers understand why)
- **Line 7**: `SELECT 1;` (the no-op DDL)

Wait — re-reading: the SCOPE section says "Append `SELECT 1;` as the new last line" + "All existing comment lines remain unchanged" + "Add or remove any comment lines" is forbidden. So the simpler interpretation IS the canonical: keep the 5 existing comment lines, append `SELECT 1;` as line 6. No comment additions. The mirror precedent's comment text differs slightly because its semantic context (sanction_action vs case_status) differs — copying it verbatim would be wrong.

**The fix is therefore:**

```sql
-- Reverse of v1-SL-a task 1 (split half 1 of 2).
-- Postgres does not support DROP VALUE without a full type rebuild;
-- the three new variants stay as orphan values in case_status (per
-- Phase 5b Restoration precedent + PRD §3.4 doc-comment).
-- This file intentionally has no DDL.
SELECT 1;
```

That is: 5 existing comment lines (verbatim) + line 6 = `SELECT 1;`.

**No other changes.** up.sql is already correct (post-fix-impl-2). All other migrations are untouched.

## 5. Validation gate

Per Shape G: push your worker branch and let GH Actions run cargo. Do NOT run cargo locally. After push:

1. **Workspace check** runs automatically on push to `junior/*`. Capture the workflow_run_id via `gh run list --branch <your-junior-branch> --limit 1 --json databaseId --jq '.[0].databaseId'`. Write DQ #135 with `kind: "validate-pending"`, `from: "impl"`, `workflow_run_id: <id>`, `branch: <your-junior-branch>`, `phase_task: "1-fix-3"`. Field shape per `.claude/rules/decision-queue.md` "validate-pending".

2. **E2E on phase-tip** does NOT run on `junior/*` push. Pre-allocate DQ #136 with `workflow_run_id: null`, `result: null`, `from: "impl"`, `branch: phase-v1-SL-a`, `phase_task: "1-fix-3"`. The advisor mutates DQ #136 to `kind: "validate-pending-laptop-e2e"` shape on the polling cycle that detects the phase-tip move (per `feedback_default_local_testing.md`).

3. Per `feedback_pipes_mask_exit_codes.md`: never pipe cargo or `gh run` through tail/head/grep. Capture full output to `.claude/PRPs/debug/sl-a-fix-impl-3-*.log`.

If workspace-check fails: STOP and surface to advisor via DQ. Do not patch around `cargo-check` or `clippy` failures.

## 6. Expected output (return to advisor)

```
## Task 1-fix-3 complete — v1-SL-a comments-only-down.sql fix

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql (modified — appended SELECT 1; as line 6 to satisfy Postgres empty-query check on diesel revert)
**Validation:** workspace-check workflow run <run-id> dispatched; e2e on phase-tip pending finalize-merge
**Next:** advisor confirms DQ #135 (workspace-check) and DQ #136 (e2e) both pass; cohort B tail closes; SL-a Task 9 retro becomes next.
```

Plus DQ commit `chore(decision-queue): impl raised DQ #135 + #136 — sl-a-fix-impl-3 validate-pending`.

## 7. Why this brief differs from the plan

This brief is a fix-impl, not a plan §13 task. The plan §13 task list ends at Task 9 (retro). This is a §G4-classifier-derived narrow fix following the e2e regression on the local-laptop e2e run for phase tip 41f84d937 surfaced by DQ #134 mutation.

**Authority trail:**
1. **DQ #134 advisor-laptop mutation** (commit `06b4d9da3` on `phase-v1-SL-a`): the §G4 classifier ran on the e2e log; identified `down.sql is comments-only` against the mirror precedent's `SELECT 1;` trailing no-op DDL.
2. **§G4 classifier verdict:** non-allowlist by current rule (allowlist = clippy-doc-lazy-continuation / missing-import / deprecated-API only) but mechanical 1-line fix matching mirror — recommend retro-time allowlist extension to add `comments-only down.sql + diesel revert empty-query` as auto-fix class.
3. **Mirror precedent:** `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/down.sql` line 7.

The fix-impl-1 brief shipped the migration split correctly per Postgres "unsafe use of new enum value" rule but didn't carry the trailing-no-op-DDL invariant from the mirror. The fix-impl-2 brief corrected the directive-position bug but didn't preempt the trailing-no-op-DDL one. The SL-a planner did not flag the empty-query/SELECT-1 invariant; this is a planner-side gap to surface in retro alongside the directive-position one.

## Constraints

### Branch + commit discipline

- Start on a Junior worktree off `phase-v1-SL-a`. Finalize merges your worktree branch back; do not push to `phase-v1-SL-a` directly.
- One source-code commit (the down.sql edit) + one DQ commit (writing 135 + 136). If workspace-check fails on the first attempt, amend or fixup the source commit; do not split it.
- Mid-task DQ visibility: after writing the DQ commit, **commit + push immediately** to your worker branch per `.claude/CLAUDE.md` cheatsheet.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Memory-cap awareness

The daemon runs under `MemoryMax=10G`, `MemoryHigh=8G`. This task does not run cargo locally (Shape G), so memory is not the bottleneck. Single-file edit + git commit + DQ JSON edit only.

### Plan-cited line numbers may have drifted

The migration file at HEAD has 5 lines per `git show` confirmation. If the file you read on your worker branch differs (extra commits between phase tip and your branch), follow the git output, not this brief.

### Lesson trailer (encouraged)

End the source-code commit-message body with a `LESSON:` line per `feedback_junior_pmd_write_convention.md`:

```
LESSON: diesel migration runner does not skip empty SQL files. A comments-only down.sql causes diesel's revert_migration → batch_execute to send an all-comment string to Postgres, which returns "Received an empty query" (rejecting the revert). The mirror at `2026-04-19-000000-0000_add_restoration_sanction_variant/down.sql` ends with `SELECT 1;` — a no-op DDL that satisfies Postgres while keeping the migration semantically a no-op. Watchpoint for next migration plan: any migration where down.sql is "intentionally a no-op" must append `SELECT 1;` (or equivalent no-op SQL) to satisfy diesel's revert path.
```

### Encoding convention

When writing back the mutated `decision-queue.json`, use **`ensure_ascii=True`** (the default for `json.dump` and the project convention — see the v1-SL-a green-gate recovery session 2026-05-03 commits `e12465f89`, `f72c5f518`, `06b4d9da3`). Do NOT use `ensure_ascii=False` — that produces a 400KB+ encoding-only diff.

Recipe:
```python
with open(path, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2, ensure_ascii=True)
```
