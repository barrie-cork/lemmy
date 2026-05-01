# ci-watcher brief — workflow run 25213381649

**Workflow run id:** 25213381649
**Branch:** junior/role-impl-task-v1-jm-e-fix-impl-2-see-claude-prps-briefs-jm-e-fix-impl-2-md-72
**Phase task:** 1-fix2
**Paired DQ entry:** #102 (kind: "validate-pending", in pending[])

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id: 25213381649`. Do NOT write a new entry.

Follow the full procedure in `.claude/PRPs/templates/ci-watcher-brief.template.md` — steps 1 through 7 verbatim. Key points:

- Long-poll: `timeout 3600 gh run watch 25213381649 --exit-status --repo barrie-cork/lemmy`
- Disambiguate result via `gh run view 25213381649 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion'` (do NOT trust `--exit-status` exit code per empirical table in `.claude/agents/ci-watcher.md`)
- On `success`: move DQ #102 from `pending[]` to `resolved[]`, set `result: "pass"`, `answered_by: "ci-watcher"`, `resolved_at`. Commit + push.
- On `failure`: capture `gh run view 25213381649 --log-failed` (last ~200 lines) + `failed_jobs`. Set `result: "fail"`, populate `log_slice` + `failed_jobs`. STAYS in `pending[]`. Commit + push.
- On `cancelled` / `timed_out`: set matching result. STAYS in `pending[]`. Commit + push.

## Hard refusals

See `.claude/agents/ci-watcher.md`. Never cargo, never edit code, never apply auto-fixes, never write a new DQ entry (only mutate existing by `workflow_run_id`), never write deprecated kinds (`validate-result` / `validate-failed`), never write `answered_by: "advisor"` or `"user"`.
