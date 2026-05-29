---
role: impl-task
plan_task: 2
phase: v1-rt-r3-followup
created: 2026-05-29
related_dq: null
---

# Brief — v1-rt-r3-followup Task 2 — Sites C+D: rep_count 4→7 + matching comments in `mod v1_sl_d_fixtures`

> **Clarify provenance:** parent planning brief clarified via `/brehon-clarify` (3 advisor-self-resolved entries `a3d0e9941441-034/-035/-036`). No clarify-DQ gates an impl-task.

## 0. Pre-flight (subagent runs this before reading anything else)

The `impl-task` agent runs the forbidden-window check from `.claude/agents/impl-task.md`. **Shape G SUSPENDED (DQ #229, until 2026-06-01): this lane runs ZERO cargo on the EliteDesk daemon.** After committing + pushing, you write a `kind: "validate-pending-laptop-e2e"` DQ entry and STOP — the advisor laptop runs the §5 cargo gates (including the headline whole-binary e2e). Do NOT run cargo on the daemon; do NOT write `kind: "validate-pending"`; do NOT capture a `workflow_run_id`.

## 1. Role + dispatch line

`[role:impl-task] v1-rt-r3-followup task 2 — see .claude/PRPs/briefs/v1-rt-r3-followup-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 2** from `.claude/PRPs/plans/v1-rt-r3-followup.plan.md` §13 "Task 2" + §10.3 (Site C anchor) + §10.4 (Site D anchor). **TWO Edits, ONE file (`crates/server/tests/e2e.rs`), ONE commit.** This is the LAST impl task; its validation is the headline phase-tip whole-binary e2e NEGATIVE-test gate.

## 2. Scope

**Produce** (one commit):
- `crates/server/tests/e2e.rs` — **Edit 2.1** (Site C, inside `mod v1_sl_d_fixtures`): multi-line replacement covering the matching comment + `assert_eq!(rep_count, 4, ...)` → `rep_count, 7`. **Edit 2.2** (Site D, inside `mod v1_sl_d_fixtures`): multi-line replacement covering the matching comment + `assert_eq!(rep_count, 4, ...)` → `rep_count, 7`.

Both edits collapse the comment update + assertion-value swap into **one Edit each** (honoring the ≤2-Edits-per-file-per-task budget). Both are **pure value + comment swaps inside an already-correct error shape** (`LemmyResult<()>` Case A — verified: `submit_jury_vote_preserves_v0_decided_for_no_sponsor_target` @L13867 and `submit_jury_vote_no_action_skips_liability_machinery` @L14060 both return `LemmyResult<()>`). No signature changes, no `.map_err` bridges, no helper extraction.

**Do NOT** in this task:
- Edit Sites A or B (Task 1 — already shipped on this branch tip at `834f9d85d`).
- Touch `crates/server/tests/e2e.rs` line ~14235 (`assert_eq!(sponsor_rep_count, 0, ...)`) — see GOTCHA below; it filters by `person_id.eq_any(&sponsor_ids)` and stays 0. Editing it is a **regression**.
- Add `"evidence_quality_recorded"` anywhere — Source 4a (evidence-cited) does not fire on these fixtures (no seeded `case_evidence`).
- Touch any file under `crates/api/**`, `crates/db_schema/**`, `crates/routes/**`, `migrations/**`, `Cargo.*` — **scope violation → catch-fire** (the emit paths are correct + merged in RT-r3; this is a test-only fix).

**Commit message** (exactly): `test(governance): bring v1_sl_d_fixtures submit_jury_vote rep_count assertions current with RT-r3 vote-outcome emit (task 2)`

### 2.1 VERBATIM EDIT ANCHORS (pre-located against the CURRENT lane worktree tip by the advisor — apply EXACTLY, do NOT "search near line N")

> **Advisor pre-locate (2026-05-29, against tip `197815f63`):** `rg -n "rep_count, 4" crates/server/tests/e2e.rs` returns EXACTLY TWO matches — line 14027 (Site C) + line 14246 (Site D). `sponsor_rep_count` appears at 14229 (query) + 14235 (assertion, value 0 — do NOT touch). These line numbers are POST-Task-1 (the Task 1 insert at ~L11209 shifted Sites C+D +1 line from the plan's pre-Task-1 mirror cites of 14025/14244). Anchors below are TEXT-based; apply by exact text match.

**Edit 2.1 — Site C `old_string`** (9-line block — comment + diesel query + assertion):

```rust
    // 3 juror reputation events (3 votes cast) + 1 reporter = 4 total.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count, 4,
      "3 juror + 1 reporter reputation events fire immediately on Decided path"
    );
```

**Edit 2.1 — Site C `new_string`:**

```rust
    // 3 ParticipationConsistency (RT-r3 vote-outcome emit, 3 majority-aligned jurors)
    // + 3 JuryReliability (3 votes cast) + 1 ReportingAccuracy (reporter) = 7 total.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count, 7,
      "3 ParticipationConsistency + 3 JuryReliability + 1 ReportingAccuracy reputation events fire immediately on Decided path (RT-r3 vote-outcome added the 3 ParticipationConsistency rows)"
    );
```

**Edit 2.2 — Site D `old_string`** (10-line block — comment + diesel query + assertion):

```rust
    // Juror events fire for the 3 who voted; reporter event fires (1 row).
    // Total = 3 juror + 1 reporter = 4.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count, 4,
      "3 juror + 1 reporter reputation events fire on NoAction Decided path"
    );
```

**Edit 2.2 — Site D `new_string`:**

```rust
    // Juror events fire for the 3 who voted (3 JuryReliability) + reporter (1 ReportingAccuracy);
    // RT-r3 vote-outcome adds 3 ParticipationConsistency (3 NoAction-aligned jurors).
    // Total = 3 ParticipationConsistency + 3 JuryReliability + 1 ReportingAccuracy = 7.
    let rep_count: i64 = reputation_event::table
      .count()
      .get_result(&mut conn)
      .await?;
    assert_eq!(
      rep_count, 7,
      "3 ParticipationConsistency + 3 JuryReliability + 1 ReportingAccuracy reputation events fire on NoAction Decided path (RT-r3 vote-outcome added the 3 ParticipationConsistency rows)"
    );
```

**Anchor-uniqueness check BEFORE applying:** both 9-line/10-line `old_string` windows are distinctive (each carries its specific stale comment text). The two `rep_count, 4` matches map 1:1 to Sites C (14027) + D (14246). If either `old_string` matches more than once, expand `old_string` upward to include the preceding `decided_count` assertion (Site C: `assert_eq!(plog_count, 1, ...)` neighbor; Site D: `assert_eq!(decided_count, 1, "1 case_decided log entry");` at ~14226) for disambiguation, per plan §10.3/§10.4. If you cannot make an anchor unique, STOP + file a `kind: "blocker"` DQ.

## 3. Required reading

In this order:
1. **Plan §10.3** (Site C anchor + Decided-path GOTCHA) + **§10.4** (Site D anchor + NoAction-path-also-fires-vote-outcome GOTCHA + "sponsor_rep_count is UNRELATED" note).
2. **Plan §13 Task 2** (the IMPLEMENT / MIRROR / GOTCHA / VALIDATE block, lines 483–545).
3. **MIRROR refs** — `crates/server/tests/e2e.rs:14018-14035` (Site C surrounding context) and `:14225-14255` (Site D context — note the `sponsor_rep_count, 0` neighbor at ~14235 that MUST stay unedited). Read these ±20-line windows ONLY. **NEVER bulk-read `e2e.rs` (18,090 lines)** — memory-headroom rule.
4. **Lessons (mandatory per `.claude/rules/advisor-orchestrator.md` §2.4 file-class injection — e2e.rs edit, ≥2 edits):**
   - `feedback_lemmy_error_no_std_error.md` — **Case A** (`mod v1_sl_d_fixtures` uses `LemmyResult<()>` outer throughout; pure value swaps, no `.map_err`). Both target fns confirmed `LemmyResult<()>`.
   - `feedback_async_pool_test_pattern.md` — pool/conn fixture pattern (consistency reference; no new helper).
   - `feedback_fix_impl_pre_locate_e2e_anchors.md` — verbatim-anchor + ≤2-Edits-per-file discipline. **(This is the present canonical e2e-edit-hang-prevention lesson. The §2.4 table also names `feedback_junior_worker_e2e_edit_hang.md`, which is ABSENT from the lessons corpus — this lesson is its functional equivalent and is the binding one. Advisor flagged the corpus drift for retro — same note as the impl-1 brief.)**
   - `feedback_phase_2_e2e_gate_enforcement.md` — **the phase-tip whole-binary `--test e2e` is THE gate** (Task 2's §5 validation). Never a per-test substitute. This is the headline NEGATIVE-test gate for the lane.
   - `feedback_pipes_mask_exit_codes.md` (always — capture-then-tail).
   - `feedback_clippy_test_style.md` (always for cargo work — `assert_eq!` only; no `.unwrap()`/`.expect()`/`dbg!`/`#[allow]` in the swaps).

## 3a. Handover from prior cohort

Task 1 (`834f9d85d` on this branch tip; finalize-merged, origin `197815f63`) shipped Sites A+B and PASSED all advisor-laptop gates (check/clippy/targeted-e2e: 2 passed; 0 failed). Verbatim HANDOVER from Task 1:

```yaml
prior_cohort_tasks:
  - task: 1
    commit: 834f9d85d
    filesCreated: []
    filesModified: ["crates/server/tests/e2e.rs"]
    keyDecisions:
      - "Site A: only L2949 rep_total moved (4->7); L2959 jury_rep_count and L2967 reporter_rep_count NOT touched (regression guard)"
      - "Site B: single-element insertion of \"vote_outcome_recorded\" between \"public_log_published\" and \"case_decided\"; \"evidence_quality_recorded\" NOT added (Source 4a does not fire on this fixture)"
    notes: "Task 2 forks from this branch tip; Sites C+D live in mod v1_sl_d_fixtures (declared at L13568). Task 1's Site B insert at ~L11209 shifted Sites C+D +1 line; current anchors are L14027 (C) + L14246 (D)."
```

## 4. Constraints

### Branch + commit discipline
- You start on a Junior worktree off `phase-v1-rt-r3-followup` (which already carries Task 1's commit). Finalize merges your worktree branch back; do NOT push to `phase-v1-rt-r3-followup` directly.
- **One commit.** If a §5 gate fails on the advisor-laptop side, the advisor surfaces — you do not split the commit.
- Mid-task DQ visibility: any `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"`/`"user"`; self-resolve only as `"impl-self-resolved"`. No `approved_by` (advisor-exclusive).

### Plan-cited line numbers may have drifted
The anchors in §2.1 are **text-based** (verbatim `old_string`), pre-located by the advisor against tip `197815f63`. Apply by text match, not line number. If an `old_string` does not match exactly (whitespace, message text), STOP and `rg -n "rep_count, 4"` to re-locate; if the assertion text itself differs from §2.1, file a `kind: "blocker"` DQ (the file changed since pre-locate).

### GOTCHA (regression guards — load-bearing)
- **Do NOT edit `assert_eq!(sponsor_rep_count, 0, ...)` at ~L14235.** It filters `reputation_event::person_id.eq_any(&sponsor_ids)`; sponsors get zero rows on the NoAction path because `compute_sponsor_liability` never runs. RT-r3's emit is for jurors + reporter, not sponsors. Stays 0. Editing it is a regression the §16a verify will catch.
- **Both `rep_count` queries are filterless `reputation_event::table.count()`** — the per-test grand total in an isolated fixture (the test's case is the only `case_id`). The +3 ParticipationConsistency from Source 3 lands in the total on BOTH paths: Site C (Decided/Remove) and Site D (NoAction Decided) each have 3 majority-aligned jurors. Source 3's gate is "juror_decision == winning_decision" (majority alignment), NOT "winning_decision is a remove" — so the NoAction path fires vote-outcome too. 3 ParticipationConsistency + 3 JuryReliability + 1 ReportingAccuracy = 7.
- **Do NOT add `"evidence_quality_recorded"`.** Source 4a requires a seeded `case_evidence` row + ≥256-char winning rationale; neither fixture seeds them.

### Lesson trailer (encouraged)
If you hit a footgun a future e2e-edit task would want to know, end the commit body with a `LESSON:` line (one discrete lesson, cite file:line).

### HANDOVER trailer (required — Task 3 retro pulls the delta)
End the commit body with (per plan §13 Task 2 HANDOVER block):
```
HANDOVER:
  filesCreated: []
  filesModified: ["crates/server/tests/e2e.rs"]
  keyDecisions:
    - Site C: single multi-line Edit covering comment + assertion; rep_count 4->7
    - Site D: single multi-line Edit covering comment + assertion; rep_count 4->7; sponsor_rep_count==0 NOT touched
  notes: Task 3 retro pulls baseline-vs-post-fix delta (115 passed/4 failed -> 119 passed/0 failed) from advisor-laptop validate-pending-laptop-e2e mutation log.
```

## 5. Validation gates (per plan §13 Task 2 VALIDATE + §15.4 — HEADLINE NEGATIVE-test gate)

**Shape-G suspended until 2026-06-01.** After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop-e2e"` DQ entry (`from: "impl"`, `branch: "phase-v1-rt-r3-followup"`, `phase_task: 2`, `result: null`, `log_slice: null`, `failed_commands: null`, `answer: null`, `answered_by: null`, `approved_by: null`, `approved_at: null`) with `commands[]` carrying the three commands below verbatim (JSON-escaped). Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`; append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment> --pending`. Commit + push the DQ entry. Then **STOP** — the advisor laptop runs these and mutates the entry.

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-rt-r3-followup-task2-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-rt-r3-followup-task2-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full > .claude/PRPs/debug/v1-rt-r3-followup-task2-e2e-full.log 2>&1 && echo E2E_EXIT_0 >> .claude/PRPs/debug/v1-rt-r3-followup-task2-e2e-full.log || echo E2E_EXIT_NONZERO >> .claude/PRPs/debug/v1-rt-r3-followup-task2-e2e-full.log"
```

EXPECT (advisor-laptop side): check exit 0; clippy exit 0; `E2E_EXIT_0` + **`test result: ok. 119 passed; 0 failed; 5 ignored`**.

**HEADLINE NEGATIVE-test gate (per plan §15.4 R3):** this is the WHOLE-binary `--test e2e` run (NO test-name filter) — the gate Task 1 could not run (Sites C+D were still red). Pre-fix baseline was `115 passed; 4 failed; 5 ignored`; post-fix MUST be `119 passed; 0 failed; 5 ignored`. **REGRESSION GATE:** if ANY of the 115 previously-green tests appears in `failures:`, that is a green-to-red flip → catch-fire (the advisor surfaces; do not attempt a fix).

## 6. Expected output (return to advisor)

```
## Task 2 complete — v1-rt-r3-followup Sites C+D

**Commit:** <sha> on <worktree-branch>
**Files changed:** crates/server/tests/e2e.rs (Edit 2.1 Site C rep_count 4→7 + comment @~14027; Edit 2.2 Site D rep_count 4→7 + comment @~14246)
**DQ raised:** validate-pending-laptop-e2e <id> (phase_task 2, awaiting advisor-laptop full e2e gate)
**Regression guards honored:** sponsor_rep_count==0 @~14235 untouched; no evidence_quality_recorded added
**Next:** advisor-laptop runs check/clippy/WHOLE-binary-e2e; on 119 passed/0 failed → Task 3 (retro)
```

## 7. Why this brief differs from the plan

Clean execution of plan §13 Task 2 + §10.3/§10.4 anchors — no overrides. Two notes (both non-blocking, same as impl-1): (a) the §2.1 anchors are pre-located against the POST-Task-1 tip (`197815f63`), so the line numbers are +1 vs the plan's pre-Task-1 mirror cites; the verbatim text is identical. (b) `feedback_junior_worker_e2e_edit_hang.md` (named in the §2.4 injection table) is absent from the corpus; `feedback_fix_impl_pre_locate_e2e_anchors.md` is the present functional equivalent and is the binding lesson (advisor flagged the drift for retro).
