---
phase: v1-SL-d
role: ci-watcher
task: fix-2
brief_n: 1
authored: 2026-05-10
---

# ci-watcher brief — workflow run 25628572689

**Workflow run id:** 25628572689
**Branch:** junior/role-impl-task-v1-sl-d-fix-impl-2-replace-allow-with-expect-dead-code-see-claude-prps-briefs-sl-d-fix-impl-2-md-190
**Phase task:** fix-impl-2
**Paired DQ entry:** #190 (kind: "validate-pending", in pending[])

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25628572689 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25628572689`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` and exit non-zero.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate with `result: "gh_unauth"`, stays in `pending[]`, commit + push, exit 0.

3. **Run-existence check:** `gh run view 25628572689 --json status`. If not found, mutate with `result: "run_not_found"`, stays in `pending[]`, commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25628572689 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → mutate with `result: "timed_out"`, stays in `pending[]`, commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory):**
   ```bash
   conclusion=$(gh run view 25628572689 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → `result: "pass"`, `log_slice: null`, `failed_jobs: null`. Move `pending[]` → `resolved[]`. Commit + push. Exit 0.
   - `failure` → `result: "fail"`, `log_slice` (last 200 lines of `--log-failed`), `failed_jobs` (array). STAYS in `pending[]`. Commit + push. Exit 0.
   - `cancelled` → `result: "cancelled"`, stays in `pending[]`. Commit + push. Exit 0.
   - `timed_out` → `result: "timed_out"`, stays in `pending[]`. Commit + push. Exit 0.
   - other → `result: "fail"`, exact conclusion in `log_slice`. STAYS in `pending[]`. Commit + push. Exit 0.

7. **Verify mutation post-write:** `git diff HEAD~1 -- .claude/decision-queue.json` must show fields-of-existing-entry-changed, no new entry inserted.

## Hard refusals

See `.claude/agents/ci-watcher.md`. Never cargo, never edit code, never write a new DQ entry (except orphan-blocker fallback), never write deprecated kinds, never write `answered_by: "advisor" | "user"`, never trust `--exit-status` exit code.
