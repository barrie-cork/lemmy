---
phase: v1-RT-r1
role: ci-watcher
task: tasks-8-9-10-bundle
brief_n: 7
authored: 2026-05-12
parent_dq: 209
---

# ci-watcher brief — RT-r1 tasks 8+9+10 bundle workflow run 25734652725

**Workflow run id:** `25734652725` (cargo-validate-workspace)
**Branch:** `junior/role-impl-task-v1-rt-r1-tasks-8-9-10-bundle-reputationeventsourcetype-enum-governance-log-shims-e2e-probe-see-232`
**Phase task:** `8+9+10`
**Phase branch:** `phase-v1-RT-r1`
**DQ entry to mutate:** `#209` (kind: `validate-pending`, in `pending[]`)

## Action

Standard ci-watcher mutation per option 2 (PMD #156, locked 2026-04-28).

1. Long-poll `gh run watch 25734652725 --exit-status --repo barrie-cork/lemmy`.
2. Locate DQ #209 in `.claude/decision-queue.json` by matching `workflow_run_id: 25734652725`.
3. Mutate in place: populate `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`.
4. On `result: "pass"` — move entry from `pending[]` to `resolved[]`.
5. On `result: "fail" | "cancelled" | "timed_out"` — leave in `pending[]` (advisor §G4 classifier handles).

**Commit mutation:**
```
chore(decision-queue): ci-watcher mutated DQ #209 — <pass|fail> tasks-8-9-10-bundle workspace run 25734652725
```

**Push to:** `phase-v1-RT-r1` (DQ #209 is on the phase branch tip `0117d7381`).

## Read first

- `.claude/agents/ci-watcher.md` — your subagent contract.
- `.claude/rules/decision-queue.md` — mutation pattern §"ci-watcher mutation pattern".
- `.claude/decision-queue.json` on `phase-v1-RT-r1` — locate DQ #209 in `pending[]`.
