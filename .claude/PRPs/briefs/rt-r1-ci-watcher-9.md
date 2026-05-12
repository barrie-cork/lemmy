---
phase: v1-RT-r1
role: ci-watcher
task: fix-impl-4
brief_n: 9
authored: 2026-05-12
parent_dq: 211
---

# ci-watcher brief — RT-r1 fix-impl-4 workspace run 25740789485

**Workflow run id:** `25740789485` (cargo-validate-workspace)
**Branch:** `junior/role-impl-task-rt-r1-fix-impl-4-applyat-semantics-add-onrestart-variant-fix-10-configkeymetadata-entries-see-c-239`
**Phase task:** `fix-impl-4`
**Phase branch:** `phase-v1-RT-r1`
**DQ entry to mutate:** `#211` (kind: `validate-pending`, in `pending[]`)

## Action

Standard ci-watcher mutation per option 2 (PMD #156, locked 2026-04-28).

1. Long-poll `gh run watch 25740789485 --exit-status --repo barrie-cork/lemmy`.
2. Locate DQ #211 in `.claude/decision-queue.json` by matching `workflow_run_id: 25740789485`.
3. Mutate in place: populate `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`.
4. On `result: "pass"` — move entry from `pending[]` to `resolved[]`.
5. On `result: "fail" | "cancelled" | "timed_out"` — leave in `pending[]` (advisor §G4 classifier handles).

**Commit mutation:**
```
chore(decision-queue): ci-watcher mutated DQ #211 — <pass|fail> fix-impl-4 workspace run 25740789485
```

**Push to:** `phase-v1-RT-r1` (DQ #211 is on the phase branch tip `ae0d0cee5`).

## Read first

- `.claude/agents/ci-watcher.md` — your subagent contract.
- `.claude/rules/decision-queue.md` — mutation pattern §"ci-watcher mutation pattern".
- `.claude/decision-queue.json` on phase-v1-RT-r1 — locate DQ #211 in `pending[]`.
