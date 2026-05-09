# [role:ci-watcher] sl-c-2 DQ #170 — workspace-check for fix-impl-3 (Case A type restore)

## 1. Role + dispatch

`[role:ci-watcher]` — poll `cargo-validate-workspace.yml` workflow run
`25610290969` and mutate DQ #170 in `.claude/decision-queue.json`.

## 2. Entry details

```yaml
dq_id: 170
workflow_run_id: 25610290969
repo: barrie-cork/lemmy
branch: junior/role-impl-task-sl-c-2-fix-impl-3-case-a-type-shape-fix-for-task-2-see-claude-prps-briefs-sl-c-2-fix-impl-3-md-166
phase_task: v1-SL-c-2-fix-impl-3
```

## 3. Procedure

Per `.claude/agents/ci-watcher.md`:

1. Poll: `gh run watch 25610290969 --repo barrie-cork/lemmy --exit-status`
2. On exit:
   - exit 0 → `result: "pass"`, move entry from `pending[]` to `resolved[]`
   - exit non-zero → `result: "fail"` (or `"cancelled"` / `"timed_out"`), leave in `pending[]`
3. Populate `answered_by: "ci-watcher"`, `resolved_at: <now>`, `log_slice` (last ~200 lines on fail, null on pass), `failed_jobs` (array on fail, null on pass).
4. Mutate DQ entry in place by matching `workflow_run_id: 25610290969`.
5. Commit + push to the worker branch:
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): ci-watcher mutated DQ #170 — <pass|fail> workspace-check fix-impl-3"
   git push origin HEAD
   ```

## 4. Constraints

- Mutate DQ #170 only (match by `workflow_run_id: 25610290969`).
- Do NOT write a new DQ entry.
- Do NOT run cargo, edit `crates/`, or invoke any impl tools.
- Worker branch for this task is the fix-impl-3 branch above.
