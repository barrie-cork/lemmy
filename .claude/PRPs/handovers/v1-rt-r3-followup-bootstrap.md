---
phase: v1-rt-r3-followup
plan: .claude/PRPs/plans/v1-rt-r3-followup.plan.md   # (not yet authored)
phase_branch: phase-v1-rt-r3-followup                # not yet created until bm-cut
worktree: C:/Users/barri/Developer/brehon-fork-rt-r3-followup   # created at bm-cut; until then canonical brehon-fork
authored: 2026-05-29
authored_by: advisor (canonical brehon-fork / governance-v0 session)
purpose: Bootstrap the v1-rt-r3-followup advisor session. Read the RESUME block first; it is the entry point.
---

# ⏩ RESUME — read this block first

**You are the advisor for Brehon v1-rt-r3-followup.** This is a fresh session (or a resumed one). The Brehon advisor runs inside the brehon-fork checkout — CWD `C:/Users/barri/Developer/brehon-fork` for governance-v0 meta-work, or `C:/Users/barri/Developer/brehon-fork-rt-r3-followup` once `bm-cut` creates the lane worktree (per `.claude/rules/multi-lane-worktree.md`). There is no homeserver session.

## Session-start ritual (do these first)

1. `pwd && git -C C:/Users/barri/Developer/brehon-fork branch --show-current && git -C C:/Users/barri/Developer/brehon-fork worktree list` — confirm CWD/lane. If `git worktree list` shows other active worktrees (e.g. `brehon-fork-redaction-r1`, `brehon-fork-rt-r3`), surface a one-line lane status BEFORE the first tool call per `advisor-orchestrator.md` §1 "Surface-first ritual" (post-RT-r2 boundary incident 2026-05-22).
2. `git -C C:/Users/barri/Developer/brehon-fork fetch origin && git -C C:/Users/barri/Developer/brehon-fork rev-parse --short governance-v0` — must equal `6784f448c` (see §"Git state at handoff"); if drifted, `git -C C:/Users/barri/Developer/brehon-fork log --oneline 6784f448c..governance-v0` and update your mental model before acting.
3. Read `.claude/decision-queue.json` for any pending entries since handoff; compare against the §"Decision-queue snapshot" below.
4. The brehon-fork `MEMORY.md` auto-loads; `workflow_state_v1_rt_r3_followup.md` is the running-state scratchpad (most-recent "Session handoff block" is authoritative on resume). The CLOSED record `workflow_state_v1_quality_r2a.md` is read ONCE at session start for carry-forward.

## Next concrete action

**Step A:** `bm-cut phase-v1-rt-r3-followup` from `governance-v0` HEAD (`6784f448c`) per `.claude/rules/phase-branch.md`. Author bm-cut brief from `.claude/PRPs/templates/bm-task-brief.template.md` (promoted 2026-05-29 in commit `6b9732277`) + 1-2 sibling `*-bm-cut-*.md` briefs as the canonical-schema-first reference (per `feedback_read_canonical_before_writing_spec.md`).

**Step B:** Author narrow planning brief at `.claude/PRPs/briefs/v1-rt-r3-followup-planning-1.md` scoped to the 4 stale e2e assertions named verbatim in §1 below + `workflow_state_v1_rt_r3_followup.md` "Root cause + target" table. Run `/brehon-clarify` BEFORE queuing planning Junior. Plan §15 must mandate phase-tip e2e gate (no other deliverable means no other gate).

**Step C:** Decide lane mode at bm-cut time. **Recommended: Mode A (dedicated lane worktree `brehon-fork-rt-r3-followup`)** because the deliverable is e2e tests + Shape G is SUSPENDED until 2026-06-01 (per `project_shape_g_suspended_2026_05_16.md`) — validate-pending-laptop requires local cargo, which is faster in a lane worktree than via Mode B daemon shuttle. Document the chosen mode in §"Lane mode" of `workflow_state_v1_rt_r3_followup.md` post-decision.

---

## 1. v1-rt-r3-followup in one paragraph

**Task range:** fix 4 stale e2e assertions on `governance-v0` that count reputation events. **Goal:** every e2e test in `crates/server/tests/e2e.rs` passes on phase-tip. **What's new vs the completing phase (v1-quality-r2a):** scope is Rust-touching (e2e assertion updates) where r2a was zero-Rust meta-work. **What's known about the root cause:** RT-r3 ship commit `996765cae feat(governance): vote-outcome + evidence-cited emit in submit_jury_vote (task 2)` added 3 new reputation-event emit paths (vote_outcome_recorded + evidence_cited related) but did NOT update the 4 downstream e2e assertions that count emits. RT-r3 shipped to trunk without phase-tip e2e (RT-r3 §15 missed; lesson candidate for retro). **DoD:** all e2e tests pass (`bash scripts/brehon/cargo-test.sh --workspace --test e2e --features full` exit 0 on phase-tip), PR merged to governance-v0, retro signed off.

**The 4 stale assertions** (lines verified at `6784f448c`; re-verify at lane-cut — the file is 14k+ lines and grows):

| Test name | File | Line | Current expectation | Required fix |
|---|---|---|---|---|
| `governance_log_sequence_matches_prd_state_machine` | `crates/server/tests/e2e.rs` | ~11215 | sequence WITHOUT `vote_outcome_recorded` | add `vote_outcome_recorded` to expected sequence |
| `report_to_modlog_golden_path` | `crates/server/tests/e2e.rs` | ~2948 | `rep_count == 4` | `rep_count == 7` |
| `v1_sl_d_fixtures::submit_jury_vote_no_action_skips_liability_machinery` | `crates/server/tests/e2e.rs` | ~14244 | `rep_count == 4` | `rep_count == 7` |
| `v1_sl_d_fixtures::submit_jury_vote_preserves_v0_decided_for_no_sponsor_target` | `crates/server/tests/e2e.rs` | ~14025 | `rep_count == 4` | `rep_count == 7` |

## 2. Why v1-rt-r3-followup is easier/harder than v1-quality-r2a

**Easier:**
- **Root cause is already known.** `git log -S "ENTRY_KIND_VOTE_OUTCOME_RECORDED" governance-v0` traces to `996765cae`. The 4 failing tests + the 3 new emit paths are enumerable.
- **Scope is bounded to 4 tests.** No new feature work, no schema migration, no AP types, no DTO churn.
- **Plan structure is mechanical.** Each fix is one Edit; the e2e test runs prove the fix.

**Not easier:**
- **e2e.rs is 14k+ lines.** Edit hangs are a real risk class — `feedback_junior_worker_e2e_edit_hang.md` mandates pre-located anchors for every Edit (§2.0 scope gate). 4 distinct sites = 4 narrow Edits. Brief §2.0 caps (≤150 lines, ≤2 file edits per task, ≤2 Edits per file per task) likely force a 2-cohort split: T1+T2 (lines 2948 + 11215, distinct functions) + T3+T4 (lines 14025 + 14244, same fixtures module).
- **Phase 2 e2e local-vs-dispatch is THE gate.** No bash-only gate substitutes (unlike r2a's zero-Rust scope). Plan §15 must mandate phase-tip e2e on the lane worktree before bm-pr.
- **CR might re-question whether the new emit paths are correct.** If CR reads the fix-impl diff as "tests rubber-stamping changed-behavior", it might raise a finding asking to verify the new sequence/count matches PRD. Pre-emptively cite `docs/brehon-law-inspired-network/04-data-model-and-api.md` §"governance_log sequence" + the RT-r3 plan §13 task that added the emits.

## 3. Lessons from v1-quality-r2a that apply to v1-rt-r3-followup

### Advisor-side
- `feedback_falsifiable_hypothesis_before_structural_fix.md` — the root-cause-trace pattern (`git log -S` + ~3 commands) that proved Lane A innocent of these failures should reuse for any e2e regression encountered during the lane.
- `feedback_bm_false_success_advisor_post_condition_catch.md` — extended this session with Haiku-BM-shortcut sub-pattern (6th confirmation). The new 4-item post-condition checklist (commit-subject discipline / produced-file history / flag discipline / HANDOVER trailer) MUST run after every BM-verb "done" in this lane.
- `feedback_advisor_watchpoint_specificity.md` — every watchpoint MUST cite a specific test name + file + line + expected/actual count, not "watch for assertion drift".
- `feedback_pre_phase_dod_smoke_test.md` — DoD smoke must run `bash scripts/brehon/cargo-test.sh --workspace --test e2e --features full` literally against current HEAD as part of plan approval (gate-1).
- `.claude/PRPs/templates/bm-task-brief.template.md` (promoted 2026-05-29 in commit `6b9732277`) — author every bm-task brief in this lane from this template + 1-2 sibling lookups for the verb-specific row.

### Planning-side
- `feedback_plan_dod_dry_run_at_write.md` — planner must dry-run §15 commands at plan-write time. For e2e, that means a placeholder `cargo-test.sh ... > /tmp/v1-rt-r3-followup-e2e-baseline.log 2>&1` line so the gate is mechanical.
- `feedback_plan_baseline_self_reference.md` — never cite "the failing tests at <SHA>" because SHA drifts. Cite by test name + assertion line range.

### Impl-side
- `feedback_junior_worker_e2e_edit_hang.md` — pre-locate verbatim `old_string`/`new_string` for every Edit in this lane. The brief §2.0 scope gate is non-negotiable.
- `feedback_lemmy_error_no_std_error.md` Case A — if any of the 4 tests uses `Result<(), Box<dyn Error>>` outer, flip to `LemmyResult<()>` per sibling-pattern mirror (re-read sibling at cited line range BEFORE authoring the brief).
- `feedback_async_pool_test_pattern.md` — applies to any new helper extracted from the test bodies (unlikely; the fix is in-place assertion updates).

### BM-side
- `feedback_branch_manager_pm_split.md` — bm-task verbs run in their own Junior tasks, NOT inline in the advisor session (with the gate-only carve-outs per `branch-manager.md` "Autonomy bounds").
- `.claude/PRPs/templates/bm-task-brief.template.md` — fresh template; cite §2.1 per-verb cheat sheet for scope + §3 required-reading row for the bm-cut / bm-pr / bm-poll-cr / bm-triage / bm-merge briefs.

## 4. v1-rt-r3-followup-specific watchlist

1. **e2e.rs line drift:** the 4 line numbers above (`~2948`, `~11215`, `~14025`, `~14244`) were correct at `6784f448c`. **Re-verify each at bm-cut time** via `grep -n "ENTRY_KIND_VOTE_OUTCOME_RECORDED\|rep_count == 4\|governance_log_sequence_matches_prd_state_machine\|report_to_modlog_golden_path" crates/server/tests/e2e.rs` and update the workflow-state table + the plan §13 task list. e2e.rs grows by ~50-200 lines per phase; the 4 numbers may have shifted by ±50 lines by the time this brief is read.
2. **RT-r3 ship was missing phase-tip e2e gate** — the root cause this lane fixes. Plan §15 for this lane MUST mandate phase-tip e2e on the lane worktree before bm-pr. The plan-author MUST cite this watchpoint as a "what we're locking in" gate.
3. **Cohort `[P]` for 4-test fix:** likely 2 cohorts (T1+T2 distinct functions in `e2e.rs`; T3+T4 same `v1_sl_d_fixtures` module — needs YAML overlap check). Per `feedback_cohort_validation_dependency_check.md`, parse `requires:` arrays at queue time. Per `feedback_cohort_shared_git_index_contention.md`, cohort ≥3 on daemon single-`.git/` auto-degrades to serial. Lane Mode A (lane worktree) sidesteps the contention class entirely.
4. **NEGATIVE-test gate (carry-forward from r2a fix-impl-3):** for each of the 4 fixes, the test that was failing pre-fix must pass post-fix; the 11 other governance-log tests should all stay green (run full e2e on the phase tip, not just the 4 changed tests). Plan §15 mandates `--test e2e` not `--test e2e <single-test>`.
5. **CR might flag the PRD-alignment question:** as noted in §2, cite `docs/brehon-law-inspired-network/04-data-model-and-api.md` §"governance_log sequence" + the RT-r3 plan §13 task that added the emit paths in the plan §10 prose so CR sees the contract was already updated; the e2e was lagging behind, not vice-versa.
6. **Shape G SUSPENDED until 2026-06-01:** verify the date at session start (`project_shape_g_suspended_2026_05_16.md` + DQ #229). If still suspended → validate-pending-laptop with bash gates per `advisor-orchestrator.md` §5.2; if re-enabled → standard Shape-G dispatch.

## 5. Operational rules

- **Polling cadence:** ~10 min `mcp__junior-brehon__list_tasks` (status only). On transition, full triage per `.claude/rules/advisor-orchestrator.md` §1.
- **Brief discipline:** briefs in `.claude/PRPs/briefs/v1-rt-r3-followup-<role>-<n>.md`, committed to `governance-v0` first. For impl-task briefs, Mode A allows direct authoring on the phase branch in the lane worktree (per `multi-lane-worktree.md` §"Lane modes"). For Mode B fallback, use SSH-merge from canonical via the daemon's main worktree.
- **Pre-queue checks:** `memory_search_hybrid` (limit 5, round-trip <2s) + `/precheck` (now runs `dq-lint-durations.sh` per r2a ship) + §2.4 mandatory file-class lesson injection (e2e.rs file class → inject `feedback_lemmy_error_no_std_error.md`, `feedback_async_pool_test_pattern.md`, and if ≥2 edits also `feedback_junior_worker_e2e_edit_hang.md` + `feedback_fix_impl_pre_locate_e2e_anchors.md`).
- **LemmyResult Case A override:** for any e2e brief in this lane, mirror the sibling-pattern verbatim per `feedback_lemmy_error_no_std_error.md` Case A. Read sibling at cited line range BEFORE authoring the brief — canonical-schema-first gate.
- **Shape G status:** SUSPENDED until 2026-06-01 (DQ #229). validate-pending-laptop with bash + cargo gates on the lane worktree (Mode A) — laptop is the canonical cargo runner (`project_laptop_canonical_cargo_runner.md`).
- **Windows e2e bat-wrapper invocation + explicit-exit-marker reads:** `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` with `run_in_background: true` per `feedback_windows_e2e_requires_bat_wrapper.md` + `feedback_task_notification_exit_summary_unreliable.md` (never trust the task-notification exit code — read the log explicit-marker).
- **Cohort discipline:** `[P]` only with FILES YAML disjoint + `requires:` resolved + budget under 10 GB + Mode A worktree (sidesteps daemon `.git/index.lock` cohort-≥3 contention). Likely 2 serial cohorts for this lane.
- **Model tiering:** Planning → Opus 4.7, Impl → Sonnet 4.6, BM/ci-watcher → Haiku 4.5. Per `feedback_brehon_subagent_model_effort_assignments.md`.
- **Clarify gate:** every planning brief runs through `/brehon-clarify` before the planning Junior is queued. Mode user-relay or self-resolve with citation.
- **The 6 user gates** (per `CLAUDE.md` "Mandatory user gates"): plan approval / judgment-heavy DQ / CR triage / e2e local-vs-dispatch / merge confirm / retro sign-off.
- **DQ attribution:** advisor commits use `^(chore|docs)\((advisor|decision-queue)\)` subject pattern. Junior subagents NEVER write `answered_by: "advisor"` or `approved_by` (DQ Hard refusals #1 + #8). Use `bash scripts/brehon/dq-v3-new-entry.sh` + `bash scripts/brehon/dq-v3-append-fragment.sh` for v3 composite-id DQ writes (Hard refusal #9).
- **Memory headroom:** NO bulk `crates/server/tests/e2e.rs` reads (file is 14k+ lines). Read ±50-line windows around the 4 cited lines only.
- **BM Haiku post-condition checklist (post-r2a 4-item carry-forward):** after every BM-verb "done", verify (1) commit-subject discipline (`git log --grep <pattern>` returns ≥1), (2) brief §2/§7 produced files (`git log -- <path>` returns ≥1), (3) brief §4 flag discipline (parse worker log; diff against brief's required flags), (4) HANDOVER trailer present (`git log -1 --format=%B <sha> | grep HANDOVER:`). Failure on any → advisor authors missing artifact post-task.

## 6. What changed from v1-quality-r2a's rule set

- **r2a was Mode B (zero-Rust); this lane is Mode A (e2e cargo).** Lane worktree `brehon-fork-rt-r3-followup` created at bm-cut time. Lane-bootstrap checklist per `feedback_phase_lane_worktree_bootstrap_checklist.md` (4 steps: submodule init + `.mcp.json` + `.env` + `settings.local.json`).
- **bm-task-brief template now exists** (`6b9732277`). Author all bm-task briefs from it + verb-sibling lookup. r2a authored briefs ad-hoc — this lane uses the template.
- **BM Haiku post-condition checklist is now load-bearing** (extended from r2a's Junior #502 incident). Apply to every BM-verb "done" in this lane.
- **The dq-lint-durations.sh + precheck.sh harness now runs on every `/precheck`** (shipped in r2a Task 1). If the lane's DQ duration entries violate the lint, `/precheck` exits non-zero pre-dispatch — surface as a `kind: "blocker"` DQ and fix the entry's `resolved_at` before re-attempting.

## 7. Catch-fire procedures

Universal triggers from `.claude/rules/advisor-orchestrator.md` §5.5 plus phase-specific additions:

- **Universal:** Junior subagent ignores hard refusals; `answered_by: "advisor"` in non-advisor commit; subagent commits to `governance-v0` or `main` directly; `bm-task` opens PR into `main`; phase branch has uncommitted state when Junior reports complete; rust-analyzer-lsp missing on EliteDesk daemon when LSP-dependent task; workflow run exceeds 60-min ci-watcher cap; cancelling a Junior task with uncommitted code (SSH tar before cancel per `feedback_cohort_shared_git_index_contention.md`).
- **Phase-specific:**
  - **e2e fix shifts another e2e test from green to red** — the lane is locked to 4 specific assertion fixes; introducing a 5th regression (a previously-passing test fails) is a catch-fire. Plan §15 includes a baseline `cargo-test.sh --workspace --test e2e` run pre-impl to record the baseline pass/fail set.
  - **Reputation-event emit count changes mid-lane** — if a parallel session lands a new emit path on `governance-v0` while this lane is open, the 4 fixes' assertion deltas (`+3`) become wrong. Daily `git log governance-v0 -S "ENTRY_KIND_VOTE_OUTCOME_RECORDED\|emit_reputation_event" --since="<lane-cut-date>"` check.
  - **CR raises a "tests rubber-stamping behavior change" finding** — pre-emptively surface via plan §10 prose; do NOT auto-bucket as fix-in-pr without user input (likely rebut with PRD citation).

## 8. Archive after v1-rt-r3-followup

The standard close: run `/brehon-phase-transition v1-rt-r3-followup <next-id>`. This skill will: close `workflow_state_v1_rt_r3_followup.md`, delete the two-ago record (`workflow_state_v1_quality_r2a.md`), create the next skeleton, write the next bootstrap file, update brehon-fork MEMORY.md, commit on governance-v0. The prior bootstrap file (this one) stays in `.claude/PRPs/handovers/` as its own archive — git history is the archive; no move.

---

## Git state at handoff (captured literally — do not paraphrase)

- governance-v0 HEAD: `6784f448c` (captured 2026-05-29) — `chore(runlog): v1-quality-r2a runlog backfill + extend BM false-success lesson`
- Phase branch HEAD: not yet created (branch `phase-v1-rt-r3-followup` cut at bm-cut)
- Recent governance-v0 commits (`git -C C:/Users/barri/Developer/brehon-fork log --oneline -5 governance-v0`):

  ```
  6784f448c chore(runlog): v1-quality-r2a runlog backfill + extend BM false-success lesson
  8b2e5b1ef Merge pull request #161 from barrie-cork/phase-v1-quality-r2
  716f20e67 chore(advisor): bm-merge-1 brief for v1-quality-r2a (gate-5 confirmed, PR #161 ready)
  6b9732277 chore(advisor): promote bm-task-brief.template.md (3x recurrence threshold met)
  be0807571 Merge junior/role-bm-task-v1-quality-r2a — bm-poll-cr-3 re-poll PR #161 after fix-impl-3
  ```

- Active concurrent worktrees at handoff:
  - `C:/Users/barri/Developer/brehon-fork-redaction-r1 @ 32e830585 [phase-v1-redaction-r1]` — DEFERRED at gate-4 e2e; resumes AFTER this lane ships
  - `C:/Users/barri/Developer/brehon-fork-rt-r3 @ ada35e1b2 [phase-v1-RT-r3]` — stale (RT-r3 shipped; lane worktree not yet pruned — close manually post-this-lane via `git worktree remove ../brehon-fork-rt-r3 && git branch -d phase-v1-RT-r3` after confirming origin branch deleted)

## Decision-queue snapshot at handoff

```decision-queue-snapshot
(empty at handoff)
```

DQ on `governance-v0 @ 6784f448c`: pending=0, resolved=214. No carry-pending entries for this lane. The C3 deferral DQ `dd6012873857-001` is in `resolved[]` and triggers only on the 3rd reputation-event emit path landing — NOT triggered by this lane's 4-assertion fix (this lane updates downstream test expectations, not the emit-side helper extraction).

## Stop-and-ask tripwires

Stop and ask if: the planning Junior proposes a `crates/api/api/src/governance/**.rs` edit — this lane is e2e-test-only; an emit-side or handler-side edit is a scope violation. The 4 assertion fixes live entirely in `crates/server/tests/e2e.rs`.

Stop and ask if: the bm-cut brief proposes any base other than `governance-v0` — per `phase-branch.md`, base MUST be `governance-v0`, never `main`.

Stop and ask if: the §15 DoD smoke at gate-1 surfaces a 5th failing e2e test (beyond the 4 enumerated above) — this means a separate regression class landed on `governance-v0` between the root-cause trace (2026-05-29) and lane-cut; the lane scope must be re-evaluated before queueing planning.

Stop and ask if: any of the 4 line numbers (`~2948`, `~11215`, `~14025`, `~14244`) has drifted by >100 lines from `6784f448c` — the file may have been substantially refactored; re-verify the test names + assertion bodies via `grep -n` before authoring the plan.

Stop and ask if: the phase-tip e2e gate (Phase 2) exceeds 60 min on the lane worktree — likely a Docker / testcontainers issue (per `feedback_orphan_test_exe_blocks_relink.md` + Windows-specific lessons). Surface diagnostic options, do NOT silently retry.
