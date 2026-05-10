---
phase: v1-RT-r1
role: impl-task
task: 0
brief_n: 0
authored: 2026-05-10
---

# [role:impl-task] v1-RT-r1 task 0 — pre-flight harness audit — see .claude/PRPs/briefs/rt-r1-impl-0.md

## §1 Role + dispatch

`[role:impl-task] v1-RT-r1 task 0 pre-flight harness audit`

## §2 Scope

Run the pre-flight harness audit for v1-RT-r1 per plan §13 Task 0.
Execute Probes 0..10 in order. Report each probe result (PASS / FAIL / WARN).
**No commit at Task 0 — verification only.**

If any of the following probes FAIL with exit 1, stop and file a `kind: "blocker"` DQ entry:
- Probe 0: Docker not running
- Probe 1: not on `phase-v1-RT-r1`
- Probe 2: `ReputationEventSourceType` already present in `enums.rs` (branch contamination)
- Probe 3: `dedupe_key` or `source_event_type` already in `schema.rs` (branch contamination)
- Probe 5: any of DQ #181-#186 not resolved
- Probe 6: 3 v1-AD-a-shipped duplicate keys not found at expected paths (per planner DQ #187 — `deltas.participation_weekly_active`, `participation.dormancy_window_days`, `deltas.participation_dormant`)
- Probe 7: concurrent open PR touches RT-r1 IMPLEMENT files
- Probe 9: any of the 6 PM-plugin hook literals missing (regression of prior phase)
- Probe 10: `ReputationEventId` or `SponsorAllowlistId` newtype missing (regression of prior phase)

All other probe failures: report as WARN (do not stop).

## §3 Required reading

- `.claude/PRPs/plans/v1-reputation-tuning-r1.plan.md` §13 Task 0 (full probe list, Probes 0..10)
- `.claude/rules/pre-phase-harness-audit.md` — R5 audit shape
- `.claude/rules/decision-queue.md` — Recipe 1 (blocker DQ) if a critical probe fails

## §3a Handover from prior cohort

(none — Task 0 is the first task; no prior cohort)

## §4 Constraints

- **No commit.** Task 0 is verification only; no files changed, no git commit.
- **No code authoring.** Do not edit `crates/**`, `migrations/**`, `tests/**`.
- **DQ mid-task push rule:** if a blocker DQ is filed, commit + push the DQ entry immediately per `decision-queue.md` "Mid-task visibility".
- **Attribution:** DQ entries use `from: "impl"`, never `from: "advisor"`.
- **Branch:** must be on `phase-v1-RT-r1` before running probes (Probe 1 verifies this).

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 0 has no e2e test authorship. No `LemmyResult` / `Box<dyn Error>` decision required.

## §5 Concurrency note

Sibling phase `phase-v1-SL-d` is concurrently active under shutter session ownership at fix-impl-1 (Junior task #188 running). RT-r1 Task 0 runs read-only probes; zero conflict with SL-d's writes. Probe 7 explicitly checks for concurrent open PRs touching RT-r1 IMPLEMENT files (none expected).
