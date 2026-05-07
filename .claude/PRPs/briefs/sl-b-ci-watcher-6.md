# ci-watcher brief — workflow run 25351212142

**Workflow run id:** 25351212142
**Branch:** junior/advisor-sl-b-fix-1
**Phase task:** sl-b-fix-1 (advisor-direct, 3 bug fixes from Phase 2 e2e on phase tip 9cfbbf977: handler community_id Option split + 2 test bugs)
**Paired DQ entry:** #150 (kind: "validate-pending", from: "advisor", in pending[])
**Head sha:** 4934db25d54ca7336c0d6e38337c72d28baceec8

## Why this is advisor-direct (not impl-task)

Per PMD #117: Junior workers reliably hang on Edit calls into the now-11910-line `crates/server/tests/e2e.rs`. The advisor session bundled all 3 bug fixes as one commit on `junior/advisor-sl-b-fix-1` to dodge the hang and compress 3 ci-watcher round-trips → 1.

Laptop pre-validation already passed all 3 commands before push:
- cargo check --workspace --features full (2m 26s, exit 0)
- cargo clippy --workspace --features full --no-deps -- -D warnings (2m 17s, exit 0)
- cargo test --no-run -p lemmy_server --test e2e (2m 00s, exit 0)

ci-watcher's job is to confirm GH workspace-check matches that result and produce canonical PR evidence.

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25351212142 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25351212142`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate paired entry with `result: "gh_unauth"`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25351212142 --json status`. If not found, mutate with `result: "run_not_found"`, fields. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25351212142 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → `result: "timed_out"`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory):**
   ```bash
   conclusion=$(gh run view 25351212142 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → `result: "pass"`, nulls, fields. `pending[]` → `resolved[]`. Commit + push. Exit 0.
   - `failure` → `gh run view 25351212142 --log-failed` (last ~200 lines), `failed_jobs` array. `result: "fail"`, `log_slice`, `failed_jobs`, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - `cancelled` → `result: "cancelled"`, nulls, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - `timed_out` → `result: "timed_out"`, nulls, fields. STAYS in `pending[]`. Commit + push. Exit 0.
   - other → `result: "fail"` + conclusion in `log_slice`. STAYS in `pending[]`. Commit + push. Exit 0.

7. **Verify mutation post-write:** `git diff HEAD~1 -- .claude/decision-queue.json` — fields-of-existing-entry-changed + position-moved (pending → resolved on pass), no new entry inserted, no entry deleted.

The paired entry is found by `workflow_run_id`, NOT by id. The entry's `id`, `from`, `timestamp`, `kind`, `branch`, `phase_task` STAY UNCHANGED.

## CRITICAL: commit-and-push hygiene

Per `feedback_ci_watcher_haiku_premature_kill.md`: **commit IMMEDIATELY after the python mutation script runs, before any verification step**. The order is:

1. python script writes `.claude/decision-queue.json`
2. `git add .claude/decision-queue.json`
3. `git commit -m '<subject>'` — DO THIS FIRST
4. `git push origin phase-v1-SL-b` — THEN PUSH
5. THEN run `git diff HEAD~1 ...` for the verify step (it can't undo the commit)

If a verify step finds anomalies, file a NEW `kind: "blocker"` DQ entry — do not try to undo the commit.

**Push target is phase-v1-SL-b** — that's where DQ #150 was raised (the SL-b DQ-of-record lives on the phase branch, not on the worker branch). The worker branch `junior/advisor-sl-b-fix-1` is for the cargo work only.

## CRITICAL: ensure_ascii=False on json.dump

Per `feedback_json_dump_ensure_ascii_false.md`: when re-writing `.claude/decision-queue.json`, use `json.dump(d, f, indent=2, ensure_ascii=False)` (or `json.dumps(d, indent=2, ensure_ascii=False)` + `Path.write_text(..., encoding="utf-8")`). Python's default escapes em-dashes (—), §-signs, accents, and any non-ASCII to `\uXXXX`, causing whole-file re-encode and noisy diff.

## Branch context (NEW for this brief)

The worker branch `junior/advisor-sl-b-fix-1` is NOT a Junior daemon worktree — it was authored on the laptop by the advisor session. There is no `/srv/brehon-fork/.junior/worktrees/job-N` to merge from. After this ci-watcher task mutates DQ #150 to `result: "pass"`, the **advisor will manually finalize-merge** `junior/advisor-sl-b-fix-1` → `phase-v1-SL-b` from the laptop, then re-run Phase 2 e2e (target: 76+ pass, 0 fail). ci-watcher must NOT attempt to merge or finalize.

## Hard refusals

See `.claude/agents/ci-watcher.md` "Hard refusals". Never cargo, never edit code, never `gh run rerun`, never write a NEW DQ entry (orphan-blocker case excepted), never `kind: "validate-result" | "validate-failed"` (DEPRECATED), never `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust `--exit-status`. Never attempt to merge `junior/advisor-sl-b-fix-1` into anything — that's the advisor's manual finalize step.
