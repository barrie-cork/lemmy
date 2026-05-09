# [role:ci-watcher] sl-c-2 DQ #172 — workspace-check for impl-3 (Task 3 no-op test)

## 1. Role + dispatch

`[role:ci-watcher]` — poll `cargo-validate-workspace` workflow run
`25612657137` and mutate DQ #172 in `.claude/decision-queue.json`.

## 2. Entry details

```yaml
dq_id: 172
workflow_run_id: 25612657137
repo: barrie-cork/lemmy
branch: junior/role-impl-task-sl-c-2-impl-3-e2e-test-3-no-op-future-grace-expires-at-see-claude-prps-briefs-sl-c-2-impl-3-md-170
phase_task: "3"
```

## 3. Procedure

Per `.claude/agents/ci-watcher.md`:

1. Poll: `gh run watch 25612657137 --repo barrie-cork/lemmy --exit-status`
2. On exit:
   - exit 0 → `result: "pass"`, move entry from `pending[]` to `resolved[]`
   - exit non-zero → `result: "fail"` (or `"cancelled"` / `"timed_out"`), leave in `pending[]`
3. Populate `answered_by: "ci-watcher"`, `resolved_at: <now>`, `log_slice` (last ~200 lines on fail, null on pass), `failed_jobs` (array on fail, null on pass).
4. Mutate DQ entry in place by matching `workflow_run_id: 25612657137`.
5. Commit + push to the **governance-v0** branch:
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): ci-watcher mutated DQ #172 — <pass|fail> workspace-check impl-3"
   git push origin governance-v0
   ```

## 4. Constraints

- Mutate DQ #172 only (match by `workflow_run_id: 25612657137`).
- Do NOT write a new DQ entry.
- Do NOT run cargo, edit `crates/`, or invoke any impl tools.
- The DQ entry is on `governance-v0`; push there after mutation.
