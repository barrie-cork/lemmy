---
role: impl-task
plan_task: 4
phase: v1-JM-d
created: 2026-04-28
status: ready
related_dq: null
---

# Brief — v1-JM-d Task 4 — admin_trigger_appeal_rejury handler + DTO + route + seat_appeal_panel helper extraction

## 1. Role + dispatch line

`[role:impl-task] v1-JM-d task 4 — see .claude/PRPs/briefs/jm-d-impl-4.md`

You are the **impl-task** subagent (Sonnet 4.6 per the four-role tiering patch verified 2026-04-28). Execute plan task 4 from `.claude/PRPs/plans/v1-jury-mechanics-d.plan.md` §13 (lines 1538-1626).

## 2. Scope

**Produce** (one commit per the plan's "one commit per task" rule):

**Part A — extract `seat_appeal_panel` helper from `request_appeal.rs`:**

Task 3 inlined the panel-seating block at `crates/api/api_crud/src/governance/request_appeal.rs:188-235` (the `if auto_select { ... }` body — calls `select_appeal_panel`, loops to seat each juror via `JuryAssignmentInsertForm`, emits `appeal_panel_assembled` governance_log, UPDATEs the appeal row's `panel_size_snapshot` + `threshold_count_snapshot`).

Plan §13 Task 4 GOTCHA (lines 1605-1611) requires extracting this into a shared helper alongside `select_appeal_panel` to avoid duplication once `admin_trigger_appeal_rejury` lands. **Extract first, then build Task 4 on top of the helper.**

Suggested signature (the plan calls this "at impl discretion" — adjust if MIRROR-grep reveals a cleaner shape):

```rust
// In crates/api/api/src/governance/admin_assign_jury.rs, alongside select_appeal_panel:
pub(crate) async fn seat_appeal_panel(
  conn: &mut diesel_async::AsyncPgConnection,
  case_id: ModerationCaseId,
  appeal_id: AppealId,
  selection: &AppealPanelSelection,
  actor_pseudonym: String,
) -> LemmyResult<Vec<String>>  // returns the panel pseudonyms in case the caller wants them
```

The body MUST be the exact behaviour Task 3 inlined — no semantic drift. Specifically:
- Pre-compute `constraints_json = selection.constraint_record.to_json()` once
- For each `person_id` in `selection.person_ids`: get-or-create pseudonym, build `JuryAssignmentInsertForm { case_id, person_id, status: Selected, selected_under_constraints: Some(constraints_json.clone()), role: Some(JuryAssignmentRole::Appeal) }`, INSERT
- Emit ONE `appeal_panel_assembled` governance_log entry with payload `{ "case_id": case_id.0, "new_panel_pseudonyms": panel_pseudonyms, "excluded_juror_count": selection.excluded_juror_count, "appeal_threshold_count": selection.threshold_count_snapshot }`, attribution `Some(actor_pseudonym.clone())`
- UPDATE `appeal::table.filter(appeal::id.eq(appeal_id))` to set `panel_size_snapshot.eq(Some(selection.panel_size_snapshot))` + `threshold_count_snapshot.eq(Some(selection.threshold_count_snapshot))`
- Return `Vec<String>` of pseudonyms

Then in `request_appeal.rs:188-235`, replace the inlined block with `seat_appeal_panel(conn, data.case_id, new_appeal.id, &selection, caller_pseudonym.clone()).await?;` (drop the `panel_pseudonyms` local if unused, or rename if still needed).

**MIRROR-verify before extraction:** `grep -n "panel_pseudonyms\|appeal_panel_assembled\|JuryAssignmentRole::Appeal" crates/api/api_crud/src/governance/request_appeal.rs` — confirm the block boundaries before editing. Line numbers may have drifted post-Task-3 fixes (3a/3b/3c).

**Part B — DTO additions in `crates/api/api_common/src/governance.rs`:**

Per plan §13 Task 4 ACTION step 1 (lines 1545-1566):

```rust
#[derive(Debug, Serialize, Deserialize, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Admin-triggered appeal-rejury request — used when
/// `appeal.auto_select_on_appeal_acceptance = false`.
pub struct AdminTriggerAppealRejury {
  pub case_id: ModerationCaseId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts-rs", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts-rs", ts(optional_fields, export))]
/// Response from admin-trigger-appeal-rejury.
pub struct AdminTriggerAppealRejuryResponse {
  pub case_id: ModerationCaseId,
  pub appeal_id: AppealId,
  pub panel_person_ids: Vec<PersonId>,
}
```

Place alphabetically near `AdminCloseCase` (currently at line 174). MIRROR `AdminCloseCase` for derives + ts-rs cfg_attr shape.

**Part C — new file `crates/api/api/src/governance/admin_trigger_appeal_rejury.rs`:**

Mirror `admin_close_case.rs` (full file) for the admin-handler shape: `is_admin` gate → fetch admin pseudonym → `run_transaction` wrapper → status guard with exhaustive match per ADR-013 → idempotency check → `select_appeal_panel` call → `seat_appeal_panel` call (Part A) → return response.

Specific requirements per plan §13 Task 4 ACTION step 2 (lines 1568-1585):

1. `is_admin(&local_user_view)?;` (mirror `admin_close_case.rs:29`)
2. Get `admin_pseudonym` via `actor_pseudonym_helper::get_or_create(&mut context.pool(), admin_id).await?` (mirror `admin_close_case.rs:35-36`)
3. Inside `run_transaction`:
   - Load case via `moderation_case::table.filter(...).select(ModerationCase::as_select()).first(conn).await?`
   - Status guard — exhaustive match. Only `CaseStatus::Appealed` proceeds; everything else returns `Err(LemmyErrorType::NotFound.into())` (or a more specific error variant if one applies; check `LemmyErrorType` enum)
   - Idempotency check: `jury_assignment::table.filter(case_id.eq(...).and(role.eq(Some(JuryAssignmentRole::Appeal)))).count().get_result::<i64>(conn).await?` — if `> 0`, return `Err` (panel already seated; admin should not re-trigger)
   - Load existing Appeal row: `appeal::table.filter(appeal::case_id.eq(...)).select(Appeal::as_select()).first(conn).await?`. Per plan: this is the appeal filed by `request_appeal.rs` while `auto_select_on_appeal_acceptance` was false.
   - `let mut cache = ConfigCache::default();` (or whatever cache type `select_appeal_panel` expects — verify by reading its signature at `admin_assign_jury.rs:1079`)
   - `let selection = select_appeal_panel(conn, &case, &mut cache).await?;`
   - `seat_appeal_panel(conn, case.id, existing_appeal.id, &selection, admin_pseudonym.clone()).await?;`
   - Return `AdminTriggerAppealRejuryResponse { case_id: case.id, appeal_id: existing_appeal.id, panel_person_ids: selection.person_ids.clone() }`

**Part D — module + route registration:**

Per plan §13 Task 4 ACTION steps 3-4 (lines 1587-1597):

1. `crates/api/api/src/governance/mod.rs` — `pub mod admin_trigger_appeal_rejury;` inserted alphabetically. Currently `admin_assign_jury` (line 14) and `admin_close_case` (line 16) are present; the new module slots between `admin_dashboard` and `admin_emergency_remove` alphabetically (after `admin_close_case`, before `admin_dashboard`). **Verify by reading mod.rs first** — Task 3 may have changed the alphabetical order.
2. `crates/api/routes/src/lib.rs`:
   - Import line ~37: add `admin_trigger_appeal_rejury::admin_trigger_appeal_rejury,` to the `lemmy_api::governance::{...}` use block, alphabetical position (between `admin_close_case` line 39 and the next entry)
   - Route line ~538: `.route("/trigger-appeal-rejury", post().to(admin_trigger_appeal_rejury))` under the `/admin` scope. Place after `.route("/close-case", ...)` (line 538). MIRROR existing admin route style verbatim.

**Do NOT** in this task:
- Touch the `appeal_window_expiry` background job (Task 5).
- Author any e2e tests or fixtures (Task 6).
- Modify `select_appeal_panel` signature or body (Task 3 — already shipped).
- Modify `submit_jury_vote.rs` (Task 3 — already shipped).
- Touch migrations (Task 1, already shipped).
- Refactor anything in `admin_close_case.rs` itself (mirror only — do not edit).

**Commit message** (exactly):
`feat(v1-JM-d): admin_trigger_appeal_rejury handler + DTO + route + seat_appeal_panel helper (task 4)`

## 3. Required reading

In this order:

1. **`.claude/decision-queue.json` resolved entries** (skim, focus on these):
   - **#75** (resolved 2026-04-28) — `select_appeal_panel` + `AppealPanelSelection` are `pub` (not `pub(crate)`) for cross-crate visibility. The plan §13 Task 4 says `pub(crate)` for the new symbols, but Task 4 lives in `lemmy_api` (same crate as `admin_assign_jury`), so `pub(crate)` works. **Use `pub(crate)` for `seat_appeal_panel`** — it's only consumed within `lemmy_api` (the new admin handler) and `lemmy_api_crud` (the existing `request_appeal`). Wait — `lemmy_api_crud` is a different crate. **Make `seat_appeal_panel` `pub`** so `request_appeal.rs` (in `lemmy_api_crud`) can import it, mirroring the `select_appeal_panel` pattern from DQ #75.
   - **#80** (closed superseded by #83) — fix-3c shipped; Phase 2 e2e green at run 25067030047. Your work is on top of `f2e4056cc` (or whatever phase-v1-JM-d tip is when Junior cuts the worktree).
   - **#83** (resolved 2026-04-28) — phase-v1-JM-d Phase 2 e2e PASSED. Task 3 is fully validated; Task 4 begins on green.

2. **Plan §13 Task 4 (lines 1538-1626)** — canonical step list. The four ACTION steps + the GOTCHA on duplication + the MIRROR refs are load-bearing.

3. **Plan §10 (Patterns to mirror)** — `crates/api/api/src/governance/admin_close_case.rs` is the canonical "small admin handler" shape; admin_assign_jury.rs is the canonical panel-seating shape.

4. **MIRROR refs — read each before writing the corresponding code:**
   - `crates/api/api/src/governance/admin_close_case.rs` (full file, ~110 lines) — handler structure, `is_admin` gate, `run_transaction` wrapper, exhaustive status match
   - `crates/api/api/src/governance/admin_assign_jury.rs:1079-1155` — `select_appeal_panel` signature + return shape (you call it from Part C)
   - `crates/api/api_crud/src/governance/request_appeal.rs:180-235` — the panel-seating block you're extracting in Part A
   - `crates/api/api_common/src/governance.rs:174-200` area — `AdminCloseCase` + `AdminCloseCaseResponse` derives (mirror for Part B)
   - `crates/api/api/src/governance/mod.rs` — module declaration order (alphabetical insertion in Part D)
   - `crates/api/routes/src/lib.rs:30-45` (import block) and `:530-545` (admin scope) — route registration sites for Part D

5. **Lessons** (Glob `.claude/lessons/`, Read each that matches):
   - `feedback_multi_write_handlers_need_transactions.md` — your handler does Appeal load + jury_assignment INSERTs + governance_log emission + appeal UPDATE. Wrap in `run_transaction` per `admin_close_case.rs:44-49` shape.
   - `feedback_features_full_p_crate_incompatible.md` — DoD uses `--workspace --features full`; never combine with `-p <crate>`.
   - `feedback_clippy_test_style.md` — workspace clippy denies `expect_used`/`unwrap_used`/`allow_attributes` even in tests.
   - `feedback_lemmy_error_no_std_error.md` — `LemmyResult` doesn't auto-convert to `Box<dyn Error>`.
   - `feedback_rust_visibility_cross_crate.md` — `pub(crate)` in `lemmy_api` is INVISIBLE to `lemmy_api_crud` callers. Use `pub` for `seat_appeal_panel` so `request_appeal.rs` can import it. (This is exactly DQ #75's lesson.)
   - `feedback_insertform_default_propagation.md` — `JuryAssignmentInsertForm` already used by Task 3; mirror that shape.
   - `feedback_pipes_mask_exit_codes.md` — capture-then-tail rule for the workflow run check.
   - `feedback_advisor_orchestrator_forbidden_window_skipped.md` — Shape G means no local cargo gates; push and exit.

6. **PRD §6 + §9.4** — `admin_trigger_appeal_rejury` is the manual fallback when `appeal.auto_select_on_appeal_acceptance = false`. Read for the why; the plan's ACTION steps are authoritative for the what.

## 4. Constraints

### Plan-cited line numbers may have drifted

Tasks 1, 2, 3, plus three fix-loop commits (fix-3a/3b/3c) have shipped to phase-v1-JM-d. The plan was authored against an earlier state. **Verify all line refs before editing:**

```bash
grep -n "panel_pseudonyms\|appeal_panel_assembled\|JuryAssignmentRole::Appeal" crates/api/api_crud/src/governance/request_appeal.rs
grep -n "select_appeal_panel\|AppealPanelSelection" crates/api/api/src/governance/admin_assign_jury.rs
grep -n "admin_close_case::admin_close_case\|admin_assign_jury::admin_assign_jury" crates/api/routes/src/lib.rs
grep -n "pub mod admin_close_case\|pub mod admin_assign_jury" crates/api/api/src/governance/mod.rs
grep -n "AdminCloseCase\|AdminAssignJury" crates/api/api_common/src/governance.rs
```

If any search returns ZERO hits, file a DQ pending entry from `from: "impl"`, `kind: "blocker"`, citing the plan line + the search you ran. Do not patch silently.

### Cross-crate visibility (load-bearing — DQ #75)

`seat_appeal_panel` MUST be `pub`, not `pub(crate)`. `request_appeal.rs` lives in `lemmy_api_crud` (a different crate from `lemmy_api` where `admin_assign_jury.rs` lives). Per DQ #75 resolution and `feedback_rust_visibility_cross_crate.md`: `pub(crate)` items in `lemmy_api` are invisible to `lemmy_api_crud` callers. Use `pub`. The plan §13 Task 4 says `pub(crate)` for the new symbols — that wording is wrong for cross-crate items; treat `select_appeal_panel`'s existing `pub` (Task 3 already corrected this) as the canonical pattern and apply the same to `seat_appeal_panel`.

### Multi-write transaction discipline (load-bearing)

Both the Part A extraction and the Part C handler do multi-write work. The flow:
- Part A `seat_appeal_panel`: N jury_assignment INSERTs + 1 governance_log INSERT + 1 appeal UPDATE
- Part C `admin_trigger_appeal_rejury`: case SELECT + jury_assignment count + appeal SELECT + `select_appeal_panel` (DB read+compute) + `seat_appeal_panel` call

The handler in Part C MUST wrap in `run_transaction` per `admin_close_case.rs:44-49`:
```rust
let outcome = conn
  .run_transaction(|conn| {
    async move { process_trigger_rejury(conn, ...).await }.scope_boxed()
  })
  .await?;
```

`seat_appeal_panel` accepts `&mut AsyncPgConnection` — it does NOT open its own transaction. Callers (both `request_appeal.rs` and the new handler) provide the connection from inside their existing `run_transaction`. This matches the Task 3 pattern.

### Exhaustive status match per ADR-013

```rust
match case.status {
  CaseStatus::Appealed => {} // proceed
  CaseStatus::Open
  | CaseStatus::ThresholdMet
  | CaseStatus::JurySelection
  | CaseStatus::InReview
  | CaseStatus::Decided
  | CaseStatus::EmergencyRemove
  | CaseStatus::AdminReview
  | CaseStatus::Closed => return Err(LemmyErrorType::NotFound.into()),
}
```

No `_ =>` catchall. Mirror `admin_close_case.rs:64-74` shape.

### Idempotency check shape

The plan says: "Verify no existing `jury_assignment` rows with `role = Appeal` exist for this case." Use `count` not `first` so the check is unambiguous:

```rust
let existing_appeal_assignments: i64 = jury_assignment::table
  .filter(jury_assignment::case_id.eq(data.case_id))
  .filter(jury_assignment::role.eq(Some(JuryAssignmentRole::Appeal)))
  .count()
  .get_result(conn)
  .await?;
if existing_appeal_assignments > 0 {
  return Err(LemmyErrorType::Unknown("appeal panel already seated".to_string()).into());
}
```

If a more specific `LemmyErrorType` variant fits ("AlreadyExists", "Conflict", etc.), prefer it. Check the enum before defaulting to `Unknown`.

### Memory-cap awareness + Shape G push-and-exit

Per `feedback_advisor_orchestrator_forbidden_window_skipped.md` and the JM-d Task 3 brief §4: **do NOT run local cargo gates**. The daemon runs under `MemoryMax=10G`. After committing, push to your worktree branch and exit. GH Actions cargo-validate-workspace catches failures within ~10-20 min.

### Branch + commit discipline

- Junior cuts a worktree off `phase-v1-JM-d`. Currently at `f2e4056cc` (post-fix-3c, post-Phase-2-e2e-green). Confirm by running `git rev-parse HEAD` at task-0.
- One commit. Parts A + B + C + D all go in the same commit. If clippy/check fails on Shape G, fix-in-PR with a follow-up commit.
- Mid-task DQ visibility: if you raise a `pending` entry, **commit + push immediately** to your worktree branch.
- No `answered_by: "advisor"` or `"user"` from this subagent. Self-resolve only as `"impl-self-resolved"`.

### Lesson trailer (encouraged)

End the commit body with a single `LESSON:` line per `feedback_junior_pmd_write_convention.md` if you discover anything plan-relevant a future task would want — particularly around helper extraction patterns when a Task N inlines what Task N+1 needs to share, or `pub` vs `pub(crate)` choices when cross-crate boundaries are involved.

## 5. Validation gates (out-of-band on GH Actions per Shape G)

Per `.claude/PRPs/briefs/jm-d-impl-3.md` §5 (verbatim pattern). After committing:

1. Capture the workflow_run id for the **workspace** validation run (NOT the migration run):
   ```bash
   gh run list --repo barrie-cork/lemmy --branch <your-branch> \
     --workflow cargo-validate-workspace --limit 1 \
     --json databaseId --jq '.[0].databaseId'
   ```
   Retry with exponential backoff up to ~2 min if the run hasn't appeared yet.

   `--repo barrie-cork/lemmy` is mandatory (per `feedback_gh_pr_fork_repo_flag.md`). `--workflow cargo-validate-workspace` is mandatory — both workspace and migration workflows trigger; capture only the workspace run id. The migration workflow runs `migrate-roundtrip.sh` which is a stub for JM-d.

2. Append a `validate-pending` entry to `.claude/decision-queue.json` (option 2 schema-v2 per PMD #156, locked 2026-04-28):
   ```json
   {
     "id": <next>,
     "from": "impl",
     "kind": "validate-pending",
     "timestamp": "<ISO 8601 UTC>",
     "workflow_run_id": <id>,
     "branch": "<your-branch>",
     "phase_task": 4,
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

ci-watcher polls the workflow asynchronously and **mutates this entry in place** per option 2. The advisor's polling loop reads the mutated entry on its next tick.

### Plan §13 GOTCHA on `check_for_backend(diesel::pg::Pg)`

If `cargo check --workspace --features full` on the runner fails with `check_for_backend(diesel::pg::Pg)`, the schema regen mismatched the `Queryable` derive's expected types — but **schema regen happened in Task 2**, not here. If this error appears, it's a downstream effect of how your code consumes the schema. Pull the failure log_slice from the mutated `validate-pending` entry, fix in a follow-up commit, do not re-regen.

### Plan §13 GOTCHA on clippy `expect_used`/`unwrap_used`

Workspace denies these. Use `?` propagation everywhere. The `seat_appeal_panel` extraction and the new handler must be `LemmyResult` end-to-end.

## 6. Expected output (return to advisor)

```
## Task 4 complete — JM-d admin_trigger_appeal_rejury + seat_appeal_panel helper

**Commit:** <sha> on <worktree-branch>
**Files changed:**
  - crates/api/api/src/governance/admin_assign_jury.rs (+seat_appeal_panel helper extracted alongside select_appeal_panel)
  - crates/api/api_crud/src/governance/request_appeal.rs (replace inlined panel-seating block with seat_appeal_panel call)
  - crates/api/api_common/src/governance.rs (+AdminTriggerAppealRejury, AdminTriggerAppealRejuryResponse DTOs)
  - crates/api/api/src/governance/admin_trigger_appeal_rejury.rs (new — admin handler)
  - crates/api/api/src/governance/mod.rs (+pub mod admin_trigger_appeal_rejury;)
  - crates/api/routes/src/lib.rs (+import, +route)
**Validate workflow:** <run_id> push-and-exit per Shape G
**DQ raised:** #<N> validate-pending for run <run_id>
**Next:** advisor queues task 5 (appeal_window_expiry background job)
```

Plus any additional DQ #N references if you raised one mid-task.

## 7. Why this brief differs from the plan

Two adjustments:

1. **Visibility:** Plan §13 Task 4 says `pub(crate)` for new helpers; this brief overrides to `pub` per DQ #75 resolution (cross-crate visibility for callers in `lemmy_api_crud`). The plan was written before Task 3's DQ #75 surfaced the issue; brief is forward-corrected.

2. **Helper extraction order:** Plan §13 Task 4 GOTCHA suggests extracting `seat_appeal_panel` "if duplication is meaningful — at impl discretion." This brief promotes that suggestion to a hard requirement (Part A) because the duplication IS meaningful — `request_appeal.rs:188-235` is the exact body the new handler needs. Extracting first means the new handler only adds ~50 lines instead of duplicating ~50 lines.

If you discover other plan/reality drifts mid-task (helper signature change, governance_log API drift, route-registration shape change), file a DQ pending entry citing the plan line — do not patch silently.
