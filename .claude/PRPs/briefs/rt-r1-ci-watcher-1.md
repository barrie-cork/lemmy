---
phase: v1-RT-r1
role: ci-watcher
task: 1
brief_n: 1
authored: 2026-05-10
---

# ci-watcher brief — RT-r1 Task 1 workflow runs 25636750748 + 25636750747

**Workflow run ids (mutate BOTH — paired):**
- `25636750748` (cargo-validate-workspace) → mutate DQ #191 (in pending[])
- `25636750747` (cargo-validate-migration) → mutate DQ #192 (in pending[])

**Branch:** `junior/role-impl-task-v1-rt-r1-task-1-add-reputation-event-v1-columns-source-event-type-enum-see-claude-prps-briefs-rt-r-208`
**Phase task:** 1
**Phase branch:** `phase-v1-RT-r1` — read DQ from this ref (per `scripts/brehon/resolve-dq-canonical.sh` semantics; trunk's reverted at `c9de27dea`)

## Action

Same per-workflow protocol as `sl-d-ci-watcher-1.md` — mutate-in-place per option 2 (PMD #156, locked 2026-04-28).

**This brief polls 2 workflow runs in sequence:**

1. Long-poll `gh run watch 25636750748 --exit-status --repo barrie-cork/lemmy` → mutate DQ #191. Pre-poll snapshot: `conclusion: "success"` (verified by advisor at brief-write 2026-05-10T19:00Z) — long-poll should return immediately.
2. Long-poll `gh run watch 25636750747 --exit-status --repo barrie-cork/lemmy` → mutate DQ #192. Pre-poll snapshot: `conclusion: "success"`.

For each: locate paired entry in `.claude/decision-queue.json` by matching `workflow_run_id` (NOT by id — the renumber resolution put them in unconventional order). Mutate `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`. Move from `pending[]` to `resolved[]` ONLY on `result: "pass"`.

Both expected to mutate to `result: "pass"` and move to `resolved[]`. Commit each mutation as a separate git commit:
- `chore(decision-queue): ci-watcher mutated DQ #191 — pass cargo-validate-workspace task 1`
- `chore(decision-queue): ci-watcher mutated DQ #192 — pass cargo-validate-migration task 1`

Push to `phase-v1-RT-r1` (the worker branch — daemon will finalize-merge).

## Hard refusals (cite — do not duplicate body)

See `.claude/agents/ci-watcher.md` "Hard refusals". Same as sl-d-ci-watcher-1.md.

## Concurrency note

Cohort A 4-way ci-watcher dispatch. ci-watcher #1 mutates DQ #191+#192 only. ci-watcher #2 mutates DQ #189+#190. ci-watcher #3 mutates DQ #193+#194 (one is FAIL — see brief). ci-watcher #4 mutates DQ #195+#196.
