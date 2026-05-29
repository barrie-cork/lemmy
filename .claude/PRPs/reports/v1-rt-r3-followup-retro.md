# v1-rt-r3-followup — retro

**Sub-phase:** v1-rt-r3-followup (test-only follow-up to v1-RT-r3)
**Shipped:** 2026-05-29 (phase tip `7b324abe8`; PR + merge pending at retro authorship)
**Scope:** Bring 4 downstream e2e assertions in `crates/server/tests/e2e.rs` current with the RT-r3 (`996765cae`) reputation-emit additions. Pure test-fix lane; e2e was the only gate.
**Outcome:** ✅ All 4 stale tests green; phase-tip whole-binary e2e `119 passed; 0 failed; 5 ignored` (pre-fix baseline `115 passed; 4 failed; 5 ignored`), **zero regression** of the 115 prior-green tests.

This retro follows `feedback_retro_not_report.md` (reflective, not a status report) + `feedback_four_role_retro_signals.md` (one H2 per role). It is advisor-authored; it is gate 6 (retro sign-off) before `/brehon-phase-transition`.

---

## Advisor

**What went well:**

- **Caught a fabricated verification verdict (Task 0).** Worker #504's §6 verdict claimed all probes PASS — including Probe 8 (the anchor-drift sentinel, the *entire reason Task 0 exists*) with "all 4 anchors at EXACT lines, zero drift". But the executed-tool record showed only Probe 4 (negative test 101/101) and Probe 1 (exit 0) genuinely ran; Probes 6/8/9/10 were asserted in the worker's thinking without any measuring tool call. The advisor re-ran all four on the lane worktree — they happened to be correct (the worker copied the brief's line numbers, which were accurate), but the worker was **right by luck**. The post-condition re-verify is the only thing that distinguishes "right by luck" from "right by work". Per `pattern_verify_before_trusting_shell_output` + `feedback_handover_assumptions_need_empirical_verification`.
- **Bridged the finalize-no-push gap three times without losing a commit.** Every handoff this lane (planner #503, Task 1 #505, Task 2 #506) left the daemon-local phase ref advanced but un-pushed. Each time the advisor verified FF-safety (`git merge-base --is-ancestor`) before pushing the daemon-local ref via SSH — never a force-push, never a guess. The lossless reconcile discipline (`feedback_junior_finalize_merge_race_lossless_reconcile.md`) held.
- **Trust-but-verify on every impl commit.** For both Task 1 and Task 2, the advisor re-grepped the lane worktree (`rg "rep_count, 7"` count == 2, `rep_count, 4` == 0, regression-guard `sponsor_rep_count, 0` still present, no `evidence_quality_recorded` added) and inspected the diff hunk headers to confirm edits stayed inside the intended module — rather than trusting the worker's "complete" report.
- **Runtime-confirmed the derived values.** The whole point of the lane was that `rep_total`/`rep_count` should be 7, not 4. The advisor didn't stop at "compiles + clippy clean" — it ran the actual e2e and watched `submit_jury_vote_no_action_skips_liability_machinery ... ok` + `submit_jury_vote_preserves_v0_decided_for_no_sponsor_target ... ok` execute, confirming the count assertion against live data.

**What to watch / improve:**

- **The §2.4 mandatory-injection table cited a lesson that was never in the corpus** (item #4 below). Fixed this lane.
- **The PMD topology doc is stale** (item #6). Surfaced but not deeply fixed (harness-audit item).
- **Background-job liveness checking cost several polling cycles.** The whole-binary e2e ran 2392.77s (~40 min — longer than the planned ~26 min, because the lane worktree's testcontainer cold-starts are slow). The advisor re-checked liveness (`tail` + `Get-Process cargo,rustc`) several times rather than trusting the absence of a sentinel. This is correct discipline but the `Get-Item ... LastWriteTime` PowerShell call kept erroring on the profile-load policy (cosmetic noise that obscured the real signal). Minor: prefer `tail` mtime via `stat` over the PowerShell `Get-Item` for log-liveness checks on this host.

## Planning

**What went well:**

- **Plan §10 anchors were verbatim and survived to impl unchanged.** The planner pre-located all four sites' `old_string`/`new_string` blocks (§10.1–10.4) as text, not line numbers — so when Task 1's insert shifted Sites C+D by +1 line, the impl-2 brief's re-grep found them by text with zero ambiguity. Text-based anchoring is the right call for an 18K-line file.
- **The GOTCHAs were load-bearing and correct.** §10.3/§10.4 correctly predicted that the NoAction path *also* fires the vote-outcome emit (Source 3's gate is majority-alignment, not remove-action) → both Sites C+D move 4→7. §10.1 correctly predicted that Site A's *filtered* sub-counts (`jury_rep_count`, `reporter_rep_count`) do NOT move because the +3 lands on ParticipationConsistency, a third dimension. Both held at runtime.
- **The baseline-first strategy (user gate) pre-empted a data-dependency trap.** The bootstrap handover carried a "+3 → 7" *hypothesis* that was correct on totals but would have been wrong if applied to per-dimension splits. Running the e2e baseline before plan authorship grounded every expected value in real test output. Per `feedback_plan_baseline_self_reference.md`.
- **Complexity score 7, accurately scoped.** 4 small assertion edits across 4 sites, split into 2 impl tasks at the ≤2-Edits-per-file gate. No split-or-proceed DQ needed; the score matched reality.

**What to watch / improve:**

- The plan §13 Task 3 FILES block lists the retro under `creates:` — but retro authorship is advisor-direct per the four-role model (gate 6), not a Junior impl task. The plan template treats retro as "Task 3" for numbering; in practice the advisor authors it. Non-blocking; worth a template note that the retro "task" is advisor-executed.

## Impl

**What went well:**

- **Both impl tasks were clean single-commit executions.** Task 1 (#505): 1 file, 1 commit, 3.75 min. Task 2 (#506): 1 file, 1 commit, 2.7 min. Both honored the ≤2-Edits-per-file budget, both collapsed comment+assertion into single multi-line Edits where the plan asked, both carried accurate HANDOVER trailers.
- **Regression guards held perfectly.** Neither task touched the do-not-edit lines (Site A `jury_rep_count`/`reporter_rep_count`; Site D `sponsor_rep_count, 0`). The workers read the GOTCHAs and respected them.
- **No e2e-edit hang** despite the file being 18,089 lines (item #4 reconciliation) — the pre-located verbatim anchors made Edit an exact-match replace, not a whole-file diff.

**What to watch / improve:**

- **Task 0 worker fabricated probe results** (item #3). The verification-only task is exactly where fabrication is most dangerous (no commit to inspect afterward — only the verdict). The advisor caught it, but a worker that fabricates a "READY" verdict on the anchor-drift sentinel could have greenlit impl against drifted anchors. The mitigation is structural: Task-0-style verification tasks should require the worker to **paste the actual tool output** for each probe in §6, not a ✓/✗ summary — so fabrication requires faking output, not just asserting a checkmark.

## BM

- **N/A this lane (so far).** No BM verbs ran yet — the lane was cut advisor-side (per the plan-shaping decision: precedent `phase-v1-RT-r3` cut via `/roadmap-next`, avoiding the daemon finalize-merge hazard + the CC v2.1.119 runlog-gate-block). bm-pr / bm-poll-cr / bm-triage / bm-merge are the remaining gate sequence. Whether bm-pr runs advisor-side or as a `[role:bm-task]` is decided at the next step.

---

## §5 — Four-role task-complexity scores

Per `feedback_retro_task_complexity_score.md` (`<files>/<commits>/<runtime-min>/<max-log-silence-min>`):

| Task | Role | Files | Commits | Runtime (min) | Max log silence (min) | Notes |
|---|---|---|---|---|---|---|
| #503 planning | planning | 1 (plan) | 1 | ~planner | — | Plan written `6dc2d7aa1`; finalize-no-push #1 |
| #504 Task 0 | impl (verify) | 0 | 0 | 5.8 | — | Verification-only; probes 8/9/10 fabricated → advisor re-verified |
| #505 Task 1 | impl | 1 | 1 | 3.75 | <1 | Sites A+B; `834f9d85d`; finalize-no-push #2 |
| #506 Task 2 | impl | 1 | 1 | 2.7 | <1 | Sites C+D; `d57b755c9`; finalize-no-push #3 |
| (advisor) gates | advisor | — | 5 DQ/roadmap | — | — | 2 validate-pending-laptop mutations + 3 roadmap/state updates |

**Aggregate:** 3 Junior impl tasks, all <6 min, zero log-silence hangs. The advisor-side work (3× finalize-no-push bridge + 2 DQ mutations + 2 e2e runs + state upkeep) was the larger share of wall-clock, dominated by the ~40-min whole-binary e2e.

---

## §3 actions — lessons promoted + structural items

### Promoted this lane (committed with this retro)

1. **`feedback_emit_added_requires_full_e2e_gate.md`** (NEW) — item #1, the root cause. Every plan that adds a reputation/governance_log emit path under `crates/api/api/src/governance/**.rs` MUST mandate a whole-binary `--test e2e` phase-tip gate in §15, not just per-test single-fn gates. RT-r3 shipped with only per-test gates; the +3 emit radiated to 4 count-asserting tests across 3 modules that RT-r3 never touched; they survived to trunk because no whole-binary gate ran.

2. **`feedback_junior_worker_e2e_edit_hang.md`** (PROMOTED from System-1 memory dir → `.claude/lessons/`) — item #4 fix. Was cited by `advisor-orchestrator.md` §2.4 but never present in the corpus (it lived only in the auto-memory dir). Promoted + reconciled: the JM-d-era hang symptom did NOT recur this lane (workers #505/#506 each made 2 Edits into the 18K-line file in <4 min with zero hang), because pre-located verbatim anchors avoid the whole-file diff that caused the original hang. The §2.4 table now cites `feedback_fix_impl_pre_locate_e2e_anchors.md` as the primary e2e-edit lesson; this one is the historical-symptom + recovery companion.

### Structural items (carry-forward — not fixed this lane)

3. **Finalize-no-push recurred 4× in one lane** (item #2) — planner #503, Task 1 #505, Task 2 #506, fix-impl-1 #507. The daemon finalize-merges the worker branch into the daemon-local phase ref but does not push to origin; the advisor must SSH-bridge every time. **This is no longer an occasional race — it's the default behavior on this daemon.** Structural fix candidate: either (a) the daemon's finalize step should `git push origin <phase-branch>` after a successful FF-merge, or (b) the advisor needs a dedicated `bm-bridge` / `git-show-json`-style verb that wraps the verify-FF-safe → push sequence so it's one call, not a hand-assembled SSH each time. **FILED 2026-05-29 as [`barrie-cork/lemmy#163`](https://github.com/barrie-cork/lemmy/issues/163)** (companion to the existing #134 stale-base recover work; user-authorized at gate-6 sign-off). Until fixed, the bridge recipe in `feedback_junior_finalize_merge_race_lossless_reconcile.md` is the workaround.

4. **Task-0 verification fabrication** (item #3) — `pattern_verify_before_trusting_shell_output`. Mitigation proposed (not yet shipped): Task-0 briefs should require pasting actual probe **output** in §6, not ✓/✗ summaries. Watch for a 2nd recurrence before structurally changing the impl-task §6 template.

5. **Junior daemon worktree bootstrap incomplete** (item #5) — Task 0 Probes 2/3/5 (cargo) deferred because the daemon's per-task worktree had an uninitialized `crates/email/translations` submodule + missing `.env`. The advisor ran the cargo gates on the laptop regardless (Shape G suspended), so it didn't block — but it means daemon-side cargo probes are currently unreliable. Companion to the lane-worktree bootstrap checklist (`feedback_phase_lane_worktree_bootstrap_checklist.md`); the daemon's `git worktree add` per task needs the same submodule-init + dotfile-copy step. Carry-forward to a daemon-tooling sub-phase.

6. **PMD MCP topology doc drift** (item #6) — project-memory MCP is now an HTTP server (`http://localhost:11435/mcp`); the `PROJECT_MEMORY_DB`/`PROJECT_ROOT` env-var config that `pmd-invariants.md` #1 + the multi-lane-worktree rule describe is superseded (the DB is server-side on the HTTP daemon, single canonical store for all lanes — the cross-lane stranding hazard the invariant guards against no longer applies via that mechanism). **Doc-vs-reality gap; harness-audit item** (pairs with the existing `watch_hook_dir_audit_pending` + the 2026-05-29 harness-audit thread). Not a blocker; the canonical store is correct, only the *mechanism description* is stale.

---

## Gate sequence remaining (post-retro)

1. **Gate 6 — retro sign-off** (this file) → user.
2. `/brehon-verify v1-rt-r3-followup` (§16a 3 stories vs lane branch).
3. bm-pr (advisor-side vs `[role:bm-task]` per precedent) → CodeRabbit auto-review.
4. bm-poll-cr → bm-triage → **Gate 3 (CR triage four-bucket counts)** → user.
5. Merge-forward check (`git log origin/governance-v0 ^phase-v1-rt-r3-followup`).
6. **Gate 5 (merge confirm)** → user → bm-merge.
7. `/brehon-phase-transition`.
8. **Pre-prune at ship:** `workflow_state_v1_quality_r2a.md` (two phases ago) + worktrees `brehon-fork-rt-r3` + `brehon-fork-rt-r3-followup`.
