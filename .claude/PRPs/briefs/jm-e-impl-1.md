---
role: impl-task
plan_task: 1
phase: v1-JM-e
created: 2026-05-01
related_dq: 99
---

# Brief — v1-JM-e Task 1 — Appeal-vote tally branch in submit_jury_vote + step_up_token DTO slot

## 1. Role + dispatch line

`[role:impl-task] v1-JM-e task 1 — see .claude/PRPs/briefs/jm-e-impl-1.md`

You are the **impl-task** subagent (Sonnet 4.6). Execute plan Task 1 from `.claude/PRPs/plans/v1-jury-mechanics-e.plan.md` §13.

**Shape G (Layer G2 push-and-exit):** do NOT run cargo locally. After your commit, push to origin and write a `kind: "validate-pending"` DQ entry referencing the `cargo-validate-workspace.yml` run id. The advisor dispatches a ci-watcher to poll the workflow.

## 2. Scope

**Produce** (one commit):

- `crates/api/api/src/governance/submit_jury_vote.rs` — extend step-1 assignment query to return `role`; insert step-2.5 dispatch branch; add `process_appeal_vote` private fn (appeal row load, per-decision tally, deadlock branch, appeal-decide UPDATE batch + governance_log emission).
- `crates/api/api_common/src/governance.rs` — add `step_up_token: Option<String>` field on `AdminTriggerAppealRejury` struct.

**Do NOT** in this task:
- Touch `crates/server/tests/e2e.rs` (Tasks 2-5).
- Touch any migration files (JM-e ships no schema changes).
- Flip `(pending)` → `(active)` in `.claude/rules/governance-log-entry-kind-registry.md` (Task 6).
- Run any cargo command locally — Shape G; push and exit only.

**Commit message** (exactly): `feat(v1-JM-e): appeal-vote tally branch in submit_jury_vote + step_up_token DTO slot (task 1)`

## 3. Required reading

Read in this order before editing any file:

1. **`.claude/decision-queue.json` resolved entry DQ #99** — planner-resolved decision on `step_up_token` DTO site (`AdminTriggerAppealRejury`). Confirms field goes on that struct, v1 ignore-behaviour.
2. **Plan §13 Task 1** — canonical step list + all GOTCHAs.
3. **Plan §4.1** — architectural decisions locked for this sub-phase (appeal-tally snapshot discipline, no-sanction-row, no-federation, no `case_decided` re-emit).
4. **Plan §10.1** — branch-detection extension pattern (MIRROR source).
5. **Plan §10.2** — appeal row load + snapshot reads (MIRROR source).
6. **Plan §10.3** — appeal-side per-decision tally (MIRROR source).
7. **Plan §10.4** — appeal-deadlock branch (MIRROR source).
8. **Plan §10.5** — appeal-decide UPDATE batch + governance_log emission (MIRROR source).
9. **Plan §10.6** — step_up_token DTO field addition (MIRROR source).
10. **Plan §4.2 watchpoints** — specific files/lines to watch (especially watchpoints 1-8).
11. **MIRROR refs** (open and read these files before editing):
    - `crates/api/api/src/governance/submit_jury_vote.rs` (entire file, 724 lines) — primary edit site.
    - `crates/api/api_common/src/governance.rs` — locate `AdminTriggerAppealRejury`.
    - `crates/db_schema/src/source/governance/appeal.rs` (53 lines) — Appeal row shape.
    - `crates/db_schema/src/source/governance/governance_log.rs:170-184` — entry-kind block.
    - `crates/db_schema_file/src/schema.rs:155-166` (appeal table), `:524-534` (jury_assignment), `:772-798` (moderation_case).
12. **Lessons** (Glob `.claude/lessons/`, read any matching these keywords):
    - `feedback_clippy_test_style.md` — LemmyResult<()> + no unwrap/expect; workspace denies escape-hatches.
    - `feedback_pipes_mask_exit_codes.md` — capture-then-tail rule (applies if you run any diagnostic command).
    - `feedback_insertform_default_propagation.md` — struct literal sites; R3 sweep discipline.
    - `feedback_clippy_rerun_after_fix.md` — if dispatch refactor unmasks an unused-imports lint, re-run clippy before push (JM-d retro §3.5).
    - `feedback_rust_visibility_cross_crate.md` — if new types need re-exporting.
    - `feedback_features_full_p_crate_incompatible.md` — never `-p <crate>` + `--features full`.

## 3a. Handover from prior cohort

(none — first cohort; Task 0 produced no commit)

## 4. Constraints

### Branch + commit discipline
- You start on a Junior worktree branched off `phase-v1-JM-e` (tip `ebb34bb41`).
- One commit. If you need to amend after a local check, amend the single commit before push.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` or `"user"` from this subagent.

### Shape G push-and-exit
After committing, push to `origin/<your-worktree-branch>` and capture the `cargo-validate-workspace.yml` workflow run id:

```bash
git push origin HEAD
# Wait ~10 seconds for GH Actions to trigger, then:
gh run list --repo barrie-cork/lemmy --branch <your-branch> \
  --workflow cargo-validate-workspace.yml --limit 1 \
  --json databaseId,status --jq '.[0]'
```

Write a `kind: "validate-pending"` DQ entry in `.claude/decision-queue.json`:

```json
{
  "id": <next_id>,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<UTC ISO 8601>",
  "question": "workspace-check for v1-JM-e task 1",
  "options": ["pass", "fail"],
  "context": "cargo-validate-workspace.yml triggered on push to <branch>",
  "workflow_run_id": <databaseId>,
  "branch": "<your-worktree-branch>",
  "phase_task": 1,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "resolved_at": null
}
```

Commit the DQ entry: `chore(decision-queue): impl raised DQ #<id> — validate-pending task 1`
Push that commit too.

### Critical GOTCHAs (from plan §13 Task 1)

**GOTCHA (idempotency guard for Appealed — LOAD-BEARING):** The existing JM-c idempotency guard at `submit_jury_vote.rs:242-255` lists `Appealed` as a terminal-status early-return. The role-branch dispatch MUST fire BEFORE the idempotency guard. Place the step-2.5 dispatch IMMEDIATELY after the step-2 case-load (line ~184) and BEFORE the `matches!` guard. Inside `process_appeal_vote`, the guard is narrower: only `Closed | EmergencyRemove | AdminReview` are terminal; `Appealed` is the EXPECTED status.

**GOTCHA (R1 — i32 ↔ i64):** every `i32 ↔ i64` comparison in the tally uses `i64::from(...)`. NEVER bare `as` cast. Specifically: `appeal_threshold_count_i64 = i64::from(appeal_threshold_count)`.

**GOTCHA (R3 — struct literal sweep):** after adding `step_up_token` to `AdminTriggerAppealRejury`, run:
```bash
git grep -l 'AdminTriggerAppealRejury {'
```
For every literal-construction site found, add `..Default::default()` or propagate the field. Per `feedback_insertform_default_propagation.md`.

**GOTCHA (R7 — DTO struct change):** the `AdminTriggerAppealRejury` change affects compile across re-exports. The workspace-check workflow's `cargo test --no-run` step will catch failures — push and let ci-watcher report rather than checking locally.

**GOTCHA (run_transaction inheritance):** the wrapper at `submit_jury_vote.rs:118-125` wraps `process_vote`; when dispatched to `process_appeal_vote`, the inner fn inherits the same transaction. Do NOT open a new transaction inside `process_appeal_vote`.

**GOTCHA (v0 literal preserved):** the `"jury_vote_submitted"` literal at `submit_jury_vote.rs:216` is preserved verbatim — do NOT clean it up (JM-c v0-literal cleanup is out of JM-e scope per plan §12).

**GOTCHA (appeal_decided vs case_decided):** `ENTRY_KIND_APPEAL_DECIDED` fires; `ENTRY_KIND_CASE_DECIDED` does NOT fire again. The original `case_decided` already fired at original-jury decision time.

**GOTCHA (no sanction row):** do NOT insert a `sanction` row on appeal verdict. Per PRD §6.7 v1 simplification — appeal verdict is audit-only.

**GOTCHA (no federation):** do NOT emit any ActivityPub outbound on `appeal_decided`. Per ADR-014.

**GOTCHA (clippy rerun):** if the dispatch refactor unmasks an `unused-mut` or `unused-imports` lint (per `feedback_clippy_rerun_after_fix.md`), fix before push — otherwise the Phase 1 ci-watcher will surface it and require a fix-impl round-trip.

### Plan-cited line numbers may have drifted
The plan was written before this task. Verify all cited line numbers by `rg -n` before editing. If counts of expected sites differ, file a DQ pending entry before proceeding.

### Lesson trailer (encouraged)
If you discover something a future impl-task on related handler code would want to know, end the commit body with a `LESSON:` line per `feedback_junior_pmd_write_convention.md`.

## 5. Validation gate (Shape G)

**No local cargo.** Validation is:
1. Push to `origin/<worktree-branch>`.
2. Confirm `cargo-validate-workspace.yml` triggered (via `gh run list`).
3. Write `kind: "validate-pending"` DQ entry with `workflow_run_id`.
4. Commit + push DQ entry.

The advisor's ci-watcher will poll the workflow and mutate the DQ entry to `result: "pass" | "fail"`.

If `gh run list` returns no run after 30 seconds, check `gh run list --limit 5 --repo barrie-cork/lemmy` for any recent run on your branch. If still nothing, write a DQ blocker entry — do not guess.

## 6. Expected output (return to advisor)

```
## Task 1 complete — v1-JM-e appeal-vote tally branch + step_up_token DTO

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/src/governance/submit_jury_vote.rs (+process_appeal_vote fn, step-2.5 dispatch, extended step-1 query)
  - crates/api/api_common/src/governance.rs (+step_up_token field on AdminTriggerAppealRejury)
**DQ raised:** #<id> kind: validate-pending, workflow_run_id: <id>, branch: <branch>
**R3 sweep:** <found N literal sites / all propagated with ..Default::default()>
**Next:** advisor queues ci-watcher for DQ #<id>, then on pass queues Task 2 (e2e capstone test)
```

Plus any additional DQ entries if you raised a blocker mid-task.
