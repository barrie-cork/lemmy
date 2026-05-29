# Brief: planning — v1-rt-r3-followup

## 1. Role + dispatch

`[role:planning] v1-rt-r3-followup — plan e2e assertion updates for RT-r3 reputation-emit drift — see .claude/PRPs/briefs/v1-rt-r3-followup-planning-1.md`

## 2. Scope

Plan the implementation of **v1-rt-r3-followup**: update the **stale e2e assertions in `crates/server/tests/e2e.rs`** that count `reputation_event` rows and `governance_log` first-occurrence kinds. RT-r3 (shipped to `governance-v0` at commit `996765cae feat(governance): vote-outcome + evidence-cited emit in submit_jury_vote (task 2)`) added two new reputation-event emit paths but did NOT update the downstream e2e tests that assert exact counts. RT-r3 shipped without a phase-tip e2e gate, so these assertions went stale on trunk undetected until a falsifiable-hypothesis pass on 2026-05-29 traced them.

**This is a test-only fix.** No handler edits, no emit-side edits, no schema, no migration, no config. The new emit behaviour is CORRECT and already merged; the tests are lagging behind the contract and must be brought current.

### 2.1 The RT-r3 emit paths that caused the drift (DO NOT re-edit — context only)

RT-r3 added these two emit paths in `crates/api/api/src/governance/submit_jury_vote.rs` (verified at lane tip):

- **Source 3 — vote-outcome** (line ~620–644): `if juror_decision == winning_decision` → emits `+ParticipationConsistency` reputation_event **per majority-aligned juror** + one `ENTRY_KIND_VOTE_OUTCOME_RECORDED = "vote_outcome_recorded"` governance_log entry per aligned juror. **Data-dependent:** the count is the number of jurors who aligned with the winning decision — a 3-0 unanimous panel adds 3; a 2-1 split adds 2. Fires regardless of whether the winning decision is NoAction or a remove (the gate is alignment with the *majority*, not the decision content).
- **Source 4a — evidence-cited** (line ~730): emits `+ReportingAccuracy` for the reporter **only if** the case has ≥1 `case_evidence` row AND the winning rationale length ≥ `participation.evidence_cited_rationale_threshold_chars` (default 256) + one `ENTRY_KIND_EVIDENCE_QUALITY_RECORDED = "evidence_quality_recorded"` entry. **Data-dependent:** fires 0 times in any test whose fixture does not seed `case_evidence` + a long rationale.

The planner reads these to understand *why* the counts changed — NOT to edit them. Any §13 task proposing an edit under `crates/api/**` or `crates/db_schema/**` is a **scope violation → catch-fire**.

### 2.2 What v1-rt-r3-followup ships

**Assertion-value updates at four test functions in `crates/server/tests/e2e.rs`.** The line numbers below are **verified at the lane tip** (`phase-v1-rt-r3-followup`, 2026-05-29). The file is 14k+ lines and the four sites span the file; re-confirm each anchor with a ±5-line Read before editing.

> ✅ **Expected values are now BASELINE-CONFIRMED (2026-05-29).** The phase-tip e2e baseline (§2.3) completed and the four panics resolved the data-dependency cleanly. The concrete `actual` values below are copied verbatim from the baseline failures summary at `C:/Users/barri/Developer/brehon-fork-rt-r3-followup/.claude/v1-rt-r3-followup-e2e-baseline.log`. The planner still re-reads the baseline to confirm (the log is the authority), but the derivation is done — these are the values the running RT-r3 code produces. The handover's flat "+3 → 7" hypothesis happens to land on 7 for the *totals* (all three failing fixtures ran 3-aligned-juror panels), but the **per-dimension split on Site A does NOT move** (see Site A row) and **evidence-cited does NOT fire on any of these paths** (no seeded `case_evidence`), so Site B is a SINGLE insertion, not two.

| # | Test function | Verified line(s) | Stale → baseline-confirmed | Notes |
|---|---|---|---|---|
| A | `report_to_modlog_golden_path` (fn at line 2485) | **2949** `rep_total, 4`; **2959** `jury_rep_count, 3`; **2967** `reporter_rep_count, 1` | **`rep_total` 4 → 7** (panic at 2948: `left: 7, right: 4`). **`jury_rep_count` stays 3. `reporter_rep_count` stays 1.** | The +3 lands on the `ParticipationConsistency` dimension (vote-outcome, per majority-aligned juror — 3 aligned jurors here). Lines 2959 + 2967 filter `JuryReliability` / `ReportingAccuracy` respectively, so they are UNAFFECTED. **Editing 2959 or 2967 is a regression — change ONLY line 2949.** No evidence-cited row (golden path seeds no `case_evidence`). |
| B | `governance_log_sequence_matches_prd_state_machine` (fn at line 11054) | **11201** `expected_prefix` vec; assert at **11215** | **Insert `"vote_outcome_recorded"` between `"public_log_published"` and `"case_decided"`.** `"evidence_quality_recorded"` does NOT fire on this path — do NOT add it. | Baseline `got` (actual) = `[report_created, threshold_met, severity_tier_frozen, jury_assigned, panel_assembled, jury_accepted, public_log_published, vote_outcome_recorded, case_decided, appeal_requested, appeal_panel_assembled, appeal_decided]`. Stale `expected_prefix` is identical MINUS `vote_outcome_recorded`. **Single-element insertion** at position 7 (0-indexed). |
| C | `submit_jury_vote_preserves_v0_decided_for_no_sponsor_target` (fn at line 13866, in `mod v1_sl_d_fixtures`) | **14025–14026** `rep_count, 4` → **`rep_count, 7`** (panic at 14025: `left: 7, right: 4`) | single total `reputation_event::table.count()` (no dimension filter). Also update the explanatory comment at line 14020 (`// 3 juror ... = 4 total.`) to reflect the +3 ParticipationConsistency rows = 7 total. | vote-outcome adds 3 ParticipationConsistency rows (3 majority-aligned jurors on the Decided path). |
| D | `submit_jury_vote_no_action_skips_liability_machinery` (fn at line 14059, in `mod v1_sl_d_fixtures`) | **14244–14245** `rep_count, 4` → **`rep_count, 7`** (panic at 14244: `left: 7, right: 4`) | single total `reputation_event::table.count()`. Also update the comment at lines 14238–14239 to reflect 7 total. The `sponsor_rep_count, 0` assertion at 14233 is UNRELATED — leave it. | vote-outcome adds 3 ParticipationConsistency rows (3 jurors aligned with the NoAction majority). |

**Baseline summary (for the plan §10 prose + §16a expected outputs):** `test result: FAILED. 115 passed; 4 failed; 5 ignored`. The 4 failed are exactly the four sites above — no 5th regression, no all-green surprise. Post-fix DoD = these 4 green + the 115 still green (NEGATIVE-test gate). The full panic transcript is in the baseline log; cite the log path + the four `left/right` pairs in the plan, not a re-paste of the log body.

**Cohort shape (planner decides, advisor validates):** sites A + B are distinct top-level functions (disjoint within the file but same file → see §4 cohort note). Sites C + D are both in `mod v1_sl_d_fixtures`. Per `feedback_junior_worker_e2e_edit_hang.md` §2.0 scope gate (≤2 file edits per task, ≤2 Edits per file per task), the natural split is **two impl tasks**: Task 1 = sites A + B (2 Edits in one function-distinct region pair), Task 2 = sites C + D (2 Edits in the fixtures module). The planner confirms and writes the FILES YAML so the advisor's cohort logic can reason about it. NOTE: because all four edits are in the **same file** (`e2e.rs`), the two tasks CANNOT run as a parallel `[P]` cohort — they share `e2e.rs` and would collide on the same file. Plan them **serial** (Task 2 `requires:` Task 1). See §4.

**Out of scope for v1-rt-r3-followup:**

- No edits under `crates/api/**`, `crates/db_schema/**`, `crates/routes/**` — the emit paths are correct and merged. Test-only fix.
- No new migrations, config keys, or `ENTRY_KIND_*` constants.
- No new test functions — this updates existing assertions in place.
- No refactor of the e2e fixtures modules — surgical assertion-value + matching-comment updates only.
- The 3 remaining RT-r3 carry-forward CR fixes (#156/#159/#160, tracked as roadmap `v1-quality-r2` unstarted) — NOT this lane.

## 2.3 The e2e baseline (COMPLETE — values already injected into §2.2)

A full phase-tip e2e run was launched on the lane worktree (`brehon-fork-rt-r3-followup`) at session start, BEFORE this brief was authored, specifically so the plan's expected assertion values come from real test output rather than the handover's unverified hypothesis. **It completed (exit nonzero, 4 failures as expected) on 2026-05-29 and the four `left/right` pairs are now baseline-confirmed in §2.2.** The log is at:

```
C:/Users/barri/Developer/brehon-fork-rt-r3-followup/.claude/v1-rt-r3-followup-e2e-baseline.log
```

The §2.2 table already carries the verified `actual` values. The planner re-reads the baseline failures summary (the `failures:` section near the tail) to **confirm** the four pairs before authoring §13 — the log is the authority, §2.2 is the pre-digested transcription. For each of the four sites, the panic shows:

```
assertion `left == right` failed: <message>
  left: <ACTUAL — what the current code produces>   ← the value to assert post-fix
 right: <EXPECTED — the stale value being asserted>  ← the value being replaced
```

For Site B (the sequence vector), the panic message includes `got {sequence:?}` — the actual kind ordering (already transcribed into §2.2 Site B Notes).

**Baseline result line (verbatim):** `test result: FAILED. 115 passed; 4 failed; 5 ignored; 0 measured; 0 filtered out`. The 4 failed = the four §2.2 sites exactly. No 5th regression; no all-green surprise. If on re-read the planner finds the baseline disagrees with §2.2 (e.g. a concurrent trunk change re-fixed a site, or a 5th test now fails), the planner files a `kind: "blocker"` DQ and surfaces to the advisor before authoring §13 — §2.2 would then be stale and the scope must be re-evaluated.

## 3. Required reading

- `workflow_state_v1_rt_r3_followup.md` (auto-loaded via MEMORY.md) — the "Root cause + target" section + the 2026-05-29 re-verification block. **Authoritative over the bootstrap's table** where they conflict (the bootstrap's "Target #4" + "rep_count == 7" rows are superseded by this brief's §2.2 table).
- `.claude/PRPs/handovers/v1-rt-r3-followup-bootstrap.md` §1–§4 — phase context, lessons-that-apply, watchlist. Read §4 watchpoint #1 (line-drift) + #4 (NEGATIVE-test gate) closely. Treat the §1 assertion table as SUPERSEDED by this brief's §2.2 (the bootstrap predates the re-verification).
- `.claude/PRPs/briefs/v1-RT-r3-planning-1.md` — the planning brief for the phase that *added* these emits. §2.2 Sources 3 + 4a define the exact emit semantics. This is the canonical-schema-first reference (`feedback_read_canonical_before_writing_spec.md`).
- `.claude/PRPs/plans/v1-RT-r3.plan.md` — the RT-r3 plan; §13 task that added the emit paths + its §16a stories. Cite this in the plan §10 prose so a future CR reviewer sees the contract was updated by RT-r3 and the e2e was merely lagging.
- `crates/api/api/src/governance/submit_jury_vote.rs` lines ~600–740 (Source 3 vote-outcome emit at ~620–644; Source 4a evidence-cited emit at ~730) — read to understand WHY the counts changed. Do NOT edit.
- `crates/server/tests/e2e.rs` — **±50-line Read windows around lines 2485–2970, 11185–11218, 13866–14060, 14059–14250 ONLY.** Do NOT bulk-read the 14k-line file (`feedback_junior_worker_e2e_edit_hang.md` + memory-headroom rule). Read each of the four sites' surrounding function bodies to understand the vote/evidence fixture setup, which determines the data-dependent expected values.
- `docs/brehon-law-inspired-network/04-data-model-and-api.md` §"governance_log sequence" (or §6.7 state machine) — the PRD contract for the first-occurrence kind ordering. Site B's `expected_prefix` must match this contract's ordering for the kinds it includes.
- `.claude/rules/advisor-orchestrator.md` §2.4 — mandatory file-class lesson injection table (e2e.rs rows).
- `.claude/lessons/feedback_lemmy_error_no_std_error.md` — **mandatory for e2e.rs edits.** All four target functions already return `LemmyResult<()>` (verified: `report_to_modlog_golden_path` and `governance_log_sequence_matches_prd_state_machine` return `lemmy_utils::error::LemmyResult<()>`; the two `v1_sl_d_fixtures` fns return `LemmyResult<()>`). This is **Case A** (sibling fixtures module uses `LemmyResult<()>` outer) — no `.map_err` bridges needed; the edits are pure value swaps inside an already-correct error-shape. Confirm at the Read windows.
- `.claude/lessons/feedback_async_pool_test_pattern.md` — **mandatory for e2e.rs edits** (pool/conn fixture pattern; no new helpers expected but read for consistency).
- `.claude/lessons/feedback_junior_worker_e2e_edit_hang.md` — **mandatory (plan totals 4 e2e.rs edits ≥ 2).** Pre-locate verbatim `old_string`/`new_string` for every Edit; §2.0 scope gate (≤150 lines, ≤2 file edits per task, ≤2 Edits per file per task) forces the 2-task serial split.
- `.claude/lessons/feedback_fix_impl_pre_locate_e2e_anchors.md` — pre-locate exact anchors for every e2e Edit (the impl briefs derived from this plan must carry verbatim anchors).
- `.claude/lessons/feedback_plan_drift_metadata_cross_check.md` — cross-check assertion values against what the code actually produces (the baseline), NOT against the plan/handover row text. This is the lesson the handover's wrong "+3 → 7" row violated.
- `.claude/lessons/feedback_advisor_watchpoint_specificity.md` — every plan §4 watchpoint cites a specific test name + file + line + expected/actual.
- `.claude/lessons/feedback_plan_dod_dry_run_at_write.md` + `.claude/lessons/feedback_plan_baseline_self_reference.md` — DoD §15 commands must be executable as written; cite the gate by test name + assertion-line range, NEVER by a drifting SHA.
- `.claude/lessons/feedback_explicit_file_arrays_on_tasks.md` — FILES YAML (`creates:` empty, `modifies: [crates/server/tests/e2e.rs]`, `requires:`) on every §13 task.
- `.claude/lessons/feedback_laptop_default_for_validate_pending.md` — Shape G suspended until 2026-06-01; cargo runs on the laptop lane worktree via `kind: "validate-pending-laptop-e2e"` DQ.

## 4. Constraints

- **TEST-ONLY scope (hard).** Every §13 task modifies ONLY `crates/server/tests/e2e.rs`. Any task proposing an edit under `crates/api/**`, `crates/db_schema/**`, `crates/routes/**`, `migrations/**`, or `Cargo.*` is a **scope violation → catch-fire**. The emit paths are correct and already merged; this lane brings the tests current.
- **Expected values are baseline-confirmed in §2.2 — cite them by `left/right` pair.** The plan §13/§16a expected-value cells MUST match the baseline-confirmed values transcribed in §2.2 (sourced from `.claude/v1-rt-r3-followup-e2e-baseline.log`, per §2.3). The planner confirms against the log on read; it does not re-derive from emit semantics. A plan whose expected values disagree with the baseline `left:` values is a process miss → advisor rejects at gate-1. Per `feedback_plan_drift_metadata_cross_check.md`.
- **The +3 is ParticipationConsistency, NOT JuryReliability/ReportingAccuracy.** The totals all land on **7** (3 majority-aligned jurors × +1 ParticipationConsistency each, on top of the prior 4) because all three failing fixtures happen to run 3-aligned-juror panels — confirmed by the baseline, not assumed. But this has TWO non-obvious consequences the plan MUST encode: (1) **Site A's per-dimension sub-counts (`jury_rep_count==3` at 2959, `reporter_rep_count==1` at 2967) do NOT change** — they filter `JuryReliability`/`ReportingAccuracy`, and the new rows are `ParticipationConsistency`. Changing them is a regression. ONLY line 2949 (`rep_total`) moves. (2) **`evidence_quality_recorded` / evidence-cited (Source 4a) fires on NONE of these four paths** (no fixture seeds `case_evidence` + ≥256-char rationale) — so Site B is a single `vote_outcome_recorded` insertion, and no ReportingAccuracy delta appears anywhere. Do NOT add `evidence_quality_recorded` to Site B's `expected_prefix`.
- **All four sites are in the SAME file → serial, NOT `[P]`.** Two impl tasks both modify `crates/server/tests/e2e.rs`; they share the file and CANNOT run as a parallel cohort (FILES YAML overlap → the advisor's §4.1 step 4 overlap check would degrade to serial anyway). Plan Task 2 with `requires: [<Task 1 id>]` so it forks from a phase branch that already has Task 1's edits. Mark NEITHER task `[P]`.
- **Comment-consistency edits are in-scope and required.** Sites C and D have explanatory comments (`// 3 juror + 1 reporter = 4 total.`) that state the old count. When the assertion value changes, the comment MUST change to match — a stale comment next to a fixed assertion is a process miss. Count the comment edit within the §2.0 ≤2-Edits-per-file budget (so Site C = assertion Edit + comment Edit if they're non-adjacent; if adjacent, one Edit covers both).
- **NEGATIVE-test gate (carry-forward from RT-r3 + r2a):** the DoD is the FULL e2e suite green on the phase tip, NOT just the 4 changed tests. Fixing the 4 assertions must not flip any currently-green test to red. Plan §15 mandates `--test e2e` (whole binary), never `--test e2e <single-test>`. The baseline log records the pre-fix pass/fail set; the post-fix run must show the 4 previously-failing tests now green AND every previously-green test still green.
- **Mandatory lesson injection per `.claude/rules/advisor-orchestrator.md` §2.4** (e2e.rs file class, ≥2 edits):
  - `feedback_lemmy_error_no_std_error.md` (Case A — sibling fixtures use `LemmyResult<()>`; no `.map_err` bridges; pure value swaps).
  - `feedback_async_pool_test_pattern.md`.
  - `feedback_junior_worker_e2e_edit_hang.md` (≥2 e2e edits across the plan).
  - `feedback_fix_impl_pre_locate_e2e_anchors.md` (impl briefs carry verbatim anchors).
- **MIRROR-ref discipline:** for each assertion edit, the plan cites the exact `old_string` line(s) verified in §2.2. The impl briefs derived from the plan pre-locate verbatim `old_string`/`new_string` per `feedback_fix_impl_pre_locate_e2e_anchors.md` — no "search for the assertion near line N" instructions; exact anchors only.
- **FILES YAML on every §13 task** — `creates: []`, `modifies: [crates/server/tests/e2e.rs]`, `requires:` (Task 2 requires Task 1). Per `feedback_explicit_file_arrays_on_tasks.md`; load-bearing for the advisor's cohort overlap + dependency checks.
- **Validation is e2e-on-laptop (Shape G suspended until 2026-06-01, DQ #229).** Impl-task briefs whose DoD includes e2e: after the worker pushes, write a `kind: "validate-pending-laptop-e2e"` DQ entry (commands array = the verbatim §15 e2e command, branch, phase_task) and **stop** — the laptop advisor session runs e2e on the lane worktree and mutates the DQ. Do NOT run e2e on the EliteDesk worker. Per `feedback_laptop_default_for_validate_pending.md` + `feedback_windows_e2e_requires_bat_wrapper.md`.
- **Plan §15 DoD (dry-runnable):** the single gate is
  `cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > <log> 2>&1 && echo E2E_EXIT_0 >> <log> || echo E2E_EXIT_NONZERO >> <log>"` (Windows lane worktree). Cite the gate by test-name + assertion-line range, never by SHA (`feedback_plan_baseline_self_reference.md`). There is no `cargo check`-only substitute for the DoD here — the deliverable IS the e2e suite passing.
- **Commit attribution:** impl-task commits use `test(governance): <description> (task N)` or `fix(e2e): <description> (task N)` subject (test-only fix; mirror RT-r3's `feat(governance): ... (task N)` numbering convention).
- **Plan §4 watchpoints** cite specific file:line:
  - `crates/server/tests/e2e.rs:2949` + `:2959` + `:2967` (Site A — three related assertions; verify the per-dimension split, not just the total).
  - `crates/server/tests/e2e.rs:11201` (Site B — `expected_prefix` vec; first-occurrence ordering must match PRD §6.7 for included kinds).
  - `crates/server/tests/e2e.rs:14025` (Site C) + `:14244` (Site D) — single total `rep_count` each; matching comments at `:14020` / `:14238`.
  - `crates/api/api/src/governance/submit_jury_vote.rs:620` (Source 3 vote-outcome emit — the cause; data-dependent on majority-alignment) + `:730` (Source 4a evidence-cited — data-dependent on seeded evidence).
- **Plan §16a stories** — minimum 2 (one per impl task), each with a checkpoint command that runs the affected test(s) on the phase tip:
  1. Sites A + B green: `report_to_modlog_golden_path` reputation counts + governance_log sequence match the post-RT-r3 emit behaviour.
  2. Sites C + D green: both `v1_sl_d_fixtures` submit-jury-vote tests' `rep_count` totals match the post-RT-r3 vote-outcome emit behaviour.
  Plus the cross-cutting story: full `--test e2e` suite green on the phase tip (NEGATIVE-test gate).
- **Stop-and-ask tripwires for the planner:**
  - Stop if any §13 task proposes a non-test edit (`crates/api/**`, `crates/db_schema/**`, `migrations/**`) → catch-fire (test-only lane).
  - Stop if the baseline log shows all-green (no failing assertions) → the 4-stale-assertion premise is falsified; file a `kind: "blocker"` DQ and surface to advisor before writing §13 tasks.
  - Stop if the baseline shows a 5th failing e2e test beyond the four enumerated in §2.2 → a separate regression class landed on trunk between the 2026-05-29 trace and lane-cut; surface to advisor — the lane scope must be re-evaluated.
  - Stop if any of the four line anchors has drifted by >100 lines from this brief's §2.2 values → the file was substantially refactored; re-verify test names + assertion bodies via `grep -n` before authoring §13.
  - Stop if a target test's outer `Result` type is NOT `LemmyResult<()>` (contradicting the §3 Case-A claim) → re-read the sibling shape per `feedback_lemmy_error_no_std_error.md` before authoring the Edit anchors.
  - Stop if the planner is tempted to propose >2 §13 tasks — four single-line assertion updates in one file is at most 2 serial tasks; more signals over-splitting.
