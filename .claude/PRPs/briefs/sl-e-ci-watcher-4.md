---
phase: v1-SL-e
role: ci-watcher
task: fix-impl-1
brief_n: 4
authored: 2026-05-13
parent_dq: 213
---

# ci-watcher brief — SL-e fix-impl-1 workspace run 25780317086

**Workflow run id:** `25780317086` (cargo-validate-workspace)
**Branch:** `junior/role-impl-task-v1-sl-e-fix-impl-1-see-claude-prps-briefs-sl-e-fix-impl-1-md-257`
**Phase task:** `fix-impl-1` (string — not a §13 task; this is a CR-triage fix task)
**Phase branch:** `phase-v1-SL-e`
**DQ entry to mutate:** `#213` (kind: `validate-pending`, in `pending[]` on the worker branch tip `e2063a7a1`)

## Action

Standard ci-watcher mutation per option 2 (PMD #156, locked 2026-04-28).

1. Long-poll `gh run watch 25780317086 --exit-status --repo barrie-cork/lemmy`.
2. Locate DQ #213 in `.claude/decision-queue.json` on the worker branch (the daemon hasn't finalize-merged into phase-v1-SL-e yet when this brief was authored, but may have by the time you start). Match `workflow_run_id: 25780317086`. If the entry is missing from the snapshot, the daemon has finalize-merged in the meantime — search `phase-v1-SL-e` instead.
3. Mutate in place: populate `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`.
4. On `result: "pass"` — move entry from `pending[]` to `resolved[]`.
5. On `result: "fail" | "cancelled" | "timed_out"` — leave in `pending[]` (advisor §G4 classifier handles).

**Commit mutation:**
```
chore(decision-queue): ci-watcher mutated DQ #213 — <pass|fail> sl-e fix-impl-1 workspace run 25780317086
```

**Push to:** the branch where the DQ entry lives (worker branch if pre-finalize-merge; `phase-v1-SL-e` if post-finalize-merge). Re-read the branch state at mutation time, do not assume.

## Read first

- `.claude/agents/ci-watcher.md` — your subagent contract.
- `.claude/rules/decision-queue.md` — mutation pattern §"ci-watcher mutation pattern".
- `.claude/decision-queue.json` on the live branch — locate DQ #213 in `pending[]`.
