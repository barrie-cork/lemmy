---
phase: v1-RT-r1
role: ci-watcher
task: revalidate
brief_n: revalidate
authored: 2026-05-10
---

# ci-watcher brief — RT-r1 bundled re-validate runs 25640324087 + 25640324089

**Workflow run ids (mutate BOTH — paired):**
- `25640324087` (cargo-validate-workspace) → mutate DQ #200 (in pending[])
- `25640324089` (cargo-validate-migration) → mutate DQ #201 (in pending[])

**Branch:** `junior/revalidate-rt-r1-bundle` (forked from phase-v1-RT-r1 tip with all 4 migrations present)
**Phase task:** 3 (nominally — this re-validates the bundled cohort that failed Task 3 in isolation)
**Phase branch:** `phase-v1-RT-r1` — read DQ from this ref

## Action

Same per-workflow protocol as `sl-d-ci-watcher-1.md`. Mutate-in-place per option 2 (PMD #156, locked 2026-04-28).

**Polls 2 workflow runs in sequence:**

1. Long-poll `gh run watch 25640324087 --exit-status --repo barrie-cork/lemmy` → mutate DQ #200. Pre-poll: in_progress at brief-write 2026-05-10T21:45Z. Expected `result: "pass"` (validating bundled phase-tip workspace check). Commit: `chore(decision-queue): ci-watcher mutated DQ #200 — <pass|fail> cargo-validate-workspace revalidate`.

2. Long-poll `gh run watch 25640324089 --exit-status --repo barrie-cork/lemmy` → mutate DQ #201. Pre-poll: in_progress. Expected `result: "pass"` (validating bundled phase-tip migration round-trip). Commit: `chore(decision-queue): ci-watcher mutated DQ #201 — <pass|fail> cargo-validate-migration revalidate`.

Match by `workflow_run_id`. Push to `phase-v1-RT-r1` after each commit.

## Important note for advisor follow-up

DQ #194 (Task 3 original FAIL on per-task isolation) STAYS in pending[] as historical evidence of the cohort dependency design bug. Do NOT mutate DQ #194; it's part of the audit trail. Cohort barrier for RT-r1 lifts when DQ #200 + #201 both reach `result: "pass"` — DQ #194's terminal `fail` state is acknowledged as a known historical issue.

If DQ #200 OR #201 returns `result: "fail"`, the §G4 classifier runs against the new log_slice. The bundled migration set is the canonical "all 4 migrations together" test; a real failure here would indicate a different bug class than DQ #194.

## Hard refusals

See `.claude/agents/ci-watcher.md`.

## Concurrency note

Single ci-watcher (not parallel) per the resurrection-bug lesson from the 4-way ci-watcher dispatch earlier today. Serial mutation avoids the merge-time race that resurrected DQ #189-#192 + #195-#196.
