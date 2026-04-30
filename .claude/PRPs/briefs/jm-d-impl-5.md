---
role: impl-task
plan_task: 5
phase: v1-JM-d
created: 2026-04-30
status: ready
related_dq: null
---

# Brief — v1-JM-d Task 5 — `appeal_window_expiry` background job + hourly scheduler tick

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d task 5 — see .claude/PRPs/briefs/jm-d-impl-5.md`

You are the **impl-task** subagent (Sonnet 4.6 per the four-role tiering patch verified 2026-04-28). Execute plan task 5 from `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §13 (lines 1630-1704), with the canonical body source at §10.6 (lines 869-1018).

## 2. Scope

**Produce** (one commit per the plan's "one commit per task" rule):

**Part A — new module `crates/api/api/src/governance/appeal_window_expiry.rs`:**

Full body listed in plan §10.6 (lines 873-963). Copy verbatim, adjusting only if a MIRROR-grep reveals current code shape has drifted (e.g. new diesel import path, new governance_log signature). Specifically:

- Module-level doc-comment naming PRD §9.5 + the SKIP LOCKED rationale per JM-c retro §3.2 amendment 1 / DQ #50.
- `pub struct AppealWindowExpiryOutcome { pub cases_processed: usize }` for batch-result tracking (mirrors `ReputationSnapshotOutcome` shape).
- `pub async fn run_appeal_window_expiry_batch(context: &LemmyContext) -> LemmyResult<AppealWindowExpiryOutcome>`:
  - SELECT `moderation_case` rows with `status = Decided` AND `appeal_window_expires_at < Utc::now()`, with `for_update().skip_locked()`.
  - For each row: UPDATE `status = Closed`, `closed_at = Some(now)`, then `governance_log::append(..., ENTRY_KIND_APPEAL_WINDOW_EXPIRED, json!({ case_id, decided_at, window_expired_at }), None)` — `None` actor (system-issued, ADR-015 allows for system identity).
  - Single `info!` after the loop iff `!candidates.is_empty()` reporting `cases_processed`.

**Part B — module declaration in `crates/api/api/src/governance/mod.rs`:**

Insert `pub mod appeal_window_expiry;` alphabetically. **Verify alphabetical order first** — Tasks 1-4 may have changed module ordering.

**Part C — scheduler tick registration in `crates/routes/src/utils/scheduled_tasks.rs`:**

Per plan §10.6 ("Module-scope additions" + "Scheduler tick registration"):

1. **At module scope** alongside `REPUTATION_SNAPSHOT_RUNNING` (currently line 71-77):

   ```rust
   static APPEAL_WINDOW_EXPIRY_RUNNING: AtomicBool = AtomicBool::new(false);

   struct AppealWindowExpiryRunningGuard;

   impl Drop for AppealWindowExpiryRunningGuard {
     fn drop(&mut self) {
       APPEAL_WINDOW_EXPIRY_RUNNING.store(false, Ordering::Release);
     }
   }
   ```

   Place adjacent to / immediately after the existing `REPUTATION_SNAPSHOT_RUNNING` block. Mirror the doc-comment pattern at `:68-69`.

2. **In `setup`** after the reputation-snapshot tick block (currently around `:188-198`):

   ```rust
   let context_appeal_expiry = context.reset_request_count();
   scheduler.every(CTimeUnits::hour(1)).run(move || {
     let context = context_appeal_expiry.reset_request_count();
     async move {
       if std::env::var("BREHON_DISABLE_APPEAL_WINDOW_JOB").as_deref() == Ok("1") {
         return;
       }
       if APPEAL_WINDOW_EXPIRY_RUNNING
         .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
         .is_err()
       {
         warn!("appeal_window_expiry: previous batch still running, skipping this tick");
         return;
       }
       let _guard = AppealWindowExpiryRunningGuard;
       lemmy_api::governance::appeal_window_expiry::run_appeal_window_expiry_batch(&context)
         .await
         .inspect_err(|e| warn!("Failed to run appeal_window_expiry batch: {e}"))
         .ok();
     }
   });
   ```

   Cadence is `CTimeUnits::hour(1)` — **NOT** `CTimeUnits::minutes(15)` (the snapshot cadence). Per plan §13 Task 5 GOTCHA (lines 1655-1657), don't copy the 15-min cadence by accident.

**Part D — registry log line in `crates/server/src/governance.rs`:**

The `schedule_governance_jobs` function currently emits one `info!` listing the snapshot job. Append a second `info!` for the appeal-window-expiry tick — mirror the existing line's shape verbatim (path-with-backslash, env-var disable mention). Per plan §13 Task 5 ACTION step 4 + plan §10.6 final GOTCHA.

**Do NOT** in this task:
- Touch `request_appeal.rs` or `submit_jury_vote.rs` (Task 3 — already shipped).
- Touch `admin_trigger_appeal_rejury` (Task 4 — already shipped).
- Author any e2e tests or fixtures (Task 6).
- Modify `reputation_snapshot.rs` itself (mirror only — do not edit).
- Modify migrations (Task 1 — already shipped).
- Add the `BREHON_DISABLE_APPEAL_WINDOW_JOB` env-var to any test bootstrap (that's Task 6's responsibility).
- Register `ENTRY_KIND_APPEAL_WINDOW_EXPIRED` — already done by JM-a (verified at `crates/db_schema/src/source/governance/governance_log.rs:184` + `crates/api/api/src/governance/governance_log.rs:46`).

**Commit message** (exactly):
`feat(v1-JM-d): appeal-window-expiry background job + hourly scheduler tick (task 5)`

## 3. Required reading

In this order:

1. **Plan §13 Task 5 (lines 1630-1704)** — canonical step list. The four ACTION steps + four GOTCHAs (hourly vs 15-min, SKIP LOCKED rationale, deadlock-free-by-construction explanation, BREHON_DISABLE env var, system-issued None actor) are load-bearing.

2. **Plan §10.6 (lines 869-1018)** — full body of the new module + scheduler block + module-scope additions. **Copy this verbatim** into the new file; this is the canonical source.

3. **Plan §10.7 (line 1020+)** — confirms `ENTRY_KIND_APPEAL_WINDOW_EXPIRED` is already registered by JM-a (verified via grep — see Constraints §"Pre-existing constants").

4. **MIRROR refs — read each before writing the corresponding code:**
   - `crates/routes/src/utils/scheduled_tasks.rs:68-77` — `REPUTATION_SNAPSHOT_RUNNING` atomic + `RunningGuard` Drop pattern (mirror for Part C step 1)
   - `crates/routes/src/utils/scheduled_tasks.rs:173-200` — reputation tick block shape (mirror for Part C step 2; cadence diverges)
   - `crates/api/api/src/governance/reputation_snapshot.rs:361-407` — batch fn shape with outcome struct + tracing (mirror for Part A)
   - `crates/api/api/src/governance/governance_log.rs` — `governance_log::append` signature; confirm `Option<String>` actor parameter
   - `crates/api/api/src/governance/mod.rs` — module declaration order (alphabetical insertion in Part B)
   - `crates/server/src/governance.rs` — current `schedule_governance_jobs` doc-stub shape (Part D appends to existing `info!`)
   - `crates/db_schema/src/source/governance/moderation_case.rs` (or schema file) — confirm `appeal_window_expires_at`, `closed_at`, `decided_at` field names match the SELECT/UPDATE shape

5. **Lessons** (Glob `.claude/lessons/`, Read each that matches):
   - `feedback_features_full_p_crate_incompatible.md` — DoD uses `--workspace --features full`; never combine with `-p <crate>`.
   - `feedback_features_full_workspace_only.md` — same family; clippy/check needs workspace scope.
   - `feedback_clippy_test_style.md` — workspace clippy denies `expect_used`/`unwrap_used`/`allow_attributes` even in tests.
   - `feedback_clippy_doc_lazy_continuation_in_doc_comments.md` — your module doc-comment is multi-paragraph; reword mid-paragraph "and"s if clippy flags lazy-continuation.
   - `feedback_lemmy_error_no_std_error.md` — `LemmyResult` doesn't auto-convert to `Box<dyn Error>`.
   - `feedback_pipes_mask_exit_codes.md` — capture-then-tail rule for the workflow run check.
   - `feedback_advisor_orchestrator_forbidden_window_skipped.md` — Shape-G workspace validation is push-and-exit; do NOT run local cargo gates.
   - `feedback_background_task_notification_lies.md` — informational only; the in-process scheduler-tick pattern here doesn't have the spawn-then-claim-success failure mode (the `inspect_err` + `.ok()` chain logs failures), but worth re-reading for the discipline.

6. **PRD §9.5** — appeal-window expiry is the close-out path. The hourly cadence is a v0 simplification (PRD §17 row 4) — production may tighten later. Read for the why.

## 4. Constraints

### Plan-cited line numbers may have drifted

Tasks 1, 2, 3, 4 (plus three fix-loops 3a/3b/3c and three fix-loops 4a/4a-v2/4a-v3) have shipped to `phase-v1-JM-d` and merged into `governance-v0` (currently at `f62d586df`). The plan was authored against an earlier state. **Verify all line refs before editing:**

```bash
grep -n "REPUTATION_SNAPSHOT_RUNNING\|RunningGuard\|reputation_snapshot::run_snapshot_batch" crates/routes/src/utils/scheduled_tasks.rs
grep -n "pub mod " crates/api/api/src/governance/mod.rs
grep -n "schedule_governance_jobs\|info!" crates/server/src/governance.rs
grep -n "ENTRY_KIND_APPEAL_WINDOW_EXPIRED" crates/api/api/src/governance/governance_log.rs
grep -n "appeal_window_expires_at\|closed_at\|decided_at" crates/db_schema_file/src/schema.rs
```

If any search returns ZERO hits where one is expected, file a DQ pending entry from `from: "impl"`, `kind: "blocker"`, citing the plan line + the search you ran. Do not patch silently.

### Pre-existing constants (do not redeclare)

`ENTRY_KIND_APPEAL_WINDOW_EXPIRED` is already declared and registered by JM-a:
- `crates/db_schema/src/source/governance/governance_log.rs:184` — `pub const ENTRY_KIND_APPEAL_WINDOW_EXPIRED: &str = "appeal_window_expired";`
- `crates/api/api/src/governance/governance_log.rs:46` — already in the registry list

**Do NOT** re-declare or re-register. Just `use` the constant from its existing path.

### Hourly cadence — NOT 15-minute

`scheduler.every(CTimeUnits::hour(1))`. The snapshot tick is `CTimeUnits::minutes(15)`. Easy to copy-paste-wrong; double-check the cadence literal before committing. Per plan §13 Task 5 GOTCHA (lines 1655-1657).

### SKIP LOCKED + lock-acquisition order (DQ #50 / JM-c retro §3.2 amendment 1)

The SELECT uses `for_update().skip_locked()`. This is **deliberate** for deadlock-free coexistence with concurrent `request_appeal` mid-tx. Specifically:

- `request_appeal` acquires its row lock implicitly via the `case load → INSERT appeal → UPDATE moderation_case` sequence (Diesel acquires X lock at the UPDATE step).
- This tick's `for_update().skip_locked()` is on the SELECT *before* the batch UPDATE.
- If the bg job's SELECT runs while `request_appeal` holds the row → SKIP LOCKED skips it → next tick (1 hour later) catches it.
- If the bg job's SELECT runs first → `request_appeal` arriving after blocks at its UPDATE (proper FIFO).
- No deadlock by construction.

Acceptable user-visible effect: appeal windows are bounded in days, not minutes, so a 1-hour delay on the close transition is invisible.

### `governance_log::append` actor is `None`

The `actor_pseudonym` argument is `None` for system-issued entries. ADR-015's pseudonymisation rule applies to USER identity, not system identity. Confirm `governance_log::append`'s signature accepts `Option<String>` (or whatever the current shape is — Tasks 1-4 may have refactored).

### Multi-write in a non-transactional batch (deliberate)

Per plan §10.6 the batch processes each row sequentially without an outer transaction. Each iteration is one UPDATE + one governance_log INSERT — these are two writes per row. **Per plan**: this is acceptable because:
- The UPDATE uses an implicit row-level lock; if it fails, the next iteration's SELECT (next tick) re-finds the row.
- The governance_log INSERT failing after the UPDATE leaves a stale `Closed` row without an audit entry — but the Decided→Closed transition is idempotent at the next tick (the row is no longer Decided, so it's not picked up again). This is preferable to wrapping all rows in one mega-transaction (long-held locks, contention).

If you find a reason to wrap each row's UPDATE+log in a per-row `run_transaction`, file a DQ pending — don't change the shape silently.

### Do not add e2e disable in test bootstrap

The `BREHON_DISABLE_APPEAL_WINDOW_JOB=1` env-var is consulted at the *scheduler tick* (Part C). Setting it in test bootstrap is **Task 6's job**, not Task 5's. This task only adds the env-var check; it does not modify any test code or `e2e_*.rs` fixtures.

### Memory-cap awareness + Shape-G push-and-exit (workspace) + laptop e2e (post-merge)

Per the v1-JM-d hybrid validation flow used for Tasks 2-4 (DQ #73, #76, #82, #88, #89 are the audit trail):

- After committing Task 5: push to your worktree branch and EXIT.
- The daemon runs under `MemoryMax=10G`; do NOT run local cargo on the EliteDesk worker (per `feedback_advisor_orchestrator_forbidden_window_skipped.md` and the JM-d Task 4 brief precedent).
- GH Actions `cargo-validate-workspace` runs the workspace + clippy gate within ~10-20 min.
- Phase-2 e2e (post-finalize-merge into `phase-v1-JM-d`) runs **on the laptop** per the `kind: "validate-pending-laptop-e2e"` flow (advisor-laptop side; not your job).

### Branch + commit discipline

- Junior cuts a worktree off `phase-v1-JM-d`. Currently at `f62d586df` (post-merge of fix-4a-v3 stack into governance-v0; phase-v1-JM-d should be at the same SHA or a descendant). Confirm by running `git rev-parse HEAD` at task-0.
- One commit. Parts A + B + C + D all go in the same commit. If clippy/check fails on Shape G, fix-in-PR with a follow-up commit.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Lesson trailer (encouraged)

End the commit body with a single `LESSON:` line per `feedback_junior_pmd_write_convention.md` if you discover anything plan-relevant a future task would want — particularly around scheduler-tick patterns when a future PRD-mandated job needs the same atomic-bool/guard shape, or any drift between plan §10.6 and current `scheduled_tasks.rs` reality.

## 5. Validation gates (out-of-band on GH Actions per Shape G — workspace only)

Per `.claude/PRPs/briefs/jm-d-impl-4.md` §5 (verbatim pattern). After committing:

1. Capture the workflow_run id for the **workspace** validation run (NOT the migration run, NOT e2e):
   ```bash
   gh run list --repo barrie-cork/lemmy --branch <your-branch> \
     --workflow cargo-validate-workspace --limit 1 \
     --json databaseId --jq '.[0].databaseId'
   ```
   Retry with exponential backoff up to ~2 min if the run hasn't appeared yet.

   `--repo barrie-cork/lemmy` is mandatory (per `feedback_gh_pr_fork_repo_flag.md`). `--workflow cargo-validate-workspace` is mandatory — both workspace and migration workflows trigger; capture only the workspace run id. The migration workflow runs `migrate-roundtrip.sh` which is a stub for JM-d (no new migrations in Task 5).

2. Append a `validate-pending` entry to `.claude/decision-queue.json` (option 2 schema-v2 per PMD #156, locked 2026-04-28):
   ```json
   {
     "id": <next>,
     "from": "impl",
     "kind": "validate-pending",
     "timestamp": "<ISO 8601 UTC>",
     "workflow_run_id": <id>,
     "branch": "<your-branch>",
     "phase_task": 5,
     "result": null,
     "log_slice": null,
     "failed_jobs": null,
     "answer": null,
     "answered_by": null,
     "resolved_at": null
   }
   ```

   `<next>` is `max(all ids across pending + resolved + every archive file) + 1`. Per `decision-queue.md` Recipe 1 — span both live + archives when computing.

3. Commit + push the DQ update.

4. **Exit with success.**

ci-watcher polls the workflow asynchronously and **mutates this entry in place** per option 2. The advisor's polling loop reads the mutated entry on its next tick. Phase-2 e2e is the advisor-laptop's job once Task 5 finalizes into `phase-v1-JM-d` — not yours.

### Plan §13 Task 5 VALIDATE block (informational, NOT to run locally)

The plan lists three gates — `cargo check --workspace --features full`, `cargo clippy --workspace --features full --no-deps -- -D warnings`, `cargo test --test e2e --no-run -p lemmy_server`. Under the v1-JM-d hybrid Shape-G flow these run on GH Actions, NOT on the worker. Do NOT invoke them locally; the workflow YAMLs at `.github/workflows/cargo-validate-workspace.yml` (and the e2e-no-run gate inside it) cover them.

## 6. Expected output (return to advisor)

```
## Task 5 complete — JM-d appeal_window_expiry background job + hourly scheduler tick

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/src/governance/appeal_window_expiry.rs (new — batch job module)
  - crates/api/api/src/governance/mod.rs (+pub mod appeal_window_expiry;)
  - crates/routes/src/utils/scheduled_tasks.rs (+APPEAL_WINDOW_EXPIRY_RUNNING atomic + Guard + hourly tick block)
  - crates/server/src/governance.rs (+info! line for the new tick in schedule_governance_jobs)
**Validate workflow:** <run_id> push-and-exit per Shape G
**DQ raised:** #<N> validate-pending for run <run_id>
**Next:** advisor queues task 6 (e2e tests under mod v1_jm_d_fixtures)
```

Plus any additional DQ #N references if you raised one mid-task.

## 7. Why this brief differs from the plan

One adjustment:

1. **No local cargo gates:** Plan §13 Task 5 VALIDATE block lists `cargo check`, `cargo clippy`, `cargo test --no-run` to run locally. This brief overrides to push-and-exit per Shape G + the hybrid validation flow established for Tasks 2-4 (`feedback_advisor_orchestrator_forbidden_window_skipped.md` + DQ #73/#76/#88 audit trail). The plan was authored before Shape G; brief is forward-corrected.

If you discover other plan/reality drifts mid-task (governance_log API shape, schema field rename, scheduler-API change), file a DQ pending entry citing the plan line — do not patch silently.

## Pre-commit dogfood

Walked through this brief mentally against §10.6's full code body:

- Part A: §10.6 provides a complete module body (lines 875-963). Brief Required Reading §3 step 2 cites this; impl just transcribes (with MIRROR-grep for drift).
- Part B: alphabetical insertion is mechanical; `grep -n "pub mod"` covers it.
- Part C step 1 + step 2: §10.6 provides both blocks verbatim (lines 1000-1012 and 967-998); brief Constraints flag the cadence-mismatch footgun explicitly.
- Part D: §10.6 final GOTCHA + the verified current shape of `crates/server/src/governance.rs:15-29` show a single `info!` to mirror.

What the dogfood caught: the original §13 Task 5 wording was cadence-ambiguous between two GOTCHAs (one says hourly, one casually mentions 15-min). Brief promotes the cadence to a hard Constraint with the reasoning rather than burying it in plan-prose.

What it did not catch: whether `governance_log::append` signature has drifted post-Task-3/4. Brief asks impl to verify via Required Reading §3 step 4 + Constraints "Plan-cited line numbers may have drifted".
