---
phase: v1-SL-e
role: ci-watcher
task: task-1
brief_n: 1
authored: 2026-05-12
parent_dq: 193
---

# ci-watcher brief — SL-e task 1 workspace run 25737219926

**Workflow run id:** `25737219926` (cargo-validate-workspace)
**Branch:** `junior/role-impl-task-v1-sl-e-task-1-e2e-test-1-revocation-during-window-escapes-open-mod-v1-sl-e-fixtures-see-claude-236`
**Phase task:** `1`
**Phase branch:** `phase-v1-SL-e`
**DQ entry to mutate:** `#193` (kind: `validate-pending`, in `pending[]`)

## Action

Standard ci-watcher mutation per option 2 (PMD #156, locked 2026-04-28).

1. Long-poll `gh run watch 25737219926 --exit-status --repo barrie-cork/lemmy`.
2. Locate DQ #193 in `.claude/decision-queue.json` by matching `workflow_run_id: 25737219926`.
3. Mutate in place: populate `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`.
4. On `result: "pass"` — move entry from `pending[]` to `resolved[]`.
5. On `result: "fail" | "cancelled" | "timed_out"` — leave in `pending[]` (advisor §G4 classifier handles).

**Commit mutation:**
```
chore(decision-queue): ci-watcher mutated DQ #193 — <pass|fail> task-1 workspace run 25737219926
```

**Push to:** `phase-v1-SL-e` (DQ #193 is on the phase branch tip `e93a54c5d`).

## Read first

- `.claude/agents/ci-watcher.md` — your subagent contract.
- `.claude/rules/decision-queue.md` — mutation pattern §"ci-watcher mutation pattern".
- `.claude/decision-queue.json` on `phase-v1-SL-e` — locate DQ #193 in `pending[]`.
