# impl-task Brief — v1-deps-r1 Task 0: Pre-flight harness audit

**Role:** [role:impl-task]
**Phase:** v1-deps-r1
**Task:** 0 (pre-flight — no commit)
**Branch:** phase-v1-deps-r1
**Authored:** 2026-05-24

---

## 1. Role + dispatch line

[role:impl-task] v1-deps-r1 task 0 pre-flight harness audit — see .claude/PRPs/briefs/v1-deps-r1-impl-0.md

---

## 2. Scope

Run the 12 probes (Probes 0–11) from plan §13 Task 0 exactly as written. **No commit.** Verification only.

Output a structured report of each probe's result (exit code + EXPECT match / DRIFT). If any probe fails its EXPECT:
- Probes 0–4, 6–7, 9–11: surface as `kind: "blocker"` DQ and stop.
- Probe 5: MUST be non-zero — if exit 0, surface as `kind: "blocker"` DQ and stop.
- Probe 8 counts drift from 42 / 30 / 38 / 8 / 0: surface as `kind: "blocker"` DQ and stop.
- Probe 6 (clippy baseline) non-zero: note "insert chore(lint) task before T1" in task output, raise `kind: "blocker"` DQ.

---

## 3. Required reading

- `.claude/PRPs/plans/v1-deps-r1.plan.md` §13 Task 0 (full probe list, lines 706–814)
- `.claude/rules/pre-phase-harness-audit.md` (R5: enumerate ALL probes explicitly)
- `.claude/lessons/feedback_validate_pending_laptop_must_use_wrapper.md`
- `.claude/lessons/feedback_windows_e2e_requires_bat_wrapper.md`

### 3a. Handover from prior cohort

(none — first cohort; planning task #452 completed at 76a1eebc5)

---

## 4. Constraints

- **No commit at Task 0** — verification only; no files created or modified.
- All cargo invocations MUST use `cmd //c "scripts\\brehon\\cargo-*.bat ..."` wrapper form (Windows libpq.dll discipline).
- Write rg output to files before `wc -l` (Windows pipe + wc unreliability per `feedback_pipes_mask_exit_codes.md`).
- Use `/tmp/probe*.txt` for intermediate files (or `$LOCALAPPDATA/Temp/` if `/tmp` unavailable on EliteDesk — check at task-time).
- If a `kind: "blocker"` DQ is raised, commit + push it immediately per `decision-queue.md` "Mid-task visibility".
- Do NOT proceed to Task 1 dispatch — Task 0 is verification only; the advisor queues Task 1 after reviewing Task 0 output.
