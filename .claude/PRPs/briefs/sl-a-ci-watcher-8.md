# ci-watcher brief — workflow run 25278271449

**Workflow run id:** 25278271449
**Workflow:** cargo-validate-migration (cargo-validate-migration.yml)
**Branch:** junior/role-impl-task-v1-sl-a-fix-impl-1-see-claude-prps-briefs-sl-a-fix-impl-1-md-95
**Phase task:** 1-fix (migration-check)
**Paired DQ entry:** #126 (kind: "validate-pending", from: "impl", in pending[])

## Critical-gate note (split-fix verification)

Unlike workspace-check entries in this sub-phase, the migration-check `conclusion: "success"` IS the **green-gate** that proves the migration split fix worked. A `result: "pass"` here verifies that Postgres no longer rejects "unsafe use of new value" (DQ #122 root cause). A `result: "fail"` here is **catch-fire** for the advisor — the split approach is wrong and the SL-a-fix-impl-1 brief needs reworking.

The ci-watcher's job is unchanged: poll the run, mutate the entry, exit. Pass/fail interpretation is advisor-side after mutation.

## Action

Per option 2 (PMD #156, locked 2026-04-28): MUTATE the existing `validate-pending` DQ entry in place by matching `workflow_run_id: 25278271449`. Do NOT write a new entry.

Follow the full procedure in `.claude/PRPs/templates/ci-watcher-brief.template.md` — steps 1 through 7 verbatim. Key points:

- Long-poll: `timeout 3600 gh run watch 25278271449 --exit-status --repo barrie-cork/lemmy`
- Disambiguate result via `gh run view 25278271449 --repo barrie-cork/lemmy --json conclusion --jq '.conclusion'` (do NOT trust `--exit-status` exit code per empirical table in `.claude/agents/ci-watcher.md`)
- On `success`: move DQ #126 from `pending[]` to `resolved[]`, set `result: "pass"`, `answered_by: "ci-watcher"`, `resolved_at`. Commit + push to **the worker branch** `junior/role-impl-task-v1-sl-a-fix-impl-1-see-claude-prps-briefs-sl-a-fix-impl-1-md-95`.
- On `failure`: capture `gh run view 25278271449 --log-failed` (last ~200 lines) + `failed_jobs`. Set `result: "fail"`, populate `log_slice` + `failed_jobs`. STAYS in `pending[]`. Commit + push.
- On `cancelled` / `timed_out`: set matching result. STAYS in `pending[]`. Commit + push.

## Hard refusals

See `.claude/agents/ci-watcher.md`. Never cargo, never edit code, never apply auto-fixes, never write a new DQ entry (only mutate existing by `workflow_run_id`), never write deprecated kinds (`validate-result` / `validate-failed`), never write `answered_by: "advisor"` or `"user"`.
