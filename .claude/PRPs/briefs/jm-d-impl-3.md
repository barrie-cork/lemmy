---
role: impl-task
plan_task: 3
phase: v1-JM-d
created: 2026-04-28
status: ready
related_dq: null
---

# Brief — v1-JM-d Task 3 — Bounded-window appeal + reporter-rights + auto-rejury + winning_decision write

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d task 3 — see .claude/PRPs/briefs/jm-d-impl-3.md`

You are the **impl-task** subagent (Sonnet 4.6 per your frontmatter and the homeserver four-role tiering patch verified 2026-04-28). Execute plan task 3 from `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §13 (lines 1370-1538).

## 2. Scope

**Produce** (one commit per the plan's "one commit per task" rule):

**Part A — `crates/api/api/src/governance/submit_jury_vote.rs`:**
- Append `moderation_case::winning_decision.eq(Some(winning_decision))` to the post-decision UPDATE block (the one that sets `status = Decided` + `decided_at`). NOT the appeal-window UPDATE — they are intentionally separate per JM-c §10.5.

**Part B — `crates/api/api_crud/src/governance/request_appeal.rs`:**
- Replace the `within_window` line (plan cites line 112; **grep first to confirm**) with:
  `case.appeal_window_expires_at.map(|c| c > Utc::now()).unwrap_or(false)`
- Add the reporter-eligibility branch:
  - `case.target_person_id == Some(caller_id)` → `requester_role = AppealRequesterRole::Defendant`
  - else `case.creator_id == Some(caller_id) && matches!(case.winning_decision, Some(JuryDecision::NoAction | JuryDecision::AdvisoryLabel))` → `requester_role = AppealRequesterRole::OriginalReporter`
  - else → `Err(LemmyErrorType::NotFound.into())`
- Set `AppealInsertForm.requester_role: Some(requester_role)` at the insert call. `..Default::default()` for the snapshot fields (filled via the panel-seating UPDATE below).
- After the case-status flip to `Appealed`, branch on `appeal.auto_select_on_appeal_acceptance` config:
  - `true` → call `select_appeal_panel` (Part C); insert N rows in `jury_assignment` with `role = Some(JuryAssignmentRole::Appeal)` and `selected_under_constraints = Some(constraint_record.to_json())`; emit ONE `appeal_panel_assembled` governance_log entry (per the existing `admin_assign_jury.rs:264-284` audit pattern — one extended emission per panel, not one per row); UPDATE the new Appeal row with `panel_size_snapshot` + `threshold_count_snapshot`.
  - `false` → no panel seating; case sits in Appealed waiting for `admin_trigger_appeal_rejury` (Task 4).

**Part C — new helper in `crates/api/api/src/governance/admin_assign_jury.rs`:**
- Add `pub(crate) async fn select_appeal_panel(...)` alongside `select_eligible_jurors` (plan cites lines 456+; grep to confirm).
- Mirror `select_eligible_jurors` signature + return-tuple shape; use existing `compute_status_tier`, `ceil_count`, `severity_tier_slug` helpers.
- Add `pub(crate) struct AppealPanelSelection` with fields per plan §13 Task 3 Part C verbatim.
- Full body shape provided in the plan; treat the plan code block as canonical.

**Do NOT** in this task:
- Touch `admin_trigger_appeal_rejury` (Task 4 — does not exist yet).
- Touch the `appeal_window_expiry` background job (Task 5).
- Author any e2e tests or fixtures (Task 6).
- Touch migrations (Task 1, already shipped).
- Touch `request_appeal.rs:122-128` AppealInsertForm site beyond the rewrite scope above — the rewrite IS the touch.

**Commit message** (exactly):
`feat(v1-JM-d): bounded-window appeal + reporter-rights + auto-rejury + winning_decision write (task 3)`

## 3. Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries**:
   - **#73** (validate-pending → resolved 2026-04-28) — Task 2's workflow run + branch. Establishes the Shape G push-and-exit pattern your validation gate inherits. Note: this is the historical two-entry shape (paired with deprecated #74 below); option 2 (PMD #156, locked 2026-04-28) replaces this with single-entry mutation. Your validate-pending entry under §5 follows the new shape.
   - **#74** (validate-result → resolved 2026-04-28) — DEPRECATED kind. Confirms run 25025616075 passed; you are building on a known-good baseline. Do NOT write a new `validate-result` or `validate-failed` entry under any circumstance — those kinds were retired by option 2. ci-watcher mutates the paired `validate-pending` entry in place going forward.
   - **#71** (Task 2 schema regen override — `diesel print-schema` not `lemmy_diesel_utils -- print-schema`). You won't regen schema, but read it to understand the diesel toolchain layout.
2. **Plan §13 Task 3 (lines 1370-1538)** — the canonical step list. Execute Parts A/B/C verbatim. The four GOTCHAs in this section are load-bearing — read them all.
3. **Plan §10.2 + §10.3 + §10.5 + §10.6** — exact field/enum shapes. The schema is already in place from Task 2; you are *consuming* it.
4. **PRD §6.1 + §6.2 + §6.3 + §6.6** — appeal-panel sizing, original-juror exclusion, threshold cascade. The plan's `select_appeal_panel` body cites these by section.
5. **MIRROR refs** — read each before writing the corresponding code:
   - `crates/api/api/src/governance/admin_assign_jury.rs:264-284` — the canonical "one governance_log emission per panel" pattern (mirror exactly for `appeal_panel_assembled`)
   - `crates/api/api/src/governance/admin_assign_jury.rs:456+` — `select_eligible_jurors` signature + return shape (mirror for `select_appeal_panel`)
   - `crates/api/api_crud/src/governance/request_appeal.rs:65` — existing `process_appeal` `run_transaction` wrapper; your auto-rejury branch lives INSIDE this transaction
   - `crates/api/api/src/community/ban.rs:59-64` — canonical `run_transaction` shape (cited in `feedback_multi_write_handlers_need_transactions.md`)
6. **Lessons** (Glob `.claude/lessons/`, Read each):
   - `feedback_multi_write_handlers_need_transactions.md` — **load-bearing**. request_appeal does Appeal INSERT + jury_assignment INSERTs + moderation_case UPDATE + governance_log emission; the existing `process_appeal` already wraps in `run_transaction` (line 65) — keep your auto-rejury branch INSIDE that wrapper.
   - `feedback_insertform_default_propagation.md` — your `AppealInsertForm` insert sets `requester_role: Some(...)` explicitly; let `..Default::default()` cover `panel_size_snapshot` + `threshold_count_snapshot` (filled via UPDATE later).
   - `feedback_features_full_p_crate_incompatible.md` — plan §15 DoD uses `--workspace --features full`; never combine with `-p <crate>`.
   - `feedback_clippy_test_style.md` — workspace clippy denies `expect_used`/`unwrap_used`/`allow_attributes` even in tests; use `?` propagation.
   - `feedback_lemmy_error_no_std_error.md` — `LemmyResult` doesn't `?`-convert into `Box<dyn Error>`; use `.map_err(|e| format!("{e}").into())` if you need the conversion in any test you happen to read.
   - `feedback_pipes_mask_exit_codes.md` — capture-then-tail rule when you push and check workflow runs.
   - `feedback_pq_sys_wrapper_env_propagation.md` — only relevant if validation gate fails; cargo `pq-sys` cache can mask libpq issues.

## 4. Constraints

### Plan-cited line numbers may have drifted

Task 2 already committed to this branch; line numbers in `request_appeal.rs`, `submit_jury_vote.rs`, and `admin_assign_jury.rs` may have shifted. **Verify by grep before editing:**

```bash
grep -n "within_window" crates/api/api_crud/src/governance/request_appeal.rs
grep -n "moderation_case::status.eq(CaseStatus::Decided)" crates/api/api/src/governance/submit_jury_vote.rs
grep -n "select_eligible_jurors" crates/api/api/src/governance/admin_assign_jury.rs
grep -n "panel_assembled\|panel\\.assembled" crates/api/api/src/governance/admin_assign_jury.rs
```

If line numbers differ from plan §13, follow the grep output, not the plan. If a search returns ZERO hits, file a DQ pending entry — the schema may have drifted post-Task-2 in a way the brief doesn't anticipate.

### Multi-write transaction discipline (load-bearing)

`request_appeal.rs` already wraps its body in `run_transaction` (line 65 per plan). Your **auto-rejury branch must live inside that same transaction** so failed panel selection rolls back the appeal-row insert. Per plan §13 GOTCHA:

> If panel seating fails (e.g. small pool), the appeal row inserts ARE rolled back per the existing `process_appeal` transaction wrapping. This is desired — failed panel selection means the appeal cannot proceed; both operations succeed atomically or both fail.

Do NOT split into two transactions. Do NOT use a separate `context.pool()` call inside the auto-rejury branch.

### `panel_size_snapshot.ok_or_else` defensive guard (plan GOTCHA)

`case.panel_size_snapshot` is `Option<i32>`. Decided cases ALWAYS have a populated snapshot by construction (status-guard at `request_appeal.rs:93-105` rejects pre-JurySelection statuses). The `ok_or_else` returning `LemmyErrorType::Unknown` is the defensive guard — keep it; do not assert/unwrap.

### Lock-ordering (plan GOTCHA — DQ #50 / JM-c retro §3.2 amendment 1)

The flow is sequential within one transaction:
1. read case (line 86)
2. INSERT appeal (FK SHARE on parent moderation_case)
3. UPDATE moderation_case status (acquires the row lock when executed)

This is **not** the JM-c FK-SHARE-then-EXCLUSIVE class. Concurrent appeal requests on the same case would deadlock under `tokio::join!`, but the existing case-already-Appealed reject path is the protection — do NOT add a `FOR UPDATE` clause; the plan explicitly says no.

### `appeal.window_days` is read at submit_jury_vote, not here (plan GOTCHA)

Per JM-c §10.5: `appeal_window_expires_at` is computed at `submit_jury_vote.rs` step 9 and stored on the case row. `request_appeal.rs` reads the column, NOT the live config. Do not introduce a config read for `appeal.window_days` in Part B.

### Memory-cap awareness

The daemon runs under `MemoryMax=10G`, `MemoryHigh=8G` (deployed at homeserver `20f251b`). Validation runs out-of-band on GH Actions (Shape G) — no LOCAL `cargo check --workspace --features full`. Local cargo work is limited to `cargo +nightly fmt` (cheap, sub-1-second).

If you find yourself wanting to run a workspace check locally to debug, **don't** — push to your worktree branch and let GH Actions cargo-validate-workspace catch failures. The Shape G turnaround is ~10-20 min.

### Branch + commit discipline

- You start on a Junior worktree off `phase-v1-JM-d` (currently at `606f9db5e` — post-merge tip carrying option (b) e2e workflow flip + option 2 schema-v2 + ci-watcher rewrite). Finalize merges your worktree branch back; do not push to `phase-v1-JM-d` directly.
- One commit. Parts A + B + C all go in the same commit. If clippy/check fails, amend or fixup; do not split.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch per `.claude/CLAUDE.md` cheatsheet. The advisor cannot read worktree-local state otherwise.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Lesson trailer (encouraged)

End the commit body with a single `LESSON:` line per `feedback_junior_pmd_write_convention.md` if you discover anything plan-relevant that a future task on this codebase would want — particularly around `select_eligible_jurors` reuse patterns, `run_transaction` async-block nesting, or governance_log payload composition for panel-level events.

## 5. Validation gates (out-of-band on GH Actions per Shape G)

Per `.claude/PRPs/briefs/jm-d-impl-2.md` §5 + `feedback_advisor_orchestrator_forbidden_window_skipped.md`. Validation runs out-of-band; **do NOT run local cargo gates**. After committing your work, push to your worktree branch and exit.

After `git push`:

1. Capture the workflow_run id for the **workspace** validation run (NOT the migration run):
   ```bash
   gh run list --repo barrie-cork/lemmy --branch <your-branch> \
     --workflow cargo-validate-workspace --limit 1 \
     --json databaseId --jq '.[0].databaseId'
   ```
   Retry with exponential backoff up to ~2 min if the run hasn't appeared yet.

   `--repo barrie-cork/lemmy` is mandatory (per `feedback_gh_pr_fork_repo_flag.md`). `--workflow cargo-validate-workspace` is mandatory on JM-d branches — both cargo-validate-workspace.yml and cargo-validate-migration.yml trigger; the migration workflow runs the `migrate-roundtrip.sh` stub which is design-intended to fail on JM-d branches until v1-JM-e replaces the stub body. Capture only the workspace run id.

2. Append a `validate-pending` entry to `.claude/decision-queue.json`. Per option 2 (PMD #156, locked 2026-04-28), the entry includes the nullable mutation fields (`result`, `log_slice`, `failed_jobs`) initialised to `null` at write time — they are populated by ci-watcher when it mutates this entry post-workflow.
   ```json
   {
     "id": <next>,
     "from": "impl",
     "kind": "validate-pending",
     "timestamp": "<ISO 8601 UTC>",
     "workflow_run_id": <id>,
     "branch": "<your-branch>",
     "phase_task": 3,
     "result": null,
     "log_slice": null,
     "failed_jobs": null,
     "answer": null,
     "answered_by": null,
     "resolved_at": null
   }
   ```

3. Commit + push the DQ update.

4. **Exit with success.**

The impl-task slot frees as soon as the push lands. ci-watcher polls the workflow asynchronously and **mutates this entry in place**: populates `result` + `log_slice` + `failed_jobs` + `answer` + `answered_by: "ci-watcher"` + `resolved_at`. The entry's `kind` stays `"validate-pending"`; on `result: "pass"` the entry moves from `pending[]` to `resolved[]`; failures (fail / cancelled / timed_out / gh_unauth / run_not_found) stay in `pending[]` for advisor §G4 triage. The advisor reads the mutated entry on its next polling tick.

### Plan §13 GOTCHA on `check_for_backend(diesel::pg::Pg)`

If `cargo check --workspace --features full` on the runner fails with `check_for_backend(diesel::pg::Pg)`, the schema regen mismatched the `Queryable` derive's expected types — but **schema regen happened in Task 2**, not here. If this error appears in Task 3, it's a downstream effect of how your code consumes the schema (e.g. wrong column type read in a `Queryable`). Pull the failure log slice from the mutated `validate-pending` entry's `log_slice` field (per option 2), fix in a fix-in-PR commit, do not re-regen.

### Plan §13 GOTCHA on clippy `expect_used`/`unwrap_used`

Workspace denies these even in test code. The new `select_appeal_panel` helper's `?` propagation must use `LemmyResult` end-to-end — no `.unwrap()`, no `.expect()`. Per `feedback_clippy_test_style.md`.

## 6. Expected output (return to advisor)

```
## Task 3 complete — JM-d bounded-window appeal + reporter-rights + auto-rejury + winning_decision write

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/src/governance/submit_jury_vote.rs (+winning_decision write to post-decision UPDATE)
  - crates/api/api_crud/src/governance/request_appeal.rs (rewrite — bounded window + reporter-rights branch + auto-rejury)
  - crates/api/api/src/governance/admin_assign_jury.rs (+select_appeal_panel helper + AppealPanelSelection struct)
**Validate workflow:** <run_id> push-and-exit per Shape G
**DQ raised:** #<N> validate-pending for run <run_id>
**Next:** advisor queues task 4 (admin_trigger_appeal_rejury handler + DTO + route)
```

Plus any additional DQ #N references if you raised one mid-task (overrides, clarifications, blockers).

## 7. Why this brief differs from the plan

No overrides at brief-write time. The plan's Part C code block is canonical; treat plan §13 lines 1370-1538 as authoritative.

If you discover a substitution mid-task (e.g. helper signature drift, governance_log API change, config-key-metadata addition), file a DQ pending entry citing the plan line — do not patch silently.
