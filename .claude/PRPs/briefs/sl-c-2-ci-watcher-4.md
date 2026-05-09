# ci-watcher brief — workflow run 25605783542

**Workflow run id:** 25605783542
**Branch:** junior/role-impl-task-sl-c-2-impl-1-replan-see-claude-prps-briefs-sl-c-2-impl-1-replan-md-162
**Phase task:** 1 (replan — Case A LemmyResult<T> uniform shape)
**Paired DQ entry:** #167 (kind: "validate-pending", in pending[])

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25605783542 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25605783542`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` (the advisor's dispatch contract was violated) and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate the paired entry with `result: "gh_unauth"`, `answer: "<one-liner>"`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25605783542 --json status`. If the run is not found, mutate the paired entry with `result: "run_not_found"`, `answer`, `answered_by`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25605783542 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → mutate the paired entry with `result: "timed_out"`, populate `answer` + `answered_by: "ci-watcher"` + `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory — `--exit-status` is unreliable on gh CLI 2.89.0 per the empirical exit-code table in `.claude/agents/ci-watcher.md`):**
   ```bash
   conclusion=$(gh run view 25605783542 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → mutate paired entry with `result: "pass"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Move from `pending[]` to `resolved[]`. Commit + push. Exit 0.
   - `failure` → run `gh run view 25605783542 --log-failed` (slice last ~200 lines), capture `failed_jobs` via `--json jobs --jq '[.jobs[] | select(.conclusion == "failure") | .name]'`. Mutate paired entry with `result: "fail"`, `log_slice` (last 200 lines), `failed_jobs` (array), `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - `cancelled` → mutate paired entry with `result: "cancelled"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - `timed_out` → mutate paired entry with `result: "timed_out"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - other (`action_required` | `neutral` | `skipped` | `stale` | empty) → mutate paired entry with `result: "fail"` and the exact conclusion recorded in `log_slice` for advisor classifier-miss handling. STAYS in `pending[]`. Commit + push. Exit 0.

7. **Verify mutation post-write:** `git diff HEAD~1 -- .claude/decision-queue.json` should show fields-of-existing-entry-changed + position-moved (pending → resolved on pass), no new entry inserted, no entry deleted. If the diff shows a new entry inserted, abort the push, restore the file, and file a blocker DQ entry — the mutation logic was wrong.

The paired entry is found by `workflow_run_id`, NOT by id. The entry's `id`, `from`, `timestamp`, `kind`, `branch`, `phase_task` STAY UNCHANGED. The `next-id` discipline does NOT apply — no new entry is written (except in the orphan-blocker fallback at step 1, where `id` is computed by `max(all_ids) + 1` across both live + archive files per `.claude/rules/decision-queue.md` next-id discipline).

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section. In short: never cargo, never edit code, never apply auto-fixes, never use `gh run rerun`, **never write a NEW DQ entry** (always mutate the existing `validate-pending` by `workflow_run_id`; orphan case files a blocker entry and exits non-zero), never write `kind: "validate-result" | "validate-failed"` (DEPRECATED 2026-04-28; option 2 supersedes), never write `kind: "blocker" | "log" | "clarify" | "validate-pending"` as new entries (only the orphan-blocker case writes a new entry), never write `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust the `--exit-status` exit code.

## Context (replan-specific)

- This Phase-1 workspace check follows the v1-SL-c-2 Task 1 replan (3-cycle E0277 catch-fire 2026-05-09; canonical RCA at `.claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md`).
- Prior workflow runs (cycles 1-3) ALL FAILED with E0277 LemmyError-vs-Box<dyn Error> shape mismatch: `25582548670`, `25595869651`, `25603848858`. The replan converts `mod v1_sl_c_fixtures` to uniform `LemmyResult<T>` (Case A per `feedback_lemmy_error_no_std_error.md`).
- Worker branch was renumbered post-push: original DQ id was `164`, collision with governance-v0 supersession entries forced renumber to `167`. Worker tip = `0be05677` (renumber commit) on top of `b5a1a741b` (replan fix). Both pushed, daemon-side ref refreshed.
- Expected outcome: `success` on this run signals end of the 3-cycle catch-fire; advance to Phase-2 e2e per stage-shape orchestration §3.1.
