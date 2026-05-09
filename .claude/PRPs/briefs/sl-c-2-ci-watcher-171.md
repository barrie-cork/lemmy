# [role:ci-watcher] sl-c-2 DQ #171 — Phase-2 e2e for phase-v1-SL-c-2 tip df1d419ef

## 1. Role + dispatch

`[role:ci-watcher]` — poll `cargo-test-e2e` workflow run
`25611232282` and mutate DQ #171 in `.claude/decision-queue.json`.

## 2. Entry details

```yaml
dq_id: 171
workflow_run_id: 25611232282
repo: barrie-cork/lemmy
branch: phase-v1-SL-c-2
phase_task: 2
```

## 3. Procedure

Per `.claude/agents/ci-watcher.md`:

1. Poll: `gh run watch 25611232282 --repo barrie-cork/lemmy --exit-status`
2. On exit:
   - exit 0 → `result: "pass"`, move entry from `pending[]` to `resolved[]`
   - exit non-zero → `result: "fail"` (or `"cancelled"` / `"timed_out"`), leave in `pending[]`
3. Populate `answered_by: "ci-watcher"`, `resolved_at: <now>`, `log_slice` (last ~200 lines on fail, null on pass), `failed_jobs` (array on fail, null on pass).
4. Mutate DQ entry in place by matching `workflow_run_id: 25611232282`.
5. Commit + push to the **governance-v0** branch (not a worker branch — this is a phase-level e2e entry raised by advisor):
   ```
   git add .claude/decision-queue.json
   git commit -m "chore(decision-queue): ci-watcher mutated DQ #171 — <pass|fail> Phase-2 e2e phase-v1-SL-c-2"
   git push origin governance-v0
   ```

## 4. Constraints

- Mutate DQ #171 only (match by `workflow_run_id: 25611232282`).
- Do NOT write a new DQ entry.
- Do NOT run cargo, edit `crates/`, or invoke any impl tools.
- This is a Phase-2 e2e run (~26 min); the 60-min ci-watcher timeout cap applies.
- The DQ entry is on `governance-v0`, not a junior worktree branch.
