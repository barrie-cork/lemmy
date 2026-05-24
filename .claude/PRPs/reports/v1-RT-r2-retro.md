# Retro — v1-RT-r2 (per-dimension chained-halving decay + bounds clamp behind feature flag)

**Sub-phase:** v1-RT-r2 — chained-halving decay per-(dimension, direction) + per-dimension bounds clamping, gated by `feature.reputation_v1_decay_enabled`
**Branch:** `phase-v1-RT-r2` — cut from governance-v0 (bm-cut `8768b7757`), not yet merged
**Deliverables shipped:** `compute_applied_delta` v1 rewrite + `chained_halve` helper (Task 1); `recompute_snapshot` flag wiring + `resolve_half_life_for_event` + `clamp_dimension_i32` + 11 unit tests (Task 2)
**This is a RETRO, not a completion report** — it records what the four roles did well, where they drifted, and what the next sub-phase should carry.

---

## 0. Outcome

| Dimension | Result |
|---|---|
| Goal achieved | **Yes.** `feature.reputation_v1_decay_enabled = false` preserves v0 behaviour verbatim. Flag=true path: per-(dim,direction) half-life resolver, chained-halving decay, per-dimension bounds clamp — all implemented and unit-tested. |
| Tests | **Pass.** 15 unit tests (11 new + 4 existing updated); all pass. e2e 108 passed, 0 failed — v0 path unchanged at default flag=false. |
| Clean execution | **Partial — recovered.** Task 2 worker ended without committing (e2e until-loop still polling when 65-min session ended). Advisor recovered by reading worker worktree diff via SSH, copying impl to laptop lane, running all 4 validate commands locally. No impl was lost; recovery added ~30 min overhead. |

---

## 1. The arc (what actually happened)

1. **Phase cut + planning (2026-05-23):** `bm-cut` created `phase-v1-RT-r2` (`8768b7757`). Planning brief authored on governance-v0; clarify DQs `a3d0e9941441-009/010` self-answered by advisor. Planning Junior shipped plan `v1-RT-r2.plan.md`.
2. **Task 0 — pre-flight audit:** Advisor ran probes 0–9 directly on laptop lane; all passed first-try. No commit (correct per plan — pre-flight produces no deliverable).
3. **Task 1 — compute_applied_delta rewrite:** Junior worker dispatched to EliteDesk; committed `c963c9066` (with HANDOVER trailer). validate-pending-laptop DQ `8aca794fb044-001` written; advisor-laptop mutated result:pass. Daemon-merge reconcile commit `8896ad5d4` merged the finalize-merge history into the lane; DQ re-applied at `8dd64f254`.
4. **Task 2 — flag wiring + resolver + clamp + tests:** Junior worker dispatched; implemented all 7 steps. Worker ended without committing because the brief instructed running e2e as an EliteDesk until-loop — the loop was still polling when the 65-min session ended. Advisor recovered impl by reading worker worktree diff via SSH (`/srv/brehon-fork/.junior/worktrees/job-435/`), copied `reputation_snapshot.rs` to lane, ran all 4 validate commands on laptop: check ✅ / clippy ✅ / unit-tests ✅ (15 passed) / e2e ✅ (108 passed, 2267s). Committed `1bdd51280`, DQ `b246616aaf8f-001` written and mutated result:pass at `a92efc5bc`.

---

## 2. Per-role signals

### 2.1 Advisor

**Did well:**
- **Worker impl recovery was complete and lossless.** After the worker ended without committing, reading the worker worktree diff via SSH produced a clean picture of all 7 steps. Copying to the laptop lane + running all 4 validate commands produced the same result as if the worker had committed.
- **validate-pending-laptop-e2e gate held.** DQ written and mutated correctly; all 4 commands captured in DQ `commands` array; result:pass mutation committed + pushed before declaring task done.
- **E2e correctly routed to laptop.** The brief's §4 Constraints specified e2e should run on laptop; the recovery followed this. 108 e2e tests passed in 37.8 min on laptop — consistent with the ~26-min typical noted for the `validate-pending-laptop` handler.

**Drifted:**
- **Brief §4 Constraints did not prohibit e2e on the EliteDesk explicitly.** The brief said to write a `validate-pending-laptop-e2e` DQ entry after commands 1-3 pass, but did not include the explicit guard "do NOT run e2e on EliteDesk — write DQ after cargo-check/clippy/unit-tests and stop." The worker interpreted the until-loop pattern from prior briefs and ran e2e on the EliteDesk daemon. → **§3 action 1.**
- **Daemon-merge reconcile overhead.** The finalize-merge commit from Task 1 introduced a merge-history divergence requiring a reconcile commit (`8896ad5d4`) and DQ re-application (`8dd64f254`) — 2 extra commits to clean up what should have been a single Task 1 commit chain. This is a multi-lane worktree artefact; not new but worth tracking in the complexity score.

### 2.2 Planning

**Did well:**
- **Plan §10 code blocks were verbatim and correct.** The 7 implementation steps were specified with exact Rust code blocks; the worker followed them without deviation. All `get_int`/`get_bool` patterns, the `resolve_half_life_for_event` key format string, and the `clamp_dimension_i32` i64-widen-then-narrow pattern were correct as specified.
- **Serial task dependency (Task 1 → Task 2) correctly encoded** in FILES YAML `requires:` field. No cohort-dispatch attempt on overlapping files.
- **Test shape specified correctly.** Plan §13 Task 2 IMPLEMENT step 6 lists all 11 tests with exact expected values; impl produced exactly those tests passing.

**Drifted:**
- **Brief §3 Required reading cited `feedback_clippy_test_style.md`** but `clamp_dimension_i32` used `unwrap_or({...})` closure form initially — the plan §10.5 note says `unwrap_or_else` is allowed. Brief noted the distinction correctly; no clippy failure occurred. Minor — no action needed.

### 2.3 Impl

**Did well:**
- **All 7 steps implemented correctly first-try.** cargo-check, clippy, unit-tests all exited 0 on the laptop copy with no fixes needed. The `resolve_half_life_for_event` borrow discipline (pass `cache` and `conn` by `&mut` reborrow — plan GOTCHA) was followed correctly.
- **11 new tests match plan spec exactly.** Values match: `chained_halving_at_2x_half_life_quarters_delta` → 25, `chained_halve_helper_saturates_at_very_old_event` → 0, all `clamp_dimension_i32_*` variants correct.
- **Module docstring inserted correctly.** The `//! ## v1 feature flag (RT-r2)` block matches the verbatim plan text.

**Drifted:**
- **Worker ended without committing.** The worker ran e2e as a until-loop on the EliteDesk, which is the wrong execution venue for e2e (laptop is canonical per `feedback_laptop_default_for_validate_pending.md`). The 65-min session ended while the loop was still running. This is a brief-authoring gap (§3 action 1), not an impl failure — the impl itself was correct; only the commit step was missed.

### 2.4 BM

BM role not yet executed for this sub-phase (bm-pr / bm-triage / bm-merge pending after retro). No signals to record.

---

## 3. Actions for the next sub-phase

### Action 1 — Brief §4 Constraints: explicit e2e prohibition on EliteDesk (carry-forward)

**Problem:** Task 2 brief's §4 did not contain the guard: "do NOT run e2e on EliteDesk; write `kind: validate-pending-laptop-e2e` DQ entry after cargo-check/clippy/unit-tests pass and stop." Worker interpreted the until-loop pattern from prior briefs.

**Fix:** All future `impl-task` briefs whose DoD includes an e2e command MUST include this line in §4 Constraints:

> e2e runs on **laptop only** — after cargo-check/clippy/unit-tests pass, write `kind: "validate-pending-laptop-e2e"` DQ entry (commands array = all 4 validate commands, branch, phase_task) and **stop**. Do NOT run e2e on the EliteDesk worker; the laptop advisor session runs it and mutates the DQ entry.

Carry-forward to: v1-RT-r3 impl briefs + the impl-task-brief template at `.claude/PRPs/templates/impl-task-brief.template.md`.

### Action 2 — Complexity score (per `feedback_retro_task_complexity_score.md`)

| Task | Files | Commits | Runtime (min) | Max log silence (min) |
|---|---|---|---|---|
| Task 0 (pre-flight) | 0 | 0 | ~5 | 0 |
| Task 1 (compute_applied_delta) | 1 | 1 (+2 advisor reconcile) | ~30 | ~10 |
| Task 2 (flag + resolver + clamp + tests) | 1 | 1 (+1 DQ mutation) | ~70 (incl. recovery) | ~65 (worker until-loop) |

---

## 4. What surprised us

- **Worker impl was complete and correct despite not committing.** All 7 steps, all 11 tests — the git diff of the uncommitted working tree was a clean, complete implementation. The only gap was the commit step. This means the impl agent's code quality held even when the session management failed.
- **E2e ran 37.8 min on the laptop (2267s).** Slightly longer than the ~26-min typical — possibly due to the High Performance power plan being set for the first time on 2026-05-22; baseline may shift going forward.

## 5. What to change

- Brief §4 Constraints for all impl-task briefs with e2e in their DoD: add the explicit e2e-on-laptop-only guard (§3 action 1). Update `impl-task-brief.template.md`.
- Consider adding `validate-pending-laptop-e2e` guard to the `feedback_laptop_default_for_validate_pending.md` lesson to make it mechanically cite the brief-authoring constraint.

## 6. What to carry forward

- **e2e testing happens on laptop only.** The EliteDesk/Junior worker must not run e2e; it writes the DQ entry and stops. This is a firm constraint for all future `impl-task` briefs whose DoD includes e2e. See §3 action 1.
- **Worker impl recovery recipe (confirmed working):** when a worker ends without committing, `git diff HEAD` in the worker worktree (`/srv/brehon-fork/.junior/worktrees/job-N/`) recovers the full uncommitted diff. Copy the target file to the lane, run all 4 validate commands locally, commit from the laptop lane. Lossless if the session ended mid-loop (not mid-edit).
- **Daemon-merge reconcile overhead is predictable:** Task 1 finalize-merge → reconcile commit → DQ re-application = 2 extra commits. Budget this in complexity scores for serial tasks.
