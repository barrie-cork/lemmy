# ci-watcher brief — workflow run 25275388975

**Workflow run id:** 25275388975
**Workflow:** cargo-validate-workspace (cargo-validate-workspace.yml)
**Branch:** junior/role-impl-task-v1-sl-a-task-3-see-claude-prps-briefs-sl-a-impl-3-md-86
**Phase task:** 3 workspace-check
**Paired DQ entry:** #118 (kind: "validate-pending", from: "impl", in pending[])

## Cohort A barrier note (planner-intentional fail-pattern, per DQ #117)

This entry is part of the v1-SL-a cohort A 5-way [P] dispatch (Tasks 1+2+3+6+7). Per plan §14 Story 1, the workspace-check `conclusion: "success"` checkpoint is **Task 5's push** (cohort B tail), NOT cohort A. Therefore a `result: "fail"` mutation here is **planner-intentional** if the failure is non-exhaustive-match clippy or missing-field compile errors at Task 4/5 sites. Advisor §G4 classifier holds without auto-queueing fix-impl-tasks for these expected fails.

The ci-watcher's job is unchanged: poll the run, mutate the entry, exit. Failure interpretation is advisor-side after mutation.

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id: 25275388975`. Do NOT write a new entry.

Follow the full procedure in `.claude/PRPs/templates/ci-watcher-brief.template.md` — steps 1 through 7 verbatim. Key points:

- Long-poll: `timeout 3600 gh run watch 25275388975 --exit-status --repo barrie-cork/lemmy`
- Disambiguate result via `gh run view 25275388975 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion'` (do NOT trust `--exit-status` exit code per empirical table in `.claude/agents/ci-watcher.md`)
- On `success`: move DQ #118 from `pending[]` to `resolved[]`, set `result: "pass"`, `answered_by: "ci-watcher"`, `resolved_at`. Commit + push to **the worker branch** `junior/role-impl-task-v1-sl-a-task-3-see-claude-prps-briefs-sl-a-impl-3-md-86`.
- On `failure`: capture `gh run view 25275388975 --log-failed` (last ~200 lines) + `failed_jobs`. Set `result: "fail"`, populate `log_slice` + `failed_jobs`. STAYS in `pending[]`. Commit + push.
- On `cancelled` / `timed_out`: set matching result. STAYS in `pending[]`. Commit + push.

## Hard refusals

See `.claude/agents/ci-watcher.md`. Never cargo, never edit code, never apply auto-fixes, never write a new DQ entry (only mutate existing by `workflow_run_id`), never write deprecated kinds (`validate-result` / `validate-failed`), never write `answered_by: "advisor"` or `"user"`.
