---
phase: v1-SL-e
role: ci-watcher
task: 3
brief_n: 3
authored: 2026-05-12
parent_dq: 198
---

# ci-watcher brief — SL-e task 3 workspace run 25761876447

**Workflow run id:** `25761876447` (cargo-validate-workspace)
**Branch:** `junior/role-impl-task-v1-sl-e-task-3-e2e-test-3-backfill-of-mid-flight-v0-v1-deploy-close-mod-v1-sl-e-fixtures-see-251`
**Phase task:** `3`
**Phase branch:** `phase-v1-SL-e`
**DQ entry to mutate:** `#198` (kind: `validate-pending`, in `pending[]`)

## Action

Standard ci-watcher mutation per option 2 (PMD #156, locked 2026-04-28).

1. Long-poll `gh run watch 25761876447 --exit-status --repo barrie-cork/lemmy`.
2. Locate DQ #198 in `.claude/decision-queue.json` by matching `workflow_run_id: 25761876447`.
3. Mutate in place: populate `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`.
4. On `result: "pass"` — move entry from `pending[]` to `resolved[]`.
5. On `result: "fail" | "cancelled" | "timed_out"` — leave in `pending[]` (advisor §G4 classifier handles).

**Commit mutation:**
```
chore(decision-queue): ci-watcher mutated DQ #198 — <pass|fail> sl-e task 3 workspace run 25761876447
```

**Push to:** `phase-v1-SL-e` (DQ #198 is on the phase branch tip `b864c91fa`).

## Read first

- `.claude/agents/ci-watcher.md` — your subagent contract.
- `.claude/rules/decision-queue.md` — mutation pattern §"ci-watcher mutation pattern".
- `.claude/decision-queue.json` on phase-v1-SL-e — locate DQ #198 in `pending[]`.
