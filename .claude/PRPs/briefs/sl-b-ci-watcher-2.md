# ci-watcher brief — workflow run 25337115742

**Workflow run id:** 25337115742
**Branch:** junior/role-impl-task-sl-b-impl-2-see-claude-prps-briefs-sl-b-impl-2-md-118
**Phase task:** sl-b-impl-2
**Paired DQ entry:** #145 (kind: "validate-pending", in pending[])

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25337115742 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25337115742`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate the paired entry with `result: "gh_unauth"`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25337115742 --json status`. If the run is not found, mutate the paired entry with `result: "run_not_found"`, `answer`, `answered_by`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25337115742 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → mutate with `result: "timed_out"`, populate fields. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory):**
   ```bash
   conclusion=$(gh run view 25337115742 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → mutate with `result: "pass"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Move `pending[]` → `resolved[]`. Commit + push. Exit 0.
   - `failure` → run `gh run view 25337115742 --log-failed` (last ~200 lines), capture `failed_jobs`. Mutate with `result: "fail"`, `log_slice`, `failed_jobs`, `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - `cancelled` → mutate with `result: "cancelled"`, nulls, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - `timed_out` → mutate with `result: "timed_out"`, nulls, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - other (`action_required` | `neutral` | `skipped` | `stale` | empty) → `result: "fail"` + conclusion in `log_slice`. STAYS in `pending[]`. Commit + push. Exit 0.

7. **Verify mutation post-write:** `git diff HEAD~1 -- .claude/decision-queue.json` — fields-of-existing-entry-changed + position-moved (pending → resolved on pass), no new entry inserted, no entry deleted.

The paired entry is found by `workflow_run_id`, NOT by id. The entry's `id`, `from`, `timestamp`, `kind`, `branch`, `phase_task` STAY UNCHANGED.

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals". Never cargo, never edit code, never `gh run rerun`, never write a NEW DQ entry (always mutate existing by `workflow_run_id`; orphan case files blocker + exit non-zero), never `kind: "validate-result" | "validate-failed"` (DEPRECATED), never write `kind: "blocker" | "log" | "clarify" | "validate-pending"` as new entries except orphan-blocker case, never `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust `--exit-status`.
