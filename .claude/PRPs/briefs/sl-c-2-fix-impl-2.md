---
role: impl-task
plan_task: 1-fix-2
phase: v1-SL-c-2
created: 2026-05-09
related_dq: 165
supersedes_brief: sl-c-2-fix-impl-1.md
---

# Brief — v1-SL-c-2 fix-impl-2 — apply §G4 canonical recipe (E0277 LemmyError ↔ Box<dyn Error>)

## 1. Role + dispatch line

`[role:impl-task] sl-c-2-fix-impl-2 — see .claude/PRPs/briefs/sl-c-2-fix-impl-2.md`

You are the **impl-task** subagent (Sonnet 4.6). Apply a small,
mechanical compile fix per the §G4 canonical recipe. ≤2 file edits
(1 file: `crates/server/tests/e2e.rs`), ≤8 line additions.

## 2. Scope

### 2.1 §G4 CANONICAL RECIPE (verbatim from `.claude/rules/advisor-orchestrator.md:435`)

> | Failure signature | Auto-fix | Source lesson |
> | `error[E0277]: ?` couldn't convert `LemmyError` (or `LemmyResult<T>`) to `Box<dyn Error>` at a `?` propagation site | wrap the call with `.map_err(\|e\| format!("{e}").into())` per the lesson; verify the test fn signature is `Result<(), Box<dyn Error>>` | `feedback_lemmy_error_no_std_error.md` |

**This recipe is canonical and load-bearing.** Two halves, both required:

1. **Test fn signature MUST be `Result<(), Box<dyn Error>>`** — NOT `LemmyResult<()>`.
2. **Each Lemmy-native call site MUST get `.map_err(|e| format!("{e}").into())`** before its `?`.

Per `feedback_lemmy_error_no_std_error.md`:

> `LemmyError` does not implement `std::error::Error`, so `LemmyResult<T>`
> cannot use `?` in test functions that return `Result<(), Box<dyn Error>>`.
> Mechanical fix: `.map_err(|e| format!("{e}").into())` on every Lemmy-native
> call.

### 2.2 The bug — full history

**Cycle 1 (DQ #164, original Task 1 commit `c2761b284`):** test fn body
calls `governance_fixtures::bootstrap()` (returns `LemmyResult`) at
e2e.rs:12049 with `?` against signature `Result<(), Box<dyn Error>>`.
E0277 because `Box<dyn Error>: From<LemmyError>` is missing. **Canonical
recipe was not applied.**

**Cycle 2 (DQ #165, fix-impl-1 commit `4be7b7f95`):** the fix-impl-1
brief incorrectly prescribed flipping the test fn signature to
`LemmyResult<()>` and explicitly forbade `.map_err`. Junior #158 followed
the brief literally. The flip broke 8 helper-call sites at e2e.rs:12069,
12070, 12071, 12106, 12111, 12116, 12121, 12122 (helpers return
`Result<X, Box<dyn Error>>`; `dyn Error` lacks Send/Sync/Sized bounds
required for `LemmyError`'s `Send + Sync + 'static`). Workflow
`25595869651` failed with 11 × E0277 (Send/Sync/Sized).

**Cycle 2 root cause** (per `.claude/PRPs/debug/rca-sl-c-2-fix-impl-1-wrong-recipe.md`):
**brief-authorship lapse**, not a Junior worker lapse. The fix-impl-1
brief contradicted the §G4 canonical recipe loaded into the advisor's
session at session start. This brief (fix-impl-2) corrects that.

### 2.3 The fix

On the **same worker branch** as impl-1 + fix-impl-1 (so daemon
finalize-merge picks up the chain as one unit), apply two changes to
`crates/server/tests/e2e.rs`:

**Change A — revert the signature flip** (line 12042 currently):

```rust
  ) -> lemmy_utils::error::LemmyResult<()> {
```

→

```rust
  ) -> Result<(), Box<dyn Error>> {
```

**Change B — add `.map_err(|e| format!("{e}").into())` bridges** at the 5
Lemmy-native call sites. Verbatim diff target (line numbers per worker
tip `3fdb666a4`):

| Line | Current | Replace with |
|---|---|---|
| 12049 | `let (_container, context, db_url) = governance_fixtures::bootstrap().await?;` | `let (_container, context, db_url) = governance_fixtures::bootstrap().await.map_err(\|e\| format!("{e}").into())?;` |
| 12050-12054 | The `Instance::read_or_create(...).await?` block — keep `.await` chain, add `.map_err(\|e\| format!("{e}").into())` before final `?` | `let instance = lemmy_db_schema::source::instance::Instance::read_or_create(&mut context.pool(), "test.invalid").await.map_err(\|e\| format!("{e}").into())?;` |
| 12055-12056 | `let (sponsee, _) = governance_fixtures::seed_user(&context, instance.id, "slc1_sponsee", false).await?;` | `let (sponsee, _) = governance_fixtures::seed_user(&context, instance.id, "slc1_sponsee", false).await.map_err(\|e\| format!("{e}").into())?;` |
| 12057-12058 | `let (sponsor1, _) = governance_fixtures::seed_user(&context, instance.id, "slc1_sponsor1", false).await?;` | same `.map_err` bridge |
| 12059-12060 | `let (sponsor2, _) = governance_fixtures::seed_user(&context, instance.id, "slc1_sponsor2", false).await?;` | same `.map_err` bridge |
| ~12075 | `let outcome = run_grace_check_batch(&context).await?;` | `let outcome = run_grace_check_batch(&context).await.map_err(\|e\| format!("{e}").into())?;` |

**DO NOT modify** the helpers `seed_pending_case`, `seed_active_surety`,
`count_log_entries`, `read_log_payload` — they return
`Result<_, Box<dyn Error>>` which `?`-propagates cleanly into
`Result<(), Box<dyn Error>>` (no bridge needed).

**DO NOT modify** the Diesel chains (`AsyncPgConnection::establish`,
`.first(...).await?`, `.get_result(...).await?`, etc) — Diesel errors
implement `std::error::Error` natively, so they `?`-propagate cleanly.

### 2.4 Verification (post-edit, before commit)

`grep -n "LemmyResult<()>" crates/server/tests/e2e.rs | grep -F "grace_check_fires"` should return **zero matches** (signature reverted).

`grep -nF ".map_err(|e| format!(\"{e}\").into())" crates/server/tests/e2e.rs | grep -E "12049|12054|12056|12058|12060|12075"` should show ~6 hits (5 bridges across 6 lines).

### 2.5 Commit + push + DQ

After the Edit:

1. Commit on the **same worker branch** as impl-1 + fix-impl-1
   (`junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154`).
   Subject:
   `fix(v1-SL-c-2): apply §G4 canonical recipe — .map_err bridges at Lemmy-native call sites (fix-impl-2)`
2. Push the worker branch.
3. Capture new workflow run id:
   ```
   gh run list --repo barrie-cork/lemmy --branch <your-branch> --workflow cargo-validate-workspace --limit 1 --json databaseId,status,createdAt
   ```
   Wait until a row appears with `status: "queued"` or later (the push triggers the workflow on the next ~30 sec).
4. Write a NEW `kind: "validate-pending"` DQ entry (id = **166**), `from: "impl"`, with the captured `workflow_run_id`. Same shape as DQ #165 (which stays in `pending[]` as the failing record from cycle 2).
5. Commit + push the DQ entry to the worker branch immediately (per `.claude/rules/decision-queue.md` "Mid-task visibility" — without push, the advisor's polling loop won't see it).

**DO NOT** mutate DQ #164 or DQ #165 — both stay in `pending[]` as the
failing records of their respective cycles. The new entry (id 166)
describes the post-fix-impl-2 workflow run.

## 3. Required reading

In order:

1. **`.claude/rules/advisor-orchestrator.md` lines 420-448** — the §G4
   classifier table (the canonical recipe is row 4).
2. **`.claude/lessons/feedback_lemmy_error_no_std_error.md`** — the
   underlying lesson. Read in full (it is short).
3. **The failing test fn body at `crates/server/tests/e2e.rs:12040-12130`**
   on worker branch tip `3fdb666a4` — read the full body before editing
   so you can identify the 5 Lemmy-native call sites + the 4 helper-fn
   call sites + the Diesel chains.
4. **`.claude/PRPs/debug/rca-sl-c-2-fix-impl-1-wrong-recipe.md`** — full
   RCA explaining why the previous fix was wrong and why this one
   matches the canonical recipe.
5. **Sibling pattern at `crates/server/tests/e2e.rs:8009-8030`** —
   `admin_assign_jury_severity_tier_regular_minor_panel_5_jurors`. The
   sibling returns `LemmyResult<()>` because **every call in its body
   returns `LemmyResult<...>`** — except `v1_jm_b_fixtures::seed_case`
   (line 8030, returns `Box<dyn Error>`), which is bridged with
   `.map_err(|e| anyhow::anyhow!(...))?`. Our test fn has the **inverse
   shape**: most calls are Lemmy-native, but the helpers stay
   `Box<dyn Error>` — so we keep the test fn as `Box<dyn Error>` and
   bridge the Lemmy calls. Same pattern, mirror direction.
6. **`.claude/lessons/feedback_features_full_p_crate_incompatible.md`** —
   bound on validate-pending DQ. Workflow uses `--workspace --features
   full`, not `-p lemmy_server --features full`.
7. **`.claude/lessons/feedback_pipes_mask_exit_codes.md`** — bound on
   any `gh run list` capture. Use `--json databaseId` (not `... | grep`).

## 4. Constraints

- **One Edit on `crates/server/tests/e2e.rs`** (or up to 6 small Edits if
  you prefer one Edit per call site for clarity — both shapes accepted,
  but no Edits to other files).
- **No use-block changes.** The `Result<(), Box<dyn Error>>` shape works
  with the existing `use std::error::Error;` (already in scope per
  e2e.rs preamble; verify by grep before editing).
- **No helper changes.** The 4 helpers (`seed_pending_case`,
  `seed_active_surety`, `count_log_entries`, `read_log_payload`) stay
  exactly as the original Task 1 commit defined them.
- **No test-body assertion changes.** The asserts after `.map_err`
  still operate on the same values — `outcome.cases_processed`,
  `outcome.fired`, etc — because `.map_err` only transforms the error
  arm, not the success arm.
- **No other files.** Only `crates/server/tests/e2e.rs`.
- **Commit on the SAME worker branch** as impl-task #154
  (`junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154`).
  Fast-forward append. Do NOT cut a new junior worker branch.
- **Per `feedback_junior_worker_e2e_edit_hang.md`**: do NOT pass the
  full e2e.rs body to a single Edit. Use surgical Edits with ~3-line
  context anchors. The file is ~12,200 lines; whole-file Edit hangs
  the worker.

### Validate-pending DQ shape

```json
{
  "id": 166,
  "from": "impl",
  "kind": "validate-pending",
  "timestamp": "<ISO 8601 UTC at write time>",
  "question": "Does cargo-validate-workspace pass on sl-c-2 fix-impl-2 (§G4 canonical recipe applied — .map_err bridges)?",
  "options": [
    "(A) pass — advance to Task 2",
    "(B) fail — advisor §G4 triage (cycle 3 — escalate to user)"
  ],
  "context": "Pushed fix(v1-SL-c-2): apply §G4 canonical recipe — .map_err bridges at Lemmy-native call sites (fix-impl-2). Reverts test fn signature to Result<(), Box<dyn Error>> and adds .map_err bridges at 5 Lemmy-native call sites per the §G4 row 4 recipe. Closes E0277 from DQ #164 + DQ #165.",
  "branch": "junior/role-impl-task-sl-c-2-impl-1-see-claude-prps-briefs-sl-c-2-impl-1-md-154",
  "phase_task": "sl-c-2-fix-impl-2",
  "workflow_run_id": <captured from gh run list>,
  "result": null,
  "log_slice": null,
  "failed_jobs": null,
  "answer": null,
  "answered_by": null,
  "supersedes_dq": [165]
}
```

Commit subject for the DQ entry: `chore(decision-queue): impl raised
DQ #166 — sl-c-2 fix-impl-2 validate-pending`.

### Hard refusals

- Do NOT change the test fn signature to `LemmyResult<()>` again. The
  signature MUST be `Result<(), Box<dyn Error>>`. This is the §G4 recipe
  half (1) of (2).
- Do NOT skip `.map_err(|e| format!("{e}").into())` at any of the 5
  Lemmy-native call sites. This is the §G4 recipe half (2) of (2).
- Do NOT touch the helpers (`seed_pending_case`, `seed_active_surety`,
  `count_log_entries`, `read_log_payload`).
- Do NOT touch any other test fn or mod in e2e.rs.
- Do NOT add a new `use` import — the existing `use std::error::Error;`
  is already in scope in e2e.rs preamble.
- Do NOT mutate DQ #164 or DQ #165 (both stay in `pending[]` as the
  failing records).
- Do NOT run cargo locally.
- Do NOT modify `.claude/**`.
- Do NOT modify `crates/api/governance/sponsor_liability_grace.rs` or
  any non-test crate.

## 5. Acceptance

- Single fix-impl-2 commit on worker branch with subject
  `fix(v1-SL-c-2): apply §G4 canonical recipe — .map_err bridges at Lemmy-native call sites (fix-impl-2)`.
- Diff: ~7 line changes in `crates/server/tests/e2e.rs` only — 1
  signature reversion + 5 `.map_err` bridge additions (or 6, depending
  on whether you bridge the `Instance::read_or_create` block as one
  unit or two).
- Worker branch pushed to origin.
- New DQ #166 (`kind: "validate-pending"`) written + committed +
  pushed to the same worker branch.
- DQ #164 and DQ #165 untouched in `pending[]`.

## 6. Why this brief is structured this way (process note)

The fix-impl-1 brief (`.claude/PRPs/briefs/sl-c-2-fix-impl-1.md`) cited
the canonical lesson but prescribed the inverse recipe. Per the RCA
recommendation in `.claude/PRPs/debug/rca-sl-c-2-fix-impl-1-wrong-recipe.md`
"Notes for retro" §1 ("Brief authorship lapse on mechanical recipe"),
this brief copy-pastes the §G4 row text VERBATIM into Scope §2.1 before
adding file:line context. This makes "I read the row but prescribed
something different" structurally impossible.

If you (the impl-task subagent) detect the brief is asking you to do
something other than the verbatim §G4 recipe, **STOP and raise a
`kind: "blocker"` DQ entry** citing this section + the §G4 row.
