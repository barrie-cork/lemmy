# ci-watcher brief — workflow run 25338856985

**Workflow run id:** 25338856985
**Branch:** junior/role-impl-task-sl-b-fix-impl-1-see-claude-prps-briefs-sl-b-fix-impl-1-md-120
**Phase task:** sl-b-fix-impl-1
**Paired DQ entry:** #146 (kind: "validate-pending", in pending[])

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25338856985 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25338856985`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate paired entry with `result: "gh_unauth"`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25338856985 --json status`. If not found, mutate with `result: "run_not_found"`, fields. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25338856985 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → `result: "timed_out"`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory):**
   ```bash
   conclusion=$(gh run view 25338856985 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → `result: "pass"`, nulls, fields. `pending[]` → `resolved[]`. Commit + push. Exit 0.
   - `failure` → `gh run view 25338856985 --log-failed` (last ~200 lines), `failed_jobs` array. `result: "fail"`, `log_slice`, `failed_jobs`, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - `cancelled` → `result: "cancelled"`, nulls, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - `timed_out` → `result: "timed_out"`, nulls, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - other → `result: "fail"` + conclusion in `log_slice`. STAYS in `pending[]`. Commit + push. Exit 0.

7. **Verify mutation post-write:** `git diff HEAD~1 -- .claude/decision-queue.json` — fields-of-existing-entry-changed + position-moved (pending → resolved on pass), no new entry inserted, no entry deleted.

The paired entry is found by `workflow_run_id`, NOT by id. The entry's `id`, `from`, `timestamp`, `kind`, `branch`, `phase_task` STAY UNCHANGED.

## CRITICAL: commit-and-push hygiene

Per `feedback_ci_watcher_haiku_premature_kill.md` (provisional — sl-b-ci-watcher-2 was killed pre-commit at 52s wall-clock with the mutation prepared but uncommitted): **commit IMMEDIATELY after the python mutation script runs, before any verification step**. The order is:

1. python script writes `.claude/decision-queue.json`
2. `git add .claude/decision-queue.json`
3. `git commit -m '<subject>'` — DO THIS FIRST
4. `git push origin <worker-branch>` — THEN PUSH
5. THEN run `git diff HEAD~1 ...` for the verify step (it can't undo the commit)

If a verify step finds anomalies, file a NEW `kind: "blocker"` DQ entry — do not try to undo the commit.

## Hard refusals

See `.claude/agents/ci-watcher.md` "Hard refusals". Never cargo, never edit code, never `gh run rerun`, never write a NEW DQ entry (orphan-blocker case excepted), never `kind: "validate-result" | "validate-failed"` (DEPRECATED), never `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust `--exit-status`.
