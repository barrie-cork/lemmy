# ci-watcher brief — workflow run 25352168003 (e2e on phase-v1-SL-b)

**Workflow run id:** 25352168003
**Branch:** phase-v1-SL-b
**Phase task:** sl-b-fix-1-finalize (Phase 2 e2e re-run on phase tip post fix-1 merge)
**Paired DQ entry:** #151 (kind: "validate-pending", from: "advisor", in pending[])
**Head sha:** e8101d661f626f1e0fac60ef3620a8ddab317105

## Why this is Phase 2 (e2e), not Phase 1 (workspace-check)

Per advisor-orchestrator.md "Stage-shape orchestration → Each impl-task complete (under Shape G)" Phase 2:

- Phase 1 (workspace-check on `junior/*`) already passed for fix-1 — DQ #150 mutated to `pass` at commit 258bc9f30.
- Phase 2 (e2e on `phase-v1-*`) was previously fired implicitly post-DQ-#149-finalize and revealed 6 failures (4 handler bugs + 1 test bug + 1 JM-a fixture bug).
- Fix-1 addressed all three root causes; advisor manually finalize-merged onto phase-v1-SL-b at e8101d661.
- This e2e run was dispatched via `workflow_dispatch` per PR #105 / `feedback_e2e_local_or_dispatch_user_choice`, option (b) — user explicitly chose dispatch over local.

ci-watcher's job is to long-poll `cargo-test-e2e.yml` (which runs `cargo test --workspace --features full --test e2e -- --test-threads=1`, ~26 min single-threaded) and surface the result.

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

Poll `gh run watch 25352168003 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). On return:

1. **Locate the paired entry:** find the entry in `.claude/decision-queue.json` `pending[]` whose `workflow_run_id` matches `25352168003`. If absent, file a `kind: "blocker"` DQ entry from `from: "ci-watcher"` and exit non-zero. Do NOT write a new `validate-pending` entry.

2. **Pre-flight:** `gh auth status`. If unauthorised, mutate paired entry with `result: "gh_unauth"`, `answer`, `answered_by: "ci-watcher"`, `resolved_at`. Stays in `pending[]`. Commit + push, exit 0.

3. **Run-existence check:** `gh run view 25352168003 --json status`. If not found, mutate with `result: "run_not_found"`, fields. Stays in `pending[]`. Commit + push, exit 0.

4. **Long-poll with shell-side timeout:**
   ```bash
   timeout 3600 gh run watch 25352168003 --exit-status --repo barrie-cork/lemmy > /tmp/ci-watch.log 2>&1
   status=$?
   ```
   If `status == 124` → `result: "timed_out"`. Stays in `pending[]`. Commit + push, exit 0.

5. **Disambiguate via conclusion (mandatory):**
   ```bash
   conclusion=$(gh run view 25352168003 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion')
   ```

6. **Classify on `conclusion` and mutate the paired entry:**
   - `success` → `result: "pass"`, nulls, fields. `pending[]` → `resolved[]`. Commit + push. Exit 0.
   - `failure` → `gh run view 25352168003 --log-failed` (last ~200 lines), `failed_jobs` array. `result: "fail"`, `log_slice`, `failed_jobs`, fields. STAYS in `pending[]`. Commit + push. Exit 0.
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

## CRITICAL: ensure_ascii=False on json.dump

Per `feedback_json_dump_ensure_ascii_false.md`: when re-writing `.claude/decision-queue.json`, use `json.dump(d, f, indent=2, ensure_ascii=False)` (or `json.dumps(d, indent=2, ensure_ascii=False)` + `Path.write_text(..., encoding="utf-8")`). Python's default escapes em-dashes (—), §-signs, accents, and any non-ASCII to `\uXXXX`, causing whole-file re-encode and noisy diff.

## Branch context (phase-tip e2e — not a worker branch)

Unlike fix-1's workspace-check (which ran on `junior/advisor-sl-b-fix-1`), this e2e run targets the phase tip `phase-v1-SL-b` directly. There is no worker branch to merge. After this ci-watcher mutates DQ #151 to `result: "pass"`, the **advisor advances to v1-SL-b retro authoring + bm-pr stages** per advisor-orchestrator stage-shape "All §16a stories `[done]`" → "/brehon-verify" → "bm-pr".

## Hard refusals

See `.claude/agents/ci-watcher.md` "Hard refusals". Never cargo, never edit code, never `gh run rerun`, never write a NEW DQ entry (orphan-blocker case excepted), never `kind: "validate-result" | "validate-failed"` (DEPRECATED), never `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust `--exit-status`. Never attempt to merge anything — the phase branch is already at the target tip.
