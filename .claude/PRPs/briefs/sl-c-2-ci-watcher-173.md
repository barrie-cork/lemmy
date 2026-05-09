# [role:ci-watcher] sl-c-2 DQ #173 — workspace-check for impl-4 (Task 4 per-case isolation)

## 1. Role + dispatch

`[role:ci-watcher]` — poll `cargo-validate-workspace` workflow run
`25613695291` and mutate DQ #173 in `.claude/decision-queue.json`.

## 2. Entry details

```yaml
dq_id: 173
workflow_run_id: 25613695291
repo: barrie-cork/lemmy
branch: junior/role-impl-task-sl-c-2-impl-4-e2e-test-4-per-case-isolation-one-bad-case-doesn-t-block-batch-see-claude-prps-br-172
phase_task: "4"
```

## 3. Procedure

Per `.claude/agents/ci-watcher.md`:

1. Poll: `gh run watch 25613695291 --repo barrie-cork/lemmy --exit-status`
2. On exit:
   - exit 0 → `result: "pass"`, move entry from `pending[]` to `resolved[]`
   - exit non-zero → `result: "fail"` (or `"cancelled"` / `"timed_out"`), leave in `pending[]`
3. Populate `answered_by: "ci-watcher"`, `resolved_at: <now>`, `log_slice` (last ~200 lines on fail, null on pass), `failed_jobs` (array on fail, null on pass).
4. Mutate DQ entry in place by matching `workflow_run_id: 25613695291`.
5. Commit + push to the **governance-v0** branch:
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): ci-watcher mutated DQ #173 — <pass|fail> workspace-check impl-4"
   git push origin governance-v0
   ```

## 4. Constraints

- Mutate DQ #173 only (match by `workflow_run_id: 25613695291`).
- Do NOT write a new DQ entry.
- Do NOT run cargo, edit `crates/`, or invoke any impl tools.
- The DQ entry is on `governance-v0`; push there after mutation.
