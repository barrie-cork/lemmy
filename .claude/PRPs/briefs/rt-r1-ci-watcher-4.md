---
phase: v1-RT-r1
role: ci-watcher
task: 4
brief_n: 4
authored: 2026-05-10
---

# ci-watcher brief — RT-r1 Task 4 workflow runs 25636769989 + 25636769981

**Workflow run ids (mutate BOTH — paired):**
- `25636769989` (cargo-validate-workspace) → mutate DQ #195 (in pending[])
- `25636769981` (cargo-validate-migration) → mutate DQ #196 (in pending[])

**Branch:** `junior/role-impl-task-v1-rt-r1-task-4-seed-26-net-new-reputation-tuning-config-keys-per-dq-187-see-claude-prps-briefs-rt-211`
**Phase task:** 4
**Phase branch:** `phase-v1-RT-r1` — read DQ from this ref

## Action

Same per-workflow protocol as `sl-d-ci-watcher-1.md`.

**Polls 2 workflow runs:**

1. Long-poll `gh run watch 25636769989 --exit-status --repo barrie-cork/lemmy` → mutate DQ #195. Pre-poll snapshot: `conclusion: "success"`.
2. Long-poll `gh run watch 25636769981 --exit-status --repo barrie-cork/lemmy` → mutate DQ #196. Pre-poll snapshot: `conclusion: "success"`.

Match by `workflow_run_id`. Both expected `result: "pass"`. Commit each separately:
- `chore(decision-queue): ci-watcher mutated DQ #195 — pass cargo-validate-workspace task 4`
- `chore(decision-queue): ci-watcher mutated DQ #196 — pass cargo-validate-migration task 4`

Push to `phase-v1-RT-r1`.

## Hard refusals

See `.claude/agents/ci-watcher.md`.

## Concurrency note

Cohort A 4-way ci-watcher dispatch. ci-watcher #4 mutates DQ #195+#196 only.
