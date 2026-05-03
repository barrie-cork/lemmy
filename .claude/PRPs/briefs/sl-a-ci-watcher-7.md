# ci-watcher brief — workflow run 25278271440

**Workflow run id:** 25278271440
**Workflow:** cargo-validate-workspace (cargo-validate-workspace.yml)
**Branch:** junior/role-impl-task-v1-sl-a-fix-impl-1-see-claude-prps-briefs-sl-a-fix-impl-1-md-95
**Phase task:** 1-fix (workspace-check)
**Paired DQ entry:** #125 (kind: "validate-pending", from: "impl", in pending[])

## Cohort A barrier note (planner-intentional fail-pattern, per DQ #117)

This entry continues the v1-SL-a cohort A barrier pattern. Per plan §14 Story 1, the workspace-check `conclusion: "success"` checkpoint is **Task 5's push** (cohort B tail), NOT cohort A or this fix. Therefore a `result: "fail"` mutation here is **planner-intentional** if the failure is non-exhaustive-match clippy or missing-field compile errors at Task 4/5 sites (sponsor_liability handler crates not yet authored).

The ci-watcher's job is unchanged: poll the run, mutate the entry, exit. Failure interpretation is advisor-side after mutation (advisor §G4 classifier).

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id: 25278271440`. Do NOT write a new entry.

Follow the full procedure in `.claude/PRPs/templates/ci-watcher-brief.template.md` — steps 1 through 7 verbatim. Key points:

- Long-poll: `timeout 3600 gh run watch 25278271440 --exit-status --repo barrie-cork/lemmy`
- Disambiguate result via `gh run view 25278271440 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion'` (do NOT trust `--exit-status` exit code per empirical table in `.claude/agents/ci-watcher.md`)
- On `success`: move DQ #125 from `pending[]` to `resolved[]`, set `result: "pass"`, `answered_by: "ci-watcher"`, `resolved_at`. Commit + push to **the worker branch** `junior/role-impl-task-v1-sl-a-fix-impl-1-see-claude-prps-briefs-sl-a-fix-impl-1-md-95`.
- On `failure`: capture `gh run view 25278271440 --log-failed` (last ~200 lines) + `failed_jobs`. Set `result: "fail"`, populate `log_slice` + `failed_jobs`. STAYS in `pending[]`. Commit + push.
- On `cancelled` / `timed_out`: set matching result. STAYS in `pending[]`. Commit + push.

## Hard refusals

See `.claude/agents/ci-watcher.md`. Never cargo, never edit code, never apply auto-fixes, never write a new DQ entry (only mutate existing by `workflow_run_id`), never write deprecated kinds (`validate-result` / `validate-failed`), never write `answered_by: "advisor"` or `"user"`.
