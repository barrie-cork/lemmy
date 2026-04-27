# ci-watcher brief — workflow run <run-id>

**Workflow run id:** <id>
**Branch:** <branch>
**Phase task:** <task-number>

## Action

Poll `gh run watch <id> --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Pre-flight:** `gh auth status`. If unauthorised, write `validate-failed result: "gh_unauth"` DQ entry, commit + push, exit 0.

2. **Run-existence check:** `gh run view <id> --json status`. If the run is not found, write `validate-failed result: "run_not_found"`, commit + push, exit 0.

3. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch <id> --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → write `validate-failed result: "timed_out"`, commit + push, exit 0.

4. **Disambiguate via conclusion (mandatory — `--exit-status` is unreliable on gh CLI 2.89.0 per the empirical exit-code table in `.claude/agents/ci-watcher.md`):**
   ```bash
   conclusion=$(gh run view <id> --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

5. **Classify on `conclusion`:**
   - `success` → write `validate-result result: "pass"` to `resolved`, `from: "ci-watcher"`, `answered_by: "ci-watcher-self-resolved"`. Commit + push. Exit 0.
   - `failure` → run `gh run view <id> --log-failed` (slice last ~200 lines), capture `failed_jobs` via `--json jobs --jq '[.jobs[] | select(.conclusion == "failure") | .name]'`. Write `validate-failed result: "fail"` with `log_slice` + `failed_jobs` to `pending`. Commit + push. Exit 0.
   - `cancelled` → write `validate-failed result: "cancelled"` to `pending`. Commit + push. Exit 0.
   - `timed_out` → write `validate-failed result: "timed_out"` to `pending`. Commit + push. Exit 0.
   - other (`action_required` | `neutral` | `skipped` | `stale` | empty) → write `validate-failed result: "fail"` with the exact conclusion recorded in `log_slice` for advisor classifier-miss handling. Commit + push. Exit 0.

The DQ entry's `id` is computed by `max(all_ids) + 1` across both `pending` and `resolved` arrays (and any `decision-queue-archive-*.json` files), per `.claude/rules/decision-queue.md` next-id discipline.

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section. In short: never cargo, never edit code, never apply auto-fixes, never use `gh run rerun`, never write `kind: "validate-pending" | "blocker" | "log" | "clarify"`, never write `answered_by: "advisor" | "user"`, never trust the `--exit-status` exit code.
