---
role: impl-task
plan_task: 1-fix-2
phase: v1-SL-a
created: 2026-05-03
related_dq: [132]
preallocated_dq: [133, 134]
---

# Brief — v1-SL-a fix-impl-2 — Move `-- no-transaction` directive to line 1 of split-enum-add migration

## 1. Role + dispatch line

`[role:impl-task] v1-SL-a fix-impl-2 — see .claude/PRPs/briefs/sl-a-fix-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). This is a narrow §G4-classifier-derived fix-impl-task: move the `-- no-transaction` directive from line 14 to line 1 of `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql`. Net file change: **1 file modified, ~13 lines reordered, 0 new files, no source code touched, no logic change.**

**Shape G:** do NOT run cargo, diesel, or `lemmy_diesel_utils` locally. After your commit, push to your worker branch and write **two** `kind: "validate-pending"` DQ entries (workspace-check + e2e on phase-tip after finalize-merge) using the **pre-allocated ids 133 and 134**.

## 2. Scope

**Produce** (one source-code commit + one DQ commit):

1. **Source-code commit:**
   - MODIFY `migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql` only.
   - Move the `-- no-transaction` directive (currently line 14) to **line 1**, before any other content.
   - The 13 header-comment lines that currently precede `-- no-transaction` move to **after** the directive (lines 2-15 shape).
   - All SQL statements (`ALTER TYPE ...`) at the bottom remain unchanged.
2. **DQ commit:** two `kind: "validate-pending"` entries with **id=133 (workspace-check)** and **id=134 (e2e on phase-tip)**, written sequentially after the workflow runs your push triggers.

**Do NOT**:
- Touch ANY other file. Not `down.sql` (already correct — comment-only no-op). Not other migrations. Not `crates/**`, `tests/**`, `scripts/**`, `docs/**`, `.github/**`, `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.claude/PRPs/plans/**`, `.claude/PRPs/prds/**`, `.claude/lessons/**`. The single up.sql is the only path you change.
- Run cargo, diesel, or `lemmy_diesel_utils` locally. Shape G runs both workflows on push.
- Add or remove any SQL statements. Pure reorder of existing lines.
- Edit the comment text content. Pure structural relocation of `-- no-transaction` to line 1, plus the header comments shift down by one line.
- Bump `PHASE_1_MIGRATION_COUNT`. That constant is owned by Task 8. The migration count is unchanged (still the same 2 SL-a migrations).
- Self-pick the directory name — the existing directory `2026-05-03-000000-0000_add_case_status_sponsor_liability_variants` is correct.
- Compute your own DQ ids. Use `id=133` for workspace-check and `id=134` for e2e on phase-tip.

**Commit messages** (exactly):

- Source-code commit: `fix(v1-SL-a): move -- no-transaction directive to line 1 of split-enum-add migration (task 1 fix-2; DQ #132)`
- DQ commit: `chore(decision-queue): impl raised DQ #133 + #134 — sl-a-fix-impl-2 validate-pending`

## 3. Required reading

Read in this order before editing:

1. **DQ #132 on `phase-v1-SL-a` tip** — `git show origin/phase-v1-SL-a:.claude/decision-queue.json` and locate `id: 132`. The `log_slice` contains the failing line:

   ```
   Error: Failed to run 2026-05-03-000000-0000_add_case_status_sponsor_liability_variants
          with: Received an empty query
   ```

   And the diagnostic ROOT CAUSE block in the `log_slice` explains: the `-- no-transaction` directive on line 14 (after 13 header comments) was missed by the diesel migration runner. The runner only recognises `-- no-transaction` as the **first non-blank line**.

2. **Existing failing up.sql on phase tip** — `git show origin/phase-v1-SL-a:migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql` (17 lines). This is the SQL you are reordering. Current shape (lines 1-13 header, line 14 directive, lines 15-17 SQL):

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

3. **Mirror precedent — `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql`** — 2 lines:

   ```sql
   -- no-transaction
   ALTER TYPE sanction_action ADD VALUE IF NOT EXISTS 'Restoration';
   ```

   This is the **canonical pattern**. The directive MUST be the first non-blank line. Header comments (if any) come AFTER the directive, not before.

4. **`.claude/lessons/feedback_lemmy_migration_runner.md`** — explains `-- no-transaction` mechanics. The directive is parsed by the runner before the SQL is sent to Postgres; the parser checks the first non-blank line only.

5. **`.claude/decision-queue.json` on phase-v1-SL-a tip** — confirm `id=133` and `id=134` are not in use. Max id on phase tip is 132 (DQ #132 itself). If you see them already taken, STOP and file a DQ entry — do NOT improvise.

## 3a. Handover from prior cohort

```yaml
prior_cohort_tasks:
  - task: 1-fix
    commit: <find via git log on phase-v1-SL-a — the sl-a-fix-impl-1 commit that split the combined migration>
    filesCreated:
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql
      - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/down.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/up.sql
      - migrations/2026-05-03-000100-0000_add_sponsor_liability_grace_window/down.sql
    filesModified: []
    keyDecisions:
      - split combined migration into two directories per Postgres "unsafe use of new enum value" rule (DQ #122)
      - new enum-only migration uses -- no-transaction directive
      - new enum-only migration's down.sql is intentionally no-op (Postgres can't DROP enum values without rebuild)
    notes: |
      Split shipped per sl-a-fix-impl-1.md but the new enum-only up.sql
      placed `-- no-transaction` on line 14 after 13 header-comment lines,
      missing the line-1 invariant. The SL-a planner did not flag
      directive position as a watchpoint; the mirror precedent
      (2026-04-19-000000-0000_add_restoration_sanction_variant) had
      the directive on line 1 with no preceding comments. This
      fix-impl-2 corrects the position.
  - task: 8
    commit: <find via git log — the e2e PHASE_1_MIGRATION_COUNT bump commit>
    filesCreated: []
    filesModified:
      - crates/server/tests/e2e.rs
    keyDecisions:
      - PHASE_1_MIGRATION_COUNT bumped 12 -> 14 (NOT +1; +2 because the cohort A split added a directory)
      - 8 post-condition probes added (4 post-up.sql + 4 post-down.sql) per plan §10.8
    notes: |
      Task 8 already shipped. Workspace-check passed (DQ #131
      resolved). E2E failed via the redo round-trip on this exact
      migration directory — this fix-impl-2 unblocks the e2e to
      green. Post fix push, the workspace-check + e2e workflows fire
      again on the new junior/* worker branch and (after daemon
      finalize-merge) on phase-v1-SL-a tip.
```

## 4. The fix (precise)

Single-file edit. Reorder line 14 (`-- no-transaction`) to be line 1. The original 13-line header block shifts down to lines 2-14. The 3 ALTER TYPE statements stay at the bottom unchanged.

**New canonical up.sql** (matches mirror pattern at `2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` for line-1 directive position):

```sql
-- no-transaction
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
--   - Fix-2: .claude/PRPs/briefs/sl-a-fix-impl-2.md (DQ #132 — diesel
--     migration runner needs `-- no-transaction` on line 1, not after
--     header comments).
--   - Mirror: migrations/2026-04-19-000000-0000_add_restoration_sanction_variant
--     (Phase 5b precedent for enum-only -- no-transaction migration).

ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityPending';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityFired';
ALTER TYPE case_status ADD VALUE IF NOT EXISTS 'SponsorLiabilityEscaped';
```

Key structural points:
- **Line 1**: `-- no-transaction` (the directive)
- **Lines 2-16**: header comment block (one new line added: the Fix-2 trailer reference)
- **Line 17**: blank line separator
- **Lines 18-20**: 3 ALTER TYPE statements (unchanged from the original)

The directive MUST be the FIRST non-blank line. Mirror at `2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` confirms: line 1 is `-- no-transaction`, line 2 is the ALTER TYPE. We extend that pattern by adding header comments BETWEEN the directive and the SQL — the diesel runner reads line 1 only and consumes the directive.

**No other changes.** down.sql is already correct (comment-only no-op for the enum values that can't be dropped). All other migrations are untouched.

## 5. Validation gate

Per Shape G: push your worker branch and let GH Actions run cargo. Do NOT run cargo locally. After push:

1. **Workspace check** runs automatically on push to `junior/*`. Capture the workflow_run_id via `gh run list --branch <your-junior-branch> --limit 1 --json databaseId --jq '.[0].databaseId'`. Write DQ #133 with `kind: "validate-pending"`, `from: "impl"`, `workflow_run_id: <id>`, `branch: <your-junior-branch>`, `phase_task: "1-fix-2"`. Field shape per `.claude/rules/decision-queue.md` "validate-pending".

2. **E2E on phase-tip** does NOT run on `junior/*` push (cohort B finalize-merge into phase-v1-SL-a is what triggers it). After Junior daemon finalize-merges your worker branch into phase-v1-SL-a, the advisor (laptop) sees the new tip on next poll and raises DQ #134 directly OR you can pre-allocate DQ #134 for the e2e workflow_run_id you anticipate. **Default behaviour:** pre-allocate DQ #134 with `workflow_run_id: null`, `result: null`, `from: "impl"`, `branch: phase-v1-SL-a`, `phase_task: "1-fix-2"`. The advisor populates `workflow_run_id` once the e2e dispatches. (Or, if the user gates Phase 2 e2e to local-laptop default per `feedback_default_local_testing.md`, the advisor mutates DQ #134 to `kind: "validate-pending-laptop-e2e"` shape on the polling cycle that detects the phase-tip move.)

3. Per `feedback_pipes_mask_exit_codes.md`: never pipe cargo or `gh run` through tail/head/grep. Capture full output to `.claude/PRPs/debug/sl-a-fix-impl-2-*.log`.

If workspace-check fails: STOP and surface to advisor via DQ. Do not patch around `cargo-check` or `clippy` failures.

## 6. Expected output (return to advisor)

```
## Task 1-fix-2 complete — v1-SL-a no-transaction-directive-position fix

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - migrations/2026-05-03-000000-0000_add_case_status_sponsor_liability_variants/up.sql (modified — directive moved line 14 → line 1; 1 line added for fix-2 authority trail; ALTER TYPE statements unchanged)
**Validation:** workspace-check workflow run <run-id> dispatched; e2e on phase-tip pending finalize-merge
**Next:** advisor confirms DQ #133 (workspace-check) and DQ #134 (e2e) both pass; cohort B tail closes; SL-a Task 9 retro becomes next.
```

Plus DQ commit `chore(decision-queue): impl raised DQ #133 + #134 — sl-a-fix-impl-2 validate-pending`.

## 7. Why this brief differs from the plan

This brief is a fix-impl, not a plan §13 task. The plan §13 task list ends at Task 9 (retro). This is a §G4-classifier-derived narrow fix following the e2e regression on workflow run 25285713986 surfaced by DQ #132 mutation.

**Authority trail:**
1. **DQ #132 advisor mutation** (commit `f72c5f518` on `phase-v1-SL-a`): the §G4 classifier ran on the `governance e2e` failed-job log; identified the missing line-1 directive against the mirror precedent.
2. **§G4 classifier verdict:** non-allowlist by current rule (allowlist = clippy-doc-lazy-continuation / missing-import / deprecated-API only) but mechanical 1-line fix matching mirror — recommend retro-time allowlist extension to add `no-transaction directive misplacement` as auto-fix class.
3. **Mirror precedent:** `migrations/2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` line 1.

The fix-impl-1 brief shipped the migration split correctly per Postgres "unsafe use of new enum value" rule but didn't carry the directive-line-1 invariant from the mirror. The SL-a planner did not flag directive position as a watchpoint; this is a planner-side gap to surface in retro.

## Constraints

### Branch + commit discipline

- Start on a Junior worktree off `phase-v1-SL-a`. Finalize merges your worktree branch back; do not push to `phase-v1-SL-a` directly.
- One source-code commit (the up.sql edit) + one DQ commit (writing 133 + 134). If workspace-check fails on the first attempt, amend or fixup the source commit; do not split it.
- Mid-task DQ visibility: after writing the DQ commit, **commit + push immediately** to your worker branch per `.claude/CLAUDE.md` cheatsheet.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Memory-cap awareness

The daemon runs under `MemoryMax=10G`, `MemoryHigh=8G`. This task does not run cargo locally (Shape G), so memory is not the bottleneck. Single-file edit + git commit + DQ JSON edit only.

### Plan-cited line numbers may have drifted

The migration file at HEAD has 17 lines per `git show` confirmation. If the file you read on your worker branch differs (extra commits between phase tip and your branch), follow the git output, not this brief.

### Lesson trailer (encouraged)

End the source-code commit-message body with a `LESSON:` line per `feedback_junior_pmd_write_convention.md`:

```
LESSON: diesel migration runner reads `-- no-transaction` ONLY when it is the first non-blank line. Header comments above the directive cause the runner to wrap ALTER TYPE ADD VALUE in a transaction (rejected by Postgres, manifests as "Received an empty query" on redo round-trip). Mirror at `2026-04-19-000000-0000_add_restoration_sanction_variant/up.sql` is the canonical pattern. Watchpoint for next migration plan: enum-only `-- no-transaction` directive must be line 1.
```

### Encoding convention

When writing back the mutated `decision-queue.json`, use **`ensure_ascii=True`** (the default for `json.dump` and the project convention — see the v1-SL-a green-gate recovery session 2026-05-03 commits `e12465f89`, `f72c5f518`). Do NOT use `ensure_ascii=False` — that produces a 400KB+ encoding-only diff.

Recipe:
```python
with open(path, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2, ensure_ascii=True)
```
