---
role: impl-task
plan_task: 3
phase: v1-JM-d
created: 2026-04-29
status: ready
related_dq: 77
---

# Brief — v1-JM-d Fix 3a — e2e backfill test migration revert limit

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d fix-3a — see .claude/PRPs/briefs/jm-d-fix-impl-3a.md`

You are the **impl-task** subagent. This is a **narrow fix-impl-task** triggered by §G4 classifier on DQ #77 (Phase 2 e2e failure, workflow run 25053153541).

## 2. Scope

**Root cause:** Task 3 added 2 new migrations (`2026-04-27-000000-0000_add_appeal_requester_role_enum` + `2026-04-27-000100-0000_add_appeals_v1_columns`). The e2e test `v1_jm_a_backfill_populates_v0_snapshot` at `crates/server/tests/e2e.rs:570` uses `schema_setup::run(Options::default().revert().limit(4), &db_url)?;` to revert the last 4 migrations LIFO. Before Task 3 the last 4 were the 4 JM-a migrations (`2026-04-23-000000` through `2026-04-23-000200`). Task 3's 2 new migrations shifted the LIFO window: `limit(4)` now reverts only back to `2026-04-23-000100`, never reaching `2026-04-23-000000` which drops the `severity_tier` enum type. The test then asserts `severity_tier` is absent and panics.

**8 tests failed, 54 passed.** The `v1_jm_a_backfill` panic is the confirmed root cause. The other 7 failures may be cascade (shared testcontainers poisoning from the panic), or they may be independent failures from Task 3's code changes. You must investigate each.

**Produce** (one commit):

1. **`crates/server/tests/e2e.rs:570`** — change `revert().limit(4)` to `revert().limit(6)`. Update the surrounding comment (lines 566-569) to reflect that the 6 = 4 JM-a migrations + 2 JM-d Task 3 migrations.

2. **Investigate the other 7 failing tests** — for each, determine if the failure is:
   - (a) cascade from the `v1_jm_a_backfill` panic (fix #1 resolves it), or
   - (b) an independent issue from Task 3's code changes (new enum variants, changed function signatures, new columns in raw SQL inserts, etc.)

   The 7 other failing tests:
   - `admin_get_config_single_key_with_provenance`
   - `admin_list_rule_sets_returns_versions_with_active_version_id`
   - `admin_set_config_persists_previous_value_and_from`
   - `all_mvp_endpoints_return_non_404`
   - `appeal_inside_window_succeeds_expired_rejects`
   - `submit_jury_vote_writes_appeal_window_live_config`
   - `v0_case_completes_under_v0_rules_after_v1_config_flip`

   If any are (b), fix them in the same commit. If you cannot determine root cause without running cargo locally, file a DQ entry with your analysis and exit.

3. **Do NOT touch** `PHASE_1_MIGRATION_COUNT` at line 346 — that test is `#[ignore]`'d and has documented uncounted drift from v1-AD-a. Separate concern per GH issue #43.

**Do NOT:**
- Run cargo locally (Shape G — validation runs on GH Actions)
- Edit any migration `up.sql` or `down.sql` files (the migrations are correct; the test's revert limit is wrong)
- Edit any file outside `crates/server/tests/e2e.rs`
- Add new tests

## 3. Required reading

- `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §13 Task 3 (understand what Task 3 shipped)
- `crates/server/tests/e2e.rs` lines 560-620 (the failing `v1_jm_a_backfill` test)
- `crates/server/tests/e2e.rs` — search for each of the 7 other failing test names to understand their setup
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — do NOT use the Edit tool on the full 8945-line e2e.rs; use targeted line-range reads + surgical edits

## 4. Constraints

- One commit. Subject: `fix(test): bump e2e backfill revert limit 4→6 for JM-d Task 3 migrations`
- Push to your worktree branch. Do NOT push to `phase-v1-JM-d` or `governance-v0`.
- Post-commit: raise a `kind: "validate-pending"` DQ entry per the Shape G contract (§5 of jm-d-impl-3.md has the JSON skeleton — same shape, your branch, your workflow_run_id from `gh run list`).
- If any of the 7 tests have independent failures you cannot fix without running cargo, file a DQ `kind: "blocker"` entry with your analysis (test name, suspected root cause, files involved) and exit cleanly. The advisor will triage.
