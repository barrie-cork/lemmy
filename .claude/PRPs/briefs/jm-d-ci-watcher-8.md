# ci-watcher brief — workflow run 25073660917

**Workflow run id:** 25073660917
**Branch:** junior/role-impl-task-v1-jm-d-task-4-see-claude-prps-briefs-jm-d-impl-4-md-45
**Phase task:** 4
**Paired DQ entry:** #84 (kind: "validate-pending", in pending[], from: "impl")

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry #84 in place by matching `workflow_run_id == 25073660917`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

This is the **Phase 1 workspace check** (cargo-validate-workspace.yml) for task-4, fired by impl on the worker branch push.

Poll `gh run watch 25073660917 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). Follow the 7-step procedure from `.claude/agents/ci-watcher.md` literally.

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section. In short: never cargo, never edit code, never apply auto-fixes, never use `gh run rerun`, **never write a NEW DQ entry** (always mutate the existing `validate-pending` by `workflow_run_id`; orphan case files a blocker entry and exits non-zero), never write `kind: "validate-result" | "validate-failed"` (DEPRECATED 2026-04-28; option 2 supersedes), never write `kind: "blocker" | "log" | "clarify" | "validate-pending"` as new entries (only the orphan-blocker case writes a new entry), never write `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust the `--exit-status` exit code.
