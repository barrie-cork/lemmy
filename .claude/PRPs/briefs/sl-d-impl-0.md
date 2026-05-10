---
phase: v1-SL-d
role: impl-task
task: 0
brief_n: 0
authored: 2026-05-10
---

# [role:impl-task] v1-SL-d task 0 — pre-flight harness audit — see .claude/PRPs/briefs/sl-d-impl-0.md

## §1 Role + dispatch

`[role:impl-task] v1-SL-d task 0 pre-flight harness audit`

## §2 Scope

Run the pre-flight harness audit for v1-SL-d per plan §13 Task 0.
Execute Probes 0..18 in order. Report each probe result (PASS / FAIL / WARN).
**No commit at Task 0 — verification only.**

If any of the following probes FAIL with exit 1, stop and file a `kind: "blocker"` DQ entry:
- Probe 0: Docker not running
- Probe 1: not on `phase-v1-SL-d`
- Probe 7: SL-c module not found
- Probe 8: JM-c TODO marker missing from `submit_jury_vote.rs`

All other probe failures: report as WARN (do not stop).

## §3 Required reading

- `.claude/PRPs/plans/v1-sponsor-liability-d.plan.md` §13 Task 0 (full probe list, Probes 0..18)
- `.claude/rules/pre-phase-harness-audit.md` — R5 audit shape
- `.claude/rules/decision-queue.md` — Recipe 1 (blocker DQ) if a critical probe fails

## §3a Handover from prior cohort

(none — Task 0 is the first task; no prior cohort)

## §4 Constraints

- **No commit.** Task 0 is verification only; no files changed, no git commit.
- **No code authoring.** Do not edit `crates/**`, `migrations/**`, `tests/**`.
- **DQ mid-task push rule:** if a blocker DQ is filed, commit + push the DQ entry immediately per `decision-queue.md` "Mid-task visibility".
- **Attribution:** DQ entries use `from: "impl"`, never `from: "advisor"`.
- **Branch:** must be on `phase-v1-SL-d` before running probes (Probe 1 verifies this).

## §4.1 CANONICAL CASE OVERRIDE

Not applicable — Task 0 has no e2e test authorship. No `LemmyResult` / `Box<dyn Error>` decision required.
