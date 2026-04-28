# ci-watcher brief — workflow run 25053153541

**Workflow run id:** 25053153541
**Branch:** phase-v1-JM-d
**Phase task:** 3
**Paired DQ entry:** #77 (kind: "validate-pending", in pending[], from: "advisor")

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry #77 in place by matching `workflow_run_id == 25053153541`. Do NOT write a new entry. The entry's `kind` STAYS `"validate-pending"`; only `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by` + `resolved_at` are populated. Entry moves `pending[]` → `resolved[]` ONLY on `result: "pass"`; failures stay in `pending[]` for advisor §G4 triage.

This is the **Phase 2 e2e check** (cargo-test-e2e.yml) for v1-JM-d Task 3, fired by the advisor's push of merge tip `bdfca2783` to phase-v1-JM-d (option (b) — e2e fires on push to phase-v1-*).

Poll `gh run watch 25053153541 --exit-status --repo barrie-cork/lemmy` (single long-poll, wrapped in `timeout 3600` for the 60-min cap). Follow the 7-step procedure from `.claude/agents/ci-watcher.md` literally.

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals" sub-section. In short: never cargo, never edit code, never apply auto-fixes, never use `gh run rerun`, **never write a NEW DQ entry** (always mutate the existing `validate-pending` by `workflow_run_id`; orphan case files a blocker entry and exits non-zero), never write `kind: "validate-result" | "validate-failed"` (DEPRECATED 2026-04-28; option 2 supersedes), never write `kind: "blocker" | "log" | "clarify" | "validate-pending"` as new entries (only the orphan-blocker case writes a new entry), never write `answered_by: "advisor" | "user"` (only `"ci-watcher"`), never trust the `--exit-status` exit code.
