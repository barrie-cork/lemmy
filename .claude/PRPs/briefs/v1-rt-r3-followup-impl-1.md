---
role: impl-task
plan_task: 1
phase: v1-rt-r3-followup
created: 2026-05-29
related_dq: null
---

# Brief — v1-rt-r3-followup Task 1 — Sites A+B: rep_total 4→7 + expected_prefix insert vote_outcome_recorded

> **Clarify provenance:** parent planning brief clarified via `/brehon-clarify` (3 advisor-self-resolved entries). No clarify-DQ gates an impl-task.

## 0. Pre-flight (subagent runs this before reading anything else)

The `impl-task` agent runs the forbidden-window check from `.claude/agents/impl-task.md`. **Shape G SUSPENDED (DQ #229, until 2026-06-01): this lane runs ZERO cargo on the EliteDesk daemon.** After committing + pushing, you write a `kind: "validate-pending-laptop"` DQ entry and STOP — the advisor laptop runs the §5 cargo gates. Do NOT run cargo on the daemon; do NOT write `kind: "validate-pending"`; do NOT capture a `workflow_run_id`.

## 1. Role + dispatch line

`[role:impl-task] v1-rt-r3-followup task 1 — see .claude/PRPs/briefs/v1-rt-r3-followup-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan **Task 1** from `.claude/PRPs/plans/v1-rt-r3-followup.plan.md` §13 "Task 1" + §10.1 (Site A anchor) + §10.2 (Site B anchor). **TWO Edits, ONE file (`crates/server/tests/e2e.rs`), ONE commit.**

## 2. Scope

**Produce** (one commit):
- `crates/server/tests/e2e.rs` — **Edit 1.1** (Site A, line 2949): `rep_total` assertion `4` → `7`. **Edit 1.2** (Site B, lines 11208–11209): insert `"vote_outcome_recorded",` into the `expected_prefix` vec between `"public_log_published",` and `"case_decided",`.

Both edits are **pure value swaps inside an already-correct error shape** (`LemmyResult<()>` Case A — verified: `report_to_modlog_golden_path` @2485 and `governance_log_sequence_matches_prd_state_machine` @11054 both return `lemmy_utils::error::LemmyResult<()>`). No signature changes, no `.map_err` bridges, no helper extraction.

**Do NOT** in this task:
- Edit Sites C or D (Task 2 — `mod v1_sl_d_fixtures`).
- Edit `crates/server/tests/e2e.rs:2959` (`jury_rep_count == 3`) or `:2967` (`reporter_rep_count == 1`) — see GOTCHA below; editing them is a **regression**.
- Add `"evidence_quality_recorded"` to Site B's vec — see GOTCHA below; Source 4a does not fire on this fixture.
- Touch any file under `crates/api/**`, `crates/db_schema/**`, `crates/routes/**`, `migrations/**`, `Cargo.*` — **scope violation → catch-fire** (the emit paths are correct + merged in RT-r3; this is a test-only fix).

**Commit message** (exactly): `test(governance): bring report_to_modlog + governance_log_sequence assertions current with RT-r3 vote-outcome emit (task 1)`

### 2.1 VERBATIM EDIT ANCHORS (pre-located per `feedback_fix_impl_pre_locate_e2e_anchors.md` — apply EXACTLY, do NOT "search near line N")

**Edit 1.1 — Site A `old_string`** (4-line `assert_eq!` block):

```rust
    assert_eq!(
      rep_total, 4,
      "4 reputation_event rows (exactly-once under late votes)"
    );
```

**Edit 1.1 — Site A `new_string`:**

```rust
    assert_eq!(
      rep_total, 7,
      "7 reputation_event rows (4 prior + 3 ParticipationConsistency from RT-r3 vote-outcome emit; exactly-once under late votes)"
    );
```

**Edit 1.2 — Site B `old_string`** (2-line block — minimal unique anchor pair):

```rust
    "public_log_published",
    "case_decided",
```

**Edit 1.2 — Site B `new_string`:**

```rust
    "public_log_published",
    "vote_outcome_recorded",
    "case_decided",
```

**Site B anchor-uniqueness check BEFORE applying Edit 1.2:** run `rg -n '"public_log_published",' crates/server/tests/e2e.rs`. It MUST return exactly one occurrence (line ~11208). If a second occurrence appears, expand `old_string` downward to include `\n    "appeal_requested",` for disambiguation (per plan §10.2 anchor-uniqueness check), then re-apply. If you cannot make the anchor unique, STOP + file a `kind: "blocker"` DQ.

## 3. Required reading

In this order:
1. **Plan §10.1** (Site A anchor + the regression GOTCHA at lines 212–214) + **§10.2** (Site B anchor + the "do NOT add evidence_quality_recorded" GOTCHA at line 235).
2. **Plan §13 Task 1** (the IMPLEMENT / MIRROR / GOTCHA / VALIDATE block, lines 423–481).
3. **MIRROR refs** — `crates/server/tests/e2e.rs:2943-2967` (Site A surrounding context — the sequential filtered counts that MUST stay unedited at 2959, 2967) and `:11201-11218` (Site B `expected_prefix` vec + panic message). Read these ±50-line windows ONLY. **NEVER bulk-read `e2e.rs` (18,089 lines)** — memory-headroom rule.
4. **Lessons (mandatory per `.claude/rules/advisor-orchestrator.md` §2.4 file-class injection — e2e.rs edit, ≥2 edits):**
   - `feedback_lemmy_error_no_std_error.md` — **Case A** (sibling fixtures use `LemmyResult<()>` outer; pure value swaps, no `.map_err`). Both target fns confirmed `LemmyResult<()>`.
   - `feedback_async_pool_test_pattern.md` — pool/conn fixture pattern (consistency reference; no new helper expected).
   - `feedback_fix_impl_pre_locate_e2e_anchors.md` — the verbatim-anchor + ≤2-Edits-per-file discipline. **(This is the present canonical e2e-edit-hang-prevention lesson. The §2.4 table also names `feedback_junior_worker_e2e_edit_hang.md`, which is ABSENT from the lessons corpus — this lesson is its functional equivalent and is the binding one. Advisor flagged the corpus drift for retro.)**
   - `feedback_pipes_mask_exit_codes.md` (always — capture-then-tail).
   - `feedback_clippy_test_style.md` (always for cargo work — `assert_eq!` only; no `.unwrap()`/`.expect()`/`dbg!`/`#[allow]` in the swaps).

## 3a. Handover from prior cohort

(none — Task 0 was verification-only with no commit; first impl task)

## 4. Constraints

### Branch + commit discipline
- You start on a Junior worktree off `phase-v1-rt-r3-followup`. Finalize merges your worktree branch back; do NOT push to `phase-v1-rt-r3-followup` directly.
- **One commit.** If a §5 gate fails on the advisor-laptop side, the advisor surfaces — you do not split the commit.
- Mid-task DQ visibility: any `pending` entry → **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"`/`"user"`; self-resolve only as `"impl-self-resolved"`. No `approved_by` (advisor-exclusive).

### Plan-cited line numbers may have drifted
The anchors in §2.1 are **text-based** (verbatim `old_string`), not line-pinned — apply by text match, not line number. If the `old_string` does not match exactly (whitespace, message text), STOP and `grep -n` to re-locate; if the assertion text itself differs from §2.1, file a `kind: "blocker"` DQ (the file changed since plan-write).

### GOTCHA (regression guards — load-bearing)
- **Do NOT edit lines 2959 / 2967.** `jury_rep_count == 3` filters `ReputationDimension::JuryReliability`; `reporter_rep_count == 1` filters `::ReportingAccuracy`. The RT-r3 +3 lands on `ParticipationConsistency` — a THIRD dimension neither filter counts. Both stay 3 and 1. Editing either is a regression that the §16a Story 1 verify-check will catch.
- **Do NOT add `"evidence_quality_recorded"` to the Site B vec.** Source 4a (evidence-cited) requires a seeded `case_evidence` row + ≥256-char winning rationale; the `governance_log_sequence_matches_prd_state_machine` fixture seeds neither. The baseline panic confirms `got` contains only `vote_outcome_recorded` between `public_log_published` and `case_decided`. **Single-element insertion.**

### Lesson trailer (encouraged)
If you hit a footgun a future e2e-edit task would want to know, end the commit body with a `LESSON:` line (one discrete lesson, cite file:line).

### HANDOVER trailer (required — Task 2 forks from this tip)
End the commit body with (per plan §13 Task 1 HANDOVER block):
```
HANDOVER:
  filesCreated: []
  filesModified: ["crates/server/tests/e2e.rs"]
  keyDecisions:
    - Site A: only L2949 rep_total moved (4->7); L2959 jury_rep_count and L2967 reporter_rep_count NOT touched (regression guard)
    - Site B: single-element insertion of "vote_outcome_recorded" between "public_log_published" and "case_decided"; "evidence_quality_recorded" NOT added (Source 4a does not fire on this fixture)
  notes: Task 2 forks from this branch tip; Sites C+D live in mod v1_sl_d_fixtures (declared at L13567).
```

## 5. Validation gates (per plan §13 Task 1 VALIDATE + §15)

**Shape-G suspended until 2026-06-01.** After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `branch: "phase-v1-rt-r3-followup"`, `phase_task: 1`, `result: null`, `log_slice: null`, `failed_commands: null`, `answered_by: null`, `approved_by: null`, `approved_at: null`) with `commands[]` carrying the three commands below verbatim (JSON-escaped). Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`; append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment> --pending`. Commit + push the DQ entry. Then **STOP** — the advisor laptop runs these and mutates the entry.

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-rt-r3-followup-task1-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-rt-r3-followup-task1-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- report_to_modlog_golden_path governance_log_sequence_matches_prd_state_machine > .claude/PRPs/debug/v1-rt-r3-followup-task1-sites-ab.log 2>&1 && echo TEST_EXIT_0 >> .claude/PRPs/debug/v1-rt-r3-followup-task1-sites-ab.log || echo TEST_EXIT_NONZERO >> .claude/PRPs/debug/v1-rt-r3-followup-task1-sites-ab.log"
```

EXPECT (advisor-laptop side): check exit 0; clippy exit 0; `TEST_EXIT_0` + `test result: ok. 2 passed; 0 failed`. (NOTE: the full `--test e2e` whole-binary gate is Task 2's — after Task 1, Sites C+D are still red, so Task 1 uses the targeted 2-test run only.)

## 6. Expected output (return to advisor)

```
## Task 1 complete — v1-rt-r3-followup Sites A+B

**Commit:** <sha> on <worktree-branch>
**Files changed:** crates/server/tests/e2e.rs (Edit 1.1 rep_total 4→7 @2949; Edit 1.2 vote_outcome_recorded insert @11208-11210)
**DQ raised:** validate-pending-laptop <id> (phase_task 1, awaiting advisor-laptop cargo)
**Regression guards honored:** L2959/L2967 untouched; no evidence_quality_recorded added
**Next:** advisor-laptop runs check/clippy/targeted-e2e; on pass → Task 2 (Sites C+D)
```

## 7. Why this brief differs from the plan

Clean execution of plan §13 Task 1 + §10.1/§10.2 anchors — no overrides. The only note: `feedback_junior_worker_e2e_edit_hang.md` (named in the §2.4 injection table) is absent from the corpus; `feedback_fix_impl_pre_locate_e2e_anchors.md` is the present functional equivalent and is the binding lesson (advisor flagged the drift for retro). Non-blocking.
