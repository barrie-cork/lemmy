---
phase: v1-RT-r1
role: ci-watcher
task: 2
brief_n: 2
authored: 2026-05-10
---

# ci-watcher brief — RT-r1 Task 2 workflow runs 25636745850 + 25636745854

**Workflow run ids (mutate BOTH — paired):**
- `25636745850` (cargo-validate-workspace) → mutate DQ #189 (in pending[])
- `25636745854` (cargo-validate-migration) → mutate DQ #190 (in pending[])

**Branch:** `junior/role-impl-task-v1-rt-r1-task-2-extend-sponsor-allowlist-for-r4-admin-readers-see-claude-prps-briefs-rt-r1-impl-2-md-209`
**Phase task:** 2
**Phase branch:** `phase-v1-RT-r1` — read DQ from this ref

## Action

Same per-workflow protocol as `sl-d-ci-watcher-1.md`.

**Polls 2 workflow runs:**

1. Long-poll `gh run watch 25636745850 --exit-status --repo barrie-cork/lemmy` → mutate DQ #189. Pre-poll snapshot: `conclusion: "success"`.
2. Long-poll `gh run watch 25636745854 --exit-status --repo barrie-cork/lemmy` → mutate DQ #190. Pre-poll snapshot: `conclusion: "success"`.

Match by `workflow_run_id`. Both expected `result: "pass"`. Commit each separately:
- `chore(decision-queue): ci-watcher mutated DQ #189 — pass cargo-validate-workspace task 2`
- `chore(decision-queue): ci-watcher mutated DQ #190 — pass cargo-validate-migration task 2`

Push to `phase-v1-RT-r1`.

## Hard refusals

See `.claude/agents/ci-watcher.md`.

## Concurrency note

Cohort A 4-way ci-watcher dispatch. ci-watcher #2 mutates DQ #189+#190 only.
