---
role: impl-task
plan_task: 0
phase: v1-rt-r3-followup
created: 2026-05-29
related_dq: null
---

# Brief — v1-rt-r3-followup Task 0 — pre-flight harness audit + branch verification

> **Clarify provenance:** this brief's parent **planning** brief was clarified via `/brehon-clarify` (advisor-mode pass, 3 entries `a3d0e9941441-034/-035/-036`, all advisor-self-resolved). No clarify-DQ gates an impl-task; clarify gates planning only.

## 0. Pre-flight (subagent runs this before reading anything else)

The `impl-task` agent runs the forbidden-window check from `.claude/agents/impl-task.md` "Task-0 pre-flight". **NOTE:** under Shape G suspension (DQ #229, until 2026-06-01), this lane runs **zero cargo on the EliteDesk daemon** — all cargo runs on the advisor laptop. The forbidden-window check is therefore non-binding for THIS task's dispatch (no daemon cargo), but the probes below run cargo via the **laptop** wrapper. Run the probes as written; if a probe's cargo invocation is needed, it executes on whatever host this worker runs on. (Task 0 is verification-only; if any cargo probe cannot run on the daemon, capture the failure and note it — the advisor re-runs §15 cargo on the laptop regardless.)

## 1. Role + dispatch line

`[role:impl-task] v1-rt-r3-followup task 0 — see .claude/PRPs/briefs/v1-rt-r3-followup-impl-0.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 0** from `.claude/PRPs/plans/v1-rt-r3-followup.plan.md` §13 "Task 0: Pre-flight harness audit + branch verification". **Verification-only — NO commit, NO file edits.**

## 2. Scope

**Produce:** the Task 0 EXPECT-block results (probe outputs). **No commit. No file edits.** This task confirms the lane environment is ready before Task 1 touches `crates/server/tests/e2e.rs`.

Run **Probes 0–10 exactly as enumerated in plan §13 Task 0** (lines 355–410 of the plan). The plan's probe list is the canonical contract — execute each, capture each EXPECT, report pass/fail per probe.

**Do NOT** in this task:
- Edit any file (this is a pre-flight audit; the e2e.rs edits are Task 1 + Task 2).
- Commit anything (Task 0 is verification-only per plan §13).
- Run the full §15.4 e2e (that is the advisor-laptop's gate after Task 2; Task 0 only checks `--no-run` compile + clippy baseline).

**Critical probe outcomes (STOP + file `kind: "blocker"` DQ if any fails):**
- **Probe 4** must exit **NON-ZERO** on both lines (negative test — confirms wrapper exit-code propagation; a zero here means the wrapper silently swallows cargo failures, invalidating every other gate).
- **Probe 6** must return `phase-v1-rt-r3-followup`.
- **Probe 8** must return **4** anchor fn lines within ±100 of plan anchors (2485, 11054, 13866, 14059). Drift >100 on any → `kind: "blocker"` DQ (the file was refactored; Task 1/2 anchors need re-verification).
- **Probe 9** must return **empty** (no concurrent open PR touches `crates/server/tests/e2e.rs`). Non-empty → STOP + reconcile (another lane editing the same file = collision risk).
- **Probe 10** must return a date `< 2026-06-01` (Shape G still suspended; the plan §15 assumes validate-pending-laptop-e2e).

## 3. Required reading

In this order:
1. **Plan §13 Task 0** (`.claude/PRPs/plans/v1-rt-r3-followup.plan.md`, the "Probes" block lines 355–410 + the "EXPECT block" lines 412–419) — the canonical probe list.
2. **Plan §7 R-guardrails** (R5: enumerate ALL probes explicitly; no implicit inheritance).
3. `.claude/lessons/feedback_pipes_mask_exit_codes.md` (always — capture-then-tail; never pipe cargo through tail when you need the exit code).
4. `.claude/lessons/feedback_wrapper_script_flag_silence.md` (Probe 4 negative-test rationale — wrappers must fail loud).
5. `.claude/lessons/feedback_features_full_p_crate_incompatible.md` (Probe 1 uses `-p lemmy_utils` WITHOUT `--features full`; Probe 2 uses `--workspace --features full` — never combine `-p <crate>` + `--features full`).

## 3a. Handover from prior cohort

(none — first task)

## 4. Constraints

### Branch + commit discipline
- You start on a Junior worktree off `phase-v1-rt-r3-followup`. **No commit this task** — Task 0 is verification-only.
- Mid-task DQ visibility: if a probe fails and you raise a `pending` entry, **commit + push immediately** to your worktree branch (`git add .claude/decision-queue.json && git commit -m "chore(decision-queue): impl raised DQ <id> — <slug>" && git push origin <branch>`).
- No `answered_by: "advisor"` or `"user"` — self-resolve only as `"impl-self-resolved"`. No `approved_by` (advisor-exclusive).

### DQ schema-v3
Any DQ entry uses `bash scripts/brehon/dq-v3-new-entry.sh` for id + `bash scripts/brehon/dq-v3-append-fragment.sh <fragment> --pending` for append. `approved_by: null`, `approved_at: null`. No `max(all_ids)+1`.

### Plan-cited line numbers may have drifted
Probe 8 IS the drift sentinel. If the 4 anchor fns are not within ±100 of (2485, 11054, 13866, 14059), do NOT proceed to report "ready" — file a `kind: "blocker"` DQ so the advisor re-verifies Task 1/2 anchors before dispatch.

## 5. Validation gates

**No cargo validate-pending entry for Task 0** — this is verification-only, no commit, no push of code. Report the probe results directly in §6. If a probe that runs cargo cannot execute on the daemon (Shape G suspended → daemon has no cargo toolchain configured for this lane), note "Probe N cargo deferred to advisor-laptop" and continue; the advisor re-runs §15 cargo on the laptop. The non-cargo probes (6, 7, 8, 9, 10) MUST run and report on the daemon.

## 6. Expected output (return to advisor)

```
## Task 0 complete — v1-rt-r3-followup pre-flight audit

**Branch:** phase-v1-rt-r3-followup (Probe 6 ✓/✗)
**RT-r3 ship present:** Probe 7 ✓/✗ (996765cae on governance-v0)
**Anchor drift sentinel:** Probe 8 — 4 fns at lines <a>/<b>/<c>/<d> (within ±100 of 2485/11054/13866/14059? ✓/✗)
**Concurrent-PR check:** Probe 9 — <empty / list>
**Shape G suspension:** Probe 10 — <date> (< 2026-06-01? ✓/✗)
**Wrapper sanity:** Probes 1-5 — <pass/fail/deferred-to-laptop per probe>
**Negative test:** Probe 4 — <both non-zero? ✓/✗>
**Verdict:** READY for Task 1 / BLOCKED (DQ <id>)
**No commit** (verification-only per plan §13).
```

Plus any DQ id if you raised one.

## 7. Why this brief differs from the plan

Clean execution of plan §13 Task 0 — no overrides. (Cargo probes may defer to advisor-laptop under Shape G suspension; that is documented in the plan §13 Task 0 / §15.6, not a deviation.)
