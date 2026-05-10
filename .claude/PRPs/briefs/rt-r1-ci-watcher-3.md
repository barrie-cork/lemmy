---
phase: v1-RT-r1
role: ci-watcher
task: 3
brief_n: 3
authored: 2026-05-10
---

# ci-watcher brief — RT-r1 Task 3 workflow runs 25636774572 + 25636774556 (one FAIL expected)

**Workflow run ids (mutate BOTH — paired):**
- `25636774572` (cargo-validate-workspace) → mutate DQ #193 (in pending[])
- `25636774556` (cargo-validate-migration) → **mutate DQ #194 to `result: "fail"`** (in pending[]; advisor pre-checked conclusion=failure)

**Branch:** `junior/role-impl-task-v1-rt-r1-task-3-backfill-reputation-event-source-event-type-per-dq-184-see-claude-prps-briefs-rt-r1-210`
**Phase task:** 3
**Phase branch:** `phase-v1-RT-r1` — read DQ from this ref

## Action

Same per-workflow protocol as `sl-d-ci-watcher-1.md`. Mutate-in-place per option 2 (PMD #156, locked 2026-04-28).

**Polls 2 workflow runs:**

1. Long-poll `gh run watch 25636774572 --exit-status --repo barrie-cork/lemmy` → mutate DQ #193. Pre-poll snapshot: `conclusion: "success"`. Expected `result: "pass"`. Commit: `chore(decision-queue): ci-watcher mutated DQ #193 — pass cargo-validate-workspace task 3`.

2. Long-poll `gh run watch 25636774556 --exit-status --repo barrie-cork/lemmy` → mutate DQ #194. Pre-poll snapshot: `conclusion: "failure"`. Expected `result: "fail"`. Capture `gh run view 25636774556 --log-failed` (last ~200 lines) into `log_slice`, capture `failed_jobs` array via `--json jobs --jq '[.jobs[] | select(.conclusion == "failure") | .name]'`. Entry STAYS in `pending[]` for advisor §G4 classifier triage. Commit: `chore(decision-queue): ci-watcher mutated DQ #194 — fail cargo-validate-migration task 3 (down.sql round-trip likely)`.

Match by `workflow_run_id`, NOT id.

Push to `phase-v1-RT-r1` after each commit.

## Important note for advisor follow-up

The advisor will run §G4 classifier on DQ #194 after this ci-watcher exits. The likely failure mode is the down.sql round-trip — Task 3's down.sql performs `UPDATE reputation_event SET source_event_type = 'Endorsement' WHERE source_event_type IN ('SponsorLiability', 'JuryVote', 'FounderSeed')`, which is fine on an empty seeded DB but may have a different problem (transaction boundary, ON CONFLICT semantics, missing cast). Advisor will read `log_slice` and decide allowlist match vs catch-fire.

## Hard refusals

See `.claude/agents/ci-watcher.md`.

## Concurrency note

Cohort A 4-way ci-watcher dispatch. ci-watcher #3 mutates DQ #193+#194 only.
