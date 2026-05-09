---
role: impl-task
plan_task: 1-replan
phase: v1-SL-c-2
created: 2026-05-09
related_dq: 164, 165, 166 (all superseded; this is a re-plan, not a §G4 fix-impl)
supersedes_brief: sl-c-2-impl-1.md, sl-c-2-fix-impl-1.md, sl-c-2-fix-impl-2.md
mandatory_lessons_fired:
  - feedback_lemmy_error_no_std_error.md (amended 2026-05-09 — case A applies)
  - feedback_async_pool_test_pattern.md
  - feedback_junior_worker_e2e_edit_hang.md
canonical_sibling: crates/server/tests/e2e.rs:11001-11924 (mod v1_sl_b_fixtures — the v1-SL-b canonical shape)
---

# Brief — v1-SL-c-2 Task 1 RE-PLAN — uniform LemmyResult<T> mirroring v1-SL-b

## 1. Role + dispatch line

`[role:impl-task] sl-c-2-impl-1-replan — see .claude/PRPs/briefs/sl-c-2-impl-1-replan.md`

You are the **impl-task** subagent (Sonnet 4.6). Apply a planned, single-shot error-shape re-plan to v1-SL-c-2 Task 1's e2e module. **This is NOT a §G4 mechanical fix** — it is a re-plan after 3 failed §G4 cycles. The advisor session has hard-reset the worker branch to the original Task 1 commit (`c2761b284`); the §G4 cycle commits are gone.

## 2. Scope

### 2.1 Why this is a re-plan, not a fix

Three §G4 mechanical-fix cycles failed on the same e2e.rs site, each producing a fresh compile error. Per `.claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md`, the recipe family was wrong-shaped — `feedback_lemmy_error_no_std_error.md` (pre-amendment) prescribed exactly one recipe but reality requires case enumeration. The lesson is now amended; the §G4 row split into 4a/4b/4c. This brief picks **Case A — uniform `LemmyResult<T>` throughout**, mirroring the v1-SL-b canonical sibling.

### 2.2 The fix — three coordinated changes to `crates/server/tests/e2e.rs`

#### Change A — flip test fn signature to `LemmyResult<()>`

At line 12041-12044 (post-revert state, mirrors original Task 1 commit `c2761b284`):

```rust
async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(
) -> Result<(), Box<dyn Error>> {
```

Replace with:

```rust
async fn grace_check_fires_expired_case_emits_per_sponsor_and_summary_entries(
) -> LemmyResult<()> {
```

#### Change B — flip all 4 helper return types to `LemmyResult<T>`

In `mod v1_sl_c_fixtures` (lines 11954-12037 post-revert):

| Helper | Current signature | Replace with |
|---|---|---|
| `count_log_entries` (~11954-11963) | `async fn count_log_entries(...) -> Result<i64, Box<dyn Error>>` | `async fn count_log_entries(...) -> LemmyResult<i64>` |
| `read_log_payload` (~11966-11977) | `async fn read_log_payload(...) -> Result<Option<Value>, Box<dyn Error>>` | `async fn read_log_payload(...) -> LemmyResult<Option<Value>>` |
| `seed_pending_case` (~11980-12021) | `async fn seed_pending_case(...) -> Result<ModerationCaseId, Box<dyn Error>>` | `async fn seed_pending_case(...) -> LemmyResult<ModerationCaseId>` |
| `seed_active_surety` (~12024-12037) | `async fn seed_active_surety(...) -> Result<SuretyId, Box<dyn Error>>` | `async fn seed_active_surety(...) -> LemmyResult<SuretyId>` |

The helper bodies themselves should not need changes for `?` propagation — Diesel chains return errors that already implement `From<DieselError> for LemmyError` via Lemmy's error type infrastructure. If a helper body has any `Err(...)` literal returning a `Box<dyn Error>` value, replace with `LemmyError::from_error_message(e, "context")` or `e.into()` (whichever the surrounding code uses for similar errors elsewhere in v1-SL-b).

#### Change C — module imports

At the top of `mod v1_sl_c_fixtures` (around line 11952-11953), the current imports include `use std::error::Error;` (or similar `Box<dyn Error>` support imports). Replace those with:

```rust
use lemmy_utils::error::LemmyResult;
```

Mirror the v1-SL-b pattern at `crates/server/tests/e2e.rs:11001` exactly.

If the module already has `use lemmy_utils::error::LemmyResult;`, leave it. If it has `use std::error::Error;` only for `Box<dyn Error>` purposes, remove it (no longer needed). If there is a `LemmyError` import needed for explicit error conversion in helper bodies, add `use lemmy_utils::error::{LemmyError, LemmyResult};` instead.

### 2.3 What the test fn body should look like after the fix

Reference the v1-SL-b shape at lines 11157-11217:

```rust
async fn revoke_endorsement_self_succeeds_updates_surety_and_recomputes_snapshots() -> LemmyResult<()> {
    let (_container, context, db_url) = governance_fixtures::bootstrap().await?;  // bare ?
    let instance = Instance::read_or_create(&mut context.pool(), "test.invalid").await?;
    let (sponsee, _) = governance_fixtures::seed_user(&context, instance.id, "...", false).await?;
    // ... helpers: bare ? everywhere
    let outcome = run_grace_check_batch(&context).await?;
    // ... asserts
    Ok(())
}
```

The post-fix v1-SL-c-2 Task 1 test fn body should look the same — bare `?` propagation, no `.map_err` anywhere.

### 2.4 Verification (post-edit, before commit)

Two grep gates the compile must pass:

1. `grep -n ".map_err" crates/server/tests/e2e.rs | grep -E "v1_sl_c|grace_check_fires"` → **zero matches** (no bridges in the v1-SL-c-2 test fn or its helpers).
2. `grep -n "Box<dyn Error>" crates/server/tests/e2e.rs | grep -E "v1_sl_c|grace_check_fires"` → **zero matches** (no `Box<dyn Error>` in the v1-SL-c-2 module).
3. `grep -n "LemmyResult" crates/server/tests/e2e.rs | grep -E "fn count_log_entries|fn read_log_payload|fn seed_pending_case|fn seed_active_surety|fn grace_check_fires"` → **5 matches** (4 helpers + test fn signature).

### 2.5 Commit + push + DQ

After the Edits:

1. Commit on the worker branch (the advisor has hard-reset it to `c2761b284`):
   - Branch: `junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154`.
   - Subject: `test(v1-SL-c-2): replan Task 1 — uniform LemmyResult shape mirroring v1-SL-b (sl-c-2-impl-1-replan)`.
   - Body cites this brief and `.claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md`.
2. Push the worker branch.
3. Capture new workflow run id:
   ```
   gh run list --repo barrie-cork/lemmy --branch <your-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId,status,createdAt
   ```
   Wait until a row appears.
4. Write a NEW `kind: "validate-pending"` DQ entry (next id = 167), `from: "impl"`, with the captured `workflow_run_id`. Cite this brief in `context`. Do NOT include `supersedes_dq` — the supersession of #164/#165/#166 is the advisor's `docs(decision-queue):` commit, not your DQ entry.
5. Commit + push the DQ entry to the worker branch immediately (per `.claude/rules/decision-queue.md` "Mid-task visibility").

## 3. Required reading

In order:

1. **`.claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md`** — full RCA. Read this first. Without it the change feels arbitrary; with it the change is mechanical.
2. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** (amended 2026-05-09) — cases A/B/C enumeration. This brief implements **Case A**.
3. **`.claude/lessons/feedback_async_pool_test_pattern.md`** — companion lesson on AsyncPgConnection + DbPool pattern that the helpers use.
4. **`.claude/lessons/feedback_junior_worker_e2e_edit_hang.md`** — surgical Edits with ~3-line context anchors. Do NOT pass the full e2e.rs body to a single Edit (file is ~12,200 lines).
5. **The v1-SL-b canonical sibling** at `crates/server/tests/e2e.rs:11001-11924`. Specifically:
   - Line 11001 — `use lemmy_utils::error::LemmyResult;` (mirror this import).
   - Lines 11006-11128 — 7 helper signatures (mirror the 4 c-2 helper signatures on this template).
   - Lines 11157-11158 + 11159-11217 — test fn signature + body with direct `?` propagation (mirror the c-2 test fn body shape).
6. **The plan** at `.claude/PRPs/plans/v1-sponsor-liability-c-2.plan.md` §13 Task 1 (lines 1316–1508) — Task 1 invariants + DoD §15 commands. The §17 carry-forward note (added by this re-plan) explains why the original §13 stub was wrong.
7. **The original Task 1 commit** at `c2761b284` (now the worker branch tip after the advisor's hard-reset). Read the full e2e.rs body around lines 11954-12108 — this is your starting point.

## 4. Constraints

- **One file edited:** `crates/server/tests/e2e.rs`. No other files.
- **Surgical Edits only:** per `feedback_junior_worker_e2e_edit_hang.md`, do NOT pass the full e2e.rs body to a single Edit. Use 5-7 small Edits with 3-line context anchors:
  - 1 Edit for the imports change (Change C).
  - 4 Edits for the helper signatures (Change B — one per helper).
  - 1 Edit for the test fn signature (Change A).
- **Mirror v1-SL-b verbatim.** When in doubt about an import line, a helper signature shape, or an error-propagation pattern, look at the v1-SL-b sibling at lines 11001-11924 and copy its shape. The canonical-schema-first gate (per `.claude/rules/advisor-orchestrator.md` §3.6) is mandatory for any spec-shape choice.
- **Same worker branch** as impl-task #154. After the advisor's hard-reset, the branch tip is `c2761b284`. Fast-forward append your replan commit; do NOT cut a new junior worker branch.
- **No `.map_err` bridges.** This is Case A — direct `?` propagation throughout. If you find yourself wanting to add `.map_err` anywhere in this module, STOP and raise a `kind: "blocker"` DQ entry citing this section. The recipe is uniform `LemmyResult<T>`, not mixed.
- **No `Box<dyn Error>` anywhere in `mod v1_sl_c_fixtures` or in the test fn.** The grep gates §2.4 enforce this.
- **No helper body logic changes.** The 4 helpers' query bodies + return values stay the same; only the return type signature changes. The body's `?` propagations now propagate `LemmyError` instead of `Box<dyn Error>`, but the assertion semantics are identical.
- **No test-body assertion changes.** The asserts after the fix still operate on the same values — `outcome.cases_processed`, `outcome.fired`, etc.
- **Per `feedback_features_full_p_crate_incompatible.md`**: cargo validation uses `--workspace --features full`, NOT `-p lemmy_server --features full`. The validate-pending DQ entry must reference the `cargo-validate-workspace.yml` workflow.
- **Per `feedback_pipes_mask_exit_codes.md`**: `gh run list` capture uses `--json databaseId`, never `... | grep`.

### Validate-pending DQ shape

```json
{
  "id": 167,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO 8601 UTC at write time>",
  "question": "Does cargo-validate-workspace pass on sl-c-2 Task 1 replan (uniform LemmyResult<T> mirroring v1-SL-b)?",
  "options": [
    "(A) pass — advance to Task 2",
    "(B) fail — advisor §G4 triage (cycle 4 — escalate)"
  ],
  "context": "Pushed test(v1-SL-c-2): replan Task 1 — uniform LemmyResult shape mirroring v1-SL-b (sl-c-2-impl-1-replan). Replan after 3 §G4 cycles failed on error-shape mismatch (RCA: .claude/PRPs/debug/rca-sl-c-2-3-cycle-error-shape.md). Test fn + 4 helpers all return LemmyResult<T>; mirrors v1-SL-b mod v1_sl_b_fixtures verbatim (e2e.rs:11001-11924). Canonical-shape parity with c-1 sibling.",
  "branch": "junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154",
  "phase_task": "sl-c-2-impl-1-replan",
  "workflow_run_id": <captured from gh run list>,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null
}
```

Commit subject for the DQ entry: `chore(decision-queue): impl raised DQ #167 — sl-c-2 task-1 replan validate-pending`.

### Hard refusals

- Do NOT add any `.map_err` to the v1-SL-c-2 test fn or its helpers. Case A is uniform `LemmyResult<T>` with bare `?` everywhere.
- Do NOT add any `Box<dyn Error>` to the v1-SL-c-2 module.
- Do NOT touch any other test fn or mod in e2e.rs (especially v1-SL-b's `mod v1_sl_b_fixtures` — that is the canonical reference; touching it breaks the mirror).
- Do NOT modify helper body query logic — only the return type signature.
- Do NOT cut a new junior worker branch — fast-forward on the advisor's hard-reset of branch md-154.
- Do NOT mutate DQ #164, #165, or #166 — supersession is the advisor's commit, not your DQ entry.
- Do NOT run cargo locally on the EliteDesk worker (per `feedback_pipes_mask_exit_codes.md` + the "Cargo never runs on the EliteDesk worker" rule in advisor-orchestrator.md).
- Do NOT modify `.claude/**` from this Junior task — those are advisor-side artefacts.

## 5. Acceptance

- Single replan commit on worker branch with subject `test(v1-SL-c-2): replan Task 1 — uniform LemmyResult shape mirroring v1-SL-b (sl-c-2-impl-1-replan)`.
- Diff: 5-7 small Edits in `crates/server/tests/e2e.rs` only.
- Net LOC: ~6 lines changed (4 helper signatures + 1 test fn signature + 1-2 import lines). The 6 `.map_err` bridges from cycle 3 are not present (the hard-reset removed them; the replan does not re-add them).
- Worker branch pushed to origin.
- New DQ #167 (`kind: "validate-pending"`) written + committed + pushed to the same worker branch.
- DQ #164, #165, #166 untouched (advisor will supersede separately).
- Grep gates §2.4 pass.

## 6. Why this brief is structured this way (process note)

The fix-impl-1 brief inverted the §G4 row text → cycle 2 catch-fire. The fix-impl-2 brief copied the §G4 row text verbatim (post anti-paraphrase gate) but the row text was insufficient → cycle 3 catch-fire. The conclusion: the §G4 mechanical-recipe family is too narrow for this code shape, and a re-plan with case enumeration is the right scope.

This brief is therefore NOT a §G4 fix-impl brief — it is a Task 1 replan dispatched after the advisor's `/auto-phase` skill caught fire and surfaced for re-plan. The brief explicitly cites the canonical sibling (v1-SL-b at e2e.rs:11001-11924), the RCA (rca-sl-c-2-3-cycle-error-shape.md), and the amended lesson (case A applies). Verbatim canonical-sibling mirror replaces the §G4 verbatim-row anti-paraphrase gate as the structural correctness check.

If you (the impl-task subagent) detect this brief asks you to do something other than mirror v1-SL-b's uniform `LemmyResult<T>` shape, **STOP and raise a `kind: "blocker"` DQ entry** citing this section + the v1-SL-b reference module.
