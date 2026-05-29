---
role: impl-task
plan_task: fix-impl-1
phase: v1-rt-r3-followup
created: 2026-05-29
related_dq: null
related_pr: 162
related_findings: [cr-10, cr-11]
---

# Brief — v1-rt-r3-followup fix-impl-1 — CR #162 stale-comment fixes in e2e.rs (Sites A + B)

> **Origin:** CodeRabbit PR #162 findings cr-10 + cr-11, both bucketed `fix-in-pr` after advisor verified each against the actual code. These are **stale explanatory comments** adjacent to the assertions Task 1 changed — NOT assertion-value changes (those are correct and already validated by e2e 119/0). This is a comment-accuracy fix only.

## 0. Pre-flight (subagent runs this before reading anything else)

The `impl-task` agent runs the forbidden-window check from `.claude/agents/impl-task.md`. **Shape G SUSPENDED (DQ #229, until 2026-06-01): this lane runs ZERO cargo on the EliteDesk daemon.** After committing + pushing, you write a `kind: "validate-pending-laptop"` DQ entry and STOP — the advisor laptop runs the §5 cargo gates. Do NOT run cargo on the daemon; do NOT write `kind: "validate-pending"`.

## 1. Role + dispatch line

`[role:impl-task] v1-rt-r3-followup fix-impl-1 — see .claude/PRPs/briefs/v1-rt-r3-followup-fix-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). **TWO Edits, ONE file (`crates/server/tests/e2e.rs`), ONE commit.** Both edits are **comment-only** (no assertion, no code, no test logic changes). The assertions themselves are already correct (`rep_total, 7` and the `expected_prefix` vec) — do NOT touch them.

## 2. Scope

**Produce** (one commit):
- `crates/server/tests/e2e.rs` — **Edit 1** (Site A comment, ~line 2931): correct the stale "4 reputation_event rows ... NOT 3" comment to reflect the post-RT-r3 count of 7. **Edit 2** (Site B comment, ~line 11194): correct `public_log_published`'s stale positional description AND add the omitted `vote_outcome_recorded` kind to the enumeration.

**Do NOT** in this task:
- Touch ANY `assert_eq!`, any `let` binding, any diesel query, or any test logic. **Comment lines (`//`) only.**
- Touch `rep_total, 7` (@~2949) or the `expected_prefix` vec (@~11201-11214) — those are correct.
- Touch Sites C or D (already shipped + validated).
- Touch any file other than `crates/server/tests/e2e.rs` — scope violation → catch-fire.

**Commit message** (exactly): `test(governance): refresh stale Site A + Site B explanatory comments post-RT-r3 vote-outcome emit (CR #162 cr-10/cr-11)`

### 2.1 VERBATIM EDIT ANCHORS (pre-located by advisor against tip d00c6da69 — apply EXACTLY)

**Edit 1 — Site A comment `old_string`** (3-line comment block):

```rust
    // Drift #8: 4 reputation_event rows (3 jurors on JuryReliability + 1
    // reporter on ReportingAccuracy) per [05 §6] — NOT 3 as the plan
    // body suggests.
```

**Edit 1 — Site A comment `new_string`:**

```rust
    // Drift #8: 7 reputation_event rows (3 jurors on JuryReliability + 1
    // reporter on ReportingAccuracy + 3 ParticipationConsistency from the
    // RT-r3 vote-outcome emit, one per majority-aligned juror) per [05 §6].
```

**Edit 2 — Site B comment `old_string`** (3-line block — the `public_log_published` bullet):

```rust
  //   - public_log_published (between case_decided and appeal_requested) —
  //     redacted public log entry created on case-decide
  //     (Phase 4b shipped, submit_jury_vote.rs)
```

**Edit 2 — Site B comment `new_string`** (corrects the position + appends the vote_outcome_recorded bullet):

```rust
  //   - public_log_published (between jury_accepted and vote_outcome_recorded,
  //     i.e. BEFORE case_decided) — redacted public log entry created on
  //     case-decide (Phase 4b shipped, submit_jury_vote.rs)
  //   - vote_outcome_recorded (between public_log_published and case_decided) —
  //     RT-r3 (996765cae) per-vote outcome emit on submit_jury_vote; first
  //     occurrence is the decision-time write.
```

**Anchor-uniqueness:** both `old_string` blocks are distinctive (each carries its specific comment text). Run `rg -n "Drift #8: 4 reputation_event"` (must be 1 match) and `rg -n "between case_decided and appeal_requested"` (must be 1 match) BEFORE applying. If either matches more than once or zero times, STOP and file a `kind: "blocker"` DQ.

## 3. Required reading

In this order:
1. **MIRROR refs** — `crates/server/tests/e2e.rs:2925-2952` (Site A: the full comment block + the `rep_total, 7` assertion it describes) and `:11185-11220` (Site B: the comment enumeration + the `expected_prefix` vec it describes). Read these ±15-line windows ONLY. **NEVER bulk-read `e2e.rs` (18K lines).**
2. **Lessons (mandatory per `.claude/rules/advisor-orchestrator.md` §2.4 — e2e.rs edit, ≥2 edits):**
   - `feedback_fix_impl_pre_locate_e2e_anchors.md` — the verbatim-anchor + ≤2-Edits-per-file discipline (the binding e2e-edit lesson; `feedback_junior_worker_e2e_edit_hang.md` was promoted to the corpus this lane and reconciled — pre-located anchors avoid the hang).
   - `feedback_lemmy_error_no_std_error.md` — Case A (no signature changes here; comment-only, but the file's `LemmyResult<()>` shape is context).
   - `feedback_pipes_mask_exit_codes.md` (always — capture-then-tail).
   - `feedback_clippy_test_style.md` (always for cargo work — but no code change here, so clippy is just a regression guard).

## 3a. Handover from prior cohort

Tasks 1+2 shipped Sites A+B+C+D assertion-value fixes (e2e 119/0). This fix-impl touches ONLY the explanatory comments at Sites A + B that Task 1 left stale (the assertion values moved 4→7 but their describing comments still said "4 rows" / wrong governance_log ordering). No assertion or vec is touched here.

## 4. Constraints

### Branch + commit discipline
- You start on a Junior worktree off `phase-v1-rt-r3-followup` (tip `d00c6da69`, carries Tasks 1+2 + the PR). Finalize merges your worktree branch back; do NOT push to `phase-v1-rt-r3-followup` directly.
- **One commit. Comment-only.** If a §5 gate fails, the advisor surfaces — you do not split.
- Mid-task DQ visibility: any `pending` entry → commit + push immediately.
- No `answered_by: "advisor"`/`"user"`; self-resolve only as `"impl-self-resolved"`. No `approved_by` (advisor-exclusive).

### Plan-cited line numbers may have drifted
Anchors in §2.1 are text-based (verbatim `old_string`). Apply by text match. If an `old_string` does not match exactly, STOP and `rg -n` to re-locate; if the comment text itself differs from §2.1, file a `kind: "blocker"` DQ.

### GOTCHA (do not over-reach)
- **Comment-only.** The `assert_eq!(rep_total, 7, ...)` and the `expected_prefix` vec are CORRECT (e2e 119/0 confirms). You are fixing the human-readable `//` comments that describe them, nothing else.
- **Site B position correctness:** the `expected_prefix` vec orders `public_log_published → vote_outcome_recorded → case_decided`. The new comment must say `public_log_published` is BEFORE `case_decided` (the old "between case_decided and appeal_requested" was wrong). Verify against the vec at `:11201-11214` before applying.
- **Do NOT touch the idempotency-guard clause** in Site A's comment (the "would be 14 ... double-penalising sponsors" lines 2935-2942) — that is still correct (it's about exactly-once semantics, not the base count). Edit 1 only touches the first 3 lines (the "Drift #8: 4 ... NOT 3" block).

### HANDOVER trailer (required)
End the commit body with:
```
HANDOVER:
  filesCreated: []
  filesModified: ["crates/server/tests/e2e.rs"]
  keyDecisions:
    - Comment-only fix; assertions (rep_total,7 + expected_prefix vec) untouched
    - Site A comment 4->7 with +3 ParticipationConsistency clause; idempotency-guard clause preserved
    - Site B comment: public_log_published repositioned (before case_decided) + vote_outcome_recorded bullet added
  notes: Addresses CR #162 cr-10 (e2e.rs:2931) + cr-11 (e2e.rs:11194). No assertion/vec/test-logic change.
```

## 5. Validation gates

**Shape-G suspended until 2026-06-01.** After committing + pushing your worktree branch, write a `kind: "validate-pending-laptop"` DQ entry (`from: "impl"`, `branch: "phase-v1-rt-r3-followup"`, `phase_task: "fix-impl-1"`, `result: null`, `log_slice: null`, `failed_commands: null`, `answer: null`, `answered_by: null`, `approved_by: null`, `approved_at: null`) with `commands[]` carrying the three commands below verbatim. Generate the id via `bash scripts/brehon/dq-v3-new-entry.sh`; append via `bash scripts/brehon/dq-v3-append-fragment.sh <fragment> --pending`. Commit + push the DQ entry. Then **STOP**.

```bash
cmd //c "scripts\\brehon\\cargo-check.bat --workspace --features full > .claude/PRPs/debug/v1-rt-r3-followup-fiximpl1-check.log 2>&1"
cmd //c "scripts\\brehon\\cargo-clippy.bat --workspace --features full --no-deps -- -D warnings > .claude/PRPs/debug/v1-rt-r3-followup-fiximpl1-clippy.log 2>&1"
cmd //c "scripts\\brehon\\cargo-test.bat --workspace --test e2e --features full -- report_to_modlog_golden_path governance_log_sequence_matches_prd_state_machine > .claude/PRPs/debug/v1-rt-r3-followup-fiximpl1-e2e.log 2>&1 && echo TEST_EXIT_0 >> .claude/PRPs/debug/v1-rt-r3-followup-fiximpl1-e2e.log || echo TEST_EXIT_NONZERO >> .claude/PRPs/debug/v1-rt-r3-followup-fiximpl1-e2e.log"
```

EXPECT (advisor-laptop side): check exit 0; clippy exit 0; `TEST_EXIT_0` + `test result: ok. 2 passed; 0 failed`. (Comment-only change — the 2 Site A/B tests stay green; the full 119/0 already held at Task 2 and a comment edit cannot change runtime behaviour, so the targeted 2-test run is sufficient regression coverage. The advisor may opt to re-run the whole binary if it judges the comment edit warrants it.)

## 6. Expected output (return to advisor)

```
## fix-impl-1 complete — v1-rt-r3-followup CR #162 stale-comment fixes

**Commit:** <sha> on <worktree-branch>
**Files changed:** crates/server/tests/e2e.rs (Edit 1 Site A comment 4→7 @~2931; Edit 2 Site B comment reposition + vote_outcome_recorded bullet @~11194)
**DQ raised:** validate-pending-laptop <id> (phase_task fix-impl-1)
**Assertions untouched:** rep_total,7 + expected_prefix vec confirmed unchanged
**Next:** advisor-laptop runs check/clippy/targeted-e2e → re-poll CR → bucket cr-10/cr-11 done
```

## 7. Why this brief exists

CR #162 caught that Task 1 updated the Site A/B assertion VALUES (4→7 rep counts + vote_outcome_recorded insertion) but left their explanatory `//` comments describing the pre-RT-r3 state. Both findings verified real by advisor reading the code. Non-allowlist (hand-authored recipe — comment-accuracy, not a mechanical lint auto-fix). Fixing stale comments adjacent to the values they describe is exactly the doc-rot-prevention the lane's root-cause lesson (`feedback_emit_added_requires_full_e2e_gate.md`) is about.
