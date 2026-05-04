# ci-watcher brief — workflow run 25297427006

**Workflow run id:** 25297427006
**Branch:** junior/role-impl-task-sl-a-fix-impl-4-see-claude-prps-briefs-sl-a-fix-impl-4-md-111
**Phase task:** fix-impl-4
**Paired DQ entry:** #137 (kind: "validate-pending", in pending[])

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25297427006 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25297427006`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` (the advisor's dispatch contract was violated) and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate the paired entry with `result: "gh_unauth"`, `answer: "<one-liner>"`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25297427006 --json status`. If the run is not found, mutate the paired entry with `result: "run_not_found"`, `answer`, `answered_by`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**

   ```bash
   timeout 3600 gh run watch 25297427006 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```

   If `status == 124` → mutate the paired entry with `result: "timed_out"`, populate `answer` + `answered_by: "ci-watcher"` + `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory — `--exit-status` is unreliable on gh CLI 2.89.0 per the empirical exit-code table in `.claude/agents/ci-watcher.md`):**

   ```bash
   conclusion=$(gh run view 25297427006 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → mutate paired entry with `result: "pass"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Move from `pending[]` to `resolved[]`. Commit + push. Exit 0.
   - `failure` → run `gh run view 25297427006 --log-failed` (slice last ~200 lines), capture `failed_jobs` via `--json jobs --jq '[.jobs[] | select(.conclusion == "failure") | .name]'`. Mutate paired entry with `result: "fail"`, `log_slice` (last 200 lines), `failed_jobs` (array), `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - `cancelled` → mutate paired entry with `result: "cancelled"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push, exit 0.
   - `timed_out` → mutate paired entry with `result: "timed_out"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - other (`action_required` | `neutral` | `skipped` | `stale` | empty) → mutate paired entry with `result: "fail"` and the exact conclusion recorded in `log_slice` for advisor classifier-miss handling. STAYS in `pending[]`. Commit + push. Exit 0.

7. **Verify mutation post-write:** `git diff HEAD~1 -- .claude/decision-queue.json` should show fields-of-existing-entry-changed + position-moved (pending → resolved on pass), no new entry inserted, no entry deleted. If the diff shows a new entry inserted, abort the push, restore the file, and file a blocker DQ entry — the mutation logic was wrong.

The paired entry is found by `workflow_run_id`, NOT by id. The entry's `id`, `from`, `timestamp`, `kind`, `branch`, `phase_task` STAY UNCHANGED. The `next-id` discipline does NOT apply — no new entry is written (except in the orphan-blocker fallback at step 1, where `id` is computed by `max(all_ids) + 1` per `.claude/rules/decision-queue.md` next-id discipline).

## Encoding convention (mandatory)

When writing back the mutated `decision-queue.json`, use **`ensure_ascii=True`** (canonical per the v1-SL-a green-gate recovery session). Do NOT flip to `ensure_ascii=False` — that produces a 400KB+ encoding-only diff.

Recipe:

```python
with open(path, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2, ensure_ascii=True)
```

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section. In short: never cargo, never edit code, never apply auto-fixes, never use `gh run rerun`, never write a new `validate-pending` entry, never write `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust the `--exit-status` exit code.

## Context for this dispatch

DQ #137 is the workspace-check validation for `sl-a-fix-impl-4`, the PR #111 CodeRabbit triage fix task. The workflow run already completed `success` on GitHub before this ci-watcher brief was queued; still run the normal `gh run watch` / conclusion-disambiguation path and mutate DQ #137 in place for the audit trail.

On pass, advisor proceeds to the Phase 2 e2e user gate for the new `phase-v1-SL-a` tip (`5af6a55f9` or later). Do not raise or mutate any e2e entry from this task.
