# ci-watcher brief — workflow run 25084505547

**Workflow run id:** 25084505547
**Branch:** junior/role-impl-task-v1-jm-d-fix-4a-v3-see-claude-prps-briefs-jm-d-fix-impl-4a-v3-md-52
**Phase task:** 4
**Paired DQ entry:** #88 (kind: "validate-pending", in pending[] on worker branch, from: "impl")

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry #88 in place by matching `workflow_run_id == 25084505547`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

This is the **Phase 1 workspace check** (cargo-validate-workspace.yml) for fix-4a-v3 (Task #52), fired by impl on the worker branch push. Workflow already completed (advisor observed `conclusion: success` at queue time); ci-watcher's `gh run watch` long-poll should return immediately with exit 0.

Note: DQ #88 was raised on the worker branch (`junior/...md-52`), not trunk — same pattern as DQ #85 / ci-watcher 9. Read the DQ file from that branch via `git show <worker-branch>:.claude/decision-queue.json` to find the entry. Commit the mutation back to the same worker branch so the daemon's finalize-merge brings it onto trunk on pass.

Poll `gh run watch 25084505547 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). Follow the 7-step procedure from `.claude/agents/ci-watcher.md` literally.

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section. In short: never cargo, never edit code, never apply auto-fixes, never use `gh run rerun`, **never write a NEW DQ entry** (always mutate the existing `validate-pending` by `workflow_run_id`; orphan case files a blocker entry and exits non-zero), never write `kind: "validate-result" | "validate-failed"` (DEPRECATED 2026-04-28; option 2 supersedes), never write `kind: "blocker" | "log" | "clarify" | "validate-pending"` as new entries (only the orphan-blocker case writes a new entry), never write `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust the `--exit-status` exit code.
