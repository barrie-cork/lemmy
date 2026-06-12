# Retro — m2-late-2 (B-publish sanction propagation: live wiring + CR atomicity fix)

**Merged:** `e1d615ee3` — PR #196 → governance-v0 @ `e876929c4`  
**Phase branch:** `phase-m2-late-2` (cut 2026-06-12T10:41Z off `649621dce`, deleted on merge)  
**Scope:** 5 impl tasks (T1–T5) across workspace + bridge. T2 = CR-A fix from m2-late-1 review.  
**Wall-clock:** ~11.5 hours (bm-cut → merge, single session day 2026-06-12)

---

## Advisor

**Signal: orchestration was clean but the session resumed from a /compact boundary mid-run.**

- Phase started with a prior-session handover already in place (`m2-late-2-auto-2026-06-12.md`). State recovered cleanly via auto-state JSON + `brehon-state-status` agent.
- Cross-lane cap (≤2 running) enforced throughout — no contention incidents.
- Phase 2 e2e-2 run1 had 2 transient PG connection-refused failures (testcontainer port collision on 33010/33011). Correctly identified as transient; retry run confirmed 135/135 PASS. e2e-3 skipped as redundant (only a DQ-mutation commit between tips).
- `/brehon-verify` ran cleanly: all 3 §16a stories ✓. Story 2 grep produced a false positive on docstring text `(no \_ =>)` — confirmed manually. ADR-gate (enqueue_sanction_event definition + callsite) confirmed present.
- CR triage: 4 findings, all in `.claude/` meta-files, no code findings. cr-2 (DQ audit-trail) was the only fix-in-pr; corrected inline via throwaway worktree.
- **One process miss:** Python string replacement on cr-2 fix produced a doubled `--test e2e` on first attempt (replaced the wrong substring). Required a second correction pass. Root cause: should verify-then-write, not write-then-verify.
- **Daemon governance-v0 sync issue:** `git merge --ff origin/governance-v0` ran against daemon HEAD=phase branch (not governance-v0), creating a spurious merge commit. Caught and reset immediately. Root cause: daemon was checked out on the phase branch after bm-merge-forward; should use `git branch -f governance-v0 origin/governance-v0` for ref-only updates.

**Carry-forwards:**
- Verify-before-write pattern for DQ Python mutations (double-substitution class).
- Daemon ref-sync: use `git branch -f <ref> origin/<ref>` not `git merge` when HEAD ≠ target branch.

---

## Planning

Planning ran in a prior session (m2-late-2-planning-1). Signals from the plan artifact:

- Plan §13 cohort grouping was correct: T1+T3 [P] (workspace + bridge additive), T2 serial (CR-A atomicity, requires T1), T4 serial (power-levels rewrite, requires T3), T5 serial (tests, requires T4). No cohort contention.
- §16a stories were well-specified — all 3 passed verification without retrofitting.
- ADR-008 atomicity constraint was explicitly load-bearing in Task 2 IMPLEMENT text — enforced correctly.
- CR-3/cr-4 findings (LINUX-BRIDGE prose + R8 prominence) show the plan could be more explicit in task-level text, but the constraints were implemented correctly. No plan-quality regression.

---

## Impl

**Per-task metrics** (`files / commits / runtime-min / max-log-silence-min`):

| Task | Description | Files | Commits | Runtime (min) | Notes |
|---|---|---|---|---|---|
| T1 | `case_id` payload + e2e assertion | 3 | 1 (`a13020989`) | ~37 | T1+T3 parallel; T1 DQ resolved first |
| T2 | CR-A `run_transaction` atomicity | 1 | 1 (`77480af62`) | ~25 | Serial after T1; clean implementation |
| T3 | Bridge `case_id` + `lookup_by_case` | 2 | 1 (`3ed5ac962`) | ~37 | Parallel with T1; bridge Docker validate |
| T4 | Matrix power-level enforcement | 1 | 1 (`210b03923`) | ~30 | Largest task; compute_power_override + GET/PUT |
| T5 | Bridge handler tests | 1 | 1 (`2019b0dbd`) | ~22 | dep-free + 1 #[ignore] live test; no new dev-dep |

**Signals:**
- All 5 tasks shipped clean on first attempt — zero §G4 failures, zero retries.
- T4 rewrite of `handle_sanction_event` was the most complex (multi-stage pipeline: auth check → DB open → room lookup → ensure puppet → power-level GET/merge/PUT). Impl followed the plan §13 pattern correctly.
- T5 correctly avoided new dev-dependencies (used `std::env::temp_dir()` for temp DB path, no `tempfile` crate).
- NO-CARGO-ON-ELITEDESK respected throughout — all cargo ran on laptop via `validate-pending-laptop` DQ pattern.
- R-bridge-lock discipline held: `services/bridge/Cargo.lock` unchanged across all 3 bridge tasks.

**One impl-side audit-trail miss:** T1 DQ `commands[2]` recorded the draft `-p lemmy_server --features full` form rather than the actual `--workspace` form run. Caught by CR (cr-2), fixed post-merge-review. Low impact (resolved DQ entry, no code effect).

---

## BM

**Verbs run:** bm-cut, bm-pr, bm-merge. All executed by Junior Haiku tasks.

| Verb | Task # | Wall-clock | Notes |
|---|---|---|---|
| bm-cut | (prior session) | — | Clean; phase branch off `649621dce` |
| bm-pr | #668 | ~2 min | Merge-forward applied (18 gov-v0 commits); PR #196 opened correctly |
| bm-merge | #669 | ~2 min | `--admin` bypass for CodeRabbit UNSTABLE (known pattern); `--merge` no-squash; branch deleted |

**Signals:**
- bm-pr correctly assembled the PR body from plan §13 task table + commit log (~32 commits).
- bm-merge runlog entry correctly cited the merge SHA (`e1d615ee3`).
- No BM file-ownership violations.
- Copilot hit quota limit on PR #196 — 0 Copilot findings (expected, non-blocking).

---

## Lessons

No new lessons needed — all findings were covered by existing lesson corpus:
- cr-2 (DQ audit-trail): existing `feedback_validate_pending_laptop_must_use_wrapper.md` covers the -p vs --workspace class; the miss was in the DQ write, not the run.
- Advisor Python double-substitution: carry-forward to next phase as a verbal note (not lesson-worthy on first occurrence).
- Daemon ref-sync: covered by existing `feedback_finalize_merge_where_to_look_first.md` (daemon-local-first discipline); the `-f` branch update form is a new nuance worth a lesson if it recurs.

---

## Summary

m2-late-2 shipped cleanly in ~11.5 hours. All 5 tasks delivered and validated. 135/135 e2e PASS. No §G4 failures. CR had 4 meta-file findings (0 code findings); 1 minor fix applied. ADR-008 (atomicity) and ADR-015 (pseudonymity) gates both confirmed by /brehon-verify. The Matrix bridge now applies power-level enforcement per sanction kind across all provisioned rooms for a case.
