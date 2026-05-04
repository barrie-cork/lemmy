# ci-watcher brief — workflow run 25285713986

**Workflow run id:** 25285713986
**Branch:** phase-v1-SL-a
**Phase task:** 8
**Paired DQ entry:** #132 (kind: "validate-pending", in pending[])

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25285713986 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25285713986`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` (the advisor's dispatch contract was violated) and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate the paired entry with `result: "gh_unauth"`, `answer: "<one-liner>"`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25285713986 --json status`. If the run is not found, mutate the paired entry with `result: "run_not_found"`, `answer`, `answered_by`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**

   ```bash
   timeout 3600 gh run watch 25285713986 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```

   If `status == 124` → mutate the paired entry with `result: "timed_out"`, populate `answer` + `answered_by: "ci-watcher"` + `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory — `--exit-status` is unreliable on gh CLI 2.89.0 per the empirical exit-code table in `.claude/agents/ci-watcher.md`):**

   ```bash
   conclusion=$(gh run view 25285713986 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → mutate paired entry with `result: "pass"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Move from `pending[]` to `resolved[]`. Commit + push. Exit 0.
   - `failure` → run `gh run view 25285713986 --log-failed` (slice last ~200 lines), capture `failed_jobs` via `--json jobs --jq '[.jobs[] | select(.conclusion == "failure") | .name]'`. Mutate paired entry with `result: "fail"`, `log_slice` (last 200 lines), `failed_jobs` (array), `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - `cancelled` → mutate paired entry with `result: "cancelled"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - `timed_out` → mutate paired entry with `result: "timed_out"`, `log_slice: null`, `failed_jobs: null`, `answer`, `answered_by`, `resolved_at`. STAYS in `pending[]`. Commit + push. Exit 0.
   - other (`action_required` | `neutral` | `skipped` | `stale` | empty) → mutate paired entry with `result: "fail"` and the exact conclusion recorded in `log_slice` for advisor classifier-miss handling. STAYS in `pending[]`. Commit + push. Exit 0.

7. **Verify mutation post-write:** `git diff HEAD~1 -- .claude/decision-queue.json` should show fields-of-existing-entry-changed + position-moved (pending → resolved on pass), no new entry inserted, no entry deleted. If the diff shows a new entry inserted, abort the push, restore the file, and file a blocker DQ entry — the mutation logic was wrong.

The paired entry is found by `workflow_run_id`, NOT by id. The entry's `id`, `from`, `timestamp`, `kind`, `branch`, `phase_task` STAY UNCHANGED. The `next-id` discipline does NOT apply — no new entry is written (except in the orphan-blocker fallback at step 1, where `id` is computed by `max(all_ids) + 1` per `.claude/rules/decision-queue.md` next-id discipline).

## Encoding convention (mandatory)

When writing back the mutated `decision-queue.json`, use **`ensure_ascii=True`** (the default for `json.dump` and the project convention — see `feedback_json_dump_ensure_ascii_false.md`'s actual nuance: the lesson title was a misnomer; the v1-SL-a green-gate recovery session 2026-05-03 confirmed `ensure_ascii=True` is canonical to keep diffs minimal in this DQ file). The advisor session set the file at `chore(decision-queue): mutate DQ #132 ...` (commit e12465f89) using `ensure_ascii=True`. Do NOT flip to `ensure_ascii=False` mid-session — that produces a 400KB+ encoding-only diff.

Recipe:

```python
with open(path, "w", encoding="utf-8") as f:
    json.dump(d, f, indent=2, ensure_ascii=True)
```

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section. In short: never cargo, never edit code, never apply auto-fixes, never use `gh run rerun`. Writing a new DQ entry is forbidden **except** in the orphan fallback case (step 1: `workflow_run_id` not found in `pending[]`), where exactly one `kind: "blocker"` entry is written and the task exits non-zero. Outside that orphan case, writing any new DQ entry — including `kind: "blocker"`, `"log"`, `"clarify"`, or `"validate-pending"` — is forbidden. Always mutate the existing `validate-pending` entry by matching `workflow_run_id`. Never write `kind: "validate-result" | "validate-failed"` (DEPRECATED 2026-04-28; option 2 supersedes). Never write `answered_by: "advisor" | "user"` (only `"ci-watcher"`). Never trust the `--exit-status` exit code.

## Context for this dispatch

The paired DQ #132 originated as `kind: "validate-pending-laptop-e2e"` for a local laptop e2e run on phase-v1-SL-a tip 9c67c0648 (Task 8 finalize-merge). The local run failed at link stage with LNK1104 (orphan stale `e2e-13e14f510bbf8509.exe` PID 7512 holding the file lock per `feedback_orphan_test_exe_blocks_relink.md`). Compile succeeded; only the linker step crashed. User chose option (b) GH-Actions dispatch escape-hatch per advisor-orchestrator.md "Phase 2 e2e — local vs dispatch". Advisor mutated DQ #132 from laptop-form to dispatch-form (commit e12465f89, branch phase-v1-SL-a, pushed 2026-05-03 17:21 UTC). This is the ONLY active validate-pending entry — no other workspace-check or e2e runs are in flight.

On `result: "pass"` the SL-a cohort B tail closes and Task 9 (SL-a retro) becomes the next advisor action. On `result: "fail"` the failure routes to advisor §G4 classifier (allowlist match → narrow fix-impl-task; non-allowlist → catch-fire to user). The wall-clock budget for this run is ~26 min standard e2e + ~5-10 min cold-cache cargo build = ~30-35 min on GitHub-hosted runner. Within the 60-min `timeout 3600` cap.
