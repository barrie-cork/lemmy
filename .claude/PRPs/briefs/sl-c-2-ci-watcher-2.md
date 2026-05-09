# ci-watcher brief — workflow run 25595869651 (sl-c-2 fix-impl-1 Phase 1 workspace check)

**Workflow run id:** 25595869651
**Branch:** `junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154` (DQ #165 raised on the worker branch by impl-task #158 (fix-impl-1) at `3fdb666a4`)
**Phase task:** sl-c-2-fix-impl-1 (E0277 fix — test fn return type changed to LemmyResult<()>)
**Paired DQ entry:** #165 (`kind: "validate-pending"`, in `pending[]` on the worker branch)

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25595869651 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25595869651`. The Junior daemon will worktree-cut from the worker branch (above) for this ci-watcher dispatch; DQ #165 is on that base branch already. Read the file directly. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` (the advisor's dispatch contract was violated) and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate the paired entry with `result: "gh_unauth"`, `answer: "<one-liner>"`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25595869651 --json status --repo barrie-cork/lemmy`. If the run is not found, mutate the paired entry with `result: "run_not_found"`, `answer`, `answered_by`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25595869651 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → mutate the paired entry with `result: "timed_out"`, populate `answer` + `answered_by: "ci-watcher"` + `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory):**
   ```bash
   conclusion=$(gh run view 25595869651 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry** per the template
   `.claude/PRPs/templates/ci-watcher-brief.template.md`.

7. **Verify mutation post-write** per the template above.

## Hard refusals

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section and `.claude/PRPs/templates/ci-watcher-brief.template.md`.
